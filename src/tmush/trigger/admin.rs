//! Admin commands for trigger management (Phase 10)
//!
//! Commands for managing the trigger system:
//! - @trigger/disable \<object\> - Disable trigger on an object
//! - @trigger/enable \<object\> - Re-enable trigger on an object
//! - @trigger/list - List all disabled objects
//! - @trigger/stats - Show rate limiter statistics
//! - @trigger/global off - Emergency shutoff
//! - @trigger/global on - Re-enable system

use crate::tmush::trigger::{RateLimitStats, TriggerRateLimiter};
use crate::tmush::TinyMushStore;
use std::sync::{OnceLock, RwLock};

static TRIGGER_RATE_LIMITER: OnceLock<RwLock<TriggerRateLimiter>> = OnceLock::new();

/// Get or initialize the global rate limiter
fn get_rate_limiter() -> &'static RwLock<TriggerRateLimiter> {
    TRIGGER_RATE_LIMITER.get_or_init(|| RwLock::new(TriggerRateLimiter::new()))
}

/// Check if trigger execution is allowed (used by integration helpers)
pub fn check_trigger_allowed(
    object_id: &str,
    player_name: &str,
) -> Result<(), crate::tmush::trigger::RateLimitReason> {
    get_rate_limiter()
        .read()
        .unwrap()
        .check_allowed(object_id, player_name)
}

/// Record successful trigger execution (used by integration helpers)
pub fn record_trigger_execution(object_id: &str, player_name: &str) {
    get_rate_limiter()
        .read()
        .unwrap()
        .record_execution(object_id, player_name);
}

/// Disable trigger execution for a specific object
pub fn disable_object_trigger(object_id: &str) {
    get_rate_limiter().read().unwrap().disable_object(object_id);
}

/// Enable trigger execution for a specific object
pub fn enable_object_trigger(object_id: &str) {
    get_rate_limiter().read().unwrap().enable_object(object_id);
}

/// Check if object trigger is disabled
pub fn is_object_trigger_disabled(object_id: &str) -> bool {
    get_rate_limiter()
        .read()
        .unwrap()
        .is_object_disabled(object_id)
}

/// Get list of all disabled objects
pub fn get_disabled_objects() -> Vec<(String, chrono::DateTime<chrono::Utc>)> {
    get_rate_limiter().read().unwrap().get_disabled_objects()
}

/// Globally enable/disable trigger system (emergency shutoff)
pub fn set_trigger_system_enabled(enabled: bool) {
    get_rate_limiter()
        .read()
        .unwrap()
        .set_global_enabled(enabled);
}

/// Check if trigger system is globally enabled
pub fn is_trigger_system_enabled() -> bool {
    get_rate_limiter().read().unwrap().is_globally_enabled()
}

/// Get rate limiter statistics
pub fn get_trigger_stats() -> RateLimitStats {
    get_rate_limiter().read().unwrap().get_stats()
}

/// Format trigger statistics for display
pub fn format_trigger_stats(store: &TinyMushStore) -> Vec<String> {
    let stats = get_trigger_stats();
    let mut output = vec![];

    output.push("=== Trigger System Statistics ===".to_string());
    output.push(format!(
        "Global Status: {}",
        if stats.globally_enabled {
            "ENABLED"
        } else {
            "DISABLED"
        }
    ));
    output.push(format!("Tracked Objects: {}", stats.tracked_objects));
    output.push(format!("Tracked Players: {}", stats.tracked_players));
    output.push(format!("Disabled Objects: {}", stats.disabled_objects));

    if stats.disabled_objects > 0 {
        output.push("".to_string());
        output.push("Disabled Objects:".to_string());
        for (obj_id, disabled_at) in get_disabled_objects() {
            // Try to get object name
            let name = store
                .get_object(&obj_id)
                .ok()
                .map(|obj| obj.name.clone())
                .unwrap_or_else(|| obj_id.clone());
            output.push(format!(
                "  - {} (disabled at {})",
                name,
                disabled_at.format("%Y-%m-%d %H:%M:%S")
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_disable_enable() {
        // Clear state first
        get_rate_limiter().read().unwrap().clear_all();

        // Test disable
        disable_object_trigger("test_obj");
        assert!(is_object_trigger_disabled("test_obj"));

        // Test enable
        enable_object_trigger("test_obj");
        assert!(!is_object_trigger_disabled("test_obj"));
    }

    #[test]
    fn test_global_enable_disable() {
        // Ensure starts enabled
        set_trigger_system_enabled(true);
        assert!(is_trigger_system_enabled());

        // Disable
        set_trigger_system_enabled(false);
        assert!(!is_trigger_system_enabled());

        // Re-enable
        set_trigger_system_enabled(true);
        assert!(is_trigger_system_enabled());
    }

    #[test]
    fn test_get_disabled_list() {
        // Clear state
        get_rate_limiter().read().unwrap().clear_all();

        // Disable some objects
        disable_object_trigger("obj1");
        disable_object_trigger("obj2");
        disable_object_trigger("obj3");

        // Get list
        let disabled = get_disabled_objects();
        assert_eq!(disabled.len(), 3);

        // Verify IDs present
        let ids: Vec<String> = disabled.iter().map(|(id, _)| id.clone()).collect();
        assert!(ids.contains(&"obj1".to_string()));
        assert!(ids.contains(&"obj2".to_string()));
        assert!(ids.contains(&"obj3".to_string()));

        // Clean up
        get_rate_limiter().read().unwrap().clear_all();
    }

    #[test]
    fn test_get_stats() {
        // Clear state
        get_rate_limiter().read().unwrap().clear_all();

        // Initial stats
        let stats = get_trigger_stats();
        assert_eq!(stats.disabled_objects, 0);
        assert!(stats.globally_enabled);

        // Disable object
        disable_object_trigger("obj1");

        // Check updated stats
        let stats = get_trigger_stats();
        assert_eq!(stats.disabled_objects, 1);

        // Clean up
        get_rate_limiter().read().unwrap().clear_all();
    }
}
