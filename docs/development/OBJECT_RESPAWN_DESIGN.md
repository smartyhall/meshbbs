# Object Respawn System Design

## Problem
Singleton museum objects (healing potion, ancient key, etc.) can only be taken once, preventing subsequent players from experiencing content.

## Solution: Room-Based Object Respawning

### Core Concept
Objects marked as `RespawnableWorldObject` regenerate in their origin room after a cooldown period when taken.

### Implementation

#### 1. Add ObjectFlag
```rust
// src/tmush/types.rs
pub enum ObjectFlag {
    // ... existing flags
    RespawnableWorldObject,  // Auto-regenerates in origin room
}
```

#### 2. Add Respawn Metadata to ObjectRecord
```rust
// src/tmush/types.rs
pub struct ObjectRecord {
    // ... existing fields
    pub respawn_room_id: Option<String>,      // Room where object respawns
    pub respawn_cooldown_secs: Option<u64>,   // Time between respawns (e.g., 300 = 5 min)
    pub last_taken_at: Option<DateTime<Utc>>, // When object was last taken
}
```

#### 3. Hook into TAKE Command
```rust
// src/tmush/commands.rs - in handle_take()
if object.flags.contains(&ObjectFlag::RespawnableWorldObject) {
    object.last_taken_at = Some(Utc::now());
    store.put_object(object.clone())?;
    
    // Schedule respawn
    schedule_respawn(&object.id, object.respawn_cooldown_secs.unwrap_or(300), store)?;
}
```

#### 4. Respawn Background Task
```rust
// src/tmush/respawn.rs (new file)
pub struct RespawnTask {
    object_id: String,
    respawn_at: DateTime<Utc>,
    room_id: String,
}

pub async fn respawn_worker(store: Arc<TinyMushStore>) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await; // Check every minute
        
        let now = Utc::now();
        let objects = store.list_all_objects()?;
        
        for object in objects {
            if !object.flags.contains(&ObjectFlag::RespawnableWorldObject) {
                continue;
            }
            
            if let Some(last_taken) = object.last_taken_at {
                let cooldown = object.respawn_cooldown_secs.unwrap_or(300);
                let elapsed = (now.timestamp() - last_taken.timestamp()) as u64;
                
                if elapsed >= cooldown {
                    // Check if object is in original room
                    if let Some(ref spawn_room) = object.respawn_room_id {
                        let room = store.get_room(spawn_room)?;
                        if !room.items.contains(&object.id) {
                            // Respawn it!
                            respawn_object(&object.id, spawn_room, store)?;
                        }
                    }
                }
            }
        }
    }
}

fn respawn_object(
    object_id: &str,
    room_id: &str,
    store: &TinyMushStore,
) -> Result<(), TinyMushError> {
    let mut room = store.get_room(room_id)?;
    
    if !room.items.contains(&object_id.to_string()) {
        room.items.push(object_id.to_string());
        store.put_room(room)?;
        
        // Reset taken timestamp
        let mut object = store.get_object(object_id)?;
        object.last_taken_at = None;
        store.put_object(object)?;
        
        info!("RESPAWN: {} respawned in {}", object_id, room_id);
    }
    
    Ok(())
}
```

#### 5. Update Museum Object Creation
```rust
// src/tmush/state.rs - in create_example_trigger_objects()
let mut healing_potion = ObjectRecord {
    // ... existing fields
    respawn_room_id: Some("mesh_museum".to_string()),
    respawn_cooldown_secs: Some(300), // 5 minutes
    last_taken_at: None,
};
healing_potion.flags.push(ObjectFlag::RespawnableWorldObject);
```

### Configuration
```toml
# config.toml
[respawn]
enabled = true
check_interval_secs = 60
default_cooldown_secs = 300  # 5 minutes
```

### User Experience

**When first player takes item:**
```
> take healing potion
You take the Healing Potion.
[OOC: This item will respawn here in 5 minutes for other players]
```

**When checking room later:**
```
> look
Mesh Museum
Glass cases showcase legendary mesh hardware.

You see:
  - Healing Potion
  - Ancient Key
  - Mystery Box
```

### Advantages
- ✅ Fair access for all players
- ✅ Configurable cooldowns
- ✅ Quest item availability guaranteed
- ✅ No player interaction needed
- ✅ Works for any world object

### Disadvantages
- ⚠️ Requires background task/scheduler
- ⚠️ Adds database fields
- ⚠️ More complex state management
- ⚠️ May need migration script

### Testing
```rust
#[tokio::test]
async fn test_object_respawn() {
    // Create object with respawn flag
    // Player takes it
    // Advance time 5 minutes
    // Verify object back in room
    // Verify object.last_taken_at reset
}
```

### Alternatives to Background Task
If you don't want a background process:

**Option A: Lazy Respawn on Room Enter**
```rust
// In handle_look() or room entry
check_and_respawn_room_objects(room_id, store)?;
```

**Option B: Admin Manual Respawn**
```rust
@respawn mesh_museum  // Respawns all respawnable objects in room
```

**Option C: Time-Based on LOOK**
```rust
// Check respawn eligibility when anyone looks at room
// More efficient than background task
```

## Recommendation
Start with **Strategy 1 (Clonable)** for immediate fix, then implement this respawn system for general world object management.
