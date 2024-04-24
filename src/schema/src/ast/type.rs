use protocol::{ObjectRelation, TupleToUserset, Userset};
use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Type {
    pub name: String,
    pub relations: Vec<Relation>,
    pub permissions: Vec<Permission>,
}

impl Type {
    pub fn new(name: String, rops: Vec<RelationOrPermission>) -> Self {
        let mut relations = vec![];
        let mut permissions = vec![];
        for rop in rops {
            match rop {
                RelationOrPermission::Relation(r) => relations.push(r),
                RelationOrPermission::Permission(p) => permissions.push(p),
            }
        }
        Self {
            name,
            relations,
            permissions,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelationOrPermission {
    Relation(Relation),
    Permission(Permission),
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Relation {
    pub name: String,
    pub subjects: Vec<RelationshipSet>,
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Permission {
    pub name: String,
    pub permission: Relationship,
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    Set(RelationshipSet),
    Union {
        children: Vec<Box<Relationship>>,
    },
    Intersection {
        children: Vec<Box<Relationship>>,
    },
    Difference {
        base: Box<Relationship>,
        subtract: Box<Relationship>,
    },
}

impl Relationship {
    pub fn compute(self) -> Relationship {
        match self {
            Relationship::Set(_) => self,
            Relationship::Union { children } => {
                let mut rss = vec![];
                let mut is_simplest = true;
                for child in children {
                    let child = child.compute();
                    match child {
                        Relationship::Set(_) => rss.push(Box::new(child)),
                        Relationship::Union { children } => {
                            is_simplest = false;
                            for child in children {
                                rss.push(Box::new(child.compute()));
                            }
                        }
                        Relationship::Intersection { .. } => rss.push(Box::new(child.compute())),
                        Relationship::Difference { .. } => rss.push(Box::new(child.compute())),
                    }
                }
                if rss.len() == 1 {
                    let rs = rss.pop().unwrap();
                    *rs
                } else {
                    if is_simplest {
                        Relationship::Union { children: rss }
                    } else {
                        Relationship::Union { children: rss }.compute()
                    }
                }
            }
            Relationship::Intersection { children } => {
                let mut rss = vec![];
                let mut is_simplest = true;
                for child in children {
                    let child = child.compute();
                    match child {
                        Relationship::Set(_) => rss.push(Box::new(child)),
                        Relationship::Union { .. } => rss.push(Box::new(child.compute())),
                        Relationship::Intersection { children } => {
                            is_simplest = false;
                            for child in children {
                                rss.push(Box::new(child.compute()));
                            }
                        }
                        Relationship::Difference { .. } => rss.push(Box::new(child.compute())),
                    }
                }
                if rss.len() == 1 {
                    let rs = rss.pop().unwrap();
                    *rs
                } else {
                    if is_simplest {
                        Relationship::Intersection { children: rss }
                    } else {
                        Relationship::Intersection { children: rss }.compute()
                    }
                }
            }
            Relationship::Difference { base, subtract } => Relationship::Difference {
                base: Box::new(base.compute()),
                subtract: Box::new(subtract.compute()),
            },
        }
    }

    pub(crate) fn to_userset(&self) -> Userset {
        match self {
            Relationship::Set(s) => match s {
                RelationshipSet::Single(rel) => Userset::Computed(ObjectRelation {
                    object: "".into(),
                    relation: rel.into(),
                }),
                RelationshipSet::Set(rel, obj) => Userset::TupleTo(TupleToUserset {
                    tupleset: ObjectRelation {
                        object: "".into(),
                        relation: obj.into(),
                    },
                    computed_userset: ObjectRelation {
                        object: "".into(),
                        relation: rel.into(),
                    },
                }),
            },
            Relationship::Union { children } => Userset::Union {
                children: children.iter().map(|child| Box::new(child.to_userset())).collect(),
            },
            Relationship::Intersection { children } => Userset::Intersection {
                children: children.iter().map(|child| Box::new(child.to_userset())).collect(),
            },
            Relationship::Difference { base, subtract } => Userset::Difference {
                base: Box::new(base.to_userset()),
                subtract: Box::new(subtract.to_userset()),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipSet {
    Single(String),
    Set(String, String),
}
