use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use std::collections::HashSet;

use protocol::{RelationReference, Typesystem, Userset};
use storage::{RelationshipTupleReaderRef, TupleFilter};

use crate::expander::error::ExpanderError;

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
                Userset::This => {
                    let rts = typesystem.get_directly_related_types(object_type, relation)?;
                    let mut object_ids = HashSet::new();
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
                                            relation_eq: Some(relation.to_owned()),
                                            user_type_eq: Some(typ.to_owned()),
                                            user_relation_eq: Some(user_relation.to_owned()),
                                            ..Default::default()
                                        };
                                        let (tuples, _) =
                                            self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                        object_ids.extend(tuples.iter().map(|t| t.object_id.to_owned()));
                                    } else {
                                        continue;
                                    }
                                } else {
                                    let filter = TupleFilter {
                                        object_type_eq: Some(object_type.to_owned()),
                                        relation_eq: Some(relation.to_owned()),
                                        user_type_eq: Some(typ.to_owned()),
                                        user_relation_is_null: Some(false),
                                        ..Default::default()
                                    };
                                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                    object_ids.extend(tuples.iter().map(|t| t.object_id.to_owned()));
                                }
                            } else {
                                let filter = TupleFilter {
                                    object_type_eq: Some(object_type.to_owned()),
                                    relation_eq: Some(relation.to_owned()),
                                    user_type_eq: Some(typ.to_owned()),
                                    user_id_eq: Some(user_id.to_owned()),
                                    ..Default::default()
                                };
                                let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                object_ids.extend(tuples.iter().map(|t| t.object_id.to_owned()));
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
                                let mid_object_ids = self
                                    .userset_to_objects(
                                        tenant_id,
                                        typesystem,
                                        userset,
                                        &rt_user_relation,
                                        &typ,
                                        user_type,
                                        user_id,
                                        user_relation,
                                    )
                                    .await?;
                                if mid_object_ids.is_empty() {
                                    continue;
                                }
                                let filter = TupleFilter {
                                    object_type_eq: Some(object_type.to_owned()),
                                    relation_eq: Some(relation.to_owned()),
                                    user_type_eq: Some(typ.to_owned()),
                                    user_id_in: Some(mid_object_ids.into_iter().collect()),
                                    user_relation_eq: Some(rt_user_relation),
                                    ..Default::default()
                                };
                                let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                object_ids.extend(tuples.iter().map(|t| t.object_id.to_owned()));
                            } else {
                                continue;
                            }
                        }
                    }
                    Ok(object_ids)
                }
                Userset::Computed(or) => {
                    let relation = typesystem.get_relation(object_type, &or.relation)?;
                    let object_ids = self
                        .userset_to_objects(
                            tenant_id,
                            typesystem,
                            &relation.rewrite,
                            &or.relation,
                            object_type,
                            user_type,
                            user_id,
                            user_relation,
                        )
                        .await?;
                    Ok(object_ids)
                }
                Userset::TupleTo(ttu) => {
                    let rel = typesystem.get_relation(object_type, &ttu.tupleset.relation)?;
                    let rts = typesystem.get_directly_related_types(object_type, &ttu.tupleset.relation)?;
                    let mut object_ids = HashSet::new();
                    for rt in rts {
                        match rt {
                            RelationReference::Direct(ot) => {
                                let mid_object_ids = self
                                    .userset_to_objects(
                                        tenant_id,
                                        typesystem,
                                        &rel.rewrite,
                                        &ttu.computed_userset.relation,
                                        &ot,
                                        user_type,
                                        user_id,
                                        user_relation,
                                    )
                                    .await?;
                                let filter = TupleFilter {
                                    object_type_eq: Some(object_type.to_owned()),
                                    relation_eq: Some(ttu.tupleset.relation.clone()),
                                    user_type_eq: Some(ot.clone()),
                                    user_id_in: Some(mid_object_ids.into_iter().collect()),
                                    user_relation_is_null: Some(true),
                                    ..Default::default()
                                };
                                let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                                object_ids.extend(tuples.iter().map(|t| t.object_id.to_owned()));
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
