use anyhow::{Ok, Result};
use async_trait::async_trait;
use protocol::{RelationReference, SetOperator, TupleKey, TupleToUserset, Userset, WILDCARD};
use storage::{RelationshipTupleReader, TupleFilter};

use crate::{
    error::CheckerError, exclusion_check, graph::ResolutionMetadata, intersection_check,
    union_check, CheckRequest, CheckResult, Checker,
};

pub struct LocalChecker {
    pub resolver: Box<dyn Checker + Send + Sync>,
    pub tuple_reader: Box<dyn RelationshipTupleReader + Send + Sync>,
}

#[async_trait]
impl Checker for LocalChecker {
    async fn check(&self, req: CheckRequest) -> Result<CheckResult> {
        let relation = req
            .typesystem
            .get_relation(&req.tuple_key.object_type, &req.tuple_key.relation)?;
        self.check_rewrite(&req, &relation.rewrite).await
    }

    async fn close(&self) {}
}

impl LocalChecker {
    async fn check_rewrite(&self, req: &CheckRequest, rewrite: &Userset) -> Result<CheckResult> {
        match rewrite {
            Userset::This => self.check_direct(req).await,
            Userset::Computed(or) => self.check_computed(req, &or.relation).await,
            Userset::TupleTo(ttu) => self.check_tuple_to(req, ttu).await,
            Userset::Union { children } => {
                self.check_set_operation(req, SetOperator::Union, children)
                    .await
            }
            Userset::Intersection { children } => {
                self.check_set_operation(req, SetOperator::Intersection, children)
                    .await
            }
            Userset::Difference { base, subtract } => {
                let children = vec![base.to_owned(), subtract.to_owned()];
                self.check_set_operation(req, SetOperator::Intersection, &children)
                    .await
            }
        }
    }
    async fn check_direct(&self, req: &CheckRequest) -> Result<CheckResult> {
        let related_usersets = req
            .typesystem
            .get_directly_related_types(&req.tuple_key.object_type, &req.tuple_key.relation)?;
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
                    if r#type.eq(&req.tuple_key.user_type)
                        && relation.eq(&req.tuple_key.user_relation)
                    {
                        Some(TupleFilter {
                            user_type_eq: Some(String::from(&req.tuple_key.user_type)),
                            user_relation_eq: Some(String::from(&req.tuple_key.user_relation)),
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
            .list(&req.typesystem.tenant_id, filter, None)
            .await?;

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
                if !t.user_type.eq(&req.tuple_key.user_type) {
                    return None;
                }
                let f = || t.user_id.eq(WILDCARD) || t.user_id.eq(&req.tuple_key.user_id);
                if let Some(relation) = &t.user_relation {
                    Some(relation.eq(&req.tuple_key.user_relation) && f())
                } else {
                    Some(f())
                }
            })
            .collect();
        if !direct_asserts.is_empty() {
            return Ok(CheckResult::new_dqc(
                direct_asserts.iter().any(|x| x.to_owned()),
                req.resolution_metadata.datastore_query_count + 1,
            ));
        }
        // TODO concurrence
        // let handlers = tuples
        tuples.iter().filter_map(|t| {
            if t.user_type.eq(&req.tuple_key.user_type) || matches!(t.user_relation, None) {
                return None;
            }
            Some(|| {
                self.resolver.check(CheckRequest {
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
            })
        });
        // .collect();

        // union_check(handlers)
        todo!()
    }
    async fn check_computed(&self, req: &CheckRequest, relation: &str) -> Result<CheckResult> {
        self.resolver
            .check(CheckRequest {
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
            })
            .await
    }
    async fn check_tuple_to(
        &self,
        req: &CheckRequest,
        ttu: &TupleToUserset,
    ) -> Result<CheckResult> {
        let filter = TupleFilter {
            object_type_eq: Some(String::from(&req.tuple_key.object_type)),
            object_id_eq: Some(String::from(&req.tuple_key.object_id)),
            relation_eq: Some(String::from(&ttu.tupleset.relation)),
            ..Default::default()
        };
        let (tuples, _) = self
            .tuple_reader
            .list(&req.typesystem.tenant_id, filter, None)
            .await?;

        // TODO concurrence
        // let handlers = tuples
        tuples.iter().filter_map(|t| {
            if t.user_type.eq(&req.tuple_key.user_type) || matches!(t.user_relation, None) {
                return None;
            }
            Some(|| {
                self.resolver.check(CheckRequest {
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
        });
        // .collect();

        // union_check(handlers)
        todo!()
    }
    async fn check_set_operation(
        &self,
        req: &CheckRequest,
        operator: SetOperator,
        children: &Vec<Box<Userset>>,
    ) -> Result<CheckResult> {
        // let handlers = children
        //     .iter()
        //     .map(|rewrite| || self.check_rewrite(&req, rewrite))
        //     .collect();
        // match operator {
        //     SetOperator::Union => union_check(handlers),
        //     SetOperator::Intersection => intersection_check(handlers),
        //     SetOperator::Exclusion => exclusion_check(
        //         handlers.get(0).unwrap().to_owned(),
        //         handlers.get(1).unwrap().to_owned(),
        //     ),
        // }
        todo!()
    }
}
