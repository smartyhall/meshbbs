# Migration to Compact UI (v1.0.36+)

## Overview

Starting with version 1.0.36, MeshBBS emphasizes the compact, letter/number-driven interface as the primary user experience. As of the current release, the legacy long-word commands (`READ`, `POST`, `TOPICS`, `LIST`) have been fully removed in favor of the compact shortcuts.

## What Changed

### User-Facing Changes

1. **Compact HELP** - No longer shows legacy commands
   - **Before**: `MSG: M topics; 1-9 pick; U up; +/-; F <txt>; READ/POST/TOPICS`
   - **After**: `MSG: M topics; 1-9 pick; U up; +/-; F <txt>`

2. **Login Hints** - New users see a helpful hint after login
   ```
   Welcome, alice you are now logged in.
   There are no new messages.
   Hint: M=messages H=help
   ```

3. **Verbose HELP** - `HELP+` now mirrors compact help and no longer lists the removed legacy commands

### Technical Changes

- Removed the `try_inline_message_command()` path and all supporting legacy command handlers
- Command processor now exclusively routes compact shortcuts (single letters/digits)
- Tests updated to reflect the compact-only behavior

## Recommended User Flow

### New User Experience
1. Login via `^LOGIN` on public channel
2. Open DM to BBS node
3. See welcome message with hint: `M=messages H=help`
4. Press **M** to enter Messages menu
5. Press **1-9** to select topics
6. Use single-letter navigation: **U** (up), **B** (back), **L** (more)

### Automation & Script Considerations
- Any external tooling that previously issued `READ`, `POST`, `TOPICS`, or `LIST` must be updated to use the compact equivalents (for example: `M`, topic digits, `P`, `R`)
- Public command prefixes continue to function unchanged

## Benefits

✅ **Simplified onboarding** - New users see one clear interface  
✅ **Bandwidth optimized** - Fewer bytes in help text  
✅ **Baseline simplicity** - One interface to learn and document  
✅ **Bandwidth optimized** - Fewer bytes in help text  
✅ **Streamlined automation** - Consistent command set for bots and scripts  

## For Documentation Writers

Update user guides and tutorials to emphasize:
- Press **M** for messages (not `TOPICS`)
- Use digits **1-9** to select items (not `READ <topic>`)
- Single-letter navigation: **U**, **B**, **L**, **H**, **Q**

Remove references to legacy commands from tutorials and reference material—they are no longer available.

## For Sysops

No configuration changes are required. If you previously customized documentation or onboarding messages, update them to reflect the compact commands.

## Timeline

- **v1.0.35 and earlier**: Both UIs advertised equally
- **v1.0.36**: Compact UI emphasized, legacy moved to verbose help
- **v1.0.37**: Legacy command handlers removed; compact shortcuts are required
- **Future**: Continue iterating on compact UX (configurable shortcuts, additional hints)

## Questions?

See the [Command Reference](user-guide/commands.md) for the authoritative compact command list.
