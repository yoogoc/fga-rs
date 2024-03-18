use std::vec;

#[derive(Debug, PartialEq)]
pub struct Schema {
    pub types: Vec<Type>,
    // pub conds: Vec<Condition>,
}

impl Schema {
    pub fn new(types: Vec<SchemaUnit>) -> Self {
        let mut ts = vec![];
        for typ in types {
            match typ {
                SchemaUnit::Type(typ) => ts.push(typ),
            }
        }
        Self { types: ts }
    }
}

#[derive(Debug, PartialEq)]
pub enum SchemaUnit {
    Type(Type),
}

#[derive(Debug, Clone, PartialEq)]
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

pub type Suite<T> = Vec<T>;

#[derive(Debug, PartialEq, Clone)]
pub enum RelationOrPermission {
    Relation(Relation),
    Permission(Permission),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Relation {
    pub name: String,
    pub subjects: Vec<RelationshipSet>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Permission {
    pub name: String,
    pub permissions: Vec<Relationship>,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum RelationshipSet {
    Single(String),
    Set(String, String),
}
