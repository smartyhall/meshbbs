# Device Connection Startup Handling

## Overview

This document describes how MeshBBS handles device connection failures at startup, providing graceful error handling with proper exit codes for service management systems (systemd, docker, etc.).

## Problem Statement

**Before**: If the configured Meshtastic device was unavailable at startup (e.g., USB serial port disconnected, wrong permissions, device not ready), MeshBBS would:
- Log a warning and continue starting
- Run without device connectivity
- Silently fail to respond to mesh messages
- Provide no indication to service managers that startup failed

This caused issues with:
- Systemd services reporting "active" when device was missing
- Docker containers appearing healthy but non-functional
- Difficult troubleshooting (logs showed "starting" but device was missing)

## Solution

Added `require_device_at_startup` configuration option that allows operators to specify whether device connectivity is essential for operation.

### Configuration Option

```toml
[meshtastic]
# Require device connection at startup (default: false)
# When true: BBS exits with error code 2 if device connection fails
# When false: BBS starts without device (useful for testing or alternative transports)
require_device_at_startup = false
```

### Exit Codes

MeshBBS now uses standardized exit codes:

| Exit Code | Meaning | Service Manager Behavior |
|-----------|---------|-------------------------|
| 0 | Clean shutdown | Service stopped normally |
| 1 | Configuration error | Service failed to start |
| 2 | Device connection failed | Service failed to start |
| Other | Runtime error | Service crashed |

## Behavior Modes

### Mode 1: Optional Device (Default)
**Config**: `require_device_at_startup = false`

**Behavior**:
- Device connection failure logs WARNING
- BBS continues starting
- Useful for:
  - Development/testing without hardware
  - Future multi-transport configurations (Bluetooth, TCP/UDP)
  - Graceful handling of temporary device issues

**Example Output**:
```
INFO  Starting Meshbbs v1.1.2
ERROR Failed to connect to device on /dev/ttyUSB0: Permission denied (os error 13)
WARN  BBS continuing without device connection
INFO  BBS server starting...
```

### Mode 2: Required Device
**Config**: `require_device_at_startup = true`

**Behavior**:
- Device connection failure logs ERROR
- BBS exits immediately with code 2
- Useful for:
  - Production deployments
  - Service monitoring
  - Ensuring connectivity before accepting requests

**Example Output**:
```
INFO  Starting Meshbbs v1.1.2
ERROR Failed to connect to device on /dev/ttyUSB0: Permission denied (os error 13)
ERROR Device connection required but failed - exiting
ERROR To allow BBS to start without a device, set:
ERROR   [meshtastic]
ERROR   require_device_at_startup = false
Exit code: 2
```

## Use Cases

### Production Deployment (Required Device)

**config.toml**:
```toml
[meshtastic]
port = "/dev/ttyUSB0"
require_device_at_startup = true
```

**systemd service** (meshbbs.service):
```ini
[Unit]
Description=MeshBBS - Mesh Network Bulletin Board System
After=network.target

[Service]
Type=simple
User=meshbbs
ExecStart=/usr/local/bin/meshbbs start
Restart=on-failure
RestartSec=10s

# Don't restart on config errors or device connection failures
RestartPreventExitStatus=1 2

[Install]
WantedBy=multi-tier.target
```

**Behavior**:
- If device missing at boot, service fails with clear error
- systemd won't restart (exit code 2 in RestartPreventExitStatus)
- Admin sees "failed" status immediately
- Logs show exact problem (device connection failed)

### Development Mode (Optional Device)

**config.toml**:
```toml
[meshtastic]
port = "/dev/ttyUSB0"
require_device_at_startup = false
```

**Behavior**:
- BBS starts even without device
- Can develop/test other features
- Can manually reconnect device later (future feature)
- Useful for offline testing

### Docker Deployment

**docker-compose.yml**:
```yaml
services:
  meshbbs:
    image: meshbbs:latest
    devices:
      - /dev/ttyUSB0:/dev/ttyUSB0
    volumes:
      - ./config.toml:/app/config.toml
      - ./data:/app/data
    restart: on-failure
    # Exit code 2 = device not available, stop retrying
    deploy:
      restart_policy:
        condition: on-failure
        max_attempts: 3
```

**config.toml**:
```toml
[meshtastic]
port = "/dev/ttyUSB0"
require_device_at_startup = true
```

**Behavior**:
- Container fails fast if device not available
- Docker Compose/Swarm sees failure and stops
- Clear logs show device connection issue
- No "zombie" containers running without connectivity

## Error Messages

### Case 1: Device Connection Failed (with require_device_at_startup = true)
```
ERROR Failed to connect to device on /dev/ttyUSB0: Permission denied (os error 13)
ERROR Device connection required but failed - exiting
ERROR To allow BBS to start without a device, set:
ERROR   [meshtastic]
ERROR   require_device_at_startup = false
```

**Exit Code**: 2

**Common Causes**:
- USB device not connected
- Wrong device path in config
- Insufficient permissions (need to add user to dialout/uucp group)
- Device already in use by another process
- Device not ready yet (boot timing)

### Case 2: No Port Specified (with require_device_at_startup = true)
```
ERROR Device connection required but no port specified
ERROR Either specify --port or set 'port' in config.toml
ERROR Or set require_device_at_startup = false to start without device
```

**Exit Code**: 2

**Solution**: Add port to config or use `--port` flag

### Case 3: Device Connection Failed (with require_device_at_startup = false)
```
ERROR Failed to connect to device on /dev/ttyUSB0: Permission denied (os error 13)
WARN  BBS continuing without device connection
INFO  BBS server starting...
```

**Exit Code**: 0 (continues)

**Behavior**: BBS starts but won't be able to send/receive mesh messages

## Troubleshooting

### Issue: "Permission denied" Error

**Symptom**:
```
ERROR Failed to connect to device on /dev/ttyUSB0: Permission denied (os error 13)
```

**Solution** (Linux):
```bash
# Add user to dialout group (Ubuntu/Debian)
sudo usermod -a -G dialout $USER

# Or add to uucp group (Arch/Fedora)
sudo usermod -a -G uucp $USER

# Log out and back in for changes to take effect
```

**Solution** (check current permissions):
```bash
ls -l /dev/ttyUSB0
# Should show: crw-rw---- 1 root dialout

# If not in dialout group:
groups $USER
# Should show: ... dialout ...
```

### Issue: "No such file or directory"

**Symptom**:
```
ERROR Failed to connect to device on /dev/ttyUSB0: No such file or directory (os error 2)
```

**Solution**:
```bash
# List available serial devices
ls /dev/ttyUSB* /dev/ttyACM*

# Check dmesg for device detection
dmesg | grep -i tty

# Verify device is connected
lsusb
```

**Common Issues**:
- Wrong device path (try ttyACM0 instead of ttyUSB0)
- USB cable loose or damaged
- Device not powered
- USB hub issue (try direct connection)

### Issue: "Device or resource busy"

**Symptom**:
```
ERROR Failed to connect to device on /dev/ttyUSB0: Device or resource busy (os error 16)
```

**Solution**:
```bash
# Check what's using the device
lsof /dev/ttyUSB0

# Common culprits:
# - Another MeshBBS instance
# - Meshtastic Python CLI
# - Screen/minicom session
# - ModemManager (interferes with serial)

# Disable ModemManager (if not needed)
sudo systemctl stop ModemManager
sudo systemctl disable ModemManager
```

### Issue: Service Fails to Start (systemd)

**Check Status**:
```bash
systemctl status meshbbs

# Should show:
# Active: failed (Result: exit-code)
# Process: ... ExitCode=2
```

**Check Logs**:
```bash
journalctl -u meshbbs -n 50

# Look for ERROR messages about device connection
```

**Verify Configuration**:
```bash
# Check config file
cat /etc/meshbbs/config.toml | grep -A5 "\[meshtastic\]"

# Test device access as service user
sudo -u meshbbs ls -l /dev/ttyUSB0
```

## Future Transport Support

This mechanism is designed to be transport-agnostic and will apply to future connection methods:

### Bluetooth (Planned)
```toml
[meshtastic]
transport = "bluetooth"
bluetooth_address = "AA:BB:CC:DD:EE:FF"
require_device_at_startup = true
```

**Error Handling**:
- Bluetooth adapter not available → exit code 2
- Device not paired → exit code 2
- Device out of range → exit code 2

### TCP/IP (Planned)
```toml
[meshtastic]
transport = "tcp"
tcp_host = "192.168.1.100"
tcp_port = 4403
require_device_at_startup = true
```

**Error Handling**:
- Connection refused → exit code 2
- Host unreachable → exit code 2
- Timeout → exit code 2

### UDP (Planned)
```toml
[meshtastic]
transport = "udp"
udp_host = "192.168.1.100"
udp_port = 4403
require_device_at_startup = true
```

**Error Handling**:
- Socket bind failure → exit code 2
- Network unreachable → exit code 2

## Best Practices

### Production Deployments
1. ✅ **Set** `require_device_at_startup = true`
2. ✅ **Configure** systemd RestartPreventExitStatus
3. ✅ **Monitor** service status with health checks
4. ✅ **Test** device permissions before deployment
5. ✅ **Document** device path in deployment notes

### Development Environments
1. ✅ **Set** `require_device_at_startup = false` for flexibility
2. ✅ **Use** mock testing when device unavailable
3. ✅ **Test** with real device before production
4. ✅ **Document** when features require device

### Docker/Container Deployments
1. ✅ **Set** `require_device_at_startup = true`
2. ✅ **Map** device with `--device` flag or compose devices
3. ✅ **Limit** restart attempts to avoid restart loops
4. ✅ **Use** healthcheck endpoints (when available)
5. ✅ **Document** device requirements in README

## Integration with Monitoring

### Prometheus/Metrics
```yaml
# Future: metrics endpoint will expose device status
meshbbs_device_connected{transport="serial",port="/dev/ttyUSB0"} 0
meshbbs_device_connection_errors_total 3
meshbbs_startup_exit_code 2
```

### Health Check Endpoint (Future)
```bash
curl http://localhost:8080/health
{
  "status": "degraded",
  "device": {
    "connected": false,
    "transport": "serial",
    "port": "/dev/ttyUSB0",
    "error": "Permission denied"
  }
}
```

## Migration Guide

### Existing Installations

**No Action Required**: Default behavior unchanged (`require_device_at_startup = false`)

**To Opt-In to Strict Checking**:

1. Edit `/etc/meshbbs/config.toml`:
```toml
[meshtastic]
require_device_at_startup = true
```

2. Update systemd service:
```bash
# Edit service file
sudo systemctl edit meshbbs

# Add:
[Service]
RestartPreventExitStatus=1 2

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart meshbbs
```

3. Test:
```bash
# Stop service and disconnect device
sudo systemctl stop meshbbs
# Disconnect USB device

# Try to start
sudo systemctl start meshbbs

# Should fail immediately with clear error
systemctl status meshbbs
# Active: failed (Result: exit-code)
```

## Summary

**Before**:
- Device failures were silent
- BBS appeared to start successfully
- Service managers couldn't detect issues
- Troubleshooting required log analysis

**After**:
- Device failures are explicit and loud
- Clear error messages guide troubleshooting
- Service managers detect failures immediately
- Production-friendly with proper exit codes
- Development-friendly with optional mode
- Future-proof for multiple transport types

**Configuration**:
```toml
[meshtastic]
require_device_at_startup = true  # Production: strict checking
# or
require_device_at_startup = false # Development: flexible
```

**Exit Codes**:
- `0` = Success (continues without device if allowed)
- `1` = Configuration error
- `2` = Device connection failed (when required)
