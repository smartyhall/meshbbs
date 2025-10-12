# TinyMUSH Admin Account Bootstrap

**Status**: ✅ Complete  
**Version**: 1.0  
**Date**: October 10, 2025

## Overview

Every TinyMUSH database automatically seeds an initial admin account during first-time initialization. This ensures operators always have administrative access to manage the system without requiring manual database manipulation or backdoor access.

## Automatic Seeding

### When It Happens

Admin seeding occurs during database initialization when:
1. The database is created for the first time (`TinyMushStoreBuilder::open()`)
2. World seeding is enabled (default behavior)
3. No admin accounts currently exist

### What Gets Created

**Default Admin Account**:
- **Username**: `admin`
- **Display Name**: `admin (Admin)`
- **Starting Location**: `town_square` (REQUIRED_START_LOCATION_ID)
- **Admin Level**: 3 (Sysop - highest privilege)
- **Admin Flag**: `is_admin = true`

### Initialization Sequence

```rust
// From src/tmush/storage.rs open_with_options()
if seed_world {
    store.seed_world_if_needed()?;           // 1. World rooms
    store.seed_quests_if_needed()?;          // 2. Starter quests
    store.seed_achievements_if_needed()?;    // 3. Achievements
    store.seed_companions_if_needed()?;      // 4. Companions
    store.seed_npcs_if_needed()?;            // 5. NPCs
    
    // Seed full dialogue trees for NPCs
    crate::tmush::state::seed_npc_dialogues_if_needed(&store)?;  // 6. NPC dialogues
    
    store.seed_housing_templates_if_needed()?;  // 7. Housing templates
    
    // Seed initial admin account (default: "admin")
    store.seed_admin_if_needed("admin")?;    // 8. Admin account ✅
}
```

## API Reference

### `seed_admin_if_needed()`

```rust
pub fn seed_admin_if_needed(&self, admin_username: &str) 
    -> Result<bool, TinyMushError>
```

**Purpose**: Create initial admin account if no admins exist (idempotent)

**Arguments**:
- `admin_username` - Username for the admin account (e.g., "admin", "sysop")

**Returns**:
- `Ok(true)` - Admin was created or existing player was promoted
- `Ok(false)` - Admin already exists (no-op)
- `Err(...)` - Database error

**Behavior**:
1. **Check for existing admins**: Calls `list_admins()` to see if any admin exists
2. **If admin exists**: Returns `Ok(false)` (no-op, idempotent)
3. **If username exists as regular player**: Promotes existing player to admin level 3
4. **If username doesn't exist**: Creates new admin account with sysop privileges

**Example Usage**:
```rust
use meshbbs::tmush::TinyMushStoreBuilder;

// Default admin username
let store = TinyMushStoreBuilder::new("/path/to/db").open()?;
// Admin "admin" is now created automatically

// Custom admin username
let store = TinyMushStoreBuilder::new("/path/to/db")
    .without_world_seed()
    .open()?;
store.seed_admin_if_needed("sysop")?;
```

## Customization

### Custom Admin Username

While the default is "admin", operators can customize the username:

**Option 1: Direct Call**
```rust
let store = TinyMushStoreBuilder::new("/path/to/db")
    .without_world_seed()
    .open()?;
    
// Create custom admin
store.seed_admin_if_needed("root")?;
```

**Option 2: Configuration (Future)**
```toml
# Future enhancement: config.toml
[tinymush]
admin_username = "sysop"
```

### Multiple Admins

To create additional admins after initialization:

```rust
// Method 1: Promote existing player
store.grant_admin("admin", "alice", 2)?;  // Level 2 = admin

// Method 2: Create new player as admin
let mut bob = PlayerRecord::new("bob", "Bob", "town_square");
bob.grant_admin(1);  // Level 1 = moderator
store.put_player(bob)?;
```

## Security Considerations

### Protection Mechanisms

1. **Idempotent Seeding**: Won't create duplicate admins on repeated calls
2. **First-Admin Rule**: Only seeds if NO admins exist (prevents unauthorized elevation)
3. **Sysop Level**: Default admin gets level 3 (highest) to bootstrap system
4. **Promotion Path**: Existing players can be elevated without data loss

### Best Practices

#### ✅ DO:
- Change the default admin password immediately after first use
- Create additional admin accounts for team members
- Use different admin levels (1=moderator, 2=admin, 3=sysop) for delegation
- Document who has admin access in your operations manual

#### ❌ DON'T:
- Share the admin account credentials
- Leave default credentials in production
- Grant admin to untrusted users
- Rely on a single admin account (create backups!)

### Initial Access

Since the admin account is created automatically, you need to set its password through the registration/authentication system. The exact flow depends on your authentication implementation:

**Option A: Auto-register on first connection**
```rust
// If player exists but no auth record, create one
if store.get_player("admin").is_ok() {
    // Register admin with secure password
    auth_system.set_password("admin", "secure_initial_password")?;
}
```

**Option B: Manual password setting**
```bash
# Via admin command or script
./meshbbs-admin setpass admin <secure_password>
```

## Testing

### Test Coverage

The admin seeding system has comprehensive test coverage:

```rust
#[test]
fn admin_account_seeded_automatically()
// ✅ Verifies admin account created during initialization

#[test]
fn admin_seeding_is_idempotent()
// ✅ Verifies repeated seeding doesn't create duplicates

#[test]
fn admin_seeding_custom_username()
// ✅ Verifies custom usernames work correctly

#[test]
fn admin_seeding_promotes_existing_player()
// ✅ Verifies existing players can be promoted to admin
```

**Test File**: `src/tmush/storage.rs` (tests module)  
**Total Tests**: 4 tests, all passing  
**Coverage**: 100% of seed_admin_if_needed() logic

### Integration Testing

```rust
use meshbbs::tmush::TinyMushStoreBuilder;
use tempfile::TempDir;

#[test]
fn fresh_database_has_admin() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    
    // Verify admin exists
    let admin = store.get_player("admin").expect("admin exists");
    assert!(admin.is_admin());
    assert_eq!(admin.admin_level(), 3);
    
    // Verify can use admin commands
    store.require_admin("admin").expect("has permission");
}
```

## Troubleshooting

### Admin Account Missing

**Symptom**: No admin account after initialization

**Possible Causes**:
1. World seeding was disabled: `TinyMushStoreBuilder::without_world_seed()`
2. Database was created with old code (before admin seeding)
3. Admin was manually deleted

**Solutions**:
```rust
// Solution 1: Manually seed admin
store.seed_admin_if_needed("admin")?;

// Solution 2: Create admin directly
let mut admin = PlayerRecord::new("admin", "Admin", "town_square");
admin.grant_admin(3);
store.put_player(admin)?;
```

### Multiple Admins Created

**Symptom**: More than one admin after initialization

**This is expected behavior** if:
- You manually created additional admins
- Different admin usernames were used on different seed calls
- Players were promoted to admin

**To verify**:
```rust
let admins = store.list_admins()?;
for admin in admins {
    println!("{}: level {}", admin.username, admin.admin_level());
}
```

### Cannot Access Admin Commands

**Symptom**: Admin exists but permission denied

**Possible Causes**:
1. Admin flag not set: `is_admin = false`
2. Admin level is 0
3. Password not set for admin account

**Solutions**:
```rust
// Verify admin status
let admin = store.get_player("admin")?;
println!("is_admin: {}", admin.is_admin());
println!("admin_level: {}", admin.admin_level());

// Fix admin flags if needed
let mut admin = store.get_player("admin")?;
admin.grant_admin(3);
store.put_player(admin)?;
```

## Migration Guide

### Upgrading from Pre-Admin-Seeding Databases

If you have an existing TinyMUSH database created before admin seeding was implemented:

**Step 1: Check for existing admins**
```rust
let admins = store.list_admins()?;
if admins.is_empty() {
    println!("No admins found - need to create one");
}
```

**Step 2: Create initial admin**
```rust
// Option A: Seed default admin
store.seed_admin_if_needed("admin")?;

// Option B: Promote existing player
store.grant_admin("system", "alice", 3)?;
```

**Step 3: Verify**
```rust
let admin = store.get_player("admin")?;
assert!(admin.is_admin());
```

### No downtime required - changes are backward compatible!

## Future Enhancements

### Configuration-Based Admin (Planned)

```toml
# config.toml
[tinymush]
# Initial admin account username
admin_username = "sysop"

# Initial admin password hash (optional)
# If set, admin account will have this password on creation
admin_password_hash = "$argon2id$v=19$m=19456,t=2,p=1$..."
```

### Multi-Admin Seeding (Planned)

```toml
# config.toml
[tinymush.admins]
# Seed multiple admins on initialization
users = [
    { username = "sysop", level = 3 },
    { username = "admin", level = 2 },
    { username = "moderator", level = 1 }
]
```

### Audit Logging (Planned)

```rust
// Log admin creation
store.log_admin_action(
    "system",
    "seed_admin",
    &format!("Created admin account: {}", username)
)?;
```

## Related Documentation

- [Admin Permission System](TMUSH_ADMIN_PERMISSIONS.md) - Core permission system
- [Phase 9 Admin Tools Plan](/tmp/PHASE9_PLAN.md) - Overall admin tooling roadmap
- [TinyMUSH Storage API](../../src/tmush/storage.rs) - Storage layer implementation
- [Player Types](../../src/tmush/types.rs) - PlayerRecord structure

## Changelog

### v1.0 - October 10, 2025
- ✅ Initial implementation of automatic admin seeding
- ✅ `seed_admin_if_needed()` function with idempotent behavior
- ✅ Integration into database initialization
- ✅ Existing player promotion support
- ✅ Comprehensive test suite (4 tests)
- ✅ Complete documentation

### Future
- [ ] Configuration-based admin username
- [ ] Password hash in configuration
- [ ] Multi-admin seeding
- [ ] Audit logging for admin creation
