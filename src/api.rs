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
pub mod ticks {

    pub async fn get_tick(product_id: &str) -> Result<super::ApiResponse, reqwest::Error> {
        let request_url = format!(
            "{api}/products/{product_id}/ticker",
            api = super::API_URL,
            product_id = product_id
        );
        println!("Making the following request: {}", request_url);

        let tick = reqwest::get(&request_url)
            .await?
            .json::<super::ApiResponse>()
            .await?;
        Ok(tick)
    }
}
