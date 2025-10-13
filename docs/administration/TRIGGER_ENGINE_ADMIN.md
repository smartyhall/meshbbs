# TinyMUSH Trigger Engine - Admin Guide

## Overview

The Trigger Engine enables interactive objects through a sandboxed scripting system. As an admin, you can monitor, control, and troubleshoot trigger execution.

**Version**: 1.0 (Phase 11 - Production Ready)  
**Status**: 240 tests passing, 95% complete

---

## Admin Commands

### View Rate Limiter Status
```
@trigger-status
```

Shows:
- Total tracked objects
- Total tracked players
- Number of disabled objects
- Global enable/disable status

**Access Level**: Moderator+ (Level 1+)

---

### Disable Problematic Triggers
```
@trigger-disable <object_id>
```

Disables all triggers on a specific object. Use this when:
- A trigger is causing errors
- A trigger is being abused
- You need to investigate an issue

The object remains in the game, but triggers won't fire.

**Access Level**: Moderator+ (Level 1+)

**Example**:
```
@trigger-disable mystery_box
→ Triggers disabled for object: mystery_box
```

---

### Re-Enable Triggers
```
@trigger-enable <object_id>
```

Re-enables triggers on a previously disabled object.

**Access Level**: Moderator+ (Level 1+)

**Example**:
```
@trigger-enable mystery_box
→ Triggers enabled for object: mystery_box
```

---

### List Disabled Objects
```
@trigger-list-disabled
```

Shows all objects with disabled triggers and when they were disabled.

**Access Level**: Moderator+ (Level 1+)

**Output Format**:
```
Disabled Triggers:
- object_id_1 (disabled: 2025-10-11 14:30:00 UTC)
- object_id_2 (disabled: 2025-10-11 15:45:00 UTC)

Total: 2 disabled objects
```

---

### Emergency Global Shutoff
```
@trigger-global off
@trigger-global on
```

**CRITICAL**: This disables/enables ALL triggers system-wide!

Use this only for:
- Emergency situations (runaway scripts)
- System maintenance
- Major bug discovery

When global triggers are off:
- No triggers fire anywhere
- Players see no error messages
- Triggers fail silently
- All trigger commands still work (for setup)

**Access Level**: Admin+ (Level 2+)

**Example**:
```
@trigger-global off
→ ⛔ Global trigger system DISABLED
→ All triggers will fail silently until re-enabled.

@trigger-global on
→ ✅ Global trigger system ENABLED
```

---

## Monitoring & Logs

### Check Trigger Execution
Trigger execution is logged at INFO level:

```
[INFO] Executing OnUse trigger for object 'healing_potion'
[INFO] Trigger executed successfully: 2 messages
```

### Check Rate Limit Events
Rate limiting events are logged at WARN level:

```
[WARN] Rate limit exceeded for object 'spam_object' (100/min limit)
[WARN] Player cooldown active for 'player_name' (1s cooldown)
```

### Check Trigger Errors
Errors are logged at ERROR level:

```
[ERROR] Trigger execution failed for 'broken_object': Parse error: Invalid syntax
[ERROR] Trigger timed out after 100ms for 'slow_object'
```

---

## Security Limits

The trigger system has hard-coded security limits:

| Limit | Value | Reason |
|-------|-------|--------|
| **Script Length** | 512 chars | Prevent giant scripts |
| **Execution Time** | 100ms | Prevent infinite loops |
| **Max Actions** | 10 | Prevent action spam |
| **Max Messages** | 3 | Prevent message spam |
| **Object Rate Limit** | 100/min | Prevent trigger abuse |
| **Player Cooldown** | 1 second | Prevent player spam |

These limits are enforced in code and **cannot be bypassed** without a code change.

---

## Rate Limiting System

### How It Works

**Per-Object Limits:**
- Each object can fire triggers max 100 times per minute
- Counter resets every 60 seconds (rolling window)
- Exceeding limit silently fails triggers

**Per-Player Cooldowns:**
- Each player has 1-second cooldown between triggering same object
- Prevents spam-clicking
- Does not apply across different objects

**Global Shutoff:**
- Admin can disable ALL triggers instantly
- Emergency use only
- No performance impact when off

### Rate Limit Scenarios

**Scenario 1: Popular Object**
A "petting zoo animal" gets poked by 50 players in 1 minute:
- ✅ First 100 pokes work (2 per player)
- ❌ Remaining pokes fail silently
- Players see: (no message, appears to do nothing)

**Admin Action**: Monitor with `@trigger-status`, consider if object needs optimization

**Scenario 2: Spam Player**
A player rapidly clicks "USE POTION" 10 times:
- ✅ First use works
- ❌ Next 9 uses fail (1-second cooldown)
- Player sees: (no response for 1 second)

**Admin Action**: None needed, working as designed

**Scenario 3: Runaway Script**
A badly written trigger causes infinite recursion:
- ✅ Execution times out after 100ms
- ❌ Script fails with error logged
- Player sees: "Trigger execution failed"

**Admin Action**: `/show <object>` to inspect, `/remove <object> <trigger>` to fix

---

## Troubleshooting Guide

### Problem: "Triggers not firing"

**Check 1: Global status**
```
@trigger-status
```
Look for "globally_enabled: false"

**Fix**: `@trigger-global on`

---

**Check 2: Object disabled**
```
@trigger-list-disabled
```
Look for your object in the list

**Fix**: `@trigger-enable <object>`

---

**Check 3: Rate limit exceeded**
Check logs for "Rate limit exceeded" warnings

**Fix**: Wait 60 seconds for object limit reset, or 1 second for player cooldown

---

**Check 4: Script error**
Use `/test <object> <trigger>` to check for syntax errors

**Fix**: Use `/script <object> <trigger>` to rewrite the script

---

### Problem: "Trigger causing errors"

**Step 1: Disable immediately**
```
@trigger-disable <object_id>
```

**Step 2: Inspect the script**
```
/show <object>
```

**Step 3: Test in isolation**
```
/test <object> <trigger>
```

**Step 4: Fix or remove**
```
/remove <object> <trigger>
```
Then recreate with `/when` or `/script`

**Step 5: Re-enable**
```
@trigger-enable <object_id>
```

---

### Problem: "Performance issues"

**Symptom**: Game feels slow, high CPU usage

**Check 1**: Look for trigger execution in logs
```
grep "Trigger executed" meshbbs.log | wc -l
```

**Check 2**: Identify hot objects
```
grep "Rate limit exceeded" meshbbs.log | cut -d"'" -f2 | sort | uniq -c | sort -rn
```

**Check 3**: Disable problem objects
```
@trigger-disable <object_with_most_triggers>
```

**Fix**: Optimize or remove problematic triggers

---

## Best Practices for Admins

### 1. Monitor New Triggers
When players create triggers, spot-check them:
```
/show <new_object>
```

Look for:
- Excessively long scripts
- Obvious infinite loops
- Inappropriate content

### 2. Set Expectations
Tell players about the limits:
- 512 character scripts
- 100ms execution time
- Rate limiting

### 3. Use Test Mode
Always test triggers before enabling:
```
/test <object> <trigger>
```

### 4. Document Disabled Objects
When you disable triggers, note why:
```
@trigger-disable spam_box
# Then post in admin channel: "Disabled spam_box - infinite loop bug"
```

### 5. Regular Audits
Weekly, check:
```
@trigger-status           # Overall health
@trigger-list-disabled    # Orphaned disabled objects
```

### 6. Emergency Procedures
Have a plan:
1. `@trigger-global off` - Immediate shutoff
2. Check logs for culprit
3. `@trigger-disable <problem_object>`
4. `@trigger-global on` - Resume normal operation
5. Fix offline

---

## Performance Considerations

### Trigger Execution Cost

**Single Trigger:**
- Parse: < 1ms
- Execute: < 10ms (typically)
- Total: < 15ms

**OnEnter in Room with 50 Objects:**
- 50 triggers × 15ms = 750ms worst case
- Parallel execution not implemented (future)
- Rate limiting prevents abuse

**Recommendation**: Limit OnEnter triggers to < 10 objects per room

---

### Database Impact

**Per Trigger Execution:**
- 2-3 reads (player, object, room)
- 0-2 writes (if action modifies state)

**Optimization**:
- Objects are cached in memory
- Rooms are cached (LRU)
- Players loaded once per session

**At Scale (1000 users):**
- Triggers add ~5-10% overhead
- Database remains primary bottleneck
- Trigger execution is negligible compared to I/O

---

## Future Enhancements

**Planned for v1.1:**
- [ ] Trigger execution logs (per-object audit trail)
- [ ] Trigger performance profiling
- [ ] Admin dashboard with trigger statistics
- [ ] Automated abuse detection
- [ ] Trigger quotas per player (max N triggers)

**Planned for v2.0:**
- [ ] Parallel OnEnter execution
- [ ] Trigger scheduling (time-based triggers)
- [ ] Trigger chaining (trigger fires another trigger)
- [ ] Trigger events (broadcast to multiple objects)

---

## API Reference

### Rate Limiter API

**Check if trigger allowed:**
```rust
let limiter = get_global_rate_limiter();
match limiter.check_allowed(object_id, player_name) {
    Ok(()) => {
        // Execute trigger
        limiter.record_execution(object_id, player_name);
    }
    Err(reason) => {
        // Handle rate limit (fail silently)
        log::warn!("Rate limited: {}", reason);
    }
}
```

**Admin operations:**
```rust
limiter.disable_object(object_id);
limiter.enable_object(object_id);
limiter.set_global_enabled(false);
limiter.is_globally_enabled();
limiter.get_stats();
limiter.get_disabled_objects();
```

---

## Support & Contact

**For trigger issues:**
- Check logs first
- Use admin commands to investigate
- Document the issue (object ID, trigger type, timestamp)
- Contact development team if needed

**For feature requests:**
- Post in #tinymush-development
- Include use case and examples
- Consider if it can be done with existing features

**For security concerns:**
- Use `@trigger-global off` immediately
- Document the vulnerability
- Contact security team directly

---

## Testing the System

Run the full trigger test suite:
```bash
# All trigger tests
cargo test tmush::trigger

# Integration tests
cargo test trigger_behaviors
cargo test trigger_security
cargo test take_drop_triggers

# Performance tests
cargo test --test trigger_security test_room_with_many_triggered_objects
```

**Expected**:
- 240 tests passing
- 0 failures
- All tests < 5 seconds

---

**Last Updated**: 2025-10-11  
**Trigger Engine Version**: 1.0  
**Documentation Version**: 1.0
