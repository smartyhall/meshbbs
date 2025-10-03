# Migration to Compact UI (v1.0.36+)

## Overview

Starting with version 1.0.36, MeshBBS emphasizes the compact, letter/number-driven interface as the primary user experience. Legacy long-word commands (`READ`, `POST`, `TOPICS`, `LIST`) remain functional for backward compatibility but are no longer advertised in the primary help system.

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

3. **Verbose HELP** - Legacy commands moved to "Deprecated" section
   - Still documented in `HELP+` or `HELP V`
   - Clearly marked as "Deprecated (backward compat only - use M menu instead)"

### Technical Changes

- Legacy commands (`READ <topic>`, `POST <topic>`, `TOPICS`, `LIST`) still work via `try_inline_message_command()`
- No breaking changes to the command processing logic
- Tests updated to reflect new help text format

## Recommended User Flow

### New User Experience
1. Login via `^LOGIN` on public channel
2. Open DM to BBS node
3. See welcome message with hint: `M=messages H=help`
4. Press **M** to enter Messages menu
5. Press **1-9** to select topics
6. Use single-letter navigation: **U** (up), **B** (back), **L** (more)

### Legacy Users
- All existing commands continue to work
- `READ general`, `POST general hello`, `TOPICS` still functional
- Use `HELP+` to see full command reference including deprecated commands

## Benefits

✅ **Simplified onboarding** - New users see one clear interface  
✅ **Bandwidth optimized** - Fewer bytes in help text  
✅ **Backward compatible** - Existing scripts/workflows unaffected  
✅ **Progressive enhancement** - Easy to learn basics, advanced users can use shortcuts  

## For Documentation Writers

Update user guides and tutorials to emphasize:
- Press **M** for messages (not `TOPICS`)
- Use digits **1-9** to select items (not `READ <topic>`)
- Single-letter navigation: **U**, **B**, **L**, **H**, **Q**

Legacy commands can be documented in "Advanced" or "Command Reference" sections.

## For Sysops

No configuration changes required. To completely disable legacy commands in a future version, you can:
1. Remove calls to `try_inline_message_command()` 
2. Remove the legacy command handlers in `commands.rs`
3. Update tests to remove legacy command assertions

For now, both interfaces coexist peacefully.

## Timeline

- **v1.0.35 and earlier**: Both UIs advertised equally
- **v1.0.36**: Compact UI emphasized, legacy moved to verbose help
- **v1.1.0** (planned): Consider config flag `legacy_commands_enabled`
- **v2.0.0** (planned): Potentially remove legacy commands entirely

## Questions?

See the [Command Reference](user-guide/commands.md) for complete documentation of both the compact UI and legacy commands.
