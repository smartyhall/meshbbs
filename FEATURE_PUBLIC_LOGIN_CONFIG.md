# Public LOGIN Configuration Feature

## Summary

Added a new configuration option `allow_public_login` to control whether users can initiate login via public channel commands.

## Changes Made

### 1. Configuration (`src/config/mod.rs`)

- Added `allow_public_login: bool` field to `BbsConfig` struct
- Defaults to `true` for backward compatibility
- Includes comprehensive documentation about the security implications
- Updated `Config::default()` to include the new field

### 2. Server Logic (`src/bbs/server.rs`)

- Modified `PublicCommand::Login` handler to check `config.bbs.allow_public_login`
- When disabled, public LOGIN commands are silently ignored (no response to prevent enumeration)
- Direct message LOGIN continues to work regardless of this setting
- Added trace logging for debugging when public LOGIN is disabled

### 3. Configuration Example (`config.example.toml`)

- Added `allow_public_login = true` with explanatory comments
- Documents the security benefits of disabling public LOGIN

### 4. Documentation (`docs/getting-started/configuration.md`)

- Added new "BBS Settings" section documenting core BBS configuration
- Explains the `allow_public_login` option
- Describes security considerations around username enumeration
- Clarifies that DM-based login is unaffected

### 5. Testing (`tests/public_login_security.rs`)

- Added comprehensive test `public_login_disabled_by_config`
- Verifies public LOGIN is blocked when configured
- Confirms DM-based LOGIN still works
- Tests both registration and login flows
- All existing tests continue to pass

## Security Benefits

When `allow_public_login = false`:

1. **Prevents Username Enumeration**: Attackers cannot probe for valid usernames on public channels
2. **Enhances Privacy**: Authentication happens only in private direct messages
3. **Reduces Public Channel Noise**: Failed login attempts don't clutter public channels
4. **Maintains Backward Compatibility**: Defaults to `true`, existing configurations continue to work

## Usage

### Enable (Default Behavior)
```toml
[bbs]
allow_public_login = true
```

Users can login via:
- Public channel: `^LOGIN username`
- Direct message: `LOGIN username [password]`

### Disable (Enhanced Security)
```toml
[bbs]
allow_public_login = false
```

Users can only login via:
- Direct message: `LOGIN username [password]`

Public channel LOGIN commands are silently ignored.

## Testing

Run the test suite:
```bash
cargo test --test public_login_security
```

All tests should pass, including the new `public_login_disabled_by_config` test.

## Implementation Notes

- The feature uses a simple boolean flag with serde default
- Server-side check occurs before processing public LOGIN commands
- No database or storage changes required
- Backward compatible - existing configs will default to `true`
- DM-based authentication is completely unaffected

## Next Steps

1. Test the feature in a live environment
2. Update CHANGELOG.md if committing to release
3. Consider documenting in security best practices guide
