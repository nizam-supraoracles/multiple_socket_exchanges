use std::{io, num::ParseFloatError};
use thiserror::Error;
use tungstenite::Error as TError;
use url::ParseError;

#[derive(Error, Debug)]
pub enum WSError {
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("Serde Error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Parse Error")]
    ParseError(#[from] ParseError),
    #[error("Tungsnite Error")]
    TungsniteError(#[from] TError),
    #[error("ParseFloatError")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Got Unknown Response")]
    UnknownResponse,
    #[error("Socket Response Error:{0}")]
    SocketResponseError(String),
}
