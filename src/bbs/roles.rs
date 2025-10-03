//! Role and privilege levels used across the BBS.
//!
//! The BBS uses simple numeric levels that map to human‑readable roles. Higher values
//! imply a superset of lower capabilities. Use [role_name] for display.
/// Role / privilege level constants
pub const LEVEL_USER: u8 = 1;
pub const LEVEL_MODERATOR: u8 = 5;
pub const LEVEL_SYSOP: u8 = 10;

/// Return the human‑readable role name for a numeric level.
///
/// Levels ≥10 are treated as "Sysop", ≥5 as "Moderator", otherwise "User".
pub fn role_name(level: u8) -> &'static str {
    match level {
        LEVEL_SYSOP => "Sysop",
        LEVEL_MODERATOR => "Moderator",
        _ => "User",
    }
}
