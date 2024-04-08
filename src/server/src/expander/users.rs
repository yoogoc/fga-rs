use anyhow::Result;
use futures::future::BoxFuture;
use std::collections::HashSet;

use protocol::{RelationReference, Typesystem, Userset};
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
        // async move {
        // }
        // .boxed()
        let _ = tenant_id;
        let _ = typesystem;
        let _ = rewrite;
        let _ = relation;
        let _ = object_type;
        let _ = object_id;
        let _ = user_type;
        let _ = user_relation;
        todo!()
    }
}

fn try_get_rr<'a>(
    user_type: &str,
    user_relation: &Option<String>,
    rts: &'a Vec<RelationReference>,
) -> Option<&'a RelationReference> {
    let rr = rts
        .iter()
        .filter(|rr| match rr {
            RelationReference::Direct(name) => name.eq(user_type),
            RelationReference::Relation { r#type, relation } => {
                r#type.eq(user_type) && user_relation.is_some() && relation.eq(user_relation.as_ref().unwrap())
            }
            RelationReference::Wildcard(name) => name.eq(user_type),
        })
        .next();
    rr
}
