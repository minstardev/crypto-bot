use crypto_bot::api::UpbitClient;
use crypto_bot::config::{Config, LiveMode};
use crypto_bot::error::{BotError, Result};
use crypto_bot::live;
use crypto_bot::notify::DiscordNotifier;
use crypto_bot::notify::discord::fmt_int;
use crypto_bot::strategy::{BollingerReversion, MarketState, Signal, Strategy};

const SHORT_TIMEFRAME_MINUTES: u32 = 15;
const SHORT_WARMUP_CANDLES: u32 = 200;

#[tokio::main]
async fn main() -> Result<()> {
    let env = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("crypto_bot=info,short_term=info,reqwest=warn")
        });
    tracing_subscriber::fmt().with_env_filter(env).init();

    let cfg = Config::from_env()?;
    let webhook = cfg.discord_webhook_short_term.clone();
    let mut live_cfg = cfg.live.clone();
    if let Some(v) = std::env::var("SHORT_TERM_MAX_KRW_PER_TRADE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
    {
        live_cfg.max_krw_per_trade = v;
    }
    let client = UpbitClient::new(cfg);
    let notifier = webhook.map(DiscordNotifier::new);

    if live_cfg.mode == LiveMode::Off {
        tracing::info!("[SHORT-TERM] LIVE_MODE=off, skipping");
        return Ok(());
    }

    let market = live_cfg.market.as_str();
    let (_, base) = live::parse_market(market)?;

    tracing::info!(
        "[SHORT-TERM] starting iteration: mode={:?} market={} max_krw={} timeframe={}m (exit-only flag ignored for short-term)",
        live_cfg.mode,
        market,
        fmt_int(live_cfg.max_krw_per_trade),
        SHORT_TIMEFRAME_MINUTES
    );

    let accounts = client.accounts().await?;
    let krw = live::balance_in(&accounts, "KRW");
    let asset_balance = live::balance_in(&accounts, base);
    let asset_avg = live::avg_buy_price_in(&accounts, base);

    tracing::info!(
        "account snapshot — KRW: {}, {}: {:.8} (avg buy: {})",
        fmt_int(krw),
        base,
        asset_balance,
        fmt_int(asset_avg)
    );

    let mut candles = client
        .candles_minutes(SHORT_TIMEFRAME_MINUTES, market, SHORT_WARMUP_CANDLES, None)
        .await?;
    candles.reverse();
    if candles.is_empty() {
        return Err(BotError::Strategy("no candles returned".into()));
    }

    let last_candle = candles.last().unwrap().clone();
    let last_price = last_candle.close();

    let mut strategy = BollingerReversion::new(40, 2.0, 0.02);
    if candles.len() > 1 {
        let mut warm = MarketState {
            cash: 1.0,
            position: 0.0,
            avg_buy_price: 0.0,
            last_price: 0.0,
        };
        for c in &candles[..candles.len() - 1] {
            warm.last_price = c.close();
            let _ = strategy.on_candle(c, &warm);
        }
    }

    let real = MarketState {
        cash: krw,
        position: asset_balance,
        avg_buy_price: asset_avg,
        last_price,
    };
    let signal = strategy.on_candle(&last_candle, &real);

    tracing::info!(
        "[SHORT-TERM] decision — strategy={} signal={:?} last_price={}",
        strategy.name(),
        signal,
        fmt_int(last_price)
    );

    let context = format!("short-term/{}", strategy.name());

    match signal {
        Signal::Buy { ratio } if ratio > 0.0 && krw > 0.0 => {
            live::handle_buy_pub(
                &client,
                notifier.as_ref(),
                &live_cfg,
                &last_candle,
                &context,
                krw,
                ratio,
            )
            .await?;
        }
        Signal::Sell { ratio } if ratio > 0.0 && asset_balance > 0.0 => {
            live::handle_sell_pub(
                &client,
                notifier.as_ref(),
                &live_cfg,
                &last_candle,
                &context,
                asset_balance,
                asset_avg,
                ratio,
            )
            .await?;
        }
        _ => {
            tracing::info!(
                "[SHORT-TERM] HOLD | KRW: {} | {}: {:.8} | last: {} KRW",
                fmt_int(krw),
                base,
                asset_balance,
                fmt_int(last_price)
            );
        }
    }

    Ok(())
}
