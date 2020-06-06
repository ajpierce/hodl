extern crate base64;
extern crate chrono;
extern crate clap;
extern crate crypto;
extern crate csv;
extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_derive;
extern crate url;

use clap::{App, Arg, SubCommand};
use csv::Writer;
use std::{env, io};

pub mod api;
use api::{
    get_history, get_tick, make_deposit, print_balances, print_payment_methods, ApiResponse,
};

static DEFAULT_PRODUCT: &'static str = "BTC-USD";

#[tokio::main]
async fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(SubCommand::with_name("balance").about("Check balances of all accounts"))
        .subcommand(SubCommand::with_name("payment-methods").about(
            "Get information about which payment methods (bank accounts) are available to you",
        ))
        .subcommand(SubCommand::with_name("buy").about("Purchase BTC with USD"))
        .subcommand(
            SubCommand::with_name("tick")
                .about("Print the latest tick (current price/volume) for the given product-id")
                .arg(
                    Arg::with_name("product-id")
                        .help("The product-id to check. Defaults to BTC-USD")
                        .default_value(DEFAULT_PRODUCT)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("history")
                .about("Fetch historical data")
                .arg(
                    Arg::with_name("product-id")
                        .help("The product-id to check. Defaults to BTC-USD")
                        .default_value(DEFAULT_PRODUCT)
                        .index(1),
                )
                .arg(
                    Arg::with_name("start")
                        .help("Start time in ISO 8601")
                        .index(2),
                )
                .arg(Arg::with_name("end").help("End time in ISO 8601").index(3))
                .arg(
                    Arg::with_name("granularity")
                        .help("Desired timeslice in seconds")
                        .index(4),
                ),
        )
        .subcommand(
            SubCommand::with_name("deposit")
                .about("Deposit USD into Coinbase Pro")
                .arg(
                    Arg::with_name("amount")
                        .help("The amount of USD to deposit into Coinbase Pro")
                        .index(1),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("tick") {
        let product = matches.value_of("product-id").unwrap_or(DEFAULT_PRODUCT);
        let response: ApiResponse = get_tick(&product).await.expect("API request failed");
        println!("Tick data for {}: {:#?}", product, response);
        return ();
    }

    if let Some(matches) = matches.subcommand_matches("history") {
        let product = matches.value_of("product-id").unwrap_or(DEFAULT_PRODUCT);
        let start = matches.value_of("start").unwrap_or("");
        let end = matches.value_of("end").unwrap_or("");
        let granularity = matches.value_of("granularity").unwrap_or("");

        let mut wtr = Writer::from_writer(io::stdout());
        wtr.write_record(&["time", "low", "high", "open", "close", "volume"])
            .expect("Failed to write CSV header");
        wtr.flush().expect("Failed to flush CSV writer");

        match get_history(product, start, end, granularity, wtr).await {
            Err(e) => {
                eprintln!("History command failed: {:?}", e);
                std::process::exit(1);
            }
            _ => {}
        };
        return ();
    }

    if let Some(_matches) = matches.subcommand_matches("balance") {
        print_balances().await;
        return ();
    }

    if let Some(_matches) = matches.subcommand_matches("payment-methods") {
        print_payment_methods().await;
        return ();
    }

    if let Some(matches) = matches.subcommand_matches("deposit") {
        let amount = match matches.value_of("amount") {
            Some(s) => match s.parse::<f64>() {
                Ok(a) => a,
                _ => {
                    println!("'{}' is an invalid dollar amount", s);
                    std::process::exit(1);
                }
            },
            None => {
                println!("You must enter an amount to deposit");
                std::process::exit(1);
            }
        };
        println!("Depositing ${} USD into Coinbase...", amount);
        match make_deposit(&amount).await {
            Some(r) => {
                println!("Successfully deposited ${} into Coinbase!", amount);
                println!("{:#?}", r);
                std::process::exit(0);
            }
            None => {
                std::process::exit(1);
            }
        };
    }

    println!("Invalid input. Type help for more information");
    return ();
}
