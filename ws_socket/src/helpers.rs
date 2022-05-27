use std::collections::HashMap;

use crate::types::{
    BinanceReqParam, CoinbaseReqParam, OkexReqParam, OkexReqParamArg, PairsCache, PricesPairs,
    ResponseEnum, SocketType, WSResult, WebSocketConfig,
};
use serde_json::Value;

/// binance web socket request url handle for pairs and return
pub fn binance_req_url(ws_base_url: &str, pairs: &Vec<String>) -> String {
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

/// create request parameters for Binance, Coinbase and Okex
pub fn create_req_params(
    socket_type: SocketType,
    data: &Value,
    pairs: &Vec<String>,
) -> WSResult<String> {
    let mut params = vec![];
    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let param = match socket_type {
                SocketType::Binance => {
                    format!("{}{}@ticker", coin[0].to_uppercase(), coin[1].to_uppercase())
                }
                SocketType::Okex => {
                    format!("{}-{}", coin[0].to_uppercase(), coin[1].to_uppercase())
                }
                SocketType::Coinbase => {
                    format!("{}-{}", coin[0].to_uppercase(), coin[1].to_uppercase())
                }
            };
            params.push(param);
        }
    }
    get_request_param_string(socket_type, data, params)
}

/// get request parameter in string
pub fn get_request_param_string(
    socket_type: SocketType,
    data: &Value,
    params: Vec<String>,
) -> WSResult<String> {
    match socket_type {
        SocketType::Binance => {
            let mut req_param: BinanceReqParam = serde_json::from_value(data.clone())?;

            params.iter().for_each(|param| {
                req_param.params.push(param.clone());
            });
            Ok(serde_json::to_string(&req_param)?)
        }
        SocketType::Okex => {
            let mut req_param: OkexReqParam =
                serde_json::from_value(data.clone()).expect("Can't parse okex_req_param");
            params.iter().for_each(|param| {
                req_param.args.push(OkexReqParamArg {
                    channel: "tickers".to_string(),
                    inst_id: param.clone(),
                });
            });
            Ok(serde_json::to_string(&req_param)?)
        }
        SocketType::Coinbase => {
            let mut req_param: CoinbaseReqParam =
                serde_json::from_value(data.clone()).expect("Can't parse coinbase_req_param");
            params.iter().for_each(|param| {
                req_param.product_ids.push(param.clone());
            });
            Ok(serde_json::to_string(&req_param)?)
        }
    }
}

/// remove "-" from the string and return the pairkey
pub fn pair_key(string: &str) -> String {
    let c_pair: Vec<&str> = string.split("-").collect();
    format!("{}{}", c_pair[0].to_uppercase(), c_pair[1].to_uppercase())
}

/// common handler fror socket response
pub fn handle_response(
    pairs_cache: &mut HashMap<String, PairsCache>,
    ws_details: &Vec<WebSocketConfig>,
    response: ResponseEnum,
) -> WSResult<()> {
    match response {
        ResponseEnum::Binance(binance_response) => {
            if binance_response.s != "" {
                let price = binance_response.c.parse::<f64>()?;
                update_price_cache(
                    pairs_cache,
                    binance_response.s.to_string(),
                    ws_details[0].name.to_string(),
                    price,
                );
            }
        }
        ResponseEnum::Okex(okex_response) => {
            if okex_response.data.len() > 0 {
                let price = okex_response.data[0].last.parse::<f64>()?;

                // get pair cache and push okex response, name and price
                let key = pair_key(&okex_response.data[0].inst_id);
                update_price_cache(pairs_cache, key, ws_details[2].name.to_string(), price);
            }
        }
        ResponseEnum::Coinbase(coinbase_response) => {
            if coinbase_response.product_id != "" {
                let price = coinbase_response.price.parse::<f64>()?;
                // get pair cache and push coinbase response, name and price
                let key = pair_key(&coinbase_response.product_id);
                update_price_cache(pairs_cache, key, ws_details[1].name.to_string(), price);
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
