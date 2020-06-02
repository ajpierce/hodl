use base64::{decode, encode};
use chrono::{DateTime, Duration, Utc};
use crypto::hmac::Hmac;
use crypto::mac::Mac; // Must be in scope so we can get the hmac result
use crypto::sha2::Sha256;
use std::{env, thread, time};
use url::form_urlencoded::byte_serialize;

static API_URL: &'static str = "https://api.pro.coinbase.com";
static CANDLES_PER_REQUEST: i64 = 300;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tick {
    trade_id: u64,
    price: String,
    size: String,
    bid: String,
    ask: String,
    volume: String,
    time: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Candlestick(u64, f64, f64, f64, f64, f64);

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ApiResponse {
    ApiError(ApiError),
    Candlesticks(Vec<Candlestick>),
    Tick(Tick),
}

/// The `build_request_headers` function is responsible for creating the headers
/// necessary to make a valid API request to the Coinbase Pro API.
///
/// The headers will be returned as a tuple in the following format:
/// ```
/// (
///     CB-ACCESS-KEY,          // API key as a string
///     CB-ACCESS-SIGN,         // base-64 encoded signature (build in this fn)
///     CB-ACCESS-TIMESTAMP,    // A tiemstamp of our request
///     CB-ACCESS-PASSPHRASE,   // The passphrase created at API key creation time
/// )
/// ```
///
/// Though hyphens are not valid for Rust variable names, they are presented as
/// such above because those are the header names expected by the Coinbase Pro API.
fn build_request_headers(request_path: &str) -> (String, String, i64, String) {
    let key = match env::var("COINBASE_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            println!("Set the COINBASE_API_KEY environment variable to check account balances");
            std::process::exit(1);
        }
    };
    let pass = match env::var("COINBASE_API_PASSPHRASE") {
        Ok(p) => p,
        Err(_) => {
            println!(
                "Set the COINBASE_API_PASSPHRASE environment variable to check account balances"
            );
            std::process::exit(1);
        }
    };
    let hmac_key = decode(&key).expect("Failed to decode base64-coinbase API key");
    let mut hmac = Hmac::new(Sha256::new(), &hmac_key);
    let timestamp = Utc::now().timestamp_millis();
    let message = format!(
        "{timestamp}{method}{request_path}{body}",
        timestamp = timestamp,
        method = "GET",
        request_path = request_path,
        body = "{}"
    );
    hmac.input(message.as_bytes());
    let signature = encode(hmac.result().code());
    (key, signature, timestamp, pass)
}

pub fn print_balances() {
    // pub async fn print_balances() {
    let path = "/accounts";
    let request_url = format!("{api}/{path}", api = API_URL, path = path);
    let headers = build_request_headers(path);
    println!("My headers is: {:?}", headers);
    ()
}

/// Functions for checking the current exchange rate of products on the Coinbase Pro API
pub async fn get_tick(product_id: &str) -> Result<super::ApiResponse, reqwest::Error> {
    let request_url = format!(
        "{api}/products/{product_id}/ticker",
        api = API_URL,
        product_id = product_id
    );
    println!("Making the following request: {}", request_url);

    let tick = reqwest::get(&request_url)
        .await?
        .json::<super::ApiResponse>()
        .await?;
    Ok(tick)
}

fn build_history_url(product_id: &str, start: &str, end: &str, granularity: &str) -> String {
    // We need to urlencode these params
    let s: String = byte_serialize(start.as_bytes()).collect();
    let e: String = byte_serialize(end.as_bytes()).collect();
    let g: String = byte_serialize(granularity.as_bytes()).collect();

    format!(
        "{api}/products/{product_id}/candles?start={s}&end={e}&granularity={g}",
        api = API_URL,
        product_id = product_id,
        s = s,
        e = e,
        g = g
    )
}

fn calc_num_requests(start: &str, end: &str, candle_size: i64) -> i64 {
    let start_date = DateTime::parse_from_rfc3339(start).expect("Failed to parse start date");
    let end_date = DateTime::parse_from_rfc3339(end).expect("Failed to parse end date");
    let duration: i64 = (end_date - start_date).num_seconds();
    let num_requests: i64 = (duration / candle_size) / CANDLES_PER_REQUEST + 1;
    println!(
        "Duration is: {} seconds, so we need to make {} requests",
        duration, num_requests
    );
    num_requests
}

pub async fn get_history(
    product_id: &str,
    start: &str,
    end: &str,
    granularity: &str,
) -> Result<Vec<Candlestick>, reqwest::Error> {
    let candle_size = granularity
        .parse::<i64>()
        .expect("Granularity must be a number (in seconds)");
    let num_requests = calc_num_requests(start, end, candle_size);
    let mut results: Vec<Candlestick> = Vec::new();
    let client = reqwest::Client::builder().user_agent("hodl").build()?;

    for i in 0..num_requests {
        let start_dt = DateTime::parse_from_rfc3339(start).expect("Failed to parse start date");
        let request_start = start_dt + Duration::seconds(i * candle_size * CANDLES_PER_REQUEST);
        let request_end = request_start + Duration::seconds(candle_size * CANDLES_PER_REQUEST);
        let request_url = build_history_url(
            product_id,
            &request_start.to_string(),
            &request_end.to_string(),
            granularity,
        );

        if let ApiResponse::Candlesticks(v) = client
            .get(&request_url)
            .send()
            .await?
            .json::<super::ApiResponse>()
            .await?
        {
            results.extend(v);
        };

        // API is rate limited to 1 request per second
        thread::sleep(time::Duration::from_millis(1000));
    }
    Ok(results)
}

#[cfg(test)]
mod tests;
