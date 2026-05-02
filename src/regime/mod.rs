use crate::model::Candle;

pub mod linreg;
pub mod price_vs_sma;
pub mod sma_slope;
pub mod vol_normalized;

pub use linreg::LinRegSlopeDetector;
pub use price_vs_sma::PriceVsSmaDetector;
pub use sma_slope::SmaSlopeDetector;
pub use vol_normalized::VolNormalizedDetector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Bull,
    Sideways,
    Bear,
    Unknown,
}

impl Regime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Regime::Bull => "Bull",
            Regime::Sideways => "Sideways",
            Regime::Bear => "Bear",
            Regime::Unknown => "Unknown",
        }
    }
}

pub trait RegimeDetector {
    fn name(&self) -> &str;
    fn detect(&mut self, candle: &Candle) -> Regime;
    fn reset(&mut self) {}
}

pub fn classify_series<D: RegimeDetector>(detector: &mut D, candles: &[Candle]) -> Vec<Regime> {
    candles.iter().map(|c| detector.detect(c)).collect()
}

pub fn apply_hysteresis(raw: &[Regime], n: usize) -> Vec<Regime> {
    let mut out = Vec::with_capacity(raw.len());
    let mut current = Regime::Unknown;
    let mut candidate = Regime::Unknown;
    let mut count = 0usize;

    for &r in raw {
        if r == Regime::Unknown {
            out.push(current);
            continue;
        }
        if r == current {
            candidate = current;
            count = 0;
        } else if current == Regime::Unknown {
            if r == candidate {
                count += 1;
            } else {
                candidate = r;
                count = 1;
            }
            if count >= n {
                current = candidate;
                count = 0;
            }
        } else if r == candidate {
            count += 1;
            if count >= n {
                current = candidate;
                count = 0;
            }
        } else {
            candidate = r;
            count = 1;
        }
        out.push(current);
    }
    out
}

#[derive(Debug, Clone)]
pub struct DetectorReport {
    pub detector_name: String,
    pub period_label: String,
    pub expected: Regime,
    pub total: usize,
    pub raw: RegimeCounts,
    pub stabilized: RegimeCounts,
    pub hysteresis_n: usize,
}

#[derive(Debug, Clone, Default)]
pub struct RegimeCounts {
    pub bull: usize,
    pub sideways: usize,
    pub bear: usize,
    pub unknown: usize,
    pub flips: usize,
}

impl RegimeCounts {
    pub fn from_series(series: &[Regime]) -> Self {
        let mut c = RegimeCounts::default();
        let mut prev: Option<Regime> = None;
        for &r in series {
            match r {
                Regime::Bull => c.bull += 1,
                Regime::Sideways => c.sideways += 1,
                Regime::Bear => c.bear += 1,
                Regime::Unknown => c.unknown += 1,
            }
            if let Some(p) = prev {
                if p != r && r != Regime::Unknown && p != Regime::Unknown {
                    c.flips += 1;
                }
            }
            prev = Some(r);
        }
        c
    }

    pub fn count_for(&self, r: Regime) -> usize {
        match r {
            Regime::Bull => self.bull,
            Regime::Sideways => self.sideways,
            Regime::Bear => self.bear,
            Regime::Unknown => self.unknown,
        }
    }

    pub fn classified_total(&self) -> usize {
        self.bull + self.sideways + self.bear
    }

    pub fn accuracy_pct(&self, expected: Regime) -> f64 {
        let denom = self.classified_total();
        if denom == 0 {
            0.0
        } else {
            self.count_for(expected) as f64 / denom as f64 * 100.0
        }
    }
}

pub fn validate<D: RegimeDetector>(
    mut detector: D,
    candles: &[Candle],
    period_label: &str,
    expected: Regime,
    hysteresis_n: usize,
) -> DetectorReport {
    let raw_series = classify_series(&mut detector, candles);
    let stab_series = apply_hysteresis(&raw_series, hysteresis_n);
    DetectorReport {
        detector_name: detector.name().to_string(),
        period_label: period_label.to_string(),
        expected,
        total: candles.len(),
        raw: RegimeCounts::from_series(&raw_series),
        stabilized: RegimeCounts::from_series(&stab_series),
        hysteresis_n,
    }
}
