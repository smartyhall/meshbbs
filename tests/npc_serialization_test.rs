#![cfg(feature = "meshtastic-proto")]
// Test: Basic NPC serialization/deserialization

use meshbbs::tmush::types::{NpcFlag, NpcRecord};

#[test]
fn test_npc_basic_serialization() {
    let npc = NpcRecord::new("test_npc", "Test", "Tester", "A test NPC", "test_room")
        .with_dialog("greeting", "Hello")
        .with_flag(NpcFlag::Vendor);

    // Serialize
    let data = bincode::serialize(&npc).expect("serialize should work");
    eprintln!("Serialized NPC to {} bytes", data.len());

    // Deserialize
    let loaded: NpcRecord = bincode::deserialize(&data).expect("deserialize should work");

    // Verify
    assert_eq!(loaded.id, "test_npc");
    assert_eq!(loaded.name, "Test");
    assert_eq!(loaded.flags.len(), 1);
    assert_eq!(loaded.flags[0], NpcFlag::Vendor);
}

#[test]
fn test_npc_with_dialogue_tree() {
    use meshbbs::tmush::types::{DialogChoice, DialogNode};
    use std::collections::HashMap;

    let mut npc = NpcRecord::new("test_npc", "Test", "Tester", "A test NPC", "test_room")
        .with_dialog("greeting", "Hello")
        .with_flag(NpcFlag::Vendor);

    // Add dialogue tree (like seed_npc_dialogues_if_needed does)
    let mut tree = HashMap::new();
    tree.insert(
        "greeting".to_string(),
        DialogNode::new("Welcome!")
            .with_choice(DialogChoice::new("Tell me more").goto("more"))
            .with_choice(DialogChoice::new("Goodbye").exit()),
    );
    tree.insert(
        "more".to_string(),
        DialogNode::new("Here's more info...").with_choice(DialogChoice::new("Thanks").exit()),
    );

    npc.dialog_tree = tree;

    // Serialize
    let data = bincode::serialize(&npc).expect("serialize with dialog_tree should work");
    eprintln!("Serialized NPC with dialog_tree to {} bytes", data.len());

    // Deserialize
    let loaded: NpcRecord =
        bincode::deserialize(&data).expect("deserialize with dialog_tree should work");

    // Verify
    assert_eq!(loaded.id, "test_npc");
    assert_eq!(loaded.dialog_tree.len(), 2);
    assert!(loaded.dialog_tree.contains_key("greeting"));
    assert!(loaded.dialog_tree.contains_key("more"));
}
