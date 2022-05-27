#[cfg(test)]
mod test;

use futures_util::StreamExt;
use std::collections::HashMap;
use std::fs::{self, File};
use std::time::Duration;
use tokio::time;

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
                let pairs: Vec<_> = pairs.split(",").collect();
                let pairs_string_vec: Vec<String> = pairs.iter().map(|i| i.to_string()).collect();

                handle_cache_mode(pairs_string_vec).await?;
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
async fn handle_cache_mode(pairs: Vec<String>) -> WSResult<()> {
    // read json file of web socket urls
    let ws_details_file: File = fs::File::open("ws_details.json")?;

    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;

    let mut binance_handler = WSHandler::new(&ws_details[0], SocketType::Binance, pairs.clone());
    let mut coinbase_handler = WSHandler::new(&ws_details[1], SocketType::Coinbase, pairs.clone());
    let mut okex_handler = WSHandler::new(&ws_details[2], SocketType::Okex, pairs.clone());

    // connect binance socket and subscribe
    binance_handler.connect().await?;
    binance_handler.subscribe().await?;

    // connect coinbase socket and subscribe
    coinbase_handler.connect().await?;
    coinbase_handler.subscribe().await?;

    // connect okex socket and subscribe
    okex_handler.connect().await?;
    okex_handler.subscribe().await?;

    let binance_s = binance_handler.socket_stream.as_mut()
        .expect("There is some issue in binance socket stream");
    let (_, mut binance_read) = binance_s.split();

    let coinbase_s = coinbase_handler.socket_stream.as_mut()
        .expect("There is some issue in coinbase socket stream");
    let (_, mut coinbase_read) = coinbase_s.split();

    let okex_s = okex_handler.socket_stream.as_mut()
        .expect("There is some issue in okex socket stream");
    let (_, mut okex_read) = okex_s.split();

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    insert_pairs(pairs, &mut pairs_cache);

    let mut interval = time::interval(Duration::from_secs(10));
    let mut interval_flag = false;
    loop {
        tokio::select! {
            msg = binance_read.next() => {
                if let Some(msg) = msg {
                    let response = parser::message_parser(SocketType::Binance,msg)?;
                    helpers::handle_response(&mut pairs_cache,&ws_details, response)?;
                }
            },
            msg = coinbase_read.next() => {
                if let Some(msg) = msg {
                    let response = parser::message_parser(SocketType::Coinbase,msg)?;
                    helpers::handle_response(&mut pairs_cache,&ws_details, response)?;
                }
            },
            msg = okex_read.next() => {
                if let Some(msg) = msg {

                    let response = parser::message_parser(SocketType::Okex,msg)?;
                    helpers::handle_response(&mut pairs_cache,&ws_details, response)?;
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

/// insert initial key and pairs in hashmap
fn insert_pairs(pairs: Vec<String>, pairs_cache: &mut HashMap<String, PairsCache>) {
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
