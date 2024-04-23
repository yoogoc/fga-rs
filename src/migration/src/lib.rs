use std::error::Error;

use sea_orm_cli::MigrateSubcommands;
pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, DbBackend, DbConn};

mod m20220101_000001_pg_snowid;
mod m20240423_011759_init_tables;

pub struct Migrator;

fn set_default<'a>(def: &'a mut ColumnDef, backend: &'a DbBackend) -> &'a mut ColumnDef {
    match backend {
        DatabaseBackend::Postgres => def.default(SimpleExpr::Custom("id_generator()".to_string())),
        _ => def.auto_increment(),
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_pg_snowid::Migration),
            Box::new(m20240423_011759_init_tables::Migration),
        ]
    }
}

pub async fn run_migrate(
    db: &DbConn,
    command: Option<MigrateSubcommands>,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    cli::run_migrate(Migrator, db, command, verbose).await
}
