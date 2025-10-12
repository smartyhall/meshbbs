# Battleship Game Design for Meshbbs

## Overview

Battleship is a perfect fit for meshbbs's async, high-latency environment. The classic naval combat game naturally works with turn-based play where each move takes minutes, not seconds. This document outlines the complete player experience and technical implementation.

## 🎯 Why Battleship Works Perfectly

### Natural Async Gameplay
- **Classic "chess by mail" style**: Players have been playing Battleship by mail/email for decades
- **Low message volume**: ~20-50 total messages per game (very mesh-friendly)
- **No time pressure**: Think time between moves is a feature, not a bug
- **Clear state**: Grid representation designed for 200-byte limit
- **Suspenseful**: Waiting for opponent's move builds tension

### Technical Advantages
- **Compact state**: Two 10×10 grids = 200 bytes of data
- **Simple commands**: Single coordinate per turn
- **Deterministic**: No real-time sync required
- **Resumable**: Games can pause/resume easily
- **Low bandwidth**: One coordinate per message
- **Message-aware design**: Display formats fit 200-byte constraint

---

## 📏 Display Format Constraints

**CRITICAL**: Meshbbs has a ~200-byte message limit. All display formats must fit this constraint.

### Byte Size Analysis (UTF-8)

**What DOESN'T Fit** ❌
- Dual-grid display (6 rows): **306 bytes** - requires 2 messages
- Full 10-row dual grids: **500+ bytes** - requires 3+ messages
- Heavy emoji usage in large grids exceeds limits

**What FITS** ✅
- Single grid (6 rows): **~150 bytes**
- Single grid row: **46 bytes**
- Status updates: **35-72 bytes**
- Turn prompts: **37 bytes**
- Shot notifications: **35-62 bytes**
- Text-only full state: **72 bytes**

**Emoji Sizes**:
- Simple emojis (⛵🚢🚤💥🎯🎉): 3-4 bytes each
- Complex emojis with variation selectors (⛴️🛳️): 6-7 bytes each
- ASCII alternative (.XOMHS): 1 byte each

### Display Strategy

This design uses **sequential messages** for grid displays:
1. Message 1: Your grid (with your ships)
2. Message 2: Enemy grid (with your shots)
3. Message 3: Status/prompt

Alternatively, use **text-only mode** for maximum efficiency (single 72-byte message).

---

## 🎮 Complete Player Experience

### **Phase 1: Initiating a Game**

#### From Main Menu
```
MAIN MENU
[M] Topics  [G] Games  [H] Help

→ G

GAMES
[T] TinyHack      [S] Slots
[B] Battleship    [8] 8-Ball

→ B

=== BATTLESHIP ===
[N] New Challenge
[A] Accept Challenge
[C] Current Games (2)
[L] Leaderboard
[R] Rules  [Q] Back

→ N
Challenge whom? (username)
→ bob

Challenging bob...
*Challenge sent!*
Bob will be notified.

Check status with 'BATTLESHIP'
```

#### Receiving Challenge
```
*ALERT* bob challenged you!
Game: BATTLESHIP
Type 'BATTLESHIP ACCEPT bob'
or 'BATTLESHIP DECLINE bob'

→ BATTLESHIP ACCEPT bob

Challenge accepted!
Setting up game vs bob...
Ready to place ships!
```

---

### **Phase 2: Ship Placement**

#### Initial Setup Screen
```
=== BATTLESHIP vs bob ===
SETUP PHASE (1/5 ships placed)

Your Grid:
  A B C D E F G H I J
1 . . . . . . . . . .
2 . . . . . . . . . .
3 . . . . . . . . . .
4 . . . . . . . . . .
5 . . . . . . . . . .
6 . . . . . . . . . .
7 . . . . . . . . . .
8 . . . . . . . . . .
9 . . . . . . . . . .
0 . . . . . . . . . .

Ships to place:
⛴️ Carrier     (5) □□□□□
🚢 Battleship  (4) □□□□
🛳️ Cruiser     (3) □□□
🚤 Submarine   (3) □□□
⛵ Destroyer   (2) □□

Place carrier (5 spaces):
Format: A1 H or A1 V
(H=horizontal, V=vertical)

→ A1 H

*Placing carrier A1-E1...*

  A B C D E F G H I J
1 ⛴️⛴️⛴️⛴️⛴️ . . . . .
2 . . . . . . . . . .
3 . . . . . . . . . .

Ships to place:
✅ Carrier     (5)
🚢 Battleship  (4) □□□□
🛳️ Cruiser     (3) □□□
🚤 Submarine   (3) □□□
⛵ Destroyer   (2) □□

Place battleship (4 spaces):
→ B3 V

*Placing battleship B3-B6...*

  A B C D E F G H I J
1 ⛴️⛴️⛴️⛴️⛴️ . . . . .
2 . . . . . . . . . .
3 . 🚢 . . . . . . . .
4 . 🚢 . . . . . . . .
5 . 🚢 . . . . . . . .
6 . 🚢 . . . . . . . .
7 . . . . . . . . . .

✅ Carrier, Battleship
Place cruiser (3):
→ D8 H
*Placing cruiser D8-F8...*

[Continue until all 5 ships placed...]

✅ ALL SHIPS PLACED!
Waiting for bob...

You'll be notified when game
starts. Type 'BATTLESHIP' to
check status.
```

#### Quick Placement Option
```
=== BATTLESHIP vs bob ===
SETUP PHASE

[M] Manual placement
[R] Random placement
[P] Preset layout #

→ R

*Ships randomly placed!*

Your Grid:
  A B C D E F G H I J
1 . . . ⛵⛵ . . . . .
2 . 🚢 . . . . 🚤 . . .
3 . 🚢 . . . . 🚤 . . .
4 . 🚢 . ⛴️⛴️⛴️⛴️⛴️ 🚤 . .
5 . 🚢 . . . . . . . .
6 . . . 🛳️🛳️🛳️ . . . .

[A]ccept [R]e-roll

→ A
✅ Ready! Waiting for bob...
```

---

### **Phase 3: Playing the Game**

#### Your Turn (Sequential Messages)

**Message 1 of 3:**
```
=== BATTLESHIP vs bob ===
YOUR GRID:
  A B C D E F
1 . . . ⛵⛵ .
2 . 🚢 . . . .
3 . 🚢 . . . .
4 . 🚢 . ⛴️⛴️⛴️
5 . 🚢 . . . .
6 . . . 🛳️🛳️🛳️

Your Fleet:
⛴️ Carrier ████░ (1 hit)
🚢 Battleship ████
🛳️ Cruiser ███
```

**Message 2 of 3:**
```
ENEMY GRID:
  A B C D E F
1 . M . . . .
2 . . . H . .
3 . . . . . .
4 M . . . . .
5 . . H H . .
6 . . . . . .

Enemy Status:
🔥 ??? (2 hits)
🔥 ??? (2 hits)
? ??? ? ???
```

**Message 3 of 3:**
```
*** YOUR TURN ***
Shots: 6  Hits: 4
Call shot (A1-J10):
→ E5
```

*FIRING at E5...*
*Message sent to bob*

Waiting for response...
(avg response: 2-3 mins)

You'll be notified when bob
responds. Type 'BATTLESHIP'
to check status.
```

#### Response Received - HIT!
```
*BATTLESHIP UPDATE*
Game vs bob

>>> HIT! <<<
E5: Direct hit! 🎯

Enemy Grid:
  A B C D E F
1 . M . . . .
2 . . . H . .
3 . . . . . .
4 M . . . . .
5 . . H H H .  ← NEW!
6 . . . . . .

Shots: 7  Hits: 5
bob's turn now...
```

#### Response Received - SUNK!
```
*BATTLESHIP UPDATE*
Game vs bob

🎯 YOU SUNK CRUISER! 🎯

Enemy Grid:
  A B C D E F
1 . M . . . .
2 . . . H . .
3 . . . . . .
4 M . . . . .
5 . . 💥💥💥 .
6 . . . . . .

💥 Cruiser SUNK!
🔥 ??? (2 hits)
? ??? ? ???

4 ships remain
bob's turn now
```

#### Opponent's Turn - You're Hit!
```
*BATTLESHIP UPDATE*
bob fired at D4!

Your Grid:
  A B C D E F
1 . . . ⛵⛵ .
2 . 🚢 . . . .
3 . 🚢 . . . .
4 . 🚢 . 💥⛴️⛴️  ← HIT!
5 . 🚢 . . . .
6 . . . 🛳️🛳️🛳️

>>> bob HIT your Carrier! <<<

Your Fleet:
⛴️ Carrier    ████🔥 (1 dmg)
🚢 Battleship ████
🛳️ Cruiser    ███
🚤 Submarine  ███
⛵ Destroyer  ██

*** YOUR TURN ***
Call shot (A1-J10):
```

#### Ship Sunk - Yours!
```
*BATTLESHIP UPDATE*
bob fired at E4!

>>> bob SUNK your Carrier! <<<

Your Grid:
  A B C D E F
1 . . . ⛵⛵ .
2 . 🚢 . . . .
3 . 🚢 . . . .
4 . 🚢 . 💥💥💥💥💥
5 . 🚢 . . . .

Your Fleet:
💥 Carrier SUNK! (5)
🚢 Battleship ████
🛳️ Cruiser    ███
🚤 Submarine  ███
⛵ Destroyer  ██

4 ships remaining!

*** YOUR TURN ***
Call shot:
```

---

### **Phase 4: End Game**

#### Victory!
```
*BATTLESHIP UPDATE*
You fired at G3!

>>> YOU SUNK DESTROYER! <<<

🎉 VICTORY! 🎉

All enemy ships destroyed!

=== FINAL STATS ===
Winner: alice
Shots fired: 42
Hit rate: 47% (20/42)
Ships lost: 2

Enemy fleet destroyed:
💥 Carrier    (5) - A1-A5
💥 Battleship (4) - C3-F3
💥 Cruiser    (3) - D8-F8
💥 Submarine  (3) - H2-H4
💥 Destroyer  (2) - G3-H3

+50 XP  +100 coins
Rating: 1215 → 1232 (+17)

[R]ematch [M]ain Menu [L]eaderboard

→ L
```

#### Defeat
```
*BATTLESHIP UPDATE*
bob fired at B6!

>>> bob SUNK your Destroyer! <<<

💀 DEFEAT 💀

All your ships destroyed!

=== FINAL STATS ===
Winner: bob
Shots fired: 38
Hit rate: 42% (16/38)
Ships lost: 5 (all)

Your fleet:
💥 Carrier    (5) - A4-E4
💥 Battleship (4) - B2-B5
💥 Cruiser    (3) - D6-F6
💥 Submarine  (3) - G2-G4
💥 Destroyer  (2) - B6-C6

+10 XP (participation)
Rating: 1215 → 1203 (-12)

[R]ematch [M]ain Menu

Better luck next time!
```

---

## 📊 Game Management Commands

### Check Active Games
```
→ BATTLESHIP

=== ACTIVE GAMES ===

vs bob (your turn)
  Shots: 15, Hits: 7
  Your fleet: 5 ships
  Started: 2 hours ago
  [P]lay

vs carol (waiting)
  Last: You hit C3
  Carol's turn
  Started: 1 day ago
  [C]heck

vs dave (setup)
  Waiting for dave to place
  ships. Challenge expires in
  23 hours.

[N] New game [Q] Quit
```

### Quick Resume
```
→ BATTLESHIP PLAY bob

=== BATTLESHIP vs bob ===
*** YOUR TURN ***

Your Grid:
  A B C D E F
1 . . . ⛵⛵ .
2 . 🚢 . . . .
3 . � . . . .
4 . 💥 . ⛴️⛴️⛴️
5 . 💥 . . . .
6 . . . 🛳️🛳️🛳️

3 hits taken
[Press any key for enemy grid]
```

```
Enemy Grid:
  A B C D E F
1 . M H . . .
2 . M . . . .
3 M . . . . .
4 . . . . . .
5 . . . . . .
6 . . . . . .

Call shot (A1-J10):
```

---

## 💬 Chat During Game

### In-Game Communication
```
→ BATTLESHIP CHAT bob Nice shot!

*Message sent to bob*

→ BATTLESHIP

=== BATTLESHIP vs bob ===

bob: "Thanks! Lucky guess :)"
alice: "Nice shot!"

Your turn...

→ BATTLESHIP CHAT bob Where's your carrier? 😏

bob: "Find out the hard way! 🎯"
```

---

## 🎯 Smart Features

### **1. Move Validation**
```
Call shot:
→ C3

⚠️ Already fired at C3!
Result was: MISS

Call shot:
→ K3

⚠️ Invalid coordinate!
Valid: A-J, 1-10

Call shot:
→ D4

✅ Firing at D4...
```

### **2. Auto-Save & Resume**
```
*Connection lost...*

[Later, reconnecting...]

→ BATTLESHIP

*Restoring game vs bob*

You were in the middle of
your turn. Last action:
Shot C5 - HIT!

Continue? [Y/N]
→ Y

[Game resumes seamlessly]
```

### **3. Game Timeout Handling**
```
=== ACTIVE GAMES ===

vs bob (STALE)
  No activity for 48 hours
  bob hasn't responded
  [R]emind [F]orfeit [W]ait

→ R

*Reminder sent to bob*
"You have a pending Battleship
turn vs alice. Game expires in
24 hours."
```

### **4. Statistics Tracking**
```
→ BATTLESHIP STATS

=== YOUR STATS ===
Games played: 47
Wins: 28 (60%)
Losses: 19 (40%)

Average shots: 41
Hit rate: 44%
Favorite target: D5 (most hits)

Ships lost most: Destroyer
Ships sunk most: Cruiser

Current streak: 3 wins
Best streak: 7 wins

Rating: 1215 (Silver)
Rank: #42 of 156

Top opponent: bob (5 games)
```

---

## 🏆 Leaderboard & Rankings

```
→ BATTLESHIP LEADERBOARD

=== BATTLESHIP RANKINGS ===

🥇 1. charlie  1450  (Gold)
      W:42 L:8  84% wins

🥈 2. alice    1215  (Silver)
      W:28 L:19  60% wins

🥉 3. bob      1198  (Silver)
      W:25 L:15  63% wins

4. dave      1105  (Bronze)
5. eve       1087  (Bronze)
6. frank     1042  (Bronze)

Your rank: #2
Next rank: 1300 pts (Gold)

[M]ore [F]riends [B]ack
```

---

## 🎨 Compact Display Modes

**NOTE**: All display modes are designed to fit meshbbs's 200-byte message limit. Standard mode uses sequential messages; other modes use single messages.

### **Standard Mode** (Emoji ships, Sequential Messages)

*Requires 2-3 messages per turn (each fits 200-byte limit)*

**Message 1:**
```
Your Grid:
  A B C D E F
1 . . . ⛵⛵ .
2 . 🚢 . . . .
3 . 🚢 . . . .
4 . 🚢 . ⛴️⛴️⛴️
5 . 🚢 . . . .
6 . . . 🛳️🛳️🛳️

Fleet: ⛴️🚢🛳️🚤⛵
Hits taken: 0
(~150 bytes)
```

**Message 2:**
```
Enemy Grid:
  A B C D E F
1 . M . . . .
2 . . . H . .
3 . . . . . .
4 M . . . . .
5 . . H H . .
6 . . . . . .

Shots: 6  Hits: 4
Your turn. Shot?
(~135 bytes)
```

### **Compact Mode** (Last 5 shots only)

*Single message, action-focused*

```
Recent shots:
You → D5: HIT
Bob → C3: MISS
You → D6: HIT
Bob → B2: HIT!
You → D7: SUNK DESTROYER!

Your turn. Shot?
→ E5
(62 bytes - fits easily)
```

### **Text-Only Mode** (Maximum compression)

*Single message, ultra-compact*

```
YOU: 5ships 0dmg | BOB: 4ships 3dmg
Last: D7 SUNK! | Turn: YOU
Shot? → E5
(72 bytes - most efficient)
```

### **ASCII Mode** (No emojis, terminal-safe)

*Single message per grid, fully ASCII*

```
Your Grid:
  A B C D E F
1 . . . S S .
2 . B . . . .
3 . B . . . .
4 . B . C C C
5 . B . . . .
6 . . . D D D

Legend:
S=Sailboat B=Battleship
C=Cruiser D=Destroyer
X=Hit O=Miss
(~130 bytes, ASCII-safe)
```

---

## 🎮 Advanced Features

### **Salvo Mode** (Multiple shots per turn)
```
=== SALVO BATTLESHIP ===
Shots per turn = ships remaining

Your fleet: 4 ships = 4 shots

Fire 4 shots (space separated):
→ D4 D5 E4 E5

*Firing salvo...*
D4: MISS
D5: HIT!
E4: HIT!
E5: MISS

2 hits, 2 misses!
Waiting for bob's salvo...
```

### **Fog of War Mode** (Can't see misses)
```
Enemy Grid (Fog of War):
  A B C D E F
1 ? ? ? ? ? ?
2 ? H ? ? ? ?
3 ? ? ? ? ? ?
4 ? ? ? H ? ?

Only hits visible!
Adds strategy & memory.
```

### **Tournament Mode**
```
🏆 WEEKEND TOURNAMENT 🏆
Single elimination
8 players, 3 rounds
Prize: 1000 coins

Bracket:
alice vs bob
carol vs dave
eve vs frank
grace vs henry

Game rules: Standard
Time limit: 24h per game

[J]oin [R]ules [B]ack
```

---

## 🔧 Technical Implementation

### **Data Structures**

```rust
pub struct BattleshipGame {
    id: String,
    player1: String,
    player2: String,
    
    // 10x10 grids
    p1_ships: Grid,      // Ship placements
    p1_shots: Grid,      // Shots received
    p2_ships: Grid,
    p2_shots: Grid,
    
    // Game state
    current_turn: String,
    phase: GamePhase,    // Setup, Playing, Finished
    started_at: DateTime<Utc>,
    last_move: DateTime<Utc>,
    
    // Fleet tracking
    p1_fleet: FleetStatus,
    p2_fleet: FleetStatus,
    
    // Stats
    p1_shots_fired: u32,
    p1_hits: u32,
    p2_shots_fired: u32,
    p2_hits: u32,
    
    // Settings
    mode: GameMode,      // Standard, Salvo, FogOfWar
    time_limit: Duration,
}

pub struct Grid {
    cells: [[CellState; 10]; 10],
}

pub enum CellState {
    Empty,
    Ship(ShipType),
    Hit,
    Miss,
    Sunk(ShipType),
}

pub enum ShipType {
    Carrier,      // 5 spaces
    Battleship,   // 4 spaces
    Cruiser,      // 3 spaces
    Submarine,    // 3 spaces
    Destroyer,    // 2 spaces
}

pub struct FleetStatus {
    carrier: ShipHealth,
    battleship: ShipHealth,
    cruiser: ShipHealth,
    submarine: ShipHealth,
    destroyer: ShipHealth,
}

pub struct ShipHealth {
    size: u8,
    hits: u8,
    sunk: bool,
    coordinates: Vec<Coord>,
}

pub enum GamePhase {
    Challenge,      // Challenge sent, awaiting accept
    Setup(String),  // Player placing ships
    Playing,        // Active game
    Finished(String), // Winner
    Abandoned,      // Timeout/forfeit
}
```

### **Storage Structure**

```
data/
  battleship/
    games/
      <game_id>.json           # Active game state
    challenges/
      pending.json             # Open challenges
    players/
      <username>/
        stats.json             # Player statistics
        active_games.json      # Game IDs
    leaderboard.json           # Rankings
    settings.json              # Game configs
```

### **Commands**

```rust
pub enum BattleshipCommand {
    // Menu
    New(String),              // Challenge opponent
    Accept(String),           // Accept challenge
    Decline(String),          // Decline challenge
    List,                     // List active games
    Stats,                    // Show statistics
    Leaderboard,              // Show rankings
    
    // Setup
    Place(ShipType, Coord, Orientation),
    Random,                   // Random placement
    Reset,                    // Clear and restart placement
    
    // Playing
    Fire(Coord),              // Take shot
    Grid,                     // Show grids
    Fleet,                    // Show fleet status
    
    // Management
    Forfeit,                  // Give up
    Chat(String, String),     // Send message to opponent
    Remind(String),           // Remind opponent
    
    // Modes
    Play(String),             // Resume game vs opponent
    Check(String),            // Check game status
}
```

### **Message Flow**

```
Player A                    Server                     Player B
   |                           |                           |
   |-- CHALLENGE bob --------→ |                           |
   |                           |-- *bob challenged* -----→ |
   |                           |                           |
   |                           | ←-- ACCEPT alice ---------|
   |← *bob accepted* ----------|                           |
   |                           |                           |
   |-- PLACE ships ----------→ |                           |
   |                           | ←-- PLACE ships ----------|
   |                           |                           |
   |                           |-- *game start* ----------→|
   |                           |-- *game start* --------→  |
   |                           |                           |
   |-- FIRE D5 --------------→ |                           |
   |                           |-- *alice shot D5* ------→|
   |                           | ←-- RESPONSE: HIT --------|
   |← HIT! --------------------|                           |
   |                           |                           |
   |                           | ←-- FIRE B3 --------------|
   |← *bob shot B3* -----------|                           |
   |-- RESPONSE: MISS -------→ |                           |
   |                           |-- MISS ----------------→  |
   |                           |                           |
   [continues...]
```

---

## 📱 Notification System

```
*BATTLESHIP ALERT*
bob accepted your challenge!
Place your ships:
BATTLESHIP PLAY bob

---

*BATTLESHIP ALERT*
Your turn vs carol!
BATTLESHIP PLAY carol

---

*BATTLESHIP ALERT*
bob hit your Destroyer at C5!
💥 DESTROYER SUNK!
Your turn: BATTLESHIP PLAY bob

---

*BATTLESHIP ALERT*
🎉 You defeated dave! 🎉
Rating: 1215 → 1232 (+17)
BATTLESHIP STATS
```

---

## 🎯 Why This Works on Mesh

### **Perfect for High Latency**
- Each turn is independent
- No time pressure
- Players expect to wait
- Async is the natural flow

### **Low Message Volume**
- ~20-50 messages per complete game
- Each message is small (<200 chars)
- Grid updates are efficient
- Doesn't flood the mesh

### **Natural Suspense**
- Waiting for responses builds tension
- Like opening a letter
- Makes hits more satisfying
- Makes sinks more dramatic

### **Social Engagement**
- Can play multiple games at once
- Chat during games
- Tournaments and leagues
- Leaderboard competition

### **Resumable & Persistent**
- Games save automatically
- Can leave and return
- Handles disconnections gracefully
- No lost progress

---

## 🚀 MVP Implementation Plan

### **Phase 1: Core Game (Week 1-2)**
- [ ] Grid data structures
- [ ] Ship placement logic
- [ ] Shot validation
- [ ] Hit/miss/sink detection
- [ ] Basic storage

### **Phase 2: Game Flow (Week 2-3)**
- [ ] Challenge system
- [ ] Turn management
- [ ] Victory/defeat conditions
- [ ] Game state persistence

### **Phase 3: Display (Week 3-4)**
- [ ] Grid rendering (ASCII)
- [ ] Fleet status display
- [ ] Compact view modes
- [ ] Move history

### **Phase 4: Polish (Week 4-5)**
- [ ] Statistics tracking
- [ ] Leaderboard
- [ ] Rating system (ELO)
- [ ] Notifications

### **Phase 5: Advanced (Week 5-6)**
- [ ] Multiple game modes
- [ ] Tournament system
- [ ] Chat integration
- [ ] Rematch system

---

## 🎨 Design Philosophy

**Battleship embodies the meshbbs philosophy:**

1. **Async-First**: Designed for turn-based play
2. **Low Bandwidth**: Minimal message overhead
3. **High Engagement**: Strategy and suspense
4. **Social**: Competitive but friendly
5. **Persistent**: Games survive disconnections
6. **Accessible**: Everyone knows the rules
7. **Replayable**: Every game is different

**The mesh network latency becomes part of the experience** - like the suspense of waiting for your opponent to call out coordinates in the physical board game. It's not a bug, it's the authentic experience.

---

## 💡 Future Enhancements

- **Team Battleship**: 2v2 cooperative play
- **Custom Fleet**: Design your own ship sizes
- **Power-Ups**: Radar sweep, sonar ping
- **Campaign Mode**: AI opponents with personality
- **Spectator Mode**: Watch other games
- **Replay System**: Review past games
- **Mobile App**: Dedicated battleship interface

---

**Ready to add this classic game to meshbbs!** ⚓🎯

Last updated: 2025-10-05 (v1.0.65-beta)
