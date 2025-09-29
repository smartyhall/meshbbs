# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This file records notable changes for meshbbs. Starting with the 1.0.0 BETA baseline, new entries will be added above this section over time (e.g., 1.0.1, 1.0.2).

# Changelog

## [1.0.32] - 2025-09-29

### Changed
- Fortune module: replaced built‑in database with ~400 user‑provided fortunes (haiku, limericks, proverbs, jokes). Ensured every entry is <= 200 chars for mesh compatibility.
- Tests: made fortune tests count‑agnostic and focused on invariants (non‑empty, <=200 chars, printable, randomness variety).
- Docs: updated module docs and README release notes to reflect the new fortune set.

## [1.0.31] - 2025-09-29

### Changed
- Storage hardening: implement atomic write-then-rename with fsync for JSON writes across the storage layer (users, runtime topics, append logs), and add read-side resilience by stripping accidental leading NUL bytes before JSON parse.
- Sysop seeding: replace direct file writes with a centralized atomic helper (upsert_user_with_hash), ensuring the sysop record is created/updated atomically.
- Meshtastic NodeCache: atomic save (temp + fsync + rename + dir fsync) and resilient load (leading-NUL trim) to prevent partial write issues.

### Notes
- These changes reduce the risk of data corruption under crashes or power loss during writes, and guard reads against previously observed leading-NUL artifacts.

## [1.0.30] - 2025-09-29

### Fixed
- Slot machine jackpot persistence: occasional jackpot.json reset due to non-seeked overwrite. We now seek to start before truncate/write to ensure atomic overwrite semantics across platforms.

### Changed
- Tests: updated integration tests to use writable temp copies of fixtures (alice.json, carol.json, etc.) to avoid mutating tracked files during test runs.

## [1.0.25] - 2025-09-28

### Changed
- Runtime: Parameterized user-facing public messages to respect the configured public command prefix:
  - IDENT beacon hint now says: `Type <prefix>HELP for commands` (default `^`)
  - Public broadcasts for Slot Machine, Magic 8‑Ball, Fortune, and Slot Stats include the configured prefix
- Documentation: Updated README, user guides, getting-started, QA plan, and rustdoc to describe and demonstrate the configurable prefix (default `^`).

### Notes
- No functional behavior change aside from message formatting consistency.

## [1.0.22] - 2025-09-28

### Added
- Unit tests to verify UTF‑8 safe truncation behavior for log previews:
  - Ensures truncation never splits multi‑byte characters (em‑dash, emoji)
  - Guards against regression of the 1.0.21 fix

### Notes
- No functional changes beyond tests and release metadata.

## [1.0.21] - 2025-09-28

### Fixed
- Critical: Prevent panic when logging long message previews containing multibyte UTF-8 characters (e.g., em dash, emoji).
  - Replaced byte-slicing in log previews with a UTF-8 safe truncation helper.
  - Hardened parser slices in public command parser to avoid accidental mid-char slicing.
  - Observed crash looked like: `byte index N is not a char boundary; it is inside '—' ...`.

### Notes
- Recommended immediate update if you run public commands that may produce non-ASCII content (e.g., `^FORTUNE`).


## [1.0.21] - 2025-09-27

### Fixed
- Ident beacon could fire twice within the same scheduled minute (e.g., at :00 and :56) due to an elapsed-time-only duplicate guard.
  - Implemented robust per-minute boundary deduplication using epoch-minute key; guarantees at most one ident per boundary.

### Internal
- Restored test data files inadvertently modified by local runs.

---

## [1.0.18] - 2025-09-28

### Added
- Ident Beacon: New frequency option "5min" (in addition to 15min default, 30min, 1hour, 2hours, 4hours)

### Fixed
- Ident could stall in reader/writer mode when no device instance was present to report initialization.
  - Added a startup grace period (~120s) that allows the first ident after boot if no device is detected,
    while still waiting for `initial_sync_complete()` when a device is present.
  - Preserves UTC boundary scheduling and duplicate suppression semantics.

### Changed
- Improved rustdoc around ident scheduling and startup gating behavior.
- Updated configuration docs to include the new "5min" option and clarified behavior.

### Tests & Docs
- Expanded tests to cover 5‑minute frequency and boundary logic; cleaned warnings.
- Added/updated documentation under `docs/getting-started/configuration.md`.

---

## [1.0.17] - 2025-09-27

### Fixed
- **Ident Beacon Timing Issue**: Fixed critical timing bug where ident beacons were sent before radio initialization completed
  - Added proper check for `device.initial_sync_complete()` before sending ident beacons
  - Prevents beacons from transmitting during "Requesting initial config from radio" phase
  - Added debug logging to track initialization status
  - Ensures ident beacons only start after Meshtastic device is fully ready

### Technical
- Enhanced ident beacon logic with radio initialization state validation
- Improved test configuration compatibility
- Added comprehensive debug logging for initialization sequence

---

## [1.0.16] - 2025-01-28

### Added
- **OpenWeatherMap Integration**: Complete replacement of wttr.in weather service with OpenWeatherMap API
  - Support for city names, ZIP codes, and OpenWeatherMap city IDs
  - Configurable API key and cache TTL settings
  - Comprehensive error handling and graceful fallbacks
  - Real-time weather data with temperature, conditions, and humidity
- **Enhanced Weather Configuration**: New `[weather]` section in config with `api_key` and `cache_ttl_minutes` settings
- **Weather Service Testing**: Comprehensive test suite including integration tests with real API validation

### Changed
- **Weather Command Improvements**: Better error messages and more reliable weather data retrieval
- **Configuration Validation**: Enhanced validation for weather service configuration
- **Documentation Updates**: Updated README and configuration examples for OpenWeatherMap setup

### Technical
- Migrated from wttr.in HTTP scraping to OpenWeatherMap REST API
- Added proper HTTP client with timeout and retry logic
- Implemented structured weather data parsing and caching
- Enhanced test coverage for weather functionality

---

### Security
- API keys properly excluded from git repository via .gitignore
- Example files load configuration from config.toml instead of hard-coded values
- Secure credential management for production deployments

## [1.0.15-beta] - 2025-09-27

### Added
- **New Feature**: Configurable ident beacon system for periodic station identification
- Ident beacon broadcasts BBS name, node ID, timestamp, and help hint to public channel
- Configurable frequency options: 15min, 30min, 1hour, 2hours, 4hours (default: 15min)
- Enable/disable control via `[ident_beacon]` section in config.toml
- Smart time-boundary scheduling (beacons fire exactly at configured intervals)
- Fallback node ID handling when dynamic ID unavailable

### Enhanced
- Added comprehensive unit tests for ident beacon configuration and timing logic
- 14 new tests covering frequency validation, serialization, and integration
- Improved configuration validation with clear error messages for invalid frequencies

### Technical
- New `IdentBeaconConfig` struct with serde support for TOML configuration
- Time-based scheduling using chrono for UTC boundary calculations
- Integration with existing public channel broadcast system
- Backward-compatible configuration (defaults to enabled with 15min frequency)

## [1.0.13-beta] - 2025-01-27

### Fixed
- **Critical**: Fixed BBS becoming unresponsive after extended runtime (~1 hour+)
- Reader task could permanently exit on serial I/O errors, causing complete message processing failure
- Implemented resilient error handling: reader continues operating despite transient serial errors
- Added small delays (50-100ms) in error paths to prevent tight error loops
- Enhanced error logging with "continuing operation" messages for transparency

### Technical
- Changed fatal `return Err()` to warning logs in serial read error paths
- Reader task now only exits on shutdown signal or EINTR (controlled shutdown)
- Improved production stability for long-running deployments

## [1.0.12-beta] - 2025-09-26

### Fixed
- **Critical**: Public HELP messages (e.g., `<prefix>HELP`, default `^HELP`) are now chunked to stay under Meshtastic's 230-byte transmission limit
- Previous help message was 302 bytes, potentially causing transmission failures
- Help content now split into multiple chunks (220-byte safety margin): first chunk with main commands + "DM for BBS access", continuation chunks with "More:" header
- Each chunk sent with 2.5-second delays to respect rate limiting
- Added comprehensive test coverage for message chunking in `tests/help_message_chunking.rs`

### Technical
- Implemented intelligent command distribution across chunks to maximize information density
- Maintains existing scheduler integration and fallback paths
- Verified worst-case scenarios: long BBS names result in 217 + 185 byte chunks (well under limit)

## [1.0.11-beta] - 2025-09-26

### Added
- Enhanced public HELP command (`<prefix>HELP`, default `^HELP`) now broadcasts all available public commands to improve discoverability
- New behavior: public HELP sends BBS instructions via DM while broadcasting public commands list to channel
- Comprehensive test coverage for new help behavior in `tests/help_public_commands.rs`

### Changed  
- HELP command now shows: "Public Commands (for {user}): <prefix>HELP - Show this help | <prefix>LOGIN <user> - Register for BBS | <prefix>SLOT - Play slot machine | <prefix>SLOTSTATS - Show your stats | <prefix>8BALL - Magic 8-Ball oracle | <prefix>FORTUNE - Random wisdom | DM for BBS access" (default prefix `^`)
- Updated documentation in README.md and user guides to reflect improved command discoverability
- Enhanced help system description in README from "Compact HELP + verbose HELP+" to "^HELP broadcasts all public commands for discovery, with BBS instructions via DM"

### Fixed
- Resolved critical UX issue where mesh network users had no way to discover available public commands without prior knowledge

## [1.0.10-beta] - 2025-09-26

### Added
- Public command `<prefix>FORTUNE` (Fortune Cookies; default `^FORTUNE`): returns random wisdom from curated Unix fortune database entries. All entries under 200 characters for mesh-friendly transmission. Broadcast-only with 5-second per-node cooldown.
- Comprehensive unit test coverage for Fortune module (11 test functions covering database validation, functionality, thread safety, and content quality)
- Helper functions for Fortune module: `fortune_count()` and `max_fortune_length()` for diagnostics and testing
- Extensive rustdoc documentation for Fortune module with examples and thread safety notes
- Development guide for Fortune module at `docs/development/fortune-module.md` with architecture, maintenance, and troubleshooting information

## [1.0.9-beta] - 2025-09-26

### Added
- Public command `<prefix>8BALL` (Magic 8‑Ball; default `^8BALL`): returns one of 20 classic responses (emoji‑prefixed). Broadcast‑only with a lightweight per‑node cooldown like `<prefix>SLOT`.

### Changed
- Docs: README badge bumped to 1.0.9‑beta; user docs updated to include Magic 8‑Ball.

### Fixed
- Slot machine docs/tests alignment: `<prefix>SLOT` clarified as broadcast‑only with a behavior test to prevent regressions (default `^SLOT`).

## [1.0.8-beta] - 2025-09-26

### Changed
- Logging hygiene: demote noisy INFO logs to DEBUG in HELP flow, weather fetch/success, cache loads, serial open, reader/writer init, resend attempts, and per-message delivered ACK logs.

### Fixed
- Public `<prefix>SLOT` behavior corrected to be broadcast-only (no DM). `<prefix>SLOTSTATS` remains broadcast-first with DM fallback for reliability. (Default `^SLOT` / `^SLOTSTATS`.)

## [1.0.7] - 2025-09-26


- README and docs tweaks to clarify public broadcast ACK confirmation semantics and command examples. No functional code changes.

## [1.0.6] - 2025-09-26
## [1.0.20] - 2025-09-27

### Added
- Expanded rustdoc coverage across modules (`bbs/commands.rs`, `bbs/public.rs`, `bbs/roles.rs`, `meshtastic/framer.rs`, crate binary docs)
- New docs pages added to close gaps referenced by the docs index:
  - Getting Started: First Run
  - User Guide: Connecting, Message Topics, Troubleshooting
  - Administration: Setup, User Management, Moderation
  - Hardware: Device Compatibility, Device Setup
  - Development: Architecture, Building from Source
- Added `docs/index.md` and updated Jekyll config for better site navigation

### Changed
- README: added local API docs build instructions and ensured API docs link prominence
- Documentation links updated/fixed for GitHub Pages compatibility (absolute links to repository files where needed)
- Minor rustdoc warnings resolved (escaped angle brackets; relaxed HTML tag checks for generated protobuf docs)

### Tests & Build
- `cargo doc` passes with all features
- Full test suite green (`cargo test`)

Broadcast reliability telemetry and optional confirmation for public messages.

### Added
- Optional broadcast ACK request: broadcasts can set `want_ack`; if any node ACKs, we consider it a success ("at least one hop").
- Lightweight broadcast ACK tracking in the writer with a short TTL (no retries) to avoid ACK storms.
- Metrics: `broadcast_ack_confirmed` and `broadcast_ack_expired` counters for trend visibility.

### Changed
- Updated HELP/public broadcast paths to request ACKs for visibility (DMs remain reliable with retries/backoff).
- Documentation updates in `README.md` to describe broadcast confirmation semantics and new metrics.

### Notes
- Direct messages remain fully reliable with ACK tracking, retries, and latency metrics; broadcasts remain best‑effort but can now indicate basic delivery when at least one ACK is observed.

## [1.0.5] - 2025-09-25

Added a public‑channel slot machine mini‑game and related documentation updates.

### Added
- New public commands:
  - `<prefix>SLOT` / `<prefix>SLOTMACHINE` — spin the slot machine (5 coins per spin; daily refill to 100 when at 0) (default `^SLOT`)
  - `<prefix>SLOTSTATS` — show your coin balance, total spins, wins, and jackpots (default `^SLOTSTATS`)
- Persistent per‑player state under `data/slotmachine/players.json` with safe file locking
- Jackpot and stats tracking (total_spins, total_wins, jackpots, last_spin, last_jackpot)
- Runtime packaging skeleton includes `data/slotmachine/.keep`

### Changed
- Updated user documentation (`docs/user-guide/commands.md`) and `README.md` to include new commands and feature blurb
- Bumped crate version to 1.0.5

## [1.0.0 BETA] - 2025-09-25

This is the first public beta for the 1.x series of meshbbs. It stabilizes the core architecture and user experience for Meshtastic devices while inviting broader testing and feedback before the final 1.0.0 release.

What’s in this beta (high level):
- Compact DM‑first UI optimized for Meshtastic frame limits (≤230 bytes), with Topics → Subtopics → Threads → Read navigation, paging, filtering, and breadcrumbs
- Robust help system: single‑frame compact HELP and multi‑chunk verbose HELP+
- Role‑based permissions: users, moderators, and sysop; moderation tools (pin/unpin, rename, lock/unlock, delete with audit log)
- Persistence: JSON‑backed users, topics/subtopics, messages, and replies; state survives restarts
- Meshtastic integration: protobuf support, hop‑limit 3, paced writer with scheduler to minimize airtime contention; optional weather integration
- Security and safety: Argon2id password hashing, command audit logging, UTF‑8 safe clamping/chunking, strict prompt‑aware size enforcement

Known limitations (to be refined before final 1.0):
- Some admin/dashboards are minimal or placeholders
- Performance tuning and CI coverage are ongoing
- Weather relies on an external service and may be rate‑limited

Upgrade and compatibility notes:
- On‑disk data persists in `data/` (topics at `data/topics.json`, messages under `data/messages/`)
- No schema migration is required from recent 0.9 builds; however, regenerating `config.toml` and reviewing new pacing/security options is recommended

Feedback welcome: please report issues and suggestions so we can tighten the final 1.0 release.