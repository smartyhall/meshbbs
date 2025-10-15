# Security Fix: Authentication Required for BBS Functions

## Issue
Unauthenticated users (not logged in) were able to access BBS functions including:
- Topics menu (M command)
- Games menu (G command)  
- Preferences menu (P command)
- Other BBS features

This was a security vulnerability that allowed unauthorized browsing of the message board system without requiring registration or login.

## Root Cause
The `handle_main_menu` function in `src/bbs/commands.rs` (line 481) was processing commands without checking if the user was authenticated via `session.is_logged_in()`.

## Fix Applied
Added an authentication check at the beginning of `handle_main_menu`:

```rust
// Security: Check authentication before allowing access to BBS functions
// Only HELP and QUIT commands are allowed for unauthenticated users
if !session.is_logged_in() && cmd != "H" && cmd != "?" && cmd != "Q" {
    return Ok(
        "Authentication required.\n\
        Please REGISTER <username> <password> or LOGIN <username> [password]\n\
        Type H for help.\n"
            .to_string(),
    );
}
```

This ensures that:
1. Unauthenticated users can only use HELP (H, ?) and QUIT (Q) commands
2. All other commands (M, P, G, etc.) require the user to be logged in
3. Users receive a clear message prompting them to register or login

## Files Changed
1. `src/bbs/commands.rs` - Added authentication check in `handle_main_menu()`
2. `tests/unauthorized_access_bug.rs` - Created tests to verify the fix
3. `tests/help_single_letter.rs` - Updated test expectations for unauthenticated responses
4. `tests/main_menu_aliases.rs` - Updated test expectations for unauthenticated responses
5. `tests/numbered_area_selection.rs` - Added login before accessing BBS functions

## Test Results
✓ All 3 new security tests pass:
  - `test_unauthenticated_blocked_from_topics`
  - `test_unauthenticated_blocked_from_games`
  - `test_unauthenticated_blocked_from_preferences`

✓ All existing tests pass (237 unit tests + integration tests)
✓ No regressions introduced

## Verification
The fix was verified by:
1. Creating a test session without logging in
2. Attempting to access Topics with 'M' command
3. Confirming the session state remains MainMenu (not Topics)
4. Confirming the response indicates "Authentication required"
5. Testing that after login, the commands work properly

## Security Impact
- **Before**: Unauthenticated users could browse topics, potentially see message counts, and access other BBS features
- **After**: Unauthenticated users must register or login before accessing any BBS functions
- Users are clearly prompted with instructions on how to authenticate

## Date
October 14, 2025
