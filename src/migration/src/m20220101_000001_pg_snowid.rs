use sea_orm_migration::{prelude::*, sea_orm::DatabaseBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::MySql => Ok(()),
            DatabaseBackend::Postgres => manager
                .get_connection()
                .execute_unprepared(include_str!("sql/postgres_snowid.sql"))
                .await
                .map(|_| ()),
            DatabaseBackend::Sqlite => Ok(()),
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::MySql => Ok(()),
            DatabaseBackend::Postgres => manager
                .get_connection()
                .execute_unprepared("DROP SEQUENCE global_id_seq;\nDROP FUNCTION id_generator;")
                .await
                .map(|_| ()),
            DatabaseBackend::Sqlite => Ok(()),
        }
    }
}
