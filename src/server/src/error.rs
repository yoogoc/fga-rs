use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Server already started: {server}")]
    AlreadyStarted { server: String },
}
