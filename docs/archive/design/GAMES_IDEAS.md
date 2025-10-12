# Game Ideas for Meshbbs

## Overview

This document explores game concepts suitable for meshbbs's unique environment: text-based communication with ~200-character message limits and high-latency mesh network delivery (30 seconds to 5+ minutes between messages).

## Current Games

### ✅ Implemented

1. **TinyHack** - ASCII roguelike door game (DM-based)
   - Dungeon exploration with fog of war
   - Turn-based combat
   - Persistent character saves
   - Mini-map feature

2. **Slot Machine** - Public emoji slot game
   - Daily coin refills
   - Jackpot system
   - Persistent coin balance

3. **Magic 8-Ball** - Public oracle (broadcast-only)
   - Classic responses
   - Emoji-prefixed answers

4. **Fortune Cookies** - Public wisdom dispenser (broadcast-only)
   - Unix fortune-style quotes
   - Random wisdom and humor

## 🎮 Potential New Games

### 🥇 Tier 1: Highly Recommended

#### 1. Text Adventure/MUD Hybrid
**Concept**: Shared persistent world with room-based exploration

**Why it works**:
- Natural chunking into rooms/descriptions
- Turn-based, state stored server-side
- Proven concept (expands on TinyHack)
- High engagement potential

**Example**:
```
You are in a dusty cave.
Exits: N, S, E
Items: torch, rope
Players here: alice
```

**Commands**: `N`, `S`, `E`, `W`, `TAKE <item>`, `DROP <item>`, `LOOK`, `INV`, `SAY <msg>`

**Features**:
- Shared world state
- Player-vs-environment combat
- Item trading between players
- Persistent inventory
- Chat with other players in same room

**Implementation Notes**:
- Could extend TinyHack's dungeon system
- Add multiplayer room occupancy
- Implement item trading protocol
- Store world state in JSON

---

#### 2. Battleship (Async Multiplayer)
**Concept**: Classic naval combat game, perfectly suited for async play

**Why it works**:
- Designed for turn-based play
- Works great with high latency
- Competitive but low-pressure
- Nostalgic appeal

**Gameplay**:
1. Each player sets up 10×10 grid with ships
2. Players take turns calling shots: "B5"
3. Response: "HIT!", "MISS", or "SUNK!"
4. First to sink all opponent ships wins

**Example Session**:
```
Your Grid:        Enemy Grid:
  A B C D E F      A B C D E F
1 . X . . . .    1 . . . . . .
2 . X . . . .    2 . M . H . .
3 . X . . . .    3 . . . . . .

Your turn! Call shot (A1-J10):
> D2
Enemy: HIT!
```

**Commands**: `SETUP`, `SHOT <coord>`, `GRID`, `RESIGN`

**Implementation Notes**:
- Store two grids per game (player + opponent)
- Challenge system: `CHALLENGE @user`
- Game state: setup, playing, finished
- Persist in `data/battleship/`

---

#### 3. Daily Trivia Challenge
**Concept**: New trivia question posted daily, community competition

**Why it works**:
- Low time commitment
- Educational value
- Natural fit for mesh/radio enthusiasts
- Builds community engagement

**Categories**:
- Mesh Networking & Radio
- Geography & Locations
- Technology & Computing
- History & Culture
- Science & Nature
- General Knowledge

**Example**:
```
📅 Daily Trivia - Oct 5, 2025
Category: Mesh Networking

Q: What frequency band does LoRa
typically use in North America?

A) 433 MHz
B) 915 MHz
C) 2.4 GHz
D) 5.8 GHz

Reply: TRIVIA <A/B/C/D>
```

**Features**:
- First correct answer gets points
- Monthly leaderboard
- Streak bonuses
- Question archive
- Difficulty levels

**Commands**: `TRIVIA <answer>`, `TRIVIASTATS`, `TRIVIABOARD`

**Implementation Notes**:
- Store questions in JSON
- Track user scores and streaks
- Auto-post new question at midnight
- Could crowdsource questions from community

---

### 🥈 Tier 2: Good Fits

#### 4. Blackjack
**Concept**: Classic card game, pairs with slot machine gambling theme

**Why it works**:
- Simple rules everyone knows
- Single-player (no coordination needed)
- Reuses coin economy
- Quick to implement

**Gameplay**:
```
Dealer: [?][7]
You: [K][5] = 15

[H]it [S]tand [D]ouble?
Bet: 10 coins
```

**Commands**: `BLACKJACK [bet]`, `H`, `S`, `D` (double down)

**Implementation Notes**:
- ~50 lines of code
- Share coin balance with slots
- Standard deck, shuffle after each hand
- Basic strategy hints available

---

#### 5. Word Chain Game
**Concept**: Last letter becomes first letter of next word

**Why it works**:
- Simple but engaging
- Works well with multiple players
- Low stakes, social

**Example**:
```
Chain started by alice:
Apple → Elephant → Tiger → ?

Your turn! Next word starts with 'R'
CHAIN <word>
```

**Rules**:
- Must be valid English word
- Can't repeat words in current chain
- Chain resets after 20 words or timeout

**Commands**: `CHAIN <word>`, `CHAINSTART <word>`, `CHAINBOARD`

---

#### 6. Hangman
**Concept**: Classic word-guessing game

**Why it works**:
- Perfect for async play
- Can be collaborative (whole mesh guesses together)
- Visual element works in ASCII

**Example**:
```
Category: Technology
Word: _ _ _ _ _ _ _  (7 letters)
Guessed: A, E, R, S, T
Wrong: 3/6
Next guess: HANGMAN <letter>
```

**Modes**:
- Solo: Player vs BBS
- Collaborative: All players guess together
- Competitive: Player sets word for others

---

#### 7. Connect Four
**Concept**: Vertical strategy game

**Why it works**:
- Turn-based
- Visual in ASCII
- Quick games

**Example**:
```
| | | | | | | |
| | | | | | | |
| | | | | | | |
| | |X| | | | |
| |O|X| | | | |
|O|X|O| | | | |
 1 2 3 4 5 6 7

Your turn! DROP <column 1-7>
```

---

#### 8. Tic-Tac-Toe / Ultimate Tic-Tac-Toe
**Concept**: Simple grid game, or nested 9-grid variant

**Why it works**:
- Everyone knows it
- Ultimate variant adds depth
- Very compact state

**Ultimate Tic-Tac-Toe**:
- 3×3 grid of 3×3 grids
- Your move determines which grid opponent plays in next
- Much more strategic than basic version

---

### 🥉 Tier 3: Experimental

#### 9. Resource Trading Game
**Concept**: Idle/incremental game with player economy

**Features**:
- Gather resources over time
- Craft items
- Trade with other players
- Build up inventory

**Why it might work**:
- Designed for infrequent check-ins
- Social/economic gameplay
- Persistent progression

**Challenges**:
- More complex state management
- Balancing economy is hard
- May need frequent checking

---

#### 10. Mystery/Whodunit
**Concept**: Daily or weekly mystery puzzle

**Example**:
```
The Case of the Missing Node
Day 1/5: Initial Report

A mesh node went offline at 3am.
Last message: "Strange signal detected"

Clue: Boot logs show unexpected
reboot 5 minutes before disappearance

What do you investigate next?
A) Check power logs
B) Scan for interference  
C) Review mesh routing table
```

**Why it works**:
- Benefits from slow reveal
- Community collaboration
- Educational about mesh networking

---

#### 11. Choose Your Own Adventure
**Concept**: Branching narrative with choices

**Example**:
```
You stand at a crossroads.
Left path: Dark forest 🌲
Right path: Mountain trail ⛰️

Choose: [L]eft or [R]ight?
```

**Features**:
- Branching storylines
- Persistent story position
- Multiple endings
- Could be collaborative (vote on choices)

---

#### 12. Chess by Mail
**Concept**: Ultra-slow chess, perfect for mesh latency

**Why it works**:
- Chess-by-mail is a proven concept
- A move every few minutes/hours is fine
- Notation is compact: "e2e4"

**Features**:
- Standard chess rules
- PGN notation for moves
- Game can last days
- Rating system

**Challenges**:
- Need chess engine for validation
- Complex state management
- Games could take weeks

---

## 🎯 Design Principles for Mesh Games

### 1. Embrace Latency
Games should work well with 30-second to 5-minute delays between messages. Turn-based games are ideal.

### 2. Compact State
All game state must fit efficiently in meshbbs's data structures. Target ~200 bytes per state update.

### 3. No Real-Time Requirements
Avoid anything requiring split-second timing or reflexes. Asynchronous turn-based is the sweet spot.

### 4. Async-First Design
Players take turns, not simultaneous action. Design for "chess by mail" style gameplay.

### 5. Persistent Everything
Save all state frequently. Players might disconnect mid-game due to mesh network conditions.

### 6. Social Elements
Include leaderboards, high scores, statistics, and community challenges to build engagement.

### 7. Clear Feedback
Every action should have clear confirmation. Players need to know their command was received.

### 8. Graceful Degradation
Games should handle timeouts, disconnections, and incomplete games gracefully.

## 🚀 Implementation Priority

### Immediate (Quick Wins)
1. **Blackjack** - Reuses coin economy, simple to implement
2. **Hangman** - Classic, well-understood, minimal state

### Short Term (High Value)
3. **Daily Trivia** - Community engagement, educational
4. **Word Chain** - Social, low complexity

### Medium Term (Strategic)
5. **Battleship** - Competitive multiplayer, high engagement
6. **Text Adventure/MUD** - Builds on TinyHack, high replayability

### Long Term (Ambitious)
7. **Resource Trading** - Complex but rewarding
8. **Mystery Game** - Unique, community-focused

## 💡 Technical Implementation Notes

### Door System Architecture

```rust
pub enum GameDoor {
    TinyHack,        // Existing roguelike
    Blackjack,       // Card game
    Battleship,      // PvP naval combat
    Trivia,          // Daily quiz
    Hangman,         // Word guessing
    WordChain,       // Collaborative word game
    Adventure,       // MUD-style exploration
}

pub struct GameState {
    door: GameDoor,
    player: String,
    state_data: serde_json::Value,
    last_action: DateTime<Utc>,
    active: bool,
}
```

### Storage Structure

```
data/
  games/
    blackjack/
      <username>.json
    battleship/
      game_<id>.json
      challenges.json
    trivia/
      questions.json
      scores.json
      daily_state.json
    wordchain/
      current_chain.json
      leaderboard.json
```

### Command Integration

Games can be accessed via:
- **Main Menu**: Door letters (T for TinyHack, B for Blackjack, etc.)
- **Public Commands**: `^TRIVIA`, `^CHAIN`, etc.
- **DM Commands**: `BLACKJACK`, `BATTLESHIP SHOT D5`, etc.

## 📊 Game Metrics to Track

For each game, consider tracking:
- **Engagement**: Games started, completed, abandoned
- **Session Length**: Average time per game
- **Player Retention**: Return rate after first game
- **Social Metrics**: Multiplayer participation rate
- **Network Impact**: Messages per game, bandwidth usage

## 🎨 Design Philosophy

Meshbbs games should:
- **Respect the medium**: Work with, not against, mesh latency
- **Build community**: Encourage interaction between players
- **Stay lightweight**: Minimal bandwidth and storage
- **Be accessible**: Easy to learn, hard to master
- **Complement the BBS**: Enhance rather than distract from messaging

## 🤝 Community Contributions

We welcome game ideas and implementations from the community! When proposing a new game:

1. **Consider the constraints**: 200 chars, high latency, text-only
2. **Define the core loop**: What does a turn look like?
3. **Sketch the commands**: How do players interact?
4. **Think about state**: What needs to be saved?
5. **Plan for failure**: What happens on disconnect/timeout?

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for code contribution guidelines.

## 📚 References & Inspiration

- **Classic BBS Doors**: Trade Wars, Legend of the Red Dragon, BRE
- **MUDs**: Multi-User Dungeons (TinyMUD, DikuMUD)
- **Play-by-Mail Games**: Diplomacy, Chess by Mail
- **Text Adventures**: Zork, Adventure, Inform games
- **IRC/Discord Bots**: Trivia bots, word games

## 📝 Notes

Last updated: 2025-10-05 (v1.0.65-beta)

Have ideas for games not listed here? Open an issue or discussion on GitHub!

---

*Remember: The best mesh game is one that embraces the constraints and turns them into features.* 🎮📡
