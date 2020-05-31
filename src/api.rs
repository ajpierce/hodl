use chrono::{DateTime, Duration};
use std::{thread, time};
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
    // results.flatten();
    /*
    println!(
        "Succesfully completed {} requests, have {} results",
        num_requests,
        results.len()
    );
    */
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_num_requets() {
        assert_eq!(
            calc_num_requests(
                "2020-01-01T00:00:00-04:00",
                "2020-01-01T00:00:01-04:00",
                300
            ),
            1
        );
        assert_eq!(
            calc_num_requests(
                "2020-01-01T00:00:00-04:00",
                "2020-01-01T00:05:00-04:00",
                300
            ),
            1
        );
        assert_eq!(
            calc_num_requests(
                "2020-01-01T00:00:00-04:00",
                "2020-01-02T00:23:55-04:00",
                300
            ),
            1
        );
        assert_eq!(
            calc_num_requests(
                "2020-01-01T00:00:00-04:00",
                "2020-01-02T01:00:00-04:00",
                300
            ),
            2
        );
    }
}
