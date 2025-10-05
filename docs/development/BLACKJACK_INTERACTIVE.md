# Interactive ^BLACKJACK Design (v2)

## Overview

This design makes Blackjack **interactive** while respecting mesh network constraints (high latency, 200-byte messages, broadcast nature). Players make real decisions, creating genuine gameplay tension and strategy.

**Core Insight**: In Blackjack, most of the excitement comes from the **initial decision** - hit or stand on your dealt hand. We can capture this with minimal message exchange.

---

## 🎮 Interactive Flow

### Phase 1: Deal

```
→ ^BLACKJACK

*Dealing...*

=== BLACKJACK ===
You: K♠ 7♥ (17)
Dealer: 6♦ ?

HIT or STAND?
Reply: ^HIT or ^STAND
(Expires in 5 min)
```

### Phase 2: Player Decision - HIT

```
→ ^HIT

*Drawing...*

You: K♠ 7♥ 3♣ (20)
Dealer: 6♦ ?

HIT or STAND?
```

```
→ ^STAND

*Dealer plays...*

You: K♠ 7♥ 3♣ (20)
Dealer: 6♦ K♣ (16) → 9♠ (25 BUST!)

🎯 Dealer busts!
>>> YOU WIN! <<<
```

### Phase 2: Player Decision - STAND

```
→ ^STAND

*Dealer plays...*

You: K♠ 7♥ (17)
Dealer: 6♦ K♣ (16) → 4♠ (20)

Dealer: 20, You: 17
House wins.
```

### Phase 3: Instant Outcomes

**Natural Blackjack (Player)**
```
→ ^BLACKJACK

=== BLACKJACK ===
You: A♠ K♥ (21 - BLACKJACK!)
Dealer: 10♦ 7♣ (17)

🎉 BLACKJACK! 🎉
Auto-win! (No decision needed)
```

**Natural Blackjack (Dealer)**
```
→ ^BLACKJACK

=== BLACKJACK ===
You: K♠ 9♥ (19)
Dealer: A♦ showing

*Dealer checks...*
Dealer: A♦ K♣ (21 - BLACKJACK!)

💔 Dealer blackjack!
House wins.
```

**Instant Bust (Player hits and busts)**
```
You: 10♠ 9♥ (19)
Dealer: 7♦ ?

→ ^HIT

You: 10♠ 9♥ 5♣ (24 - BUST!)

💥 BUST!
House wins.
(Game over, no dealer play needed)
```

---

## 🎯 Design Principles

### 1. **Minimize Messages**
- **Best case**: 2 messages (deal + instant resolution)
- **Average case**: 3 messages (deal + stand + result)
- **Worst case**: 5 messages (deal + hit + hit + stand + result)
- Most games resolve in 3 messages or less

### 2. **Respect Latency**
- Each decision can take 30s-5min (normal mesh delay)
- Game state persists between messages
- 5-minute timeout for decisions (auto-stand)
- Player can resume from any device

### 3. **Maintain Excitement**
- **Tension**: Hidden dealer card creates suspense
- **Strategy**: Real decisions matter (hit on 16 vs dealer 10?)
- **Variety**: Every hand plays differently
- **Agency**: Player controls outcome

### 4. **Broadcast-Friendly**
- All state fits in each message
- Other players can watch games in progress
- Spectators see the drama unfold
- Social element (advice, cheering, groaning)

---

## 📱 Message Formats

### Initial Deal Message

**Standard Format**
```
=== BLACKJACK ===
You: K♠ 7♥ (17)
Dealer: 6♦ ?

HIT or STAND?
Reply: ^HIT or ^STAND
Expires: 5 min
(~88 bytes)
```

**Compact Format**
```
BLACKJACK
You: K♠7♥ (17)
Dealer: 6♦ ?
^HIT or ^STAND? (5m)
(~58 bytes)
```

### Decision Response Messages

**Hit (with new card)**
```
*Drawing...*

You: K♠ 7♥ 3♣ (20)
Dealer: 6♦ ?

HIT or STAND?
(~62 bytes)
```

**Stand (dealer plays)**
```
*Dealer plays...*

You: K♠ 7♥ (17)
Dealer: 6♦ K♣ 4♠ (20)

Dealer wins.
(~65 bytes)
```

**Bust (instant loss)**
```
You: 10♠ 9♥ 5♣ (24)
💥 BUST! House wins.
(~35 bytes)
```

### Final Result Messages

**Player Wins**
```
Final:
You: 20
Dealer: 19

>>> YOU WIN! <<<
(~42 bytes)
```

**Player Blackjack**
```
You: A♠ K♥ (BLACKJACK!)
Dealer: 10♦ 7♣ (17)

🎉 BLACKJACK! 🎉
(~65 bytes)
```

**Push (Tie)**
```
Final:
You: 18
Dealer: 18

🤝 PUSH (Tie)
(~38 bytes)
```

---

## 🔧 Technical Implementation

### Game State Structure

```rust
pub struct BlackjackGame {
    player_node: String,
    player_hand: Vec<Card>,
    dealer_hand: Vec<Card>,
    dealer_hole_card: Card,  // Hidden until dealer plays
    status: GameStatus,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    message_count: u32,
}

pub enum GameStatus {
    AwaitingDecision,  // Player must HIT or STAND
    PlayerBust,        // Player busted, game over
    DealerBust,        // Dealer busted, player wins
    PlayerBlackjack,   // Player natural 21
    DealerBlackjack,   // Dealer natural 21
    PlayerWin,         // Player score > dealer
    DealerWin,         // Dealer score > player
    Push,              // Tie
}
```

### Command Flow

```rust
// In src/bbs/public.rs
pub enum PublicCommand {
    // ... existing commands
    Blackjack,           // Start new game: ^BLACKJACK
    BlackjackHit,        // Hit: ^HIT
    BlackjackStand,      // Stand: ^STAND
    BlackjackDouble,     // Double down: ^DOUBLE (future)
    BlackjackSplit,      // Split pairs: ^SPLIT (future)
}

// Parser additions
if cmd == "blackjack" || cmd == "bj" {
    return PublicCommand::Blackjack;
}
if cmd == "hit" || cmd == "h" {
    return PublicCommand::BlackjackHit;
}
if cmd == "stand" || cmd == "s" {
    return PublicCommand::BlackjackStand;
}
```

### Game State Management

```rust
// In src/bbs/server.rs or dedicated blackjack module

pub struct BlackjackGameManager {
    active_games: HashMap<String, BlackjackGame>,  // node_key -> game
}

impl BlackjackGameManager {
    /// Start a new game for a player
    pub fn start_game(&mut self, node_key: &str) -> Result<String, String> {
        // Check if player already has active game
        if self.active_games.contains_key(node_key) {
            return Err("You already have a game in progress! ^HIT or ^STAND".into());
        }
        
        // Deal cards
        let mut deck = create_deck();
        deck.shuffle(&mut rand::thread_rng());
        
        let player_hand = vec![deck.pop().unwrap(), deck.pop().unwrap()];
        let dealer_up = deck.pop().unwrap();
        let dealer_hole = deck.pop().unwrap();
        
        let game = BlackjackGame {
            player_node: node_key.to_string(),
            player_hand: player_hand.clone(),
            dealer_hand: vec![dealer_up],
            dealer_hole_card: dealer_hole,
            status: GameStatus::AwaitingDecision,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(5),
            message_count: 1,
        };
        
        // Check for instant blackjack
        if hand_value(&player_hand) == 21 {
            game.status = GameStatus::PlayerBlackjack;
            let msg = format_blackjack_result(&game);
            self.active_games.remove(node_key);
            return Ok(msg);
        }
        
        if hand_value(&[dealer_up, dealer_hole]) == 21 {
            game.status = GameStatus::DealerBlackjack;
            let msg = format_blackjack_result(&game);
            self.active_games.remove(node_key);
            return Ok(msg);
        }
        
        self.active_games.insert(node_key.to_string(), game);
        Ok(format_deal_message(&player_hand, &dealer_up))
    }
    
    /// Player hits
    pub fn hit(&mut self, node_key: &str) -> Result<String, String> {
        let game = self.active_games.get_mut(node_key)
            .ok_or("No active game. Start with ^BLACKJACK")?;
        
        // Check expiration
        if Utc::now() > game.expires_at {
            self.active_games.remove(node_key);
            return Err("Game expired. Start a new game with ^BLACKJACK".into());
        }
        
        // Draw card
        let mut deck = create_deck();
        deck.shuffle(&mut rand::thread_rng());
        let new_card = deck.pop().unwrap();
        game.player_hand.push(new_card);
        game.message_count += 1;
        
        let player_total = hand_value(&game.player_hand);
        
        // Check for bust
        if player_total > 21 {
            game.status = GameStatus::PlayerBust;
            let msg = format_bust_message(&game.player_hand);
            self.active_games.remove(node_key);
            return Ok(msg);
        }
        
        // Check for 21 (auto-stand)
        if player_total == 21 {
            return self.stand_internal(game);
        }
        
        // Still playing
        Ok(format_hit_message(&game.player_hand, &game.dealer_hand[0]))
    }
    
    /// Player stands
    pub fn stand(&mut self, node_key: &str) -> Result<String, String> {
        let game = self.active_games.remove(node_key)
            .ok_or("No active game. Start with ^BLACKJACK")?;
        
        // Check expiration
        if Utc::now() > game.expires_at {
            return Err("Game expired. Start a new game with ^BLACKJACK".into());
        }
        
        self.stand_internal_owned(game)
    }
    
    fn stand_internal_owned(&mut self, mut game: BlackjackGame) -> Result<String, String> {
        // Reveal hole card
        game.dealer_hand.push(game.dealer_hole_card);
        
        // Dealer plays (hit on 16, stand on 17)
        let mut deck = create_deck();
        deck.shuffle(&mut rand::thread_rng());
        
        while hand_value(&game.dealer_hand) < 17 {
            game.dealer_hand.push(deck.pop().unwrap());
        }
        
        let player_total = hand_value(&game.player_hand);
        let dealer_total = hand_value(&game.dealer_hand);
        
        // Determine winner
        game.status = if dealer_total > 21 {
            GameStatus::DealerBust
        } else if player_total > dealer_total {
            GameStatus::PlayerWin
        } else if dealer_total > player_total {
            GameStatus::DealerWin
        } else {
            GameStatus::Push
        };
        
        Ok(format_final_result(&game))
    }
    
    /// Clean up expired games
    pub fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.active_games.retain(|_, game| game.expires_at > now);
    }
}
```

### Message Formatters

```rust
fn format_deal_message(player_hand: &[Card], dealer_up: &Card) -> String {
    let player_str = format_cards(player_hand);
    let player_val = hand_value(player_hand);
    let dealer_str = format_card(dealer_up);
    
    format!(
        "=== BLACKJACK ===\n\
         You: {} ({})\n\
         Dealer: {} ?\n\n\
         HIT or STAND?\n\
         Reply: ^HIT or ^STAND\n\
         Expires: 5 min",
        player_str, player_val, dealer_str
    )
}

fn format_hit_message(player_hand: &[Card], dealer_up: &Card) -> String {
    let player_str = format_cards(player_hand);
    let player_val = hand_value(player_hand);
    let dealer_str = format_card(dealer_up);
    
    format!(
        "*Drawing...*\n\n\
         You: {} ({})\n\
         Dealer: {} ?\n\n\
         HIT or STAND?",
        player_str, player_val, dealer_str
    )
}

fn format_bust_message(player_hand: &[Card]) -> String {
    let player_str = format_cards(player_hand);
    let player_val = hand_value(player_hand);
    
    format!(
        "You: {} ({})\n\
         💥 BUST! House wins.",
        player_str, player_val
    )
}

fn format_final_result(game: &BlackjackGame) -> String {
    let player_str = format_cards(&game.player_hand);
    let dealer_str = format_cards(&game.dealer_hand);
    let player_val = hand_value(&game.player_hand);
    let dealer_val = hand_value(&game.dealer_hand);
    
    let outcome = match game.status {
        GameStatus::PlayerWin => ">>> YOU WIN! <<<",
        GameStatus::DealerWin => "House wins.",
        GameStatus::DealerBust => "🎯 Dealer busts! YOU WIN!",
        GameStatus::Push => "🤝 PUSH (Tie)",
        GameStatus::PlayerBlackjack => "🎉 BLACKJACK! 🎉",
        GameStatus::DealerBlackjack => "💔 Dealer blackjack!",
        _ => "",
    };
    
    format!(
        "*Dealer plays...*\n\n\
         You: {} ({})\n\
         Dealer: {} ({})\n\n\
         {}",
        player_str, player_val, dealer_str, dealer_val, outcome
    )
}
```

### Server Integration

```rust
// In src/bbs/server.rs handle_message()

PublicCommand::Blackjack => {
    // Check cooldown (30s between new games)
    if !self.public_state.allow_blackjack(&node_key) {
        // Silently ignore or send DM about cooldown
        return;
    }
    
    match self.blackjack_manager.start_game(&node_key) {
        Ok(msg) => {
            #[cfg(feature = "meshtastic-proto")]
            {
                if let Err(e) = self.send_broadcast(&msg).await {
                    warn!("Blackjack deal broadcast failed: {e:?}");
                }
            }
        }
        Err(e) => {
            // Send error as DM
            if let Err(e2) = self.send_dm(&node_key, &e).await {
                warn!("Failed to send blackjack error DM: {e2:?}");
            }
        }
    }
}

PublicCommand::BlackjackHit => {
    match self.blackjack_manager.hit(&node_key) {
        Ok(msg) => {
            #[cfg(feature = "meshtastic-proto")]
            {
                if let Err(e) = self.send_broadcast(&msg).await {
                    warn!("Blackjack hit broadcast failed: {e:?}");
                }
            }
        }
        Err(e) => {
            if let Err(e2) = self.send_dm(&node_key, &e).await {
                warn!("Failed to send blackjack error DM: {e2:?}");
            }
        }
    }
}

PublicCommand::BlackjackStand => {
    match self.blackjack_manager.stand(&node_key) {
        Ok(msg) => {
            #[cfg(feature = "meshtastic-proto")]
            {
                if let Err(e) = self.send_broadcast(&msg).await {
                    warn!("Blackjack stand broadcast failed: {e:?}");
                }
            }
        }
        Err(e) => {
            if let Err(e2) = self.send_dm(&node_key, &e).await {
                warn!("Failed to send blackjack error DM: {e2:?}");
            }
        }
    }
}
```

---

## 🎯 Strategy & Decision Making

### When to HIT

**Strong Hit Situations** (player should almost always hit):
- **5-8**: Always hit (can't bust)
- **9-11**: Hit (maybe double if implemented)
- **12 vs dealer 2-3**: Hit (dealer likely to bust)
- **12-16 vs dealer 7-A**: Hit (dealer strong)

### When to STAND

**Strong Stand Situations**:
- **17+**: Always stand (too risky)
- **12-16 vs dealer 2-6**: Stand (dealer likely to bust)
- **Soft 18+ (A+7)**: Stand (good hand)

### Advanced Strategy (Future)

**Double Down** (double bet, get exactly 1 card):
- 11 vs dealer 2-10
- 10 vs dealer 2-9
- 9 vs dealer 3-6

**Split Pairs** (if you have two of same rank):
- Always split Aces and 8s
- Never split 10s or 5s
- Split 2s, 3s, 6s, 7s, 9s situationally

---

## 🎮 Example Game Sessions

### Session 1: Conservative Play

```
alice: ^BLACKJACK

=== BLACKJACK ===
You: 10♠ 6♥ (16)
Dealer: K♦ ?

HIT or STAND?
Reply: ^HIT or ^STAND
Expires: 5 min

bob: Ooh tough hand Alice!
carol: Hit! Dealer has a face card

alice: ^HIT

*Drawing...*

You: 10♠ 6♥ 4♣ (20)
Dealer: K♦ ?

HIT or STAND?

alice: Nice! ^STAND

*Dealer plays...*

You: 10♠ 6♥ 4♣ (20)
Dealer: K♦ 5♣ 6♠ (21)

Dealer: 21, You: 20
House wins.

alice: Nooo so close!
bob: Brutal beat
```

### Session 2: Risky Play Pays Off

```
dave: Feeling lucky! ^BLACKJACK

=== BLACKJACK ===
You: 7♠ 8♥ (15)
Dealer: 6♦ ?

HIT or STAND?

eve: Stand! Dealer will bust with that 6
dave: Nah, living dangerously! ^HIT

*Drawing...*

You: 7♠ 8♥ 6♣ (21)
Dealer: 6♦ ?

HIT or STAND?

dave: Perfect! ^STAND

*Dealer plays...*

You: 7♠ 8♥ 6♣ (21)
Dealer: 6♦ 10♣ 8♠ (24 - BUST!)

🎯 Dealer busts!
>>> YOU WIN! <<<

dave: Told you! 🎉
eve: Lucky lucky!
```

### Session 3: Natural Blackjack

```
frank: ^BLACKJACK

=== BLACKJACK ===
You: A♠ K♥ (21 - BLACKJACK!)
Dealer: 7♦ 5♣ (12)

🎉 BLACKJACK! 🎉
Auto-win! (No decision needed)

frank: BOOM! Natural 21!
grace: That's what I'm talking about!
```

### Session 4: Multiple Hits

```
grace: My turn! ^BLACKJACK

=== BLACKJACK ===
You: 3♠ 2♥ (5)
Dealer: 9♦ ?

HIT or STAND?

grace: Obviously hitting ^HIT

*Drawing...*

You: 3♠ 2♥ 7♣ (12)
Dealer: 9♦ ?

HIT or STAND?

grace: Again! ^HIT

*Drawing...*

You: 3♠ 2♥ 7♣ 4♦ (16)
Dealer: 9♦ ?

HIT or STAND?

grace: Dealer has 9... risky but ^HIT

*Drawing...*

You: 3♠ 2♥ 7♣ 4♦ 9♠ (25 - BUST!)

💥 BUST! House wins.

grace: Knew I shouldn't have gone for that last one 😅
henry: The greed got you!
```

---

## 📊 Message Count Analysis

### Typical Game Scenarios

| Scenario | Messages | Example |
|----------|----------|---------|
| Natural Blackjack | 1 | Deal → Instant win |
| Bust on first hit | 2 | Deal → Hit → Bust |
| Stand immediately | 2 | Deal → Stand → Result |
| Hit once then stand | 3 | Deal → Hit → Stand → Result |
| Hit twice then stand | 4 | Deal → Hit → Hit → Stand → Result |
| Hit three times | 5 | Deal → Hit → Hit → Hit → Stand/Bust |

**Average**: ~2.5 messages per game

**Bandwidth**: 
- Average message: ~70 bytes
- Average game: 175 bytes total
- Very mesh-friendly!

---

## 🎯 Why This Design Works

### 1. **Preserves Excitement**
- ✅ Real decisions matter
- ✅ Hidden dealer card creates tension
- ✅ Risk/reward on every hit
- ✅ Strategy discussions between players

### 2. **Respects Mesh Constraints**
- ✅ Low message count (avg 2-3 per game)
- ✅ All messages <200 bytes
- ✅ Works with high latency (30s-5min per decision)
- ✅ 5-minute timeout prevents hanging games

### 3. **Social & Observable**
- ✅ Other players can watch games
- ✅ Spectators can give advice/commentary
- ✅ Creates shared moments (big wins, bad beats)
- ✅ Learning tool (see how others play)

### 4. **Technical Simplicity**
- ✅ Game state fits in memory
- ✅ Auto-cleanup of expired games
- ✅ No persistent storage required
- ✅ Simple command set (3 commands)

### 5. **Progressive Enhancement**
- ✅ Start with HIT/STAND
- ✅ Add DOUBLE DOWN later
- ✅ Add SPLIT later
- ✅ Add statistics/leaderboard later

---

## 🔮 Future Enhancements

### Phase 1: Core Gameplay (MVP)
- [x] Deal initial hand
- [x] HIT command
- [x] STAND command
- [x] Dealer AI (hit 16, stand 17)
- [x] Winner determination
- [x] 5-minute timeout
- [x] One active game per player

### Phase 2: Advanced Moves
- [ ] **DOUBLE DOWN**: Double your bet, get exactly one more card
  - Available only on first decision
  - Shows as ^DOUBLE command
  - Requires coin/bet system

- [ ] **SPLIT PAIRS**: Split matching cards into two hands
  - Available when dealt pair (7♠ 7♥)
  - Play two separate hands
  - More complex state management

- [ ] **INSURANCE**: Side bet when dealer shows Ace
  - Pays 2:1 if dealer has blackjack
  - Optional safety net

### Phase 3: Statistics & Tracking
- [ ] Track wins/losses per player
- [ ] ^BJSTATS command
- [ ] Leaderboards (win rate, blackjacks, biggest streaks)
- [ ] Achievements (10 wins in a row, etc.)

### Phase 4: Advanced Features
- [ ] Betting with coins (integrate with ^SLOT economy)
- [ ] Side bets (21+3, Perfect Pairs)
- [ ] Multiple deck shoes (reduces card counting)
- [ ] Tournament mode
- [ ] Private challenges (player vs player)

---

## 🎲 Comparison: Automated vs Interactive

| Aspect | Automated | Interactive |
|--------|-----------|-------------|
| **Messages per game** | 1 | 2-5 |
| **Player decisions** | 0 | 1-4 |
| **Excitement** | Low | High |
| **Strategy** | None | Critical |
| **Social element** | Announcement | Drama + discussion |
| **Latency impact** | None | Manageable |
| **Complexity** | Very low | Low-medium |
| **Replayability** | Low | High |
| **Learning curve** | None | Minimal |
| **Fun factor** | 5/10 | 9/10 |

**Verdict**: Interactive version is significantly more engaging with only moderate complexity increase.

---

## ✅ Implementation Checklist

### Core Game Logic
- [ ] Create `src/bbs/blackjack.rs` module
- [ ] Implement `Card`, `Rank`, `Suit` types
- [ ] Implement deck creation and shuffling
- [ ] Implement hand value calculation (with Aces as 1 or 11)
- [ ] Implement `BlackjackGame` state struct
- [ ] Implement `BlackjackGameManager` with active game tracking

### Commands & Parsing
- [ ] Add `Blackjack`, `BlackjackHit`, `BlackjackStand` to `PublicCommand`
- [ ] Update parser to recognize ^BLACKJACK, ^HIT, ^STAND
- [ ] Add short aliases (^BJ, ^H, ^S)

### Game Flow
- [ ] Implement `start_game()` - deal initial hand
- [ ] Implement `hit()` - draw card, check bust
- [ ] Implement `stand()` - dealer plays, determine winner
- [ ] Implement timeout/expiration logic
- [ ] Implement one-game-per-player enforcement

### Message Formatting
- [ ] Format deal message (player hand, dealer up card)
- [ ] Format hit message (updated hand)
- [ ] Format stand/result message (final hands, winner)
- [ ] Format instant outcomes (blackjack, bust)
- [ ] Verify all messages <200 bytes

### Server Integration
- [ ] Add `BlackjackGameManager` to `BBSServer`
- [ ] Handle `PublicCommand::Blackjack` in message loop
- [ ] Handle `PublicCommand::BlackjackHit` in message loop
- [ ] Handle `PublicCommand::BlackjackStand` in message loop
- [ ] Implement error handling (no active game, expired game)
- [ ] Add periodic cleanup of expired games

### Testing
- [ ] Unit tests for hand value calculation
- [ ] Unit tests for winner determination
- [ ] Unit tests for dealer AI (hit 16, stand 17)
- [ ] Integration test for full game flow
- [ ] Test timeout/expiration behavior
- [ ] Test one-game-per-player enforcement
- [ ] Test message byte sizes

### Documentation
- [ ] Update README with ^BLACKJACK commands
- [ ] Update user guide with gameplay instructions
- [ ] Add strategy tips document
- [ ] Update public commands list

### Deployment
- [ ] Test in dev environment
- [ ] Test on actual mesh network
- [ ] Verify latency handling (30s-5min delays)
- [ ] Monitor for edge cases
- [ ] Gather user feedback

---

## 🎉 Summary

**Interactive ^BLACKJACK** transforms the concept from a simple random outcome generator into a genuine game:

✅ **Strategic**: Real decisions matter  
✅ **Exciting**: Hidden cards create suspense  
✅ **Social**: Others can watch and comment  
✅ **Mesh-friendly**: 2-5 messages per game  
✅ **Scalable**: Easy to add features later  
✅ **Fun**: Genuine Blackjack experience  

The key insight is that **one interactive decision** (hit or stand on initial hand) captures 90% of the Blackjack excitement while only adding 1-2 extra messages per game. The trade-off is absolutely worth it.

**Recommendation**: Implement interactive version as the primary design, with automated mode available as a fallback if bandwidth becomes critical.
