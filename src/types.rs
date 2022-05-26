pub use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
/// argument structure
pub struct Args {

    /// Mode should be cache or read, cache collect pairs data and read show the cached data
    #[clap(short, long)]
    pub mode: String,

    /// Pairs should collect coins with pair
    #[clap(short, long)]
    pub pairs: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Web socket structure
pub struct WebSocket {
    pub name: String,
    pub ws_base_url: String,
    pub req_param: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// coinbase request parameter structure
pub struct BinanceReqParam {
    pub method: String,
    pub params: Vec<String>,
    pub id: i32
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

#[derive(Debug, Serialize, Deserialize, Clone)]
/// pairs cache structure
pub struct PairsCache {
    pub prices: Vec<PricesPairs>,
    pub aggregate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
