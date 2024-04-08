use anyhow::Result;
use futures::future::BoxFuture;
use std::collections::HashSet;

use protocol::{Typesystem, Userset};
use storage::RelationshipTupleReaderRef;

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
        // async move {
        // }
        // .boxed()
        let _ = tenant_id;
        let _ = typesystem;
        let _ = userset;
        let _ = relation;
        let _ = object_type;
        let _ = user_type;
        let _ = user_id;
        let _ = user_relation;
        todo!()
    }
}
