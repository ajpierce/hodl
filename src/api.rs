/// Functions for checking the current exchange rate of products on the Coinbase Pro API
pub mod check {
    static API_URL: &'static str = "https://api.pro.coinbase.com";

    #[derive(Deserialize, Debug)]
    pub struct Tick {
        trade_id: u32,
        message: String,
        price: String,
        size: String,
        bid: String,
        ask: String,
        volume: String,
        time: String,
    }

    pub struct ApiErr {
        message: String,
    }

    pub async fn get_tick(product_id: &str) -> Result<Tick, reqwest::Error> {
        let request_url = format!(
            "{api}/products/{product_id}/ticker",
            api = API_URL,
            product_id = product_id
        );
        println!("Making the following request: {}", request_url);
        let response = await reqwest::get(&request_url).await?.json() {

        };
        let tick = response.json::<Tick>().await?;
        // println!("{:?}", tick);
        Ok(tick)
    }
}
