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
            if check_pairs(&pairs) {
                handle_cach_mode(pairs.split(",").collect()).await;
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
fn check_pairs(pairs: &str) -> bool {
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
async fn handle_cach_mode(pairs: Vec<&str>) {
    // read json file of web socket urls
    let ws_details_file: File =
        fs::File::open("ws_details.json").expect("Unable to open ws_details.json file ");

    let ws_details: Vec<WebSocket> = serde_json::from_reader(&ws_details_file)
        .expect("There is some issue in ws_details.json file format");

    let binance_ws_api: String = binance_req_url(&ws_details[0].ws_base_url, &pairs);
    let coinbase_ws_api: &String = &ws_details[1].ws_base_url;
    let okex_ws_api: &String = &ws_details[2].ws_base_url;

    let binance_url_parse = Url::parse(&binance_ws_api).expect("Invalid binance socket url");
    // connect binance socket
    let (binance_socket, _binance_response) = connect_async(binance_url_parse)
        .await
        .expect("Can't connect binance.");

    let (mut _binance_write, mut binance_read) = binance_socket.split();

    let binance_req_param: String = binance_req_param(&ws_details[0].req_param, &pairs);
    _binance_write
        .send(Message::Text(binance_req_param))
        .await
        .expect("There is some issue while write data in binance");

    let coinbase_url_parse = Url::parse(&coinbase_ws_api).expect("Invalid coinbase socket url");
    // connect coinbase socket
    let coinbase_req_param: String = coinbase_req_param(&ws_details[1].req_param, &pairs);
    let (coinbase_socket, _coinbase_response) = connect_async(coinbase_url_parse)
        .await
        .expect("Can't connect coinbase.");

    // subscribe coinbase websockets
    let (mut coinbase_write, mut coinbase_read) = coinbase_socket.split();

    coinbase_write
        .send(Message::Text(coinbase_req_param))
        .await
        .expect("There is some issue while write data in coninbase");

    let okex_url_parse = Url::parse(&okex_ws_api).expect("Invalid okex socket url");
    // connect okex socket
    let okex_req_param: String = okex_req_param(&ws_details[2].req_param, &pairs);
    let (okex_socket, _okex_response) = connect_async(okex_url_parse)
        .await
        .expect("Can't connect okex.");

    // subscribe okex websocket
    let (mut okex_write, mut okex_read) = okex_socket.split();
    okex_write
        .send(Message::Text(okex_req_param))
        .await
        .expect("There is some issue while write data in okex");

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();

        if coin.len() == 2 {
            pairs_cache.insert(
                format!("{}{}", coin[0].to_uppercase(), coin[1].to_uppercase()),
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

                    let msg_binance = match msg {
                        Ok(s) => {
                            match s {
                                Message::Text(s) => s,
                                _ => {
                                    eprintln!("Error read_message binance");
                                    std::process::exit(1);
                                }
                            }
                        },
                        Err(_err) => {
                            eprintln!("Error read_message binance");
                            std::process::exit(1);
                        }
                    };

                    let binance_error_check: serde_json::Value =  serde_json::from_str (&msg_binance)
                        .expect("Binance response format not valids");

                    if binance_error_check["result"] == "error" {
                        eprintln!("Binance: {:?}", binance_error_check);
                        break;
                    }

                    // Serialize binance response
                    let binance_response: BinanceResponse = match  serde_json::from_str(&msg_binance) {
                        Ok(p) => p,
                        Err(_) => BinanceResponse { s: "".to_string(), c: "0.0".to_string() }
                    };

                    if binance_response.s != "" {
                        let price = binance_response.c.parse::<f64>().expect("There is some issue while parse price in binance");

                        (*pairs_cache.get_mut(&binance_response.s).expect("There is some issue with get pairs_cache hashmap"))
                        .prices.push(PricesPairs{
                            name: ws_details[0].name.to_string(),
                            price: price
                        });
                    }

                }
            },
            msg = coinbase_read.next() => {
                if let Some(msg) = msg {

                    let msg_coinbase = match msg {
                        Ok(s) => {
                            match s {
                                Message::Text(s) => s,
                                _ => {
                                    eprintln!("Error read_message coinbase");
                                    std::process::exit(1);
                                }
                            }
                        },
                        Err(_err) => {
                            eprintln!("Error read_message coinbase");
                            std::process::exit(1);
                        }
                    };

                    let coinbase_error_check: serde_json::Value =  serde_json::from_str (&msg_coinbase)
                        .expect("Coinbase response format not valids");

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

                        let price = coinbase_response.price.parse::<f64>().expect("There is some issue while parse price in coinbase");

                        // get pair cache and push coinbase response, name and price
                        let key = pair_key(&coinbase_response.product_id);
                        (*pairs_cache.get_mut(&key).expect("There is some issue with get pairs_cache hashmap"))
                        .prices.push(PricesPairs{
                            name: ws_details[1].name.to_string(),
                            price: price
                        });
                    }
                }
            },
            msg = okex_read.next() => {
                if let Some(msg) = msg {

                    let msg_okex = match msg {
                        Ok(s) => {
                            match s {
                                Message::Text(s) => s,
                                _ => {
                                    eprintln!("Error read_message okex");
                                    std::process::exit(1);
                                }
                            }
                        },
                        Err(_err) => {
                            eprintln!("Error read_message okex");
                            std::process::exit(1);
                        }
                    };

                    let okex_error_check: serde_json::Value =  serde_json::from_str (&msg_okex)
                        .expect("Okex response format not valids");

                    if okex_error_check["event"] == "error" {
                        eprintln!("Okex: {:?}", okex_error_check["msg"]);
                        break;
                    }

                    // Serialize okex response
                    let okex_response: OkexResponse = match serde_json::from_str (&msg_okex) {
                        Ok (p) => p,
                        Err (_) => OkexResponse { data: vec![ ] },
                    };

                    if okex_response.data.len() > 0 {

                        let price = okex_response.data[0].last.parse::<f64>().expect("There is some issue while parse price in okex");

                        // get pair cache and push okex response, name and price
                        let key = pair_key(&okex_response.data[0].inst_id);
                        (*pairs_cache.get_mut(&key).expect("There is some issue with get pairs_cache hashmap"))
                        .prices.push(PricesPairs{
                            name: ws_details[2].name.to_string(),
                            price: price
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
    let content =
        serde_json::to_string(&pairs_save).expect("There is some issue while ro_string pairs");
    fs::write("exchanges.json", content).expect("Unable to write file")
}

/// Handle Read mode argument and print the aggregate of pairs
fn handle_read_mode() {
    let content = fs::File::open("exchanges.json").expect("Unable to open file");
    let pairs: HashMap<String, PairsCache> = serde_json::from_reader(&content)
        .expect("There is some issue in cache format, please re-cache pairs again");

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

fn binance_req_param(data: &str, pairs: &Vec<&str>) -> String {
    let mut req_param: BinanceReqParam =
        serde_json::from_str(&data).expect("Can't parse coinbase_req_param");

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let params: String =
                format!("{}{}ticker", coin[0].to_uppercase(), coin[1].to_uppercase());
            req_param.params.push(params);
        }
    }

    serde_json::to_string(&req_param).expect("can't string coinbase_req_param")
}

/// coinbase request parameter for write in socket
fn coinbase_req_param(data: &str, pairs: &Vec<&str>) -> String {
    let mut req_param: CoinbaseReqParam =
        serde_json::from_str(&data).expect("Can't parse coinbase_req_param");

    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let product_id: String =
                format!("{}-{}", coin[0].to_uppercase(), coin[1].to_uppercase());
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
            let inst_id: String = format!("{}-{}", coin[0].to_uppercase(), coin[1].to_uppercase());
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
