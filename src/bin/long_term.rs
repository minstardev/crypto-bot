use crypto_bot::api::UpbitClient;
use crypto_bot::config::Config;
use crypto_bot::error::Result;
use crypto_bot::live;
use crypto_bot::notify::DiscordNotifier;

#[tokio::main]
async fn main() -> Result<()> {
    let env = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("crypto_bot=info,long_term=info,reqwest=warn")
        });
    tracing_subscriber::fmt().with_env_filter(env).init();

    let cfg = Config::from_env()?;
    let webhook = cfg.discord_webhook_long_term.clone();
    let mut live_cfg = cfg.live.clone();
    if let Some(v) = std::env::var("LONG_TERM_MAX_KRW_PER_TRADE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
    {
        live_cfg.max_krw_per_trade = v;
    }
    let client = UpbitClient::new(cfg);

    let notifier = webhook.map(DiscordNotifier::new);

    tracing::info!("[LONG-TERM] starting iteration");
    live::run_live(&client, notifier.as_ref(), &live_cfg).await
}
