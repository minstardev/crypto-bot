use std::cmp::Ordering;

use crypto_bot::api::UpbitClient;
use crypto_bot::backtest::{BacktestConfig, run as run_backtest};
use crypto_bot::config::Config;
use crypto_bot::error::Result;
use crypto_bot::model::Candle;
use crypto_bot::report::{StrategyReport, comparison_table};
use crypto_bot::strategy::{BollingerReversion, Pullback, RsiReversion, Strategy};

#[tokio::main]
async fn main() -> Result<()> {
    let env = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("crypto_bot=info,research_short=info,reqwest=warn")
        });
    tracing_subscriber::fmt().with_env_filter(env).init();

    let cfg = Config::from_env()?;
    let client = UpbitClient::new(cfg);
    let market = "KRW-BTC";
    let bt_cfg = BacktestConfig::default();

    tracing::info!("fetching 15-minute candles (30 days = ~2880 candles)");
    let candles_15m = client
        .candles_minutes_paginated(15, market, 2880)
        .await?;
    tracing::info!("loaded {} 15-min candles", candles_15m.len());

    tracing::info!("fetching 60-minute candles (30 days = ~720 candles)");
    let candles_1h = client
        .candles_minutes_paginated(60, market, 720)
        .await?;
    tracing::info!("loaded {} 1-hour candles", candles_1h.len());

    // Period stats — what was the underlying market move?
    let bh_15m = (candles_15m.last().unwrap().close() / candles_15m.first().unwrap().close() - 1.0) * 100.0;
    let bh_1h = (candles_1h.last().unwrap().close() / candles_1h.first().unwrap().close() - 1.0) * 100.0;
    println!(
        "\nUnderlying BTC move over backtest window:  15m_period={:+.2}%   1h_period={:+.2}%\n",
        bh_15m, bh_1h
    );

    println!("=== 15-minute timeframe — strategy candidates ===");
    let mut reports_15m: Vec<StrategyReport> = Vec::new();
    // RSI mean-reversion variants
    for (period, oversold, target, sl) in [
        (7usize, 25.0, 50.0, 0.010),
        (7, 25.0, 55.0, 0.010),
        (14, 25.0, 50.0, 0.010),
        (14, 30.0, 50.0, 0.010),
        (14, 30.0, 55.0, 0.015),
        (21, 30.0, 55.0, 0.010),
        (21, 30.0, 60.0, 0.015),
    ] {
        reports_15m.push(report(
            RsiReversion::new(period, oversold, target, sl),
            market,
            &candles_15m,
            &bt_cfg,
        ));
    }
    // Bollinger reversion variants
    for (period, sigma, sl) in [
        (20usize, 2.0, 0.010),
        (20, 2.0, 0.015),
        (20, 2.5, 0.010),
        (20, 2.5, 0.015),
        (40, 2.0, 0.015),
    ] {
        reports_15m.push(report(
            BollingerReversion::new(period, sigma, sl),
            market,
            &candles_15m,
            &bt_cfg,
        ));
    }
    // Pullback variants — short-term trend with pullback
    for (long, short, tp, sl) in [
        (96usize, 20usize, 0.010, 0.010),
        (96, 20, 0.015, 0.010),
        (96, 40, 0.015, 0.010),
        (192, 40, 0.015, 0.010),
    ] {
        reports_15m.push(report(
            Pullback::new(long, short, tp, sl),
            market,
            &candles_15m,
            &bt_cfg,
        ));
    }
    reports_15m.sort_by(cmp_total_return_desc);
    println!("{}", comparison_table(&reports_15m));

    println!("\n=== 1-hour timeframe — strategy candidates ===");
    let mut reports_1h: Vec<StrategyReport> = Vec::new();
    for (period, oversold, target, sl) in [
        (7usize, 30.0, 50.0, 0.015),
        (14, 30.0, 50.0, 0.015),
        (14, 25.0, 55.0, 0.020),
        (21, 30.0, 55.0, 0.020),
        (21, 30.0, 60.0, 0.025),
    ] {
        reports_1h.push(report(
            RsiReversion::new(period, oversold, target, sl),
            market,
            &candles_1h,
            &bt_cfg,
        ));
    }
    for (period, sigma, sl) in [
        (20usize, 2.0, 0.015),
        (20, 2.5, 0.020),
        (40, 2.0, 0.020),
    ] {
        reports_1h.push(report(
            BollingerReversion::new(period, sigma, sl),
            market,
            &candles_1h,
            &bt_cfg,
        ));
    }
    for (long, short, tp, sl) in [
        (24usize, 5usize, 0.015, 0.015),
        (24, 10, 0.020, 0.015),
        (48, 10, 0.025, 0.020),
    ] {
        reports_1h.push(report(
            Pullback::new(long, short, tp, sl),
            market,
            &candles_1h,
            &bt_cfg,
        ));
    }
    reports_1h.sort_by(cmp_total_return_desc);
    println!("{}", comparison_table(&reports_1h));

    Ok(())
}

fn report<S: Strategy>(
    strategy: S,
    market: &str,
    candles: &[Candle],
    cfg: &BacktestConfig,
) -> StrategyReport {
    StrategyReport::from_result(run_backtest(strategy, market, candles, cfg))
}

fn cmp_total_return_desc(a: &StrategyReport, b: &StrategyReport) -> Ordering {
    b.metrics
        .total_return
        .partial_cmp(&a.metrics.total_return)
        .unwrap_or(Ordering::Equal)
}
