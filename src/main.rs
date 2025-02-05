use crate::kryptosekken::KsRow;
use clap::Parser;
use std::path::PathBuf;

mod convert;
mod kryptosekken;
mod money;
mod time;
mod xapo;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'a', long, value_name = "BTC_ACCOUNT_FILE")]
    btc_account_file: PathBuf,

    #[arg(short = 's', long, value_name = "BTC_SAVINGS_FILE")]
    btc_savings_file: PathBuf,
}

fn read(xapo_file: PathBuf) -> Vec<KsRow> {
    xapo::read_file(&xapo_file)
        .map_err(|err| err.to_string())
        .and_then(|row| convert::xapo_to_ks(row).map_err(|err| err.to_string()))
        .unwrap_or_else(|err| panic!("Error reading {}: {err}", xapo_file.display()))
}

fn main() {
    let cli = Cli::parse();
    let btc_account = read(cli.btc_account_file);
    let btc_savings = read(cli.btc_savings_file);
    kryptosekken::write_csv(btc_account, btc_savings)
        .unwrap_or_else(|err| eprintln!("Error producing CSV: {err}"));
}
