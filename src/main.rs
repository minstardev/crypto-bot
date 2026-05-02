use std::time::Duration;

use crypto_bot::api::UpbitClient;
use crypto_bot::backtest::{BacktestConfig, BacktestResult, run as run_backtest};
use crypto_bot::config::{Config, LiveMode};
use crypto_bot::error::Result;
use crypto_bot::executor::paper::Trade;
use crypto_bot::live;
use crypto_bot::model::Candle;
use crypto_bot::notify::DiscordNotifier;
use crypto_bot::regime::PriceVsSmaDetector;
use crypto_bot::report::{StrategyReport, comparison_table};
use crypto_bot::strategy::{BuyAndHold, Pullback, RegimeSwitcher, RsiReversion};

const BULL_END_TO: &str = "2024-04-01 00:00:00";
const SIDEWAYS_END_TO: &str = "2024-10-16 00:00:00";

const BULL_LABEL: &str = "bull (2023-09 ~ 2024-03)";
const SIDEWAYS_LABEL: &str = "sideways (2024-04 ~ 2024-10)";
const BEAR_LABEL: &str = "bear (last 200d)";

const HYSTERESIS_N: usize = 5;

#[tokio::main]
async fn main() -> Result<()> {
    let env = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("crypto_bot=info,reqwest=warn"));
    tracing_subscriber::fmt().with_env_filter(env).init();

    let cfg = Config::from_env()?;
    let discord_url = cfg.discord_webhook_long_term.clone();
    let live_cfg = cfg.live.clone();
    let client = UpbitClient::new(cfg);

    let notifier = discord_url.map(DiscordNotifier::new);

    if live_cfg.mode != LiveMode::Off {
        return live::run_live(&client, notifier.as_ref(), &live_cfg).await;
    }

    run_backtests(&client, notifier.as_ref()).await
}

async fn run_backtests(
    client: &UpbitClient,
    notifier: Option<&DiscordNotifier>,
) -> Result<()> {
    let market = "KRW-BTC";
    let bt_cfg = BacktestConfig::default();

    let bull_candles = fetch(client, market, Some(BULL_END_TO)).await?;
    let sideways_candles = fetch(client, market, Some(SIDEWAYS_END_TO)).await?;
    let bear_candles = fetch(client, market, None).await?;
    tracing::info!(
        "loaded candles: bull={} sideways={} bear={}",
        bull_candles.len(),
        sideways_candles.len(),
        bear_candles.len()
    );

    let switcher_bull = run_backtest(make_switcher(), market, &bull_candles, &bt_cfg);
    let switcher_side = run_backtest(make_switcher(), market, &sideways_candles, &bt_cfg);
    let switcher_bear = run_backtest(make_switcher(), market, &bear_candles, &bt_cfg);

    let oracle_bull = run_backtest(
        Pullback::new(30, 10, 0.05, 0.05),
        market,
        &bull_candles,
        &bt_cfg,
    );
    let oracle_side = run_backtest(
        RsiReversion::new(21, 35.0, 50.0, 0.05),
        market,
        &sideways_candles,
        &bt_cfg,
    );
    let oracle_bear = run_backtest(
        Pullback::new(30, 5, 0.05, 0.05),
        market,
        &bear_candles,
        &bt_cfg,
    );

    let bh_bull = run_backtest(BuyAndHold::new(), market, &bull_candles, &bt_cfg);
    let bh_side = run_backtest(BuyAndHold::new(), market, &sideways_candles, &bt_cfg);
    let bh_bear = run_backtest(BuyAndHold::new(), market, &bear_candles, &bt_cfg);

    print_period(BULL_LABEL, &switcher_bull, &oracle_bull, &bh_bull);
    print_period(SIDEWAYS_LABEL, &switcher_side, &oracle_side, &bh_side);
    print_period(BEAR_LABEL, &switcher_bear, &oracle_bear, &bh_bear);

    if let Some(notifier) = notifier {
        tracing::info!("sending Discord notifications for bear-period switcher trades");
        let header = format!(
            "**RegimeSwitcher backtest — {}**\n{} trades on {}",
            BEAR_LABEL,
            switcher_bear.trades.len(),
            market
        );
        notifier.send_text(&header).await?;
        notify_trades(notifier, market, "RegimeSwitcher", &switcher_bear.trades).await?;
        tracing::info!("sent {} trade notifications", switcher_bear.trades.len());
    } else {
        tracing::info!("DISCORD_WEBHOOK_URL not set — skipping notifications");
    }

    Ok(())
}

fn make_switcher() -> RegimeSwitcher {
    RegimeSwitcher::new(
        Box::new(PriceVsSmaDetector::new(50, 10, 0.05, 0.05)),
        Box::new(Pullback::new(30, 10, 0.05, 0.05)),
        Box::new(RsiReversion::new(21, 35.0, 50.0, 0.05)),
        Box::new(Pullback::new(30, 5, 0.05, 0.05)),
        HYSTERESIS_N,
    )
}

async fn fetch(client: &UpbitClient, market: &str, to: Option<&str>) -> Result<Vec<Candle>> {
    let mut c = client.candles_days(market, 200, to).await?;
    c.reverse();
    Ok(c)
}

fn print_period(
    label: &str,
    switcher: &BacktestResult,
    oracle: &BacktestResult,
    buyhold: &BacktestResult,
) {
    let reports = vec![
        StrategyReport::from_result(switcher.clone()),
        StrategyReport::from_result(oracle.clone()),
        StrategyReport::from_result(buyhold.clone()),
    ];
    println!("\n=== {} ===", label);
    println!("{}", comparison_table(&reports));
}

async fn notify_trades(
    notifier: &DiscordNotifier,
    market: &str,
    strategy: &str,
    trades: &[Trade],
) -> Result<()> {
    for trade in trades {
        notifier
            .send_trade(trade, market, strategy, None)
            .await?;
        tokio::time::sleep(Duration::from_millis(800)).await;
    }
    Ok(())
}
