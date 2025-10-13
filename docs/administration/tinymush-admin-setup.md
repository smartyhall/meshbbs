# TinyMUSH Admin Setup

This guide explains how the TinyMUSH admin system works and how to manage administrative privileges.

## Overview

The TinyMUSH admin system is **automatically synchronized** with the BBS sysop configuration. When the TinyMUSH database is initialized, it creates an admin account using the username specified in your `config.toml` file.

## Automatic Admin Creation

### During Initial Setup

When you run `meshbbs` for the first time (or when the TinyMUSH database is created), the system:

1. Reads the `sysop` username from `config.toml` (under `[bbs]` section)
2. Creates a TinyMUSH admin account with that username
3. Grants level 3 (Sysop) privileges to that account

**Example configuration:**

```toml
[bbs]
sysop = "martin"
sysop_password_hash = "..."
```

**Result:** A TinyMUSH admin account named `"martin"` is automatically created with full admin privileges.

### How It Works

```
BBS Login          TinyMUSH Entry           Admin Status
────────────────   ──────────────────────   ────────────────────
User "martin"  →   Player "martin"      →   ✅ Admin Level 3 (Sysop)
  (BBS sysop)        (auto-created)            (auto-granted)

User "alice"   →   Player "alice"       →   ❌ No admin (Level 0)
  (regular user)     (auto-created)            (regular player)
```

## Admin Levels

| Level | Role | Capabilities |
|-------|------|-------------|
| **0** | Player | No admin access |
| **1** | Moderator | Player monitoring, basic moderation |
| **2** | Admin | Full moderation, backups, configuration |
| **3** | Sysop | All powers, restoration, user management |

## Granting Admin to Other Users

Once you're logged in as the sysop, you can grant admin privileges to other users:

### Command

```
@SETADMIN <username> <level>
```

### Examples

```
@SETADMIN alice 2          # Grant admin privileges to alice
@SETADMIN bob 1            # Grant moderator privileges to bob
@SETADMIN charlie 3        # Grant full sysop privileges to charlie
```

### Permission Requirements

- **You must be logged in** as a user with admin level 2 or higher
- **You cannot grant a level higher than your own**
- Example: Level 2 admin can only grant levels 0-2, not level 3

## Revoking Admin Privileges

### Commands

```
@REMOVEADMIN <username>
@REVOKEADMIN <username>
```

### Example

```
@REMOVEADMIN alice         # Remove admin privileges from alice
```

**Permission**: Admin Level 2+

## Viewing Admin Status

### Check Your Own Status

```
@ADMIN
```

Shows your current admin level and available commands.

### List All Administrators

```
@LISTADMINS
```

Shows all users with admin privileges and their levels.

## Security Features

### Reserved Username Protection

The following usernames are **reserved** and cannot be used by regular users for BBS registration:

- `admin`
- `administrator`
- `root`
- `system`
- `sysop`
- `operator`
- And many more...

This prevents unauthorized users from creating accounts with privileged names.

### Automatic Authentication

- TinyMUSH players are **automatically created** when BBS users enter the game
- Player usernames **match BBS usernames** (session-based authentication)
- Admin privileges are **granted based on configured sysop username**
- No separate TinyMUSH password system (uses BBS authentication)

## Changing the Sysop User

If you need to change which user has sysop privileges:

### Option 1: Change Config (Before First Run)

Edit `config.toml` **before** initializing the database:

```toml
[bbs]
sysop = "newadmin"        # Change this
sysop_password_hash = "..."
```

Then initialize the system. The TinyMUSH admin will be created as `newadmin`.

### Option 2: Grant Admin (After Setup)

If the database already exists:

1. Log in as the current sysop
2. Enter TinyMUSH
3. Grant sysop to the new user:
   ```
   @SETADMIN newuser 3
   ```
4. Optionally revoke from old user:
   ```
   @REMOVEADMIN olduser
   ```

## Troubleshooting

### "I can't access admin commands"

**Check your BBS username:**
```
# In BBS main menu
WHO
```

If your BBS username doesn't match the `sysop` in config.toml, you won't have auto-granted admin.

**Solution:** Have the current sysop grant you admin, or log in as the sysop user.

### "The admin account doesn't exist"

**Cause:** The TinyMUSH database was created before the BBS sysop user logged in for the first time.

**Solution:**
1. Log in to BBS as the sysop user (from config.toml)
2. Enter TinyMUSH (creates player automatically)
3. The system should recognize you as admin

If still not working, check that your BBS username **exactly matches** `sysop` in config.toml (case-sensitive).

### "I want to use a different admin username"

**Before database creation:**
1. Edit `config.toml` and change `sysop = "desired_username"`
2. Delete `data/tinymush/` directory (if exists)
3. Restart meshbbs (database will be recreated)

**After database exists:**

Use `@SETADMIN` to grant privileges to the new user, then optionally revoke from the old admin.

## Best Practices

1. ✅ **Use a strong password** for your BBS sysop account (set via `meshbbs sysop-passwd`)
2. ✅ **Grant admin sparingly** - only to trusted users
3. ✅ **Use level 1 (moderator)** for users who just need monitoring capabilities
4. ✅ **Document who has admin** and why
5. ✅ **Revoke admin** when no longer needed
6. ❌ **Don't share the sysop password**
7. ❌ **Don't grant level 3** unless absolutely necessary

## See Also

- [Admin Commands Reference](commands.md) - Complete command documentation
- [First Run Guide](../getting-started/first-run.md) - Initial setup
- [Configuration Guide](setup.md) - config.toml documentation
