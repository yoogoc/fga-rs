use sea_orm_migration::{
    prelude::*,
    sea_orm::{DatabaseBackend, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::MySql => todo!(),
            DatabaseBackend::Postgres => manager
                .get_connection()
                .execute(Statement::from_string(
                    manager.get_database_backend(),
                    include_str!("sql/postgres_snowid.sql"),
                ))
                .await
                .map(|_| ()),
            DatabaseBackend::Sqlite => todo!(),
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::MySql => todo!(),
            DatabaseBackend::Postgres => manager
                .get_connection()
                .execute(Statement::from_string(
                    manager.get_database_backend(),
                    "DROP SEQUENCE global_id_seq;\nDROP FUNCTION id_generator;",
                ))
                .await
                .map(|_| ()),
            DatabaseBackend::Sqlite => todo!(),
        }
    }
}
