# Configuration Security Summary

## Security Improvements (October 12, 2025)

### Problem
Previous configuration files contained potential security issues:
- Default passwords or hashes in examples
- Public login enabled by default
- Real API keys in examples
- No password validation during installation

### Solution
Comprehensive security hardening of configuration and installation:

---

## config.example.toml Changes

### Secure Defaults
```toml
# Public login disabled (prevents username enumeration)
allow_public_login = false

# Placeholder for password hash (no defaults)
[admin]
password_hash = "REPLACE_WITH_YOUR_PASSWORD_HASH"

# Weather API key placeholder (prevents key exposure)
[weather]
api_key = "REPLACE_WITH_YOUR_API_KEY"
default_location = "90210"  # Obviously fake

# Welcome system disabled by default (opt-in feature)
[welcome]
enabled = false
```

### Added Sections
- `[meshtastic.ident]` - Beacon configuration (1 hour default)
- `[storage.backup]` - Automatic backup settings
- `[games]` - TinyHack and TinyMUSH enabled
- `[games.tinymush]` - TinyMUSH data directory
- `[welcome]` - Welcome message system (disabled)

---

## install.sh Security Features

### Interactive Password Setup
```bash
# Prompts for password during installation
# Minimum 8 characters enforced
# Confirmation required (prevents typos)
# Automatic hashing using meshbbs binary
```

### Serial Port Selection
```bash
# Interactive menu for common ports:
# 1) /dev/ttyUSB0 (default)
# 2) /dev/ttyACM0
# 3) /dev/ttyUSB1
# 4) Custom
```

### Configuration Generation
```bash
# Creates config.toml with:
# - Hashed password (not plaintext)
# - Absolute path for data_dir
# - Selected serial port
# - Secure defaults applied
```

---

## Security Benefits

### 1. No Default Passwords
- ✅ No password hashes in repository
- ✅ Forces users to set strong passwords
- ✅ Password hashing automated
- ✅ Confirmation prevents typos

### 2. API Key Protection
- ✅ Clear placeholder values
- ✅ No real keys in examples
- ✅ Obvious "REPLACE_WITH" markers
- ✅ Links to get API keys provided

### 3. Minimal Attack Surface
- ✅ Public login disabled by default
- ✅ Welcome system disabled by default
- ✅ Session timeout enforced
- ✅ Conservative rate limiting

### 4. Clear Configuration
- ✅ All sensitive values clearly marked
- ✅ Comments explain security implications
- ✅ Absolute paths used in production
- ✅ Secure defaults throughout

---

## Installation Flow

### Old Flow (Insecure)
```bash
1. Copy config.example.toml to config.toml
2. Edit config.toml manually
3. Hope user changes all defaults
4. Hope user hashes password correctly
```

### New Flow (Secure)
```bash
1. Run: sudo ./install.sh
2. Enter secure password (prompted, verified)
3. Select serial port (menu-driven)
4. config.toml generated with hashed password
5. Ready to run with secure configuration
```

---

## Migration Guide

### For Existing Installations

If you have an existing config.toml:

```bash
# 1. Backup current config
cp config.toml config.toml.backup

# 2. Review new config.example.toml
diff config.toml config.example.toml

# 3. Add missing sections:
[meshtastic.ident]
enabled = true
interval_seconds = 3600

[storage.backup]
enabled = true
retention_days = 30
interval_hours = 24

[games]
tinyhack_enabled = true
tinymush_enabled = true

[games.tinymush]
data_dir = "./data/tinymush"

[welcome]
enabled = false
private_enabled = false
public_enabled = false
rate_limit_minutes = 5

# 4. Review security settings:
allow_public_login = false  # Recommended

# 5. Verify password hash is not default
[admin]
password_hash = "your_actual_hash_here"
```

---

## Testing Checklist

Before deployment, verify:

- [ ] No default passwords in config files
- [ ] All API keys replaced with real values
- [ ] Public login set to `false` (unless intentional)
- [ ] Data directory path is absolute
- [ ] Serial port is correct for your device
- [ ] Password hash is unique (not example value)
- [ ] Welcome system configured if enabled
- [ ] Backup retention appropriate for disk space

---

## Security Audit Notes

**Date:** October 12, 2025  
**Auditor:** Configuration security review  
**Scope:** Configuration files and installation process

### Findings
- ✅ No hardcoded credentials
- ✅ No default passwords
- ✅ Interactive password setup
- ✅ Secure defaults applied
- ✅ Clear security guidance
- ✅ Placeholder values obvious

### Recommendations Implemented
- [x] Remove all default passwords
- [x] Add password prompting to installer
- [x] Set secure defaults (public_login = false)
- [x] Use absolute paths in production
- [x] Add configuration validation
- [x] Document security implications

---

## Production Deployment

### Pre-Deployment
```bash
# 1. Generate secure password hash
echo -n "your_secure_password" | ./meshbbs --hash-password

# 2. Get weather API key (if using)
# Visit: https://openweathermap.org/api

# 3. Run installation
sudo ./install.sh

# 4. Verify configuration
sudo nano /opt/meshbbs/config.toml
```

### Post-Deployment
```bash
# 1. Verify no default values remain
grep -i "REPLACE_WITH" /opt/meshbbs/config.toml
# Should return API key line only (if not configured)

# 2. Check permissions
ls -la /opt/meshbbs/config.toml
# Should be: -rw-r--r-- bbs bbs

# 3. Test authentication
sudo systemctl start meshbbs
# Try logging in via mesh
```

---

## Support

For questions about configuration security:
- GitHub Issues: https://github.com/martinbogo/meshbbs/issues
- Tag: `security`, `configuration`

---

**Status:** ✅ SECURE - Ready for production deployment
