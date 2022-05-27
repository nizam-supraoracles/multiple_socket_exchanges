pub use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::{errors::WSError, helpers};

#[derive(Parser, Debug)]
#[clap(author = "Nizam", version, about)]
/// Web socket argument structure
pub struct Args {
    /// Mode should be cache or read, cache collect pairs data and read show the cached data
    #[clap(short, long)]
    pub mode: String,

    /// Pairs should collect coins with pair
    #[clap(short, long, default_value = "")]
    pub pairs: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Web socket structure
pub struct WebSocketConfig {
    pub name: String,
    pub ws_base_url: String,
    pub req_param: Value,
}

#[derive(Debug, Serialize, Deserialize)]
/// coinbase request parameter structure
pub struct BinanceReqParam {
    pub method: String,
    pub params: Vec<String>,
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
/// coinbase request parameter structure
pub struct CoinbaseReqParam {
    #[serde(rename = "type")]
    pub type_name: String,
    pub channels: Vec<String>,
    pub product_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
/// okex request parameter structure
pub struct OkexReqParam {
    pub op: String,
    pub args: Vec<OkexReqParamArg>,
}

#[derive(Debug, Serialize, Deserialize)]
/// okex request parameter argument structure
pub struct OkexReqParamArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// pairs cache structure
pub struct PairsCache {
    pub prices: Vec<PricesPairs>,
    pub aggregate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// prices pairs structure
pub struct PricesPairs {
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
/// binanase socket response structure
pub struct BinanceResponse {
    pub s: String,
    pub c: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// coinbase socket response structuer
pub struct CoinbaseResponse {
    pub product_id: String,
    pub price: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// okex socket response child structure
pub struct OkexResponseChild {
    #[serde(rename = "instId")]
    pub inst_id: String,
    pub last: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// okex socket response parent structure
pub struct OkexResponse {
    pub data: Vec<OkexResponseChild>,
}

pub type WSResult<T> = Result<T, WSError>;

#[derive(Debug, Serialize, Deserialize)]
pub enum SocketType {
    Binance,
    Okex,
    Coinbase,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseEnum {
    Binance(BinanceResponse),
    Okex(OkexResponse),
    Coinbase(CoinbaseResponse),
}

#[derive(Debug)]
pub struct WSHandler {
    config: WebSocketConfig,
    socket_type: SocketType,
    pairs: Vec<String>,
    pub socket_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WSHandler {
    pub fn new(config: &WebSocketConfig, socket_type: SocketType, pairs: Vec<String>) -> WSHandler {
        WSHandler {
            config: config.clone(),
            socket_type,
            pairs,
            socket_stream: None,
        }
    }

    /// connect to web socket
    pub async fn connect(&mut self) -> WSResult<()> {
        match self.socket_type {
            SocketType::Binance => {
                let binance_ws_api: String =
                    helpers::binance_req_url(&self.config.ws_base_url, &self.pairs);
                let binance_url_parse = Url::parse(&binance_ws_api)?;

                let (binance_socket, _binance_response) = connect_async(binance_url_parse).await?;
                self.socket_stream = Some(binance_socket);
            }
            SocketType::Okex => {
                let okex_url_parse = Url::parse(&self.config.ws_base_url)?;
                let (okex_socket, _okex_response) = connect_async(okex_url_parse).await?;
                self.socket_stream = Some(okex_socket);
            }
            SocketType::Coinbase => {
                let coinbase_url_parse = Url::parse(&self.config.ws_base_url)?;
                let (coinbase_socket, _coinbase_response) =
                    connect_async(coinbase_url_parse).await?;
                self.socket_stream = Some(coinbase_socket);
            }
        }
        Ok(())
    }

    /// subscribe web socket
    pub async fn subscribe(&mut self) -> WSResult<()> {
        match self.socket_type {
            SocketType::Binance => {
                let binance_socket = self
                    .socket_stream
                    .as_mut()
                    .expect("There is some issue in binance socket");
                let (mut _binance_write, _) = binance_socket.split();
                let binance_req_param: String = helpers::create_req_params(
                    SocketType::Binance,
                    &self.config.req_param,
                    &self.pairs,
                )?;
                _binance_write
                    .send(Message::Text(binance_req_param))
                    .await?;
            }
            SocketType::Okex => {
                let okex_socket = self
                    .socket_stream
                    .as_mut()
                    .expect("There is some issue in okex socket");
                let okex_req_param: String = helpers::create_req_params(
                    SocketType::Okex,
                    &self.config.req_param,
                    &self.pairs,
                )?;
                let (mut okex_write, _) = okex_socket.split();
                okex_write.send(Message::Text(okex_req_param)).await?;
            }
            SocketType::Coinbase => {
                let coinbase_socket = self
                    .socket_stream
                    .as_mut()
                    .expect("There is some issue in binance socket");
                let coinbase_req_param: String = helpers::create_req_params(
                    SocketType::Coinbase,
                    &self.config.req_param,
                    &self.pairs,
                )?;
                let (mut coinbase_write, _) = coinbase_socket.split();
                coinbase_write
                    .send(Message::Text(coinbase_req_param))
                    .await?;
            }
        }
        Ok(())
    }
}
