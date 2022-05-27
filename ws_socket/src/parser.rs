use crate::types::WSResult;
use tokio_tungstenite::tungstenite::{Error, Message};

/// parse message
pub fn message_parser(msg: Result<Message, Error>) -> WSResult<String> {
    let message = match msg? {
        Message::Text(s) => s,
        _ => {
            eprintln!("Error read_message");
            std::process::exit(1);
        }
    };

    Ok(message)
}
