extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_derive;

use clap::{App, Arg, SubCommand};
use std::env;

pub mod api;
use api::ticks::get_tick;
use api::ApiResponse;

static DEFAULT_PRODUCT: &'static str = "BTC-USD";

#[tokio::main]
async fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(SubCommand::with_name("buy").about("Purchase BTC with USD"))
        .subcommand(
            SubCommand::with_name("tick")
                .about("Print the latest tick (current price/volume) for the given product-id")
                .version(env!("CARGO_PKG_VERSION"))
                .arg(
                    Arg::with_name("product-id")
                        .help("The product-id to check. Defaults to BTC-USD")
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("lookback").about("Fetch historical data"))
        .subcommand(SubCommand::with_name("transfer").about("Transfer USD to Coinbase Pro"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("tick") {
        let product = matches.value_of("product-id").unwrap_or(DEFAULT_PRODUCT);
        let response: ApiResponse = get_tick(&product).await.expect("API request failed");
        println!("Tick data for {}: {:#?}", product, response);
        return ();
    }

    println!("Invalid input. Type help for more information");
    return ();
}
