# Alpha Testing Quick Setup Guide

**Date**: October 9, 2025  
**Purpose**: Clean database setup for TinyMUSH alpha testing without running `meshbbs init`

## TL;DR - Quick Clean Start

```bash
# 1. Remove old databases
rm -rf data/tinymush

# 2. Start the server (it will auto-create and seed the world)
./target/release/meshbbs start

# That's it! The database will be automatically initialized on first use.
```

## What Happens Automatically

When you start meshbbs, the TinyMUSH system automatically:

### 1. Database Creation ✅
- Creates `data/tinymush/` directory if it doesn't exist
- Opens Sled database in that directory
- Creates all required internal trees (tables):
  - Player records
  - Room data
  - Objects
  - Mail system
  - Bulletin boards
  - Housing system
  - NPCs, quests, achievements
  - Companions
  - Secondary indexes (for performance)

### 2. World Seeding ✅
Automatically seeds the "Old Towne Mesh" world with:
- **Town Square** (starting location)
- **Market District**
- **Residential Quarter** (housing office)
- **Tavern** (social area)
- **Library**
- **Crafting Workshop**
- **Guild Hall**
- All room connections (exits)
- NPCs in appropriate locations
- Shop inventories

### 3. Game Content Seeding ✅
- **Quests**: Starter quests and progression chains
- **Achievements**: Various achievement categories
- **Companions**: Tameable creatures in the wild
- **Housing Templates**: Available housing options

### 4. Configuration ✅
- Loads default world configuration
- Sets up error messages and prompts
- Initializes currency system
- Sets room capacity limits

## What You Need to Do Manually

### 1. Set Sysop Password (Recommended)
```bash
./target/release/meshbbs sysop-passwd
```

This is **not required** for TinyMUSH to work, but recommended for:
- Admin commands in the main BBS
- Moderation features
- User management

### 2. Configure config.toml (Optional)
The default `config.toml` works fine, but you may want to customize:

```toml
[games]
tinymush_enabled = true
tinymush_db_path = "data/tinymush"  # Where TinyMUSH stores its data
```

**Default location**: `data/tinymush`  
**No need to change**: Unless you want a different path

## Clean Start Procedures

### Option 1: Fresh Database (Recommended for Alpha)
```bash
# Remove existing TinyMUSH database
rm -rf data/tinymush

# Start server (will auto-create everything)
./target/release/meshbbs start

# First user to connect will be created as a new player
# All default content will be seeded automatically
```

### Option 2: Keep Main BBS Data, Reset TinyMUSH Only
```bash
# Only remove TinyMUSH database
rm -rf data/tinymush

# Keep main BBS data intact
# (users, messages, topics remain)
ls data/
# You should see: messages/ users/ topics.json (preserved)

# Start server
./target/release/meshbbs start
```

### Option 3: Complete Clean Start (Main BBS + TinyMUSH)
```bash
# Remove everything
rm -rf data/

# Recreate data directory
mkdir -p data

# Start server
./target/release/meshbbs start

# You'll need to recreate:
# - Sysop password
# - Any custom topics
# - User accounts
```

## Verification Steps

After starting the server, verify everything initialized correctly:

### 1. Check Directory Structure
```bash
ls -la data/tinymush/
```

You should see Sled database files (lots of `.sled` files and directories).

### 2. Connect via Meshtastic
1. Send a message to the BBS from your Meshtastic device
2. When you get the main menu, select the Games option
3. Choose TinyMUSH
4. You should see the welcome message and start in Town Square

### 3. Test Basic Commands
In TinyMUSH, try:
```
LOOK
WHERE
MAP
HELP
```

All should work immediately without any manual database setup.

## What `meshbbs init` Does (For Reference)

The `meshbbs init` command is for the **main BBS system**, not TinyMUSH. It:
- Creates `config.toml` with defaults
- Seeds `data/topics.json` with default forum topics
- Sets up main BBS directory structure

**TinyMUSH does NOT need this** - it handles its own initialization automatically.

## Common Alpha Testing Scenarios

### Scenario 1: Testing Performance Improvements
```bash
# Start fresh
rm -rf data/tinymush

# Run server
./target/release/meshbbs start

# Connect multiple users
# Test concurrent access
# Monitor performance
```

### Scenario 2: Testing Data Persistence
```bash
# Connect, create some data (move around, send mail, etc.)
# Stop server (Ctrl+C)

# Restart server
./target/release/meshbbs start

# Verify player is still in last location
# Verify mail is preserved
# Verify housing persists
```

### Scenario 3: Testing Migration/Upgrades
```bash
# Start with fresh database
rm -rf data/tinymush
./target/release/meshbbs start

# Let users create data
# Update code (new features)
# Restart server

# Verify backward compatibility
# Check automatic index rebuilds
```

## Troubleshooting

### Problem: Database Won't Open
```bash
# Check permissions
ls -la data/
chmod 755 data

# Verify directory exists
mkdir -p data/tinymush

# Try starting again
./target/release/meshbbs start
```

### Problem: World Not Seeding
Check logs for errors:
```bash
tail -f meshbbs.log | grep -i tinymush
```

If you see errors about seeding, try:
```bash
# Complete clean start
rm -rf data/tinymush
./target/release/meshbbs start
```

### Problem: Performance Issues
```bash
# Check database size
du -sh data/tinymush/

# Check for too many connections
# (shouldn't be an issue with async integration)

# Monitor in logs
tail -f meshbbs.log | grep -i "spawn_blocking\|tinymush"
```

## Performance Monitoring

During alpha testing, watch for:

### Metrics to Collect
- Command response latency
- Concurrent user count
- Database operation times
- Memory usage

### Log Analysis
```bash
# Watch for blocking operations
grep "Task join error" meshbbs.log

# Monitor command processing
grep "TinyMUSH command parsed" meshbbs.log

# Check for errors
grep -i error meshbbs.log | grep -i tinymush
```

## Summary

**You don't need to run `meshbbs init` for TinyMUSH alpha testing.**

Just:
1. ✅ Remove `data/tinymush/` if you want a clean start
2. ✅ Start the server with `./target/release/meshbbs start`
3. ✅ Everything else happens automatically

The async integration means the database initialization is fast and non-blocking, so even the first startup should be quick.

---

**Ready for alpha testing!** 🎉
