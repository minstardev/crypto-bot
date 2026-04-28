use super::engine::BacktestResult;
use crate::executor::paper::TradeSide;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub max_drawdown: f64,
    pub sharpe: f64,
    pub win_rate: f64,
    pub trade_count: usize,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
}

pub fn compute(result: &BacktestResult) -> Metrics {
    let total_return = if result.initial_cash > 0.0 {
        (result.final_equity / result.initial_cash - 1.0) * 100.0
    } else {
        0.0
    };

    let mut peak = result.initial_cash;
    let mut max_dd = 0.0f64;
    for (_, eq) in &result.equity_curve {
        if *eq > peak {
            peak = *eq;
        }
        let dd = if peak > 0.0 { (peak - *eq) / peak } else { 0.0 };
        if dd > max_dd {
            max_dd = dd;
        }
    }

    let returns: Vec<f64> = result
        .equity_curve
        .windows(2)
        .map(|w| {
            let p0 = w[0].1;
            let p1 = w[1].1;
            if p0 > 0.0 { (p1 / p0) - 1.0 } else { 0.0 }
        })
        .collect();
    let n = returns.len() as f64;
    let mean = if n > 0.0 {
        returns.iter().sum::<f64>() / n
    } else {
        0.0
    };
    let var = if n > 1.0 {
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0)
    } else {
        0.0
    };
    let std = var.sqrt();
    let sharpe = if std > 0.0 { mean / std * n.sqrt() } else { 0.0 };

    let mut buys: Vec<(f64, f64)> = Vec::new();
    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut win_sum = 0.0f64;
    let mut loss_sum = 0.0f64;
    for t in &result.trades {
        match t.side {
            TradeSide::Buy => buys.push((t.price, t.volume)),
            TradeSide::Sell => {
                let mut remaining = t.volume;
                while remaining > 0.0 && !buys.is_empty() {
                    let (bp, bv) = buys[0];
                    let consumed = remaining.min(bv);
                    let pnl = (t.price - bp) * consumed;
                    if pnl >= 0.0 {
                        wins += 1;
                        win_sum += pnl;
                    } else {
                        losses += 1;
                        loss_sum += -pnl;
                    }
                    remaining -= consumed;
                    if consumed >= bv {
                        buys.remove(0);
                    } else {
                        buys[0].1 -= consumed;
                    }
                }
            }
        }
    }
    let trade_count = wins + losses;
    let win_rate = if trade_count > 0 {
        wins as f64 / trade_count as f64 * 100.0
    } else {
        0.0
    };
    let avg_win = if wins > 0 { win_sum / wins as f64 } else { 0.0 };
    let avg_loss = if losses > 0 { loss_sum / losses as f64 } else { 0.0 };
    let profit_factor = if loss_sum > 0.0 {
        win_sum / loss_sum
    } else if win_sum > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    let annualized_return = match (result.equity_curve.first(), result.equity_curve.last()) {
        (Some(first), Some(last)) if result.initial_cash > 0.0 => {
            let secs = (last.0 - first.0) as f64 / 1000.0;
            let years = secs / (365.25 * 86400.0);
            if years > 0.0 {
                let r = result.final_equity / result.initial_cash;
                (r.powf(1.0 / years) - 1.0) * 100.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    };

    Metrics {
        total_return,
        annualized_return,
        max_drawdown: max_dd * 100.0,
        sharpe,
        win_rate,
        trade_count,
        avg_win,
        avg_loss,
        profit_factor,
    }
}
