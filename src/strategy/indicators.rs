pub fn sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}

pub fn std_dev(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    let m = sma(prices);
    let var =
        prices.iter().map(|p| (p - m).powi(2)).sum::<f64>() / (prices.len() - 1) as f64;
    var.sqrt()
}

/// Cutler's RSI — uses simple moving average of gains/losses over the last `period` diffs.
/// `closes` must contain at least `period + 1` entries; otherwise returns 50.0 (neutral).
pub fn rsi(closes: &[f64], period: usize) -> f64 {
    if closes.len() < period + 1 {
        return 50.0;
    }
    let n = closes.len();
    let window = &closes[n - period - 1..];

    let mut gain = 0.0f64;
    let mut loss = 0.0f64;
    for w in window.windows(2) {
        let d = w[1] - w[0];
        if d >= 0.0 {
            gain += d;
        } else {
            loss -= d;
        }
    }
    let avg_gain = gain / period as f64;
    let avg_loss = loss / period as f64;
    if avg_loss == 0.0 {
        return 100.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - 100.0 / (1.0 + rs)
}

pub struct Bands {
    pub middle: f64,
    pub upper: f64,
    pub lower: f64,
}

pub fn bollinger(prices: &[f64], std_mul: f64) -> Bands {
    let m = sma(prices);
    let s = std_dev(prices);
    Bands {
        middle: m,
        upper: m + s * std_mul,
        lower: m - s * std_mul,
    }
}
