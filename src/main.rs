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
    get_history, get_tick, list_orders, make_deposit, place_order, print_balance,
    print_payment_methods, ApiResponse,
};

static DEFAULT_PRODUCT: &str = "BTC-USD";

#[tokio::main]
async fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("balance")
                .about("Check balance(s).")
                .arg(
                    Arg::with_name("currency")
                        .help("Leave this blank to see balance of every currency")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("payment-methods")
                .about("Get information about available payment methods (bank accounts)"),
        )
        .subcommand(
            SubCommand::with_name("buy")
                .about("Purchase cryptocurrency with USD at the current market rate")
                .arg(
                    Arg::with_name("currency")
                        .help("The currency you wish to purchase with USD (ex: BTC)")
                        .index(1),
                )
                .arg(
                    Arg::with_name("amount")
                        .help("The amount, in USD, you wish to purchase (ex: 5.25")
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("orders")
                .about("Print the latest tick (current price/volume) for the given product-id")
                .arg(
                    Arg::with_name("product-id")
                        .help("[optional] filter orders by this product-id (ex: BTC-USD)")
                        .index(1),
                ),
        )
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
        if let Some(tick) = get_tick(&product).await {
            println!("{} {:#?}", product, tick);
            return;
        }
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

        if let Err(e) = get_history(product, start, end, granularity, wtr).await {
            eprintln!("History command failed: {:?}", e);
            std::process::exit(1);
        };
    }

    if let Some(matches) = matches.subcommand_matches("balance") {
        let currency = matches.value_of("currency");
        print_balance(currency).await;
        return;
    }

    if let Some(_matches) = matches.subcommand_matches("payment-methods") {
        print_payment_methods().await;
        return;
    }

    if let Some(matches) = matches.subcommand_matches("orders") {
        let product_id = matches.value_of("product-id");
        if let Some(orders) = list_orders(product_id).await {
            println!("{:#?}", orders);
        }
        return;
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

    if let Some(matches) = matches.subcommand_matches("buy") {
        let currency = match matches.value_of("currency") {
            Some(s) => s,
            None => {
                println!("You must enter a currency to purchase");
                std::process::exit(1);
            }
        };
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
        println!("Purchasing ${} worth of {}...", amount, currency);
        match place_order(&amount, &currency).await {
            Some(r) => {
                println!("Purchase successful!");
                println!("{:#?}", r);
                std::process::exit(0);
            }
            None => {
                std::process::exit(1);
            }
        };
    }

    eprintln!("Invalid input. Type help for more information");
    std::process::exit(1);
}
