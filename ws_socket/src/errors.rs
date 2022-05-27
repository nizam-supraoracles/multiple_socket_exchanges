use std::{io, num::ParseFloatError};
use thiserror::Error;
use url::ParseError;
use tungstenite::Error as TError;

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
    ParseFloatError(#[from] ParseFloatError)
}