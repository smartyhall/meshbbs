# TinyMUSH Admin Commands

**Status**: Phase 9.2 Complete  
**Last Updated**: 2025-10-10

## Overview

This document describes the administrative commands available in TinyMUSH for managing user privileges and viewing system administrators. These commands provide a secure, hierarchical permission system with three admin levels: Moderator (1), Admin (2), and Sysop (3).

## Command Reference

### @ADMIN - Show Admin Status

**Syntax**: `@ADMIN`

**Permission**: Available to all players

**Description**: Displays your admin status, level, and available commands based on your permission level.

**Example Output (Admin)**:
```
🛡️  ADMIN STATUS

Player: Alice
Username: alice

✅ Admin Status: ACTIVE
🔐 Admin Level: 2 (Admin)

Available Admin Commands:
  @ADMINS - List all administrators
  @SETADMIN <player> <level> - Grant admin privileges
  @REMOVEADMIN <player> - Revoke admin privileges

Total Administrators: 3
```

**Example Output (Non-Admin)**:
```
🛡️  ADMIN STATUS

Player: Bob
Username: bob

❌ Admin Status: NOT ADMIN

You do not have administrative privileges.
Contact a system administrator if you need admin access.
```

---

### @SETADMIN - Grant Admin Privileges

**Syntax**: `@SETADMIN <player> <level>`

**Aliases**: None

**Permission**: Admin level 2 or higher required

**Description**: Grants administrative privileges to the specified player with the given level. The caller cannot grant a level higher than their own.

**Arguments**:
- `<player>`: Username of the player to grant admin to (case-insensitive)
- `<level>`: Admin level to grant (0-3)
  - `0`: None (revoked) - use @REMOVEADMIN instead
  - `1`: Moderator
  - `2`: Admin
  - `3`: Sysop

**Examples**:
```
@SETADMIN alice 2
@SETADMIN bob 1
@SETADMIN charlie 3
```

**Success Output**:
```
✅ SUCCESS

Granted Admin admin privileges to 'alice'.

Admin Level: 2 (Admin)

The change is effective immediately.
```

**Error Cases**:
- **Not Admin**: `⛔ Permission denied: Not an administrator`
- **Insufficient Level**: `⛔ Permission denied: Insufficient admin level. Only level 2+ administrators can grant admin privileges.`
- **Level Too High**: `⛔ Permission denied: Cannot grant level 3 admin. Your admin level is 2. You can only grant levels up to your own level.`
- **Player Not Found**: `❌ Failed to grant admin: record not found: player: alice`

---

### @REMOVEADMIN - Revoke Admin Privileges

**Syntax**: `@REMOVEADMIN <player>`

**Aliases**: `@REVOKEADMIN`

**Permission**: Admin level 2 or higher required

**Description**: Revokes administrative privileges from the specified player, demoting them to a regular user. You cannot revoke your own admin privileges (self-protection).

**Arguments**:
- `<player>`: Username of the player to revoke admin from (case-insensitive)

**Examples**:
```
@REMOVEADMIN alice
@REVOKEADMIN bob
```

**Success Output**:
```
✅ SUCCESS

Revoked admin privileges from 'alice'.

They are now a regular user.

The change is effective immediately.
```

**Error Cases**:
- **Not Admin**: `⛔ Permission denied: Not an administrator`
- **Insufficient Level**: `⛔ Permission denied: Insufficient admin level. Only level 2+ administrators can revoke admin privileges.`
- **Self-Revocation**: `⛔ Cannot revoke your own admin privileges. Have another administrator revoke your access if needed.`
- **Player Not Found**: `❌ Failed to revoke admin: record not found: player: alice`

---

### @ADMINS - List All Administrators

**Syntax**: `@ADMINS`

**Aliases**: `@ADMINLIST`

**Permission**: Available to all players (public command)

**Description**: Lists all players with administrative privileges, sorted by admin level (highest first) and then by username. This is a transparency feature - everyone can see who the administrators are.

**Example**:
```
@ADMINS
@ADMINLIST
```

**Output**:
```
🛡️  SYSTEM ADMINISTRATORS

Total: 3

  [3] Sysop     - Admin (Admin) (you)
  [2] Admin     - Alice
  [1] Moderator - Bob

Levels: 1=Moderator, 2=Admin, 3=Sysop
```

**Notes**:
- The current player is marked with "(you)" indicator
- Sorted by level (descending), then by username (alphabetical)
- Useful for players to know who can help with admin requests
- Promotes transparent governance

---

## Admin Levels

### Level 0: Not Admin (Default)
- Regular player with no administrative privileges
- Can view admin list (@ADMINS)
- Cannot access admin commands

### Level 1: Moderator
- Entry-level administrative privileges
- Can view admin list (@ADMINS)
- Can check own admin status (@ADMIN)
- Future: Basic moderation commands (kick, mute, warn)

### Level 2: Admin
- Mid-level administrative privileges
- All Moderator commands
- Can grant/revoke Moderator and Admin privileges (@SETADMIN, @REMOVEADMIN)
- Cannot grant Sysop privileges
- Future: Room editing, NPC editing, player management

### Level 3: Sysop (System Operator)
- Full administrative privileges
- All Admin commands
- Can grant any admin level up to Sysop (3)
- Full system access including:
  - Room editing (@EDITROOM)
  - NPC editing (@EDITNPC)
  - Dialogue management (@DIALOG)
  - Future: World configuration, backups, advanced tools

---

## Permission Hierarchy

```
Level 3 (Sysop)
  ├─ Can grant: Moderator, Admin, Sysop
  ├─ Can revoke: Moderator, Admin, Sysop (except self)
  └─ Full admin commands

Level 2 (Admin)
  ├─ Can grant: Moderator, Admin
  ├─ Can revoke: Moderator, Admin (except self)
  └─ Most admin commands

Level 1 (Moderator)
  ├─ Cannot grant/revoke
  └─ Basic moderation commands

Level 0 (Regular)
  └─ No admin commands
```

---

## Security Features

### Self-Protection
- Administrators cannot revoke their own admin privileges
- This prevents accidental lockouts
- Another administrator must revoke access if needed

### Level Validation
- Cannot grant a level higher than your own
- Level 2 admins cannot create Sysops
- Prevents privilege escalation

### Username Normalization
- Usernames are automatically lowercased for storage
- Case-insensitive lookups prevent confusion
- Consistent with TinyMUSH storage conventions

### Permission Checking
- All admin commands verify caller has appropriate level
- Grant/revoke commands require level 2+
- Clear error messages explain permission denials

---

## Implementation Details

### Code Location
- **Module**: `src/tmush/commands.rs`
- **Enum**: `TinyMushCommand` (lines 119-133)
- **Parser**: `parse_tinymush_command()` (lines 886-905)
- **Handlers**: `handle_admin()`, `handle_set_admin()`, `handle_remove_admin()`, `handle_admins()` (lines 4701-4913)
- **Tests**: `tests/tmush_admin_command_handlers.rs` (8 integration tests)

### Storage
- Admin privileges stored in `PlayerRecord`
- Fields: `is_admin: bool`, `admin_level: u8`
- Persisted in Sled database under `players:*` namespace
- Changes are effective immediately (no caching issues)

### Testing
All commands have comprehensive integration tests covering:
- Success cases
- Permission denied cases
- Self-protection (cannot revoke self)
- Level validation (cannot grant higher than own level)
- Public viewing (non-admins can view admin list)
- Error handling (player not found, etc.)

**Test Results**: 8/8 passing ✅

---

## Future Enhancements (Phase 9.3+)

### Player Monitoring Commands
- `/PLAYERS` - list all online players
- `/WHERE <player>` - locate player
- `/GOTO <player|room>` - teleport admin
- `/SUMMON <player>` - teleport player to admin
- `/BOOT <player>` - disconnect player
- `/BAN <player>` - ban player access

### Admin Logging
- Log all admin commands with timestamps
- Track who granted/revoked privileges
- Audit trail for governance
- Security monitoring

### Advanced Features
- Temporary admin grants (time-limited)
- Role-based permissions (custom permission sets)
- Admin groups/teams
- Delegation (deputy admin system)

---

## Related Documentation

- **Admin Permissions**: `docs/development/TMUSH_ADMIN_PERMISSIONS.md`
- **Admin Bootstrap**: `docs/development/TMUSH_ADMIN_BOOTSTRAP.md`
- **Implementation Plan**: `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md`
- **Design Document**: `docs/development/MUD_MUSH_DESIGN.md`

---

## Change Log

### 2025-10-10 - Phase 9.2 Complete
- Implemented all four admin commands (@ADMIN, @SETADMIN, @REMOVEADMIN, @ADMINS)
- Added comprehensive permission checking
- Implemented level validation and self-protection
- Created 8 integration tests (all passing)
- Enhanced rustdoc documentation for API reference
- Created this user-facing documentation

### 2025-10-09 - Phase 9.1 Complete
- Implemented admin permission system
- Added automatic admin bootstrap
- Created storage methods for admin operations

---

## Questions or Issues?

For questions about admin commands or to report issues:
- Check the integration tests: `tests/tmush_admin_command_handlers.rs`
- Review the handler implementations: `src/tmush/commands.rs`
- Consult the design document: `docs/development/MUD_MUSH_DESIGN.md`
- Contact the development team via GitHub issues
