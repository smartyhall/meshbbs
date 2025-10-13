# First Run

This guide covers your first end-to-end run after installation.

## 1) Initial Setup

### Option A: Using install.sh (Recommended for Linux/Raspberry Pi)

If you used the `install.sh` script, configuration is already complete. Skip to step 2.

### Option B: Manual Setup

Create your configuration file:

```bash
cp config.example.toml config.toml
```

Then set your sysop password:

```bash
./target/release/meshbbs sysop-passwd
```

## 2) Configure

Edit `config.toml` and set:
- `bbs.name`, `bbs.sysop`, and `logging.level`
- `meshtastic.port` to your device path (e.g., `/dev/tty.usbserial-*` on macOS)

## 3) Start the server

```bash
./target/release/meshbbs start
```

If a device is connected and in PROTO serial mode, the server will connect; otherwise it will run offline.

## 4) Create a session

On the public channel, send:

```
^LOGIN yourname
```

Note: Public commands use a configurable prefix. The default is `^`. If you've changed the prefix in `config.toml` (bbs.public_command_prefix), use your configured prefix here.

Then open a direct message to the BBS node and log in:

```
LOGIN yourname yourpassword
```

Type `HELP` for commands. Use `M` to browse topics.

## 5) Troubleshooting

- If the device is not detected, verify the serial port and that Meshtastic is set to PROTO serial mode.
- Use `meshbbs status` to print a quick status summary.
- See the Troubleshooting guide for more.
