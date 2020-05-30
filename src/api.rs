use chrono::format::ParseError;
use chrono::{DateTime, Duration};
use url::form_urlencoded::byte_serialize;

static API_URL: &'static str = "https://api.pro.coinbase.com";

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
pub struct ApiError {
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Tick(Tick),
    ApiError(ApiError),
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

// pub async fn get_history(
pub fn get_history(
    product_id: &str,
    start: &str,
    end: &str,
    granularity: &str,
    // ) -> Result<super::ApiResponse, reqwest::Error> {
) -> String {
    let start_date = DateTime::parse_from_rfc3339(start).expect("Failed to parse start date");
    let end_date = DateTime::parse_from_rfc3339(end).expect("Failed to parse end date");
    let duration: Duration = end_date - start_date;
    println!("Duration is: {:?}", duration);
    let request_url = build_history_url(product_id, start, end, granularity);
    println!("My encoded URL is: {}", request_url);
    request_url
}
