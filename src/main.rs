use std::env;

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
