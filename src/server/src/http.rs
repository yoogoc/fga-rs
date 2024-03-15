mod authz_model;
mod tenant;
mod zanzibar;

use aide::{
    axum::{ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi, Server as OpenAPIServer},
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
use axum::{
    routing::{delete, get, post},
    Extension, Json, Router,
};
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
                title: "FGA-rs HTTP API".to_string(),
                description: Some("FGA-rs HTTP API".to_string()),
                version: "v1".to_string(),
                ..Info::default()
            },
            servers: vec![OpenAPIServer {
                url: "/api/v1".to_string(),
                ..OpenAPIServer::default()
            }],
            ..OpenApi::default()
        };

        let zanzibar_route = ApiRouter::new()
            .route("/read", get(zanzibar::read).with_state(self.tuple_reader.clone()))
            .route(
                "/save",
                post(zanzibar::write_save).with_state(self.tuple_writer.clone()),
            )
            .route(
                "/delete",
                post(zanzibar::write_delete).with_state(self.tuple_writer.clone()),
            )
            .route("/check", post(zanzibar::check_x).with_state(self.checker.clone()))
            .route("/expand", get(zanzibar::expand).with_state(self.tuple_reader.clone()))
            .finish_api(&mut api)
            .layer(Extension(api.clone()));

        let authz_model_route = ApiRouter::new()
            .route(
                "/",
                post(authz_model::create).with_state(self.authz_model_writer.clone()),
            )
            .route("/", get(authz_model::list).with_state(self.authz_model_reader.clone()))
            .route(
                "/:id",
                get(authz_model::get).with_state(self.authz_model_reader.clone()),
            )
            .finish_api(&mut api)
            .layer(Extension(api.clone()));
        let tenant_route = ApiRouter::new()
            .route("/", get(tenant::list))
            .route("/", post(tenant::create))
            .route("/:id", get(tenant::get))
            .route("/:id", delete(tenant::delete))
            .with_state(self.tenant_operator.clone())
            .finish_api(&mut api)
            .layer(Extension(api.clone()));

        Router::new()
            .nest("/api/v1/zanzibar/:tenant_id", zanzibar_route)
            .nest("/api/v1/authz_models/:tenant_id", authz_model_route)
            .nest("/api/v1/tenants", tenant_route)
            .route("/api/v1/api.json", get(serve_api))
            .layer(TraceLayer::new_for_http())
            .layer(Extension(api.clone()))
    }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
