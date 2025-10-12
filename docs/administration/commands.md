# Admin Commands Reference

Complete reference for administrative commands in MeshBBS TinyMUSH mode.

## Command Overview

Admin commands are prefixed with `/` or `@` and require specific permission levels:

| Level | Role | Capabilities |
|-------|------|-------------|
| 0 | Player | No admin access |
| 1 | Moderator | Player monitoring, basic moderation |
| 2 | Admin | Full moderation, backups, configuration |
| 3 | Sysop | All powers, restoration, user management |

## Permission Management

### View Admin Status

```
@ADMIN
```

Shows your current admin level and available commands.

### Grant Admin Privileges

```
@SETADMIN <username> <level>
```

**Permission**: Admin Level 2+  
**Levels**: 0 (none), 1 (moderator), 2 (admin), 3 (sysop)

**Example**:
```
@SETADMIN alice 2          # Grant admin privileges
@SETADMIN bob 1            # Grant moderator privileges
```

### Revoke Admin Privileges

```
@REMOVEADMIN <username>
@REVOKEADMIN <username>
```

**Permission**: Admin Level 2+

### List All Administrators

```
@ADMINS
@ADMINLIST
```

**Permission**: Public (all players)

Shows all users with admin privileges and their levels.

## Player Monitoring

### List All Players

```
/PLAYERS
```

**Permission**: Admin Level 1+

Shows all players with online status and current location.

### Teleport to Player/Room

```
/GOTO <player>
/GOTO <room_name>
```

**Permission**: Admin Level 1+

Teleport to a player's location or specific room for monitoring.

### Locate Player

```
/WHERE <player>
```

**Permission**: Admin Level 1+

Find a player's current location without teleporting.

## Backup & Recovery

### Create Backup

```
/BACKUP [name]
```

**Permission**: Admin Level 2+

Create manual backup with optional name. Manual backups are never auto-deleted.

### List Backups

```
/LISTBACKUPS
```

**Permission**: Admin Level 2+

View all backups with metadata and verification status.

### Verify Backup

```
/VERIFYBACKUP <backup_id>
```

**Permission**: Admin Level 2+

Verify backup integrity using SHA256 checksum.

### Delete Backup

```
/DELETEBACKUP <backup_id>
```

**Permission**: Admin Level 2+

Delete specific backup. Manual backups require confirmation.

### Restore Backup

```
/RESTORE <backup_id>
```

**Permission**: Admin Level 3 (Sysop only)

Shows restoration instructions. Requires offline operation.

### Configure Auto-Backups

```
/BACKUPCONFIG status
/BACKUPCONFIG enable
/BACKUPCONFIG disable
/BACKUPCONFIG frequency <hourly|2h|4h|6h|12h|daily>
```

**Permission**: Admin Level 2+

Configure automatic backup scheduling.

## Clone Monitoring

### List Player Clones

```
/LISTCLONES [player]
```

**Permission**: Admin Level 1+

View all cloned objects owned by a player with genealogy.

### Clone Statistics

```
/CLONESTATS
```

**Permission**: Admin Level 1+

Server-wide cloning statistics and abuse detection.

## World Management

### Currency Conversion

```
/CONVERT_CURRENCY <decimal|multitier> [--dry-run]
```

**Permission**: Admin Level 3 (Sysop only)

Convert all currency in the world. Use `--dry-run` to preview changes.

### Edit Room Description

```
@EDITROOM <room_id> <description>
```

**Permission**: Admin Level 2+

Edit description of any room in the world.

### Edit NPC Properties

```
@EDITNPC <npc_id> <field> <value>
```

**Permission**: Admin Level 2+

Modify NPC properties (dialogue, behavior, etc.).

### Manage NPC Dialogues

```
@DIALOG <npc> <subcommand> [args]
```

**Permission**: Admin Level 2+

Manage NPC dialogue trees and conversation options.

### View Abandoned Housing

```
@LISTABANDONED
```

**Permission**: Admin Level 2+

List housing units at risk of abandonment or already abandoned.

## Configuration

### View Configuration

```
@GETCONFIG [field]
```

**Permission**: Admin Level 1+

View world configuration. Omit field to see all settings.

### Set Configuration

```
@SETCONFIG <field> <value>
```

**Permission**: Admin Level 2+

Modify world configuration values.

## Builder Management

### View Builder Status

```
/BUILDER
```

**Permission**: Public (shows your own status)

Display your builder privileges and level.

### Grant Builder Privileges

```
/SETBUILDER <player> <level>
```

**Permission**: Admin Level 2+  
**Levels**: 0 (none), 1 (apprentice), 2 (builder), 3 (architect)

### Revoke Builder Privileges

```
/REMOVEBUILDER <player>
```

**Permission**: Admin Level 2+

### List All Builders

```
/BUILDERS
```

**Permission**: Public

Shows all users with builder privileges.

## Debug & Diagnostics

### Debug Information

```
/DEBUG <target>
```

**Permission**: Admin Level 2+

View detailed diagnostic information about objects, rooms, or systems.

## Best Practices

### Permission Levels

- **Moderator (1)**: For trusted players who help with basic moderation
- **Admin (2)**: For staff who manage world and technical operations  
- **Sysop (3)**: For server owners only (full destructive powers)

### Backup Strategy

- Create manual backup before major changes
- Enable automatic backups for daily safety
- Verify backups weekly
- Test restoration procedures periodically

### Monitoring

- Review `/PLAYERS` regularly for activity patterns
- Check `/CLONESTATS` weekly for abuse
- Monitor `@LISTABANDONED` monthly to reclaim housing

### Security

- All admin commands are logged
- Use `/GOTO` and `/WHERE` transparently
- Document configuration changes
- Review admin list regularly (`@ADMINS`)

## Related Documentation

- [Backup & Recovery Guide](backup-recovery.md) - Detailed backup procedures
- [Moderation Guide](moderation.md) - Content moderation tools
- [User Management](user-management.md) - Player account management
- [Daemon Mode](daemon-mode.md) - Server management

---

**Version**: 1.0.23  
**Last Updated**: 2024-10-12  
**Applies to**: MeshBBS TinyMUSH Mode
