use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};

use crate::backtest::engine::BacktestResult;
use crate::backtest::metrics::{Metrics, compute};

pub struct StrategyReport {
    pub result: BacktestResult,
    pub metrics: Metrics,
}

impl StrategyReport {
    pub fn from_result(result: BacktestResult) -> Self {
        let metrics = compute(&result);
        Self { result, metrics }
    }
}

pub fn comparison_table(reports: &[StrategyReport]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Strategy",
            "Market",
            "Trades",
            "Win%",
            "TotalRet%",
            "AnnRet%",
            "MDD%",
            "Sharpe",
            "PF",
            "Final Equity",
        ]);

    for r in reports {
        table.add_row(vec![
            Cell::new(&r.result.strategy_name),
            Cell::new(&r.result.market),
            Cell::new(r.metrics.trade_count),
            Cell::new(format!("{:.2}", r.metrics.win_rate)),
            Cell::new(format!("{:+.2}", r.metrics.total_return)),
            Cell::new(format!("{:+.2}", r.metrics.annualized_return)),
            Cell::new(format!("{:.2}", r.metrics.max_drawdown)),
            Cell::new(format!("{:.2}", r.metrics.sharpe)),
            Cell::new(if r.metrics.profit_factor.is_finite() {
                format!("{:.2}", r.metrics.profit_factor)
            } else {
                "inf".into()
            }),
            Cell::new(format!("{:.0}", r.result.final_equity)),
        ]);
    }

    table
}
