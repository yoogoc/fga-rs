mod objects;
mod users;

use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use protocol::{Typesystem, Userset};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use storage::{RelationshipTupleReaderRef, TupleFilter};

pub use objects::ObjectsExpander;
pub use users::UsersExpander;

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
