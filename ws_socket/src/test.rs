use crate::{
    check_pairs,
    helpers::{self, create_req_params, handle_response},
    insert_pairs,
    types::{
        BinanceResponse, CoinbaseResponse, OkexResponse, OkexResponseChild, PairsCache,
        PricesPairs, ResponseEnum,
        SocketType::{Binance, Coinbase, Okex},
        WSResult, WebSocketConfig,
    },
};
use std::{
    collections::HashMap,
    fs::{self, File},
};

#[test]
/// check valid pairs for single and multiple
fn check_valid_pairs() {
    let signle = check_pairs("btc_usdt");
    assert!(signle);
    let multiple = check_pairs("btc_usdt,eth_usdt");
    assert!(multiple);
}

#[test]
/// check invalid pairs for single and multiple
fn check_invalid_pairs_() {
    let single = check_pairs("btcusdt");
    assert!(!single);
    let multiple = check_pairs("btcusdt,eth_usdt");
    assert!(!multiple);
}

#[test]
/// check binance url for single and multiple pairs
fn check_binance_url() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;

    let single_pairs = vec!["btc_usdt".to_string()];
    let single_pair_url = helpers::binance_req_url(&ws_details[0].ws_base_url, &single_pairs);

    assert_eq!(
        single_pair_url,
        "wss://stream.binance.com:9443/ws/btcusdt@ticker"
    );

    let multiple_pairs = vec!["btc_usdt".to_string(), "eth_usdt".to_string()];
    let multiple_pair_url = helpers::binance_req_url(&ws_details[0].ws_base_url, &multiple_pairs);

    assert_eq!(
        multiple_pair_url,
        "wss://stream.binance.com:9443/ws/btcusdt@ticker/ethusdt@ticker"
    );

    Ok(())
}

#[test]
/// check binance subscription parameter with single and multiple pairs
fn check_binance_subscribe_param() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;
    let single_pair = vec!["btc_usdt".to_string()];

    let sin_res_pair = create_req_params(Binance, &ws_details[0].req_param, &single_pair)?;

    assert_eq!(
        sin_res_pair,
        "{\"method\":\"SUBSCRIBE\",\"params\":[\"BTCUSDT@ticker\"],\"id\":1}".to_string()
    );

    let multiple_pair = vec!["btc_usdt".to_string(), "eth_usdt".to_string()];

    let mul_res_pair = create_req_params(Binance, &ws_details[0].req_param, &multiple_pair)?;

    assert_eq!(
        mul_res_pair,
        "{\"method\":\"SUBSCRIBE\",\"params\":[\"BTCUSDT@ticker\",\"ETHUSDT@ticker\"],\"id\":1}"
            .to_string()
    );
    Ok(())
}

#[test]
/// check coinbase subscription parameter with single and multiple pairs
fn check_coinbase_subscribe_param() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;
    let single_pair = vec!["btc_usdt".to_string()];

    let sin_res_pair = create_req_params(Coinbase, &ws_details[1].req_param, &single_pair)?;

    assert_eq!(
        sin_res_pair,
        "{\"type\":\"subscribe\",\"channels\":[\"ticker\"],\"product_ids\":[\"BTC-USDT\"]}"
            .to_string()
    );

    let multiple_pair = vec!["btc_usdt".to_string(), "eth_usdt".to_string()];

    let mul_res_pair = create_req_params(Coinbase, &ws_details[1].req_param, &multiple_pair)?;

    assert_eq!(
        mul_res_pair,
        "{\"type\":\"subscribe\",\"channels\":[\"ticker\"],\"product_ids\":[\"BTC-USDT\",\"ETH-USDT\"]}".to_string()
    );
    Ok(())
}

#[test]
/// check okex subscription parameter with single and multiple pairs
fn check_okex_subscribe_param() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;
    let single_pair = vec!["btc_usdt".to_string()];

    let sin_res_pair = create_req_params(Okex, &ws_details[2].req_param, &single_pair)?;

    assert_eq!(
        sin_res_pair,
        "{\"op\":\"subscribe\",\"args\":[{\"channel\":\"tickers\",\"instId\":\"BTC-USDT\"}]}"
            .to_string()
    );

    let multiple_pair = vec!["btc_usdt".to_string(), "eth_usdt".to_string()];

    let mul_res_pair = create_req_params(Okex, &ws_details[2].req_param, &multiple_pair)?;

    assert_eq!(
        mul_res_pair,
        "{\"op\":\"subscribe\",\"args\":[{\"channel\":\"tickers\",\"instId\":\"BTC-USDT\"},{\"channel\":\"tickers\",\"instId\":\"ETH-USDT\"}]}".to_string()
    );
    Ok(())
}

#[test]
/// check binance response
fn check_binance_response() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;
    let pairs = vec!["btc_usdt".to_string()];

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    insert_pairs(pairs, &mut pairs_cache);

    let binance_response = BinanceResponse {
        s: "BTCUSDT".to_string(),
        c: "28933.33".to_string(),
    };

    handle_response(
        &mut pairs_cache,
        &ws_details,
        ResponseEnum::Binance(binance_response),
    )?;

    let mut expect_response: HashMap<String, PairsCache> = HashMap::new();
    expect_response.insert(
        "BTCUSDT".to_string(),
        PairsCache {
            aggregate: 0.0,
            prices: vec![PricesPairs {
                name: "binance".to_string(),
                price: 28_933.33,
            }],
        },
    );

    assert_eq!(expect_response, pairs_cache);

    Ok(())
}

#[test]
/// check coinbase response
fn check_coinbase_response() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;
    let pairs = vec!["btc_usdt".to_string()];

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    insert_pairs(pairs, &mut pairs_cache);

    let coinbase_response = CoinbaseResponse {
        price: "28933.33".to_string(),
        product_id: "btc-usdt".to_string(),
    };

    handle_response(
        &mut pairs_cache,
        &ws_details,
        ResponseEnum::Coinbase(coinbase_response),
    )?;

    let mut expect_response: HashMap<String, PairsCache> = HashMap::new();
    expect_response.insert(
        "BTCUSDT".to_string(),
        PairsCache {
            aggregate: 0.0,
            prices: vec![PricesPairs {
                name: "coinbase".to_string(),
                price: 28_933.33,
            }],
        },
    );

    assert_eq!(expect_response, pairs_cache);

    Ok(())
}

#[test]
/// check okex response
fn check_okex_response() -> WSResult<()> {
    let ws_details_file: File = fs::File::open("../ws_details.json")?;
    let ws_details: Vec<WebSocketConfig> = serde_json::from_reader(&ws_details_file)?;
    let pairs = vec!["btc_usdt".to_string()];

    let mut pairs_cache: HashMap<String, PairsCache> = HashMap::new();

    insert_pairs(pairs, &mut pairs_cache);

    let okex_response = OkexResponse {
        data: vec![OkexResponseChild {
            inst_id: "btc-usdt".to_string(),
            last: "28933.33".to_string(),
        }],
    };

    handle_response(
        &mut pairs_cache,
        &ws_details,
        ResponseEnum::Okex(okex_response),
    )?;

    let mut expect_response: HashMap<String, PairsCache> = HashMap::new();
    expect_response.insert(
        "BTCUSDT".to_string(),
        PairsCache {
            aggregate: 0.0,
            prices: vec![PricesPairs {
                name: "okx".to_string(),
                price: 28_933.33,
            }],
        },
    );

    assert_eq!(expect_response, pairs_cache);

    Ok(())
}
