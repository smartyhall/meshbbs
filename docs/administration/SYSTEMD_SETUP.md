# Systemd Service Setup

This guide shows how to run MeshBBS as a systemd service on Linux systems (including Raspberry Pi).

---

## 1. Create Service File

Create `/etc/systemd/system/meshbbs.service`:

```ini
[Unit]
Description=MeshBBS - Bulletin Board System for Meshtastic
After=network.target
Wants=network.target

[Service]
Type=simple
User=bbs
Group=bbs
WorkingDirectory=/opt/meshbbs
ExecStart=/opt/meshbbs/meshbbs
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/meshbbs/data /opt/meshbbs/meshbbs.log

# Environment
Environment="RUST_LOG=info"

# Resource limits (adjust for your Pi model)
MemoryLimit=512M
TasksMax=100

[Install]
WantedBy=multi-user.target
```

---

## 2. Installation Steps

```bash
# Create dedicated user
sudo useradd -r -s /bin/false -d /opt/meshbbs bbs

# Create installation directory
sudo mkdir -p /opt/meshbbs/data
sudo cp target/release/meshbbs /opt/meshbbs/
sudo cp config.toml /opt/meshbbs/
sudo cp topics.example.json /opt/meshbbs/data/topics.json

# Add bbs user to dialout group for serial access
sudo usermod -a -G dialout bbs

# Set permissions
sudo chown -R bbs:bbs /opt/meshbbs
sudo chmod 755 /opt/meshbbs
sudo chmod 644 /opt/meshbbs/config.toml

# Install service file
sudo cp docs/administration/meshbbs.service /etc/systemd/system/
sudo systemctl daemon-reload
```

---

## 3. Enable and Start Service

```bash
# Enable service to start on boot
sudo systemctl enable meshbbs

# Start the service
sudo systemctl start meshbbs

# Check status
sudo systemctl status meshbbs
```

---

## 4. Service Management

### View Logs

```bash
# View live logs
sudo journalctl -u meshbbs -f

# View last 100 lines
sudo journalctl -u meshbbs -n 100

# View logs since today
sudo journalctl -u meshbbs --since today
```

### Control Service

```bash
# Start
sudo systemctl start meshbbs

# Stop
sudo systemctl stop meshbbs

# Restart
sudo systemctl restart meshbbs

# Status
sudo systemctl status meshbbs

# Disable (prevent auto-start)
sudo systemctl disable meshbbs
```

### Update MeshBBS

```bash
# Stop service
sudo systemctl stop meshbbs

# Update binary
cd ~/meshbbs
git pull
cargo build --release
sudo cp target/release/meshbbs /opt/meshbbs/

# Start service
sudo systemctl start meshbbs

# Check it's running
sudo systemctl status meshbbs
```

---

## 5. Troubleshooting

### Service won't start

```bash
# Check service status
sudo systemctl status meshbbs

# View detailed logs
sudo journalctl -u meshbbs -n 50 --no-pager

# Check permissions
ls -la /opt/meshbbs
groups bbs

# Test manual start
sudo -u bbs /opt/meshbbs/meshbbs
```

### Serial port access denied

```bash
# Verify bbs user is in dialout group
groups bbs

# Add if missing
sudo usermod -a -G dialout bbs

# Restart service
sudo systemctl restart meshbbs
```

### Out of memory

Adjust `MemoryLimit` in the service file:

```ini
# For Pi 3B+ (2GB RAM)
MemoryLimit=256M

# For Pi 4/5 (4GB+ RAM)
MemoryLimit=512M
```

Then reload and restart:
```bash
sudo systemctl daemon-reload
sudo systemctl restart meshbbs
```

---

## 6. Security Considerations

The service file includes security hardening:

- **NoNewPrivileges**: Prevents privilege escalation
- **PrivateTmp**: Uses private /tmp directory
- **ProtectSystem=strict**: Makes most of the filesystem read-only
- **ProtectHome**: Makes /home read-only
- **ReadWritePaths**: Only allows writing to specific directories

### Adjust Permissions

If you need different paths:

```ini
ReadWritePaths=/opt/meshbbs/data /opt/meshbbs/logs /var/log/meshbbs
```

---

## 7. Monitoring

### Check Resource Usage

```bash
# CPU and memory usage
systemctl status meshbbs

# Detailed resource stats
systemd-cgtop | grep meshbbs
```

### Set Up Alerts

Create `/etc/systemd/system/meshbbs-failure@.service`:

```ini
[Unit]
Description=MeshBBS failure notification

[Service]
Type=oneshot
ExecStart=/usr/local/bin/notify-meshbbs-failure.sh
```

Then configure systemd to use it:
```ini
# Add to meshbbs.service [Unit] section
OnFailure=meshbbs-failure@%n.service
```

---

## Complete Service File Template

Save as `/etc/systemd/system/meshbbs.service`:

```ini
[Unit]
Description=MeshBBS - Bulletin Board System for Meshtastic
Documentation=https://martinbogo.github.io/meshbbs/
After=network.target
Wants=network.target

[Service]
Type=simple
User=bbs
Group=bbs
WorkingDirectory=/opt/meshbbs
ExecStart=/opt/meshbbs/meshbbs
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
TimeoutStopSec=30

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=meshbbs

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/meshbbs/data /opt/meshbbs/meshbbs.log
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true

# Environment
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"

# Resource limits (adjust for your hardware)
MemoryLimit=512M
TasksMax=100
CPUQuota=80%

[Install]
WantedBy=multi-user.target
```

---

## Next Steps

- Set up [Backup Automation](BACKUP_RECOVERY.md)
- Configure [Log Rotation](LOG_MANAGEMENT.md)
- Review [Security Best Practices](SECURITY.md)
- Set up [Monitoring](MONITORING.md)
