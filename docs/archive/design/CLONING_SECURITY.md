# Object Cloning Security Design

**Author**: Security Review  
**Date**: 2025-10-11  
**Status**: Implemented  
**Phase**: Phase 6 - Object Cloning & Script Persistence

## Overview

Object cloning in MeshBBS TinyMUSH allows players to duplicate objects they own, including their trigger scripts. This powerful feature requires robust security controls to prevent economic exploits, resource exhaustion attacks, and permission escalation.

This document outlines the threat model, security controls, and implementation strategy for safe object cloning.

## Threat Model

### 1. Exponential Cloning Attack (Resource Exhaustion)
**Attack Vector**: Clone object A → B, then clone B → C, recursively creating thousands of objects.

**Impact**:
- Memory exhaustion
- Disk space exhaustion
- CPU saturation processing clone operations
- Denial of service for all users

**Real-world parallel**: WoW item duplication exploits (2004-2007)

**Mitigation**:
- Clone depth limit (max 3 generations)
- Per-player clone quota (20/hour)
- Cooldown timer (60 seconds between clones)
- Global clone rate monitoring

### 2. Currency Duplication (Economic Exploit)
**Attack Vector**: Clone objects with gold value or containers holding currency.

**Impact**:
- Hyperinflation
- Economy collapse
- Devaluation of legitimate currency
- Game balance destruction

**Real-world parallel**: Second Life L$ duplication bugs (2007)

**Mitigation**:
- Strip gold value on clone (set to 0)
- Value threshold (objects >100 gold non-clonable)
- Empty containers on clone
- NO_VALUE flag for currency items

### 3. Quest Item Duplication (Progression Break)
**Attack Vector**: Clone unique quest keys, artifacts, or rewards.

**Impact**:
- Quest progression breaks
- Multiple players complete quests prematurely
- World lore inconsistency ("The One Ring" × 100)
- Game balance destruction

**Mitigation**:
- UNIQUE flag blocks cloning
- QUEST_ITEM flag strips quest associations
- Quest completion checks original object ID

### 4. Permission Escalation (Security Vulnerability)
**Attack Vector**: Clone admin/architect-owned object, inherit elevated permissions.

**Impact**:
- Privilege escalation
- Unauthorized world editing
- Security policy bypass
- Trust boundary violation

**Real-world parallel**: WoW GM item duplication (2006)

**Mitigation**:
- Clone always owned by cloner (never inherits owner)
- Strip all elevated permissions
- Audit log all clone operations

### 5. Trigger Script Abuse (Economic Exploit)
**Attack Vector**: Clone object with valuable trigger (heal, teleport, give gold), mass-activate.

**Impact**:
- Unlimited healing/resources
- Economic balance destruction
- Server performance degradation

**Mitigation**:
- Existing trigger execution limits (action_count)
- Clone tracking in trigger context
- Optional: Degraded triggers in clones

### 6. Storage Quota Abuse (Resource Exhaustion)
**Attack Vector**: Create thousands of objects to fill database.

**Impact**:
- Disk space exhaustion
- Backup storage costs
- Database performance degradation
- Denial of service

**Mitigation**:
- Per-player object ownership limit (100 max)
- Per-room item limits (already implemented: 25/50)
- Admin monitoring dashboard

### 7. Clone Bombing (Griefing Attack)
**Attack Vector**: Clone and drop thousands of objects in public areas.

**Impact**:
- Room unusability (spam)
- Server lag
- New player experience degradation

**Mitigation**:
- Per-room item limits (already implemented)
- Clone cooldown prevents rapid spam
- Moderation tools for cleanup

### 8. Companion/NPC Duplication (Game Balance)
**Attack Vector**: Clone trained companions, pets, or NPCs.

**Impact**:
- Free max-level companions
- Companion economy bypass
- Pet training progression meaningless

**Mitigation**:
- NO_CLONE flag on all companions
- Type check: companions always non-clonable

### 9. Script Variable Leakage (Information Disclosure)
**Attack Vector**: Clone object with script referencing original owner's data.

**Impact**:
- Password/secret disclosure
- Privacy violation
- Information leakage

**Example**:
```
Script: "Say $original_owner_password"
Clone → executes with leaked data
```

**Mitigation**:
- Clear sensitive variables on clone
- Sanitize script context
- Owner variable rewrite

### 10. Audit Trail Gaps (Detection Failure)
**Attack Vector**: Exploit unlogged operations, hide abuse patterns.

**Impact**:
- Undetected exploitation
- Unable to trace abuse
- No forensic evidence

**Mitigation**:
- Comprehensive clone logging
- Track clone ancestry (genealogy)
- Admin alert on suspicious patterns

## Security Controls

### Object Flags

```rust
pub struct ObjectFlags {
    /// Can this object be cloned? (Default: false - opt-in)
    pub clonable: bool,
    
    /// Unique object - blocks cloning entirely (quest items, artifacts)
    pub unique: bool,
    
    /// Strip currency/value to 0 on clone
    pub no_value: bool,
    
    /// Quest-critical item - non-clonable, strips quest flags on clone
    pub quest_item: bool,
    
    /// Refuse to clone if object has contents (containers)
    pub no_clone_children: bool,
}
```

### Clone Tracking Metadata

```rust
pub struct ObjectRecord {
    // ... existing fields ...
    
    /// Clone genealogy depth (0 = original, 1 = first clone, etc.)
    pub clone_depth: u8,
    
    /// Source object ID if this is a clone
    pub clone_source_id: Option<String>,
    
    /// How many times THIS specific object has been cloned
    pub clone_count: u32,
    
    /// Username of player who created/cloned this object
    pub created_by: String,
    
    /// Unix timestamp of creation/clone
    pub created_at: u64,
}
```

### Player Limits

```rust
pub struct PlayerRecord {
    // ... existing fields ...
    
    /// Remaining clones this hour (resets hourly)
    pub clone_quota: u32,
    
    /// Unix timestamp of last clone operation (cooldown enforcement)
    pub last_clone_time: u64,
    
    /// Total objects owned by this player (quota enforcement)
    pub total_objects_owned: u32,
}
```

### Configuration Constants

```rust
/// Maximum clone genealogy depth (prevent exponential growth)
const MAX_CLONE_DEPTH: u8 = 3;

/// Maximum objects one player can own (prevent storage abuse)
const MAX_OBJECTS_PER_PLAYER: u32 = 100;

/// Cooldown between clone operations in seconds (prevent spam)
const CLONE_COOLDOWN: u64 = 60;

/// Clone quota per player per hour (resets hourly)
const CLONES_PER_HOUR: u32 = 20;

/// Maximum gold value for clonable objects (economic protection)
const MAX_CLONABLE_VALUE: u32 = 100;
```

## Implementation Strategy

### Phase 6.1: Data Model Updates
1. Add ObjectFlags struct to types.rs
2. Add clone tracking fields to ObjectRecord
3. Add player limit fields to PlayerRecord
4. Migration: Set defaults for existing objects
5. Tests: Verify serialization round-trips

### Phase 6.2: Clone Function Implementation
1. Implement `clone_object()` with all safety checks
2. Fail-fast validation (check all conditions before mutation)
3. Ownership transfer (always set to cloner)
4. Attribute sanitization (strip currency, quest flags, contents)
5. Metadata tracking (depth, source, timestamps)
6. Audit logging (comprehensive operation log)

### Phase 6.3: Player Commands
1. `/clone <object>` command implementation
2. Integrate with builder_commands.rs
3. User-friendly error messages for each failure case
4. Success message with clone details

### Phase 6.4: Admin Tools
1. `/listclones <player>` - Show all clones by player
2. `/clonestats` - Server-wide clone statistics
3. Alert system for suspicious patterns
4. Cleanup tools for clone spam

### Phase 6.5: Testing
1. Unit tests for each safety check
2. Integration tests for exploit scenarios
3. Load tests (mass cloning simulation)
4. Fuzzing (malformed inputs)

## Clone Operation Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Player executes: /clone <object>                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │ Resolve object name          │
          │ (use resolver.rs)            │
          └──────────┬───────────────────┘
                     │
                     ▼
          ┌──────────────────────────────┐
          │ Safety Checks (FAIL FAST)    │
          │ ├─ Object clonable?          │
          │ ├─ Not unique?               │
          │ ├─ Clone depth < 3?          │
          │ ├─ Player quota remaining?   │
          │ ├─ Player object count < 100?│
          │ ├─ Cooldown expired?         │
          │ ├─ Value < 100 gold?         │
          │ └─ Player owns original?     │
          └──────────┬───────────────────┘
                     │
            ┌────────┴────────┐
            │                 │
         FAIL              SUCCESS
            │                 │
            ▼                 ▼
    ┌──────────────┐   ┌─────────────────────────┐
    │ Return error │   │ Create sanitized clone  │
    │ with reason  │   │ ├─ New ULID             │
    └──────────────┘   │ ├─ Set owner to cloner  │
                       │ ├─ Increment depth      │
                       │ ├─ Strip currency       │
                       │ ├─ Clear quest flags    │
                       │ ├─ Empty container      │
                       │ └─ Copy triggers        │
                       └──────────┬──────────────┘
                                  │
                                  ▼
                       ┌──────────────────────────┐
                       │ Update metadata          │
                       │ ├─ Source clone_count++  │
                       │ ├─ Player clone_quota--  │
                       │ ├─ Player objects_owned++│
                       │ └─ Set last_clone_time   │
                       └──────────┬──────────────┘
                                  │
                                  ▼
                       ┌──────────────────────────┐
                       │ Audit log                │
                       │ log::info!("CLONE: ...") │
                       └──────────┬──────────────┘
                                  │
                                  ▼
                       ┌──────────────────────────┐
                       │ Return success message   │
                       │ "✨ Cloned 'Crystal'!"   │
                       └──────────────────────────┘
```

## Audit Logging

Every clone operation logs:
```
[INFO] CLONE: alice cloned obj_crystal_01 -> obj_crystal_02 (depth 1, quota 19/20, total_objects 47/100)
```

Suspicious patterns trigger alerts:
- Clone rate > 10/minute (bot detection)
- Clone depth = 3 (approaching limit)
- Player objects > 90 (approaching quota)
- Global clone rate spike

## Testing Strategy

### Unit Tests
- Each safety check in isolation
- Flag validation
- Metadata tracking
- Quota enforcement

### Integration Tests
- Full clone operation flow
- Multi-generation cloning
- Quota exhaustion scenarios
- Cooldown timing

### Exploit Tests (Negative Testing)
- Exponential cloning attempt
- Currency duplication attempt
- Permission escalation attempt
- Container nesting attack
- Clone bombing simulation

### Performance Tests
- 100 concurrent clone operations
- Database load under clone spam
- Memory usage with 10,000 clones

## Security Review Checklist

- [ ] All threat model scenarios have mitigations
- [ ] No unchecked user input reaches database
- [ ] Fail-fast: All validation before mutation
- [ ] Ownership always set to cloner (never inherited)
- [ ] Currency/value stripped on clone
- [ ] Clone depth enforced (max 3)
- [ ] Player quotas enforced (20/hour, 100 total)
- [ ] Cooldown enforced (60 seconds)
- [ ] Audit logging comprehensive
- [ ] Error messages don't leak sensitive info
- [ ] Admin monitoring tools implemented
- [ ] Tests cover all exploit scenarios
- [ ] Documentation complete

## References

- **World of Warcraft Item Duplication Exploits**: [Blizzard Security Archives]
- **Second Life L$ Duplication**: CVE-2007-XXXX (currency replication bug)
- **OWASP Top 10**: Business Logic Flaws, Resource Exhaustion
- **CWE-770**: Allocation of Resources Without Limits or Throttling
- **CWE-799**: Improper Control of Interaction Frequency

## Changelog

- **2025-10-11**: Initial security design document
- **Phase 6**: Implementation with comprehensive safety controls

---

**Security Principle**: *Defense in depth - multiple overlapping controls ensure no single failure compromises security.*
