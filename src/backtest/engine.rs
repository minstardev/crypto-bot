use crate::executor::paper::{PaperExecutor, Trade};
use crate::model::Candle;
use crate::strategy::Strategy;

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_cash: f64,
    pub fee_rate: f64,
    pub slippage_rate: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_cash: 1_000_000.0,
            fee_rate: 0.0005,
            slippage_rate: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub strategy_name: String,
    pub market: String,
    pub initial_cash: f64,
    pub final_equity: f64,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<(i64, f64)>,
    pub candles_used: usize,
}

pub fn run<S: Strategy>(
    mut strategy: S,
    market: &str,
    candles: &[Candle],
    cfg: &BacktestConfig,
) -> BacktestResult {
    let mut exec = PaperExecutor::new(cfg.initial_cash)
        .with_fee(cfg.fee_rate)
        .with_slippage(cfg.slippage_rate);

    for candle in candles {
        let state = exec.snapshot(candle.close());
        let sig = strategy.on_candle(candle, &state);
        exec.apply(sig, candle.close(), candle.timestamp);
    }

    let final_price = candles.last().map(|c| c.close()).unwrap_or(0.0);
    let final_equity = exec.equity(final_price);

    BacktestResult {
        strategy_name: strategy.name().to_string(),
        market: market.to_string(),
        initial_cash: cfg.initial_cash,
        final_equity,
        trades: exec.trades,
        equity_curve: exec.equity_curve,
        candles_used: candles.len(),
    }
}
