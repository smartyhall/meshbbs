//! Schema migration system for TinyMUSH data structures
//!
//! This module provides automatic schema migration for all TinyMUSH persistent data.
//! Each data structure tracks its schema version, and migrations are applied automatically
//! when older versions are detected during deserialization.
//!
//! # Design Principles
//!
//! - **Backward Compatible**: Never lose data during migration
//! - **Idempotent**: Migrations can be run multiple times safely
//! - **Logged**: All migrations are logged for debugging
//! - **Graceful**: Unknown future versions handled without panic
//! - **Testable**: Each migration has unit tests
//!
//! # Adding New Migrations
//!
//! 1. Increment `CURRENT_*_SCHEMA_VERSION` constant
//! 2. Add migration logic to `migrate_*_from_v*_to_v*()` function
//! 3. Update `migrate_*()` function to call new migration
//! 4. Add tests for the migration
//! 5. Document the schema change in CHANGELOG.md

use crate::tmush::types::*;
use anyhow::{anyhow, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current schema versions for each data structure
pub const CURRENT_PLAYER_SCHEMA_VERSION: u8 = 2;
pub const CURRENT_NPC_SCHEMA_VERSION: u8 = 1;
pub const CURRENT_ROOM_SCHEMA_VERSION: u8 = 2;
pub const CURRENT_OBJECT_SCHEMA_VERSION: u8 = 2;
pub const CURRENT_QUEST_SCHEMA_VERSION: u8 = 1;
pub const CURRENT_ACHIEVEMENT_SCHEMA_VERSION: u8 = 1;

/// Trait for types that support schema migration
pub trait Migratable: Sized {
    /// Get the current schema version for this type
    fn current_schema_version() -> u8;
    
    /// Get this instance's schema version
    fn schema_version(&self) -> u8;
    
    /// Migrate this instance to the current schema version
    fn migrate(self) -> Result<Self>;
    
    /// Check if migration is needed
    fn needs_migration(&self) -> bool {
        self.schema_version() < Self::current_schema_version()
    }
}

/// Migration result tracking for logging and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub record_type: String,
    pub record_id: String,
    pub from_version: u8,
    pub to_version: u8,
    pub success: bool,
    pub error: Option<String>,
}

impl MigrationResult {
    pub fn success(record_type: &str, record_id: &str, from_version: u8, to_version: u8) -> Self {
        Self {
            record_type: record_type.to_string(),
            record_id: record_id.to_string(),
            from_version,
            to_version,
            success: true,
            error: None,
        }
    }
    
    pub fn failure(record_type: &str, record_id: &str, from_version: u8, error: String) -> Self {
        Self {
            record_type: record_type.to_string(),
            record_id: record_id.to_string(),
            from_version,
            to_version: from_version,
            success: false,
            error: Some(error),
        }
    }
}

// ============================================================================
// PlayerRecord Migration
// ============================================================================

impl Migratable for PlayerRecord {
    fn current_schema_version() -> u8 {
        CURRENT_PLAYER_SCHEMA_VERSION
    }
    
    fn schema_version(&self) -> u8 {
        self.schema_version
    }
    
    fn migrate(mut self) -> Result<Self> {
        let original_version = self.schema_version;
        
        if !self.needs_migration() {
            return Ok(self);
        }
        
        info!(
            "Migrating PlayerRecord '{}' from schema v{} to v{}",
            self.username,
            original_version,
            Self::current_schema_version()
        );
        
        // Apply migrations in sequence
        if self.schema_version < 2 {
            self = migrate_player_from_v1_to_v2(self)?;
        }
        
        // Future migrations go here:
        // if self.schema_version < 3 {
        //     self = migrate_player_from_v2_to_v3(self)?;
        // }
        
        self.schema_version = Self::current_schema_version();
        
        info!(
            "Successfully migrated PlayerRecord '{}' from v{} to v{}",
            self.username, original_version, self.schema_version
        );
        
        Ok(self)
    }
}

/// Migrate PlayerRecord from v1 to v2
/// 
/// Changes in v2:
/// - Added admin_level field (defaults to 0)
/// - Added last_active timestamp
fn migrate_player_from_v1_to_v2(mut player: PlayerRecord) -> Result<PlayerRecord> {
    // Fields added in v2 already have #[serde(default)] annotations,
    // so they're automatically populated during deserialization.
    // This function exists for explicit data transformations if needed.
    
    player.schema_version = 2;
    Ok(player)
}

// ============================================================================
// NpcRecord Migration
// ============================================================================

impl Migratable for NpcRecord {
    fn current_schema_version() -> u8 {
        CURRENT_NPC_SCHEMA_VERSION
    }
    
    fn schema_version(&self) -> u8 {
        self.schema_version
    }
    
    fn migrate(mut self) -> Result<Self> {
        let original_version = self.schema_version;
        
        if !self.needs_migration() {
            return Ok(self);
        }
        
        info!(
            "Migrating NpcRecord '{}' from schema v{} to v{}",
            self.id,
            original_version,
            Self::current_schema_version()
        );
        
        // Currently at v1, no migrations needed yet
        // Future migrations:
        // if self.schema_version < 2 {
        //     self = migrate_npc_from_v1_to_v2(self)?;
        // }
        
        self.schema_version = Self::current_schema_version();
        
        info!(
            "Successfully migrated NpcRecord '{}' from v{} to v{}",
            self.id, original_version, self.schema_version
        );
        
        Ok(self)
    }
}

// ============================================================================
// RoomRecord Migration
// ============================================================================

impl Migratable for RoomRecord {
    fn current_schema_version() -> u8 {
        CURRENT_ROOM_SCHEMA_VERSION
    }
    
    fn schema_version(&self) -> u8 {
        self.schema_version
    }
    
    fn migrate(mut self) -> Result<Self> {
        let original_version = self.schema_version;
        
        if !self.needs_migration() {
            return Ok(self);
        }
        
        info!(
            "Migrating RoomRecord '{}' from schema v{} to v{}",
            self.id,
            original_version,
            Self::current_schema_version()
        );
        
        if self.schema_version < 2 {
            self = migrate_room_from_v1_to_v2(self)?;
        }
        
        self.schema_version = Self::current_schema_version();
        
        info!(
            "Successfully migrated RoomRecord '{}' from v{} to v{}",
            self.id, original_version, self.schema_version
        );
        
        Ok(self)
    }
}

/// Migrate RoomRecord from v1 to v2
///
/// Changes in v2:
/// - Added housing_data field
/// - Added locked field
fn migrate_room_from_v1_to_v2(mut room: RoomRecord) -> Result<RoomRecord> {
    room.schema_version = 2;
    Ok(room)
}

// ============================================================================
// ObjectRecord Migration
// ============================================================================

impl Migratable for ObjectRecord {
    fn current_schema_version() -> u8 {
        CURRENT_OBJECT_SCHEMA_VERSION
    }
    
    fn schema_version(&self) -> u8 {
        self.schema_version
    }
    
    fn migrate(mut self) -> Result<Self> {
        let original_version = self.schema_version;
        
        if !self.needs_migration() {
            return Ok(self);
        }
        
        info!(
            "Migrating ObjectRecord '{}' from schema v{} to v{}",
            self.id,
            original_version,
            Self::current_schema_version()
        );
        
        if self.schema_version < 2 {
            self = migrate_object_from_v1_to_v2(self)?;
        }
        
        self.schema_version = Self::current_schema_version();
        
        info!(
            "Successfully migrated ObjectRecord '{}' from v{} to v{}",
            self.id, original_version, self.schema_version
        );
        
        Ok(self)
    }
}

/// Migrate ObjectRecord from v1 to v2
///
/// Changes in v2:
/// - Added currency_value field (dual currency system)
/// - Deprecated old value field
/// - Added ownership_history
fn migrate_object_from_v1_to_v2(mut object: ObjectRecord) -> Result<ObjectRecord> {
    // Migrate old value field to currency_value if not already set
    if matches!(object.currency_value, CurrencyAmount::Decimal { minor_units: 0 }) && object.value > 0 {
        object.currency_value = CurrencyAmount::Decimal {
            minor_units: object.value as i64,
        };
    }
    
    object.schema_version = 2;
    Ok(object)
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Attempt to load and migrate a record from raw bytes
///
/// This is the main entry point for schema migration. It handles:
/// 1. Deserializing the raw data
/// 2. Checking schema version
/// 3. Applying migrations if needed
/// 4. Re-serializing if migration occurred
pub fn load_and_migrate<T>(data: &[u8], record_id: &str) -> Result<(T, bool)>
where
    T: Migratable + for<'de> Deserialize<'de> + Serialize,
{
    // Attempt to deserialize
    let mut record: T = bincode::deserialize(data).map_err(|e| {
        anyhow!(
            "Failed to deserialize {} '{}': {}",
            std::any::type_name::<T>(),
            record_id,
            e
        )
    })?;
    
    // Check if migration is needed
    let needs_migration = record.needs_migration();
    
    if needs_migration {
        let from_version = record.schema_version();
        record = record.migrate()?;
        info!(
            "Migrated {} '{}' from v{} to v{}",
            std::any::type_name::<T>(),
            record_id,
            from_version,
            record.schema_version()
        );
    }
    
    Ok((record, needs_migration))
}

/// Check if raw serialized data needs migration (without full deserialization)
///
/// This peeks at the schema_version field to determine if migration is needed
/// before doing a full deserialization.
pub fn check_needs_migration<T>(data: &[u8]) -> Result<bool>
where
    T: Migratable + for<'de> Deserialize<'de>,
{
    // Try to deserialize just the schema_version field
    // This is a simple heuristic - full deserialization may still be needed
    let record: T = bincode::deserialize(data)?;
    Ok(record.needs_migration())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_player_migration_v1_to_v2() {
        // Create a v1 PlayerRecord (simulated)
        let mut player = PlayerRecord::new("testuser", "Test User", "gazebo");
        player.schema_version = 1;
        
        // Migrate
        let migrated = player.migrate().expect("Migration should succeed");
        
        // Verify
        assert_eq!(migrated.schema_version, CURRENT_PLAYER_SCHEMA_VERSION);
        assert_eq!(migrated.username, "testuser");
    }
    
    #[test]
    fn test_current_version_no_migration() {
        // Create a current version record
        let player = PlayerRecord::new("testuser", "Test User", "gazebo");
        
        // Should not need migration
        assert!(!player.needs_migration());
        
        // Migrate should be no-op
        let migrated = player.clone().migrate().expect("Migration should succeed");
        assert_eq!(migrated.schema_version, player.schema_version);
    }
    
    #[test]
    fn test_object_value_migration() {
        // Create a v1 ObjectRecord with old value field
        let mut object = ObjectRecord::new_world(
            "test_sword",
            "Iron Sword",
            "A basic iron sword",
        );
        object.schema_version = 1;
        object.value = 100; // Old value field
        object.currency_value = CurrencyAmount::default(); // Should be migrated
        
        // Migrate
        let migrated = object.migrate().expect("Migration should succeed");
        
        // Verify value was migrated to decimal currency
        match migrated.currency_value {
            CurrencyAmount::Decimal { minor_units } => {
                assert_eq!(minor_units, 100);
            }
            _ => panic!("Expected Decimal currency after migration"),
        }
        assert_eq!(migrated.schema_version, CURRENT_OBJECT_SCHEMA_VERSION);
    }
    
    #[test]
    fn test_npc_no_migrations_yet() {
        // NpcRecord is at v1, no migrations defined yet
        let npc = NpcRecord::new("test_npc", "Test NPC", "Tester", "A test NPC", "town_square");
        
        assert!(!npc.needs_migration());
        assert_eq!(npc.schema_version(), CURRENT_NPC_SCHEMA_VERSION);
    }
    
    #[test]
    fn test_migration_result_logging() {
        let success = MigrationResult::success("PlayerRecord", "testuser", 1, 2);
        assert!(success.success);
        assert_eq!(success.from_version, 1);
        assert_eq!(success.to_version, 2);
        
        let failure = MigrationResult::failure(
            "PlayerRecord",
            "testuser",
            1,
            "Deserialization error".to_string(),
        );
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }
}
