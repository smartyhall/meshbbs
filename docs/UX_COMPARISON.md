# UX Comparison: Before and After Compact UI Streamlining

## Login Experience

### Before (v1.0.35)
```
Welcome, alice you are now logged in.
There are no new messages.
unauth>
```

### After (v1.0.36+)
```
Welcome, alice you are now logged in.
There are no new messages.
Hint: M=messages H=help
alice (lvl1)>
```

---

## Compact Help (HELP)

### Before (v1.0.35)
```
alice (lvl1)> HELP
ACCT: SETPASS <new> | CHPASS <old> <new> | LOGOUT
MSG: M topics; 1-9 pick; U up; +/-; F <txt>; READ/POST/TOPICS
OTHER: WHERE | U | Q
alice (lvl1)>
```

**Issue**: Users see both `M topics` AND `READ/POST/TOPICS`, creating confusion about which to use.

### After (v1.0.36+)
```
alice (lvl1)> HELP
ACCT: SETPASS <new> | CHPASS <old> <new> | LOGOUT
MSG: M topics; 1-9 pick; U up; +/-; F <txt>
OTHER: WHERE | U | Q
alice (lvl1)>
```

**Improvement**: Clear, streamlined interface. Press **M** for messages, use digits to navigate.

---

## Verbose Help (HELP+)

### Before (v1.0.35)
```
...
Legacy commands (compat):
  TOPICS / LIST           List topics
  READ <topic>            Read recent messages
  POST <topic> <text>     Post a message
...
```

**Issue**: "Legacy" doesn't communicate that these are deprecated.

### After (v1.0.36+)
```
...
Compact commands only:
  M / digits              Navigate topics and threads
  P                       Open preferences
  WHERE                   Show breadcrumb
  HELP / HELP+            Compact help (same output)
...
```

**Improvement**: Verbose help now reinforces the single compact command set without referencing removed long-form commands.

---

## User Journey

### Confused User (Before)
1. Login: "Welcome, alice..."
2. Type `HELP`
3. See: "MSG: M topics; 1-9 pick; U up; +/-; F <txt>; READ/POST/TOPICS"
4. Think: "Wait, do I press M or type READ?"
5. Try both, get confused by different outputs
6. Ask sysop for help

### Happy User (After)
1. Login: "Welcome, alice... Hint: M=messages H=help"
2. Press **M**
3. See: "[Meshbbs] Topics\n1. general  2. community  3. technical"
4. Press **1**
5. See threads in that topic
6. Navigate naturally with digits and letters

---

## Command Set Summary

- **Before**: Compact shortcuts and legacy long-form commands operated side by side
- **After**: Only compact shortcuts remain (`M`, digits, `P`, `R`, `B`, etc.)
- **Result**: Lower cognitive load, fewer bytes on the wire, and simpler docs/support

## Design Philosophy

**Before**: Two equal interfaces competing for user attention  
**After**: A single, compact interface for every user path  

**Goal**: Reduce cognitive load for new users while keeping navigation fast and predictable.

---

## Metrics

### Help Text Size
- **Before**: ~82 bytes for MSG line (includes legacy commands)
- **After**: ~52 bytes for MSG line (30 bytes saved)
- **Benefit**: More room for other content within 230-byte frame limit

### User Onboarding
- **Before**: 3 different ways to list topics (M, TOPICS, LIST)
- **After**: One clear way (M and digits) with no legacy fallbacks
- **Benefit**: Faster learning curve, less confusion

---

## For Sysops

Action items:
1. Educate users and moderators about the compact-only command set
2. Update any SOPs, scripts, or cheat sheets referencing long-form commands
3. Encourage use of `HELP`/`HELP+` for quick refresher on shortcuts

---

## Next Steps

See `docs/MIGRATION_COMPACT_UI.md` for:
- Full migration plan
- Timeline for future versions
- Instructions for documentation writers
- Technical details for developers
