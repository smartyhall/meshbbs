# TinyMUSH Schema Migration System

## Overview

The TinyMUSH migration system provides automatic, safe, and transparent schema evolution for all persistent game data. When data structures change, old data is automatically migrated to the new format on load.

## Design Principles

1. **Backward Compatible**: Never lose data during migration
2. **Automatic**: Migrations happen transparently on data access
3. **Logged**: All migrations are logged for debugging and monitoring
4. **Idempotent**: Migrations can run multiple times safely
5. **Testable**: Each migration has comprehensive unit tests
6. **Fail-Safe**: Corrupted data is detected and reported, not silently ignored

## Architecture

### Schema Versioning

Every persistent data structure includes a `schema_version: u8` field that tracks its format version:

```rust
pub struct PlayerRecord {
    pub username: String,
    // ... other fields ...
    pub schema_version: u8,  // Current: 2
}
```

### Migratable Trait

All migratable types implement the `Migratable` trait:

```rust
pub trait Migratable: Sized {
    fn current_schema_version() -> u8;
    fn schema_version(&self) -> u8;
    fn migrate(self) -> Result<Self>;
    fn needs_migration(&self) -> bool;
}
```

### Automatic Migration

The storage layer automatically migrates data on read:

```rust
// In TinyMushStore::get_player()
let (record, was_migrated) = load_and_migrate(&bytes, username)?;
if was_migrated {
    // Save migrated version back to disk
    self.put_player(record.clone())?;
}
```

## Current Schema Versions

| Structure | Current Version | Last Migration |
|-----------|----------------|----------------|
| PlayerRecord | v2 | Added admin_level, last_active |
| NpcRecord | v1 | Initial version |
| RoomRecord | v2 | Added housing_data, locked |
| ObjectRecord | v2 | Added currency_value, ownership_history |
| QuestRecord | v1 | Initial version |
| AchievementRecord | v1 | Initial version |

## Adding a New Migration

### Step 1: Update Schema Version

In `src/tmush/migration.rs`, increment the constant:

```rust
pub const CURRENT_PLAYER_SCHEMA_VERSION: u8 = 3;  // Was 2
```

### Step 2: Implement Migration Function

Add a migration function for the version transition:

```rust
/// Migrate PlayerRecord from v2 to v3
///
/// Changes in v3:
/// - Added companion_slots field
/// - Added tutorial_completed flag
fn migrate_player_from_v2_to_v3(mut player: PlayerRecord) -> Result<PlayerRecord> {
    // Add new field with default value
    player.companion_slots = 2;
    player.tutorial_completed = false;
    
    player.schema_version = 3;
    Ok(player)
}
```

### Step 3: Update migrate() Method

Add the new migration to the migration chain:

```rust
impl Migratable for PlayerRecord {
    fn migrate(mut self) -> Result<Self> {
        // ... existing migrations ...
        
        if self.schema_version < 3 {
            self = migrate_player_from_v2_to_v3(self)?;
        }
        
        self.schema_version = Self::current_schema_version();
        Ok(self)
    }
}
```

### Step 4: Add Tests

Write tests for the migration:

```rust
#[test]
fn test_player_migration_v2_to_v3() {
    let mut player = PlayerRecord::new("testuser", "Test", "gazebo");
    player.schema_version = 2;
    
    let migrated = player.migrate().expect("Migration should succeed");
    
    assert_eq!(migrated.schema_version, 3);
    assert_eq!(migrated.companion_slots, 2);
    assert_eq!(migrated.tutorial_completed, false);
}
```

### Step 5: Document Changes

Update `CHANGELOG.md` with schema changes:

```markdown
## [Unreleased]

### Changed
- **BREAKING**: PlayerRecord schema v2 → v3
  - Added companion_slots (defaults to 2)
  - Added tutorial_completed flag (defaults to false)
  - Existing players automatically migrated on first login
```

## Migration Strategies

### Adding Fields

**Recommended**: Use `#[serde(default)]` annotation for backward compatibility:

```rust
pub struct PlayerRecord {
    pub username: String,
    
    #[serde(default)]  // Uses Default::default() if missing
    pub companion_slots: u8,
}
```

Migration function:
```rust
fn migrate_player_from_v2_to_v3(mut player: PlayerRecord) -> Result<PlayerRecord> {
    // serde(default) handles field initialization automatically
    player.schema_version = 3;
    Ok(player)
}
```

### Removing Fields

**Warning**: Removing fields requires careful migration:

```rust
// OLD v2:
pub struct PlayerRecord {
    pub old_field: String,  // To be removed
}

// NEW v3:
pub struct PlayerRecord {
    // old_field removed
}
```

Migration function:
```rust
fn migrate_player_from_v2_to_v3(mut player: PlayerRecord) -> Result<PlayerRecord> {
    // Log removed field value for debugging
    log::info!("Removing old_field='{}' from {}", player.old_field, player.username);
    
    player.schema_version = 3;
    Ok(player)
}
```

### Renaming Fields

Use `#[serde(rename)]` and deserialize old format:

```rust
#[derive(Deserialize)]
struct PlayerV2 {
    pub old_name: String,
}

fn migrate_player_from_v2_to_v3(player: PlayerRecord) -> Result<PlayerRecord> {
    // Custom deserialization or field copying
    // ...
    Ok(player)
}
```

### Complex Transformations

For complex data transformations:

```rust
fn migrate_object_from_v1_to_v2(mut object: ObjectRecord) -> Result<ObjectRecord> {
    // Convert old copper value to new currency structure
    if object.currency_value.copper == 0 && object.value > 0 {
        object.currency_value = CurrencyAmount {
            copper: object.value,
            silver: 0,
            gold: 0,
        };
    }
    
    // Clear deprecated field
    object.value = 0;
    
    object.schema_version = 2;
    Ok(object)
}
```

## Troubleshooting

### Migration Fails

If a migration fails:

1. **Check logs**: Migration failures are logged with full error context
2. **Verify data**: Use diagnostic tools to inspect raw Sled data
3. **Add recovery logic**: Implement graceful degradation for corrupted data

```rust
fn migrate_player_from_v2_to_v3(mut player: PlayerRecord) -> Result<PlayerRecord> {
    // Validate data before migration
    if player.username.is_empty() {
        return Err(anyhow!("Cannot migrate player with empty username"));
    }
    
    // Migration logic...
    Ok(player)
}
```

### Detecting Corrupted Data

The migration system catches deserialization errors:

```rust
let (record, was_migrated) = load_and_migrate(&bytes, record_id)?;
// If this succeeds, data is valid
```

If deserialization fails, the original error includes:
- Record type
- Record ID
- Raw bytes (for forensic analysis)
- Exact bincode error

### Rolling Back Migrations

**Important**: Migrations are one-way. To roll back:

1. Restore from backup before migration
2. Or implement reverse migration logic (not recommended)

Best practice: Test migrations thoroughly in development before production deployment.

## Testing Guidelines

### Unit Tests

Every migration must have tests:

```rust
#[test]
fn test_migration_preserves_data() {
    let old_record = create_v2_record();
    let migrated = old_record.migrate().expect("Migration failed");
    
    // Verify all important data preserved
    assert_eq!(migrated.username, old_record.username);
    assert_eq!(migrated.created_at, old_record.created_at);
}

#[test]
fn test_migration_adds_defaults() {
    let old_record = create_v2_record();
    let migrated = old_record.migrate().expect("Migration failed");
    
    // Verify new fields have correct defaults
    assert_eq!(migrated.new_field, expected_default);
}

#[test]
fn test_idempotent_migration() {
    let record = create_current_record();
    let migrated = record.clone().migrate().expect("Migration failed");
    
    // Migration of current version should be no-op
    assert_eq!(record.schema_version, migrated.schema_version);
}
```

### Integration Tests

Test full load/migrate/save cycle:

```rust
#[test]
fn test_storage_auto_migration() {
    let store = TinyMushStore::open_temp().unwrap();
    
    // Write v1 data
    let mut player = PlayerRecord::new("test", "Test", "gazebo");
    player.schema_version = 1;
    store.put_player_raw(player).unwrap();
    
    // Load should auto-migrate
    let loaded = store.get_player("test").unwrap();
    assert_eq!(loaded.schema_version, CURRENT_PLAYER_SCHEMA_VERSION);
    
    // Verify migrated version was saved
    let reloaded = store.get_player("test").unwrap();
    assert_eq!(reloaded.schema_version, CURRENT_PLAYER_SCHEMA_VERSION);
}
```

## Performance Considerations

### Migration Cost

- **First Load**: Migration occurs, data is re-serialized and saved
- **Subsequent Loads**: No migration, normal deserialization
- **One-time cost per record**

### Batch Migration

For production deployments with large datasets, consider batch migration:

```rust
// Offline migration script
fn migrate_all_players(store: &TinyMushStore) -> Result<usize> {
    let player_ids = store.list_player_ids()?;
    let mut migrated_count = 0;
    
    for id in player_ids {
        if let Ok(player) = store.get_player(&id) {
            // Auto-migration happens in get_player()
            migrated_count += 1;
        }
    }
    
    Ok(migrated_count)
}
```

## Future Enhancements

Potential improvements to the migration system:

1. **Migration History**: Track which migrations have been applied
2. **Bulk Migration Tool**: CLI tool for pre-deployment batch migrations
3. **Migration Validation**: Pre-flight checks before applying migrations
4. **Rollback Support**: Reversible migrations with undo logic
5. **Schema Registry**: Centralized tracking of all schema versions

## References

- Implementation: `src/tmush/migration.rs`
- Storage Integration: `src/tmush/storage.rs`
- Type Definitions: `src/tmush/types.rs`
- Tests: `src/tmush/migration.rs` (tests module)
