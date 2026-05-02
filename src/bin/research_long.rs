use std::cmp::Ordering;

use crypto_bot::api::UpbitClient;
use crypto_bot::backtest::{BacktestConfig, run as run_backtest};
use crypto_bot::config::Config;
use crypto_bot::error::Result;
use crypto_bot::model::Candle;
use crypto_bot::regime::PriceVsSmaDetector;
use crypto_bot::report::{StrategyReport, comparison_table};
use crypto_bot::strategy::{BuyAndHold, Pullback, RegimeSwitcher, RsiReversion, Strategy};

const BULL_END_TO: &str = "2024-04-01 00:00:00";
const SIDEWAYS_END_TO: &str = "2024-10-16 00:00:00";

const HYSTERESIS_N: usize = 5;

#[tokio::main]
async fn main() -> Result<()> {
    let env = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("crypto_bot=info,research_long=info,reqwest=warn"));
    tracing_subscriber::fmt().with_env_filter(env).init();

    let cfg = Config::from_env()?;
    let client = UpbitClient::new(cfg);
    let market = "KRW-BTC";
    let bt_cfg = BacktestConfig::default();

    let bull = fetch(&client, market, Some(BULL_END_TO)).await?;
    let sideways = fetch(&client, market, Some(SIDEWAYS_END_TO)).await?;
    let bear = fetch(&client, market, None).await?;
    tracing::info!(
        "loaded: bull={} sideways={} bear={}",
        bull.len(),
        sideways.len(),
        bear.len()
    );

    println!("\n=== STAGE 1: Bull-regime sub-strategy candidates (200d bull period) ===");
    let mut bull_results = Vec::new();
    bull_results.push(report(BuyAndHold::new(), market, &bull, &bt_cfg));
    for tp in [0.05, 0.07, 0.10, 0.15] {
        for sl in [0.05, 0.07] {
            bull_results.push(report(Pullback::new(30, 10, tp, sl), market, &bull, &bt_cfg));
            bull_results.push(report(Pullback::new(50, 10, tp, sl), market, &bull, &bt_cfg));
        }
    }
    bull_results.sort_by(cmp_total_return_desc);
    println!("{}", comparison_table(&bull_results));

    println!("\n=== STAGE 2: Sideways-regime sub-strategy candidates (200d sideways period) ===");
    let mut side_results = Vec::new();
    for period in [14usize, 21, 28] {
        for oversold in [25.0, 30.0, 35.0] {
            for target in [50.0, 55.0, 60.0] {
                for sl in [0.05, 0.07] {
                    side_results.push(report(
                        RsiReversion::new(period, oversold, target, sl),
                        market,
                        &sideways,
                        &bt_cfg,
                    ));
                }
            }
        }
    }
    side_results.sort_by(cmp_total_return_desc);
    side_results.truncate(15);
    println!("{}", comparison_table(&side_results));

    println!("\n=== STAGE 3: Bear-regime sub-strategy candidates (200d bear period) ===");
    let mut bear_results = Vec::new();
    for tp in [0.05, 0.07, 0.10] {
        for sl in [0.05, 0.07] {
            bear_results.push(report(Pullback::new(30, 5, tp, sl), market, &bear, &bt_cfg));
            bear_results.push(report(Pullback::new(50, 5, tp, sl), market, &bear, &bt_cfg));
            bear_results.push(report(Pullback::new(30, 10, tp, sl), market, &bear, &bt_cfg));
        }
    }
    bear_results.sort_by(cmp_total_return_desc);
    println!("{}", comparison_table(&bear_results));

    println!("\n=== STAGE 4: RegimeSwitcher candidates — full 3-regime test ===");
    let switcher_configs: Vec<(&str, fn() -> RegimeSwitcher)> = vec![
        ("Current(Pull5/RSI21os35t50/Pull5)", current_switcher),
        ("BH/RSI21os35t50/Pull5tp7", switcher_bh_rsi_pull7),
        ("Pull10tp10/RSI21os35t60/Pull5tp7", switcher_pull10_rsi_pull),
        ("BH/RSI21os30t55/Pull5tp10", switcher_bh_rsi30_pull10),
    ];

    let mut switcher_reports = Vec::new();
    for (label, factory) in &switcher_configs {
        let r_bull = run_backtest(factory(), market, &bull, &bt_cfg);
        let r_side = run_backtest(factory(), market, &sideways, &bt_cfg);
        let r_bear = run_backtest(factory(), market, &bear, &bt_cfg);
        let compound = (r_bull.final_equity / r_bull.initial_cash)
            * (r_side.final_equity / r_side.initial_cash)
            * (r_bear.final_equity / r_bear.initial_cash);
        let max_mdd = [
            crypto_bot::backtest::compute(&r_bull).max_drawdown,
            crypto_bot::backtest::compute(&r_side).max_drawdown,
            crypto_bot::backtest::compute(&r_bear).max_drawdown,
        ]
        .iter()
        .cloned()
        .fold(0.0f64, f64::max);
        println!(
            "{label}: bull={:+.2}% / sideways={:+.2}% / bear={:+.2}% / compound={:+.2}% / max-MDD={:.2}%",
            (r_bull.final_equity / r_bull.initial_cash - 1.0) * 100.0,
            (r_side.final_equity / r_side.initial_cash - 1.0) * 100.0,
            (r_bear.final_equity / r_bear.initial_cash - 1.0) * 100.0,
            (compound - 1.0) * 100.0,
            max_mdd,
        );
        switcher_reports.push((label.to_string(), compound, max_mdd));
    }

    println!("\n=== Switcher ranking by 3-regime compound ===");
    let mut ranked = switcher_reports.clone();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    for (label, compound, mdd) in &ranked {
        println!("  {label}: compound={:+.2}% / max-MDD={:.2}%", (compound - 1.0) * 100.0, mdd);
    }

    Ok(())
}

fn current_switcher() -> RegimeSwitcher {
    RegimeSwitcher::new(
        Box::new(PriceVsSmaDetector::new(50, 10, 0.05, 0.05)),
        Box::new(Pullback::new(30, 10, 0.05, 0.05)),
        Box::new(RsiReversion::new(21, 35.0, 50.0, 0.05)),
        Box::new(Pullback::new(30, 5, 0.05, 0.05)),
        HYSTERESIS_N,
    )
}

fn switcher_bh_rsi_pull7() -> RegimeSwitcher {
    RegimeSwitcher::new(
        Box::new(PriceVsSmaDetector::new(50, 10, 0.05, 0.05)),
        Box::new(BuyAndHold::new()),
        Box::new(RsiReversion::new(21, 35.0, 50.0, 0.05)),
        Box::new(Pullback::new(30, 5, 0.07, 0.05)),
        HYSTERESIS_N,
    )
}

fn switcher_pull10_rsi_pull() -> RegimeSwitcher {
    RegimeSwitcher::new(
        Box::new(PriceVsSmaDetector::new(50, 10, 0.05, 0.05)),
        Box::new(Pullback::new(30, 10, 0.10, 0.05)),
        Box::new(RsiReversion::new(21, 35.0, 60.0, 0.05)),
        Box::new(Pullback::new(30, 5, 0.07, 0.05)),
        HYSTERESIS_N,
    )
}

fn switcher_bh_rsi30_pull10() -> RegimeSwitcher {
    RegimeSwitcher::new(
        Box::new(PriceVsSmaDetector::new(50, 10, 0.05, 0.05)),
        Box::new(BuyAndHold::new()),
        Box::new(RsiReversion::new(21, 30.0, 55.0, 0.05)),
        Box::new(Pullback::new(30, 5, 0.10, 0.05)),
        HYSTERESIS_N,
    )
}

fn report<S: Strategy>(strategy: S, market: &str, candles: &[Candle], cfg: &BacktestConfig) -> StrategyReport {
    StrategyReport::from_result(run_backtest(strategy, market, candles, cfg))
}

fn cmp_total_return_desc(a: &StrategyReport, b: &StrategyReport) -> Ordering {
    b.metrics
        .total_return
        .partial_cmp(&a.metrics.total_return)
        .unwrap_or(Ordering::Equal)
}

async fn fetch(client: &UpbitClient, market: &str, to: Option<&str>) -> Result<Vec<Candle>> {
    let mut c = client.candles_days(market, 200, to).await?;
    c.reverse();
    Ok(c)
}
