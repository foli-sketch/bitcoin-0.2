//! Thermal Governor (FINAL)
//!
//! Mobile-safe thermal & battery protection layer.
//! Authoritative signal: Battery
//! Advisory signal: Temperature
//! Throttling via pulse mining + sleep (no fake rate limiters)

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use thiserror::Error;

/// Thermal states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    Normal,
    Warning,
    Critical,
    Shutdown,
}

/// Pulse states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PulseState {
    Working,
    Sleeping,
}

/// FINAL Thermal Governor
pub struct ThermalGovernor {
    state: Arc<RwLock<ThermalState>>,
    last_check: Arc<RwLock<Instant>>,
    check_interval: Duration,

    // Pulse mining
    pulse_work: Duration,
    pulse_sleep: Duration,
    pulse_state: Arc<RwLock<PulseState>>,
    last_pulse_change: Arc<RwLock<Instant>>,
}

impl ThermalGovernor {
    /// Create governor with mobile-safe defaults
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ThermalState::Normal)),
            last_check: Arc::new(RwLock::new(Instant::now())),
            check_interval: Duration::from_secs(10),

            pulse_work: Duration::from_secs(30),
            pulse_sleep: Duration::from_secs(20),
            pulse_state: Arc::new(RwLock::new(PulseState::Working)),
            last_pulse_change: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /* ================= PUBLIC API ================= */

    /// Update thermal state (call periodically)
    pub fn update(&self) -> Result<(), ThermalError> {
        let now = Instant::now();
        let last = *self.last_check.read().map_err(|_| ThermalError::Lock)?;

        if now.duration_since(last) < self.check_interval {
            return Ok(());
        }

        let battery = self.read_battery_percent()?;
        let temp = self.read_temperature_celsius();

        let new_state = if battery <= 5.0 {
            ThermalState::Shutdown
        } else if battery <= 10.0 {
            ThermalState::Critical
        } else if temp >= Some(45.0) {
            ThermalState::Critical
        } else if temp >= Some(40.0) {
            ThermalState::Warning
        } else {
            ThermalState::Normal
        };

        let mut state = self.state.write().map_err(|_| ThermalError::Lock)?;
        if *state != new_state {
            info!(
                "Thermal state: {:?} → {:?} (battery {:.1}%, temp {:?}°C)",
                *state, new_state, battery, temp
            );
            *state = new_state;
        }

        *self.last_check.write().map_err(|_| ThermalError::Lock)? = now;
        Ok(())
    }

    /// Can mining/work proceed now?
    pub fn allow_work(&self) -> Result<bool, ThermalError> {
        let state = *self.state.read().map_err(|_| ThermalError::Lock)?;

        match state {
            ThermalState::Shutdown => Ok(false),
            ThermalState::Critical => self.pulse_allowed(Duration::from_secs(10), Duration::from_secs(40)),
            ThermalState::Warning => self.pulse_allowed(Duration::from_secs(20), Duration::from_secs(30)),
            ThermalState::Normal => self.pulse_allowed(self.pulse_work, self.pulse_sleep),
        }
    }

    /// Scaling factor for mining intensity
    pub fn scaling_factor(&self) -> Result<f32, ThermalError> {
        let state = *self.state.read().map_err(|_| ThermalError::Lock)?;
        Ok(match state {
            ThermalState::Normal => 1.0,
            ThermalState::Warning => 0.5,
            ThermalState::Critical => 0.2,
            ThermalState::Shutdown => 0.0,
        })
    }

    /// Emergency hard stop
    pub fn emergency_shutdown(&self) -> Result<bool, ThermalError> {
        let battery = self.read_battery_percent()?;
        if battery <= 2.0 {
            error!("Emergency shutdown: battery {:.1}%", battery);
            return Ok(true);
        }
        Ok(false)
    }

    /* ================= INTERNAL ================= */

    fn pulse_allowed(
        &self,
        work: Duration,
        sleep: Duration,
    ) -> Result<bool, ThermalError> {
        let now = Instant::now();
        let mut pulse = self.pulse_state.write().map_err(|_| ThermalError::Lock)?;
        let mut last = self.last_pulse_change.write().map_err(|_| ThermalError::Lock)?;

        match *pulse {
            PulseState::Working => {
                if now.duration_since(*last) >= work {
                    *pulse = PulseState::Sleeping;
                    *last = now;
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            PulseState::Sleeping => {
                if now.duration_since(*last) >= sleep {
                    *pulse = PulseState::Working;
                    *last = now;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn read_battery_percent(&self) -> Result<f32, ThermalError> {
        #[cfg(target_os = "android")]
        {
            use battery::Manager;
            let manager = Manager::new().map_err(|e| ThermalError::Battery(e.to_string()))?;
            if let Ok(mut batteries) = manager.batteries() {
                if let Some(Ok(b)) = batteries.next() {
                    return Ok(b.state_of_charge().value * 100.0);
                }
            }
        }
        Ok(80.0) // safe fallback
    }

    fn read_temperature_celsius(&self) -> Option<f32> {
        #[cfg(target_os = "android")]
        {
            for i in 0..10 {
                let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
                if let Ok(v) = std::fs::read_to_string(&path) {
                    if let Ok(milli) = v.trim().parse::<f32>() {
                        return Some(milli / 1000.0);
                    }
                }
            }
        }
        None
    }
}

/* ================= ERRORS ================= */

#[derive(Debug, Error)]
pub enum ThermalError {
    #[error("lock poisoned")]
    Lock,
    #[error("battery error: {0}")]
    Battery(String),
          }
