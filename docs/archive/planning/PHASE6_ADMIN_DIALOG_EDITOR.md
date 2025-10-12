# Phase 6 Complete: Admin Dialog Editor

**Date**: October 9, 2025  
**Status**: ✅ Deployed to Production  
**Effort**: 2.5 hours  

## Overview

Complete admin toolset for creating and managing NPC dialogue trees without manually editing JSON files.

## Commands Implemented

### @DIALOG NPC LIST
Shows all dialogue topics for an NPC.

**Usage**: `@DIALOG <npc> LIST`

**Example**:
```
> @DIALOG merchant LIST

=== Dialogue Topics for Merchant ===

Simple Dialogue:
  greeting - Welcome to my shop! How can I help...
  farewell - Thank you for visiting!

Dialogue Trees:
  trade_quest - 3 choices, 2 actions, 1 conditions
  special_offer - 5 choices, 1 actions, 0 conditions

Commands:
  @DIALOG <npc> VIEW <topic> - View dialogue details
  @DIALOG <npc> ADD <topic> <text> - Add simple dialogue
  @DIALOG <npc> EDIT <topic> <json> - Edit dialogue tree
  @DIALOG <npc> DELETE <topic> - Remove dialogue
  @DIALOG <npc> TEST <topic> - Test dialogue conditions
```

### @DIALOG NPC VIEW TOPIC
View full JSON of a dialogue topic.

**Usage**: `@DIALOG <npc> VIEW <topic>`

**Example**:
```
> @DIALOG merchant VIEW trade_quest

=== Dialogue: Merchant - trade_quest ===

Type: Dialogue Tree

JSON Definition:
{
  "text": "I need rare gems. Can you help?",
  "actions": [
    {
      "type": "start_quest",
      "quest_id": "gem_collection"
    }
  ],
  "choices": [
    {
      "label": "I'll help you",
      "goto": "accept_quest"
    },
    {
      "label": "Not interested",
      "exit": true
    }
  ]
}
```

### @DIALOG NPC ADD TOPIC TEXT
Add simple text-only dialogue.

**Usage**: `@DIALOG <npc> ADD <topic> <text>`

**Example**:
```
> @DIALOG merchant ADD busy Sorry, I'm busy right now!

Added simple dialogue for Merchant by alice.
Topic: busy
Text: Sorry, I'm busy right now!
```

**Validations**:
- Max 500 characters
- Topic must not already exist
- Text required

### @DIALOG NPC EDIT TOPIC JSON
Edit complex dialogue tree with JSON.

**Usage**: `@DIALOG <npc> EDIT <topic> <json>`

**Example**:
```
> @DIALOG merchant EDIT quest_reward {"text":"Here's your reward!","actions":[{"type":"give_currency","amount":500},{"type":"complete_quest","quest_id":"gem_collection"}],"choices":[{"label":"Thank you!","exit":true}]}

Updated dialogue tree for Merchant by alice.
Topic: quest_reward

Use @DIALOG merchant VIEW quest_reward to see the result.
```

**Features**:
- Full JSON validation
- Helpful error messages
- Moves from simple to tree dialogue automatically
- Supports all DialogNode features (actions, conditions, choices)

### @DIALOG NPC DELETE TOPIC
Remove a dialogue topic.

**Usage**: `@DIALOG <npc> DELETE <topic>`

**Example**:
```
> @DIALOG merchant DELETE old_topic

Deleted dialogue tree 'old_topic' from Merchant by alice.
```

**Notes**:
- Works for both simple and tree dialogue
- Graceful error if topic doesn't exist
- Cannot be undone

### @DIALOG NPC TEST TOPIC
Test conditions and preview dialogue for current player.

**Usage**: `@DIALOG <npc> TEST <topic>`

**Example**:
```
> @DIALOG merchant TEST trade_quest

=== Testing Dialogue: Merchant - trade_quest ===

Node Conditions:
  ✓ PASS - HasDiscussed { topic: "greeting" }
  ✗ FAIL - HasCurrency { amount: 100 }

Visible Choices:
  (none visible - player doesn't meet conditions)

Actions:
  StartQuest { quest_id: "gem_collection" }
  SendMessage { text: "Good luck finding gems!" }
```

**Shows**:
- ✓/✗ status for each condition
- Which choices are visible to you
- What actions would execute
- Helpful for debugging conditional dialogue

## Example Workflow

### Create Quest-Giving NPC

**Step 1: Add greeting**
```
@DIALOG quest_giver ADD greeting Hello traveler! I have a task for you.
```

**Step 2: Create quest offer tree**
```
@DIALOG quest_giver EDIT quest_offer {"text":"Will you help defend the village?","actions":[{"type":"start_quest","quest_id":"village_defense"},{"type":"give_item","item_id":"village_map","quantity":1}],"choices":[{"label":"Yes, I'll help!","goto":"accepted"},{"label":"Not right now","exit":true}]}
```

**Step 3: Add acceptance response**
```
@DIALOG quest_giver EDIT accepted {"text":"Thank you! Check your map for details.","actions":[{"type":"send_message","text":"Quest started! Type QUEST to view details."}],"choices":[{"label":"I'll do my best","exit":true}]}
```

**Step 4: Test it**
```
@DIALOG quest_giver TEST quest_offer
```

**Step 5: Try in-game**
```
TALK quest_giver
```

## JSON Format Reference

### Simple Node
```json
{
  "text": "Hello!"
}
```

### Node with Choices
```json
{
  "text": "What would you like?",
  "choices": [
    {"label": "Tell me more", "goto": "info"},
    {"label": "Goodbye", "exit": true}
  ]
}
```

### Node with Actions
```json
{
  "text": "Here's your reward!",
  "actions": [
    {"type": "give_currency", "amount": 100},
    {"type": "give_item", "item_id": "sword", "quantity": 1}
  ],
  "choices": [
    {"label": "Thanks!", "exit": true}
  ]
}
```

### Node with Conditions
```json
{
  "text": "Welcome back, hero!",
  "conditions": [
    {"type": "quest_status", "quest_id": "main_quest", "status": "Completed"}
  ],
  "choices": [
    {"label": "What's next?", "goto": "next_quest"}
  ]
}
```

### Complete Example
```json
{
  "text": "I see you have the gem!",
  "conditions": [
    {"type": "has_item", "item_id": "rare_gem"}
  ],
  "actions": [
    {"type": "take_item", "item_id": "rare_gem", "quantity": 1},
    {"type": "give_currency", "amount": 500},
    {"type": "complete_quest", "quest_id": "gem_hunt"},
    {"type": "set_flag", "flag": "traded_gem", "value": true}
  ],
  "choices": [
    {
      "label": "Tell me about other gems",
      "goto": "other_gems",
      "conditions": [
        {"type": "has_discussed", "topic": "gem_types"}
      ]
    },
    {"label": "Farewell", "exit": true}
  ]
}
```

## Tips & Best Practices

1. **Start Simple**: Use ADD for basic dialogue, upgrade to EDIT when you need complexity
2. **Test Conditions**: Use TEST command to verify conditions before deploying
3. **Use VIEW**: Check JSON formatting with VIEW after EDIT
4. **Version Control**: Keep backups of complex trees (copy JSON before major edits)
5. **Iterative Design**: Build dialogue trees incrementally, test frequently
6. **Clear Labels**: Use descriptive choice labels ("Tell me about quests" not "Option 1")
7. **Exit Paths**: Always provide clear exit options for players
8. **Condition Feedback**: Use SendMessage actions to explain why conditions weren't met

## Common Patterns

### Quest Giver
- Greeting → Quest Offer → Accept/Decline → Quest Started
- Use StartQuest action on acceptance
- Use GiveItem to provide quest items

### Merchant
- Greeting → Browse/Trade → Item Selection → Transaction
- Use conditions to check HasCurrency or HasItem
- Use TakeCurrency/GiveItem for trades

### Information NPCs
- Greeting → Topic Menu → Topic Details → Back to Menu
- Use multiple goto nodes for navigation
- Use "back" choice labels for clarity

### Progressive Story
- Initial Meeting → Relationship Building → Quest Chain
- Use HasDiscussed conditions for progressive unlocks
- Use SetFlag to track story progress

## System Integration

**Works with**:
- Quest system (StartQuest, CompleteQuest actions)
- Inventory (GiveItem, TakeItem, HasItem conditions)
- Currency (GiveCurrency, TakeCurrency, HasCurrency)
- Achievements (GrantAchievement, HasAchievement)
- Conversation state (SetFlag, HasFlag, HasDiscussed)

## Status

✅ **All 6 phases complete!**
- Phase 1: Multi-topic dialogue
- Phase 2: Conversation state
- Phase 3: Branching trees
- Phase 4: Conditional responses
- Phase 5: Dialog actions
- Phase 6: Admin editor

**Total Effort**: ~15 hours (original estimate: 37-50 hours)

---

**Binary**: `/opt/meshbbs/bin/meshbbs`  
**Ready for**: Content creation and alpha testing
