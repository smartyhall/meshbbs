# Meshbbs Daemon Mode

## Overview

Meshbbs can run as a background daemon/service on Linux, macOS, and Windows, allowing it to run continuously without an active terminal session.

## Features

- **Graceful Shutdown**: Responds to SIGTERM, SIGHUP, and SIGINT signals
- **Clean State Management**: Properly closes sessions and saves data on shutdown
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **PID File Management**: Tracks running process for easy management
- **Log Rotation Compatible**: Supports external log rotation tools

## Quick Start

### Build with Daemon Support

```bash
# Linux/macOS
cargo build --release --features daemon

# Windows (daemon mode not required, runs as foreground or service)
cargo build --release
```

### Start as Daemon (Linux/macOS)

```bash
# Direct invocation
./target/release/meshbbs --config config.toml start --daemon

# With custom PID file location
./target/release/meshbbs start --daemon --pid-file /var/run/meshbbs.pid

# Using the control script (recommended)
./scripts/meshbbs-daemon.sh start
```

### Stop the Daemon

```bash
# Using the control script
./scripts/meshbbs-daemon.sh stop

# Or manually
kill -TERM $(cat /tmp/meshbbs.pid)
```

## Platform-Specific Instructions

### Linux

#### Systemd Service (Recommended)

Create `/etc/systemd/system/meshbbs.service`:

```ini
[Unit]
Description=Meshbbs - Bulletin Board System for Meshtastic
After=network.target
Documentation=https://github.com/martinbogo/meshbbs

[Service]
Type=forking
User=meshbbs
Group=meshbbs
WorkingDirectory=/opt/meshbbs
ExecStart=/opt/meshbbs/meshbbs --config /opt/meshbbs/config.toml start --daemon --pid-file /var/run/meshbbs.pid
PIDFile=/var/run/meshbbs.pid
Restart=on-failure
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/meshbbs/data /var/log/meshbbs

[Install]
WantedBy=multi-user.target
```

Then manage with systemd:

```bash
# Enable and start
sudo systemctl enable meshbbs
sudo systemctl start meshbbs

# Check status
sudo systemctl status meshbbs

# View logs
sudo journalctl -u meshbbs -f

# Stop
sudo systemctl stop meshbbs

# Restart
sudo systemctl restart meshbbs
```

#### Using the Control Script

```bash
# Start
./scripts/meshbbs-daemon.sh start

# Stop
./scripts/meshbbs-daemon.sh stop

# Restart
./scripts/meshbbs-daemon.sh restart

# Status
./scripts/meshbbs-daemon.sh status

# View logs
./scripts/meshbbs-daemon.sh logs
```

### macOS

#### Using the Control Script (Recommended)

```bash
# Start daemon
./scripts/meshbbs-daemon.sh start

# Stop daemon
./scripts/meshbbs-daemon.sh stop

# Check status
./scripts/meshbbs-daemon.sh status
```

#### Launchd Service (System-Wide)

Create `/Library/LaunchDaemons/com.meshbbs.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.meshbbs.server</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/meshbbs</string>
        <string>--config</string>
        <string>/usr/local/etc/meshbbs/config.toml</string>
        <string>start</string>
    </array>
    
    <key>WorkingDirectory</key>
    <string>/usr/local/var/meshbbs</string>
    
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/meshbbs/meshbbs.log</string>
    
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/meshbbs/meshbbs-error.log</string>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Manage with launchctl:

```bash
# Load and start
sudo launchctl load /Library/LaunchDaemons/com.meshbbs.server.plist

# Stop
sudo launchctl stop com.meshbbs.server

# Unload
sudo launchctl unload /Library/LaunchDaemons/com.meshbbs.server.plist

# View status
sudo launchctl list | grep meshbbs
```

#### User-Level Launchd (Per-User)

Create `~/Library/LaunchAgents/com.meshbbs.server.plist` with similar content but user-specific paths:

```bash
# Load
launchctl load ~/Library/LaunchAgents/com.meshbbs.server.plist

# Unload
launchctl unload ~/Library/LaunchAgents/com.meshbbs.server.plist
```

### Windows

#### Running as Console Application

```powershell
# Start in background
Start-Process -FilePath ".\target\release\meshbbs.exe" -ArgumentList "--config config.toml start" -WindowStyle Hidden

# Or use a task scheduler
```

#### Windows Service (Advanced)

For production Windows deployments, consider using tools like:
- **NSSM (Non-Sucking Service Manager)**: Easy service wrapper
- **Windows Service Control Manager**: Native Windows service
- **Task Scheduler**: Built-in Windows task automation

Example with NSSM:

```powershell
# Install NSSM
choco install nssm

# Install service
nssm install meshbbs "C:\path\to\meshbbs.exe" "--config C:\path\to\config.toml start"

# Configure service
nssm set meshbbs AppDirectory "C:\path\to\meshbbs"
nssm set meshbbs DisplayName "Meshbbs BBS Server"
nssm set meshbbs Description "Bulletin Board System for Meshtastic"

# Start service
nssm start meshbbs

# Stop service
nssm stop meshbbs
```

## Signal Handling

Meshbbs responds to the following signals for graceful shutdown:

### Unix (Linux/macOS)
- **SIGTERM**: Graceful shutdown (recommended)
- **SIGINT**: Graceful shutdown (Ctrl+C)
- **SIGHUP**: Graceful shutdown and reload

### Windows
- **Ctrl+C**: Graceful shutdown
- **Ctrl+Break**: Graceful shutdown

All signals trigger the same graceful shutdown sequence:
1. Stop accepting new connections
2. Notify all active sessions
3. Close sessions and flush user state
4. Send shutdown signals to reader/writer tasks
5. Disconnect from Meshtastic device
6. Flush all pending writes
7. Exit cleanly

## PID File Management

The daemon creates a PID file to track the running process:

- **Default location**: `/tmp/meshbbs.pid`
- **Custom location**: Use `--pid-file /path/to/file.pid`
- **Security**: Ensure PID file directory has proper permissions

The PID file contains only the process ID and is automatically cleaned up on graceful shutdown.

## Log Management

### Log Files

Configure logging in `config.toml`:

```toml
[logging]
file = "meshbbs.log"
security_file = "meshbbs-security.log"
```

### Log Rotation

Meshbbs supports external log rotation. Example with `logrotate`:

```
/var/log/meshbbs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 meshbbs meshbbs
    postrotate
        # Send SIGHUP to reload (or just let it continue writing)
        kill -HUP $(cat /var/run/meshbbs.pid) 2>/dev/null || true
    endscript
}
```

## Troubleshooting

### Daemon Won't Start

1. **Check if already running**:
   ```bash
   ./scripts/meshbbs-daemon.sh status
   ```

2. **Check logs**:
   ```bash
   tail -f meshbbs.log
   ```

3. **Check permissions**:
   ```bash
   # Ensure binary is executable
   chmod +x target/release/meshbbs
   
   # Ensure data directory is writable
   chmod 755 data
   ```

4. **Verify configuration**:
   ```bash
   # Test configuration without daemon mode
   ./target/release/meshbbs --config config.toml start
   ```

### Daemon Won't Stop

1. **Check if running**:
   ```bash
   ps aux | grep meshbbs
   ```

2. **Force stop**:
   ```bash
   # Send SIGKILL (last resort)
   kill -KILL $(cat /tmp/meshbbs.pid)
   
   # Clean up PID file
   rm /tmp/meshbbs.pid
   ```

### PID File Stale

If the PID file exists but process is not running:

```bash
# Remove stale PID file
rm /tmp/meshbbs.pid

# Restart
./scripts/meshbbs-daemon.sh start
```

## Environment Variables

The control script supports these environment variables:

- `MESHBBS_BIN`: Path to meshbbs binary (default: `./target/release/meshbbs`)
- `MESHBBS_CONFIG`: Path to config file (default: `config.toml`)
- `MESHBBS_PID_FILE`: Path to PID file (default: `/tmp/meshbbs.pid`)
- `MESHBBS_PORT`: Meshtastic device port (optional)

Example:

```bash
export MESHBBS_BIN=/usr/local/bin/meshbbs
export MESHBBS_CONFIG=/etc/meshbbs/config.toml
export MESHBBS_PID_FILE=/var/run/meshbbs.pid
export MESHBBS_PORT=/dev/ttyUSB0

./scripts/meshbbs-daemon.sh start
```

## Security Considerations

1. **Run as non-root user**: Create a dedicated `meshbbs` user
2. **Restrict file permissions**: Use `0640` for config files, `0755` for data directories
3. **Use systemd hardening**: Enable security features in systemd service files
4. **Monitor logs**: Set up log monitoring for security events
5. **Update regularly**: Keep meshbbs and dependencies up to date

## Best Practices

1. **Use systemd on Linux**: More robust than manual daemonization
2. **Use launchd on macOS**: Better integration with system services
3. **Set up monitoring**: Use tools like Prometheus, Nagios, or Zabbix
4. **Enable automatic restart**: Configure systemd/launchd to restart on failure
5. **Backup configuration**: Regularly backup `config.toml` and data directory
6. **Test graceful shutdown**: Verify data integrity after SIGTERM
7. **Monitor resource usage**: Check CPU, memory, and disk usage regularly

## Development and Testing

### Testing Graceful Shutdown

```bash
# Start in foreground
cargo run -- --config config.toml start

# In another terminal, send signals
kill -TERM $(pgrep meshbbs)  # Should see graceful shutdown in logs
```

### Building with Daemon Support

```bash
# Debug build
cargo build --features daemon

# Release build
cargo build --release --features daemon

# Test daemon mode
cargo run --release --features daemon -- start --daemon
```

### Debugging

```bash
# Run in foreground with verbose logging
cargo run -- -vv --config config.toml start

# Check signal handling
cargo test --features daemon
```

## Migration from Foreground to Daemon

If you're currently running meshbbs in a terminal:

1. **Stop the foreground process**: Press Ctrl+C
2. **Verify clean shutdown**: Check logs for "shutdown complete"
3. **Start as daemon**: `./scripts/meshbbs-daemon.sh start`
4. **Verify daemon started**: `./scripts/meshbbs-daemon.sh status`
5. **Monitor logs**: `./scripts/meshbbs-daemon.sh logs`

No data migration is needed - the daemon uses the same data directory and configuration.
