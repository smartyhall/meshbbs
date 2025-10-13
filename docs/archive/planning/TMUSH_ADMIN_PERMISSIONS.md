# TinyMUSH Admin Permission System

**Status**: ✅ Complete  
**Version**: 1.0  
**Date**: October 10, 2025

> **Note**: Fresh databases automatically seed an admin account! See [Admin Bootstrap Documentation](TMUSH_ADMIN_BOOTSTRAP.md) for details on the automatic admin account creation.

## Overview

The TinyMUSH admin permission system provides a foundation for administrative commands and moderation tools. It introduces an `is_admin` flag and optional `admin_level` hierarchy to PlayerRecord.

Every fresh TinyMUSH database automatically creates an initial admin account (default username: "admin") with sysop-level privileges during initialization.

## Features

### Player Fields

```rust
pub struct PlayerRecord {
    // ... existing fields ...
    
    /// Admin/moderator flag (grants access to admin commands)
    #[serde(default)]
    pub is_admin: bool,
    
    /// Admin level for future role hierarchy (0=none, 1=moderator, 2=admin, 3=sysop)
    #[serde(default)]
    pub admin_level: Option<u8>,
}
```

### Admin Levels

| Level | Role       | Description                                    |
|-------|------------|------------------------------------------------|
| 0     | Regular    | No admin privileges                            |
| 1     | Moderator  | Basic moderation (kick, mute, view logs)       |
| 2     | Admin      | Full admin (teleport, summon, grant mod)       |
| 3     | Sysop      | Super admin (grant admin, system config)       |

### Helper Methods

#### PlayerRecord Methods

```rust
impl PlayerRecord {
    /// Check if player has admin privileges
    pub fn is_admin(&self) -> bool;
    
    /// Grant admin privileges to this player
    pub fn grant_admin(&mut self, level: u8);
    
    /// Revoke admin privileges from this player
    pub fn revoke_admin(&mut self);
    
    /// Get admin level (0 if not admin)
    pub fn admin_level(&self) -> u8;
}
```

#### TinyMushStore Methods

```rust
impl TinyMushStore {
    /// Check if a player has admin privileges
    pub fn is_admin(&self, username: &str) -> Result<bool, TinyMushError>;
    
    /// Require admin privileges, return PermissionDenied error if not admin
    pub fn require_admin(&self, username: &str) -> Result<(), TinyMushError>;
    
    /// Grant admin privileges to a player (requires caller to be admin)
    pub fn grant_admin(&self, granter: &str, target: &str, level: u8) 
        -> Result<(), TinyMushError>;
    
    /// Revoke admin privileges from a player (requires caller to be admin)
    pub fn revoke_admin(&self, revoker: &str, target: &str) 
        -> Result<(), TinyMushError>;
    
    /// List all admin players
    pub fn list_admins(&self) -> Result<Vec<PlayerRecord>, TinyMushError>;
}
```

## Usage Examples

### Checking Admin Status

```rust
// Check if player is admin
if store.is_admin("alice")? {
    println!("Alice is an admin");
}

// Require admin (throws error if not admin)
store.require_admin("alice")?;
```

### Granting Admin

```rust
// Grant level 2 admin to alice (caller must be admin)
store.grant_admin("sysop", "alice", 2)?;

// Grant moderator (level 1) to bob
store.grant_admin("alice", "bob", 1)?;
```

### Revoking Admin

```rust
// Revoke admin from bob (caller must be admin)
store.revoke_admin("alice", "bob")?;

// Cannot revoke own admin (will fail)
let result = store.revoke_admin("alice", "alice");
assert!(result.is_err());
```

### Listing Admins

```rust
let admins = store.list_admins()?;
for admin in admins {
    println!("{} - Level {}", admin.username, admin.admin_level());
}
```

## Error Handling

### PermissionDenied Error

```rust
pub enum TinyMushError {
    // ... other variants ...
    
    /// Permission denied (admin-only command)
    #[error("permission denied: {0}")]
    PermissionDenied(String),
}
```

**When thrown**:
- Non-admin tries to use admin command
- Admin tries to revoke own privileges
- Insufficient level for specific operation (future)

## Security Considerations

### Protection Mechanisms

1. **Self-Protection**: Admins cannot revoke their own privileges
2. **Permission Checks**: All admin operations verify caller permissions
3. **Audit Trail**: Admin operations update `updated_at` timestamp
4. **Persistent Storage**: Admin status survives server restarts

### Best Practices

1. **Minimum Viable Privilege**: Grant lowest level needed
2. **Regular Audits**: Use `list_admins()` to review admin list
3. **Sysop Protection**: Reserve level 3 for system operators only
4. **Documentation**: Log all admin grants/revokes

## Implementation Details

### Backward Compatibility

- Uses `#[serde(default)]` for new fields
- Existing players load with `is_admin = false`
- No migration needed for existing databases

### Storage

- Admin status stored in PlayerRecord
- No separate admin tree needed
- Indexed via normal player lookups

### Performance

- Admin checks: O(1) - single player lookup
- List admins: O(n) - scans all players (cached in future)

## Testing

Comprehensive test suite covers:

✅ New players are not admin by default  
✅ Grant admin with level  
✅ Revoke admin  
✅ Permission checks for grant/revoke  
✅ Cannot revoke self  
✅ List all admins  
✅ Admin level hierarchy  
✅ require_admin helper

**Test File**: `tests/tmush_admin_commands.rs` (9 tests, all passing)

## Future Enhancements

### Phase 9.2+ (Planned)

1. **Admin Commands**:
   - `@ADMIN` - Show admin status
   - `@SETADMIN <player> <level>` - Grant admin
   - `@REMOVEADMIN <player>` - Revoke admin
   - `@ADMINS` - List all admins

2. **Logging**:
   - Record all admin actions
   - Audit trail for grants/revokes
   - Who granted whom, when

3. **Level Enforcement**:
   - Restrict commands by level
   - Level 1 = moderator commands only
   - Level 2 = full admin commands
   - Level 3 = system commands

4. **Performance**:
   - Cache admin list in memory
   - Invalidate on grant/revoke
   - O(1) admin checks

## Related Documentation

- [Admin Bootstrap System](TMUSH_ADMIN_BOOTSTRAP.md) - **Automatic admin account creation**
- [Phase 9 Admin Tools Plan](/tmp/PHASE9_PLAN.md)
- [TinyMUSH Storage API](../src/tmush/storage.rs)
- [TinyMUSH Types](../src/tmush/types.rs)
- [TinyMUSH Errors](../src/tmush/errors.rs)

## Changelog

### v1.0 - October 10, 2025
- ✅ Added `is_admin` and `admin_level` fields to PlayerRecord
- ✅ Implemented helper methods (is_admin, grant_admin, revoke_admin, admin_level)
- ✅ Added storage methods (is_admin, require_admin, grant_admin, revoke_admin, list_admins)
- ✅ Automatic admin account seeding on database initialization
- ✅ Added PermissionDenied error type
- ✅ Comprehensive test suite (9 tests)
- ✅ Documentation complete
