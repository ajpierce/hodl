use base64::{decode, encode};
use chrono::{DateTime, Duration};
use crypto::hmac::Hmac;
use crypto::mac::Mac; // Must be in scope so we can get the hmac result
use crypto::sha2::Sha256;
use csv::Writer;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde_json::Value;
use std::time::SystemTime;
use std::{env, io, thread, time};
use url::form_urlencoded::byte_serialize;

static API_URL: &str = "https://api.pro.coinbase.com";
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
pub struct Account {
    id: String,
    currency: String,
    balance: String,
    available: String,
    hold: String,
    profile_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DepositResponse {
    id: String,
    amount: String,
    currency: String,
    payout_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    id: String,
    size: String,
    price: String,
    side: String,
    status: String,
    product_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentMethod {
    id: String,
    #[serde(rename = "type")]
    type_name: String,
    name: String,
    currency: String,
    primary_buy: bool,
    primary_sell: bool,
    allow_buy: bool,
    allow_sell: bool,
    allow_deposit: bool,
    allow_withdraw: bool,
    limits: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Account(Account),
    Accounts(Vec<Account>),
    ApiError(ApiError),
    DepositResponse(DepositResponse),
    Candlesticks(Vec<Candlestick>),
    Order(Order),
    Orders(Vec<Order>),
    PaymentMethod(PaymentMethod),
    PaymentMethods(Vec<PaymentMethod>),
    Tick(Tick),
}

/// The `build_request_headers` function is responsible for creating the headers
/// necessary to make a valid API request to the Coinbase Pro API.
///
/// The headers will be returned as a reqwest::header::HeaderMap containing the following headers:
/// ```
/// CB-ACCESS-KEY          // API key as a string
/// CB-ACCESS-SIGN         // base-64 encoded signature (build in this fn)
/// CB-ACCESS-TIMESTAMP    // A tiemstamp of our request
/// CB-ACCESS-PASSPHRASE   // The passphrase created at API key creation time
/// ```
fn build_request_headers(request_path: &str, method: &str, body: &str) -> Option<HeaderMap> {
    let key = match env::var("COINBASE_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("Set the COINBASE_API_KEY environment variable to make this request");
            std::process::exit(1);
        }
    };
    let secret = match env::var("COINBASE_API_SECRET") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Set the COINBASE_API_SECRET environment variable to make this request");
            std::process::exit(1);
        }
    };
    let pass = match env::var("COINBASE_API_PASSPHRASE") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Set the COINBASE_API_PASSPHRASE environment variable to make this request");
            std::process::exit(1);
        }
    };
    let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs().to_string(),
        Err(_) => {
            eprintln!("Current system time falls before the epoch; cannot make valid request");
            std::process::exit(1);
        }
    };

    let message = format!("{}{}{}{}", timestamp, method, request_path, body);
    let hmac_key = decode(&secret).expect("Failed to base64 decode Coinbase API secret");
    let mut hmac = Hmac::new(Sha256::new(), &hmac_key);
    hmac.input(message.as_bytes());
    let signature = encode(hmac.result().code());

    let mut headers = HeaderMap::new();
    headers.append("CB-ACCESS-KEY", key.parse().unwrap());
    headers.append("CB-ACCESS-SIGN", signature.parse().unwrap());
    headers.append("CB-ACCESS-TIMESTAMP", timestamp.parse().unwrap());
    headers.append("CB-ACCESS-PASSPHRASE", pass.parse().unwrap());
    Some(headers)
}

async fn get_request(path: &str) -> Result<ApiResponse, reqwest::Error> {
    let headers = build_request_headers(path, "GET", "").unwrap();
    let client = Client::builder().user_agent("hodl").build()?;
    let request_url = format!("{api}{path}", api = API_URL, path = path);
    let response = client.get(&request_url).headers(headers).send().await?;
    let data = response.json::<ApiResponse>().await?;
    Ok(data)
}

async fn post_request(
    path: &str,
    body: String,
    json: Value,
) -> Result<ApiResponse, reqwest::Error> {
    let headers = build_request_headers(path, "POST", &body[..]).unwrap();
    let client = Client::builder().user_agent("hodl").build()?;
    let request_url = format!("{api}{path}", api = API_URL, path = path);
    let response = client
        .post(&request_url)
        .json(&json)
        .headers(headers)
        .send()
        .await?;
    let data = response.json::<ApiResponse>().await?;
    Ok(data)
}

pub async fn print_balance(currency: Option<&str>) {
    let path = "/accounts";
    let accounts = match get_request(path).await.unwrap() {
        ApiResponse::Accounts(a) => a,
        ApiResponse::ApiError(e) => {
            eprintln!("Error message from Coinbase API: {:?}", e.message);
            std::process::exit(1);
        }
        _ => {
            eprintln!("Failed to request account information");
            std::process::exit(1);
        }
    };

    if let Some(c) = currency {
        let account = accounts.iter().find(|x| x.currency == c);
        if let Some(a) = account {
            println!("{:#?}", a);
            return;
        } else {
            eprintln!("No account found containing {}", c);
            std::process::exit(1);
        }
    }
    println!("{:#?}", accounts);
}

pub async fn print_payment_methods() {
    let path = "/payment-methods";
    let response = match get_request(path).await.unwrap() {
        ApiResponse::PaymentMethods(a) => a,
        ApiResponse::ApiError(e) => {
            eprintln!("Error message from Coinbase API: {:?}", e.message);
            std::process::exit(1);
        }
        _ => {
            eprintln!("Failed to fetch payment methods");
            std::process::exit(1);
        }
    };
    println!("Payment methods: {:#?}", response);
}

pub async fn make_deposit(amount: &f64) -> Option<DepositResponse> {
    let bank_id = match env::var("BANK_ID") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("You must set the BANK_ID environment variable to make deposits.");
            eprintln!("Looking for your bank id? Use the 'payment-methods' command");
            std::process::exit(1);
        }
    };
    let payload = format!(
        r#"{{
    "amount": {amount},
    "currency": "USD",
    "payment_method_id": "{bank_id}"
}}"#,
        amount = amount,
        bank_id = bank_id
    );
    let json: Value = match serde_json::from_str(&payload) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to parse the following as JSON:");
            eprintln!("{}", payload);
            eprintln!("{:?}", e);
            return None;
        }
    };
    let path = "/deposits/payment-method";
    let body: String = json.to_string();
    match post_request(path, body, json).await.unwrap() {
        ApiResponse::DepositResponse(r) => Some(r),
        ApiResponse::ApiError(e) => {
            eprintln!("Deposit failed; error from Coinbase API: {:?}", e.message);
            None
        }
        _ => {
            eprintln!("Deposit failed for unknown reason");
            None
        }
    }
}

pub async fn place_order(amount: &f64, currency: &str) -> Option<Order> {
    let product_id = &format!("{}-USD", currency)[..];
    let payload = format!(
        r#"{{
    "type": "market",
    "side": "buy",
    "product_id": "{product_id}",
    "funds": {amount}
}}"#,
        amount = amount,
        product_id = product_id
    );
    let json: Value = match serde_json::from_str(&payload) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to parse the following as JSON:");
            eprintln!("{}", payload);
            eprintln!("{:?}", e);
            return None;
        }
    };
    let path = "/orders";
    let body: String = json.to_string();
    match post_request(path, body, json).await.unwrap() {
        ApiResponse::Order(r) => Some(r),
        ApiResponse::ApiError(e) => {
            eprintln!("Purcahse failed; error from Coinbase API: {:?}", e.message);
            None
        }
        _ => {
            eprintln!("Something unexpected happened; log into Coinbase and check");
            None
        }
    }
}

pub async fn list_orders(product_id: Option<&str>) -> Option<Vec<Order>> {
    let mut path = String::from("/orders");
    if let Some(pid) = product_id {
        path = format!("{}?product_id={}", path, pid);
    }
    let response = get_request(&path[..]).await.unwrap();
    match response {
        ApiResponse::Orders(o) => Some(o),
        ApiResponse::ApiError(e) => {
            eprintln!("Failed to fetch order information {:?}", e.message);
            None
        }
        _ => Some(Vec::new()),
    }
}

/// Check the current exchange rate of products on the Coinbase Pro API
pub async fn get_tick(product_id: &str) -> Result<ApiResponse, reqwest::Error> {
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
    num_requests
}

pub async fn get_history(
    product_id: &str,
    start: &str,
    end: &str,
    granularity: &str,
    mut writer: Writer<io::Stdout>,
) -> Result<(), reqwest::Error> {
    let candle_size = granularity
        .parse::<i64>()
        .expect("Granularity must be a number (in seconds)");
    let num_requests = calc_num_requests(start, end, candle_size);
    let client = Client::builder().user_agent("hodl").build()?;

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

        if let ApiResponse::Candlesticks(candlesticks) = client
            .get(&request_url)
            .send()
            .await?
            .json::<super::ApiResponse>()
            .await?
        {
            for c in candlesticks {
                writer
                    .serialize(c)
                    .expect("Failed to write candlestick to CSV");
                writer.flush().expect("Failed to flush CSV writer");
            }
        };

        // API is rate limited to 1 request per second
        thread::sleep(time::Duration::from_millis(1000));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
