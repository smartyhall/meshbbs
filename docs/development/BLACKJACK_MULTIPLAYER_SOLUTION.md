# Multi-Player Blackjack: Identity & Concurrency Solutions

## The Problems

### 1. **Identity Problem**
When multiple players have active games, how does each player know which broadcast message is for them?

```
❌ CONFUSING:
=== BLACKJACK ===
You: K♠ 7♥ (17)
Dealer: 6♦ ?
^HIT or ^STAND

(Wait... is this MY game or Bob's game?)
```

### 2. **Command Ambiguity**
When Alice and Bob both have active games, whose game does `^HIT` apply to?

```
❌ AMBIGUOUS:
alice: ^BLACKJACK
[alice's game starts]
bob: ^BLACKJACK
[bob's game starts]
alice: ^HIT
(Which game does this ^HIT affect?)
```

### 3. **Concurrent Games**
Multiple games broadcasting simultaneously creates message chaos:

```
❌ CHAOS:
alice's deal...
bob's deal...
alice hits...
carol's deal...
bob stands...
alice's result...
carol hits...
bob's result...

(Total mess!)
```

### 4. **Cross-Player Interference**
What prevents Bob from sending `^HIT` to affect Alice's game?

---

## The Solution: Automatic Game Association with Clear Attribution

### Core Principles

1. **One game per player**: Each node_key can have only ONE active game
2. **Automatic association**: Commands automatically apply to YOUR game
3. **Clear attribution**: All broadcasts clearly show player name
4. **Validation**: BBS verifies sender owns the game
5. **Spectator-friendly**: Non-players can watch without interference

---

## 🎯 Solution 1: Named Attribution (RECOMMENDED)

### Implementation

Every broadcast message includes the player's name/callsign prominently:

```rust
fn format_deal_message(player_name: &str, player_hand: &[Card], dealer_up: &Card) -> String {
    format!(
        "🎰 {}'s BLACKJACK 🎰\n\
         {}: {} ({})\n\
         Dealer: {} ?\n\n\
         {} → ^HIT or ^STAND",
        player_name,
        player_name,
        format_cards(player_hand),
        hand_value(player_hand),
        format_card(dealer_up),
        player_name
    )
}
```

### User Experience

```
alice: ^BLACKJACK

🎰 alice's BLACKJACK 🎰
alice: K♠ 7♥ (17)
Dealer: 6♦ ?

alice → ^HIT or ^STAND

---

bob: ^BLACKJACK

🎰 bob's BLACKJACK 🎰
bob: 9♠ 9♥ (18)
Dealer: 7♦ ?

bob → ^HIT or ^STAND

---

alice: ^STAND

🎰 alice's result 🎰
alice: K♠ 7♥ (17)
Dealer: 6♦ K♣ 3♠ (19)
Dealer wins.

---

bob: ^STAND

🎰 bob's result 🎰
bob: 9♠ 9♥ (18)
Dealer: 7♦ J♣ (17)
>>> bob WINS! <<<
```

### Benefits
- ✅ Crystal clear who each message is for
- ✅ Spectators know what they're watching
- ✅ No command confusion
- ✅ Natural conversation flow

### Byte Cost
- Adding name: +5-15 bytes per message
- Still well under 200-byte limit

---

## 🔒 Solution 2: Game Ownership Validation

### Implementation

```rust
impl BlackjackGameManager {
    /// Start a new game - enforces one game per player
    pub fn start_game(&mut self, node_key: &str, player_name: &str) 
        -> Result<String, String> 
    {
        // Check if player already has active game
        if self.active_games.contains_key(node_key) {
            let game = self.active_games.get(node_key).unwrap();
            return Err(format!(
                "You already have a game in progress!\n\
                 Your hand: {} ({})\n\
                 Dealer: {} ?\n\
                 Reply: ^HIT or ^STAND",
                format_cards(&game.player_hand),
                hand_value(&game.player_hand),
                format_card(&game.dealer_hand[0])
            ));
        }
        
        // Create new game associated with this node_key
        let game = BlackjackGame {
            player_node: node_key.to_string(),
            player_name: player_name.to_string(),
            // ... rest of game state
        };
        
        self.active_games.insert(node_key.to_string(), game);
        Ok(format_deal_message(player_name, &player_hand, &dealer_up))
    }
    
    /// Process HIT command - validates ownership
    pub fn hit(&mut self, node_key: &str, player_name: &str) 
        -> Result<String, String> 
    {
        // Get player's game (not anyone else's)
        let game = self.active_games.get_mut(node_key)
            .ok_or("No active game. Start with ^BLACKJACK")?;
        
        // Verify ownership (should always match, but defensive)
        if game.player_node != node_key {
            return Err("This isn't your game!".into());
        }
        
        // Process hit for THIS player's game only
        // ... hit logic
        
        Ok(format_hit_message(player_name, &game.player_hand, &game.dealer_hand[0]))
    }
    
    /// Process STAND command - validates ownership
    pub fn stand(&mut self, node_key: &str, player_name: &str) 
        -> Result<String, String> 
    {
        // Remove and get player's game
        let game = self.active_games.remove(node_key)
            .ok_or("No active game. Start with ^BLACKJACK")?;
        
        // Verify ownership
        if game.player_node != node_key {
            // Put it back if wrong player
            self.active_games.insert(game.player_node.clone(), game);
            return Err("This isn't your game!".into());
        }
        
        // Process stand for THIS player's game only
        // ... stand logic
        
        Ok(format_result_message(player_name, &game))
    }
}
```

### How It Works

1. **Game starts**: `active_games.insert(alice_node_key, game)`
2. **Alice sends ^HIT**: BBS looks up `active_games[alice_node_key]`
3. **Bob sends ^HIT**: BBS looks up `active_games[bob_node_key]`
4. **Carol sends ^HIT**: BBS looks up `active_games[carol_node_key]` → None → Error or silently ignore

### Benefits
- ✅ Automatic game association by node_key
- ✅ No manual game IDs needed
- ✅ Impossible for players to interfere with each other
- ✅ Simple data structure (HashMap)

---

## 📨 Solution 3: Command Routing Strategy

### Option A: Broadcast Everything (Recommended)

**All messages broadcast publicly**, clearly attributed:

```
alice: ^BLACKJACK
→ Broadcasts "alice's BLACKJACK..."

alice: ^HIT
→ Broadcasts "alice draws..."

alice: ^STAND
→ Broadcasts "alice's result..."
```

**Pros**:
- ✅ Spectators can follow along
- ✅ Social/educational (watch others play)
- ✅ Creates shared moments
- ✅ Consistent with ^SLOT, ^8BALL pattern

**Cons**:
- ⚠️ Public channel can get busy with multiple games
- ⚠️ Messages interleave if multiple players playing

### Option B: Hybrid (DM State, Broadcast Results)

**Game state as DM**, final results broadcast:

```
alice: ^BLACKJACK
→ DM to alice: "Your hand: K♠ 7♥ (17)..."

alice: ^HIT
→ DM to alice: "Drew 3♣, now: 20..."

alice: ^STAND
→ BROADCAST: "alice wins with 20 vs dealer 19!"
```

**Pros**:
- ✅ Keeps public channel clean
- ✅ Private gameplay
- ✅ Still broadcasts notable outcomes

**Cons**:
- ❌ Loses social "watching" element
- ❌ Less exciting
- ❌ Educational value reduced

### Option C: Opt-In Quiet Mode

**Default is broadcast**, but players can opt for private:

```
alice: ^BLACKJACK        → Public game (default)
bob: ^BLACKJACK QUIET    → Private game (DM only)
```

**Pros**:
- ✅ Best of both worlds
- ✅ Player choice
- ✅ Reduces channel spam if needed

**Cons**:
- ⚠️ More complex to implement
- ⚠️ Inconsistent UX

**Recommendation**: Start with **Option A (Broadcast Everything)**, add Option C later if needed.

---

## 🎮 Concurrent Games: UI/UX Design

### Visual Differentiation

Use emojis or prefixes to distinguish active players:

```
🎰 alice's BLACKJACK 🎰
alice: K♠ 7♥ (17)
Dealer: 6♦ ?
alice → ^HIT or ^STAND

---

🎲 bob's BLACKJACK 🎲
bob: 9♠ 9♥ (18)
Dealer: 7♦ ?
bob → ^HIT or ^STAND

---

🃏 carol's BLACKJACK 🃏
carol: A♠ 6♥ (17)
Dealer: 10♦ ?
carol → ^HIT or ^STAND
```

### Alternate: Player-Specific Emojis

Assign consistent emoji per player (based on node_key hash):

```
🎰 alice's turn
🎲 bob's turn
🃏 carol's turn
♠️ dave's turn
♥️ eve's turn
```

### Message Grouping

Add separator lines between different players' games:

```
━━━━━━━━━━━━━━━━
🎰 alice's BLACKJACK
alice: K♠ 7♥ (17)
Dealer: 6♦ ?
━━━━━━━━━━━━━━━━

━━━━━━━━━━━━━━━━
🎲 bob's BLACKJACK
bob: 9♠ 9♥ (18)
Dealer: 7♦ ?
━━━━━━━━━━━━━━━━
```

---

## 🚦 Rate Limiting & Fairness

### Cooldown Strategy

**Per-player cooldown** (not global):

```rust
pub struct PublicState {
    blackjack_cooldowns: HashMap<String, Instant>,  // node_key -> last_game_start
}

impl PublicState {
    pub fn allow_blackjack_start(&mut self, node_key: &str) -> bool {
        let now = Instant::now();
        let cooldown = Duration::from_secs(60);  // 60s between games
        
        if let Some(last) = self.blackjack_cooldowns.get(node_key) {
            if now.duration_since(*last) < cooldown {
                return false;
            }
        }
        
        self.blackjack_cooldowns.insert(node_key.to_string(), now);
        true
    }
}
```

**Benefits**:
- ✅ Each player can start a game every 60 seconds
- ✅ Multiple players can play simultaneously
- ✅ Prevents single-player spam
- ✅ Fair access for everyone

### Concurrent Game Limit

Optionally limit total active games:

```rust
const MAX_CONCURRENT_GAMES: usize = 5;

pub fn start_game(&mut self, node_key: &str) -> Result<String, String> {
    if self.active_games.len() >= MAX_CONCURRENT_GAMES {
        return Err("Too many active games! Wait for one to finish.".into());
    }
    // ... proceed
}
```

**Benefits**:
- ✅ Prevents channel spam
- ✅ Keeps messages manageable
- ✅ First-come, first-served

---

## 📋 Complete Flow Examples

### Example 1: Two Concurrent Games

```
[10:00:00] alice: Hey everyone! ^BLACKJACK

[10:00:02] 🎰 alice's BLACKJACK 🎰
           alice: K♠ 7♥ (17)
           Dealer: 6♦ ?
           alice → ^HIT or ^STAND

[10:00:05] bob: I'll play too! ^BLACKJACK

[10:00:07] 🎲 bob's BLACKJACK 🎲
           bob: 9♠ 9♥ (18)
           Dealer: 7♦ ?
           bob → ^HIT or ^STAND

[10:00:15] alice: Standing on 17! ^STAND

[10:00:17] 🎰 alice's result 🎰
           alice: K♠ 7♥ (17)
           Dealer: 6♦ K♣ 4♠ (20)
           Dealer wins.

[10:00:20] alice: Darn! So close

[10:00:25] bob: I'll stand too ^STAND

[10:00:27] 🎲 bob's result 🎲
           bob: 9♠ 9♥ (18)
           Dealer: 7♦ K♣ (17)
           >>> bob WINS! <<<

[10:00:30] bob: Yes! Better luck next time alice
[10:00:32] alice: Nice win bob!
```

### Example 2: Command Validation

```
[10:00:00] alice: ^BLACKJACK

[10:00:02] 🎰 alice's BLACKJACK 🎰
           alice: K♠ 7♥ (17)
           Dealer: 6♦ ?
           alice → ^HIT or ^STAND

[10:00:05] bob: ^HIT
           (Bob has no active game - command silently ignored)

[10:00:10] alice: ^HIT

[10:00:12] 🎰 alice draws
           alice: K♠ 7♥ 3♣ (20)
           Dealer: 6♦ ?
           alice → ^HIT or ^STAND

[10:00:15] carol: ^STAND
           (Carol has no active game - command silently ignored)

[10:00:20] alice: ^STAND

[10:00:22] 🎰 alice's result 🎰
           alice: K♠ 7♥ 3♣ (20)
           Dealer: 6♦ K♣ (16) → 8♠ (24 BUST!)
           🎯 Dealer busts! alice WINS!
```

### Example 3: Existing Game Prevention

```
[10:00:00] alice: ^BLACKJACK

[10:00:02] 🎰 alice's BLACKJACK 🎰
           alice: K♠ 7♥ (17)
           Dealer: 6♦ ?
           alice → ^HIT or ^STAND

[10:00:05] alice: ^BLACKJACK
           (alice already has active game)

[10:00:06] (DM to alice)
           You already have a game in progress!
           Your hand: K♠ 7♥ (17)
           Dealer: 6♦ ?
           Reply: ^HIT or ^STAND

[10:00:10] alice: Oh right! ^STAND

[10:00:12] 🎰 alice's result 🎰
           [game completes normally]
```

---

## 🔧 Implementation Details

### Server Integration

```rust
// In src/bbs/server.rs

pub struct BBSServer {
    // ... existing fields
    blackjack_manager: BlackjackGameManager,
}

// In handle_message()
PublicCommand::Blackjack => {
    // Check per-player cooldown
    if !self.public_state.allow_blackjack_start(&node_key) {
        // Silently ignore or send DM about cooldown
        if let Err(e) = self.send_dm(&node_key, 
            "Cooldown active. Wait 60s between games.").await {
            warn!("Failed to send cooldown DM: {e:?}");
        }
        return;
    }
    
    // Get player's display name
    let player_name = self.get_player_name(&node_key);
    
    // Start game
    match self.blackjack_manager.start_game(&node_key, &player_name) {
        Ok(msg) => {
            // Broadcast game start
            #[cfg(feature = "meshtastic-proto")]
            {
                if let Err(e) = self.send_broadcast(&msg).await {
                    warn!("Blackjack deal broadcast failed: {e:?}");
                }
            }
        }
        Err(e) => {
            // Send error as DM (already has game, etc.)
            if let Err(e2) = self.send_dm(&node_key, &e).await {
                warn!("Failed to send blackjack error DM: {e2:?}");
            }
        }
    }
}

PublicCommand::BlackjackHit => {
    let player_name = self.get_player_name(&node_key);
    
    match self.blackjack_manager.hit(&node_key, &player_name) {
        Ok(msg) => {
            // Broadcast hit result
            #[cfg(feature = "meshtastic-proto")]
            {
                if let Err(e) = self.send_broadcast(&msg).await {
                    warn!("Blackjack hit broadcast failed: {e:?}");
                }
            }
        }
        Err(e) => {
            // No game, expired, etc. - send DM
            if let Err(e2) = self.send_dm(&node_key, &e).await {
                warn!("Failed to send blackjack error DM: {e2:?}");
            }
        }
    }
}

PublicCommand::BlackjackStand => {
    let player_name = self.get_player_name(&node_key);
    
    match self.blackjack_manager.stand(&node_key, &player_name) {
        Ok(msg) => {
            // Broadcast final result
            #[cfg(feature = "meshtastic-proto")]
            {
                if let Err(e) = self.send_broadcast(&msg).await {
                    warn!("Blackjack stand broadcast failed: {e:?}");
                }
            }
        }
        Err(e) => {
            // No game, expired, etc. - send DM
            if let Err(e2) = self.send_dm(&node_key, &e).await {
                warn!("Failed to send blackjack error DM: {e2:?}");
            }
        }
    }
}

impl BBSServer {
    fn get_player_name(&self, node_key: &str) -> String {
        // Try to get BBS username first
        if let Some(session) = self.sessions.get(node_key) {
            if let Some(user) = &session.user {
                return user.username.clone();
            }
        }
        
        // Fall back to node short name or ID
        self.get_node_short_name(node_key)
            .unwrap_or_else(|| format!("User_{}", &node_key[..6]))
    }
}
```

### Error Handling Strategy

**Silent Ignore vs DM Notification**:

| Error Type | Action | Reason |
|------------|--------|--------|
| No active game | DM notification | User might be confused |
| Game expired | DM notification | User should know |
| Cooldown active | DM notification | Inform about timing |
| Wrong player command | Silent ignore | Prevents spam/confusion |
| Already has game | DM notification | Remind player of game state |

---

## 📊 Byte Budget with Names

### Message Size Analysis

**Deal message with name**:
```
🎰 alice's BLACKJACK 🎰
alice: K♠ 7♥ (17)
Dealer: 6♦ ?

alice → ^HIT or ^STAND
```
**Bytes**: ~95 (up from ~80 without name) ✅

**Hit message with name**:
```
🎰 alice draws
alice: K♠ 7♥ 3♣ (20)
Dealer: 6♦ ?

alice → ^HIT or ^STAND
```
**Bytes**: ~88 ✅

**Result message with name**:
```
🎰 alice's result 🎰
alice: K♠ 7♥ (17)
Dealer: 6♦ K♣ 4♠ (20)
Dealer wins.
```
**Bytes**: ~95 ✅

**All messages still comfortably under 200 bytes!**

---

## ✅ Summary: The Complete Solution

### Identity & Attribution
✅ **Player names in every message** - No confusion about whose game it is  
✅ **Emoji/visual prefixes** - Easy to distinguish multiple games  
✅ **Clear prompt attribution** - "alice → ^HIT or ^STAND"

### Command Routing
✅ **Automatic association** - node_key maps to game  
✅ **One game per player** - Simple, enforceable rule  
✅ **Ownership validation** - Only game owner can send commands  
✅ **Silent ignore wrong commands** - Prevents confusion/spam

### Concurrency
✅ **Multiple simultaneous games** - HashMap supports N players  
✅ **Per-player cooldown** - 60s between games per player  
✅ **Optional game limit** - Cap at 5 concurrent games if needed  
✅ **Message interleaving** - Acceptable in async environment

### Error Handling
✅ **DM for user errors** - Cooldown, expired game, no active game  
✅ **Silent ignore interference** - Wrong player commands dropped  
✅ **Helpful reminders** - Show current game state in error messages

### User Experience
✅ **Spectator-friendly** - Everyone can watch all games  
✅ **Social element** - Advice, commentary, shared moments  
✅ **Educational** - Learn by watching others  
✅ **Non-intrusive** - Commands only affect your own game

---

## 🎯 Recommendation

**Implement the Named Attribution + Automatic Association approach**:

1. **Every broadcast includes player name prominently**
2. **Commands automatically apply to sender's game** (via node_key)
3. **One active game per player** (enforced by HashMap)
4. **Broadcast all game messages** (for social/spectator value)
5. **Per-player 60s cooldown** (fair access, prevents spam)

This solution is:
- ✅ Simple to implement
- ✅ Intuitive for users
- ✅ Handles concurrency naturally
- ✅ Maintains social element
- ✅ Fits byte constraints

The multi-player experience becomes:
> "Everyone can play simultaneously, you only control your own game, everyone can watch everyone else, and it's always clear whose turn it is."

Perfect for the mesh environment! 🎰
