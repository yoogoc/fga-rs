use thiserror::Error;

#[derive(Error, Debug)]
#[allow(unused)]
pub enum ExpanderError {
    #[error("`{tupleset}` relation used inside from allows only direct")]
    NotOnlyDirect { tupleset: String },
}
