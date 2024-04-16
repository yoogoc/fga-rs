use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use std::collections::HashSet;

use protocol::{Typesystem, Userset};
use storage::RelationshipTupleReaderRef;

pub struct UsersExpander {
    tuple_reader: RelationshipTupleReaderRef,
}

impl UsersExpander {
    pub fn new(tuple_reader: RelationshipTupleReaderRef) -> Self {
        Self { tuple_reader }
    }
}

impl UsersExpander {
    pub async fn users(
        &self,
        typesystem: Typesystem,
        tenant_id: String,
        relation: String,
        object_type: String,
        object_id: String,
        user_type: String,
        user_relation: Option<String>,
    ) -> Result<HashSet<String>> {
        let typ = typesystem.get_relation(&object_type, &relation)?;

        self.userset_to_users(
            &tenant_id,
            &typesystem,
            &typ.rewrite,
            &relation,
            &object_type,
            &object_id,
            &user_type,
            &user_relation,
        )
        .await
    }

    fn userset_to_users<'a, 'b>(
        &'a self,
        tenant_id: &'b str,
        typesystem: &'b Typesystem,
        rewrite: &'b Userset,
        relation: &'b str,
        object_type: &'b str,
        object_id: &'b str,
        user_type: &'b str,
        user_relation: &'b Option<String>,
    ) -> BoxFuture<'b, Result<HashSet<String>>>
    where
        'a: 'b,
    {
        async move {
            match rewrite {
                Userset::This => {
                    let _ = self.tuple_reader;
                    todo!()
                }
                Userset::Computed(or) => {
                    let relation = typesystem.get_relation(object_type, &or.relation)?;
                    let user_ids = self
                        .userset_to_users(
                            tenant_id,
                            typesystem,
                            &relation.rewrite,
                            &or.relation,
                            object_type,
                            object_id,
                            user_type,
                            user_relation,
                        )
                        .await?;
                    Ok(user_ids)
                }
                Userset::TupleTo(_) => todo!(),
                Userset::Union { children } => {
                    let mut user_ids = HashSet::new();
                    for child in children {
                        user_ids.extend(
                            self.userset_to_users(
                                tenant_id,
                                typesystem,
                                child,
                                relation,
                                object_type,
                                object_id,
                                user_type,
                                user_relation,
                            )
                            .await?,
                        )
                    }

                    Ok(user_ids)
                }
                Userset::Intersection { children } => {
                    let mut user_ids = HashSet::new();
                    for child in children {
                        let child_user_ids = self
                            .userset_to_users(
                                tenant_id,
                                typesystem,
                                child,
                                relation,
                                object_type,
                                object_id,
                                user_type,
                                user_relation,
                            )
                            .await?;
                        if child_user_ids.len() == 0 {
                            return Ok(HashSet::new());
                        } else {
                            user_ids = user_ids
                                .clone()
                                .intersection(&child_user_ids)
                                .map(|x| x.to_owned())
                                .collect();
                        }
                    }

                    Ok(user_ids)
                }
                Userset::Difference { base, subtract } => {
                    let base_user_ids = self
                        .userset_to_users(
                            tenant_id,
                            typesystem,
                            base,
                            relation,
                            object_type,
                            object_id,
                            user_type,
                            user_relation,
                        )
                        .await?;
                    let subtract_user_ids = self
                        .userset_to_users(
                            tenant_id,
                            typesystem,
                            subtract,
                            relation,
                            object_type,
                            object_id,
                            user_type,
                            user_relation,
                        )
                        .await?;

                    Ok(base_user_ids
                        .difference(&subtract_user_ids)
                        .map(|x| x.to_owned())
                        .collect())
                }
            }
        }
        .boxed()
    }
}
