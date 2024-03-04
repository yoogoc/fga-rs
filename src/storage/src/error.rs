use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Not found authz model")]
    NotFoundAuthzModel,
    #[error("Not found tenant")]
    NotFoundTenant,
}
