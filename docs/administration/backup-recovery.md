# Backup & Recovery Guide

Complete guide to backing up and restoring your MeshBBS TinyMUSH world database.

## Overview

The backup system provides automated and manual backup capabilities with:
- **Compressed archives** (tar.gz format with SHA256 verification)
- **Retention policies** (automatic cleanup of old backups)
- **Manual backups** (never auto-deleted)
- **Integrity verification** (checksum validation)
- **Offline restoration** (safe world recovery)

## Permission Requirements

| Operation | Permission Level | Notes |
|-----------|-----------------|-------|
| Create backup | Admin Level 2+ | Admin or Sysop |
| List backups | Admin Level 2+ | Admin or Sysop |
| Verify backup | Admin Level 2+ | Admin or Sysop |
| Delete backup | Admin Level 2+ | Admin or Sysop |
| Restore backup | Admin Level 3 | Sysop only |
| Configure auto-backups | Admin Level 2+ | Admin or Sysop |

## Backup Storage

- **Database path**: `data/tinymush/`
- **Backup directory**: `data/backups/`
- **Backup format**: `backup_YYYYMMDD_HHMMSS_mmm.tar.gz`
- **Metadata file**: `data/backups/backups.json`

## Creating Backups

### Manual Backup

Create a manual backup with optional name:

```
/BACKUP
/BACKUP "pre-update"
/BACKUP "before-event-changes"
```

**Output:**
```
💾 Backup created successfully!
ID: backup_20240315_143022_123
Name: pre-update
Size: 2.5 MB
Checksum: a3f8b9c1... (SHA256)

Use /LISTBACKUPS to view all backups
Use /RESTORE backup_20240315_143022_123 to restore from this backup
```

**Notes:**
- Manual backups are never automatically deleted
- Names are optional but recommended for important backups
- Backup ID is based on timestamp (YYYYMMDD_HHMMSS_milliseconds)

### Automatic Backups

Configure scheduled backups:

```
/BACKUPCONFIG status            # View current configuration
/BACKUPCONFIG enable            # Enable automatic backups
/BACKUPCONFIG disable           # Disable automatic backups
/BACKUPCONFIG frequency daily   # Set backup frequency
```

**Supported frequencies:**
- `hourly` - Every hour
- `2h` - Every 2 hours
- `4h` - Every 4 hours
- `6h` - Every 6 hours
- `12h` - Every 12 hours
- `daily` - Once per day (default)

**Example output:**
```
⚙️ Automatic Backup Configuration
Status: enabled
Frequency: daily
Last automatic backup: 2024-03-15 06:00:00 UTC
Next scheduled backup: 2024-03-16 06:00:00 UTC
```

## Retention Policy

The system automatically manages backup retention to prevent disk space issues:

| Backup Type | Retention | Notes |
|-------------|-----------|-------|
| Manual | Forever | Never auto-deleted |
| Daily | 7 backups | 1 week of history |
| Weekly | 4 backups | 1 month of history |
| Monthly | 12 backups | 1 year of history |

**Automatic cleanup:**
- Runs after each backup creation
- Only affects automatic backups
- Manual backups are always preserved
- Keeps the most recent backups in each category

## Listing Backups

View all available backups:

```
/LISTBACKUPS
```

**Output:**
```
📦 Available Backups (3 total, 8.4 MB)

ID: backup_20240315_143022_123
Name: pre-update
Created: 2024-03-15 14:30:22 UTC (2 hours ago)
Size: 2.5 MB
Type: Manual
Checksum: a3f8b9c1d2e4f5a6... (verified ✅)

ID: backup_20240315_060000_456
Created: 2024-03-15 06:00:00 UTC (8 hours ago)
Size: 2.4 MB
Type: Daily (Automatic)
Checksum: b2c3d4e5f6a7b8c9... (verified ✅)

ID: backup_20240308_060000_789
Created: 2024-03-08 06:00:00 UTC (7 days ago)
Size: 3.5 MB
Type: Weekly (Automatic)
Checksum: c3d4e5f6a7b8c9d0... (not verified ⏳)

Use /VERIFYBACKUP <id> to verify integrity
Use /RESTORE <id> to restore from backup
Use /DELETEBACKUP <id> to delete backup
```

**Backup information includes:**
- Unique ID (for restore operations)
- Human-readable name (if provided)
- Creation timestamp (with relative time)
- Archive size
- Backup type (Manual, Daily, Weekly, Monthly)
- Checksum verification status

## Verifying Backups

Verify backup integrity using SHA256 checksum:

```
/VERIFYBACKUP backup_20240315_143022_123
```

**Successful verification:**
```
✅ Backup verified successfully!
ID: backup_20240315_143022_123
Checksum: a3f8b9c1d2e4f5a6... matches stored value
Archive integrity: OK
```

**Failed verification:**
```
❌ Backup verification failed!
ID: backup_20240315_143022_123
Expected: a3f8b9c1d2e4f5a6...
Actual: b2c3d4e5f6a7b8c9...
Archive may be corrupted - do not use for restoration
```

**Best practices:**
- Verify backups periodically (weekly recommended)
- Always verify before restoration
- Test restoration procedures on non-production systems
- Verify after copying backups to external storage

## Deleting Backups

Remove specific backup by ID:

```
/DELETEBACKUP backup_20240315_060000_456
```

**Manual backup deletion (requires confirmation):**
```
⚠️ Delete Manual Backup
This is a manual backup and will not be recreated automatically.
Type: Manual
Created: 2024-03-15 14:30:22 UTC
Size: 2.5 MB

Are you sure? This action cannot be undone.
Confirm deletion: [YES/NO]
```

**Automatic backup deletion:**
```
🗑️ Backup deleted successfully
ID: backup_20240315_060000_456
Type: Daily (Automatic)
Freed space: 2.4 MB
```

**Protection mechanisms:**
- Cannot delete last remaining backup
- Manual backups require confirmation
- Deletion is immediate and permanent
- Frees disk space immediately

## Restoring Backups

Restore world database from backup (requires sysop privileges):

```
/RESTORE backup_20240315_143022_123
```

**⚠️ IMPORTANT RESTORATION NOTES:**

1. **Offline operation required** - You cannot restore while the server is running
2. **Verification is automatic** - System checks backup integrity before proceeding
3. **Current data is backed up** - Existing database is saved as emergency backup
4. **Server restart required** - After restore, restart the MeshBBS daemon

**Restoration output:**
```
🔧 Restore Backup Instructions

To restore from backup: backup_20240315_143022_123
Created: 2024-03-15 14:30:22 UTC
Size: 2.5 MB
Checksum: a3f8b9c1... (verified ✅)

⚠️ WARNING: This operation requires server shutdown and restart

RESTORATION PROCEDURE:
1. Shut down MeshBBS server
2. Backup current database (if not already done):
   cp -r data/tinymush data/tinymush.emergency-backup

3. Extract backup archive:
   cd data/backups
   tar -xzf backup_20240315_143022_123.tar.gz

4. Replace current database:
   rm -rf ../tinymush/*
   mv tinymush/* ../tinymush/

5. Restart MeshBBS server
6. Verify world state after restoration

EMERGENCY ROLLBACK:
If restoration fails, restore from emergency backup:
   rm -rf data/tinymush/*
   mv data/tinymush.emergency-backup/* data/tinymush/
```

**Post-restoration verification:**
1. Check server starts successfully
2. Verify player accounts exist
3. Test room navigation
4. Confirm object persistence
5. Review restoration logs

## Backup Scenarios

### Pre-Update Backup

Before applying major updates:

```
/BACKUP "pre-update-v1.1.0"
/VERIFYBACKUP <backup-id>
# Proceed with update
```

### Disaster Recovery

If world corruption occurs:

```
/LISTBACKUPS                    # Find recent good backup
/VERIFYBACKUP <backup-id>       # Verify integrity
/RESTORE <backup-id>            # Follow restoration instructions
```

### Off-site Backup Copy

Regularly copy backups to external storage:

```bash
# On server (as admin user)
cd /path/to/meshbbs/data/backups
rsync -av backup_*.tar.gz remote-server:/backup/meshbbs/
rsync -av backups.json remote-server:/backup/meshbbs/
```

### Testing Restoration

Test backup procedures on separate system:

```bash
# Copy backup files to test system
scp data/backups/backup_*.tar.gz test-server:/tmp/
scp data/backups/backups.json test-server:/tmp/

# On test system
cd /path/to/test-meshbbs
mkdir -p data/backups
cp /tmp/backup_*.tar.gz data/backups/
cp /tmp/backups.json data/backups/

# Test restoration procedure (follow /RESTORE instructions)
```

## Backup Best Practices

### Frequency

- **High-activity worlds**: Hourly backups (`/BACKUPCONFIG frequency hourly`)
- **Medium-activity worlds**: 6-hour backups (`/BACKUPCONFIG frequency 6h`)
- **Low-activity worlds**: Daily backups (`/BACKUPCONFIG frequency daily`)

### Manual Backups

Create manual backups before:
- Major software updates
- World restructuring
- Large data imports
- Permission changes
- Event setup/teardown

### Verification Schedule

- **Weekly**: Verify most recent backup
- **Monthly**: Verify all manual backups
- **Before restoration**: Always verify target backup

### Off-site Storage

- **Daily**: Copy latest backup to external storage
- **Weekly**: Verify off-site backup integrity
- **Monthly**: Test restoration from off-site backup

### Retention Tuning

Adjust retention if needed by modifying retention policy in configuration:

```toml
[backup]
retention_daily = 7      # Keep 7 daily backups (1 week)
retention_weekly = 4     # Keep 4 weekly backups (1 month)
retention_monthly = 12   # Keep 12 monthly backups (1 year)
```

## Troubleshooting

### "Permission denied" Error

**Problem**: User lacks admin privileges

**Solution**: Admin level 2+ required for backup operations, level 3 for restore
```
Contact a sysop to request admin privileges or to perform the operation
```

### "Backup verification failed" Error

**Problem**: Backup archive is corrupted

**Solution**: Do not use this backup for restoration
```
1. Create new backup immediately
2. Delete corrupted backup
3. Investigate disk space or filesystem issues
4. Restore from previous verified backup if needed
```

### "Cannot delete last backup" Error

**Problem**: Attempting to delete only remaining backup

**Solution**: Create new backup before deleting
```
1. /BACKUP "new-backup"
2. /VERIFYBACKUP <new-backup-id>
3. /DELETEBACKUP <old-backup-id>
```

### Restoration Fails to Start

**Problem**: Server still running during restore

**Solution**: Stop server first
```
sudo systemctl stop meshbbs
# Follow /RESTORE instructions
sudo systemctl start meshbbs
```

### Backup Directory Full

**Problem**: Insufficient disk space

**Solution**: Clean up old automatic backups
```
1. /LISTBACKUPS
2. Delete old automatic backups: /DELETEBACKUP <id>
3. Keep manual backups for important milestones
4. Consider moving old backups to off-site storage
```

## Security Considerations

### Access Control

- Only admin level 2+ can create/manage backups
- Only sysop (level 3) can restore backups
- All backup operations are logged
- Review logs periodically for unauthorized access

### Backup Contents

Backups contain **all world data**:
- Player accounts and passwords (hashed)
- Private messages and descriptions
- Room content and connections
- Object ownership and attributes
- Financial transactions

**Protect backup files accordingly:**
- Restrict filesystem permissions (chmod 600)
- Encrypt off-site backups
- Use secure transfer methods (scp, rsync with SSH)
- Store off-site backups in trusted locations only

### Audit Trail

All backup operations are logged:
- Backup creation (manual and automatic)
- Restoration attempts
- Backup deletion
- Verification checks
- Configuration changes

Review logs at: `meshbbs.log`

## Related Documentation

- [Daemon Mode](daemon-mode.md) - Server management
- [Setup Guide](setup.md) - Initial configuration
- [User Management](user-management.md) - Player administration
- [Moderation Guide](moderation.md) - Content management

## Support

For backup issues:
1. Check `meshbbs.log` for error messages
2. Verify disk space: `df -h data/backups`
3. Test backup verification: `/VERIFYBACKUP <id>`
4. Contact development team with logs if needed

---

**Version**: 1.0.23  
**Last Updated**: 2024-03-15  
**Applies to**: MeshBBS TinyMUSH Mode
