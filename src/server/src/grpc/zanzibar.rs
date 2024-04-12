use checker::CheckRequest as InnerCheckRequest;
use checker::CheckerRef;
use proto::ResolutionMetadata;
use proto::{fgars_service_server::FgarsService, CheckReply, CheckRequest};
use protocol::TupleKey;
use storage::AuthzModelReaderRef;
use tonic::{Request, Response, Status};
use tracing::Instrument;

pub struct Service {
    pub checker: CheckerRef,
    pub model_reader: AuthzModelReaderRef,
}

#[tonic::async_trait]
impl FgarsService for Service {
    async fn check(&self, request: Request<CheckRequest>) -> Result<Response<CheckReply>, Status> {
        let req = request.into_inner();
        let (id, model) = if let Some(model_id) = req.model_id {
            self.model_reader
                .clone()
                .get(String::from(&req.tenant_id), model_id)
                .await
                .map_err(|err| Status::internal(err.to_string()))?
        } else {
            self.model_reader
                .clone()
                .get_latest(String::from(&req.tenant_id))
                .await
                .map_err(|err| Status::internal(err.to_string()))?
        };
        let span = trace_span!("check");

        let tuple_key = req.tuple_key.unwrap();

        let contextual_tuples = req
            .contextual_tuples
            .into_iter()
            .map(|ct| TupleKey {
                user_type: ct.user_type,
                user_id: ct.user_id,
                user_relation: ct.user_relation,
                relation: ct.relation,
                object_type: ct.object_type,
                object_id: ct.object_id,
            })
            .collect();

        let cr = InnerCheckRequest {
            tenant_id: req.tenant_id,
            model_id: id,
            tuple_key: TupleKey {
                user_type: tuple_key.user_type,
                user_id: tuple_key.user_id,
                user_relation: tuple_key.user_relation,
                relation: tuple_key.relation,
                object_type: tuple_key.object_type,
                object_id: tuple_key.object_id,
            },
            contextual_tuples,
            typesystem: model.to_typesystem(),
            ..Default::default()
        };
        let result = self
            .checker
            .clone()
            .check(cr)
            .instrument(span)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(CheckReply {
            allow: result.allow,
            resolution_metadata: Some(ResolutionMetadata {
                depth: result.resolution_metadata.depth,
                datastore_query_count: result.resolution_metadata.datastore_query_count,
            }),
        }))
    }
}
