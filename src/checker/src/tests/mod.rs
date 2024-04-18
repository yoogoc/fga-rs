mod check;
mod expand_objects;
mod expand_users;

use std::sync::Arc;

use protocol::Typesystem;
use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};
use serde::{Deserialize, Serialize};
use storage::{
    sea::{tuple::Entity as TupleEntity, Storage},
    RelationshipTupleReaderRef, RelationshipTupleWriterRef,
};

#[derive(Clone)]
struct Model {
    tenant_id: String,
    typesystem: Typesystem,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct ModelJson {
    tenant_id: String,
    id: String,
    dsl: String,
}

async fn init() -> (Model, RelationshipTupleReaderRef) {
    let models: Vec<ModelJson> = serde_json::from_str(include_str!("../fixtures/models.json")).unwrap();
    let model = &models[0];
    let tuples = serde_json::from_str(include_str!("../fixtures/tuples.json")).unwrap();

    let (schema, _) = schema::parse(&model.dsl).unwrap();
    let authz_model = schema.to_typesystem();

    let conn = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(DbBackend::Sqlite);
    let stmt = schema.create_table_from_entity(TupleEntity);
    conn.execute(conn.get_database_backend().build(&stmt)).await.unwrap();

    let storage = Storage::new(Arc::new(conn));
    let tuple_writer: RelationshipTupleWriterRef = Arc::new(storage.clone());
    let tuple_reader: RelationshipTupleReaderRef = Arc::new(storage.clone());
    tuple_writer.save(&model.tenant_id, tuples).await.unwrap();

    (
        Model {
            tenant_id: model.tenant_id.clone(),
            // model_id: model.id.clone(),
            typesystem: authz_model,
        },
        tuple_reader,
    )
}

#[tokio::test]
async fn test_init() {
    let _ = init().await;
}
