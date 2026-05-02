use super::{MarketState, Signal, Strategy};
use crate::model::Candle;
use crate::regime::{Regime, RegimeDetector};

pub struct RegimeSwitcher {
    name: String,
    detector: Box<dyn RegimeDetector + Send>,
    bull_strategy: Box<dyn Strategy + Send>,
    sideways_strategy: Box<dyn Strategy + Send>,
    bear_strategy: Box<dyn Strategy + Send>,
    current_regime: Regime,
    candidate_regime: Regime,
    candidate_count: usize,
    hysteresis_n: usize,
    last_regime_change_ts: Option<i64>,
}

impl RegimeSwitcher {
    pub fn new(
        detector: Box<dyn RegimeDetector + Send>,
        bull: Box<dyn Strategy + Send>,
        sideways: Box<dyn Strategy + Send>,
        bear: Box<dyn Strategy + Send>,
        hysteresis_n: usize,
    ) -> Self {
        let name = format!(
            "Switcher[{}|N{}|B:{}|S:{}|R:{}]",
            detector.name(),
            hysteresis_n,
            bull.name(),
            sideways.name(),
            bear.name(),
        );
        Self {
            name,
            detector,
            bull_strategy: bull,
            sideways_strategy: sideways,
            bear_strategy: bear,
            current_regime: Regime::Unknown,
            candidate_regime: Regime::Unknown,
            candidate_count: 0,
            hysteresis_n,
            last_regime_change_ts: None,
        }
    }

    pub fn current_regime(&self) -> Regime {
        self.current_regime
    }

    fn step_hysteresis(&mut self, raw: Regime) -> bool {
        let prev = self.current_regime;
        if raw == Regime::Unknown {
            return false;
        }
        if raw == self.current_regime {
            self.candidate_regime = self.current_regime;
            self.candidate_count = 0;
        } else if self.current_regime == Regime::Unknown {
            if raw == self.candidate_regime {
                self.candidate_count += 1;
            } else {
                self.candidate_regime = raw;
                self.candidate_count = 1;
            }
            if self.candidate_count >= self.hysteresis_n {
                self.current_regime = self.candidate_regime;
                self.candidate_count = 0;
            }
        } else if raw == self.candidate_regime {
            self.candidate_count += 1;
            if self.candidate_count >= self.hysteresis_n {
                self.current_regime = self.candidate_regime;
                self.candidate_count = 0;
            }
        } else {
            self.candidate_regime = raw;
            self.candidate_count = 1;
        }
        prev != self.current_regime
    }
}

impl Strategy for RegimeSwitcher {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_candle(&mut self, candle: &Candle, state: &MarketState) -> Signal {
        let raw = self.detector.detect(candle);
        let changed = self.step_hysteresis(raw);

        let bull_sig = self.bull_strategy.on_candle(candle, state);
        let side_sig = self.sideways_strategy.on_candle(candle, state);
        let bear_sig = self.bear_strategy.on_candle(candle, state);

        if changed {
            self.last_regime_change_ts = Some(candle.timestamp);
            if state.position > 0.0 {
                return Signal::Sell { ratio: 1.0 };
            }
        }

        match self.current_regime {
            Regime::Bull => bull_sig,
            Regime::Sideways => side_sig,
            Regime::Bear => bear_sig,
            Regime::Unknown => Signal::Hold,
        }
    }

    fn reset(&mut self) {
        self.detector.reset();
        self.bull_strategy.reset();
        self.sideways_strategy.reset();
        self.bear_strategy.reset();
        self.current_regime = Regime::Unknown;
        self.candidate_regime = Regime::Unknown;
        self.candidate_count = 0;
        self.last_regime_change_ts = None;
    }
}
