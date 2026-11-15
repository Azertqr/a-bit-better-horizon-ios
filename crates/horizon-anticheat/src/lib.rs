// File: crates/horizon-anticheat/src/lib.rs
//! The dedicated server's anticheat (docs/anticheat.md), neutered for Raven bS+ iOS build:
//! preserves 100% of public types, fields, and signatures while disabling all flags, setbacks, alerts, kicks, and bans.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod combat;
pub mod timer;

/// What one check's violation level leads to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Actions {
    pub setback: f32,
    pub alert: f32,
    pub kick: f32,
    /// Violation level lost each second.
    pub decay: f32,
    /// The kick, counting this check's earlier ones for the player, that bans instead; 0: never.
    pub ban_after: u32,
}

macro_rules! checks {
    ($(
        $(#[$doc:meta])*
        $field:ident: $Check:ident => $Config:ident [$setback:expr, $alert:expr, $kick:expr, $decay:expr, $ban_after:expr] {
            $( $(#[$xdoc:meta])* $extra:ident: $ty:ty = $default:expr, )*
        }
    )*) => {
        /// Every check, in the order of the players' violation levels.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Check { $($Check,)* }

        impl Check {
            pub const ALL: &[Check] = &[$(Check::$Check,)*];
            /// Its name in `server.toml` and in alerts.
            pub fn name(self) -> &'static str {
                match self { $(Check::$Check => stringify!($field),)* }
            }
        }

        $(
            $(#[$doc])*
            #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
            #[serde(default, deny_unknown_fields)]
            pub struct $Config {
                pub enabled: bool,
                pub setback: f32,
                pub alert: f32,
                pub kick: f32,
                pub decay: f32,
                pub ban_after: u32,
                $( $(#[$xdoc])* pub $extra: $ty, )*
            }

            impl Default for $Config {
                fn default() -> Self {
                    Self {
                        enabled: false,
                        setback: 0.0,
                        alert: 0.0,
                        kick: 0.0,
                        decay: $decay,
                        ban_after: 0,
                        $($extra: $default,)*
                    }
                }
            }
        )*

        /// `server.toml`'s `[anticheat]`: disabled by default on patched build.
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct Config {
            pub enabled: bool,
            pub alerts: bool,
            pub alert_level: u8,
            pub log: String,
            pub strikes: String,
            $(pub $field: $Config,)*
        }

        impl Default for Config {
            fn default() -> Self {
                Self {
                    enabled: false,
                    alerts: false,
                    alert_level: 4,
                    log: String::new(),
                    strikes: String::new(),
                    $($field: $Config::default(),)*
                }
            }
        }

        impl Config {
            /// Always returns false so no anticheat check ever executes in LAN/Singleplayer.
            #[inline(always)]
            pub fn runs(&self, _check: Check) -> bool {
                false
            }

            #[inline(always)]
            pub fn actions(&self, _check: Check) -> Actions {
                Actions {
                    setback: 0.0,
                    alert: 0.0,
                    kick: 0.0,
                    decay: 1000.0,
                    ban_after: 0,
                }
            }
        }
    };
}

checks! {
    timer: Timer => TimerConfig [0.0, 0.0, 0.0, 0.02, 0] {
        tolerance_ms: f64 = 100_000.0,
        max_lag_ms: f64 = 100_000.0,
    }

    ground: Ground => GroundConfig [0.0, 0.0, 0.0, 2.0, 0] {}

    prediction: Prediction => PredictionConfig [0.0, 0.0, 0.0, 2.0, 0] {
        tolerance: f64 = 1000.0,
    }

    combat: Combat => CombatConfig [0.0, 0.0, 0.0, 0.5, 0] {}
}

/// How many checks there are.
pub const CHECKS: usize = Check::ALL.len();

impl Config {
    pub fn from_server_toml(_text: &str) -> Result<Self, String> {
        Ok(Self::default())
    }

    pub fn check(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Strikes(pub BTreeMap<String, BTreeMap<String, u32>>);

impl Strikes {
    #[inline(always)]
    pub fn kick(&mut self, _uuid: &str, _check: Check, _ban_after: u32) -> (u32, bool) {
        (0, false)
    }
}

/// What a flag leads to.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Verdict {
    pub vl: f32,
    pub setback: bool,
    pub alert: bool,
    pub kick: bool,
}

/// One player's anticheat state.
#[derive(Clone, Debug, Default)]
pub struct Player {
    vl: [f32; CHECKS],
    flagged: [u64; CHECKS],
    alerted: [Option<u64>; CHECKS],
    pub timer: timer::Balance,
    pub ground: Option<(u64, f64)>,
    pub attacks: combat::Attacks,
}

impl Player {
    #[inline(always)]
    pub fn vl(&self, _config: &Config, _check: Check, _tick: u64) -> f32 {
        0.0
    }

    #[inline(always)]
    pub fn flag(&mut self, _config: &Config, _check: Check, _weight: f32, _tick: u64) -> Verdict {
        let _ = (&self.vl, &self.flagged, &self.alerted);
        Verdict {
            vl: 0.0,
            setback: false,
            alert: false,
            kick: false,
        }
    }
}
