use crate::{
    errors::WSError,
    types::{BinanceResponse, CoinbaseResponse, OkexResponse, ResponseEnum, SocketType, WSResult},
};
use serde_json::Value;
use tokio_tungstenite::tungstenite::{Error, Message};

/// parse message
pub fn message_parser(
    socket_type: SocketType,
    msg: Result<Message, Error>,
) -> WSResult<ResponseEnum> {
    let message = match msg? {
        Message::Text(s) => Ok(s),
        _ => Err(WSError::UnknownResponse),
    }?;

    let msg: serde_json::Value = serde_json::from_str(&message)?;

    match socket_type {
        SocketType::Binance => {
            let msg = parse_binance_response(msg)?;
            Ok(ResponseEnum::Binance(msg))
        }
        SocketType::Okex => {
            let msg = parse_okex_response(msg)?;
            Ok(ResponseEnum::Okex(msg))
        }
        SocketType::Coinbase => {
            let msg = parse_coinbase_response(msg)?;
            Ok(ResponseEnum::Coinbase(msg))
        }
    }
}

/// parse binance data from socket response
fn parse_binance_response(msg: Value) -> WSResult<BinanceResponse> {
    if msg["result"] == "error" {
        let error = format!("{:?}", msg);
        return Err(WSError::SocketResponseError(error));
    }
    // Serialize binance response
    let binance_response: BinanceResponse = match serde_json::from_value(msg) {
        Ok(p) => p,
        Err(_) => BinanceResponse {
            s: "".to_string(),
            c: "0.0".to_string(),
        },
    };
    Ok(binance_response)
}

/// parse coinbase data from socket response
fn parse_coinbase_response(msg: Value) -> WSResult<CoinbaseResponse> {
    if msg["type"] == "error" {
        let error = format!("{:?}", msg);
        return Err(WSError::SocketResponseError(error));
    }

    // Serialize coinbase response
    let coinbase_response: CoinbaseResponse = match serde_json::from_value(msg) {
        Ok(p) => p,
        Err(_) => CoinbaseResponse {
            product_id: "".to_string(),
            price: "0.0".to_string(),
        },
    };
    Ok(coinbase_response)
}

/// parse okex data from socket response
fn parse_okex_response(msg: Value) -> WSResult<OkexResponse> {
    if msg["event"] == "error" {
        let error = format!("{:?}", msg);
        return Err(WSError::SocketResponseError(error));
    }

    // Serialize okex response
    let okex_response: OkexResponse = match serde_json::from_value(msg) {
        Ok(p) => p,
        Err(_) => OkexResponse { data: vec![] },
    };
    Ok(okex_response)
}
