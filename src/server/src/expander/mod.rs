use std::collections::HashSet;

use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use protocol::{ObjectRelation, Typesystem, Userset};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use storage::{RelationshipTupleReaderRef, TupleFilter};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExpandTreeNode {
    user_type: String,
    user_id: String,
    user_relation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub enum ExpandTree {
    This(Vec<ExpandTreeNode>),
    Computed(ExpandTreeNode),
    TupleTo {
        tupleset: ExpandTreeNode,
        computed: Vec<ExpandTreeNode>,
    },
    Union {
        children: Vec<Box<ExpandTree>>,
    },
    Intersection {
        children: Vec<Box<ExpandTree>>,
    },
    Difference {
        base: Box<ExpandTree>,
        subtract: Box<ExpandTree>,
    },
}
pub struct Expander {
    tuple_reader: RelationshipTupleReaderRef,
}

impl Expander {
    pub fn new(tuple_reader: RelationshipTupleReaderRef) -> Self {
        Self { tuple_reader }
    }
}

impl Expander {
    pub async fn expand(
        &self,
        typesystem: Typesystem,
        tenant_id: String,
        relation: String,
        object_type: String,
        object_id: String,
    ) -> Result<ExpandTree> {
        let typ = typesystem.get_relation(&object_type, &relation)?;
        self.userset_to_tree(&tenant_id, &typ.rewrite, &relation, &object_type, &object_id)
            .await
    }

    fn userset_to_tree<'a, 'b>(
        &'a self,
        tenant_id: &'b str,
        userset: &'b Userset,
        relation: &'b str,
        object_type: &'b str,
        object_id: &'b str,
    ) -> BoxFuture<'b, Result<ExpandTree>>
    where
        'a: 'b,
    {
        async move {
            match userset {
                Userset::This => {
                    let filter = TupleFilter {
                        object_type_eq: Some(object_type.to_string()),
                        object_id_eq: Some(object_id.to_string()),
                        relation_eq: Some(relation.to_string()),
                        ..Default::default()
                    };
                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                    Ok(ExpandTree::This(
                        tuples
                            .iter()
                            .map(|t| ExpandTreeNode {
                                user_type: t.user_type.to_owned(),
                                user_id: t.user_id.to_owned(),
                                user_relation: t.user_relation.to_owned(),
                            })
                            .collect(),
                    ))
                }
                Userset::Computed(rel) => Ok(ExpandTree::Computed(ExpandTreeNode {
                    user_type: object_type.to_owned(),
                    user_id: object_id.to_owned(),
                    user_relation: Some(rel.relation.to_owned()),
                })),
                Userset::TupleTo(rel) => {
                    let filter = TupleFilter {
                        object_type_eq: Some(object_type.to_string()),
                        object_id_eq: Some(object_id.to_string()),
                        relation_eq: Some(rel.tupleset.relation.to_string()),
                        ..Default::default()
                    };
                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;

                    Ok(ExpandTree::TupleTo {
                        tupleset: ExpandTreeNode {
                            user_type: object_type.to_owned(),
                            user_id: object_id.to_owned(),
                            user_relation: Some(rel.tupleset.relation.to_owned()),
                        },
                        computed: tuples
                            .iter()
                            .map(|t| ExpandTreeNode {
                                user_type: t.user_type.to_owned(),
                                user_id: t.user_id.to_owned(),
                                user_relation: t.user_relation.to_owned(),
                            })
                            .collect(),
                    })
                }
                Userset::Union { children } => {
                    let mut list = vec![];
                    for child in children {
                        list.push(Box::new(
                            self.userset_to_tree(&tenant_id, &child, relation, object_type, object_id)
                                .await?,
                        ))
                    }
                    Ok(ExpandTree::Union { children: list })
                }
                Userset::Intersection { children } => {
                    let mut list = vec![];
                    for child in children {
                        list.push(Box::new(
                            self.userset_to_tree(&tenant_id, &child, relation, object_type, object_id)
                                .await?,
                        ))
                    }
                    Ok(ExpandTree::Intersection { children: list })
                }
                Userset::Difference { base, subtract } => Ok(ExpandTree::Difference {
                    base: Box::new(
                        self.userset_to_tree(&tenant_id, &base, relation, object_type, object_id)
                            .await?,
                    ),
                    subtract: Box::new(
                        self.userset_to_tree(&tenant_id, &subtract, relation, object_type, object_id)
                            .await?,
                    ),
                }),
            }
        }
        .boxed()
    }
}

impl Expander {
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
        let _ = tenant_id;
        let _ = user_type;
        let _ = user_id;
        let _ = user_relation;

        self.userset_to_objects(
            &tenant_id,
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
                    let filter = TupleFilter {
                        relation_eq: Some(relation.to_string()),
                        object_type_eq: Some(object_type.to_string()),
                        user_type_eq: Some(user_type.to_string()),
                        user_id_eq: Some(user_id.to_string()),
                        user_relation_eq: user_relation.to_owned(),
                        ..Default::default()
                    };
                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                    Ok(tuples.iter().map(|t| t.object_id.to_owned()).collect())
                }
                Userset::Computed(rel) => {
                    let filter = TupleFilter {
                        relation_eq: Some(rel.relation.to_string()),
                        object_type_eq: Some(rel.object.to_string()),
                        user_type_eq: Some(user_type.to_string()),
                        user_id_eq: Some(user_id.to_string()),
                        user_relation_eq: user_relation.to_owned(),
                        ..Default::default()
                    };
                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;
                    let cumputed_object_ids = tuples.iter().map(|t| t.object_id.to_owned()).collect::<Vec<_>>();

                    let filter = TupleFilter {
                        relation_eq: Some(rel.relation.to_string()),
                        object_type_eq: Some(object_type.to_string()),
                        user_type_eq: Some(user_type.to_string()),
                        user_id_in: Some(cumputed_object_ids),
                        ..Default::default()
                    };
                    let (tuples, _) = self.tuple_reader.clone().list(tenant_id, filter, None).await?;

                    Ok(tuples.iter().map(|t| t.object_id.to_owned()).collect())
                }
                Userset::TupleTo(rel) => {
                    let userset = Userset::Computed(ObjectRelation {
                        object: rel.computed_userset.object.to_owned(),
                        relation: rel.tupleset.relation.to_owned(),
                    });
                    Ok(self
                        .userset_to_objects(
                            tenant_id,
                            &userset,
                            &rel.tupleset.relation,
                            object_type,
                            user_type,
                            user_id,
                            user_relation,
                        )
                        .await?)
                }
                Userset::Union { children } => {
                    let mut object_ids = HashSet::new();
                    for child in children {
                        object_ids.extend(
                            self.userset_to_objects(
                                tenant_id,
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

impl Expander {
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
        let _ = typesystem;
        let _ = tenant_id;
        let _ = relation;
        let _ = object_type;
        let _ = user_type;
        let _ = object_id;
        let _ = user_relation;
        todo!()
    }
}
