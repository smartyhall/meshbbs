# Meshbbs: Daemon Mode & Graceful Shutdown Implementation

## Overview

This document describes the cross-platform daemon mode and graceful shutdown features implemented in Meshbbs v1.0.65.

**Latest Updates (v1.0.65)**:
- Daemon mode now included in default features
- Custom fork-based implementation (no external dependencies)
- TTY-aware logging (eliminates duplicates in daemon mode)
- Dependency cleanup: removed 5 unused crates

## Features Implemented

### ✅ 1. Cross-Platform Graceful Shutdown

Meshbbs now properly handles shutdown signals on all platforms:

**Unix (Linux/macOS)**:
- `SIGTERM` - Standard termination signal (systemd, launchd)
- `SIGINT` - Interrupt signal (Ctrl+C)
- `SIGHUP` - Hangup signal (reload/graceful shutdown)

**Windows**:
- `Ctrl+C` - Interrupt
- `Ctrl+Break` - Break signal

All signals trigger the same graceful shutdown sequence:
1. Log shutdown initiation
2. Notify all active user sessions
3. Close sessions and flush user data
4. Send shutdown signals to reader/writer tasks
5. Disconnect from Meshtastic device
6. Exit cleanly with return code 0

### ✅ 2. Optional Daemon Mode (Unix Only)

Meshbbs can run as a background daemon on Linux and macOS:

**Features**:
- Custom fork-based implementation (no external dependencies)
- PID file management
- TTY-aware logging (eliminates duplicates)
- Working directory preservation
- Proper signal handling in daemon mode

**Command Line**:
```bash
# Run as daemon
meshbbs --config config.toml start --daemon

# Custom PID file location
meshbbs start --daemon --pid-file /var/run/meshbbs.pid
```

### ✅ 3. TTY-Aware Logging (v1.0.65)

Smart logging behavior based on terminal detection:

**Daemon Mode** (stdout redirected to file):
- TTY detection returns `false`
- Logs written to file only (single copy)
- No duplicate log entries
- Clean, efficient logging

**Foreground Mode** (stdout is terminal):
- TTY detection returns `true`  
- Logs written to both file (persistence) and console (real-time)
- Interactive development/debugging
- Same log content in both outputs

**Implementation**: Uses `atty` crate to detect if stdout is connected to a TTY.

### ✅ 4. Management Script

A cross-platform shell script (`scripts/meshbbs-daemon.sh`) provides easy daemon management:

```bash
# Start daemon
./scripts/meshbbs-daemon.sh start

# Stop daemon
./scripts/meshbbs-daemon.sh stop

# Restart daemon
./scripts/meshbbs-daemon.sh restart

# Check status
./scripts/meshbbs-daemon.sh status

# View logs
./scripts/meshbbs-daemon.sh logs [lines]
```

### ✅ 5. System Integration

#### Linux systemd
Full systemd service file template included in documentation with:
- Automatic restart on failure
- Security hardening directives
- Proper dependency management
- Journal logging integration

#### macOS launchd
Launchd plist templates for both:
- System-wide daemons (`/Library/LaunchDaemons`)
- User-level agents (`~/Library/LaunchAgents`)

#### Windows
Guidance for running as Windows Service using:
- NSSM (Non-Sucking Service Manager)
- Windows Service Control Manager
- Task Scheduler

## Technical Implementation

### Signal Handling Architecture

The signal handlers are implemented in `src/bbs/server.rs:run()` using Tokio's async signal API:

```rust
// Setup signal handlers
#[cfg(unix)]
let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;
#[cfg(unix)]
let mut sighup = tokio::signal::unix::signal(SignalKind::hangup())?;
#[cfg(windows)]
let mut ctrl_break = tokio::signal::windows::ctrl_break()?;

// Main event loop with signal handling
loop {
    tokio::select! {
        // ... message processing ...
        
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT, initiating graceful shutdown...");
            break;
        }
        
        _ = async { sigterm.recv().await } => {
            info!("Received SIGTERM, initiating graceful shutdown...");
            break;
        }
        
        // ... other signals ...
    }
}

// After loop exits, perform graceful shutdown
self.shutdown().await?;
```

### Graceful Shutdown Sequence

The `shutdown()` method (now public) performs:

```rust
pub async fn shutdown(&mut self) -> Result<()> {
    info!("Shutting down BBS server...");

    // 1. Close all active sessions
    for (session_id, session) in &mut self.sessions {
        info!("Closing session: {}", session_id);
        session.logout().await?;
    }
    self.sessions.clear();

    // 2. Send shutdown signals to async tasks
    #[cfg(feature = "meshtastic-proto")]
    {
        if let Some(ref tx) = self.reader_control_tx {
            let _ = tx.send(ControlMessage::Shutdown);
        }
        if let Some(ref tx) = self.writer_control_tx {
            let _ = tx.send(ControlMessage::Shutdown);
        }
    }

    // 3. Disconnect Meshtastic device
    if let Some(device) = &mut self.device {
        device.disconnect().await?;
    }

    info!("BBS server shutdown complete");
    Ok(())
}
```

### Daemon Process Implementation

The daemon functionality is implemented in `src/main.rs` using the `daemonize` crate:

```rust
#[cfg(all(unix, feature = "daemon"))]
fn daemonize_process(config: &Config, pid_file: &str) -> Result<()> {
    use daemonize::Daemonize;
    use std::fs::File;
    
    // Determine log file paths from config
    let stdout_path = config.logging.file.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("meshbbs.log");
    let stderr_path = stdout_path;
    
    // Create log files
    let stdout_file = File::create(stdout_path)?;
    let stderr_file = File::create(stderr_path)?;
    
    // Configure and start daemon
    let daemon = Daemonize::new()
        .pid_file(pid_file)
        .working_directory(std::env::current_dir()?)
        .stdout(stdout_file)
        .stderr(stderr_file)
        .umask(0o027); // Restrictive permissions
    
    daemon.start()?;
    Ok(())
}
```

## Cargo.toml Changes (v1.0.65)

**Daemon feature now included in default build**:

```toml
[features]
default = ["serial", "meshtastic-proto", "weather", "api-reexports", "daemon"]
daemon = [] # Custom implementation, no external dependencies
```

**Key Changes**:
- Removed `daemonize` crate dependency (custom implementation)
- Daemon feature included in default features for production deployments
- Added `atty` crate for TTY detection
- Removed 5 unused dependencies (getrandom, unsigned-varint, daemonize, axum, tower)

## Building

### Standard Build (All Platforms)
```bash
# Includes daemon support on Linux/macOS
cargo build --release
```

### Without Daemon Support (if needed)
```bash
# Explicitly exclude daemon feature
cargo build --release --no-default-features --features "serial,meshtastic-proto,weather,api-reexports"
```

## Testing

All existing tests continue to pass (184 tests). The graceful shutdown is tested implicitly through:
- Session cleanup tests
- Integration tests that create and destroy servers
- Signal handling can be tested manually

Manual testing:
```bash
# Start server in foreground
cargo run -- start

# In another terminal, send SIGTERM
kill -TERM $(pgrep meshbbs)

# Check logs for "Shutting down BBS server..." and "shutdown complete"
```

## Platform Compatibility

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Graceful Shutdown (SIGTERM) | ✅ | ✅ | ❌ |
| Graceful Shutdown (SIGHUP) | ✅ | ✅ | ❌ |
| Graceful Shutdown (Ctrl+C) | ✅ | ✅ | ✅ |
| Graceful Shutdown (Ctrl+Break) | ❌ | ❌ | ✅ |
| Daemon Mode (--daemon) | ✅ | ✅ | ❌* |
| PID File Management | ✅ | ✅ | ❌* |
| System Service (systemd) | ✅ | ❌ | ❌ |
| System Service (launchd) | ❌ | ✅ | ❌ |
| System Service (NSSM) | ❌ | ❌ | ✅ |

*Windows can run as a service using NSSM or Windows Service Control Manager, which is the Windows-native equivalent.

## Documentation

Complete documentation available in:
- `docs/administration/daemon-mode.md` - Comprehensive daemon mode guide
- `scripts/meshbbs-daemon.sh` - Self-documented management script
- This document - Technical implementation details

## Security Considerations

1. **PID File Permissions**: Default location `/tmp/meshbbs.pid` is world-writable
   - Recommendation: Use `/var/run/meshbbs.pid` in production
   - Set proper ownership: `chown meshbbs:meshbbs /var/run/meshbbs.pid`

2. **Log File Permissions**: Created with umask 0o027 (group-readable, other-denied)
   - Ensure log directory has proper permissions
   - Consider log rotation with proper signal handling

3. **Run as Non-Root**: Always run meshbbs as a dedicated user
   - Create `meshbbs` user: `sudo useradd -r -s /bin/false meshbbs`
   - Set file ownership: `sudo chown -R meshbbs:meshbbs /opt/meshbbs`

4. **Systemd Hardening**: Use security directives in service file:
   - `NoNewPrivileges=true`
   - `PrivateTmp=true`
   - `ProtectSystem=strict`
   - `ProtectHome=true`

## Compatibility Notes

### Breaking Changes
- **None**: This is a pure addition of functionality
- Existing deployments continue to work without changes
- Default behavior unchanged (foreground mode)

### Deprecated Features
- None

### Migration Path (v1.0.65)
No migration needed. Daemon mode now included in default builds:
1. Rebuild with standard `cargo build --release`
2. Update deployment scripts to use `--daemon` flag
3. Create systemd/launchd service files (optional)
4. Benefit from TTY-aware logging (no duplicates in daemon mode)

## Future Enhancements

Potential improvements for future versions:
1. **Config Reload**: Implement SIGHUP-triggered config reload without full restart
2. **Windows Service**: Native Windows service support without NSSM
3. **Health Checks**: HTTP endpoint for health/readiness probes
4. **Metrics**: Prometheus metrics endpoint
5. **Graceful Drain**: Connection draining period before shutdown
6. **PID Lock**: Advisory file locking to prevent multiple instances

## Version Information

- **Initial Implementation**: v1.0.60
- **Current Version**: v1.0.65
- **Status**: Production-ready
- **Key Updates**:
  - v1.0.65: Custom implementation, TTY-aware logging, included in default features
  - v1.0.60: Initial daemon mode and graceful shutdown
- **Tested On**: 
  - macOS 14+ (Apple Silicon & Intel)
  - Ubuntu 22.04 LTS
  - Debian 12
  - Windows 11 (graceful shutdown only)

## Credits

Implementation by Martin Bogomolni based on requirements for production-grade daemon operation with proper signal handling across all supported platforms.
