use sea_orm_migration::{prelude::*, sea_query::Index};

use crate::set_default;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        manager
            .create_table(
                Table::create()
                    .table(RelationTuples::Table)
                    .if_not_exists()
                    .col(set_default(
                        ColumnDef::new(RelationTuples::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                        &backend,
                    ))
                    .col(ColumnDef::new(RelationTuples::TenantId).string_len(255).not_null())
                    .col(ColumnDef::new(RelationTuples::UserType).string_len(255).not_null())
                    .col(ColumnDef::new(RelationTuples::UserId).string_len(255).not_null())
                    .col(ColumnDef::new(RelationTuples::UserRelation).string_len(255))
                    .col(ColumnDef::new(RelationTuples::Relation).string_len(255).not_null())
                    .col(ColumnDef::new(RelationTuples::ObjectType).string_len(255).not_null())
                    .col(ColumnDef::new(RelationTuples::ObjectId).string_len(255).not_null())
                    .col(ColumnDef::new(RelationTuples::CreatedAt).timestamp())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_relation_tuples_tenant_id")
                    .table(RelationTuples::Table)
                    .col(RelationTuples::TenantId)
                    .take(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tenants::Id).string_len(255).not_null().primary_key())
                    .col(ColumnDef::new(Tenants::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Tenants::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(AuthzModels::Table)
                    .if_not_exists()
                    .col(set_default(
                        ColumnDef::new(AuthzModels::Id).big_integer().not_null().primary_key(),
                        &backend,
                    ))
                    .col(ColumnDef::new(AuthzModels::TenantId).string_len(255).not_null())
                    .col(ColumnDef::new(AuthzModels::Model).json_binary().not_null())
                    .col(ColumnDef::new(AuthzModels::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_authz_models_tenant_id")
                    .table(AuthzModels::Table)
                    .col(AuthzModels::TenantId)
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RelationTuples::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AuthzModels::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RelationTuples {
    Table,
    Id,
    TenantId,
    UserType,
    UserId,
    UserRelation,
    Relation,
    ObjectType,
    ObjectId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
    Name,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AuthzModels {
    Table,
    Id,
    TenantId,
    Model,
    CreatedAt,
}
