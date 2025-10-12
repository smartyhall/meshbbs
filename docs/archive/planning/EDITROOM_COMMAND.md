# @EDITROOM Command Documentation

## Overview

The `@EDITROOM` command allows administrators to edit the long description of any room in the world, including seeded world rooms, housing rooms, and player-created rooms.

## Syntax

```
@EDITROOM <room_id> <new_description>
```

## Parameters

- **room_id**: The unique identifier for the room (e.g., `gazebo_landing`, `town_square`)
- **new_description**: The new long description text (max 500 characters)

## Examples

### Edit the Landing Gazebo

```
@EDITROOM gazebo_landing You stand in a beautiful octagonal gazebo with polished wooden railings. Soft mesh lanterns cast a warm glow over the area.
```

### Edit Town Square

```
@EDITROOM town_square A bustling plaza centered around a tall mesh beacon. Stone paths radiate outward to various districts of Old Towne Mesh.
```

### Edit a Custom Room

```
@EDITROOM my_custom_room_001 This is my custom room with a new description!
```

## Common Room IDs

The following world rooms are seeded by default:

| Room ID | Room Name | Purpose |
|---------|-----------|---------|
| `gazebo_landing` | Landing Gazebo | Tutorial starting point |
| `town_square` | Old Towne Square | Main hub |
| `city_hall_lobby` | City Hall Lobby | Government building entrance |
| `mayor_office` | Mayor's Office | Tutorial completion |
| `mesh_museum` | Mesh Museum | Educational area |
| `north_gate` | Northern Gate | World boundary |
| `south_market` | Southern Market | Shopping district |

## Permissions

**Current Status**: Alpha testing - any authenticated user can use this command for testing purposes.

**Future**: Will be restricted to users with admin/builder roles once the permission system is implemented.

## Output Format

On success, the command shows:
```
Room 'gazebo_landing' description updated by alice.

OLD:
Soft mesh lanterns line the gazebo railing while onboarding scripts hum to life.
Wizards finalize character details here before stepping into the public square.

NEW:
You stand in an octagonal gazebo with polished wooden railings. Soft mesh lanterns 
cast a warm glow, and a carved wooden sign reads 'Welcome to Old Towne Mesh!' 
Through the northern archway, you can see the bustling Town Square. This is a 
safe place to learn the basics - try typing LOOK to examine your surroundings, 
INVENTORY to check what you're carrying, or HELP to see available commands. 
When ready, head NORTH to begin your adventure!
```

## Error Handling

### Room Not Found

```
Room 'invalid_room' not found.

Common room IDs:
- gazebo_landing (Landing Gazebo)
- town_square (Old Towne Square)
- city_hall_lobby (City Hall Lobby)
- mayor_office (Mayor's Office)
- mesh_museum (Mesh Museum)
- north_gate (Northern Gate)
- south_market (Southern Market)
```

### Description Too Long

```
Description too long: 532 chars (max 500)
```

## Technical Details

### Database Storage

- Room descriptions are stored in the Sled database under `room:*` keys
- Changes are persisted immediately
- No undo/rollback functionality (yet)

### Character Limit

The 500-character limit is enforced to ensure:
- Reasonable message chunk sizes for Meshtastic
- Good UX (concise descriptions)
- Database efficiency

### Async Implementation

The command uses async storage operations via:
```rust
store.get_room_async(&room_id).await?;
store.put_room_async(room).await?;
```

This ensures the command doesn't block the Tokio runtime under load.

## Use Cases

### Alpha Testing

- **Test room descriptions**: Iterate on room text during playtest
- **Fix typos**: Quick corrections without rebuilding
- **Seasonal updates**: Change descriptions for events
- **A/B testing**: Try different descriptions to see what resonates

### Production

Once role-based permissions are added:
- **World builders**: Edit public world rooms
- **Game masters**: Update quest locations
- **Event coordinators**: Modify rooms for special events
- **Moderators**: Fix player-reported issues

## Related Commands

- `LOOK` / `L` - View current room description
- `@GETCONFIG` - View world configuration
- `@SETCONFIG` - Edit world configuration fields
- `DESCRIBE` - Edit description of your housing room (housing only)

## Future Enhancements

Potential improvements for future releases:

1. **Audit Log**: Track who edited what and when
2. **Rollback**: Undo recent changes
3. **Bulk Edit**: Edit multiple rooms at once
4. **Templates**: Apply standard formatting
5. **Validation**: Check for profanity, spam, etc.
6. **Preview**: See how description looks before saving
7. **Edit Short Desc**: Also edit the room's short name
8. **Edit Exits**: Modify exit descriptions

## Testing Checklist

When testing `@EDITROOM`:

- [ ] Edit a world room (gazebo_landing)
- [ ] Edit a housing room (requires housing instance)
- [ ] Try invalid room ID (should show error)
- [ ] Try description > 500 chars (should reject)
- [ ] Use LOOK to verify changes
- [ ] Log out and back in (verify persistence)
- [ ] Check database with different user (verify visibility)

## Implementation Notes

**File**: `src/tmush/commands.rs`

**Handler Function**: `handle_edit_room()`

**Key Features**:
- Validates room existence
- Enforces 500-char limit
- Shows before/after for confirmation
- Uses async storage operations
- Provides helpful error messages with common room IDs

**TODO**:
- Add role-based permission check
- Add edit history/audit log
- Add undo functionality
