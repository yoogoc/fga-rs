use anyhow::Result;
use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt};
use protocol::{RelationReference, SetOperator, TupleKey, TupleToUserset, Userset, WILDCARD};
use storage::{RelationshipTupleReaderRef, TupleFilter};
use tracing::Instrument;

use crate::{
    error::CheckerError, exclusion_check, graph::ResolutionMetadata, intersection_check, union_check, CheckRequest,
    CheckResult, Checker, CheckerRef,
};

pub struct LocalChecker {
    resolver: Option<CheckerRef>,
    tuple_reader: RelationshipTupleReaderRef,
}

#[async_trait]
impl Checker for LocalChecker {
    async fn check(&self, req: CheckRequest) -> Result<CheckResult> {
        let span = info_span!("local-checker");
        let _enter = span.enter();
        trace!("tuple is {}, model id is {}", &req.tuple_key, &req.model_id);
        let relation = req
            .typesystem
            .get_relation(&req.tuple_key.object_type, &req.tuple_key.relation)?;

        self.check_rewrite(&req, &relation.rewrite)
            .instrument(span.clone())
            .await
    }

    async fn close(&self) {}

    fn name(&self) -> &str {
        "local"
    }
}

impl LocalChecker {
    pub fn new(resolver: Option<CheckerRef>, tuple_reader: RelationshipTupleReaderRef) -> Self {
        Self { resolver, tuple_reader }
    }

    async fn check_rewrite(&self, req: &CheckRequest, rewrite: &Userset) -> Result<CheckResult> {
        match rewrite {
            Userset::This => self.check_direct(req).await,
            Userset::Computed(or) => self.check_computed(req, &or.relation).await,
            Userset::TupleTo(ttu) => self.check_tuple_to(req, ttu).await,
            Userset::Union { children } => self.check_set_operation(req, SetOperator::Union, children).await,
            Userset::Intersection { children } => {
                self.check_set_operation(req, SetOperator::Intersection, children).await
            }
            Userset::Difference { base, subtract } => {
                let children = vec![base.to_owned(), subtract.to_owned()];
                self.check_set_operation(req, SetOperator::Intersection, &children)
                    .await
            }
        }
    }
    async fn check_direct(&self, req: &CheckRequest) -> Result<CheckResult> {
        let span = info_span!("direct-check");
        let _enter = span.enter();
        trace!("tuple request: {}", &req.tuple_key);
        let related_usersets = req
            .typesystem
            .get_directly_related_types(&req.tuple_key.object_type, &req.tuple_key.relation)?;

        trace!("related_usersets: {:?}", &related_usersets);

        if related_usersets.is_empty() {
            return Err(CheckerError::NotFoundThisTypes {
                object_type: String::from(&req.tuple_key.object_type),
                relation: String::from(&req.tuple_key.relation),
            }
            .into());
        }
        let mut filter = TupleFilter {
            object_type_eq: Some(String::from(&req.tuple_key.object_type)),
            object_id_eq: Some(String::from(&req.tuple_key.object_id)),
            relation_eq: Some(String::from(&req.tuple_key.relation)),
            ..Default::default()
        };
        let or_filter: Vec<TupleFilter> = related_usersets
            .iter()
            .filter_map(|ru| match ru {
                RelationReference::Direct(typ) => {
                    if typ.eq(&req.tuple_key.user_type) {
                        Some(TupleFilter {
                            user_type_eq: Some(String::from(&req.tuple_key.user_type)),
                            user_id_eq: Some(String::from(&req.tuple_key.user_id)),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                }
                RelationReference::Relation { r#type, relation } => {
                    if !r#type.eq(&req.tuple_key.user_type) {
                        Some(TupleFilter {
                            user_type_eq: Some(String::from(r#type)),
                            user_relation_eq: Some(String::from(relation)),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                }
                RelationReference::Wildcard(typ) => {
                    if typ.eq(&req.tuple_key.user_type) {
                        Some(TupleFilter {
                            user_type_eq: Some(String::from(&req.tuple_key.user_type)),
                            user_id_eq: Some(String::from(&req.tuple_key.user_id)),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                }
            })
            .collect();
        if !or_filter.is_empty() {
            filter.or = Some(or_filter);
        }

        let (tuples, _) = self
            .tuple_reader
            .clone()
            .list(&req.tenant_id, filter, None)
            .instrument(span.clone())
            .await?;

        let _enter = span.enter();
        trace!("tuples: {:?}", tuples);

        if tuples.is_empty() {
            return Ok(CheckResult::new_dqc(
                false,
                req.resolution_metadata.datastore_query_count + 1,
            ));
        }

        // TODO concurrence
        let direct_asserts: Vec<bool> = tuples
            .iter()
            .filter_map(|t| {
                if !t.user_type.eq(&req.tuple_key.user_type) || !t.object_type.eq(&req.tuple_key.object_type) {
                    return None;
                }
                let allow = t.user_id.eq(WILDCARD) || t.user_id.eq(&req.tuple_key.user_id);
                if let Some(relation) = &t.user_relation {
                    Some(relation.eq(&req.tuple_key.user_relation) && allow)
                } else {
                    Some(allow)
                }
            })
            .collect();

        if !direct_asserts.is_empty() {
            let allow = direct_asserts.iter().any(|x| x.to_owned());
            trace!("direct_asserts present, allow is {}", allow);
            return Ok(CheckResult::new_dqc(
                allow,
                req.resolution_metadata.datastore_query_count + 1,
            ));
        }
        let handlers: Vec<_> = tuples
            .iter()
            .filter(|t| !(t.user_type.eq(&req.tuple_key.user_type) || matches!(t.user_relation, None)))
            .map(move |t| CheckRequest {
                tenant_id: req.tenant_id.to_owned(),
                model_id: req.model_id.to_owned(),
                typesystem: req.typesystem.clone(),
                tuple_key: TupleKey {
                    user_type: String::from(&req.tuple_key.user_type),
                    user_id: String::from(&req.tuple_key.user_id),
                    user_relation: String::from(&req.tuple_key.user_relation),
                    relation: String::from(t.user_relation.as_ref().unwrap()),
                    object_type: String::from(&t.user_type),
                    object_id: String::from(&t.user_id),
                },
                contextual_tuples: req.contextual_tuples.clone(),
                resolution_metadata: ResolutionMetadata {
                    depth: req.resolution_metadata.depth,
                    datastore_query_count: req.resolution_metadata.datastore_query_count,
                },
                visited_paths: req.visited_paths.clone(),
            })
            .collect();

        trace!(
            "collect possible requests, tuples: {:?}",
            handlers.iter().map(|x| &x.tuple_key).collect::<Vec<_>>()
        );

        if let Some(r) = self.resolver.clone() {
            let r = r.clone();
            trace!("use {} checker", r.name());
            union_check(handlers.len(), |i| r.check(handlers.get(i).unwrap().to_owned()))
                .instrument(span.clone())
                .await
        } else {
            trace!("use {} checker", self.name());
            union_check(handlers.len(), |i| self.check(handlers.get(i).unwrap().to_owned()))
                .instrument(span.clone())
                .await
        }
    }

    async fn check_computed(&self, req: &CheckRequest, relation: &str) -> Result<CheckResult> {
        let span = info_span!("computed-check");
        let _enter = span.enter();
        let check_request = CheckRequest {
            tenant_id: req.tenant_id.to_owned(),
            model_id: req.model_id.to_owned(),
            typesystem: req.typesystem.clone(),
            tuple_key: TupleKey {
                user_type: String::from(&req.tuple_key.user_type),
                user_id: String::from(&req.tuple_key.user_id),
                user_relation: String::from(&req.tuple_key.user_relation),
                relation: String::from(relation),
                object_type: String::from(&req.tuple_key.object_type),
                object_id: String::from(&req.tuple_key.object_id),
            },
            contextual_tuples: req.contextual_tuples.clone(),
            resolution_metadata: ResolutionMetadata {
                depth: req.resolution_metadata.depth,
                datastore_query_count: req.resolution_metadata.datastore_query_count,
            },
            visited_paths: req.visited_paths.clone(),
        };
        if let Some(r) = self.resolver.clone() {
            trace!("use {} checker", r.name());
            r.check(check_request).instrument(span.clone()).await
        } else {
            trace!("use {} checker", self.name());
            self.check(check_request).instrument(span.clone()).await
        }
    }
    async fn check_tuple_to(&self, req: &CheckRequest, ttu: &TupleToUserset) -> Result<CheckResult> {
        let span = info_span!("tuple-to-check");
        let _enter = span.enter();
        let filter = TupleFilter {
            object_type_eq: Some(String::from(&req.tuple_key.object_type)),
            object_id_eq: Some(String::from(&req.tuple_key.object_id)),
            relation_eq: Some(String::from(&ttu.tupleset.relation)),
            ..Default::default()
        };
        let (tuples, _) = self.tuple_reader.clone().list(&req.tenant_id, filter, None).await?;

        let handlers: Vec<_> = tuples
            .iter()
            .filter_map(|t| {
                if t.user_type.eq(&req.tuple_key.user_type) || matches!(t.user_relation, None) {
                    return None;
                }
                Some(CheckRequest {
                    tenant_id: req.tenant_id.to_owned(),
                    model_id: req.model_id.to_owned(),
                    typesystem: req.typesystem.clone(),
                    tuple_key: TupleKey {
                        user_type: String::from(&t.user_type),
                        user_id: String::from(&t.user_id),
                        user_relation: String::from(""),
                        relation: String::from(&ttu.computed_userset.relation),
                        object_type: String::from(&t.user_type),
                        object_id: String::from(&t.user_id),
                    },
                    contextual_tuples: req.contextual_tuples.clone(),
                    resolution_metadata: ResolutionMetadata {
                        depth: req.resolution_metadata.depth,
                        datastore_query_count: req.resolution_metadata.datastore_query_count,
                    },
                    visited_paths: req.visited_paths.clone(),
                })
            })
            .collect();

        if let Some(r) = self.resolver.clone() {
            let r = r.clone();
            trace!("use {} checker", r.name());
            union_check(handlers.len(), |i| r.check(handlers.get(i).unwrap().to_owned()))
                .instrument(span.clone())
                .await
        } else {
            trace!("use {} checker", self.name());
            union_check(handlers.len(), |i| self.check(handlers.get(i).unwrap().to_owned()))
                .instrument(span.clone())
                .await
        }
    }

    fn check_set_operation<'a, 'b>(
        &'a self,
        req: &'b CheckRequest,
        operator: SetOperator,
        children: &'b Vec<Box<Userset>>,
    ) -> BoxFuture<'b, Result<CheckResult>>
    where
        'a: 'b,
    {
        async move {
            match operator {
                SetOperator::Union => {
                    union_check(children.len(), |i| self.check_rewrite(&req, children.get(i).unwrap())).await
                }
                SetOperator::Intersection => {
                    intersection_check(children.len(), |i| self.check_rewrite(&req, children.get(i).unwrap())).await
                }
                SetOperator::Exclusion => {
                    exclusion_check(
                        self.check_rewrite(&req, children.get(0).unwrap()),
                        self.check_rewrite(&req, children.get(1).unwrap()),
                    )
                    .await
                }
            }
        }
        .boxed()
    }
}
