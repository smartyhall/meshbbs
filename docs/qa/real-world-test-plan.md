---
title: Real‑World QA Test Plan
description: Comprehensive checklist to validate MeshBBS 1.0.41-beta on live Meshtastic devices
---

# Real‑World QA Test Plan — 1.0.41-beta

Use this comprehensive checklist to validate MeshBBS behavior on a live Meshtastic device and shared mesh network. Document all test results with exact commands, observed responses, and relevant log excerpts from `meshbbs.log`.

**Print-friendly:** Use your browser's Print function to save as PDF.

---

## Setup and Prerequisites

### Environment Requirements

- [ ] **Meshtastic Device**: Connected and configured to same channel as BBS
- [ ] **Platform**: macOS/Linux/Windows with terminal access
- [ ] **Repo Status**: Latest code builds and all tests pass
- [ ] **Configuration**: Valid `config.toml` with sysop credentials set

### Initial Build

```bash
cargo build --release
cargo test  # Verify all tests pass
```

- [ ] Build completes without errors
- [ ] Test suite passes (including `public_login_security` tests)

### Server Startup

```bash
./target/release/meshbbs start
```

Verify in logs:
- [ ] Meshtastic device connection established
- [ ] Primary channel logged (e.g., "Primary Meshtastic channel: 0")
- [ ] Ident beacon scheduled (if enabled in config)
- [ ] No startup errors or warnings

### Data Safety

Before testing:
- [ ] **Backup** `config.toml` and entire `data/` directory
- [ ] **Verify** sysop password is set: `./target/release/meshbbs sysop-passwd`
- [ ] **Document** your test environment: device model, firmware version, channel settings

---

## Public Channel Commands

**Note**: All public commands use a configurable prefix (default: `^`). Replace `^` with your configured `public_command_prefix` if different.

### Test: Public HELP Command

**Command**: Send `^HELP` on public channel

**Expected Behavior**:
- [ ] Receive DM with concise help text (compact, single-frame if possible)
- [ ] Public channel may show delayed acknowledgment (scheduler-controlled)
- [ ] DM includes authentication hints: "Open DM, say HI or LOGIN <name>"
- [ ] Prompt shown: `unauth>` (when not logged in)
- [ ] No rate-limit errors in logs

**Log Check**: Verify HELP DM queued and delivered without errors

### Test: Public LOGIN Security (Default Enabled)

**Command**: Send `^LOGIN testuser` on public channel

**Expected Behavior**:
- [ ] Public reply: "Login pending for 'testuser'. Open a direct message..."
- [ ] Subsequent DM to server allows completing login
- [ ] Public channel does NOT expose password or authentication details

**Security Note**: This is the legacy public login flow (enabled by default). Test the secure alternative in next section.

### Test: Public LOGIN Disabled (Security Enhanced)

**Setup**: Set `allow_public_login = false` in `config.toml`, restart server

**Command**: Send `^LOGIN testuser` on public channel

**Expected Behavior**:
- [ ] **No response** on public channel (silently ignored)
- [ ] Trace log entry: "Public LOGIN ignored (disabled by config)"
- [ ] User must open DM to authenticate

**Verification**: This prevents username enumeration attacks. Confirm DM-based login still works (see next section).

**Restore**: Set `allow_public_login = true` for remaining tests (unless testing secure-only mode)

### Test: Public Weather Command (Optional)

**Prerequisites**: Weather API key configured and `weather.enabled = true`

**Command**: Send `^WEATHER` on public channel

**Expected Behavior**:
- [ ] Broadcast weather message to public channel, OR
- [ ] DM fallback if broadcast fails
- [ ] Response includes location, temperature, conditions
- [ ] No spam or rate-limit violations (check scheduler logs)
- [ ] Cached result if requested within TTL window

### Test: Public Slot Machine

**Command**: Send `^SLOT` on public channel

**Expected Behavior**:
- [ ] Public broadcast with slot results: `^SLOT ⟶ 🍒 | 🍋 | 🍇 — <outcome>`
- [ ] Win/loss/jackpot handled correctly
- [ ] Per-user cooldown enforced (3 seconds default)
- [ ] Coin balance updates persisted

**Edge Case**: Rapid spins from same user should be rate-limited

### Test: Public Magic 8-Ball

**Command**: Send `^8BALL` on public channel

**Expected Behavior**:
- [ ] Public broadcast with random fortune-telling response
- [ ] Per-user cooldown (2 seconds default) enforced
- [ ] Response includes 🎱 emoji

### Test: Public Fortune Cookie

**Command**: Send `^FORTUNE` on public channel

**Expected Behavior**:
- [ ] Public broadcast with random fortune/proverb/joke
- [ ] Message ≤200 characters (mesh-compatible)
- [ ] Per-user cooldown (5 seconds default) enforced

---

## Direct Message Authentication

**Security Best Practice**: Always authenticate via DM to prevent public channel exposure of credentials.

### Test: New User Registration (DM)

**Command**: Open DM to BBS, send:
```
REGISTER testuser SecurePass123
```

**Expected Behavior**:
- [ ] Success message: "Registered as testuser."
- [ ] Welcome banner with quick-start hints: "M=messages, H=help"
- [ ] Unread message count (if any)
- [ ] Prompt changes to: `testuser (lvl1)>`
- [ ] User file created: `data/users/testuser_<timestamp>.json`

**Log Check**: Security log entry for new registration

**Edge Cases**:
- [ ] Reject invalid username (special chars, too long)
- [ ] Reject weak password (if validation enabled)
- [ ] Reject duplicate username (case-insensitive)

### Test: Existing User Login (DM)

**Setup**: Logout first: `LOGOUT`

**Command**: Send in DM:
```
LOGIN testuser SecurePass123
```

**Expected Behavior**:
- [ ] Success: "Logged in as testuser."
- [ ] Unread message summary shown
- [ ] Prompt: `testuser (lvl1)>`

**Wrong Password Test**:
```
LOGIN testuser WrongPassword
```
- [ ] Error: "Login failed. Check username/password."
- [ ] Remain logged out (`unauth>` prompt)
- [ ] Security log warning

### Test: First-Time DM Greeting

**Command**: Open fresh DM (new node), send:
```
HI
```

**Expected Behavior**:
- [ ] Welcome message with BBS name
- [ ] Instructions: "To register: REGISTER <name> <password>"
- [ ] Instructions: "To login: LOGIN <name> <password>"
- [ ] Prompt: `unauth>`

### Test: Password Change (DM)

**Prerequisites**: Logged in as testuser

**Command**:
```
CHPASS SecurePass123 NewSecurePass456
```

**Expected Behavior**:
- [ ] Success: "Password changed."
- [ ] Future logins require new password
- [ ] Old password no longer works

**Test New Password**:
```
LOGOUT
LOGIN testuser NewSecurePass456
```
- [ ] Login succeeds with new password

### Test: Set Password (Passwordless Account)

**Setup**: Create account without password via public LOGIN (if enabled):
1. Public: `^LOGIN nopassuser`
2. DM: `HI` (auto-login for new user)

**Command**:
```
SETPASS FirstPassword123
```

**Expected Behavior**:
- [ ] Success: "Password set. Required for future logins."
- [ ] Subsequent logins from new nodes require password

---

## Compact UI Navigation

MeshBBS 1.0.40+ uses a compact, keyboard-driven UI optimized for small messages.

### Test: Main Menu

**Command**: After login, press `M` or observe main menu

**Expected Behavior**:
- [ ] Banner: `[<BBS Name>] Main Menu`
- [ ] Options displayed:
  - `[M]essages` - Browse topics
  - `[U]ser account` - Account settings
  - `[T]inyHack` - Game (if `games.tinyhack_enabled = true`)
  - `[Q]uit` - Logout
- [ ] Single-letter shortcuts work: `M`, `U`, `T`, `Q`
- [ ] Prompt: `testuser (lvl1)>`

### Test: Topics Menu (Messages)

**Command**: Press `M` from main menu

**Expected Behavior**:
- [ ] Header: `[<BBS Name>] Topics`
- [ ] List shows root topics (max 5 per page)
- [ ] Topics with subtopics show `›` marker
- [ ] Each topic shows: number, name, description
- [ ] Footer: `1-9 pick | L more | H help | X exit`

**Navigation**:
- [ ] `1-9`: Select topic by number
- [ ] `L`: Load more topics (pagination)
- [ ] `U` or `B`: Go up/back (to main menu from topics)
- [ ] `X`: Exit session
- [ ] `M`: Return to topics (from anywhere)

### Test: Subtopics View

**Prerequisites**: Navigate to topic with subtopics (shows `›`)

**Command**: Press digit to select topic

**Expected Behavior**:
- [ ] Header: `[<BBS Name>][parent] Subtopics`
- [ ] List shows child topics
- [ ] Nested subtopics show `›` marker
- [ ] Footer includes `U` to go up

**Multi-Level Nesting**:
- [ ] Navigate through 3+ levels of subtopics
- [ ] `U` goes up one level at a time
- [ ] `M` returns directly to root topics
- [ ] Breadcrumb via `WHERE` shows full path

### Test: Threads List (Leaf Topic)

**Prerequisites**: Navigate to leaf topic (no subtopics)

**Expected Behavior**:
- [ ] Header: `[<BBS Name>][topic] Threads`
- [ ] Thread list shows: number, title, author, date
- [ ] Pinned threads show `📌` icon
- [ ] Unread threads show `●` indicator (if tracking enabled)
- [ ] Footer: `1-9 read | N new | F <text> filter | L more | B back`

**Thread Actions**:
- [ ] `1-9`: Read thread by number
- [ ] `N`: Create new thread (compose mode)
- [ ] `F hello`: Filter threads by keyword
- [ ] `F` (alone): Clear filter
- [ ] `L`: Paginate to next threads
- [ ] `B`: Back to parent (subtopics or topics)

### Test: Create New Thread

**Command**: From threads list, press `N`

**Expected Behavior**:
- [ ] Prompt: "New thread title (≤32):"
- [ ] Enter title: `Test Thread Title`
- [ ] Prompt: "Body (type '.' alone to finish):"
- [ ] Enter body text, type `.` on new line to finish
- [ ] Success: "Thread created."
- [ ] Return to threads list with new thread visible

**Validation**:
- [ ] Title truncated to 32 chars if too long
- [ ] Body accepts multiple lines
- [ ] Empty body rejected
- [ ] Thread appears in list immediately

### Test: Read Thread

**Command**: From threads list, press `1` (or any thread number)

**Expected Behavior**:
- [ ] Header: `[<BBS Name>][topic > title] p1/N`
- [ ] Message body displayed (truncated if >230 bytes)
- [ ] Reply preview shown (if replies exist): `— <author>: <preview>`
- [ ] Footer: `+ next | - prev | Y reply | B back | H help`

**Navigation**:
- [ ] `+`: Next thread
- [ ] `-`: Previous thread
- [ ] `Y`: Reply to thread
- [ ] `B`: Back to threads list

### Test: Reply to Thread

**Command**: From read view, press `Y`

**Expected Behavior**:
- [ ] Prompt: "Reply (type '.' alone to finish):"
- [ ] Enter reply text, type `.` to submit
- [ ] Success: "Reply added."
- [ ] Return to read view
- [ ] Reply preview appears on subsequent reads

### Test: Thread Filtering

**Command**: From threads list:
```
F security
```

**Expected Behavior**:
- [ ] Only threads with "security" in title shown
- [ ] Header updates: `[Filtered: security]`
- [ ] `F` (alone) clears filter
- [ ] Filter persists across pagination (`L`)

### Test: WHERE Command (Breadcrumb)

**Command**: At any navigation level, send `WHERE` or `W`

**Expected Behavior**:
- [ ] Breadcrumb path displayed:
  - Main Menu: `You are at: <BBS> > Main`
  - Topics: `You are at: <BBS> > Topics`
  - Subtopics: `You are at: <BBS> > Topics > parent > child`
  - Threads: `You are at: <BBS> > Topics > topic > Threads`
  - Read: `You are at: <BBS> > Topics > topic > Read`

---

## User Account Management

### Test: User Menu

**Command**: From main menu, press `U`

**Expected Behavior**:
- [ ] Header: `[<BBS Name>] User Account`
- [ ] Options shown:
  - `[C]hange password`
  - `[N]ew password (if none set)`
  - `[P]references` (if implemented)
  - `[B]ack to main menu`

### Test: Compact Help

**Command**: Send `HELP` or `H`

**Expected Behavior**:
- [ ] Single-frame compact help (≤230 bytes with prompt)
- [ ] Sections shown based on role:
  - **ACCT**: P (account menu), LOGOUT
  - **MSG**: M (topics), digits 1-9, +/-, F filter, N new, Y reply
  - **NAV**: WHERE/W, U/B (up/back), X (exit)
  - **MODS** (if moderator): PIN, RENAME, LOCK/UNLOCK, DELETE, DELLOG
  - **ADMIN** (if sysop): PROMOTE/DEMOTE, BROADCAST, SYSLOG, USERS, DASHBOARD
- [ ] Prompt appended to end

### Test: Verbose Help

**Command**: Send `HELP+`

**Expected Behavior**:
- [ ] Multiple DM chunks with detailed help
- [ ] Navigation section: Topics → Subtopics → Threads → Read flow
- [ ] Posting section: N (new thread), Y (reply), compose with `.`
- [ ] Admin sections shown only if authorized
- [ ] Prompt appears only on **last chunk**

### Test: Logout

**Command**: Send `LOGOUT`

**Expected Behavior**:
- [ ] Confirmation: "Goodbye! 73s"
- [ ] Session cleared
- [ ] Prompt changes to: `unauth>`
- [ ] Subsequent commands require re-authentication

---

## Moderator Features (Level ≥ 5)

**Setup**: Login as sysop, then:
```
PROMOTE testuser
```
Or use the role shortcut:
```
G @testuser=5
```

Verify:
- [ ] Success: "Set testuser role to moderator (5)."
- [ ] Log entry in security log

### Test: Pin/Unpin Thread

**Command**: From threads list:
```
PIN1
```
Or toggle:
```
P1
```

**Expected Behavior**:
- [ ] Success: "Thread 1 pinned." (or "unpinned")
- [ ] Thread shows `📌` icon
- [ ] Pinned threads appear at top of list
- [ ] State persists across restarts

### Test: Rename Thread

**Command**: From threads list:
```
RENAME1 Updated Thread Title
```
Or shortcut:
```
R1 Updated Thread Title
```

**Expected Behavior**:
- [ ] Success: "Thread 1 renamed."
- [ ] Title updated in list (truncated to 32 chars)
- [ ] Original author preserved
- [ ] Audit log entry created

### Test: Lock/Unlock Topic

**Command**: From threads list:
```
LOCK
```

**Expected Behavior**:
- [ ] Success: "Topic locked."
- [ ] Header shows: `[<BBS>][topic] Threads [locked]`
- [ ] New threads (`N`) rejected: "Topic is locked."
- [ ] Existing threads remain readable

**Unlock**:
```
UNLOCK
```
- [ ] Success: "Topic unlocked."
- [ ] `[locked]` indicator removed
- [ ] Posting restored

### Test: Delete Thread

**Command**: From threads list:
```
DELETE1
```
Or shortcut:
```
D1
```

**Expected Behavior**:
- [ ] Confirmation prompt: "Delete thread 1? (Y/N)"
- [ ] Press `Y`: "Thread deleted."
- [ ] Thread removed from list and filesystem
- [ ] Audit log entry: timestamp, topic, thread ID, moderator

**Cancel**:
- [ ] Press `N`: "Delete cancelled."
- [ ] Thread remains in list

### Test: Deletion Audit Log

**Command**:
```
DELLOG
```

**Expected Behavior**:
- [ ] List of deleted items with:
  - Timestamp
  - Topic/subtopic path
  - Thread/message ID
  - Moderator username
- [ ] Pagination: `DELLOG 2` for page 2
- [ ] Empty log: "No deletion log entries."

---

## Admin/Sysop Features (Level ≥ 10)

**Prerequisites**: Login as configured sysop account

### Test: User Management

**Command**:
```
USERS
```

**Expected Behavior**:
- [ ] List all registered users:
  - Username
  - Level/role
  - Registration date
  - Last seen (if tracked)
- [ ] Pagination if many users

**Filter by pattern**:
```
USERS test
```
- [ ] Shows only users matching "test"

### Test: User Info

**Command**:
```
USERINFO testuser
```

**Expected Behavior**:
- [ ] Detailed information:
  - Username
  - Level/role name
  - Registration timestamp
  - Post count (if tracked)
  - Last login (if tracked)
  - Password status (set/not set)

### Test: Promote/Demote Users

**Promote to moderator**:
```
PROMOTE alice
```
- [ ] Success: "Set alice role to moderator (5)."

**Demote**:
```
DEMOTE alice
```
- [ ] Success: "Set alice role to user (1)."

**Set specific level**:
```
G @alice=8
```
- [ ] Success: "Set alice role to level 8."
- [ ] Verify with `USERINFO alice`

### Test: Active Sessions

**Command**:
```
WHO
```
Or:
```
SESSIONS
```

**Expected Behavior**:
- [ ] List currently logged-in users
- [ ] Shows: username, level, node ID
- [ ] Indicates active vs. idle sessions

### Test: Kick User

**Command**:
```
KICK alice
```

**Expected Behavior**:
- [ ] Success: "User alice kicked."
- [ ] Alice's session forcibly logged out
- [ ] Alice receives: "You have been disconnected."
- [ ] Security log entry

### Test: Broadcast Message

**Command**:
```
BROADCAST Important mesh network announcement!
```

**Expected Behavior**:
- [ ] Message sent to public channel OR
- [ ] DM sent to all logged-in users
- [ ] Success confirmation
- [ ] Security log entry

### Test: System Log Entry

**Command**:
```
SYSLOG INFO Server maintenance scheduled for tomorrow
```

**Expected Behavior**:
- [ ] Entry added to security log file
- [ ] Format: `[timestamp] [INFO] [sysop] Server maintenance...`
- [ ] Supported levels: DEBUG, INFO, WARN, ERROR

### Test: Dashboard

**Command**:
```
ADMIN
```
Or:
```
DASHBOARD
```

**Expected Behavior**:
- [ ] Metrics summary:
  - Total registered users
  - Users by level (guests/users/mods/admins)
  - Total messages/threads
  - Recent registrations (last 24h)
  - Active sessions
  - Uptime
- [ ] Fits in 1-2 DM frames

---

## TinyHack Mini-Game (Optional)

**Prerequisites**: `games.tinyhack_enabled = true` in config

### Test: TinyHack Access

**Command**: From main menu, press `T`

**Expected Behavior**:
- [ ] TinyHack game starts in DM
- [ ] Persistent save loaded (if existing)
- [ ] New game initialized if first time
- [ ] Game prompts for actions: move, attack, inventory, etc.

**Game Flow**:
- [ ] Movement commands work (N/S/E/W)
- [ ] Combat encounters function
- [ ] Items can be collected and used
- [ ] Save persists: exit game, re-enter, state restored

**Save Location**: `data/tinyhack/<node_id>_<username>.json`

---

## Permission and Security Tests

### Test: Unauthorized Access Attempts

**Setup**: Login as level 1 user (testuser)

**Attempt moderator commands**:
```
DELETE1
LOCK
DELLOG
PIN1
```

**Expected**:
- [ ] Error: "Permission denied."
- [ ] HELP does not show restricted commands
- [ ] No state changes occur

**Attempt admin commands**:
```
PROMOTE alice
BROADCAST test
SYSLOG INFO test
USERS
KICK alice
```

**Expected**:
- [ ] Error: "Permission denied."
- [ ] Security log warning (unauthorized attempt)

### Test: Topic-Level Permissions

**Setup**: Create topic with restricted access (via `data/topics.json`):
```json
{
  "id": "restricted",
  "name": "Restricted",
  "read_level": 5,
  "post_level": 10
}
```

**As level 1 user**:
- [ ] Topic not visible in topics list
- [ ] Direct access attempt rejected

**As level 5+ user (moderator)**:
- [ ] Topic visible and readable
- [ ] Posting rejected (requires level 10)

**As level 10+ user (admin)**:
- [ ] Topic visible, readable, and postable

---

## Legacy Command Rejection

MeshBBS 1.0.40+ removed long-form commands. Verify rejection:

### Test: Deprecated Commands

**Commands to try**:
```
TOPICS
LIST
READ general
POST general Hello world!
```

**Expected for each**:
- [ ] Error: `Invalid command "TOPICS"` (or respective command)
- [ ] Truncated with `…` if too long for frame
- [ ] No side effects (navigation unchanged)
- [ ] Prompt remains same

**Alternative**: User must use compact UI (M, digits, etc.)

---

## Edge Cases and Stress Tests

### Test: Message Size Limits

**Attempt oversized message**:
1. From threads, press `N` to create thread
2. Paste body > 10 KB

**Expected**:
- [ ] Error: "Message too large (max 10240 bytes)."
- [ ] No thread created

### Test: UTF-8 and Special Characters

**Create thread with emoji/unicode**:
```
Title: 🎉 Test Unicode 日本語
Body: Testing UTF-8 characters: émojis 🚀 and symbols ™
```

**Expected**:
- [ ] Saved and displayed correctly
- [ ] No mojibake or character corruption
- [ ] Truncation respects UTF-8 boundaries (no split multi-byte chars)

### Test: Rapid Command Spam

**Send 10+ commands rapidly** (e.g., `HELP` x10)

**Expected**:
- [ ] Rate limiting enforces cooldowns
- [ ] No server crashes or hangs
- [ ] Scheduler queues messages appropriately
- [ ] Log shows no critical errors

### Test: Invalid Input

**Try malformed commands**:
```
CHPASS (no args)
LOGIN (no args)
F 
DELETE999 (non-existent thread)
```

**Expected**:
- [ ] Clear error messages
- [ ] No crashes or undefined behavior
- [ ] Session remains stable

### Test: Concurrent Sessions

**Setup**: Login from 2+ different nodes simultaneously

**Expected**:
- [ ] Both sessions work independently
- [ ] Session limits enforced (per `config.bbs.max_users`)
- [ ] No state collision or corruption

---

## Persistence and Restart Tests

### Test: Data Persistence

**Procedure**:
1. Create threads, post messages
2. Pin a thread, lock a topic
3. Stop server: `Ctrl+C`
4. Restart: `./target/release/meshbbs start`

**Expected After Restart**:
- [ ] All topics, subtopics, threads persist
- [ ] Pinned state preserved
- [ ] Locked state preserved
- [ ] User accounts and passwords intact
- [ ] Message history complete

**Files to check**:
- [ ] `data/topics.json` - topic structure
- [ ] `data/messages/<topic>/` - message files
- [ ] `data/users/` - user account files

### Test: Configuration Hot Reload

**Change config.toml** (e.g., toggle `allow_public_login`)

**Expected**:
- [ ] Server restart required for most changes
- [ ] No corruption of existing data
- [ ] New settings take effect after restart

---

## Weather Service Tests (Optional)

**Prerequisites**: API key configured, `weather.enabled = true`

### Test: Weather Caching

**Command**: Send `^WEATHER` 3 times within 10 minutes

**Expected**:
- [ ] First request: API call made, fresh data
- [ ] Subsequent requests: Cached data returned
- [ ] Cache TTL respected (10 min default)
- [ ] No excessive API usage

### Test: Weather Error Handling

**Setup**: Use invalid API key or location

**Expected**:
- [ ] Graceful error: "Error fetching weather. Please try again later."
- [ ] No server crash
- [ ] Log shows detailed error for debugging

---

## Ident Beacon Tests (Optional)

**Prerequisites**: `ident_beacon.enabled = true` in config

### Test: Ident Broadcast

**Wait for scheduled ident** (e.g., every 15 min at UTC boundaries)

**Expected**:
- [ ] Public broadcast: `[IDENT] <BBS name> (<node_id>) - <timestamp> UTC - Type ^HELP for commands`
- [ ] Sent at correct UTC boundary (e.g., :00, :15, :30, :45 for 15min freq)
- [ ] No duplicate idents within same minute
- [ ] Log: "Sent ident beacon"

**Startup Grace Period**:
- [ ] Ident waits for device sync if using `MeshtasticDevice`
- [ ] Short grace period if using reader/writer pattern

---

## Logging and Monitoring

### Test: Security Log

**File**: `meshbbs-security.log`

**Check for entries**:
- [ ] User registrations: `[SECURITY] User registered: testuser`
- [ ] Login attempts: `[SECURITY] Login: testuser from node <id>`
- [ ] Failed auth: `[SECURITY] Login failed: testuser`
- [ ] Admin actions: `[SECURITY] PROMOTE testuser by sysop`
- [ ] Unauthorized attempts: `[SECURITY] Permission denied: DELETE by testuser`

### Test: Application Log

**File**: `meshbbs.log`

**Check for**:
- [ ] Startup sequence complete
- [ ] Device connection established
- [ ] Message queue activity
- [ ] Scheduler stats (if enabled)
- [ ] No unexpected errors or warnings

---

## Performance and Stability

### Test: Long-Running Stability

**Procedure**: Leave server running for 24+ hours

**Monitor**:
- [ ] No memory leaks (check process memory usage)
- [ ] No log file bloat (reasonable size growth)
- [ ] Session cleanup working (idle sessions timed out)
- [ ] Scheduler stats healthy (queue not growing unbounded)

### Test: High Message Volume

**Simulate**: Multiple users posting many messages

**Expected**:
- [ ] Scheduler handles queue without drops (unless over max_queue)
- [ ] Messages delivered in order (mostly—mesh networks are best-effort)
- [ ] No database corruption
- [ ] Performance remains acceptable

---

## Final Checklist

### Documentation Alignment

- [ ] README.md reflects current version (1.0.41-beta)
- [ ] CHANGELOG.md documents all recent changes
- [ ] Configuration guide includes `allow_public_login` option
- [ ] User guide matches compact UI behavior

### Test Coverage

- [ ] All unit tests pass: `cargo test`
- [ ] Integration tests pass: `cargo test --test '*'`
- [ ] Manual testing covers all major features
- [ ] Edge cases documented and handled

### Deployment Readiness

- [ ] Release build tested: `cargo build --release`
- [ ] Example config up to date: `config.example.toml`
- [ ] Migration path from 1.0.40 to 1.0.41 documented
- [ ] Known issues documented (if any)

---

## Notes and Observations

Use this section to document:
- Unexpected behaviors
- Performance observations
- Mesh network quirks
- Device-specific issues
- Suggested improvements

**Example Entry**:
```
2025-10-02 14:35 PST: Tested public LOGIN disable feature. 
Successfully blocks enumeration. DM login works perfectly.
Node: Heltec v3, Firmware 2.3.2, Channel 0.
```

---

**Testing Complete**: Sign off below

| Tester | Date | Result | Notes |
|--------|------|--------|-------|
|        |      | ✅ / ⚠️ / ❌ |       |

---

*This test plan reflects MeshBBS 1.0.41-beta implementation as of October 2025.*
*Source: [MeshBBS GitHub Repository](https://github.com/martinbogo/meshbbs)*
 Real‑World QA Test Plan
description: Printer‑friendly checklist to validate meshbbs on a live Meshtastic device and shared mesh.
---

# Real‑World QA Test Plan

Use this printer‑friendly checklist to validate meshbbs behavior against a live Meshtastic device and the shared mesh. Keep a notes doc during testing with exact commands sent, observed responses, and relevant `meshbbs.log` snippets.

Tip for printing: Use your browser’s Print function to save this page as a PDF.

## Setup

### Prereqs

- [ ] One Meshtastic device connected and configured to the same channel as your BBS
- [ ] macOS Terminal (zsh)
- [ ] Current repo builds and tests pass

### Build and run

- [ ] Build: `cargo build --release`
- [ ] Start: `./target/release/meshbbs start`
- [ ] Confirm server logs show your Meshtastic port and that the device is connected

### Data safety

- [ ] Backup confirmed: `config.toml` and `data/topics.json`
- [ ] `config.toml` contains a valid sysop name and password set (see `meshbbs sysop-passwd`)

Note: For any problem found, capture the exact command, what happened, and relevant excerpts from `meshbbs.log`.

## Public channel (shared mesh) sanity

Note: Public commands use a configurable prefix. The default is `^`. In the examples below, replace `^` with your configured prefix if different.

### `^HELP` (from your Meshtastic client)

Expect:
- [ ] A DM arrives with compact help guidance
- [ ] A delayed public notice may appear after the DM (scheduler delay)

Check:
- [ ] DM contains a concise help sentence (not verbose), fits in one frame with prompt
- [ ] You see the prompt at the end of the DM: `unauth>` (if not logged in)

### `^LOGIN alice`

Expect:
- [ ] Public reply indicates login pending for `alice`
- [ ] You’re told to open DM to continue

### `^WEATHER` (optional)

Expect:
- [ ] Either a broadcast weather message (if allowed/available), or a DM fallback

Check:
- [ ] No floods; messages obey pacing. No rate‑limit errors in logs

## Direct Message: Authentication

### `REGISTER alice secretPass1`

Expect:
- [ ] Success message; a welcome/unread summary that fits in one frame
- [ ] Prompt changes to `alice (lvl1)>`

Check:
- [ ] First HELP after login shows a single shortcuts line once: `Shortcuts: M=areas U=user Q=quit`

### `HELP` (compact)

Expect:
- [ ] Single‑frame compact help including:
  - ACCT: `P` menu with `[C]hange/[N]ew pass` guidance, `LOGOUT`
  - MSG navigation hints (`M` topics; digits pick; `+`/`-`; `F` filter)
  - OTHER: `WHERE | U | Q`
- [ ] Fits in 230 bytes including prompt

### `HELP+` (verbose)

Expect:
- [ ] Multiple DM chunks; prompt appended to the last chunk only
- [ ] Detailed sections for navigation plus moderator/sysop tools (no legacy command references)

### `SETPASS newPassOnlyIfPasswordless` (only if the account had no password)

Expect:
- [ ] Success (or an explanatory message if already set)

### `CHPASS secretPass1 secretPass2`

Expect:
- [ ] Success; future logins require the new password on unbound nodes

### `LOGOUT` then `LOGIN alice secretPass2`

Expect:
- [ ] Login success; prompt is `alice (lvl1)>`
- [ ] Wrong password attempt yields an error

### `WHERE` (or `W`)

Expect:
- [ ] Breadcrumb reporting where you are (Login/Main/Topics/etc)

## Compact UI: Topics → Subtopics → Threads → Read

### `M` (go to Topics)

Expect:
- [ ] List of root topics only
- [ ] Items with children show a `›` marker
- [ ] Footer provides short instructions (L more, H help, X exit)

### Select a topic with subtopics (press a digit)

Expect:
- [ ] Subtopics view listing child topics (with `›` if nested)
- [ ] `U`/`B` goes up one level; `M` returns to root Topics; `L` paginates

### Select a leaf subtopic (press a digit)

Expect:
- [ ] Threads list for that subtopic
- [ ] Pinned threads show a `📌` icon
- [ ] If the topic is locked, header shows `[locked]`
- [ ] Footer: `Reply: 1-9 read, N new, L more, B back, F <text> filter` (if no filter applied)

### `N` (create a new thread)

Expect:
- [ ] Prompt for title (≤32)
- [ ] Then prompt for body (single‑message)
- [ ] Return to Threads with your new thread listed (title shown, truncated if needed)

### `F <text>` (filter) and `F` (clear)

Expect:
- [ ] Filter narrows visible thread titles; `F` with no text clears
- [ ] `L` paginates filtered results if many

### `1` (read first thread)

Expect:
- [ ] Read view shows `[BBS][topic > title] …` and body (truncated for space)
- [ ] If a reply exists, last reply line preview is shown prefixed with `—`
- [ ] Footer: `+` next, `-` prev, `Y` reply, `B` back, `H` help

### `Y` (reply), then type a short reply

Expect:
- [ ] Return to Read view; your reply is saved and previewed on subsequent displays

### `+` and `-` (navigate between threads)

Expect:
- [ ] Moves to next/previous thread; prompt and headers update accordingly

### `B` (from Read)

Expect:
- [ ] Returns to Threads
- [ ] Another `B` goes up (Threads → Subtopics; Subtopics → Topics); `M` always returns to Topics

### `X` (exit)

Expect:
- [ ] Ends the session (`Goodbye! 73s`) and prompt no longer updates

## Legacy inline commands (should reject)

Verify that deprecated long-form commands are no longer accepted.

### `TOPICS` / `LIST`

Expect:
- [ ] Reply: `Invalid command "TOPICS"` (or `"LIST"`)
- [ ] Prompt remains unchanged; use `M` to browse instead

### `READ general` (or another topic)

Expect:
- [ ] Reply: `Invalid command "READ general"` (truncated with `…` if needed)
- [ ] No navigation side effects; compact digits still work afterward

### `POST general Hello world!`

Expect:
- [ ] Reply: `Invalid command "POST general Hello world!"`
- [ ] No new thread appears in the Threads list

### `POST general` (multi-line)

Expect:
- [ ] Reply: `Invalid command "POST general"`
- [ ] Session stays in the same view (no hidden compose state)

### Unknown command (e.g., `NOPE`)

Expect:
- [ ] Terse reply: `Invalid command "NOPE"` followed by the prompt
- [ ] Stays within frame limit

## Moderator features (level ≥5)

### Login as sysop and promote `alice` to moderator

As sysop (login as your configured sysop account), then:

- `PROMOTE alice`
- or `G @alice=5` (equivalent)

Expect:
- [ ] Confirmation; role changes reflected in subsequent HELP output for `alice`

### Moderator in Threads: Pin/Unpin, Rename, Lock/Unlock

In Threads list:

- `P1` (toggle pinned for item 1)
- `R1 New Title` (rename thread title; truncates to 32)
- `K` (toggle lock state for current topic; also try `LOCK` and `UNLOCK` explicitly)

Expect:
- [ ] Visual changes in list headers: `📌` appears; `[locked]` on header when locked
- [ ] Posting while locked is denied; unlocking restores posting

### Delete with confirmation

- `D1` (delete item 1) → Confirm delete? (Y/N)

Expect:
- [ ] `Y` deletes and returns to Threads; `N` cancels
- [ ] Deletion removes the message and logs to audit

### Deletion audit log

- `DELLOG` (and optionally `DELLOG 2` for page 2)

Expect:
- [ ] Entries including timestamp, topic/id, and actor

## Admin/sysop dashboards and user admin (level ≥10)

### `USERS [pattern]`

Expect:
- [ ] List of users; optional filtering when pattern is provided

### `USERINFO alice`

Expect:
- [ ] Detailed info: level, posts, registered date

### `WHO` / `SESSIONS`

Expect:
- [ ] Basic summaries (if supported in your build; some are placeholders)

### `BROADCAST <message>`

Expect:
- [ ] Broadcast to public; DM fallback possible or admin‑log entry

### `SYSLOG INFO Hello mesh!`

Expect:
- [ ] Logged to security/audit log with the indicated level; success reply

### `ADMIN` or `DASHBOARD`

Expect:
- [ ] Quick metrics snapshot: total users, total messages, moderators, recent registrations, etc.

## Permissions/denials

### As a regular user (level 1)

Try moderator commands (`DELETE`, `LOCK/UNLOCK`, `DELLOG`) and sysop commands (`PROMOTE/DEMOTE`, `SYSLOG`, `BROADCAST`).

Expect:
- [ ] “Permission denied.” for restricted commands; HELP hides these unless authorized

## Size and prompt checks (sanity)

### DM length checks (visual)

Observe that:

- [ ] `HELP` fits in one frame with prompt
- [ ] Registration/login banners are concise and never get cut mid‑character
- [ ] `HELP+`: prompt only on last chunk

## Persistence checks

### Restart BBS

Stop and start the server.

Expect:
- [ ] Topics, subtopics, locked states, and messages persist (`data/topics.json` and files under `data/messages/…`)
- [ ] Users persist; logging back in works with saved password (if set)

## Optional: Weather caching sanity

### Send `^WEATHER` multiple times

Expect:
- [ ] Caching in effect; public broadcast or DM fallback; no spam; pacing respected

## Optional: Negative/edge cases

### Topic permission checks

Create a topic that requires higher read/post levels, then attempt actions as a lower‑level user.

Expect:
- [ ] Topic hidden when below `read_level`; post denied when below `post_level`

### Over‑long post content

From Threads, use `N` (new thread) or `Y` (reply) and attempt to send an oversized message (>10 KB) or one containing invalid characters.

Expect:
- [ ] Error about invalid characters or size limit; no message stored

### Invalid usernames on `REGISTER`/`LOGIN`

Expect:
- [ ] Clear validation error with guidance; no user created

## Operational tips

- [ ] Use `WHERE`/`W` often for breadcrumb clarity at each level you test
- [ ] For subtopics, verify multi‑level nesting by creating a child of a child, then navigating up with `B`/`U`/`M`
- [ ] For moderator actions, verify that UI hints appear in HELP only when the role is high enough

---

Source on GitHub: This plan reflects the current implementation documented in the project README and command reference.
