# Troubleshooting

Common issues and fixes.

## Device not detected

- Confirm the serial port path and permissions
- Ensure Meshtastic serial is set to PROTO mode
- Try power-cycling the device and cable

## No DM after ^LOGIN

- Mesh can be congested; try `^HELP` to re-engage
- Ensure your node can reach the BBS node (mesh connectivity)

## Session logs out unexpectedly

- The BBS enforces inactivity timeouts; interact periodically

## Weather not working

- Set `weather.api_key` in `config.toml`
- Verify your location and `location_type`

## Getting Help

- Run `meshbbs status` for a quick summary
- Open an issue on GitHub with logs and steps to reproduce
