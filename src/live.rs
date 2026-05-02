use crate::api::UpbitClient;
use crate::config::{LiveConfig, LiveMode};
use crate::error::{BotError, Result};
use crate::executor::LiveExecutor;
use crate::model::{Account, Candle};
use crate::notify::DiscordNotifier;
use crate::notify::discord::fmt_int;
use crate::regime::{PriceVsSmaDetector, Regime};
use crate::strategy::{BuyAndHold, MarketState, Pullback, RegimeSwitcher, RsiReversion, Signal, Strategy};

const UPBIT_MIN_KRW_ORDER: f64 = 5_000.0;
const HYSTERESIS_N: usize = 5;

pub async fn run_live(
    client: &UpbitClient,
    notifier: Option<&DiscordNotifier>,
    cfg: &LiveConfig,
) -> Result<()> {
    if cfg.mode == LiveMode::Off {
        return Ok(());
    }

    let market = cfg.market.as_str();
    let (_quote, base) = parse_market(market)?;

    tracing::info!(
        "live mode={:?} exit_only={} market={} max_krw={} base_asset={}",
        cfg.mode,
        cfg.exit_only,
        market,
        fmt_int(cfg.max_krw_per_trade),
        base
    );

    let accounts = client.accounts().await?;
    let krw = balance_in(&accounts, "KRW");
    let asset_balance = balance_in(&accounts, base);
    let asset_avg = avg_buy_price_in(&accounts, base);

    tracing::info!(
        "account snapshot — KRW: {}, {}: {:.8} (avg buy: {})",
        fmt_int(krw),
        base,
        asset_balance,
        fmt_int(asset_avg)
    );

    let mut candles = client.candles_days(market, 200, None).await?;
    candles.reverse();
    if candles.is_empty() {
        return Err(BotError::Strategy("no candles returned".into()));
    }

    let last_candle = candles.last().unwrap().clone();
    let last_price = last_candle.close();

    let mut strategy = make_switcher();
    if candles.len() > 1 {
        let mut warm_state = MarketState {
            cash: 1.0,
            position: 0.0,
            avg_buy_price: 0.0,
            last_price: 0.0,
        };
        for c in &candles[..candles.len() - 1] {
            warm_state.last_price = c.close();
            let _ = strategy.on_candle(c, &warm_state);
        }
    }

    let real_state = MarketState {
        cash: krw,
        position: asset_balance,
        avg_buy_price: asset_avg,
        last_price,
    };
    let signal = strategy.on_candle(&last_candle, &real_state);
    let regime = strategy.current_regime();

    tracing::info!(
        "decision — regime={} signal={:?} last_price={}",
        regime.as_str(),
        signal,
        fmt_int(last_price)
    );

    match signal {
        Signal::Buy { .. } if cfg.exit_only => {
            tracing::info!(
                "Buy signal suppressed (exit-only mode) — regime={} last_price={}",
                regime.as_str(),
                fmt_int(last_price)
            );
        }
        Signal::Buy { ratio } if ratio > 0.0 && krw > 0.0 => {
            handle_buy(client, notifier, cfg, &last_candle, regime, krw, ratio).await?;
        }
        Signal::Sell { ratio } if ratio > 0.0 && asset_balance > 0.0 => {
            handle_sell(
                client,
                notifier,
                cfg,
                &last_candle,
                regime,
                asset_balance,
                asset_avg,
                ratio,
            )
            .await?;
        }
        _ => {
            let msg = format!(
                "ℹ️ HOLD — regime={} | KRW: {} | {}: {:.8} | last: {} KRW",
                regime.as_str(),
                fmt_int(krw),
                base,
                asset_balance,
                fmt_int(last_price)
            );
            tracing::info!("{}", msg);
            if !cfg.exit_only {
                if let Some(n) = notifier {
                    let _ = n.send_text(&msg).await;
                }
            }
        }
    }

    Ok(())
}

async fn handle_buy(
    client: &UpbitClient,
    notifier: Option<&DiscordNotifier>,
    cfg: &LiveConfig,
    candle: &Candle,
    regime: Regime,
    krw: f64,
    ratio: f64,
) -> Result<()> {
    let raw = krw * ratio.clamp(0.0, 1.0);
    let order_amount = raw.min(cfg.max_krw_per_trade);

    if order_amount < UPBIT_MIN_KRW_ORDER {
        let msg = format!(
            "⚠️ BUY skipped — calc {} KRW < Upbit min {} KRW (ratio={:.2}, regime={})",
            fmt_int(order_amount),
            fmt_int(UPBIT_MIN_KRW_ORDER),
            ratio,
            regime.as_str()
        );
        tracing::warn!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    let price_str = fmt_int(candle.close());

    if cfg.mode == LiveMode::DryRun {
        let msg = format!(
            "🟡 [DRY-RUN] BUY {} — would spend {} KRW at ~{} KRW (ratio={:.2}, regime={})",
            cfg.market,
            fmt_int(order_amount),
            price_str,
            ratio,
            regime.as_str()
        );
        tracing::info!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    tracing::info!(
        "🚨 LIVE-REAL placing market BUY {} for {} KRW",
        cfg.market,
        fmt_int(order_amount)
    );

    let live = LiveExecutor::new(client.clone(), cfg.market.clone());
    match live.market_buy_krw(order_amount).await {
        Ok(order) => {
            let msg = format!(
                "🟢 LIVE BUY placed — {} KRW of {} (uuid={}, state={}, regime={})",
                fmt_int(order_amount), cfg.market, order.uuid, order.state, regime.as_str()
            );
            tracing::info!("{}", msg);
            notify(notifier, &msg).await;
            Ok(())
        }
        Err(e) => {
            let msg = format!("🚨 LIVE BUY FAILED — {}", e);
            tracing::error!("{}", msg);
            notify(notifier, &msg).await;
            Err(e)
        }
    }
}

async fn handle_sell(
    client: &UpbitClient,
    notifier: Option<&DiscordNotifier>,
    cfg: &LiveConfig,
    candle: &Candle,
    regime: Regime,
    position: f64,
    avg_buy: f64,
    ratio: f64,
) -> Result<()> {
    let volume = position * ratio.clamp(0.0, 1.0);
    let est_value = volume * candle.close();

    if est_value < UPBIT_MIN_KRW_ORDER {
        let msg = format!(
            "⚠️ SELL skipped — est value {} KRW < Upbit min {} KRW (vol={:.8}, regime={})",
            fmt_int(est_value),
            fmt_int(UPBIT_MIN_KRW_ORDER),
            volume,
            regime.as_str()
        );
        tracing::warn!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    let pnl_pct = if avg_buy > 0.0 {
        (candle.close() - avg_buy) / avg_buy * 100.0
    } else {
        0.0
    };

    if cfg.mode == LiveMode::DryRun {
        let msg = format!(
            "🟡 [DRY-RUN] SELL {} — would sell {:.8} (~{} KRW, PnL {:+.2}%, regime={})",
            cfg.market,
            volume,
            fmt_int(est_value),
            pnl_pct,
            regime.as_str()
        );
        tracing::info!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    tracing::info!(
        "🚨 LIVE-REAL placing market SELL {} volume {:.8}",
        cfg.market,
        volume
    );

    let live = LiveExecutor::new(client.clone(), cfg.market.clone());
    match live.market_sell_volume(volume).await {
        Ok(order) => {
            let msg = format!(
                "🔴 LIVE SELL placed — {:.8} of {} (~{} KRW, PnL {:+.2}%, uuid={}, regime={})",
                volume, cfg.market, fmt_int(est_value), pnl_pct, order.uuid, regime.as_str()
            );
            tracing::info!("{}", msg);
            notify(notifier, &msg).await;
            Ok(())
        }
        Err(e) => {
            let msg = format!("🚨 LIVE SELL FAILED — {}", e);
            tracing::error!("{}", msg);
            notify(notifier, &msg).await;
            Err(e)
        }
    }
}

async fn notify(notifier: Option<&DiscordNotifier>, msg: &str) {
    if let Some(n) = notifier {
        if let Err(e) = n.send_text(msg).await {
            tracing::warn!("discord notify failed: {}", e);
        }
    }
}

fn make_switcher() -> RegimeSwitcher {
    RegimeSwitcher::new(
        Box::new(PriceVsSmaDetector::new(50, 10, 0.05, 0.05)),
        Box::new(BuyAndHold::new()),
        Box::new(RsiReversion::new(28, 35.0, 55.0, 0.05)),
        Box::new(Pullback::new(30, 5, 0.05, 0.07)),
        HYSTERESIS_N,
    )
}

pub fn parse_market(market: &str) -> Result<(&str, &str)> {
    let mut parts = market.splitn(2, '-');
    let quote = parts
        .next()
        .ok_or_else(|| BotError::Config(format!("invalid market: {}", market)))?;
    let base = parts
        .next()
        .ok_or_else(|| BotError::Config(format!("invalid market: {}", market)))?;
    Ok((quote, base))
}

pub fn balance_in(accounts: &[Account], currency: &str) -> f64 {
    accounts
        .iter()
        .find(|a| a.currency == currency)
        .map(|a| a.balance_f64())
        .unwrap_or(0.0)
}

pub fn avg_buy_price_in(accounts: &[Account], currency: &str) -> f64 {
    accounts
        .iter()
        .find(|a| a.currency == currency)
        .map(|a| a.avg_buy_price_f64())
        .unwrap_or(0.0)
}

pub async fn handle_buy_pub(
    client: &UpbitClient,
    notifier: Option<&DiscordNotifier>,
    cfg: &LiveConfig,
    candle: &Candle,
    context: &str,
    krw: f64,
    ratio: f64,
) -> Result<()> {
    let raw = krw * ratio.clamp(0.0, 1.0);
    let order_amount = raw.min(cfg.max_krw_per_trade);

    if order_amount < UPBIT_MIN_KRW_ORDER {
        let msg = format!(
            "⚠️ BUY skipped — calc {} KRW < Upbit min {} KRW (ratio={:.2}, ctx={})",
            fmt_int(order_amount),
            fmt_int(UPBIT_MIN_KRW_ORDER),
            ratio,
            context
        );
        tracing::warn!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    let price_str = fmt_int(candle.close());

    if cfg.mode == LiveMode::DryRun {
        let msg = format!(
            "🟡 [DRY-RUN] BUY {} — would spend {} KRW at ~{} KRW (ratio={:.2}, ctx={})",
            cfg.market, fmt_int(order_amount), price_str, ratio, context
        );
        tracing::info!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    let live = LiveExecutor::new(client.clone(), cfg.market.clone());
    match live.market_buy_krw(order_amount).await {
        Ok(order) => {
            let msg = format!(
                "🟢 LIVE BUY placed — {} KRW of {} (uuid={}, state={}, ctx={})",
                fmt_int(order_amount), cfg.market, order.uuid, order.state, context
            );
            tracing::info!("{}", msg);
            notify(notifier, &msg).await;
            Ok(())
        }
        Err(e) => {
            let msg = format!("🚨 LIVE BUY FAILED — {}", e);
            tracing::error!("{}", msg);
            notify(notifier, &msg).await;
            Err(e)
        }
    }
}

pub async fn handle_sell_pub(
    client: &UpbitClient,
    notifier: Option<&DiscordNotifier>,
    cfg: &LiveConfig,
    candle: &Candle,
    context: &str,
    position: f64,
    avg_buy: f64,
    ratio: f64,
) -> Result<()> {
    let volume = position * ratio.clamp(0.0, 1.0);
    let est_value = volume * candle.close();

    if est_value < UPBIT_MIN_KRW_ORDER {
        let msg = format!(
            "⚠️ SELL skipped — est value {} KRW < Upbit min {} KRW (vol={:.8}, ctx={})",
            fmt_int(est_value),
            fmt_int(UPBIT_MIN_KRW_ORDER),
            volume,
            context
        );
        tracing::warn!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    let pnl_pct = if avg_buy > 0.0 {
        (candle.close() - avg_buy) / avg_buy * 100.0
    } else {
        0.0
    };

    if cfg.mode == LiveMode::DryRun {
        let msg = format!(
            "🟡 [DRY-RUN] SELL {} — would sell {:.8} (~{} KRW, PnL {:+.2}%, ctx={})",
            cfg.market, volume, fmt_int(est_value), pnl_pct, context
        );
        tracing::info!("{}", msg);
        notify(notifier, &msg).await;
        return Ok(());
    }

    let live = LiveExecutor::new(client.clone(), cfg.market.clone());
    match live.market_sell_volume(volume).await {
        Ok(order) => {
            let msg = format!(
                "🔴 LIVE SELL placed — {:.8} of {} (~{} KRW, PnL {:+.2}%, uuid={}, ctx={})",
                volume, cfg.market, fmt_int(est_value), pnl_pct, order.uuid, context
            );
            tracing::info!("{}", msg);
            notify(notifier, &msg).await;
            Ok(())
        }
        Err(e) => {
            let msg = format!("🚨 LIVE SELL FAILED — {}", e);
            tracing::error!("{}", msg);
            notify(notifier, &msg).await;
            Err(e)
        }
    }
}
