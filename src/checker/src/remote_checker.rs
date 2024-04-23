use async_trait::async_trait;
use proto::fgars_service_client::FgarsServiceClient;
use proto::{CheckRequest as ProtoCheckRequest, TupleKey};
use tonic::transport::{Channel, Endpoint};
use tower::discover::Change;

use crate::graph::ResolutionMetadata;
use crate::{CheckRequest, CheckResult, Checker};

pub struct RemoteChecker {
    client: FgarsServiceClient<Channel>,
}

impl RemoteChecker {
    pub async fn new(addr: String) -> Self {
        // TODO load balance, add crate
        let (channel, sender) = Channel::balance_channel(1024);
        sender
            .send(Change::Insert(addr.clone(), Endpoint::from_shared(addr).unwrap()))
            .await
            .unwrap();

        let client = FgarsServiceClient::new(channel);
        Self { client }
    }
}

#[async_trait]
impl Checker for RemoteChecker {
    async fn check(&self, req: CheckRequest) -> anyhow::Result<CheckResult> {
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
        let mut client = self.client.clone();
        let reply = client
            .check(ProtoCheckRequest {
                tenant_id: req.tenant_id,
                model_id: Some(req.model_id),
                tuple_key: Some(TupleKey {
                    user_type: req.tuple_key.user_type,
                    user_id: req.tuple_key.user_id,
                    user_relation: req.tuple_key.user_relation,
                    relation: req.tuple_key.relation,
                    object_type: req.tuple_key.object_type,
                    object_id: req.tuple_key.object_id,
                }),
                contextual_tuples,
            })
            .await?;
        let result = reply.into_inner();
        let resolution_metadata = result
            .resolution_metadata
            .map(|rm| ResolutionMetadata {
                depth: rm.depth,
                datastore_query_count: rm.datastore_query_count,
            })
            .unwrap_or_default();
        Ok(CheckResult {
            allow: result.allow,
            resolution_metadata,
        })
    }

    async fn close(&self) {}

    fn name(&self) -> &str {
        "remote"
    }
}
