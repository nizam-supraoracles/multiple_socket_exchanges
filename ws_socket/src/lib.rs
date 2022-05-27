#[cfg(test)]
mod test;

use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::fs::{self, File};
use std::time::Duration;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

mod types;
use crate::types::*;
pub mod errors;
pub mod helpers;
pub mod parser;

/// start execution
pub async fn start() -> WSResult<()> {
    let args: Args = Args::parse();
    let mode: String = args.mode;

    if mode == "cache" {
        // get pairs from the argument
        let pairs: String = args.pairs;

        if pairs != "" {
            if check_pairs(&pairs) {
                handle_cache_mode(pairs.split(",").collect()).await?;
            }
        } else {
            println!("Pairs is required");
        }
    } else if mode == "read" {
        handle_read_mode()?;
    } else {
        println!("Invalid mode");
    }
    Ok(())
}

/// check pair is valid format
pub fn check_pairs(pairs: &str) -> bool {
    let pairs_split: Vec<&str> = pairs.split(",").collect();

    let mut count = 0;
    for pair in &pairs_split {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            count += 1;
            println!("Pair: {}", pair);
        } else {
            println!("Pair: {} is not valid format", pair);
        }
    }
    if count == pairs_split.len() {
        true
    } else {
        false
    }
}

/// handle cache mode argument and collect data from multiple exchange
async fn handle_cache_mode(pairs: Vec<&str>) -> WSResult<()> {
    // read json file of web socket urls
    let ws_details_file: File = fs::File::open("ws_details.json")?;

    let ws_details: Vec<WebSocket> = serde_json::from_reader(&ws_details_file)?;

    let binance_ws_api: String = helpers::binance_req_url(&ws_details[0].ws_base_url, &pairs);
    let coinbase_ws_api: &String = &ws_details[1].ws_base_url;
    let okex_ws_api: &String = &ws_details[2].ws_base_url;

    let binance_url_parse = Url::parse(&binance_ws_api)?;
    // connect binance socket
    let (binance_socket, _binance_response) = connect_async(binance_url_parse).await?;

    let (mut _binance_write, mut binance_read) = binance_socket.split();

    let binance_req_param: String =
        helpers::create_req_params(RequestParamType::Binance, &ws_details[0].req_param, &pairs)?;
    _binance_write
        .send(Message::Text(binance_req_param))
        .await?;

    let coinbase_url_parse = Url::parse(&coinbase_ws_api)?;
    // connect coinbase socket
    let coinbase_req_param: String =
        helpers::create_req_params(RequestParamType::Coinbase, &ws_details[1].req_param, &pairs)?;
    let (coinbase_socket, _coinbase_response) = connect_async(coinbase_url_parse).await?;

    // subscribe coinbase websockets
    let (mut coinbase_write, mut coinbase_read) = coinbase_socket.split();

    coinbase_write
        .send(Message::Text(coinbase_req_param))
        .await?;

    let okex_url_parse = Url::parse(&okex_ws_api)?;
    // connect okex socket
    let okex_req_param: String =
        helpers::create_req_params(RequestParamType::Okex, &ws_details[2].req_param, &pairs)?;
    let (okex_socket, _okex_response) = connect_async(okex_url_parse).await?;

    // subscribe okex websocket
    let (mut okex_write, mut okex_read) = okex_socket.split();
    okex_write.send(Message::Text(okex_req_param)).await?;

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    insert_pairs(pairs, &mut pairs_cache);

    let mut interval = time::interval(Duration::from_secs(10));
    let mut interval_flag = false;
    loop {
        tokio::select! {
            msg = binance_read.next() => {
                if let Some(msg) = msg {

                    let msg_binance = parser::message_parser(msg)?;

                    let binance_error_check: serde_json::Value =  serde_json::from_str(&msg_binance)?;

                    if binance_error_check["result"] == "error" {
                        eprintln!("Binance: {:?}", binance_error_check);
                        break;
                    }

                    // Serialize binance response
                    let binance_response: BinanceResponse = match serde_json::from_str(&msg_binance) {
                        Ok(p) => p,
                        Err(_) => BinanceResponse { s: "".to_string(), c: "0.0".to_string() }
                    };

                    if binance_response.s != "" {
                        let price = binance_response.c.parse::<f64>()?;
                        update_price_cache(&mut pairs_cache, binance_response.s.to_string(), ws_details[0].name.to_string(), price);
                    }

                }
            },
            msg = coinbase_read.next() => {
                if let Some(msg) = msg {

                    let msg_coinbase = parser::message_parser(msg)?;

                    let coinbase_error_check: serde_json::Value =  serde_json::from_str (&msg_coinbase)?;

                    if coinbase_error_check["type"] == "error" {
                        eprintln!("Coinbase: {:?}", coinbase_error_check["reason"]);
                        break;
                    }

                    // Serialize coinbase response
                    let coinbase_response: CoinbaseResponse = match serde_json::from_str (&msg_coinbase) {
                        Ok (p) => p,
                        Err (_) => CoinbaseResponse { product_id: "".to_string(), price: "0.0".to_string() },
                    };

                    if coinbase_response.product_id != "" {

                        let price = coinbase_response.price.parse::<f64>()?;

                        // get pair cache and push coinbase response, name and price
                        let key = helpers::pair_key(&coinbase_response.product_id);
                        update_price_cache(&mut pairs_cache, key, ws_details[1].name.to_string(), price);
                    }
                }
            },
            msg = okex_read.next() => {
                if let Some(msg) = msg {

                    let msg_okex = parser::message_parser(msg)?;

                    let okex_error_check: serde_json::Value =  serde_json::from_str(&msg_okex)?;

                    if okex_error_check["event"] == "error" {
                        eprintln!("Okex: {:?}", okex_error_check["msg"]);
                        break;
                    }

                    // Serialize okex response
                    let okex_response: OkexResponse = match serde_json::from_str(&msg_okex) {
                        Ok (p) => p,
                        Err (_) => OkexResponse { data: vec![ ] },
                    };

                    if okex_response.data.len() > 0 {

                        let price = okex_response.data[0].last.parse::<f64>()?;

                        // get pair cache and push okex response, name and price
                        let key = helpers::pair_key(&okex_response.data[0].inst_id);
                        update_price_cache(&mut pairs_cache, key, ws_details[2].name.to_string(), price);
                    }
                }
            }
            _ = interval.tick() => {
                if interval_flag {

                    write_pairs_cache(pairs_cache).await?;
                    println!("Cache complete");

                    break;
                }
                interval_flag =true;
            }
        }
    }
    Ok(())
}

/// update price cache in hashmap
fn update_price_cache(
    pairs_cache: &mut HashMap<String, PairsCache>,
    key: String,
    name: String,
    price: f64,
) {
    pairs_cache.get_mut(&key).map(|pair| {
        pair.prices.push(PricesPairs { name, price });
    });
}

/// insert initial key and pairs in hashmap
fn insert_pairs(pairs: Vec<&str>, pairs_cache: &mut HashMap<String, PairsCache>) {
    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();

        pairs_cache.insert(
            format!("{}{}", coin[0].to_uppercase(), coin[1].to_uppercase()),
            PairsCache {
                prices: vec![],
                aggregate: 0.0,
            },
        );
    }
}

/// aggregate prices pair wise and write caches in file
async fn write_pairs_cache(pairs: HashMap<String, PairsCache>) -> WSResult<()> {
    let mut pairs_save = pairs.clone();
    for pair in pairs {
        let (key, mut pari_cache) = pair;

        let mut amount = 0.0;
        for price in &pari_cache.prices {
            amount += &price.price;
        }
        pari_cache.aggregate = amount / pari_cache.prices.len() as f64;
        pairs_save.insert(key, pari_cache);
    }
    let content = serde_json::to_string(&pairs_save)?;
    fs::write("exchanges.json", &content)?;

    Ok(())
}

/// Handle Read mode argument and print the aggregate of pairs
fn handle_read_mode() -> WSResult<()> {
    let content = fs::File::open("exchanges.json")?;
    let pairs: HashMap<String, PairsCache> = serde_json::from_reader(&content)?;

    for pair in &pairs {
        let (key, pari_cache) = pair;

        println!("pair: {:?} -> aggregate: {:?}", key, pari_cache.aggregate);
    }

    Ok(())
}