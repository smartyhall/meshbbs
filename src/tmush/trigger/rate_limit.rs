//! Rate limiting system for trigger execution (Phase 10)
//!
//! Prevents trigger spam and runaway scripts by:
//! - Tracking executions per object (max 100/minute)
//! - Per-player cooldowns to prevent spam
//! - Global emergency shutoff for admins
//! - Automatic disabling of problematic triggers

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc, Duration};

/// Maximum trigger executions per object per minute
const MAX_EXECUTIONS_PER_OBJECT_PER_MINUTE: usize = 100;

/// Minimum seconds between trigger executions for same player
const PLAYER_COOLDOWN_SECONDS: i64 = 1;

/// Entry tracking trigger execution count and timing
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Number of executions in current time window
    count: usize,
    /// Start of current time window
    window_start: DateTime<Utc>,
    /// Last execution time (for player cooldown)
    last_execution: DateTime<Utc>,
}

impl RateLimitEntry {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            count: 0,
            window_start: now,
            // Initialize last_execution far enough in the past to avoid cooldown
            last_execution: now - Duration::seconds(PLAYER_COOLDOWN_SECONDS + 1),
        }
    }
    
    /// Reset the rate limit window
    fn reset_window(&mut self, now: DateTime<Utc>) {
        self.count = 0;
        self.window_start = now;
    }
    
    /// Check if time window has expired (1 minute)
    fn is_window_expired(&self, now: DateTime<Utc>) -> bool {
        now.signed_duration_since(self.window_start).num_seconds() >= 60
    }
    
    /// Check if player is still in cooldown period
    fn is_in_cooldown(&self, now: DateTime<Utc>) -> bool {
        now.signed_duration_since(self.last_execution).num_seconds() < PLAYER_COOLDOWN_SECONDS
    }
}

/// Reason why trigger execution was rate limited
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitReason {
    /// Object exceeded max executions per minute
    ObjectLimitExceeded { object_id: String, limit: usize },
    /// Player is in cooldown period
    PlayerCooldown { player_name: String, seconds_remaining: i64 },
    /// Trigger has been disabled by admin or automatic detection
    TriggerDisabled { object_id: String },
    /// Global trigger system has been disabled
    GlobalDisabled,
}

impl std::fmt::Display for RateLimitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitReason::ObjectLimitExceeded { object_id, limit } => {
                write!(f, "Object {} has exceeded the rate limit of {} executions per minute", 
                       object_id, limit)
            }
            RateLimitReason::PlayerCooldown { player_name, seconds_remaining } => {
                write!(f, "Player {} must wait {} seconds before triggering again", 
                       player_name, seconds_remaining)
            }
            RateLimitReason::TriggerDisabled { object_id } => {
                write!(f, "Trigger for object {} has been disabled", object_id)
            }
            RateLimitReason::GlobalDisabled => {
                write!(f, "Trigger system is currently disabled by an administrator")
            }
        }
    }
}

/// Rate limiter for trigger system
pub struct TriggerRateLimiter {
    /// Per-object execution tracking
    object_limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Per-player execution tracking
    player_limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Objects with disabled triggers (auto-disabled or admin-disabled)
    disabled_objects: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Global kill switch (emergency shutoff)
    global_enabled: Arc<RwLock<bool>>,
}

impl TriggerRateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            object_limits: Arc::new(RwLock::new(HashMap::new())),
            player_limits: Arc::new(RwLock::new(HashMap::new())),
            disabled_objects: Arc::new(RwLock::new(HashMap::new())),
            global_enabled: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Check if trigger execution is allowed
    ///
    /// Returns Ok(()) if allowed, Err(reason) if rate limited
    pub fn check_allowed(&self, object_id: &str, player_name: &str) -> Result<(), RateLimitReason> {
        let now = Utc::now();
        
        // Check global enable flag
        {
            let enabled = self.global_enabled.read().unwrap();
            if !*enabled {
                return Err(RateLimitReason::GlobalDisabled);
            }
        }
        
        // Check if object is disabled
        {
            let disabled = self.disabled_objects.read().unwrap();
            if disabled.contains_key(object_id) {
                return Err(RateLimitReason::TriggerDisabled { 
                    object_id: object_id.to_string() 
                });
            }
        }
        
        // Check object rate limit
        {
            let mut limits = self.object_limits.write().unwrap();
            let entry = limits.entry(object_id.to_string())
                .or_insert_with(RateLimitEntry::new);
            
            // Reset window if expired
            if entry.is_window_expired(now) {
                entry.reset_window(now);
            }
            
            // Check if limit exceeded
            if entry.count >= MAX_EXECUTIONS_PER_OBJECT_PER_MINUTE {
                return Err(RateLimitReason::ObjectLimitExceeded {
                    object_id: object_id.to_string(),
                    limit: MAX_EXECUTIONS_PER_OBJECT_PER_MINUTE,
                });
            }
        }
        
        // Check player cooldown
        {
            let mut limits = self.player_limits.write().unwrap();
            let entry = limits.entry(player_name.to_string())
                .or_insert_with(RateLimitEntry::new);
            
            if entry.is_in_cooldown(now) {
                let seconds_remaining = PLAYER_COOLDOWN_SECONDS - 
                    now.signed_duration_since(entry.last_execution).num_seconds();
                return Err(RateLimitReason::PlayerCooldown {
                    player_name: player_name.to_string(),
                    seconds_remaining,
                });
            }
        }
        
        Ok(())
    }
    
    /// Record successful trigger execution
    pub fn record_execution(&self, object_id: &str, player_name: &str) {
        let now = Utc::now();
        
        // Update object limit
        {
            let mut limits = self.object_limits.write().unwrap();
            let entry = limits.entry(object_id.to_string())
                .or_insert_with(RateLimitEntry::new);
            entry.count += 1;
            entry.last_execution = now;
        }
        
        // Update player limit
        {
            let mut limits = self.player_limits.write().unwrap();
            let entry = limits.entry(player_name.to_string())
                .or_insert_with(RateLimitEntry::new);
            entry.last_execution = now;
        }
    }
    
    /// Disable trigger for a specific object
    pub fn disable_object(&self, object_id: &str) {
        let mut disabled = self.disabled_objects.write().unwrap();
        disabled.insert(object_id.to_string(), Utc::now());
    }
    
    /// Re-enable trigger for a specific object
    pub fn enable_object(&self, object_id: &str) {
        let mut disabled = self.disabled_objects.write().unwrap();
        disabled.remove(object_id);
    }
    
    /// Check if object trigger is disabled
    pub fn is_object_disabled(&self, object_id: &str) -> bool {
        let disabled = self.disabled_objects.read().unwrap();
        disabled.contains_key(object_id)
    }
    
    /// Get list of all disabled objects with timestamps
    pub fn get_disabled_objects(&self) -> Vec<(String, DateTime<Utc>)> {
        let disabled = self.disabled_objects.read().unwrap();
        disabled.iter()
            .map(|(id, time)| (id.clone(), *time))
            .collect()
    }
    
    /// Globally disable all trigger execution (emergency shutoff)
    pub fn set_global_enabled(&self, enabled: bool) {
        let mut global = self.global_enabled.write().unwrap();
        *global = enabled;
    }
    
    /// Check if trigger system is globally enabled
    pub fn is_globally_enabled(&self) -> bool {
        let enabled = self.global_enabled.read().unwrap();
        *enabled
    }
    
    /// Get current execution counts for monitoring
    pub fn get_stats(&self) -> RateLimitStats {
        let object_limits = self.object_limits.read().unwrap();
        let player_limits = self.player_limits.read().unwrap();
        let disabled = self.disabled_objects.read().unwrap();
        let enabled = self.global_enabled.read().unwrap();
        
        RateLimitStats {
            tracked_objects: object_limits.len(),
            tracked_players: player_limits.len(),
            disabled_objects: disabled.len(),
            globally_enabled: *enabled,
        }
    }
    
    /// Clear all rate limit data (for testing/admin)
    pub fn clear_all(&self) {
        self.object_limits.write().unwrap().clear();
        self.player_limits.write().unwrap().clear();
        self.disabled_objects.write().unwrap().clear();
    }
}

impl Default for TriggerRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about rate limiter state
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Number of objects being tracked
    pub tracked_objects: usize,
    /// Number of players being tracked
    pub tracked_players: usize,
    /// Number of disabled objects
    pub disabled_objects: usize,
    /// Whether trigger system is globally enabled
    pub globally_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;
    
    #[test]
    fn test_basic_rate_limiting() {
        let limiter = TriggerRateLimiter::new();
        
        // First execution should be allowed
        assert!(limiter.check_allowed("obj1", "player1").is_ok());
        limiter.record_execution("obj1", "player1");
        
        // Immediate second execution should fail (player cooldown)
        let result = limiter.check_allowed("obj1", "player1");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RateLimitReason::PlayerCooldown { .. }));
    }
    
    #[test]
    fn test_object_rate_limit() {
        let limiter = TriggerRateLimiter::new();
        
        // Execute 100 times (limit)
        for i in 0..MAX_EXECUTIONS_PER_OBJECT_PER_MINUTE {
            assert!(limiter.check_allowed("obj1", &format!("player{}", i)).is_ok());
            limiter.record_execution("obj1", &format!("player{}", i));
        }
        
        // 101st execution should fail
        let result = limiter.check_allowed("obj1", "player_new");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RateLimitReason::ObjectLimitExceeded { .. }));
    }
    
    #[test]
    fn test_player_cooldown() {
        let limiter = TriggerRateLimiter::new();
        
        // First execution
        assert!(limiter.check_allowed("obj1", "player1").is_ok());
        limiter.record_execution("obj1", "player1");
        
        // Immediate retry fails
        assert!(limiter.check_allowed("obj1", "player1").is_err());
        
        // Wait for cooldown (using a slightly longer duration to be safe)
        sleep(StdDuration::from_millis(1100));
        
        // Should work now
        assert!(limiter.check_allowed("obj1", "player1").is_ok());
    }
    
    #[test]
    fn test_disable_object() {
        let limiter = TriggerRateLimiter::new();
        
        // Initially allowed
        assert!(limiter.check_allowed("obj1", "player1").is_ok());
        
        // Disable object (without recording execution to avoid cooldown issues)
        limiter.disable_object("obj1");
        assert!(limiter.is_object_disabled("obj1"));
        
        // Should now fail with different player to avoid cooldown
        let result = limiter.check_allowed("obj1", "player2");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RateLimitReason::TriggerDisabled { .. }));
        
        // Re-enable
        limiter.enable_object("obj1");
        assert!(!limiter.is_object_disabled("obj1"));
        assert!(limiter.check_allowed("obj1", "player3").is_ok());
    }
    
    #[test]
    fn test_global_disable() {
        let limiter = TriggerRateLimiter::new();
        
        // Initially enabled
        assert!(limiter.is_globally_enabled());
        assert!(limiter.check_allowed("obj1", "player1").is_ok());
        
        // Globally disable
        limiter.set_global_enabled(false);
        assert!(!limiter.is_globally_enabled());
        
        // All triggers should fail (use different player to avoid any cooldown confusion)
        let result = limiter.check_allowed("obj1", "player2");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RateLimitReason::GlobalDisabled));
        
        // Re-enable
        limiter.set_global_enabled(true);
        assert!(limiter.check_allowed("obj1", "player3").is_ok());
    }
    
    #[test]
    fn test_get_stats() {
        let limiter = TriggerRateLimiter::new();
        
        // Initial stats
        let stats = limiter.get_stats();
        assert_eq!(stats.tracked_objects, 0);
        assert_eq!(stats.tracked_players, 0);
        assert_eq!(stats.disabled_objects, 0);
        assert!(stats.globally_enabled);
        
        // Record some executions
        limiter.record_execution("obj1", "player1");
        limiter.record_execution("obj2", "player2");
        limiter.disable_object("obj3");
        
        // Check updated stats
        let stats = limiter.get_stats();
        assert_eq!(stats.tracked_objects, 2);
        assert_eq!(stats.tracked_players, 2);
        assert_eq!(stats.disabled_objects, 1);
    }
}
