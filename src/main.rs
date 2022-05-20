use futures_util::{SinkExt, StreamExt};
use std::fs::{self, File};
use std::time::Duration;
use std::{collections::HashMap, error::Error};
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

mod types;
use crate::types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    let mode: String = args.mode;

    if mode == "cache" {
        // get pairs from the argument
        let pairs: String = args.pairs;

        if pairs != "" {
            let pairs_split: Vec<&str> = pairs.split(",").collect();
            if check_pairs(&pairs_split) {
                handle_cach_mode(pairs_split).await;
            }
        } else {
            println!("Pairs is required");
        }

    } else if mode == "read" {
        handle_read_mode();
    } else {
        println!("Invalid mode");
    }

    Ok(())
}

/// check pair is valid format
fn check_pairs(pairs: &Vec<&str>) -> bool {
    let mut pair_execute = false;
    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            pair_execute = true;
            println!("Pair: {}", pair);
        } else {
            println!("Pair: {} is not valid format", pair);
        }
    }
    pair_execute
}

/// handle cache mode argument and collect data from multiple exchange
async fn handle_cach_mode(pairs: Vec<&str>) {
    // read json file of web socket urls
    let ws_details_file: File =
        fs::File::open("ws_details.json").expect("Unable to open ws_details.json file ");
    let ws_details: Vec<WebSocket> = serde_json::from_reader(&ws_details_file).unwrap();

    let binance_ws_api: String = binance_req_url(&ws_details[0].ws_base_url, &pairs);
    let coinbase_ws_api: &String = &ws_details[1].ws_base_url;
    let okex_ws_api: &String = &ws_details[2].ws_base_url;

    // connect binance socket
    let (binance_socket, _binance_response) = connect_async(Url::parse(&binance_ws_api).unwrap())
        .await
        .expect("Can't connect binance.");

    let (_binance_write, mut binance_read) = binance_socket.split();

    // connect coinbase socket
    let coinbase_req_param: String = coinbase_req_param(&ws_details[1].req_param, &pairs);
    let (coinbase_socket, _coinbase_response) =
        connect_async(Url::parse(&coinbase_ws_api).unwrap())
            .await
            .expect("Can't connect coinbase.");

    // subscribe coinbase websockets
    let (mut coinbase_write, mut coinbase_read) = coinbase_socket.split();

    coinbase_write
        .send(Message::Text(coinbase_req_param))
        .await
        .unwrap();

    // connect okex socket
    let okex_req_param: String = okex_req_param(&ws_details[2].req_param, &pairs);
    let (okex_socket, _okex_response) = connect_async(Url::parse(&okex_ws_api).unwrap())
        .await
        .expect("Can't connect okex.");

    // subscribe okex websocket
    let (mut okex_write, mut okex_read) = okex_socket.split();
    okex_write
        .send(Message::Text(okex_req_param))
        .await
        .unwrap();

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();

        if coin.len() == 2 {
            pairs_cache.insert(
                format!(
                    "{}{}",
                    coin[0].to_uppercase(),
                    coin[1].to_uppercase()
                ),
                PairsCache {
                    prices: vec![],
                    aggregate: 0.0,
                },
            );
        }
    }

    let mut interval = time::interval(Duration::from_secs(10));
    let mut interval_flag = false;
    loop {
        tokio::select! {
            msg = binance_read.next() => {
                if let Some(msg) = msg {

                    let msg_binance = match msg.unwrap() {
                        Message::Text(s) => s,
                        _ => { panic!("Error read_message binance"); }
                    };

                    // Serialize binance response
                    let binance_response: BinanceResponse =
                        serde_json::from_str(&msg_binance).expect("Can't parse binance response");

                    // get pair cache and push binance response, name and price
                    (*pairs_cache.get_mut(&binance_response.s).unwrap()).prices.push(PricesPairs{
                        name: ws_details[0].name.to_string(),
                        price: binance_response.c.parse::<f64>().unwrap()
                    });
                }
            },
            msg = coinbase_read.next() => {
                if let Some(msg) = msg {

                    let msg_coinbase = match msg.unwrap() {
                        Message::Text(s) => s,
                        _ => { panic!("Error read_message coinbase");  }
                    };

                    // Serialize coinbase response
                    let coinbase_response: CoinbaseResponse = match serde_json::from_str (&msg_coinbase) {
                        Ok (p) => p,
                        Err (_) => CoinbaseResponse { product_id: "".to_string(), price: "0.0".to_string() },
                    };

                    if coinbase_response.product_id != "" {

                        // get pair cache and push coinbase response, name and price
                        let key = pair_key(&coinbase_response.product_id);
                        (*pairs_cache.get_mut(&key).unwrap()).prices.push(PricesPairs{
                            name: ws_details[1].name.to_string(),
                            price: coinbase_response.price.parse::<f64>().unwrap()
                        });
                    }
                }
            },
            msg = okex_read.next() => {
                if let Some(msg) = msg {

                    let msg_okex = match msg.unwrap() {
                        Message::Text(s) => s,
                        _ => { panic!("Error read_message okex");  }
                    };

                    // Serialize okex response
                    let okex_response: OkexResponse = match serde_json::from_str (&msg_okex) {
                        Ok (p) => p,
                        Err (_) => OkexResponse { data: vec![ ] },
                    };

                    if okex_response.data.len() > 0 {

                        // get pair cache and push okex response, name and price
                        let key = pair_key(&okex_response.data[0].inst_id);
                        (*pairs_cache.get_mut(&key).unwrap()).prices.push(PricesPairs{
                            name: ws_details[2].name.to_string(),
                            price: okex_response.data[0].last.parse::<f64>().unwrap()
                        });
                    }
                }
            }
            _ = interval.tick() => {
                if interval_flag {

                    write_pairs_cache(pairs_cache).await;
                    println!("Cache complete");
                    break;
                }
                interval_flag =true;
            }
        }
    }
}

/// aggregate prices pair wise and write caches in file
async fn write_pairs_cache(pairs: HashMap<String, PairsCache>) {
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
    let content = serde_json::to_string(&pairs_save).unwrap();
    fs::write("exchanges.json", content).expect("Unable to write file")
}

/// Handle Read mode argument and print the aggregate of pairs
fn handle_read_mode() {
    let content = fs::File::open("exchanges.json").expect("Unable to open file");
    let pairs: HashMap<String, PairsCache> = serde_json::from_reader(&content).unwrap();

    for pair in pairs {
        let (key, pari_cache) = pair;

        println!("pair: {:?} -> aggregate: {:?}", key, pari_cache.aggregate);
    }
}

/// binance web socket request url handle for pairs and return
fn binance_req_url(ws_base_url: &str, pairs: &Vec<&str>) -> String {
    let mut binance_ws_api: String = format!("{}/ws", ws_base_url);

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let query: String = format!(
                "/{}{}@ticker",
                coin[0].to_lowercase(),
                coin[1].to_lowercase()
            );
            binance_ws_api.push_str(&query)
        }
    }

    binance_ws_api
}

/// coinbase request parameter for write in socket
fn coinbase_req_param(data: &str, pairs: &Vec<&str>) -> String {
    let mut req_param: CoinbaseReqParam =
        serde_json::from_str(&data).expect("Can't parse coinbase_req_param");

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let product_id: String = format!(
                "{}-{}",
                coin[0].to_uppercase(),
                coin[1].to_uppercase()
            );
            req_param.product_ids.push(product_id);
        }
    }

    serde_json::to_string(&req_param).expect("can't string coinbase_req_param")
}

/// okex request parameter for write in socket
fn okex_req_param(data: &str, pairs: &Vec<&str>) -> String {
    let mut req_param: OkexReqParam =
        serde_json::from_str(&data).expect("Can't parse okex_req_param");

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let inst_id: String = format!(
                "{}-{}",
                coin[0].to_uppercase(),
                coin[1].to_uppercase()
            );
            req_param.args.push(OkexReqParamArg {
                channel: "tickers".to_string(),
                inst_id: inst_id,
            });
        }
    }

    serde_json::to_string(&req_param).expect("can't string okex_req_param")
}

/// remove "-" from the string and return the pairkey
fn pair_key(string: &str) -> String {
    let c_pair: Vec<&str> = string.split("-").collect();
    format!("{}{}", c_pair[0].to_uppercase(), c_pair[1].to_uppercase())
}
