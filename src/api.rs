use reqwest::Error;
use serde::{Debug, Deserialize, Serialize};

/// Functions for checking the current exchange rate of products on the Coinbase Pro API
pub mod check {
    static API_URL: &'static str = "https://api.pro.coinbase.com";

    #[derive(Deserialize, Debug)]
    struct Tick {
        trade_id: u32,
        price: String,
        size: String,
        bid: String,
        ask: String,
        volume: String,
        time: String,
    }

    pub async fn get_tick(product_id: &str) -> Result<(), reqwest::Error> {
        let request_url = format!(
            "{api}/products/{product_id}/ticker",
            api = API_URL,
            product_id = product_id
        );
        println!("Making the following request: {}", request_url);
        let tick = reqwest::get(&request_url).await?.json::<Tick>().await?;
        println!("{:?}", tick);
        Ok(())
    }
}
