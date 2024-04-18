use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use std::collections::HashSet;

use protocol::{RelationReference, Typesystem, Userset};
use storage::{RelationshipTupleReaderRef, TupleFilter};

use super::error::ExpanderError;

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
                    let rts = typesystem.get_directly_related_types(object_type, relation)?;
                    let mut user_ids = HashSet::new();
                    for rt in rts {
                        let (typ, rt_user_relation, user_relation_is_exist) = match rt {
                            RelationReference::Direct(typ) => (typ, None, false),
                            RelationReference::Relation { r#type, relation } => (r#type, Some(relation), true),
                            RelationReference::Wildcard(typ) => (typ, None, true),
                        };
                        if user_type.eq(&typ) {
                            if user_relation_is_exist {
                                if let Some(user_relation) = user_relation {
                                    if matches!(rt_user_relation, Some(ur) if ur.eq(user_relation)) {
                                        let filter = TupleFilter {
                                            object_type_eq: Some(object_type.to_owned()),
                                            object_id_eq: Some(object_id.to_owned()),
                                            relation_eq: Some(relation.to_owned()),
                                            user_type_eq: Some(typ.to_owned()),
                                            user_relation_eq: Some(user_relation.to_owned()),
                                            ..Default::default()
                                        };
                                        let (tuples, _) =
                                            self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                        user_ids.extend(tuples.into_iter().map(|t| t.user_id));
                                    } else {
                                        continue;
                                    }
                                } else {
                                    let filter = TupleFilter {
                                        object_type_eq: Some(object_type.to_owned()),
                                        object_id_eq: Some(object_id.to_owned()),
                                        relation_eq: Some(relation.to_owned()),
                                        user_type_eq: Some(typ.to_owned()),
                                        user_relation_is_null: Some(false),
                                        ..Default::default()
                                    };
                                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                    user_ids.extend(tuples.into_iter().map(|t| t.user_id));
                                }
                            } else {
                                let filter = TupleFilter {
                                    object_type_eq: Some(object_type.to_owned()),
                                    object_id_eq: Some(object_id.to_owned()),
                                    relation_eq: Some(relation.to_owned()),
                                    user_type_eq: Some(typ.to_owned()),
                                    ..Default::default()
                                };
                                let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                user_ids.extend(tuples.into_iter().map(|t| t.user_id));
                            }
                        } else {
                            // type is difference, but rt_user_relation exists
                            // examples: check request = {ot=folder, r=view, ut=user, ui=1}
                            // relation:
                            // user
                            // group { member: user }
                            // folder {
                            //   viewer: user | group#member
                            //   parent: folder
                            //   view: viewer | parent#view
                            // }
                            //
                            // current condition is group#member
                            if let Some(rt_user_relation) = rt_user_relation {
                                let mid_user_ids = self
                                    .userset_to_users(
                                        tenant_id,
                                        typesystem,
                                        rewrite,
                                        relation,
                                        object_type,
                                        object_id,
                                        &typ,
                                        &None,
                                    )
                                    .await?;
                                if mid_user_ids.is_empty() {
                                    continue;
                                }
                                let filter = TupleFilter {
                                    object_type_eq: Some(typ.to_owned()),
                                    object_id_in: Some(mid_user_ids.into_iter().collect()),
                                    relation_eq: Some(rt_user_relation),
                                    user_type_eq: Some(user_type.to_owned()),
                                    user_relation_is_null: Some(true),
                                    ..Default::default()
                                };
                                let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                user_ids.extend(tuples.into_iter().map(|t| t.user_id));
                            } else {
                                continue;
                            }
                        }
                    }
                    Ok(user_ids)
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
                Userset::TupleTo(ttu) => {
                    let rel = typesystem.get_relation(object_type, &ttu.tupleset.relation)?;
                    let rts = typesystem.get_directly_related_types(object_type, &ttu.tupleset.relation)?;
                    let mut object_ids = HashSet::new();
                    for rt in rts {
                        match rt {
                            RelationReference::Direct(ot) => {
                                let mid_user_ids = self
                                    .userset_to_users(
                                        tenant_id,
                                        typesystem,
                                        &rel.rewrite,
                                        &ttu.computed_userset.relation,
                                        &ot,
                                        object_id,
                                        user_type,
                                        user_relation,
                                    )
                                    .await?;
                                let filter = TupleFilter {
                                    object_type_eq: Some(object_type.to_owned()),
                                    object_id_in: Some(mid_user_ids.into_iter().collect()),
                                    relation_eq: Some(ttu.tupleset.relation.clone()),
                                    user_type_eq: Some(ot.clone()),
                                    user_relation_is_null: Some(true),
                                    ..Default::default()
                                };
                                let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                object_ids.extend(tuples.into_iter().map(|t| t.object_id));
                            }
                            RelationReference::Wildcard(_) | RelationReference::Relation { .. } => {
                                return Err(ExpanderError::NotOnlyDirect {
                                    tupleset: ttu.tupleset.relation.clone(),
                                }
                                .into());
                            }
                        }
                    }
                    Ok(object_ids)
                }
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
