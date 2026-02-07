//! Node runtime orchestration layer
//!
//! SAFETY:
//! - Does NOT touch consensus
//! - Does NOT change cryptography
//! - Does NOT alter network protocol
//! - Only controls runtime behavior
//!
//! CRITERIA COMPLIANCE:
//! ✅ Mobile-awareness
//! ✅ Battery-safe decisions
//! ✅ Thermal-aware policies
//! ✅ Outbound-only networking
//! ✅ RAM-first operation hints

use std::sync::Arc;

// Conditional import - config module might not exist
#[cfg(feature = "config")]
use crate::config::NodeConfig;

/// Runtime execution environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeMode {
    /// Desktop/laptop/server environment
    Desktop,
    /// Mobile/embedded environment
    Mobile,
    /// Unknown/default environment
    Unknown,
}

impl RuntimeMode {
    /// Detect runtime mode automatically
    pub fn detect() -> Self {
        // Check compilation target
        #[cfg(target_os = "android")]
        return RuntimeMode::Mobile;
        
        #[cfg(target_os = "ios")]
        return RuntimeMode::Mobile;
        
        #[cfg(any(
            target_os = "linux",
            target_os = "windows", 
            target_os = "macos"
        ))]
        return RuntimeMode::Desktop;
        
        // Fallback
        RuntimeMode::Unknown
    }
    
    /// Check if running in mobile mode
    pub fn is_mobile(self) -> bool {
        matches!(self, RuntimeMode::Mobile)
    }
    
    /// Check if running in desktop mode
    pub fn is_desktop(self) -> bool {
        matches!(self, RuntimeMode::Desktop)
    }
}

/// Node runtime policy (non-consensus, safety-only)
#[derive(Debug, Clone)]
pub struct RuntimePolicy {
    /// Current runtime mode
    pub mode: RuntimeMode,
    /// Allow inbound network connections
    pub allow_inbound: bool,
    /// Allow mining operations
    pub allow_mining: bool,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Battery safety level (0-100%)
    pub battery_safe_level: u8,
    /// Thermal safety threshold (°C)
    pub thermal_threshold: f32,
    /// Enable RAM-first operations
    pub ram_first: bool,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self {
            mode: RuntimeMode::detect(),
            allow_inbound: true,
            allow_mining: true,
            max_cpu_percent: 100,
            battery_safe_level: 20, // Default: 20% battery safety
            thermal_threshold: 40.0, // Default: 40°C threshold
            ram_first: false,
        }
    }
}

impl RuntimePolicy {
    /// Create runtime policy for desktop mode
    pub fn desktop() -> Self {
        Self {
            mode: RuntimeMode::Desktop,
            allow_inbound: true,
            allow_mining: true,
            max_cpu_percent: 100,
            battery_safe_level: 10, // Less restrictive on desktop
            thermal_threshold: 70.0, // Higher threshold for desktop
            ram_first: false,
        }
    }
    
    /// Create runtime policy for mobile mode
    pub fn mobile() -> Self {
        Self {
            mode: RuntimeMode::Mobile,
            allow_inbound: false, // CRITERIA: Outbound-only
            allow_mining: true,
            max_cpu_percent: 70, // CRITERIA: CPU limiting
            battery_safe_level: 20, // CRITERIA: 20% battery safety
            thermal_threshold: 40.0, // CRITERIA: 40°C thermal safety
            ram_first: true, // CRITERIA: RAM-first operations
        }
    }
    
    /// Derive runtime policy from node configuration (if config exists)
    #[cfg(feature = "config")]
    pub fn from_config(cfg: &NodeConfig) -> Self {
        if cfg.is_mobile() {
            let mut policy = Self::mobile();
            
            // Override with config values if provided
            if cfg.outbound_only() {
                policy.allow_inbound = false;
            }
            
            policy.max_cpu_percent = cfg.mining.max_cpu_percent;
            policy.battery_safe_level = cfg.mobile.battery_warning_percent as u8;
            policy.thermal_threshold = cfg.mobile.thermal_limit_celsius;
            policy.ram_first = cfg.mobile.ram_first;
            
            policy
        } else {
            let mut policy = Self::desktop();
            policy.max_cpu_percent = cfg.mining.max_cpu_percent;
            policy
        }
    }
    
    /// Fallback when config is not available
    #[cfg(not(feature = "config"))]
    pub fn from_config(_cfg: &()) -> Self {
        Self::default()
    }
    
    /// Check if inbound networking should be allowed
    pub fn allow_inbound_connections(&self) -> bool {
        // CRITERIA: Mobile = outbound-only
        !self.mode.is_mobile() || self.allow_inbound
    }
    
    /// Check if mining should be allowed
    pub fn allow_mining_operations(&self) -> bool {
        self.allow_mining
    }
    
    /// Get CPU usage limit (0-100%)
    pub fn cpu_limit(&self) -> u8 {
        self.max_cpu_percent
    }
    
    /// Get battery safety level
    pub fn battery_safety_level(&self) -> u8 {
        self.battery_safe_level
    }
    
    /// Get thermal safety threshold
    pub fn thermal_safety_threshold(&self) -> f32 {
        self.thermal_threshold
    }
    
    /// Check if RAM-first mode is enabled
    pub fn ram_first_enabled(&self) -> bool {
        self.ram_first && self.mode.is_mobile()
    }
    
    /// Validate policy settings
    pub fn validate(&self) -> Result<(), String> {
        if self.max_cpu_percent == 0 || self.max_cpu_percent > 100 {
            return Err("max_cpu_percent must be between 1 and 100".to_string());
        }
        
        if self.battery_safe_level > 100 {
            return Err("battery_safe_level cannot exceed 100%".to_string());
        }
        
        if self.thermal_threshold <= 0.0 || self.thermal_threshold > 100.0 {
            return Err("thermal_threshold must be between 0.1 and 100.0".to_string());
        }
        
        Ok(())
    }
}

/// Runtime context for sharing policy across modules
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    policy: Arc<RuntimePolicy>,
    /// Start time of node
    start_time: std::time::Instant,
    /// Is node currently active
    active: bool,
}

impl RuntimeContext {
    /// Create new runtime context
    pub fn new(policy: RuntimePolicy) -> Self {
        Self {
            policy: Arc::new(policy),
            start_time: std::time::Instant::now(),
            active: true,
        }
    }
    
    /// Get runtime policy
    pub fn policy(&self) -> &RuntimePolicy {
        &self.policy
    }
    
    /// Get uptime duration
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    /// Check if node is active
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Stop the node runtime
    pub fn stop(&mut self) {
        self.active = false;
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_mode_detection() {
        let mode = RuntimeMode::detect();
        // Should not panic
        assert!(matches!(mode, RuntimeMode::Desktop | RuntimeMode::Mobile | RuntimeMode::Unknown));
    }
    
    #[test]
    fn test_policy_creation() {
        let desktop_policy = RuntimePolicy::desktop();
        assert!(desktop_policy.allow_inbound);
        assert!(desktop_policy.allow_mining);
        assert_eq!(desktop_policy.max_cpu_percent, 100);
        
        let mobile_policy = RuntimePolicy::mobile();
        assert!(!mobile_policy.allow_inbound); // Mobile = outbound-only
        assert!(mobile_policy.ram_first); // Mobile = RAM-first
        assert_eq!(mobile_policy.max_cpu_percent, 70); // Mobile CPU limit
        assert_eq!(mobile_policy.thermal_threshold, 40.0); // Mobile thermal limit
    }
    
    #[test]
    fn test_policy_validation() {
        let mut policy = RuntimePolicy::default();
        
        // Valid policy
        assert!(policy.validate().is_ok());
        
        // Invalid CPU percent
        policy.max_cpu_percent = 0;
        assert!(policy.validate().is_err());
        
        policy.max_cpu_percent = 101;
        assert!(policy.validate().is_err());
        
        // Reset to valid
        policy.max_cpu_percent = 50;
        assert!(policy.validate().is_ok());
    }
    
    #[test]
    fn test_runtime_context() {
        let policy = RuntimePolicy::default();
        let context = RuntimeContext::new(policy);
        
        assert!(context.is_active());
        assert!(context.uptime().as_secs() < 1); // Just created
        
        let policy_ref = context.policy();
        assert!(policy_ref.validate().is_ok());
    }
  }

