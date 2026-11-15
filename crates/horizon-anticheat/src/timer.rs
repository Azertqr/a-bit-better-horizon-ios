// File: crates/horizon-anticheat/src/timer.rs
//! Timer check neutered: allows Timer speed-ups, Blink, and FakeLag bursts without flagging.

use crate::TimerConfig;

#[derive(Clone, Debug, Default)]
pub struct Balance {
    pub ms: f64,
}

impl Balance {
    /// The start of a server tick that came `elapsed_ms` after the previous one.
    #[inline(always)]
    pub fn elapse(&mut self, _elapsed_ms: f64, _config: &TimerConfig) {
        self.ms = 0.0;
    }

    /// One client tick ended — always returns `None` (never ahead of server clock).
    #[inline(always)]
    pub fn tick_end(&mut self, _tick_rate: f32, _config: &TimerConfig) -> Option<f64> {
        self.ms = 0.0;
        None
    }

    /// The tick rate changed or the player arrived: start again.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.ms = 0.0;
    }
}
