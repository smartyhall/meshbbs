# Command Reference

Complete reference for all meshbbs commands available to users.

See also: the [Games](./games.md) page for public channel mini‑games (e.g., Slot Machine, Magic 8‑Ball).

## Connection Commands

### Initial Discovery (Public Channel)

These commands are used on the public Meshtastic channel and require a prefix. The default is `^`, but your sysop can change it via `bbs.public_command_prefix`.

Reliability:
- Public broadcasts are best‑effort; the BBS may request an ACK and treats any single ACK as basic delivery confirmation, but it does not retry broadcasts.
- Direct messages (DM) are reliable with ACK tracking and retries.

| Command | Description | Example |
|---------|-------------|---------|
| `<prefix>HELP` | Show available public commands (broadcast) and send BBS instructions (DM) | `^HELP` (default) |
| `<prefix>LOGIN username` | Register for a private session | `^LOGIN alice` (default) |
| `<prefix>WEATHER` | Show current weather | `^WEATHER` (default) |
| `<prefix>SLOT` / `<prefix>SLOTMACHINE` | Spin the emoji slot machine (5 coins per spin; daily refill) | `^SLOT` (default) |
| `<prefix>SLOTSTATS` | Show your coin balance and slot stats | `^SLOTSTATS` (default) |
| `<prefix>8BALL` | Ask the Magic 8‑Ball a question; get a random response | `^8BALL` (default) |
| `<prefix>FORTUNE` | Get a random fortune from classic Unix wisdom databases | `^FORTUNE` (default) |

> 💡 **Discovery Tip**: New to the BBS? Send `<prefix>HELP` (default `^HELP`) on the public channel to see all available public commands broadcasted to everyone, plus get BBS setup instructions via DM.

## Session Commands (Direct Message)

After using `<prefix>LOGIN` (default `^LOGIN`) on the public channel, open a direct message to the BBS node to access these commands:

### Authentication

| Command | Description | Example |
|---------|-------------|---------|
| `LOGIN username [password]` | Log in (sets password the first time) | `LOGIN alice mypass` |
| `REGISTER username password` | Create a new account | `REGISTER bob secret123` |
| `LOGOUT` | End the current session | `LOGOUT` |
| `CHPASS old new` | Change your password | `CHPASS oldpass newpass` |
| `SETPASS new` | Set an initial password for passwordless accounts | `SETPASS mypassword` |

### Global shortcuts

| Command | Description | Notes |
|---------|-------------|-------|
| `HELP` / `H` / `?` | Show compact help | Fits within one frame (≤230 bytes) and adapts to your role |
| `HELP+` / `HELP V` | Show verbose help | Multi-part reply with full command explanations |
| `WHERE` / `W` | Show your breadcrumb | Displays the current location (e.g., `Meshbbs > Topics > general > Threads`) |

### Main menu shortcuts

| Command | Description |
|---------|-------------|
| `M` | Open the Topics view (paged list of root areas) |
| `P` | Open the Preferences menu (account & stats) |
| `T` | Launch TinyHack (only shown if the game is enabled) |
| `Q` | Log out and end the session (`Goodbye! 73s`) |

### Topics navigation (compact UI)

The compact UI uses single-letter commands and digits to navigate topics, threads, and messages quickly.

#### Topics view

- `1-9` — open the corresponding topic on the current page
- `L` — load the next page of topics (5 items per page)
- `H` — show inline help for the Topics view
- `B` / `Q` — return to the main menu
- `X` — exit the session immediately

#### Subtopics view

- `1-9` — open the selected subtopic; nested branches stay in Subtopics until a leaf is chosen
- `L` — load more subtopics
- `U` / `B` — go up one level (back to parent topics)
- `M` — jump directly back to the root Topics view
- `X` — exit the session

#### Threads view

- `1-9` — open the thread shown in that slot
- `N` — start a new thread (prompts for title, then body)
- `L` — load more threads (pinned items stay at the top)
- `F <text>` — filter thread titles (send `F` with no text to clear)
- `B` — go back (to Subtopics or Topics depending on hierarchy)
- `M` — return to Topics
- `Q` — return to the main menu; `X` exits the session

#### Reading a thread

- `+` / `-` — jump to the next/previous thread in the current topic
- `Y` — reply to the thread (single-message reply)
- `B` — return to the Threads list
- `H` — show inline help for available shortcuts
- `M` — return to Topics; `Q` returns to the main menu

### Posting threads and replies

- `N` (from Threads) — create a new thread. You will:
  1. Enter a title (truncated at 32 characters if longer)
  2. Enter the body as a single message (230-byte limit)
- `Y` (while reading) — reply to the current thread in one message
- Locked topics show `[locked]` in the header; posting or replying is blocked until unlocked

### Filtering threads

- `F <text>` — filter thread titles to those containing `<text>` (case-insensitive)
- `F` — clear the active filter and redisplay the full list
- Unread threads show a trailing `*`; pinned threads include a `📌` marker

### Preferences menu (`P`)

- `I` — view user details (username, node ID, level, session duration)
- `S` — view high-level BBS statistics (total users, messages, recent registrations)
- `C` — change password (prompts for current then new password; only when one exists)
- `N` — set an initial password (when no password is set yet)
- `L` — log out from the preferences screen
- `B` — return to the main menu while staying logged in

## Moderator tools (Level 5+)

Moderators gain additional actions in the compact UI plus access to administrative utilities.

### Threads view actions

| Command | Description |
|---------|-------------|
| `D<n>` | Delete the nth thread on the current page (prompts for confirmation) |
| `P<n>` | Toggle pinned state for the nth thread |
| `R<n> <new title>` | Rename the nth thread (title truncated to 32 characters) |
| `K` | Lock or unlock the current topic |

### Read view actions

| Command | Description |
|---------|-------------|
| `D` | Delete the currently open thread (prompts for confirmation) |
| `P` | Pin or unpin the current thread |
| `R <new title>` | Rename the current thread while reading it |
| `K` | Lock or unlock the current topic |

### Moderation utilities

| Command | Description | Notes |
|---------|-------------|-------|
| `DELLOG [page]` / `DL [page]` | View the deletion audit log | Optional page number (defaults to 1) |
| `USERS [pattern]` | List registered users, optionally filtered | Pattern matches against usernames (case-insensitive) |
| `WHO` | Show logged-in users | Returns a placeholder list in offline mode |
| `USERINFO user` | View detailed info (role, posts, registration date) | Requires an exact username |
| `SESSIONS` | List active sessions | Returns a placeholder list in offline mode |
| `KICK user` | Request that a user be logged out | Action is deferred for safety |
| `BROADCAST message` | Send a system broadcast to all users | Message is sanitized and limited to 5 KB |
| `LOCK topic` / `UNLOCK topic` | Lock or unlock a topic by name | Useful for automation scripts |
| `ADMIN` / `DASHBOARD` | Show aggregate statistics | Mirrors the Preferences `S` view with additional detail |

## Sysop Commands (Level 10)

Available only to system operators:

| Command | Description | Example |
|---------|-------------|---------|
| `PROMOTE user` | Increase a user's access level by one tier | `PROMOTE alice` |
| `DEMOTE user` | Decrease a user's access level by one tier | `DEMOTE bob` |
| `G @user=LEVEL\|ROLE` | Set level directly (1/5/10 or USER/MOD/SYSOP) | `G @alice=5` |
| `SYSLOG level message` | Write to the admin/security log | `SYSLOG info System check OK` |

## Dynamic Prompts

Meshbbs shows contextual prompts that reflect your current state:

| Prompt | Meaning |
|--------|---------|
| `unauth>` | Not logged in |
| `alice (lvl1)>` | Logged in as alice, user level 1 |
| `alice@general>` | Reading messages in 'general' topic |
| `post@general>` | Posting a message to 'general' topic |
| `alice@community>` → `alice (lvl1)>` | Using `B`/`U` goes up from Threads to Subtopics, then to Topics |

## Tips and Shortcuts

- **First-time help**: The first time you use `HELP` after login, you'll see a shortcuts reminder
- **Topic names**: Long topic names are truncated in prompts with ellipsis
- **Message limits**: Each message is limited to 230 bytes (optimized for Meshtastic)
	- The server clamps outputs UTF‑8 safely and appends the prompt; the total frame (body + prompt) ≤ 230 bytes.
    - Long thread reads are automatically split across multiple messages; the interactive prompt appears only on the final part.
- **Session timeout**: Sessions automatically timeout after inactivity (configurable by sysop)
- **Case sensitivity**: Commands are case-insensitive (`help`, `HELP`, and `Help` all work)

## Error Messages

Common error messages and their meanings:

| Error | Meaning | Solution |
|-------|---------|----------|
| `Invalid username` | Username doesn't meet requirements | Use 2-20 chars, letters/numbers/underscore only |
| `Wrong password` | Incorrect password provided | Check password or use `SETPASS` if passwordless |
| `Topic not found` | Message topic doesn't exist or you lack access | Use `M` to browse available topics or check permissions |
| `Access denied` | Insufficient privileges | Check your user level with sysop |
| `Message too long` | Message exceeds 230 byte limit | Shorten your message |
| `Session timeout` | Inactive too long | Log in again |

## Examples
### Compact Single-Letter Flow (DM Session)

```
> M
[Meshbbs] Topics
1. hello  2. general  3. technical
Type number to select topic. L more. H help. X exit
alice (lvl1)>

> 1
Messages in hello:
[BBS][hello] Threads
1 Test Title  2 Intro…
Reply: 1-9 read, N new, L more, B back, F <text> filter
alice@hello>

> N
[BBS] New thread title (≤32):
post@hello>

> Test Title
Body: (single message)
post@hello>

> This is the body of the test thread.
Messages in hello:
[BBS][hello] Threads
1 Test Title
Reply: 1-9 read, N new, L more, B back, F <text> filter
alice@hello>

> 1
[BBS][hello > Test Title] p1/1
This is the body of the test thread.
Reply: + next, - prev, Y reply, B back, H help
alice@hello>

> W
[BBS] You are at: Meshbbs > Topics > hello > Read
alice@hello>
```

### Basic Session Flow

```
Public channel (using the configured prefix; default shown):
> ^LOGIN alice
< Meshbbs: Pending login for 'alice'. Open a DM to start your session.

Direct message:
> LOGIN alice mypassword
< Welcome, alice you are now logged in.
< There are no new messages.
< Hint: M=messages H=help
< Main Menu:
< [M]essages [P]references [Q]uit
alice (lvl1)> M
< [Meshbbs] Topics
< 1. general  2. community  3. technical
< Type number to select topic. L more. H help. X exit
alice (lvl1)> 1
< Messages in general:
< [BBS][general] Threads
< 1. Welcome
< Reply: 1-9 read, N new, L more, B back, F <text> filter
alice@general> N
< [BBS] New thread title (≤32):
post@general>
> Mesh meetup
< Body: (single message)
post@general>
> Meet at 18:00 on main channel.
< Messages in general:
< [BBS][general] Threads
< 1. Mesh meetup
< Reply: 1-9 read, N new, L more, B back, F <text> filter
alice@general> B
< [Meshbbs] Topics
< 1. general  2. community  3. technical
< Type number to select topic. L more. H help. X exit
alice (lvl1)> Q
< Goodbye! 73s
```

### Moderator Example

```
mod (lvl5)> DELLOG
mod (lvl5)> DL 2
< Recent deletions:
< 2025-09-23 10:30 - general/msg456 deleted by mod
< 2025-09-23 09:15 - announcements/msg789 deleted by admin
mod (lvl5)> LOCK general
< Area 'general' is now locked to new posts
mod (lvl5)> UNLOCK general  
< Area 'general' is now open for posts
```