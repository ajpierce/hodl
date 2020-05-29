extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_derive;

use clap::{App, Arg, SubCommand};
use std::env;

pub mod api;
use api::check::get_tick;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(SubCommand::with_name("buy").about("Purchase BTC with USD"))
        .subcommand(
            SubCommand::with_name("check")
                .about("Print the current exchange rate")
                .version(env!("CARGO_PKG_VERSION"))
                .arg(
                    Arg::with_name("product")
                        .help("The product-id to check. Defaults to USDBTC")
                        .short("p"),
                ),
        )
        .subcommand(SubCommand::with_name("lookback").about("Fetch historical data"))
        .subcommand(SubCommand::with_name("transfer").about("Transfer USD to Coinbase Pro"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("check") {
        if matches.is_present("product") {
            get_tick(matches.value_of("product").unwrap());
        }
    }
}

/*
static USAGE_MSG: &'static str = r#"
Usage:
  $ hodl [command] [args]

Valid commands are:
  + buy - Purcahse BTC with USD
  + transfer - Transfer USD from bank to Coinbase Pro exchange
  + check - Print the current exchange rate"#;

fn print_help_for(cmd: &str) {
    let msg = match cmd {
        "buy" => String::from(
            r#"
The `buy` command is used to purchase BTC.
Specify the amount (in USD) to purchase with the 2nd argument.
Usage:
  $ hodl buy 10     # Will purchase $10.00 USD worth of bitcoin"#,
        ),
        _ => format!(
            "Unknown command: {cmd}.\n{default}",
            cmd = cmd,
            default = USAGE_MSG.to_string()
        ),
    };
    println!("{}", msg);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = &args.get(0).expect(USAGE_MSG);
    print_help_for(&command);
}
*/
