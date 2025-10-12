# TinyMUSH Data Schema (Phase 1)

Phase&nbsp;1 introduces a dedicate```rust
use meshbbs::tmush::{
    TinyMushStore, PlayerRecord, REQUIRED_START_LOCATION_ID,
};

let store = TinyMushStore::open("./data/tinymush")?;
let mut player = PlayerRecord::new("alice", "Alice", REQUIRED_START_LOCATION_ID);database for TinyMUSH content. The goals are to keep
player saves, canonical rooms, and builder-created assets in a compact binary format while
preserving forward migration paths via explicit schema versions.

## Storage Layout

The database lives under `tinymush/` inside the main `data` directory by default. You can
override the location by setting `games.tinymush_db_path` in `config.toml`.

Sled trees and key prefixes:

| Tree name           | Key prefix      | Stored value                | Notes                                           |
|--------------------|-----------------|-----------------------------|-------------------------------------------------|
| `tinymush`         | `players:{id}`  | `PlayerRecord` (bincode)    | Player location, stats, inventory, state        |
| `tinymush`         | `rooms:world:{id}` / `rooms:player:{owner}:{id}` | `RoomRecord` | Canonical map vs. builder rooms                  |
| `tinymush_objects` | `objects:world:{id}` / `objects:player:{owner}:{id}` | `ObjectRecord` | Item definitions and scripted interactions       |
| `tinymush_mail`    | `mail:{user}:{ts}` | UTF-8 text payload         | Queued in-game mail / async notifications       |
| `tinymush_logs`    | `logs:{ts}`     | UTF-8 text payload           | Operational breadcrumbs for debugging           |

> **Serialization**: values are encoded using `bincode` with fixed schema versions embedded in
the struct. Deserializing a mismatched version returns a `TinyMushError::SchemaMismatch` so
migrations can reroute data before Phase&nbsp;2 adds write paths.

## Core Data Structures

All structs live in `src/tmush/types.rs` and derive `Serialize`/`Deserialize` so they can be
persisted directly. Highlights:

- `PlayerRecord`
  - `username`, `display_name`
  - `current_room`, `state` (`Exploring`, `InDialog`, `InCombat`, `Shopping`, `ViewingInventory`, `Dead`)
  - `stats` (`hp`, `mp`, attributes, armor class)
  - `inventory` (vector of object IDs) and `credits`
  - `created_at`, `updated_at`, `schema_version`
- `RoomRecord`
  - `owner` (`World` or `Player { username }`)
  - `short_desc`, `long_desc`, `exits: HashMap<Direction, String>`
  - `flags` (`Safe`, `Shop`, `QuestLocation`, etc.), `max_capacity`, `schema_version`
- `ObjectRecord`
  - `owner`, `weight`, `value`, `takeable`, `usable`
  - `actions: HashMap<ObjectTrigger, String>` for scripted responses
  - `flags` (e.g., `Consumable`, `KeyItem`) and `schema_version`

All schema constants start at `1` for players, rooms, and objects. Future migrations can bump
them independently.

## Canonical World Seed

`tmush::state::canonical_world_seed()` yields a deterministic snapshot of the "Old Towne Mesh"
starter area. `TinyMushStore::seed_world_if_needed()` writes the seed to the database on first
launch and no-ops afterward.

Seeded location IDs (Old Towne sample world):

- `gazebo_landing` – required landing location where wizards finalize new characters
- `town_square` – required starting location after creation completes
- `city_hall_lobby`
- `mesh_museum`
- `north_gate`
- `south_market`

Only the first two IDs are mandatory system requirements. Operators are encouraged to replace
or augment the remaining locations with their own layouts once the landing/start flow is preserved.

## Usage

```rust
use meshbbs::tmush::{
  TinyMushStore, PlayerRecord, CANONICAL_PUBLIC_START_ROOM_ID,
};

let store = TinyMushStore::open("./data/tinymush")?;
let mut player = PlayerRecord::new("alice", "Alice", CANONICAL_PUBLIC_START_ROOM_ID);
player.credits = 25;
store.put_player(player)?;

let fetched = store.get_player("alice")?;
assert_eq!(fetched.current_room, "town_square");
```

Unit tests in `tmush::storage` cover round-trip serialization and the canonical world seeding
path using `tempfile` directories.

## Next Steps

Phase&nbsp;2 will layer command parsing and session plumbing on top of this foundation. Expected
follow-ups include:

- Storing verb execution logs in the `logs` tree for auditability
- Appending NPC mail via `enqueue_mail`
- Adding migrations for player stats or world expansions
- Exposing metrics for TinyMUSH storage operations

Document updates should accompany structural changes so this page stays in sync with runtime
expectations.
