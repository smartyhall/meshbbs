# First Run

This guide covers your first end-to-end run after installation.

## 1) Initialize

If you haven't already, create the default configuration and runtime topics:

```bash
./target/release/meshbbs init
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
