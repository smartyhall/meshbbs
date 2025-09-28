# Device Setup

## Enable PROTO serial mode

Use the official Meshtastic CLI to enable PROTO serial mode:

```bash
meshtastic --set serial.enabled true --set serial.mode PROTO
```

## Find your serial port

- macOS: `/dev/tty.usbserial-*`
- Linux: `/dev/ttyUSB0` or `/dev/ttyACM0`
- Windows: `COM3`, `COM4`, etc.

## Optional: Baud rate

Default is 115200. Set in `config.toml` under `[meshtastic]`.
