# Connecting to the BBS

There are two channels of communication with the BBS:

1. Public channel (broadcasts)
2. Direct message (DM) session

## Public Channel

Anyone on the mesh can send prefixed commands (default prefix `^`; your sysop can configure this):

- `<prefix>HELP` — shows public commands and triggers a DM with setup tips (default `^HELP`)
- `<prefix>LOGIN <username>` — request a private session (default `^LOGIN`)

## Direct Message Session

After `<prefix>LOGIN` (default `^LOGIN`), open a DM to the BBS node and authenticate:

```
LOGIN <username> <password>
```

Then use `HELP` to see available commands. Use `M` to browse message topics.

## Tips

- Keep messages short (230 bytes max)
- If you get no reply, the link may be congested; try again later
- Use `WHERE` to display a breadcrumb of where you are
