# NPC Dialogue System - Complete Design Specification

**Status**: 📋 Planned  
**Priority**: High (Alpha Enhancement)  
**Complexity**: Medium-High  

## Overview

A comprehensive dialogue system for NPCs that supports multi-topic conversations, branching trees, conditional responses, and integration with quests, shops, and player state.

## Phased Implementation

### Phase 1: Multi-Topic Dialogue (Quick Win)
**Effort**: 1-2 hours  
**Value**: High  

Allow accessing specific dialog keys directly:
```
TALK <npc> <topic>
```

**Examples**:
- `TALK MIRA` → greeting (default)
- `TALK MIRA WARES` → wares dialog
- `TALK MIRA STORY` → story dialog
- `TALK GUARD WARNING` → warning dialog

**Implementation**:
- Parse optional topic from TALK command
- Look up `dialog[topic]` or fall back to greeting
- Return dialog text or list available topics

### Phase 2: Conversation State & History
**Effort**: 4-6 hours  
**Value**: Medium  

Track conversation state per player:
- What topics have been discussed
- Conversation progress
- Unlocked dialogue options

**Data Structure**:
```rust
pub struct ConversationState {
    npc_id: String,
    player_id: String,
    topics_discussed: HashSet<String>,
    last_topic: Option<String>,
    conversation_flags: HashMap<String, bool>,
    last_conversation_time: DateTime<Utc>,
}
```

### Phase 3: Branching Dialogue Trees
**Effort**: 8-12 hours  
**Value**: High  

Full dialogue tree support with choices:

```json
{
  "greeting": {
    "text": "Welcome! What can I do for you?",
    "choices": [
      {
        "label": "Tell me about your wares",
        "goto": "wares"
      },
      {
        "label": "What's your story?",
        "goto": "story"
      },
      {
        "label": "Goodbye",
        "exit": true
      }
    ]
  },
  "wares": {
    "text": "I have fine goods...",
    "choices": [
      {
        "label": "Tell me more",
        "goto": "wares_detail"
      },
      {
        "label": "Back",
        "goto": "greeting"
      }
    ]
  }
}
```

### Phase 4: Conditional Responses
**Effort**: 6-8 hours  
**Value**: High  

Dialog varies based on conditions:

```json
{
  "greeting": {
    "conditions": [
      {
        "if": "quest:welcome_towne:completed",
        "text": "Welcome back, hero!"
      },
      {
        "if": "item:town_map:owned",
        "text": "I see you got the map!"
      },
      {
        "default": true,
        "text": "Hello, stranger."
      }
    ]
  }
}
```

**Condition Types**:
- `quest:<id>:<status>` - Quest progress
- `item:<id>:owned` - Inventory check
- `achievement:<id>:unlocked` - Achievement status
- `flag:<name>:true/false` - Custom flags
- `currency:>=<amount>` - Money check
- `time:<hour>` - Time of day
- `talked:<npc>:<topic>` - Conversation history

### Phase 5: Dialog Scripting & Actions
**Effort**: 8-10 hours  
**Value**: Medium  

Dialog can trigger actions:

```json
{
  "accept_quest": {
    "text": "I need your help!",
    "actions": [
      {
        "type": "give_quest",
        "quest_id": "help_vendor"
      },
      {
        "type": "set_flag",
        "flag": "vendor_quest_offered",
        "value": true
      }
    ],
    "choices": [...]
  }
}
```

**Action Types**:
- `give_quest` - Start quest
- `complete_quest` - Mark quest complete
- `give_item` - Add to inventory
- `take_item` - Remove from inventory
- `give_currency` - Add money
- `take_currency` - Remove money
- `set_flag` - Set conversation flag
- `teleport` - Move player
- `spawn_npc` - Create NPC
- `trigger_event` - Custom event

### Phase 6: Admin Dialog Editor
**Effort**: 10-12 hours  
**Value**: Medium  

In-game dialogue editor with commands:
- `@DIALOG <npc> LIST` - Show all topics
- `@DIALOG <npc> VIEW <topic>` - View topic details
- `@DIALOG <npc> ADD <topic> <text>` - Add simple dialog
- `@DIALOG <npc> EDIT <topic>` - Edit with full JSON
- `@DIALOG <npc> DELETE <topic>` - Remove topic
- `@DIALOG <npc> TEST <topic>` - Test conditions

## Technical Specifications

### Data Structures

```rust
// Dialog node in a conversation tree
pub struct DialogNode {
    pub text: String,
    pub conditions: Vec<DialogCondition>,
    pub choices: Vec<DialogChoice>,
    pub actions: Vec<DialogAction>,
}

// Choice presented to player
pub struct DialogChoice {
    pub label: String,
    pub goto: Option<String>,
    pub conditions: Vec<DialogCondition>,
    pub exit: bool,
}

// Condition for dialog/choice availability
pub enum DialogCondition {
    QuestStatus { quest_id: String, status: QuestStatus },
    ItemOwned { item_id: String },
    AchievementUnlocked { achievement_id: String },
    FlagSet { flag: String, value: bool },
    CurrencyAmount { operator: CompareOp, amount: i64 },
    TimeOfDay { hour: u8 },
    PreviouslyTalked { npc_id: String, topic: String },
    Custom(String), // For advanced scripting
}

// Action triggered by dialog
pub enum DialogAction {
    GiveQuest(String),
    CompleteQuest(String),
    GiveItem { item_id: String, quantity: u32 },
    TakeItem { item_id: String, quantity: u32 },
    GiveCurrency(CurrencyAmount),
    TakeCurrency(CurrencyAmount),
    SetFlag { flag: String, value: bool },
    Teleport(String),
    TriggerEvent(String),
}
```

### Storage Schema

```
dialog:{npc_id}:{topic} -> DialogNode (JSON)
conversation_state:{player_id}:{npc_id} -> ConversationState
```

### Command Syntax

```
TALK <npc>              # Start conversation (greeting)
TALK <npc> <topic>      # Jump to specific topic
TALK <npc> <number>     # Select choice by number (in conversation)
```

## Integration Points

### Quest System
- NPCs give/complete quests via dialog actions
- Quest progress affects available dialog
- Quest objectives can include "Talk to NPC"

### Shop System
- Vendors have "browse wares" dialog option
- Dialog can transition to shop interface
- Purchase history affects dialog

### Tutorial System
- Tutorial NPCs use conditional dialog
- Progress unlocks new topics
- Completion triggers rewards via actions

### Achievement System
- Unlocked achievements change dialog
- Dialog can grant achievements
- Achievement titles affect NPC reactions

## Performance Considerations

- Dialog trees cached in memory (LRU)
- Condition evaluation optimized
- Conversation state indexed by player
- Max dialog tree depth: 10 levels
- Max choices per node: 6

## Testing Strategy

- Unit tests for condition evaluation
- Integration tests for dialog flow
- Test NPCs with complex trees
- Stress test with 100+ concurrent conversations
- Validation of JSON dialog format

## Documentation

- Player guide: How to talk to NPCs
- Builder guide: Creating dialog trees
- Admin guide: Using @DIALOG commands
- Developer guide: Adding new condition types

## Timeline Estimate

- Phase 1 (Multi-topic): 1-2 hours
- Phase 2 (State): 4-6 hours
- Phase 3 (Trees): 8-12 hours
- Phase 4 (Conditions): 6-8 hours
- Phase 5 (Actions): 8-10 hours
- Phase 6 (Editor): 10-12 hours

**Total**: 37-50 hours (5-7 days of focused work)

## Success Metrics

- NPCs feel interactive and engaging
- Dialog trees provide player choice
- Conditions create dynamic conversations
- No performance degradation
- Builders can create complex dialogs
- Players understand conversation system

---

**Next Steps**: Start with Phase 1 (multi-topic) as it provides immediate value with minimal effort.
