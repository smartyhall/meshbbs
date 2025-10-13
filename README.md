<div align="center">
   <img src="images/meshbbs_logo.png" alt="meshbbs Logo" width="200" height="200">
  
   # Meshbbs
  
  **A modern Bulletin Board System for Meshtastic mesh networks**
  
   [![Version](https://img.shields.io/badge/version-1.0.101--beta-blue.svg)](https://github.com/martinbogo/meshbbs/releases)
   [![License](https://img.shields.io/badge/license-CC--BY--NC--4.0-green.svg)](LICENSE)
   [![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
   [![Platform](https://img.shields.io/badge/platform-Meshtastic-purple.svg)](https://meshtastic.org/)
  
  *Bringing the classic BBS experience to modern mesh networks*
  
   [🚀 Quick Start](#quick-start) • [📖 User Guide](#usage) • [📚 Documentation](docs/) • [🔧 API Reference](https://martinbogo.github.io/meshbbs/api/) • [🤝 Contributing](#contributing) • [💬 Support](#support)
</div>

---

## 🌟 Overview

Meshbbs revolutionizes communication on mesh networks by bringing the beloved Bulletin Board System experience to Meshtastic devices. Exchange messages, participate in forums, and build communities over long-range, low-power radio networks, all without traditional internet infrastructure.

Perfect for emergency communications, remote areas, outdoor adventures, and building resilient community networks.

## 📝 Release notes

- **1.0.101-beta (2025-10-12): Schema Alignment & Constructor Improvements** 🔧
   - **Schema Version Fix**: Corrected demo trigger objects from v1 to v2 schema
   - **Constructor Future-Proofing**: All constructors now use schema version constants
   - **Code Quality**: Eliminated magic numbers in favor of named constants
   - **Bug Fix**: Removed archived binary reference from Cargo.toml

- **1.0.100-beta (2025-10-12): TinyMUSH Game Engine - Production Ready** 🎮
   - **Complete MUD/MUSH Engine**: Interactive text-based adventure game with 20+ rooms
   - **Interactive NPCs**: 5 NPCs with dialogue trees, quests, and dynamic interactions
   - **Player Progression**: Inventory system, skills, achievements, and quest tracking
   - **Advanced Triggers**: 14 action types with conditional logic and rate limiting
   - **Economy System**: Currency, shops, trading, and item management
   - **5 Games**: 8ball, fortune, slots, tinyhack (dungeon crawler), tinymush (MUD)
   - **Production Infrastructure**: 
     - Automated install/uninstall scripts
     - Systemd service configuration
     - Raspberry Pi deployment guide
     - Backup system with retention policies
     - Security hardening (zero critical vulnerabilities)
   - **Test Coverage**: 237 tests passing (98% coverage)
   - **Performance**: 458 db ops/sec, 14.58 concurrent users/sec
   - **Documentation**: 73 markdown files with complete API reference

- 1.0.65-beta (2025-10-05): **Production-Ready Daemon Mode & Graceful Shutdown**
   - **🔧 Daemon Mode**: Run meshbbs as a background service on Linux/macOS with `--daemon` flag
   - **⚡ Graceful Shutdown**: Cross-platform signal handling (SIGTERM, SIGHUP, SIGINT, Ctrl+C/Break)
   - **📝 Smart Logging**: TTY-aware logging - file-only in daemon mode, console+file in foreground
   - **🧹 Dependency Cleanup**: Removed 5 unused crates (220 fewer lines in Cargo.lock)
   - **🎯 Custom Implementation**: No external daemon dependencies, works perfectly on macOS/Linux
   - Includes management script (`scripts/meshbbs-daemon.sh`) for start/stop/restart/status/logs

- 1.0.60-beta (2025-10-05): **Welcome Queue Rate Limiting Optimization**
   - Startup welcome queue now processes every 30 seconds (10× faster than before)
   - Real-time node detections still rate-limited at 5 minutes to prevent spam
   - **Impact**: 17-node startup queue completes in ~8.5 minutes instead of ~85 minutes
   - Queue monitoring available via `data/welcome_queue.json` with real-time countdown
   - Bifurcated rate limiting: planned queue welcomes bypass cooldown, spontaneous detections enforce it

- 1.0.55-beta (2025-10-04): **Message Replication Infrastructure**
   - Added `message_id` (6-byte unique identifier) and `crc16` (integrity checksum) to all messages
   - Foundation for future inter-BBS message distribution and synchronization
   - Migration tool (`migrate_messages`) for updating existing message archives
   - Fully backward compatible with existing messages

- 1.0.50-beta (2025-10-04): **Welcome System & Reliable Ping Implementation**
   - Automatic welcome messages for new nodes with default "Meshtastic XXXX" names
   - Private DM with setup instructions and fun personalized name suggestions (e.g., "🦊 Clever Fox")
   - Reliable ping system using TEXT_MESSAGE_APP with routing ACK confirmation
   - 50 adjectives × 50 animals × emojis = 2,500+ possible name combinations

- 1.0.44-beta (2025-10-03): **TinyHack Mini-Map Feature**
   - Added **M** command to display compact ASCII mini-map with fog of war
   - 6×6 grid showing player position, unexplored areas, and room types
   - Persistent exploration tracking across game sessions

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory and hosted at [GitHub Pages](https://martinbogo.github.io/meshbbs):

- **[Installation Guide](docs/getting-started/installation.md)** - Complete setup instructions
- **[Command Reference](docs/user-guide/commands.md)** - All available commands and usage
- **[API Documentation](https://martinbogo.github.io/meshbbs/api/)** - Generated Rust API docs
- **[Administration Guide](docs/administration/)** - BBS setup and management
- **[Hardware Compatibility](docs/hardware/)** - Supported devices and setup

> The documentation is maintained alongside the code and automatically updated with each release.

See also: [Permissions and third-party notices](PERMISSIONS.md) for external conversation links documenting permission context (e.g., Reddit thread referencing Anycubic ACE Pro RFID tooling).

### Building the API docs locally

You can generate the same Rust API docs on your machine:

1. Ensure Rust is installed (rustup).
2. Run: `cargo doc --no-deps --all-features`
3. Open: `target/doc/meshbbs/index.html`

These docs reflect the inline rustdoc comments throughout the codebase. If you add or change public APIs, please include rustdoc so the generated docs stay complete.

## ✨ Features

### 🔌 **Connectivity & Integration**
- **📡 Meshtastic Integration**: Direct communication via serial (USB/UART)
- **🛎️ Public Discovery + DM Sessions**: Low-noise public channel handshake leading to authenticated Direct Message sessions
- **📨 Broadcast Semantics**: Broadcasts are best‑effort; we can request an ACK and consider any single ACK as basic delivery confirmation (no retries). DMs remain reliable with ACK tracking and retries.
- **⚡ Async Design**: Built with Tokio for high performance
- **🔧 Daemon Mode**: Production-ready background service with graceful shutdown (Linux/macOS)

### 💬 **Communication & Messaging**
- **📚 Message Boards**: Traditional BBS-style message topics and forums
- **🎯 Dynamic Contextual Prompts**: Smart prompts showing current state (`unauth>`, `user@topic>`, `post@topic>`)
- **📜 Enhanced Help System**: `<prefix>HELP` (default `^HELP`) broadcasts all public commands for discovery, with BBS instructions via DM
- **📏 Optimized Message Size**: 230-byte limit optimized for Meshtastic constraints
   - **🎰 Public Slot Machine**: Fun `<prefix>SLOT` mini‑game (default `^SLOT`) with daily coin refills and jackpots
   - **🎱 Magic 8‑Ball (public)**: Ask `<prefix>8BALL` (default `^8BALL`) for a classic, emoji‑prefixed response (broadcast‑only)
   - **🔮 Fortune Cookies (public)**: Use `<prefix>FORTUNE` (default `^FORTUNE`) to get random Unix wisdom, quotes, and humor (broadcast‑only)
   - **🧭 TinyHack (DM)**: Optional ASCII roguelike door reachable via the `[G]ames` submenu (`G1` when enabled); per-user saves under `data/tinyhack/`

### 👥 **User Management & Security**
- **🔐 Robust Security**: Argon2id password hashing with configurable parameters
- **👑 Role-Based Access**: User, Moderator, and Sysop roles with granular permissions
- **🛂 Per-Topic Access Levels**: Config-driven read/post level gating
- **💡 Smart User Experience**: One-time shortcuts reminder, streamlined login flow

### 🛠️ **Administration & Moderation**
- **🧷 Persistent Topic Locks**: Moderators can LOCK/UNLOCK topics; state survives restarts
- **📊 Deletion Audit Log**: `DELLOG` command for accountability tracking using immutable audit logs
- **📈 Network Statistics**: Usage and performance monitoring

## 🚀 Quick Start

> **Prerequisites**: Rust 1.82+, Meshtastic device, USB cable

### 🦀 Installing Rust

Meshbbs requires Rust 1.82 or later. If you don't have Rust installed:

**Linux & macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**
Download and run [rustup-init.exe](https://rustup.rs/) from the official Rust website.

**All Platforms:**
For detailed installation instructions, visit the official Rust installation guide:
- 🌐 **[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)**

After installation, verify Rust is installed:
```bash
rustc --version
cargo --version
```

### 📦 Installation

**Option 1: Automated Installation (Linux/Raspberry Pi - Recommended)**

For Linux systems and Raspberry Pi, use the provided installation script:

```bash
# Clone the repository
git clone --recurse-submodules https://github.com/martinbogo/meshbbs.git
cd meshbbs

# Run the installer (will prompt for configuration)
sudo ./install.sh
```

The installer will:
- Build the release binary
- Create necessary directories
- Guide you through configuration (sysop password, serial port, etc.)
- Set up systemd service for automatic startup
- Install to `/opt/meshbbs`

**Option 2: Manual Installation (All Platforms)**

```bash
# Clone the repository
git clone --recurse-submodules https://github.com/martinbogo/meshbbs.git
cd meshbbs

# Build the project
cargo build --release

# Copy example configuration
cp config.example.toml config.toml

# Edit configuration (see below)
nano config.toml
```
### ⚙️ Configure Your BBS

After initialization, edit the `config.toml` file to set up your BBS:

```bash
# Open config.toml in your preferred editor
nano config.toml  # or vim, code, etc.
```

**Critical settings to configure:**

1. **📡 Meshtastic Connection** - Update your serial port:
   ```toml
   [meshtastic]
   port = "/dev/ttyUSB0"  # Change to your device port
   # macOS: often /dev/tty.usbserial-*
   # Windows: often COM3, COM4, etc.
   # Linux: often /dev/ttyUSB0, /dev/ttyACM0
   ```

2. **👑 Sysop Information** - Set your admin details:
   ```toml
   [bbs]
   name = "Your BBS Name"
   sysop = "sysop"  # This becomes your admin username
   location = "Your Location"
   ```

4. **🌤️ Weather Setup** - Configure OpenWeatherMap integration:
   ```toml
   [weather]
   api_key = "your_openweathermap_api_key"  # Get free at openweathermap.org
   default_location = "Portland"            # City name, zipcode, or city ID
   location_type = "city"                   # "city", "zipcode", or "city_id"
   country_code = "US"                      # Optional country code
   enabled = true                           # Enable weather functionality
   ```

3. **🔐 Set Sysop Password** - Secure your admin account:
   ```bash
   ./target/release/meshbbs sysop-passwd
   ```

### 🚀 Start Your BBS

```bash
# Start the BBS server (use your configured port)
./target/release/meshbbs start

# Or specify port if different from config
./target/release/meshbbs start --port /dev/ttyUSB0
```

### ⚡ Quick Commands

| Command | Description |
|---------|-------------|
| `meshbbs sysop-passwd` | Set/update sysop password interactively |
| `meshbbs hash-password` | Hash a password from stdin (for scripts) |
| `meshbbs start` | Start BBS server with config.toml settings |
| `meshbbs start --port /dev/ttyUSB0` | Override port from command line |
| `meshbbs status` | Show server statistics and status |
| `meshbbs check-device --port <PORT>` | Verify Meshtastic device connectivity |

## ⚙️ Configuration

Meshbbs uses a `config.toml` file for all settings. For automated setup, use `install.sh` (Linux/Raspberry Pi). For manual setup, copy `config.example.toml` to `config.toml` and edit as needed.

Topics are managed in `data/topics.json` (runtime store) and are seeded automatically on first startup. Manage topics interactively from within the BBS; existing installations with `[message_topics.*]` in TOML remain supported for backward compatibility (they'll be merged into the runtime store at startup).

<details>
<summary><strong>📋 View Example Configuration</strong></summary>

```toml
[bbs]
name = "meshbbs Station"
sysop = "sysop"
location = "Your Location" 
description = "A bulletin board system for mesh networks"
max_users = 100             # Hard cap on concurrent logged-in sessions
session_timeout = 10        # Minutes of inactivity before auto-logout
welcome_message = "Welcome to Meshbbs! Type HELP for commands."

[meshtastic]
port = "/dev/ttyUSB0"
baud_rate = 115200
# node_id = "0x1234ABCD"   # optional; used only as display fallback before radio reports its ID
channel = 0
min_send_gap_ms = 2000                  # Enforced minimum between sends (ms)
dm_resend_backoff_seconds = [4, 8, 16]  # Reliable DM retry schedule (s)
post_dm_broadcast_gap_ms = 1200         # Delay broadcast after DM (ms)
dm_to_dm_gap_ms = 600                   # Gap between DMs (ms)
help_broadcast_delay_ms = 3500          # Delay HELP public broadcast after DM (ms)

[storage]
data_dir = "./data"
max_message_size = 230        # Protocol hard cap

[weather]
api_key = "your_openweathermap_api_key"   # Get free at openweathermap.org
default_location = "Portland"             # City name, zipcode, or city ID  
location_type = "city"                    # "city", "zipcode", or "city_id"
country_code = "US"                       # Optional country code
cache_ttl_minutes = 10                    # Cache weather data (minutes)
timeout_seconds = 5                       # API request timeout
enabled = true                            # Enable weather functionality

[logging]
level = "info"
file = "meshbbs.log"
```
</details>

### 🎛️ Key Configuration Options

| Section | Purpose | Key Settings |
|---------|---------|--------------|
| `[bbs]` | Basic BBS settings | `name`, `sysop`, `max_users`, `session_timeout` |
| `[meshtastic]` | Device connection | `port`, `baud_rate`, `channel` |
| `[weather]` | OpenWeatherMap integration | `api_key`, `default_location`, `enabled` |
### Fairness / Writer Tuning Fields

These pacing controls reduce airtime contention and avoid triggering device / network rate limits:

* `min_send_gap_ms` – Global enforced minimum between any two text sends (hard floor 2000ms)
* `dm_resend_backoff_seconds` – Retry schedule for reliable DM ACKs (default `[4,8,16]` seconds)
* `post_dm_broadcast_gap_ms` – Additional gap before a broadcast that immediately follows a reliable DM
* `dm_to_dm_gap_ms` – Gap enforced between consecutive reliable DMs
* `help_broadcast_delay_ms` – Higher-level scheduling delay for the public HELP notice after its DM reply; effective delay is `max(help_broadcast_delay_ms, min_send_gap_ms + post_dm_broadcast_gap_ms)` (default 3500ms) to prevent an immediate broadcast rate-limit right after a DM

Metrics (preview):

- Reliable DMs: `reliable_sent`, `reliable_acked`, `reliable_failed`, `reliable_retries`, `ack_latency_avg_ms`
- Broadcasts: `broadcast_ack_confirmed` (at least one ACK observed), `broadcast_ack_expired` (no ACK before TTL)

| `[storage]` | Data management | `max_message_size` |
| `topics.json` | Forum topics (runtime) | Create/manage interactively; persisted to `data/topics.json` |

## 📖 Usage

### 🎮 Command Line Interface

```bash
# Start the BBS server
meshbbs start --port /dev/ttyUSB0

# Show status and statistics
meshbbs status

# Check Meshtastic device connectivity
meshbbs check-device --port /dev/ttyUSB0

# Set/update sysop password
meshbbs sysop-passwd

# Enable verbose logging
meshbbs -vv start
```

### 📡 Connecting via Meshtastic

Meshbbs uses a **two-step interaction model** that keeps the shared mesh channel quiet while enabling rich private sessions.

#### 🔍 **Step 1: Say Hello on the Public Channel**
Commands require a prefix to address the BBS. The default is `^`, but your sysop can set a different one in `bbs.public_command_prefix`:
- `<prefix>HELP` - Shows all public commands and BBS login info (default `^HELP`)
- `<prefix>LOGIN <username>` - Registers pending login for your node ID (default `^LOGIN`)
- `<prefix>WEATHER` - Get current weather information (default `^WEATHER`)
 - `<prefix>SLOT` / `<prefix>SLOTMACHINE` - Spin the emoji slot machine (costs 5 coins; daily refill to 100 when at 0) (default `^SLOT`)
 - `<prefix>SLOTSTATS` - Show your slot coin balance, wins, and jackpots (default `^SLOTSTATS`)
- `<prefix>8BALL <question>` - Magic 8-Ball oracle for life's mysteries (default `^8BALL`)
- `<prefix>FORTUNE` - Receive random wisdom and inspiration (default `^FORTUNE`)

#### 💬 **Step 2: Start Your Private Conversation**
After public `LOGIN`, open a private message to the BBS node to start your authenticated session.

#### 🎛️ Compact Message UI (DM Session)

Once logged in via DM, use the compact, single-letter flow:

- Topics (press M)
   - Digits 1‑9: select topic on the current page (root topics only)
   - Topics with children show a ‘›’ marker; selecting opens Subtopics
   - L: more topics, H: help, B: back, X: exit
   
  Subtopics
   - Digits 1‑9: select subtopic; nested levels supported
   - U/B: up one level; M: back to root Topics; L: more
- Threads (inside a topic)
   - Digits 1‑9: read thread
   - N: new thread (2 steps: title ≤32, then body ≤200)
   - F <text>: filter thread titles (repeat F to clear)
   - L: more, B: back (to Subtopics or Topics), M: topics, H: help
- Read view
   - +: next, -: prev, Y: reply, B: back, H: help
   - Shows the latest reply preview (prefixed with "- ")

Shortcuts:
- HELP / HELP+: compact vs. verbose help
- WHERE / W: show breadcrumb path, e.g. `[BBS] You are at: Meshbbs > Topics > hello > Threads`

Indicators:
- Topics list shows per-topic new message counts since your last login, e.g. `1. general (2)`
- Threads list shows a `*` on titles with new content since your last login

<details>
<summary><strong>📋 Complete Command Reference</strong></summary>

**Authentication Commands:**
```bash
LOGIN <user> [pass]       # Authenticate (set password if first time)
REGISTER <user> <pass>    # Create new account
LOGOUT                    # End session
CHPASS <old> <new>        # Change password
SETPASS <new>             # Set initial password (passwordless accounts)
```

**Navigation & Help:**
```bash
HELP / H / ?              # Compact help with shortcuts
HELP+ / HELP V            # Verbose help (chunked if needed)
M                         # Open message topics list
1-9                       # Pick a topic/thread from the current page
L                         # Load more topics/threads (next page)
WHERE / W                 # Show current breadcrumb path
U / B                     # Up/back (to parent menu)
Q                         # Quit/logout
```

**Message & Thread Actions:**
```bash
R                         # View recent messages in the current topic
P                         # Compose a new post in the current topic
N                         # Start a new thread from the threads list
Y                         # Reply when reading a thread
F <text>                  # Filter topics/threads by text
+ / -                     # Next/previous page within lists
.                         # Finish posting (if text already sent) or cancel
```

**Moderator Commands** (level ≥5):
```bash
D<n>                      # Delete the nth thread/message (with confirm)
P<n>                      # Pin/unpin the nth thread
R<n> <title>              # Rename a thread
K                         # Toggle topic lock in the current area
DL [page] / DELLOG [p]    # View deletion audit entries
```

**Sysop Commands** (level 10):
```bash
G @user=LEVEL|ROLE        # Grant a level (1/5/10) or role (USER/MOD/SYSOP)
USERS [pattern]           # List users (optional filter)
USERINFO <user>           # Show details for a user
WHO                       # List currently logged-in users
SESSIONS                  # Show active sessions
KICK <user>               # Disconnect a user session
BROADCAST <message>       # Send a broadcast to all users
SYSLOG <INFO|WARN|ERROR> <msg>  # Write to the admin/security log
ADMIN / DASHBOARD         # Summary of system statistics
```
</details>

### 🎯 Dynamic Prompts

Meshbbs shows contextual prompts that reflect your current state:

| Prompt | Meaning |
|--------|---------|
| `unauth>` | Not logged in |
| `alice (lvl1)>` | Logged in as alice, user level 1 |
| `alice@general>` | Reading messages in 'general' topic |
| `post@general>` | Posting a message to 'general' topic |

### 📏 Message Size Limit

Each outbound message (body + optional newline + dynamic prompt) is limited to **230 bytes** (not characters) to match Meshtastic constraints. Multi‑byte UTF‑8 characters reduce visible character count. The server applies a UTF‑8 safe clamp at send‑time and then appends the prompt, ensuring frames always fit.

Reply storage is structured and backward compatible: new replies record `timestamp`, `author`, and `content`, while legacy plain-string replies continue to display correctly.

## 🏗️ Architecture

Meshbbs is built with a clean, modular architecture in Rust:

```mermaid
graph TD
   M["Meshtastic Device"]
   SIO["Serial (USB/UART)"]
   R["Meshtastic Reader Task"]
   W["Meshtastic Writer Task"]

   M --- SIO
   SIO --> R
   W --> SIO

   R -- "TextEvent (mpsc)" --> SV["BBS Server"]
   R -- "our_node_id (mpsc)" --> SV

   SV -- "Outgoing (mpsc)" --> SCH["Scheduler"]
   SCH -- dispatch --> W

   SV --> SESS["Sessions"]
   SESS -->|per-node| SV
   SV --> PST["Public State"]
   SV --> STOR["Storage Layer"]
   STOR --> MSGDB["Message DB"]
   STOR --> USERDB["User DB"]
   SV --> CFG["Configuration"]
   SV --> WX["Weather Service"]
```

### 📁 Module Structure

- **`bbs/`**: Core BBS functionality and user interface
- **`meshtastic/`**: Meshtastic device communication layer
  - Parses protobuf frames and emits structured `TextEvent` items
- **`storage/`**: Message and file storage subsystem  
- **`config/`**: Configuration management

## 🛠️ Development

### 🔧 Building from Source

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run comprehensive test suite
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- start
```

### 🎛️ Feature Flags

Control optional functionality with Cargo features:

| Feature | Default | Description |
|---------|---------|-------------|
| `serial` | ✅ | Serial port communication |
| `meshtastic-proto` | ✅ | Protobuf parsing of Meshtastic packets |
| `weather` | ✅ | Real-time weather via OpenWeatherMap API |
| `api-reexports` | ✅ | Re-export internal types |

```bash
# Minimal build without optional features
cargo build --no-default-features

# Build with specific features only
cargo build --features "serial,weather"
```

### 📡 Meshtastic Protobuf Integration

For rich packet handling, enable the `meshtastic-proto` feature. Upstream protobuf definitions are included as a git submodule.

<details>
<summary><strong>🔧 Protobuf Setup Instructions</strong></summary>

**Fresh clone with submodules:**
```bash
git clone --recurse-submodules https://github.com/martinbogo/meshbbs.git
```

**Initialize submodules in existing clone:**
```bash
git submodule update --init --recursive
```

**Build with protobuf support:**
```bash
cargo build --features meshtastic-proto
```

**Update submodules:**
```bash
git submodule update --remote third_party/meshtastic-protobufs
git add third_party/meshtastic-protobufs
git commit -m "chore(deps): bump meshtastic protobufs"
```

**Use custom proto directory:**
```bash
MESHTASTIC_PROTO_DIR=path/to/protos cargo build --features meshtastic-proto
```
</details>

### 📂 Project Structure

```
meshbbs/
├── 📄 src/
│   ├── main.rs             # Application entry point
│   ├── lib.rs              # Library exports
│   ├── validation.rs       # Input validation helpers
│   ├── 🎮 bbs/             # Core BBS functionality
│   │   ├── server.rs       # BBS server implementation
│   │   ├── session.rs      # User session management
│   │   ├── commands.rs     # BBS command processing
│   │   ├── public.rs       # Public channel command parsing
│   │   └── roles.rs        # User role definitions
│   ├── 📡 meshtastic/      # Meshtastic integration
│   │   ├── framer.rs
│   │   ├── slip.rs
│   │   └── mod.rs
│   ├── 💾 storage/
│   │   └── mod.rs          # Data persistence
│   ├── ⚙️ config/
│   │   └── mod.rs          # Configuration management
│   └── 📋 protobuf/
│       └── mod.rs          # Protobuf definitions
├── 📚 docs/                # Project documentation (GitHub Pages)
│   ├── getting-started/
│   ├── user-guide/
│   ├── administration/
│   ├── hardware/
│   ├── development/
│   └── qa/
├── 🖼️ images/
│   └── meshbbs_logo.png
├── 🧰 scripts/
│   └── clean_workspace.sh
├── 🔧 third_party/
│   └── meshtastic-protobufs/
├── 📦 protos/              # Local proto placeholders
│   ├── meshtastic_placeholder.proto
│   └── README.md
├── 🧪 tests/               # Integration tests
│   └── test-data-int/      # Integration test fixtures used by Cargo tests
├── 📊 data/                # Runtime data (topics, messages, users)
├── 🛠️ build.rs
├── 📦 Cargo.toml
├── 📦 Cargo.lock
├── ⚙️ config.toml
├── 📝 config.example.toml
├── 🗒️ CHANGELOG.md
└── 📘 README.md
```

## 🗺️ Roadmap

### ✅ Recent Releases
- **v1.0.0 BETA** (2025-09-25): First public beta of the 1.x series

### 🚀 Upcoming Features
- [ ] **🔐 Locally encrypted data storage**: Enhanced security for stored messages and user data
- [ ] **📶 Support connecting node via WiFi and Ethernet**

## 💻 Hardware Compatibility

Meshbbs has been tested on the following Meshtastic devices:

| Device | Status |
|--------|--------|
| **Heltec V3** | ✅ Tested |
| **Heltec T114** | ✅ Tested |
| **LilyGO T-Deck** | ✅ Tested |
| **LilyGO T-Beam** | ✅ Tested |
| **RAK WisBlock** | ✅ Tested |

> **Other Meshtastic devices**: Meshbbs should work with any Meshtastic-compatible device, but we'd love to hear about your experiences adapting the BBS to other hardware! Please share your results in the discussions or issues.

## 🤝 Contributing

We welcome contributions from the community! Here's how to get started:

### 🚀 Quick Contribution Guide

1. **🍴 Fork** the repository
2. **🌟 Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **💻 Make** your changes with tests
4. **✅ Test** your changes: `cargo test && cargo clippy`
5. **📝 Commit** with clear messages: `git commit -m 'feat: add amazing feature'`
6. **📤 Push** to your branch: `git push origin feature/amazing-feature`
7. **🔄 Submit** a Pull Request

### 📋 Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation for user-facing changes
- Run `cargo fmt` and `cargo clippy` before committing
- Keep commits focused and atomic

**Note**: All code contributions require appropriate unit tests.

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 📄 License

<div align="center">

[![License: CC BY-NC 4.0](https://img.shields.io/badge/License-CC%20BY--NC%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc/4.0/)

</div>

This project is licensed under the **Creative Commons Attribution-NonCommercial 4.0 International License**.

**You are free to:**
- ✅ **Share** - copy and redistribute in any medium or format
- ✅ **Adapt** - remix, transform, and build upon the material

**Under these terms:**
- 🏷️ **Attribution** - Give appropriate credit and indicate changes
- 🚫 **NonCommercial** - No commercial use without permission

See the [LICENSE](LICENSE) file or visit [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/) for details.

## 🙏 Acknowledgments

Special thanks to the projects and communities that make meshbbs possible:

- 🌐 **[Meshtastic](https://meshtastic.org/)** - The open source mesh networking project
- ⚡ **[Tokio](https://tokio.rs/)** - Asynchronous runtime for Rust  
- 📻 **Amateur Radio Community** - For mesh networking innovations
- 🦀 **Rust Community** - For the amazing language and ecosystem

## 💬 Support

<div align="center">

**Need help? We're here for you!**

[![Email](https://img.shields.io/badge/Email-martinbogo%40gmail.com-blue?style=for-the-badge&logo=gmail)](mailto:martinbogo@gmail.com)
[![Issues](https://img.shields.io/badge/Issues-GitHub-orange?style=for-the-badge&logo=github)](https://github.com/martinbogo/meshbbs/issues)
[![Docs](https://img.shields.io/badge/Documentation-GitHub%20Pages-green?style=for-the-badge&logo=gitbook)](https://martinbogo.github.io/meshbbs)

</div>

### 🐛 Bug Reports
Found a bug? Please [open an issue](https://github.com/martinbogo/meshbbs/issues/new) with:
- Steps to reproduce
- Expected vs actual behavior  
- System information (OS, Rust version, device model)
- Relevant log output

### 💡 Feature Requests
Have an idea? We'd love to hear it! [Start a discussion](https://github.com/martinbogo/meshbbs/discussions) or create an issue.

### 🆘 Getting Help
- Check the [Documentation](docs/) for comprehensive guides
- Browse the [API Reference](https://martinbogo.github.io/meshbbs/meshbbs/) for technical details
- Search existing [Issues](https://github.com/martinbogo/meshbbs/issues) for solutions
- Join the discussion in [GitHub Discussions](https://github.com/martinbogo/meshbbs/discussions)

---

<div align="center">
  
**🎯 Meshbbs - Bringing bulletin board systems to the mesh networking age! 📡**

*Built with ❤️ for the mesh networking community*

[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Powered by Meshtastic](https://img.shields.io/badge/Powered%20by-Meshtastic-purple?style=flat)](https://meshtastic.org/)

</div>
