# BBS Setup

This guide covers initial setup and ongoing administration.

## Initialize and configure

### Automated Setup (Recommended)
- For Linux/Raspberry Pi: Run `sudo ./install.sh`
- The installer handles configuration, password setup, and systemd service

### Manual Setup
- Copy example config: `cp config.example.toml config.toml`
- Edit `config.toml` to set BBS name, sysop, and Meshtastic port
- Set the sysop password: `meshbbs sysop-passwd`
- Topics are automatically seeded on first startup

## Managing topics

- Topics are maintained in `data/topics.json`
- Use moderator commands to lock/unlock topics and delete messages

## Backups

- Back up the `data/` directory (messages, users, slotmachine)
- Keep a copy of `config.toml`

## Logs

- Set `logging.file` to enable file logging
- Consider separate security/audit logs via `security_file` if configured
