# Combat System Implementation Plan

**Status**: Planning Phase  
**Estimated Effort**: 4-6 weeks  
**Priority**: Post-Alpha Enhancement  
**Last Updated**: 2025-10-17

## Core Principles

This combat system follows MeshBBS data-driven architecture:
- **Leverage existing systems** (NPCs, Objects, Companions, Achievements)
- **JSON-configurable** content (no hardcoded values)
- **Builder-friendly** (@NPC, @OBJECT commands for combat setup)
- **Async-first** for 500-1000 user scale
- **Zero new data structures** unless absolutely necessary

## Current State Analysis

### ✅ Existing Infrastructure (~30% Complete)

**PlayerStats (src/tmush/types.rs)**:
- HP/Max HP, MP/Max MP
- Strength, Dexterity, Intelligence, Constitution, Armor Class
- Already displayed in STATS command

**Combat-Ready Fields**:
- `player.in_combat: bool` - State flag
- `WorldConfig.err_teleport_in_combat` - Already blocks teleport

**NPC System**:
- NpcRecord with flags (Guard, Immortal, etc.)
- Can be extended with combat stats
- Dialog system for combat interactions

**Companion System**:
- `CompanionBehavior::CombatAssist { damage_bonus }` exists
- Mercenary/Construct types ready
- Auto-follow behavior implemented

**Object Triggers**:
- `ObjectTrigger::OnCombat` - Fires during combat
- `ObjectTrigger::OnHeal` - Healing items
- Trigger scripts: `heal(amount)`, `message()`, `consume()`

**Achievement System**:
- `AchievementCategory::Combat` exists
- `TriggerCondition::KillCount { count }` defined
- Ready for combat tracking

**Room Flags**:
- `RoomFlag::Safe` - No combat zones
- `RoomFlag::PvpEnabled` - PvP areas
- Already in RoomRecord

### ❌ Missing Components

1. Combat commands (ATTACK, DEFEND, FLEE, CAST)
2. Damage calculation formulas
3. Combat state machine
4. Enemy AI/behavior
5. Death/respawn mechanics
6. Loot drop system
7. Experience/leveling

---

## Phase 1: Data Structures (Week 1)

### 1.1 Extend NpcRecord for Combat

**File**: `src/tmush/types.rs`

Add to existing `NpcRecord`:
```rust
/// Combat stats (None if NPC is non-hostile)
#[serde(default)]
pub combat_stats: Option<NpcCombatStats>,
```

**New struct** (keep minimal):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcCombatStats {
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u8,
    pub defense: u8,
    pub damage_range: (u8, u8),  // (min, max)
    pub xp_reward: u32,
    pub aggression: AggressionType,
    pub loot_table: Vec<LootDrop>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AggressionType {
    Passive,      // Never attacks
    Defensive,    // Only if attacked
    Aggressive,   // Attacks on sight
    Territorial,  // Attacks if player lingers
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LootDrop {
    pub object_id: String,
    pub chance: f32,  // 0.0-1.0
    pub quantity: (u32, u32),  // (min, max)
}
```

**JSON Seed Example** (`data/seeds/npcs.json`):
```json
{
  "id": "goblin_guard",
  "name": "Goblin Guard",
  "flags": ["Guard"],
  "combat_stats": {
    "hp": 30,
    "max_hp": 30,
    "attack": 12,
    "defense": 8,
    "damage_range": [3, 8],
    "xp_reward": 50,
    "aggression": "defensive",
    "loot_table": [
      {
        "object_id": "rusty_sword",
        "chance": 0.3,
        "quantity": [1, 1]
      },
      {
        "object_id": "copper_coin",
        "chance": 0.8,
        "quantity": [5, 15]
      }
    ]
  }
}
```

### 1.2 Combat State Tracking

Add to `PlayerRecord`:
```rust
/// Active combat session
#[serde(default)]
pub combat_session: Option<CombatSession>,
```

**New struct**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombatSession {
    pub enemy_npc_id: String,
    pub enemy_current_hp: u32,
    pub started_at: DateTime<Utc>,
    pub turn_count: u32,
    pub player_can_flee: bool,  // Boss fights might lock this
}
```

### 1.3 Admin Commands Extension

**@NPC Command** - Add combat subcommands:
```
@NPC EDIT <id> COMBAT ENABLE
@NPC EDIT <id> COMBAT HP <max>
@NPC EDIT <id> COMBAT ATTACK <value>
@NPC EDIT <id> COMBAT DEFENSE <value>
@NPC EDIT <id> COMBAT DAMAGE <min> <max>
@NPC EDIT <id> COMBAT XP <reward>
@NPC EDIT <id> COMBAT AGGRESSION <passive|defensive|aggressive|territorial>
@NPC EDIT <id> COMBAT LOOT ADD <object_id> <chance> <qty_min> <qty_max>
@NPC EDIT <id> COMBAT LOOT REMOVE <object_id>
@NPC EDIT <id> COMBAT DISABLE
```

**Implementation**: Extend `handle_admin_npc()` in `src/tmush/commands.rs`

---

## Phase 2: Combat Commands (Week 2)

### 2.1 ATTACK Command

**File**: `src/tmush/commands.rs`

```rust
async fn handle_attack(&mut self, username: &str, args: &str) -> Result<String> {
    let player = self.store().get_player(username)?;
    
    // Check if already in combat
    if player.combat_session.is_some() {
        return Ok("You're already in combat! Use ATTACK to strike.".to_string());
    }
    
    // Parse target (NPC name or "npc" for only one in room)
    let target_name = if args.is_empty() {
        // Find hostile NPC in room
        self.find_hostile_npc_in_room(&player.current_room)?
    } else {
        args.trim().to_string()
    };
    
    // Find NPC
    let npc = self.find_npc_by_name(&target_name, &player.current_room)?;
    
    // Check if NPC is hostile
    let combat_stats = npc.combat_stats.as_ref()
        .ok_or("You can't attack that NPC.")?;
    
    // Check room flags
    let room = self.store().get_room(&player.current_room)?;
    if room.flags.contains(&RoomFlag::Safe) {
        return Ok("This is a safe zone. Combat is not allowed here.".to_string());
    }
    
    // Initiate combat
    self.start_combat(username, &npc).await
}
```

### 2.2 Combat Loop

```rust
async fn start_combat(&mut self, username: &str, npc: &NpcRecord) -> Result<String> {
    let mut player = self.store().get_player(username)?;
    let combat_stats = npc.combat_stats.as_ref().unwrap();
    
    // Create combat session
    let session = CombatSession {
        enemy_npc_id: npc.id.clone(),
        enemy_current_hp: combat_stats.max_hp,
        started_at: Utc::now(),
        turn_count: 0,
        player_can_flee: true,
    };
    
    player.combat_session = Some(session);
    player.in_combat = true;
    self.store().put_player(player.clone())?;
    
    // First turn
    let mut output = format!("🗡️ Combat begins with {}!\n\n", npc.name);
    output.push_str(&self.execute_combat_turn(username, "attack").await?);
    
    Ok(output)
}

async fn execute_combat_turn(&mut self, username: &str, action: &str) -> Result<String> {
    let mut player = self.store().get_player(username)?;
    let session = player.combat_session.as_mut()
        .ok_or("Not in combat")?;
    
    session.turn_count += 1;
    
    let npc = self.store().get_npc(&session.enemy_npc_id)?;
    let combat_stats = npc.combat_stats.as_ref().unwrap();
    
    let mut output = String::new();
    
    // Player turn
    match action {
        "attack" => {
            let damage = self.calculate_player_damage(&player, combat_stats);
            session.enemy_current_hp = session.enemy_current_hp.saturating_sub(damage);
            output.push_str(&format!("You strike for {} damage! ", damage));
            
            // Check companion assist
            if let Some(assist_dmg) = self.get_companion_combat_bonus(&player).await? {
                session.enemy_current_hp = session.enemy_current_hp.saturating_sub(assist_dmg);
                output.push_str(&format!("Your companion adds {} damage! ", assist_dmg));
            }
        },
        "defend" => {
            output.push_str("You take a defensive stance. ");
            // Reduce incoming damage this turn (implemented in calculate_enemy_damage)
        },
        _ => {}
    }
    
    // Check enemy death
    if session.enemy_current_hp == 0 {
        return self.end_combat_victory(username, &npc, combat_stats).await;
    }
    
    output.push_str(&format!("\n{} HP: {}/{}\n\n", npc.name, session.enemy_current_hp, combat_stats.max_hp));
    
    // Enemy turn
    let enemy_damage = self.calculate_enemy_damage(&player, combat_stats, action == "defend");
    player.stats.hp = player.stats.hp.saturating_sub(enemy_damage);
    output.push_str(&format!("💥 {} attacks for {} damage!\n", npc.name, enemy_damage));
    
    // Check player death
    if player.stats.hp == 0 {
        return self.end_combat_defeat(username).await;
    }
    
    output.push_str(&format!("Your HP: {}/{}\n\n", player.stats.hp, player.stats.max_hp));
    output.push_str("⚔️ ATTACK | 🛡️ DEFEND | 🏃 FLEE");
    
    self.store().put_player(player)?;
    
    Ok(output)
}
```

### 2.3 Damage Calculations

```rust
fn calculate_player_damage(&self, player: &PlayerRecord, enemy_stats: &NpcCombatStats) -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Base damage from strength
    let base = player.stats.strength as u32;
    
    // Random variance
    let variance = rng.gen_range(0..=5);
    
    // Defense reduction
    let defense_reduction = (enemy_stats.defense as u32) / 2;
    
    // Critical hit (10% chance)
    let crit_multiplier = if rng.gen_ratio(1, 10) { 2 } else { 1 };
    
    ((base + variance).saturating_sub(defense_reduction) * crit_multiplier).max(1)
}

fn calculate_enemy_damage(&self, player: &PlayerRecord, enemy_stats: &NpcCombatStats, defending: bool) -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let base = rng.gen_range(enemy_stats.damage_range.0..=enemy_stats.damage_range.1) as u32;
    let defense_reduction = (player.stats.armor_class as u32) / 2;
    let defend_bonus = if defending { player.stats.constitution as u32 } else { 0 };
    
    base.saturating_sub(defense_reduction + defend_bonus).max(1)
}
```

### 2.4 Other Combat Commands

```rust
// DEFEND - Reduce incoming damage
async fn handle_defend(&mut self, username: &str) -> Result<String>

// FLEE - Escape combat (50% success rate, higher with Dexterity)
async fn handle_flee(&mut self, username: &str) -> Result<String>

// CAST <spell> - Magic system (Phase 3)
async fn handle_cast(&mut self, username: &str, spell: &str) -> Result<String>
```

---

## Phase 3: Victory/Defeat/Rewards (Week 3)

### 3.1 Victory Handler

```rust
async fn end_combat_victory(&mut self, username: &str, npc: &NpcRecord, combat_stats: &NpcCombatStats) -> Result<String> {
    let mut player = self.store().get_player(username)?;
    let mut output = format!("\n🎉 Victory! You defeated {}!\n\n", npc.name);
    
    // Award XP
    player.stats.xp += combat_stats.xp_reward;
    output.push_str(&format!("✨ +{} XP\n", combat_stats.xp_reward));
    
    // Level up check (if leveling system implemented)
    if let Some(level_up_msg) = self.check_level_up(&mut player)? {
        output.push_str(&level_up_msg);
    }
    
    // Loot drops
    let loot = self.generate_loot(&combat_stats.loot_table)?;
    if !loot.is_empty() {
        output.push_str("\n💰 Loot:\n");
        for item in loot {
            player.inventory.push(item.clone());
            output.push_str(&format!("  • {}\n", item.name));
        }
    }
    
    // Clear combat state
    player.combat_session = None;
    player.in_combat = false;
    self.store().put_player(player.clone())?;
    
    // Update achievements
    self.increment_achievement_progress(username, "KILLCOUNT").await?;
    
    // Trigger OnCombat objects in inventory
    self.trigger_inventory_events(username, ObjectTrigger::OnCombat).await?;
    
    Ok(output)
}
```

### 3.2 Defeat Handler

```rust
async fn end_combat_defeat(&mut self, username: &str) -> Result<String> {
    let mut player = self.store().get_player(username)?;
    let mut output = "💀 You have been defeated!\n\n".to_string();
    
    // Death penalty configuration (from WorldConfig)
    let config = self.store().get_world_config()?;
    
    // Respawn location (default: starting area)
    let respawn_room = config.respawn_room.clone()
        .unwrap_or_else(|| "mesh_plaza".to_string());
    
    // XP penalty (configurable, default 10%)
    let xp_loss = (player.stats.xp as f32 * config.death_xp_penalty).floor() as u32;
    player.stats.xp = player.stats.xp.saturating_sub(xp_loss);
    output.push_str(&format!("📉 Lost {} XP\n", xp_loss));
    
    // Item drop (configurable)
    if config.death_drops_items {
        // Drop random item(s) in death location
        if let Some(dropped) = self.drop_random_items(&mut player, config.death_drop_count)? {
            output.push_str(&format!("\n⚠️ You dropped: {}\n", dropped));
        }
    }
    
    // Restore HP
    player.stats.hp = player.stats.max_hp;
    
    // Teleport to respawn
    player.current_room = respawn_room.clone();
    
    // Clear combat
    player.combat_session = None;
    player.in_combat = false;
    
    self.store().put_player(player)?;
    
    output.push_str(&format!("\nYou awaken in {}...\n", respawn_room));
    
    Ok(output)
}
```

### 3.3 WorldConfig Extensions

Add to `WorldConfig` in `src/tmush/types.rs`:
```rust
/// Death/respawn configuration
#[serde(default)]
pub respawn_room: Option<String>,
#[serde(default = "default_death_xp_penalty")]
pub death_xp_penalty: f32,  // 0.0-1.0
#[serde(default)]
pub death_drops_items: bool,
#[serde(default = "default_death_drop_count")]
pub death_drop_count: u32,

fn default_death_xp_penalty() -> f32 { 0.1 }
fn default_death_drop_count() -> u32 { 1 }
```

---

## Phase 4: Enemy AI & Aggression (Week 4)

### 4.1 Aggressive NPC Tick System

**File**: `src/tmush/combat.rs` (new module)

```rust
/// Check for aggressive NPCs in player's room
pub async fn check_aggressive_npcs(store: &TinyMushStore, player: &PlayerRecord) -> Result<Option<String>> {
    if player.in_combat {
        return Ok(None);  // Already fighting
    }
    
    let room = store.get_room(&player.current_room)?;
    
    // Find NPCs in room
    let npcs = store.get_npcs_in_location(&player.current_room)?;
    
    for npc in npcs {
        if let Some(combat_stats) = &npc.combat_stats {
            match combat_stats.aggression {
                AggressionType::Aggressive => {
                    // Immediate attack
                    return Ok(Some(format!(
                        "⚠️ {} attacks you!\nUse ATTACK to fight back!",
                        npc.name
                    )));
                },
                AggressionType::Territorial => {
                    // Check if player has been in room too long
                    // (requires last_room_entry timestamp on PlayerRecord)
                    // Auto-attack after 30 seconds
                },
                _ => {}
            }
        }
    }
    
    Ok(None)
}
```

**Integration**: Call from `handle_look()` and room entry events

### 4.2 Boss Fights

Add `NpcFlag::Boss`:
```rust
pub enum NpcFlag {
    TutorialNpc,
    QuestGiver,
    Vendor,
    Guard,
    Immortal,
    Boss,  // NEW: Cannot flee, special mechanics
}
```

**Boss mechanics**:
- `player_can_flee = false` in CombatSession
- Higher stats
- Special attacks (data-driven via dialog responses)
- Unique loot tables

---

## Phase 5: PvP System (Week 5)

### 5.1 PvP Consent & Zones

**Approach**: Opt-in PvP with zone-based rules

Add to `PlayerRecord`:
```rust
#[serde(default)]
pub pvp_enabled: bool,  // Opt-in flag
```

**PvP Command**:
```
PVP ON   - Enable PvP (can be attacked)
PVP OFF  - Disable PvP (safe from players)
PVP STATUS - Show current setting
```

**Room Flag Enforcement**:
```rust
fn can_attack_player(&self, attacker: &PlayerRecord, target: &PlayerRecord, room: &RoomRecord) -> Result<bool> {
    // Check PvP zone
    if !room.flags.contains(&RoomFlag::PvpEnabled) {
        return Ok(false);
    }
    
    // Check opt-in
    if !target.pvp_enabled {
        return Ok(false);
    }
    
    // Check faction (optional: same faction = no PvP)
    // if self.same_faction(attacker, target)? {
    //     return Ok(false);
    // }
    
    Ok(true)
}
```

### 5.2 PvP Combat Mechanics

**Key differences from PvE**:
- No loot drops (prevent griefing)
- Lower XP penalty on death (10% vs 25%)
- Cooldown after defeat (5 minutes)
- Flagging system (attacker gets "combatant" flag for 5 minutes)

---

## Phase 6: Integration & Polish (Week 6)

### 6.1 Companion Combat Integration

```rust
async fn get_companion_combat_bonus(&self, player: &PlayerRecord) -> Result<Option<u32>> {
    for companion_id in &player.companions {
        let companion = self.store().get_companion(companion_id)?;
        
        // Check if companion is in same room
        if companion.room_id != player.current_room {
            continue;
        }
        
        // Find CombatAssist behavior
        for behavior in &companion.behaviors {
            if let CompanionBehavior::CombatAssist { damage_bonus } = behavior {
                return Ok(Some(*damage_bonus as u32));
            }
        }
    }
    Ok(None)
}
```

### 6.2 Achievement Triggers

Extend `handle_achievement_trigger()`:
```rust
TriggerCondition::KillCount { count } => {
    // Increment on each NPC kill
    progress.increment(1);
    if progress.progress >= *count {
        self.award_achievement(username, achievement_id).await?;
    }
}
```

### 6.3 Object Trigger Events

```rust
async fn trigger_inventory_events(&self, username: &str, trigger: ObjectTrigger) -> Result<()> {
    let player = self.store().get_player(username)?;
    
    for item in &player.inventory {
        if let Some(script) = item.actions.get(&trigger) {
            self.execute_trigger_script(username, script).await?;
        }
    }
    
    Ok(())
}
```

Example: Healing potion with `OnCombat` trigger
```json
{
  "id": "combat_potion",
  "name": "Emergency Healing Potion",
  "actions": {
    "on_combat": "heal(10) && consume()"
  }
}
```

### 6.4 Help System Updates

Add combat sections:
```rust
HELP COMBAT - Combat system overview
HELP ATTACK - Attack command
HELP DEFEND - Defensive stance
HELP FLEE - Fleeing combat
HELP PVP - Player vs Player rules
```

---

## Data-Driven Content Examples

### Example 1: Friendly Training Dummy

```json
{
  "id": "training_dummy",
  "name": "Training Dummy",
  "room_id": "training_grounds",
  "flags": [],
  "combat_stats": {
    "hp": 100,
    "max_hp": 100,
    "attack": 0,
    "defense": 5,
    "damage_range": [0, 0],
    "xp_reward": 1,
    "aggression": "passive",
    "loot_table": []
  }
}
```

### Example 2: Goblin Raider

```json
{
  "id": "goblin_raider",
  "name": "Goblin Raider",
  "room_id": "dark_forest",
  "flags": ["Guard"],
  "combat_stats": {
    "hp": 45,
    "max_hp": 45,
    "attack": 15,
    "defense": 10,
    "damage_range": [5, 12],
    "xp_reward": 75,
    "aggression": "aggressive",
    "loot_table": [
      {"object_id": "goblin_sword", "chance": 0.25, "quantity": [1, 1]},
      {"object_id": "leather_armor_scraps", "chance": 0.4, "quantity": [1, 3]},
      {"object_id": "gold_coin", "chance": 0.9, "quantity": [3, 10]}
    ]
  }
}
```

### Example 3: Dragon Boss

```json
{
  "id": "ancient_dragon",
  "name": "Ancient Red Dragon",
  "room_id": "dragon_lair",
  "flags": ["Boss", "Immortal"],
  "combat_stats": {
    "hp": 500,
    "max_hp": 500,
    "attack": 30,
    "defense": 25,
    "damage_range": [20, 40],
    "xp_reward": 1000,
    "aggression": "territorial",
    "loot_table": [
      {"object_id": "dragon_scale", "chance": 1.0, "quantity": [3, 5]},
      {"object_id": "ancient_sword", "chance": 0.5, "quantity": [1, 1]},
      {"object_id": "platinum_coin", "chance": 1.0, "quantity": [50, 100]}
    ]
  }
}
```

---

## Testing Strategy

### Unit Tests

```rust
// tests/combat_basic.rs
#[tokio::test]
async fn test_initiate_combat()

#[tokio::test]
async fn test_player_victory()

#[tokio::test]
async fn test_player_defeat()

#[tokio::test]
async fn test_combat_flee()

// tests/combat_damage.rs
#[test]
fn test_damage_calculation()

#[test]
fn test_critical_hit()

#[test]
fn test_defense_reduction()

// tests/combat_npc_aggression.rs
#[tokio::test]
async fn test_aggressive_npc_attacks()

#[tokio::test]
async fn test_passive_npc_ignores()

// tests/combat_pvp.rs
#[tokio::test]
async fn test_pvp_consent_required()

#[tokio::test]
async fn test_pvp_zone_enforcement()
```

### Integration Tests

1. Full combat loop (10 turns)
2. Companion assist damage
3. Achievement unlock on kills
4. Loot drop generation
5. Boss fight mechanics
6. Death penalty application
7. PvP flagging system

---

## Configuration Files

### data/seeds/combat_npcs.json
```json
[
  // Starter enemies
  // Mid-tier threats
  // Boss encounters
]
```

### data/seeds/combat_loot.json
```json
[
  // Weapons
  // Armor
  // Consumables
  // Crafting materials
]
```

### config.example.toml
```toml
[combat]
enabled = true
death_xp_penalty = 0.1
death_drops_items = false
death_drop_count = 1
respawn_room = "mesh_plaza"
pvp_enabled = true
pvp_xp_penalty = 0.05
combat_timeout_seconds = 300
```

---

## Migration Path

### From Current State to Combat-Enabled

1. **Backwards Compatible**: All new fields use `#[serde(default)]`
2. **Existing NPCs**: Non-hostile by default (`combat_stats: None`)
3. **Opt-in**: Players must enable PvP explicitly
4. **Safe Zones**: Most rooms remain `Safe` by default
5. **Admin Control**: World builders decide where combat occurs

### Database Migration
```rust
// No migration needed - all fields optional
// Existing NPCs work as-is
// New NPCs get combat_stats via @NPC COMBAT commands
```

---

## Admin Documentation

### Creating a Combat NPC

```bash
# 1. Create NPC
@NPC CREATE forest_wolf "Grey Wolf"

# 2. Set location
@NPC EDIT forest_wolf ROOM dark_forest

# 3. Enable combat
@NPC EDIT forest_wolf COMBAT ENABLE

# 4. Set stats
@NPC EDIT forest_wolf COMBAT HP 40
@NPC EDIT forest_wolf COMBAT ATTACK 14
@NPC EDIT forest_wolf COMBAT DEFENSE 9
@NPC EDIT forest_wolf COMBAT DAMAGE 4 10
@NPC EDIT forest_wolf COMBAT XP 60
@NPC EDIT forest_wolf COMBAT AGGRESSION aggressive

# 5. Add loot
@NPC EDIT forest_wolf COMBAT LOOT ADD wolf_pelt 0.7 1 1
@NPC EDIT forest_wolf COMBAT LOOT ADD gold_coin 0.5 2 8

# 6. Save to JSON
@NPC EXPORT forest_wolf
```

---

## Performance Considerations

### Async Operations
- All combat turns via `spawn_blocking` for database writes
- Combat state cached in memory during session
- Loot generation parallelized with `join_all`

### Scaling
- Combat sessions stored per-player (no global state)
- NPC HP tracked in player's `CombatSession` (not NPC record)
- Achievement increments batched (max 1 write per combat)

### Memory
- Combat calculations stateless (no arena tracking)
- Enemy AI checks only on room entry (not per-tick)
- PvP cooldowns use timestamp comparisons (no timers)

---

## Future Enhancements (Beyond Initial Implementation)

- [ ] Combo system (sequential attacks)
- [ ] Status effects (poison, stun, slow)
- [ ] Elemental damage types
- [ ] Weapon durability
- [ ] Combat pets (separate from companions)
- [ ] Arena/tournament system
- [ ] Combat logs/replays
- [ ] Group combat (parties)
- [ ] Raid bosses (multi-player)

---

## Success Criteria

✅ **Minimum Viable Combat**:
- [ ] ATTACK command works
- [ ] NPCs can be defeated
- [ ] Player can die and respawn
- [ ] Loot drops functional
- [ ] XP rewards working
- [ ] Achievements track kills
- [ ] Safe zones enforced
- [ ] 100% test coverage

✅ **Data-Driven**:
- [ ] All NPCs configurable via JSON
- [ ] All loot tables in seed files
- [ ] All combat values tweakable via @NPC commands
- [ ] Zero hardcoded enemy stats

✅ **Performance**:
- [ ] Combat turn < 100ms
- [ ] No blocking database calls
- [ ] Scales to 1000 concurrent combats

✅ **Documentation**:
- [ ] Player-facing HELP COMBAT
- [ ] Builder guide for combat NPCs
- [ ] API docs for combat module
- [ ] Integration examples

---

**Status**: Ready for implementation  
**Dependencies**: None (all systems in place)  
**Risk Level**: Low (extends existing architecture)  
**Estimated Completion**: 6 weeks (1 week per phase)
