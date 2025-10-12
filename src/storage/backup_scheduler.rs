//! Automated backup scheduler for TinyMUSH world database.
//!
//! This module provides a cross-platform backup scheduler that runs based on system clock
//! checks rather than relying on OS-specific cron systems. The scheduler is designed to
//! work on Windows, Linux, macOS, and embedded systems.
//!
//! # Features
//! - Automatic backups at configurable intervals
//! - UTC time boundary alignment (like IDENT beacons)
//! - Configurable via in-game admin commands
//! - Automatic retention policy enforcement
//! - No external dependencies (no cron required)

use std::time::Instant;
use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, debug};

use super::backup::{BackupManager, BackupType, RetentionPolicy};

/// Backup frequency configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupFrequency {
    /// Disabled - no automatic backups
    Disabled,
    /// Every hour at the top of the hour
    Hourly,
    /// Every 2 hours (0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22)
    Every2Hours,
    /// Every 4 hours (0, 4, 8, 12, 16, 20)
    Every4Hours,
    /// Every 6 hours (0, 6, 12, 18)
    Every6Hours,
    /// Every 12 hours (0, 12)
    Every12Hours,
    /// Once daily at midnight UTC
    Daily,
}

impl BackupFrequency {
    /// Get the frequency in minutes (for display purposes)
    pub fn minutes(&self) -> u32 {
        match self {
            BackupFrequency::Disabled => 0,
            BackupFrequency::Hourly => 60,
            BackupFrequency::Every2Hours => 120,
            BackupFrequency::Every4Hours => 240,
            BackupFrequency::Every6Hours => 360,
            BackupFrequency::Every12Hours => 720,
            BackupFrequency::Daily => 1440,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            BackupFrequency::Disabled => "Disabled",
            BackupFrequency::Hourly => "Every hour",
            BackupFrequency::Every2Hours => "Every 2 hours",
            BackupFrequency::Every4Hours => "Every 4 hours",
            BackupFrequency::Every6Hours => "Every 6 hours",
            BackupFrequency::Every12Hours => "Every 12 hours",
            BackupFrequency::Daily => "Daily at midnight UTC",
        }
    }

    /// Parse from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "disabled" | "off" | "none" => Some(BackupFrequency::Disabled),
            "hourly" | "1h" | "60m" => Some(BackupFrequency::Hourly),
            "2h" | "2hours" | "every2hours" => Some(BackupFrequency::Every2Hours),
            "4h" | "4hours" | "every4hours" => Some(BackupFrequency::Every4Hours),
            "6h" | "6hours" | "every6hours" => Some(BackupFrequency::Every6Hours),
            "12h" | "12hours" | "every12hours" => Some(BackupFrequency::Every12Hours),
            "daily" | "1d" | "24h" => Some(BackupFrequency::Daily),
            _ => None,
        }
    }
}

impl Default for BackupFrequency {
    fn default() -> Self {
        BackupFrequency::Every6Hours
    }
}

/// Backup scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedulerConfig {
    /// Whether automatic backups are enabled
    pub enabled: bool,
    /// How often to create automatic backups
    pub frequency: BackupFrequency,
    /// Database path to backup
    pub db_path: std::path::PathBuf,
    /// Where to store backups
    pub backup_path: std::path::PathBuf,
    /// Retention policy for automatic backups
    pub retention: RetentionPolicy,
}

impl Default for BackupSchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency: BackupFrequency::Every6Hours,
            db_path: std::path::PathBuf::from("data/tinymush"),
            backup_path: std::path::PathBuf::from("data/backups"),
            retention: RetentionPolicy::default(),
        }
    }
}

/// Backup scheduler state tracker
pub struct BackupScheduler {
    config: BackupSchedulerConfig,
    last_backup_time: Option<Instant>,
    last_backup_boundary_minute: Option<i64>,
}

impl BackupScheduler {
    /// Create a new backup scheduler with the given configuration
    pub fn new(config: BackupSchedulerConfig) -> Self {
        Self {
            config,
            last_backup_time: None,
            last_backup_boundary_minute: None,
        }
    }

    /// Update the scheduler configuration (for admin commands)
    pub fn update_config(&mut self, config: BackupSchedulerConfig) {
        self.config = config;
        info!("Backup scheduler configuration updated: enabled={}, frequency={:?}", 
              self.config.enabled, self.config.frequency);
    }

    /// Get the current configuration
    pub fn config(&self) -> &BackupSchedulerConfig {
        &self.config
    }

    /// Enable automatic backups
    pub fn enable(&mut self) {
        self.config.enabled = true;
        info!("Automatic backups enabled");
    }

    /// Disable automatic backups
    pub fn disable(&mut self) {
        self.config.enabled = false;
        info!("Automatic backups disabled");
    }

    /// Set the backup frequency
    pub fn set_frequency(&mut self, frequency: BackupFrequency) {
        self.config.frequency = frequency;
        info!("Backup frequency set to: {}", frequency.description());
    }

    /// Check if it's time to create a backup and create one if so.
    ///
    /// This should be called periodically (e.g., every second or minute) from the main event loop.
    /// The scheduler will determine if a backup should be created based on UTC time boundaries.
    ///
    /// # Returns
    /// - `Ok(Some(backup_id))` if a backup was created
    /// - `Ok(None)` if no backup was needed
    /// - `Err(e)` if backup creation failed
    pub fn check_and_backup(&mut self) -> Result<Option<String>> {
        // Check if automatic backups are enabled
        if !self.config.enabled {
            return Ok(None);
        }

        // Check if frequency is disabled
        if self.config.frequency == BackupFrequency::Disabled {
            return Ok(None);
        }

        let now = Utc::now();
        let should_backup = self.should_backup_now(&now);

        if !should_backup {
            return Ok(None);
        }

        // Compute boundary minute key (Unix epoch minutes) for dedupe within the same scheduled minute
        let boundary_minute = now.timestamp() / 60;
        if let Some(last_min) = self.last_backup_boundary_minute {
            if last_min == boundary_minute {
                // Already created a backup in this minute boundary; skip duplicate
                debug!("Backup already created in this minute boundary, skipping");
                return Ok(None);
            }
        }

        // Create the backup
        info!("Creating automatic backup (frequency: {})", self.config.frequency.description());
        
        let mut manager = BackupManager::new(
            self.config.db_path.clone(),
            self.config.backup_path.clone(),
            self.config.retention.clone(),
        )?;

        // Determine backup type based on frequency
        let backup_type = match self.config.frequency {
            BackupFrequency::Daily => BackupType::Daily,
            _ => BackupType::Automatic,
        };

        let backup_name = format!("auto_{}", now.format("%Y%m%d_%H%M%S"));
        let metadata = manager.create_backup(Some(backup_name), backup_type)?;
        
        info!("Automatic backup created: {} ({} bytes)", metadata.id, metadata.size_bytes);
        
        // Apply retention policy to clean up old backups
        let deleted = manager.apply_retention_policy()?;
        if !deleted.is_empty() {
            info!("Retention policy deleted {} old backup(s)", deleted.len());
        }

        // Update state
        self.last_backup_time = Some(Instant::now());
        self.last_backup_boundary_minute = Some(boundary_minute);

        Ok(Some(metadata.id))
    }

    /// Determine if a backup should be created at this time based on UTC boundaries
    fn should_backup_now(&self, now: &DateTime<Utc>) -> bool {
        let hour = now.hour();
        let minute = now.minute();

        match self.config.frequency {
            BackupFrequency::Disabled => false,
            BackupFrequency::Hourly => minute == 0,
            BackupFrequency::Every2Hours => minute == 0 && hour % 2 == 0,
            BackupFrequency::Every4Hours => minute == 0 && hour % 4 == 0,
            BackupFrequency::Every6Hours => minute == 0 && hour % 6 == 0,
            BackupFrequency::Every12Hours => minute == 0 && hour % 12 == 0,
            BackupFrequency::Daily => minute == 0 && hour == 0,
        }
    }

    /// Get status information for display
    pub fn status(&self) -> BackupSchedulerStatus {
        BackupSchedulerStatus {
            enabled: self.config.enabled,
            frequency: self.config.frequency,
            last_backup_time: self.last_backup_time,
            db_path: self.config.db_path.clone(),
            backup_path: self.config.backup_path.clone(),
        }
    }
}

/// Status information for the backup scheduler
#[derive(Debug, Clone)]
pub struct BackupSchedulerStatus {
    pub enabled: bool,
    pub frequency: BackupFrequency,
    pub last_backup_time: Option<Instant>,
    pub db_path: std::path::PathBuf,
    pub backup_path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_frequency_parsing() {
        assert_eq!(BackupFrequency::from_str("disabled"), Some(BackupFrequency::Disabled));
        assert_eq!(BackupFrequency::from_str("hourly"), Some(BackupFrequency::Hourly));
        assert_eq!(BackupFrequency::from_str("2h"), Some(BackupFrequency::Every2Hours));
        assert_eq!(BackupFrequency::from_str("daily"), Some(BackupFrequency::Daily));
        assert_eq!(BackupFrequency::from_str("invalid"), None);
    }

    #[test]
    fn test_frequency_descriptions() {
        assert_eq!(BackupFrequency::Hourly.description(), "Every hour");
        assert_eq!(BackupFrequency::Daily.description(), "Daily at midnight UTC");
    }

    #[test]
    fn test_scheduler_enable_disable() {
        let config = BackupSchedulerConfig::default();
        let mut scheduler = BackupScheduler::new(config);
        
        scheduler.disable();
        assert!(!scheduler.config().enabled);
        
        scheduler.enable();
        assert!(scheduler.config().enabled);
    }

    #[test]
    fn test_scheduler_frequency() {
        let config = BackupSchedulerConfig::default();
        let mut scheduler = BackupScheduler::new(config);
        
        scheduler.set_frequency(BackupFrequency::Daily);
        assert_eq!(scheduler.config().frequency, BackupFrequency::Daily);
    }
}
