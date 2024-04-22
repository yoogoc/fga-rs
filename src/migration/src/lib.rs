use std::error::Error;

use sea_orm_cli::MigrateSubcommands;
pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbConn;

mod m20220101_000001_pg_snowid;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_pg_snowid::Migration)]
    }
}

async fn run_migrate(db: &DbConn, command: Option<MigrateSubcommands>, verbose: bool) -> Result<(), Box<dyn Error>> {
    cli::run_migrate(Migrator, db, command, verbose).await
}
