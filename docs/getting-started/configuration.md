# Configuration Guide

This guide covers the `config.toml` options for MeshBBS.

## Ident Beacon

Periodic station identification broadcast on the public channel.

```toml
[ident_beacon]
# Enable or disable the periodic ident beacon
enabled = true
# Frequency of ident beacon broadcasts
# Options: "5min", "15min", "30min", "1hour", "2hours", "4hours"
# Default: "15min"
frequency = "15min"
```

Behavior:
- Message format: `[IDENT] <BBS name> (<short node id>) - <YYYY-MM-DD HH:MM:SS> UTC - Type <prefix>HELP for commands` (default prefix `^`)
- Short node id is 24-bit hex (e.g., 0x1A2B3C); falls back to config value or "Unknown".
- Scheduling is in UTC and respects the selected boundary (e.g., every 5 minutes at :00/:05/:10/...).
- Startup gating:
  - If the server has a Meshtastic device instance, ident waits for initial sync to complete.
  - If using the reader/writer pattern without a device instance, ident begins after a short startup grace period, then follows UTC boundaries.
- Duplicate prevention avoids multiple idents within the same minute.

## Logging

```toml
[logging]
level = "info"              # info | debug | warn | error
file = "meshbbs.log"        # optional log file
security_file = "meshbbs-security.log"   # optional security/audit log file
```

## Meshtastic

```toml
[meshtastic]
port = "/dev/tty.usbserial-XXXXX"
baud_rate = 115200
node_id = ""                 # optional; can be decimal or 0xHEX
channel = 0
# Optional tuning
min_send_gap_ms = 2000
post_dm_broadcast_gap_ms = 1200
dm_to_dm_gap_ms = 600
```

For all available fields, check the generated API docs or `src/config/mod.rs` for defaults and serde names.
