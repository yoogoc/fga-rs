use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use std::collections::HashSet;

use protocol::{Typesystem, Userset};
use storage::RelationshipTupleReaderRef;

#[allow(unused)]
pub struct ObjectsExpander {
    tuple_reader: RelationshipTupleReaderRef,
}

impl ObjectsExpander {
    pub fn new(tuple_reader: RelationshipTupleReaderRef) -> Self {
        Self { tuple_reader }
    }
}

impl ObjectsExpander {
    pub async fn objects(
        &self,
        typesystem: Typesystem,
        tenant_id: String,
        relation: String,
        object_type: String,
        user_type: String,
        user_id: String,
        user_relation: Option<String>,
    ) -> Result<HashSet<String>> {
        let typ = typesystem.get_relation(&object_type, &relation)?;

        self.userset_to_objects(
            &tenant_id,
            &typesystem,
            &typ.rewrite,
            &relation,
            &object_type,
            &user_type,
            &user_id,
            &user_relation,
        )
        .await
    }
    fn userset_to_objects<'a, 'b>(
        &'a self,
        tenant_id: &'b str,
        typesystem: &'b Typesystem,
        userset: &'b Userset,
        relation: &'b str,
        object_type: &'b str,
        user_type: &'b str,
        user_id: &'b str,
        user_relation: &'b Option<String>,
    ) -> BoxFuture<'b, Result<HashSet<String>>>
    where
        'a: 'b,
    {
        async move {
            match userset {
                Userset::This => todo!(),
                Userset::Computed(_) => todo!(),
                Userset::TupleTo(_) => todo!(),
                Userset::Union { children } => {
                    let mut object_ids = HashSet::new();
                    for child in children {
                        object_ids.extend(
                            self.userset_to_objects(
                                tenant_id,
                                typesystem,
                                child,
                                relation,
                                object_type,
                                user_type,
                                user_id,
                                user_relation,
                            )
                            .await?,
                        )
                    }

                    Ok(object_ids)
                }
                Userset::Intersection { children } => {
                    let mut object_ids = HashSet::new();
                    for child in children {
                        let child_object_ids = self
                            .userset_to_objects(
                                tenant_id,
                                typesystem,
                                child,
                                relation,
                                object_type,
                                user_type,
                                user_id,
                                user_relation,
                            )
                            .await?;
                        if child_object_ids.len() == 0 {
                            return Ok(HashSet::new());
                        } else {
                            object_ids = object_ids
                                .clone()
                                .intersection(&child_object_ids)
                                .map(|x| x.to_owned())
                                .collect();
                        }
                    }

                    Ok(object_ids)
                }
                Userset::Difference { base, subtract } => {
                    let base_object_ids = self
                        .userset_to_objects(
                            tenant_id,
                            typesystem,
                            base,
                            relation,
                            object_type,
                            user_type,
                            user_id,
                            user_relation,
                        )
                        .await?;
                    let subtract_object_ids = self
                        .userset_to_objects(
                            tenant_id,
                            typesystem,
                            subtract,
                            relation,
                            object_type,
                            user_type,
                            user_id,
                            user_relation,
                        )
                        .await?;

                    Ok(base_object_ids
                        .difference(&subtract_object_ids)
                        .map(|x| x.to_owned())
                        .collect())
                }
            }
        }
        .boxed()
    }
}
