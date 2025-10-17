# Queue Health Monitoring & Circuit Breakers

## Overview

This document describes the preventative measures added to MeshBBS to prevent queue saturation and system lockup issues where the BBS stops responding to commands after extended runtime.

## Problem Summary

After ~2 hours of operation, MeshBBS was observed to:
- Stop responding to incoming commands on the public channel
- Continue sending ident beacons (showing partial functionality)
- Display commands arriving on screen but produce no response

Root causes identified:
1. **Scheduler Queue Saturation**: Bounded queue (512 messages) fills faster than messages can be sent
2. **Writer Pending HashMap Growth**: Unbounded accumulation of awaiting ACKs
3. **No Backpressure**: System accepts messages even when overloaded
4. **No Health Monitoring**: Silent failure mode with no warnings

## Preventative Measures Implemented

### 1. Scheduler Health Monitoring

**Location**: `src/bbs/server.rs`

**What it does**:
- Checks scheduler stats every 30 seconds
- Warns when queue depth exceeds 80% capacity
- Alerts when messages are being dropped
- Logs recovery when queue returns to normal

**Key metrics tracked**:
```rust
- queued: current queue size
- dropped_total: cumulative drops
- dropped_overflow: drops due to full queue
- escalations: priority bumps due to aging
```

**Example output**:
```
WARN  Scheduler queue high: 450/512 messages (87.9%) - drops=0 escalations=12 (warning #3)
ERROR Message scheduler dropping messages! New drops: 5 (total: 5, overflow: 5)
INFO  Scheduler queue recovered: 0 messages queued
```

### 2. Circuit Breaker on Send

**Location**: `src/bbs/server.rs::send_message()`

**What it does**:
- Checks queue depth before enqueuing new messages
- Rejects messages when queue >95% full
- Prevents cascade failure by failing fast
- Returns error to caller rather than silently dropping

**Behavior**:
```rust
if stats.queued > (max_queue * 95 / 100) {
    warn!("Scheduler queue critically full, dropping message");
    return Err(anyhow!("Message queue full - system overloaded"));
}
```

**Impact**:
- Better to fail one user's command than to lock up entire system
- User sees "command failed" rather than silent timeout
- Prevents accumulation of messages that will be dropped anyway

### 3. Pending HashMap Bounds

**Location**: `src/meshtastic/mod.rs::MeshtasticWriter`

**What it does**:
- Limits pending sends to 100 entries (MAX_PENDING_SENDS)
- Cleans up old entries every 5 minutes (PENDING_CLEANUP_INTERVAL_SECS)
- Removes entries older than 10 minutes (PENDING_MAX_AGE_SECS)
- Warns when approaching limit (>80%)

**Cleanup logic**:
```rust
// Phase 1: Remove entries older than 10 minutes
pending.retain(|id, p| {
    age < PENDING_MAX_AGE_SECS
});

// Phase 2: If still over limit, remove oldest entries
if pending.len() > MAX_PENDING_SENDS {
    // Remove oldest entries to get back under limit
}
```

**Example output**:
```
WARN  Writer pending HashMap getting large: 85 entries (limit 100)
WARN  Dropping stale pending send: id=12345 to=0xABCD1234 age=645s
INFO  Pending cleanup: removed 3 entries (2 by age, 1 by limit), 82 remaining
```

### 4. Periodic Health Logging

**Frequency**: Every 30 seconds in main event loop

**What it monitors**:
1. **Scheduler queue depth** - early warning of saturation
2. **Message drop count** - immediate alert when dropping begins
3. **Priority escalations** - sign of messages aging in queue
4. **Pending HashMap size** - prevent unbounded growth
5. **Recovery detection** - log when system returns to healthy state

## Configuration Tunables

### Scheduler Configuration
```toml
[meshtastic]
scheduler_max_queue = 512              # Maximum queued messages
scheduler_aging_threshold_ms = 5000    # Priority escalation time
min_send_gap_ms = 2000                 # Minimum gap between sends
```

### Writer Tuning (Code Constants)
```rust
const MAX_PENDING_SENDS: usize = 100;           // HashMap size limit
const PENDING_CLEANUP_INTERVAL_SECS: u64 = 300; // 5 minutes
const PENDING_MAX_AGE_SECS: u64 = 600;          // 10 minutes
```

## Monitoring in Production

### What to Watch For

**Warning Signs** (indicate approaching problems):
```
WARN  Scheduler queue high: 410/512 messages (80.1%)
WARN  Writer pending HashMap getting large: 85 entries
```

**Critical Issues** (indicate active problems):
```
ERROR Message scheduler dropping messages! New drops: 10
WARN  Scheduler queue critically full (487/512), dropping message
```

**Recovery Signs** (indicate problems resolved):
```
INFO  Scheduler queue recovered: 0 messages queued
INFO  Pending cleanup: removed 15 entries, 45 remaining
```

### Log Analysis Commands

**Check for queue saturation**:
```bash
grep "Scheduler queue high" meshbbs.log
```

**Check for dropped messages**:
```bash
grep "dropping messages" meshbbs.log
```

**Check pending HashMap growth**:
```bash
grep "pending HashMap" meshbbs.log
```

**Check cleanup activity**:
```bash
grep "Pending cleanup" meshbbs.log
```

## Expected Behavior

### Healthy System
- Queue depth: 0-50 messages
- Pending sends: 0-20 entries
- No drops or circuit breaker triggers
- Occasional priority escalations during bursts

### Stressed System (Still Operational)
- Queue depth: 50-400 messages
- Pending sends: 20-80 entries
- Warnings logged but no drops
- Frequent priority escalations
- Messages delayed but eventually delivered

### Overloaded System (Circuit Breaker Active)
- Queue depth: >400 messages
- Pending sends: >80 entries
- Messages being dropped
- Circuit breaker rejecting new messages
- Some users see "system overloaded" errors
- Ident beacons still working (low priority, infrequent)

## Tuning Recommendations

### If Queue Fills Frequently

**Symptom**: Regular "queue high" warnings
**Solution**: Reduce message frequency or increase queue size

```toml
[meshtastic]
scheduler_max_queue = 1024  # Double the queue size
min_send_gap_ms = 2500      # Slow down slightly if radio can't keep up
```

### If Pending HashMap Grows

**Symptom**: Regular "pending HashMap getting large" warnings
**Solution**: Investigate why ACKs aren't arriving

Possible causes:
- Radio coverage issues
- Mesh network congestion
- Recipients offline/out of range
- Too many unreachable nodes in conversations

Consider:
- Reducing MAX_PENDING_SENDS if memory is concern
- Reducing PENDING_MAX_AGE_SECS to expire faster
- Implementing sender reputation tracking

### If Circuit Breaker Triggers Often

**Symptom**: Users frequently see "system overloaded"
**Solution**: Either increase capacity or reduce load

Increase capacity:
```toml
[meshtastic]
scheduler_max_queue = 1024
min_send_gap_ms = 1800  # Allow faster throughput
```

Reduce load:
- Implement rate limiting on public commands
- Reduce welcome message complexity
- Disable non-essential broadcasts
- Limit concurrent active sessions

## Integration with Existing Systems

### Metrics Module
All counters are already wired into the metrics module:
- `metrics::inc_reliable_sent()`
- `metrics::inc_reliable_acked()`
- `metrics::inc_reliable_failed()`
- `metrics::observe_ack_latency()`

### Logging
Uses standard log macros:
- `warn!()` for health warnings
- `error!()` for critical issues
- `info!()` for recovery/normal operations
- `debug!()` for detailed diagnostics

### No Breaking Changes
All monitoring is additive:
- No configuration changes required
- No API changes
- No behavioral changes under normal load
- Only affects behavior when system is overloaded

## Future Enhancements

### Potential Improvements
1. **Adaptive Rate Limiting**: Automatically throttle based on queue depth
2. **Per-User Fairness**: Prevent single user from flooding queue
3. **Priority Classes**: Different limits for system vs user messages
4. **Metrics Export**: Prometheus/Grafana integration
5. **Auto-Recovery**: Automatic restart of writer on detected failure
6. **Congestion Control**: Backoff algorithms based on ACK success rate

### Monitoring Dashboard Ideas
- Queue depth over time (line graph)
- Drop rate over time (line graph)
- Pending sends histogram
- ACK latency percentiles (p50, p95, p99)
- Messages per minute by category
- Circuit breaker trigger frequency

## Testing Recommendations

### Load Testing
Simulate the 2-hour issue:
```bash
# Generate sustained load
for i in {1..1000}; do
    echo "Test message $i" | mosquitto_pub -t meshbbs/test
    sleep 0.5
done
```

### Monitor During Test
```bash
# Watch for warnings
tail -f meshbbs.log | grep -E "WARN|ERROR"

# Check queue stats
tail -f meshbbs.log | grep "scheduler stats"
```

### Success Criteria
After 2+ hours:
- System still responds to commands
- Queue depth under 50% capacity
- No circuit breaker triggers
- Pending HashMap stable (<50 entries)

## Conclusion

These preventative measures provide "ounces of prevention" by:
1. **Early Warning**: Detect problems before total failure
2. **Graceful Degradation**: Fail cleanly rather than silently
3. **Bounded Resources**: Prevent unbounded growth
4. **Observable Behavior**: Clear logs for debugging

The system now fails **loudly and gracefully** rather than silently locking up. This allows operators to:
- Detect issues early via logs
- Understand what's happening during overload
- Tune configuration appropriately
- Plan capacity based on observed metrics

**Result**: The "pound of cure" (debugging a frozen system) is replaced with proactive monitoring and controlled failure modes.
