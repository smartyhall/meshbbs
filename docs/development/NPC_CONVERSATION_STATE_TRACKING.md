# NPC Conversation State Tracking

## Overview

NPCs track conversation state **per player** using a `ConversationState` system. This allows NPCs to remember what each individual player has discussed, received, or done, enabling personalized interactions.

## How It Works

### 1. **Storage Layer** (`src/tmush/storage.rs`)

Conversation states are stored in the `npcs` sled tree with a composite key:

```rust
Key format: "conversation:{player_id}:{npc_id}"
Example: "conversation:alice:museum_curator"
```

**Storage Methods:**
```rust
pub fn put_conversation_state(&self, state: ConversationState) -> Result<()>
pub fn get_conversation_state(&self, player_id: &str, npc_id: &str) -> Result<Option<ConversationState>>
```

### 2. **Data Structure** (`src/tmush/types.rs`)

```rust
pub struct ConversationState {
    pub npc_id: String,
    pub player_id: String,
    pub topics_discussed: Vec<String>,           // Topics talked about
    pub last_topic: Option<String>,               // Most recent topic
    pub conversation_flags: HashMap<String, bool>, // ← Key feature!
    pub last_conversation_time: DateTime<Utc>,
    pub first_conversation_time: DateTime<Utc>,
    pub total_conversations: u32,
}
```

**Key Methods:**
```rust
impl ConversationState {
    pub fn set_flag(&mut self, flag: &str, value: bool) {
        self.conversation_flags.insert(flag.to_string(), value);
    }

    pub fn get_flag(&self, flag: &str) -> bool {
        self.conversation_flags.get(flag).copied().unwrap_or(false)
    }
}
```

### 3. **Dialog Actions** (`src/tmush/commands.rs:4525`)

When a dialog node is executed, its actions run:

```rust
DialogAction::SetFlag { flag, value } => {
    // Get or create conversation state for THIS player + NPC pair
    let mut conv_state = self.store()
        .get_conversation_state(player_name, npc_id)?
        .unwrap_or_else(|| ConversationState::new(player_name, npc_id));

    // Set the flag in THIS player's conversation state
    conv_state.set_flag(flag, *value);
    
    // Save to database: conversation:alice:museum_curator
    self.store().put_conversation_state(conv_state)?;
}
```

### 4. **Dialog Conditions** (`src/tmush/commands.rs:4249`)

Before showing a dialog choice or node, conditions are checked:

```rust
DialogCondition::HasFlag { flag, value } => {
    // Get THIS player's conversation state with THIS NPC
    if let Ok(Some(conv_state)) = self.store()
        .get_conversation_state(&player.username, npc_id)
    {
        // Check if flag matches expected value
        if conv_state.get_flag(flag) != *value {
            return Ok(false); // Hide this choice/node
        }
    } else {
        return Ok(false); // No conversation state = flag not set
    }
}
```

## Example: Dr. Reeves Research Kit

### Initial State (Alice's First Visit)

```
Database key: "conversation:alice:museum_curator"
Value: (doesn't exist yet)
```

Alice sees:
```
1. "I'd be honored!" → Goes to receive_kit
2. "What's in the kit?" → Goes to kit_details
```

### After Receiving Kit

**Dialog Node Executes:**
```rust
tree.insert("receive_kit".to_string(), DialogNode::new("...")
    .with_action(DialogAction::GiveItem { ... })  // Gives 6 items
    .with_action(DialogAction::SetFlag {           // ← Sets flag!
        flag: "received_research_kit".to_string(), 
        value: true 
    })
```

**Database After:**
```
Key: "conversation:alice:museum_curator"
Value: ConversationState {
    player_id: "alice",
    npc_id: "museum_curator",
    conversation_flags: {
        "received_research_kit": true  // ← Flag is set!
    },
    ...
}
```

### Alice's Second Visit

**Condition Check:**
```rust
.with_choice(DialogChoice::new("I'd be honored!")
    .goto("receive_kit")
    .with_condition(DialogCondition::HasFlag { 
        flag: "received_research_kit".to_string(), 
        value: false  // ← Expects false
    }))
```

**Evaluation:**
1. Load: `conversation:alice:museum_curator`
2. Check: `conv_state.get_flag("received_research_kit")` → Returns `true`
3. Expected: `false`
4. Result: **Choice is hidden** ❌

Alice no longer sees "I'd be honored!" option.

### Bob's First Visit (Different Player)

```
Database key: "conversation:bob:museum_curator"
Value: (doesn't exist yet - Bob's state is separate!)
```

Bob sees:
```
1. "I'd be honored!" → Goes to receive_kit ✅
2. "What's in the kit?" → Goes to kit_details ✅
```

**Bob gets his own kit** because his conversation state is independent!

## Key Benefits

### ✅ **Per-Player State**
- Each player has their own conversation state with each NPC
- Alice receiving the kit doesn't affect Bob
- State persists between sessions

### ✅ **Flexible Flag System**
Flags can track anything:
- `"received_research_kit"` - Got the museum kit
- `"knows_secret_password"` - Learned a secret
- `"helped_with_quest"` - Completed a favor
- `"angry_at_player"` - Reputation state
- `"seen_special_cutscene"` - Story progression

### ✅ **Composable Conditions**
Multiple conditions can be combined:
```rust
.with_choice(DialogChoice::new("Special option")
    .goto("special_node")
    .with_condition(DialogCondition::HasFlag { 
        flag: "received_research_kit", 
        value: true 
    })
    .with_condition(DialogCondition::HasItem { 
        item_id: "example_ancient_key" 
    })
    .with_condition(DialogCondition::MinLevel { level: 5 }))
```

## Other Dialog Conditions

```rust
pub enum DialogCondition {
    HasDiscussed { topic: String },          // Player talked about topic before
    HasFlag { flag: String, value: bool },   // Custom flag check
    HasItem { item_id: String },             // Player has item in inventory
    HasCurrency { amount: i64 },             // Player has enough currency
    MinLevel { level: u32 },                 // Player meets level requirement
    QuestStatus { quest_id: String, status: String }, // Quest progress
    Always,                                   // Always show (default)
}
```

## Database Structure

```
sled tree: npcs
├── "npc:mayor_thompson" → NpcRecord
├── "npc:museum_curator" → NpcRecord
├── "conversation:alice:mayor_thompson" → ConversationState
├── "conversation:alice:museum_curator" → ConversationState
├── "conversation:bob:mayor_thompson" → ConversationState
└── "conversation:bob:museum_curator" → ConversationState
```

## Admin Tools

### View Conversation State
```bash
# (Would need to implement)
@CONVERSATION SHOW alice museum_curator
```

### Reset Conversation State
```bash
# (Would need to implement)
@CONVERSATION RESET alice museum_curator
@CONVERSATION RESET alice ALL
```

### Set Flag Manually
```bash
# (Would need to implement)
@CONVERSATION SETFLAG alice museum_curator received_research_kit false
```

## Implementation Flow

```
┌─────────────┐
│ Alice talks │
│  to Dr.     │
│  Reeves     │
└──────┬──────┘
       │
       ▼
┌────────────────────────────────┐
│ Load conversation state:       │
│ conversation:alice:museum...   │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│ Check conditions:              │
│ - HasFlag "received..." false? │
│   → YES, show "I'd be honored!"│
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│ Alice chooses option           │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│ Execute actions:               │
│ 1. GiveItem (6 items)          │
│ 2. SetFlag "received..." true  │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│ Save conversation state:       │
│ conversation:alice:museum...   │
│ { flags: {                     │
│   "received_research_kit": true│
│ }}                             │
└────────────────────────────────┘
```

## Best Practices

### 1. **Use Descriptive Flag Names**
```rust
// Good
"received_research_kit"
"knows_vault_location"
"completed_museum_tour"

// Bad
"flag1"
"got_stuff"
"done"
```

### 2. **Check Conditions Before Giving Items**
```rust
.with_choice(DialogChoice::new("Give me the kit!")
    .goto("receive_kit")
    .with_condition(DialogCondition::HasFlag { 
        flag: "received_research_kit", 
        value: false  // Only show if NOT received
    }))
```

### 3. **Set Flags AFTER Successful Actions**
```rust
.with_action(DialogAction::GiveItem { ... })  // Give items first
.with_action(DialogAction::SetFlag { ... })    // Then set flag
```

### 4. **Provide Alternative Dialogs**
```rust
// For players who already received kit
tree.insert("already_received".to_string(), DialogNode::new(
    "You already have your research kit!"
)
.with_condition(DialogCondition::HasFlag { 
    flag: "received_research_kit", 
    value: true 
}));
```

## Testing

```rust
#[test]
fn test_conversation_flags() {
    let store = TinyMushStore::open(temp_path)?;
    
    // Alice's first conversation
    let mut conv_state = ConversationState::new("alice", "museum_curator");
    assert_eq!(conv_state.get_flag("received_research_kit"), false);
    
    // Set flag
    conv_state.set_flag("received_research_kit", true);
    store.put_conversation_state(conv_state)?;
    
    // Load and verify
    let loaded = store.get_conversation_state("alice", "museum_curator")?.unwrap();
    assert_eq!(loaded.get_flag("received_research_kit"), true);
    
    // Bob has separate state
    let bob_state = store.get_conversation_state("bob", "museum_curator")?;
    assert!(bob_state.is_none()); // No state yet
}
```

## Summary

**Question:** How does the NPC know if a player has already received an object?

**Answer:** 

1. **Per-Player Storage**: Each player has a unique `ConversationState` with each NPC, stored as `conversation:{player}:{npc}`

2. **Flag System**: When Dr. Reeves gives items, he sets a flag: `received_research_kit = true` in **that player's** conversation state

3. **Condition Checking**: Before showing dialog options, the system checks flags in **that player's** state

4. **Independence**: Alice receiving the kit doesn't affect Bob - they have separate conversation states

**Result:** Every player can get their own copy of the research kit from Dr. Reeves, exactly once!
