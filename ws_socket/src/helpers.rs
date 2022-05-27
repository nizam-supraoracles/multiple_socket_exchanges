use crate::types::{
    BinanceReqParam, CoinbaseReqParam, OkexReqParam, OkexReqParamArg, RequestParamType, WSResult,
};
use serde_json::Value;

/// binance web socket request url handle for pairs and return
pub fn binance_req_url(ws_base_url: &str, pairs: &Vec<&str>) -> String {
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
    req_type: RequestParamType,
    data: &Value,
    pairs: &Vec<&str>,
) -> WSResult<String> {
    let mut params = vec![];
    for pair in pairs {
        let coin: Vec<&str> = pair.split("_").collect();
        if coin.len() == 2 {
            let param = match req_type {
                RequestParamType::Binance => {
                    format!("{}{}ticker", coin[0].to_uppercase(), coin[1].to_uppercase())
                }
                RequestParamType::Okex => {
                    format!("{}-{}", coin[0].to_uppercase(), coin[1].to_uppercase())
                }
                RequestParamType::Coinbase => {
                    format!("{}-{}", coin[0].to_uppercase(), coin[1].to_uppercase())
                }
            };
            params.push(param);
        }
    }
    get_request_param_string(req_type, data, params)
}

/// get request parameter in string
pub fn get_request_param_string(
    req_type: RequestParamType,
    data: &Value,
    params: Vec<String>,
) -> WSResult<String> {
    match req_type {
        RequestParamType::Binance => {
            let mut req_param: BinanceReqParam = serde_json::from_value(data.clone())?;

            params.iter().for_each(|param| {
                req_param.params.push(param.clone());
            });
            Ok(serde_json::to_string(&req_param)?)
        }
        RequestParamType::Okex => {
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
        RequestParamType::Coinbase => {
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
