# HODL: Accumulate Cryptocurrency on the Cron 🐢

`hodl` is a lightweight, compiled CLI tool for accumulating cryptocurrency through the Coinbase Pro API.
It is intended to be run periodically on the crontab, and is cross-compiled to both x86_64 and ARMv7 chipsets to support
a wide range of processors. Hook your Raspberry Pi to a UPS, configure the cron, and grow your nest egg :smile:

The name is [not a typo](https://en.wikipedia.org/wiki/Hodl) :grin:

## Motivation
This project was born out of two desires:
1. The desire to reduce cryptocurrency transaction fees, and
2. The desire to learn Rust

As a Coinbase customer, I set up recurring purchases of BTC using the commercial website.
Because cryptocurrency prices fluctuate wildly,
I figured it was in my best interest to undertake cost averaging daily, rather than (semi-)monthly.

The transaction fees for commercial customers with this purchasing strategy are not insignificant.
On every (daily) $10 BTC transaction, I paid $1 in fees -- effectively, a 10% service charge.

The Coinbase Pro trading platform has much more favorable rates: at monthly volumes of less than $10k,
the fees are 0.5%. Migrating to Coinbase Pro means 9.5% more BTC per dollar spent!

Coinbase Pro does not have the ability to set up recurring purchases. They do, however, have an API.
And this is where `hodl` comes in.

### Rust
Some may call me paranoid, but I am not about to put API keys that can access my personal Coinbase
account on servers that I don't physically control. This means running servers at home, which means
electricity costs and heat.

The electricity and heat problems associated with running servers at home can be mitigated by using
hardware like a Raspberry Pi: a credit-card sized computer that draws around 5W of power,
and doesn't put out enough heat to even require fans. The downsides include limited processing power,
and downright atrocious disk IO.

The Pi's performance limitations mean that I want software running on it to be _small_ and _fast_.
Furthermore, because I'm dealing with money and network-based APIs, it's also important that the
software I'm using is _correct._  The [Rust](https://www.rust-lang.org/) Programming Language
checks all of these boxes -- with performance on par with C and C++, and a compiler that requires you
to account for failure cases and not just happy paths, it becomes difficult to write incorrect code.
Furthermore, Rust makes it easy to compile to multiple targets, which means getting it
onto an ARM processor (the Pi) is not a significant undertaking.

In fact, thanks to Github's new pricing strategy,
we just let Github Actions compile all the binaries for us! :grin:

# Using `hodl`

## Binaries
To use `hodl`, head over to the [releases](https://github.com/ajpierce/hodl/releases) page and
download the latest version for the architecture you intend to use -- x86_64 or ARMv7.

## Environment Variables
Everyone is going to have different API keys, and I'm not about to check mine into the repository :)
In order to use `hodl`, you need to set the following environment variables:

+ `COINBASE_API_KEY`
+ `COINBASE_API_SECRET`
+ `COINBASE_API_PASSPHRASE`
+ `BANK_ID`

My recommendation would be to create a script, `hodl`, that sets these and invokes the binary you download:

```bash
#!/bin/bash

export COINBASE_API_KEY=[your key]
export COINBASE_API_SECRET=[your secret]
export COINBASE_API_PASSPHRASE=[your passphrase]
export BANK_ID=[id of your associated bank account]

./hodl-ARMv7 $@
```

Then, you can set your crontab to run periodic `deposit` and `buy` commands. For example,

```crontab
# m h  dom mon dow   command
0   0    */3 *  * /home/pi/hodl deposit 39 2>&1 | tee -a /home/pi/logs/hodl.log
7   */18 *   *  * /home/pi/hodl buy BTC 6 2>&1 | tee -a /home/pi/logs/hodl.log
8   0    *   *  * /home/pi/hodl buy ETH 5 2>&1 | tee -a /home/pi/logs/hodl.log
```

Et voila, cryptocurrency cost-average investing as a fraction of the retail cost!

The crontab above will:
+ Deposit $39 USD into Coinbase every 3 days (every 72 hours)
+ Buy $6 USD worth of BTC every 18 hours
+ Buy $5 USD worth of ETH every day (every 24 hours)
+ Send both standard error and standard output to a log file

## Usage
To get the most recent list of commands, use the `help` command:

```
$ ./hodl help
```

Here are a couple more common commands:

```
$ ./hodl deposit 10.44
```
This will initiate a $10.44 deposit from your bank account to Coinbase Pro.
Please note that the funds will take DAYS to clear, so initiate your deposits well in advance of your buys!

Please also note that this command requires your `BANK_ID` environment variable to be set;
it's from this account that funds will be deposited into Coinbase Pro.

To find your `BANK_ID`, run the `payment-methods` command

```
$ ./hodl payment-methods
```

The above command will print out the banks that you have linked with your Coinbase Pro account.
Use this to determine which `BANK_ID` you will use for depositing funds.

```
$ ./hodl buy BTC 5.55
```

The `buy` command will place a market order for $5.55 (USD) worth of BTC.
You can use the `buy` command for any Coinbase Pro product listing USD as the quote half of the pair;
ETH (ETH-USD), XLM (XLM-USD), etc.

Note that the number always represents the amount of USD you wish to use, regardless of the product you're buying.

```
$ ./hodl balance USD
```

The `balance` command can be used to show you your balances in Coinbase Pro.
Replace USD with another currency (BTC, ETH, etc.) to see its balance,
or omit the currency argument to see balances for every currency.

```
$ ./hodl history BTC-USD 2019-01-01T00:00:00-04:00 2020-01-01T00:00:00-04:00 300
```

The `history` command can be used to fetch historical data about Coinbase Pro products.
The arguments appear in the following order:
1. product id
1. start date (ISO 8601)
1. end date (ISO 8601)
1. granularity (in seconds)

In the example above, we're fetching 5m candlesticks for BTC-USD for 2019, in the EDT timezone.

Historical data is returned in the CSV format, and because the API is rate limited,
long time periods will result in long-running processes.

The output is piped to STDOUT, so you should probably redirect it to a file:

```
$ ./hodl history BTC-USD 2019-01-01T00:00:00-04:00 2020-01-01T00:00:00-04:00 300 \
> /tmp/BTC-USD_2019-01-01_2020-01-01_5m.csv
```

# Developing
Make sure you have Rust installed; visit https://rustup.rs/

Once you've got Rust installed, you can run in development mode by typing

```bash
$ cargo run
```

To compile it for production:

```bash
$ cargo build --release
# Outputs the executable to ./target/release/hodl
```

This is my first attempt at writing a Rust binary -- please help me improve by filing issues or
submitting pull requests! Any internal inconsistency in the codebase is a result of my picking
up best (better?) practices as time goes on.

よろしくお願いします！
