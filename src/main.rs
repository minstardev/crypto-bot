mod api;
mod backtest;
mod config;
mod error;
mod executor;
mod model;
mod report;
mod strategy;

use api::UpbitClient;
use backtest::{BacktestConfig, run as run_backtest};
use config::Config;
use error::Result;
use report::{StrategyReport, comparison_table};
use strategy::buy_and_hold::BuyAndHold;

#[tokio::main]
async fn main() -> Result<()> {
    let env = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("crypto_bot=info,reqwest=warn"));
    tracing_subscriber::fmt().with_env_filter(env).init();

    let cfg = Config::from_env()?;
    let client = UpbitClient::new(cfg);

    let market = "KRW-BTC";
    tracing::info!("fetching daily candles for {market}");
    let mut candles = client.candles_days(market, 200, None).await?;
    candles.reverse();
    tracing::info!("loaded {} candles", candles.len());

    let bt_cfg = BacktestConfig::default();

    let reports = vec![StrategyReport::from_result(run_backtest(
        BuyAndHold::new(),
        market,
        &candles,
        &bt_cfg,
    ))];

    println!("{}", comparison_table(&reports));
    Ok(())
}
