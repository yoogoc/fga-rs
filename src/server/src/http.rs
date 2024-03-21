mod authz_model;
mod tenant;
mod zanzibar;

use aide::{
    axum::{routing as apirouting, ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi},
    redoc::Redoc,
    scalar::Scalar,
};
use std::{net::SocketAddr, time::Duration};
use tower_http::trace::TraceLayer;

use checker::CheckerRef;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use storage::{
    AuthzModelReaderRef, AuthzModelWriterRef, RelationshipTupleReaderRef, RelationshipTupleWriterRef, TenantOperatorRef,
};
use tokio::sync::oneshot::{self, Sender};

use crate::{error::ServerError, Server};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use axum::{extract::MatchedPath, http::Request, routing::get, Extension, Json, Router};
use tokio::sync::Mutex;

pub struct HttpServer {
    tuple_reader: RelationshipTupleReaderRef,
    tuple_writer: RelationshipTupleWriterRef,
    authz_model_reader: AuthzModelReaderRef,
    authz_model_writer: AuthzModelWriterRef,
    tenant_operator: TenantOperatorRef,
    checker: CheckerRef,
    shutdown_tx: Mutex<Option<Sender<()>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpOptions {
    pub addr: String,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:4000".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl HttpServer {
    pub fn new(
        tuple_reader: RelationshipTupleReaderRef,
        tuple_writer: RelationshipTupleWriterRef,
        authz_model_reader: AuthzModelReaderRef,
        authz_model_writer: AuthzModelWriterRef,
        tenant_operator: TenantOperatorRef,
        checker: CheckerRef,
    ) -> Self {
        Self {
            tuple_reader,
            tuple_writer,
            authz_model_reader,
            authz_model_writer,
            tenant_operator,
            checker,
            shutdown_tx: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Server for HttpServer {
    async fn shutdown(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_tx.take() {
            if tx.send(()).is_err() {
                // info!("Receiver dropped, the HTTP server has already existed");
            }
        }

        Ok(())
    }
    async fn start(&self, listening: SocketAddr) -> Result<SocketAddr> {
        let (tx, rx) = oneshot::channel();
        let server = {
            let mut shutdown_tx = self.shutdown_tx.lock().await;
            ensure!(
                shutdown_tx.is_none(),
                ServerError::AlreadyStarted { server: "HTTP".into() }
            );

            let router = self.make_router();

            let listener = tokio::net::TcpListener::bind(listening).await.unwrap();

            let server = axum::serve(listener, router.into_make_service()).with_graceful_shutdown(rx.map(drop));

            *shutdown_tx = Some(tx);

            server
        };

        let _ = server.await?;

        Ok(listening)
    }
    fn name(&self) -> &str {
        "http"
    }
}

impl HttpServer {
    fn make_router(&self) -> Router {
        let mut api = OpenApi {
            info: Info {
                title: "fga-rs HTTP API".to_string(),
                description: Some("fga-rs HTTP API".to_string()),
                version: "v1".to_string(),
                ..Info::default()
            },
            ..OpenApi::default()
        };

        let zanzibar_route = ApiRouter::new()
            .api_route(
                "/zanzibar/:tenant_id/read",
                apirouting::get(zanzibar::read).with_state(self.tuple_reader.clone()),
            )
            .api_route(
                "/zanzibar/:tenant_id/save",
                apirouting::post(zanzibar::write_save).with_state(self.tuple_writer.clone()),
            )
            .api_route(
                "/zanzibar/:tenant_id/delete",
                apirouting::post(zanzibar::write_delete).with_state(self.tuple_writer.clone()),
            )
            .api_route(
                "/zanzibar/:tenant_id/check",
                apirouting::post(zanzibar::check_x).with_state((self.checker.clone(), self.authz_model_reader.clone())),
            )
            .api_route(
                "/zanzibar/:tenant_id/expand",
                apirouting::get(zanzibar::expand).with_state(self.tuple_reader.clone()),
            );

        let authz_model_route = ApiRouter::new()
            .api_route(
                "/authz_models/:tenant_id",
                apirouting::post(authz_model::create).with_state(self.authz_model_writer.clone()),
            )
            .api_route(
                "/authz_models/:tenant_id/dsl",
                apirouting::post(authz_model::create_by_dsl).with_state(self.authz_model_writer.clone()),
            )
            .api_route(
                "/authz_models/:tenant_id",
                apirouting::get(authz_model::list).with_state(self.authz_model_reader.clone()),
            )
            .api_route(
                "/authz_models/:tenant_id/:id",
                apirouting::get(authz_model::get).with_state(self.authz_model_reader.clone()),
            );
        let tenant_route = ApiRouter::new()
            .api_route("/tenants", apirouting::get(tenant::list))
            .api_route("/tenants", apirouting::post(tenant::create))
            .api_route("/tenants/:id", apirouting::get(tenant::get))
            .api_route("/tenants/:id", apirouting::delete(tenant::delete))
            .with_state(self.tenant_operator.clone());

        ApiRouter::new()
            .nest("/api/v1", zanzibar_route)
            .nest("/api/v1", authz_model_route)
            .nest("/api/v1", tenant_route)
            .route("/api/v1/api.json", get(serve_api))
            .route(
                "/api/v1/redoc",
                get(Redoc::new("/api/v1/api.json").with_title("fga-rs").axum_handler()),
            )
            .route(
                "/api/v1/scalar",
                get(Scalar::new("/api/v1/api.json").with_title("fga-rs").axum_handler()),
            )
            .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request.extensions().get::<MatchedPath>().map(MatchedPath::as_str);
                trace_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }))
            .finish_api(&mut api)
            .layer(Extension(api.clone()))
    }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
