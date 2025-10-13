use meshbbs::tmush::types::PlayerRecord;
/// Integration tests for TinyMUSH admin commands
/// Tests: admin permissions, grant/revoke, admin listing
use meshbbs::tmush::TinyMushStoreBuilder;
use tempfile::TempDir;

#[test]
fn admin_check_new_player_not_admin() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    let player = PlayerRecord::new("alice", "Alice", "town_square");
    store.put_player(player.clone()).expect("save player");

    // New players should not be admins
    assert!(!store.is_admin("alice").expect("check admin"));
    assert_eq!(player.admin_level(), 0);
}

#[test]
fn admin_grant_and_check() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create admin and regular user
    let mut admin = PlayerRecord::new("sysop", "Sysop", "town_square");
    admin.grant_admin(3); // Level 3 = sysop
    store.put_player(admin).expect("save admin");

    let mut alice = PlayerRecord::new("alice", "Alice", "town_square");
    store.put_player(alice.clone()).expect("save alice");

    // Grant admin to alice
    store.grant_admin("sysop", "alice", 2).expect("grant admin");

    // Verify alice is now admin
    assert!(store.is_admin("alice").expect("check admin"));

    // Reload player and verify admin fields
    alice = store.get_player("alice").expect("get alice");
    assert!(alice.is_admin());
    assert_eq!(alice.admin_level(), 2);
}

#[test]
fn admin_grant_requires_admin() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create two regular users
    let alice = PlayerRecord::new("alice", "Alice", "town_square");
    let bob = PlayerRecord::new("bob", "Bob", "town_square");
    store.put_player(alice).expect("save alice");
    store.put_player(bob).expect("save bob");

    // Alice (non-admin) tries to grant admin to bob
    let result = store.grant_admin("alice", "bob", 1);

    // Should fail with PermissionDenied
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("permission denied") || err_msg.contains("Permission denied"));
}

#[test]
fn admin_revoke() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create sysop and admin user
    let mut sysop = PlayerRecord::new("sysop", "Sysop", "town_square");
    sysop.grant_admin(3);
    store.put_player(sysop).expect("save sysop");

    let mut alice = PlayerRecord::new("alice", "Alice", "town_square");
    alice.grant_admin(2);
    store.put_player(alice.clone()).expect("save alice");

    // Verify alice is admin
    assert!(store.is_admin("alice").expect("check admin"));

    // Revoke admin from alice
    store.revoke_admin("sysop", "alice").expect("revoke admin");

    // Verify alice is no longer admin
    assert!(!store.is_admin("alice").expect("check admin"));

    // Reload and verify
    alice = store.get_player("alice").expect("get alice");
    assert!(!alice.is_admin());
    assert_eq!(alice.admin_level(), 0);
}

#[test]
fn admin_revoke_requires_admin() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create admin and regular user
    let mut alice = PlayerRecord::new("alice", "Alice", "town_square");
    alice.grant_admin(2);
    store.put_player(alice).expect("save alice");

    let bob = PlayerRecord::new("bob", "Bob", "town_square");
    store.put_player(bob).expect("save bob");

    // Bob (non-admin) tries to revoke alice's admin
    let result = store.revoke_admin("bob", "alice");

    // Should fail
    assert!(result.is_err());
}

#[test]
fn admin_cannot_revoke_self() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create admin
    let mut alice = PlayerRecord::new("alice", "Alice", "town_square");
    alice.grant_admin(2);
    store.put_player(alice).expect("save alice");

    // Try to revoke own admin
    let result = store.revoke_admin("alice", "alice");

    // Should fail
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Cannot revoke your own"));
}

#[test]
fn admin_list_admins() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create mix of admins and regular users
    let mut sysop = PlayerRecord::new("sysop", "Sysop", "town_square");
    sysop.grant_admin(3);
    store.put_player(sysop).expect("save sysop");

    let mut alice = PlayerRecord::new("alice", "Alice", "town_square");
    alice.grant_admin(2);
    store.put_player(alice).expect("save alice");

    let bob = PlayerRecord::new("bob", "Bob", "town_square");
    store.put_player(bob).expect("save bob");

    let charlie = PlayerRecord::new("charlie", "Charlie", "town_square");
    store.put_player(charlie).expect("save charlie");

    // List admins
    let admins = store.list_admins().expect("list admins");

    // Should have exactly 3 admins (auto-seeded "admin" + sysop + alice)
    assert_eq!(admins.len(), 3);

    let admin_names: Vec<&str> = admins.iter().map(|p| p.username.as_str()).collect();
    assert!(
        admin_names.contains(&"admin"),
        "should have auto-seeded admin"
    );
    assert!(admin_names.contains(&"sysop"));
    assert!(admin_names.contains(&"alice"));
    assert!(!admin_names.contains(&"bob"));
    assert!(!admin_names.contains(&"charlie"));
}

#[test]
fn admin_levels() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create players with different admin levels
    let mut mod1 = PlayerRecord::new("mod1", "Moderator 1", "town_square");
    mod1.grant_admin(1); // Level 1 = moderator
    store.put_player(mod1.clone()).expect("save mod1");

    let mut admin1 = PlayerRecord::new("admin1", "Admin 1", "town_square");
    admin1.grant_admin(2); // Level 2 = admin
    store.put_player(admin1.clone()).expect("save admin1");

    let mut sysop1 = PlayerRecord::new("sysop1", "Sysop 1", "town_square");
    sysop1.grant_admin(3); // Level 3 = sysop
    store.put_player(sysop1.clone()).expect("save sysop1");

    // Verify levels
    assert_eq!(mod1.admin_level(), 1);
    assert_eq!(admin1.admin_level(), 2);
    assert_eq!(sysop1.admin_level(), 3);

    // All should be recognized as admins
    assert!(mod1.is_admin());
    assert!(admin1.is_admin());
    assert!(sysop1.is_admin());
}

#[test]
fn admin_require_admin_helper() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create admin and regular user
    let mut admin = PlayerRecord::new("admin", "Admin", "town_square");
    admin.grant_admin(2);
    store.put_player(admin).expect("save admin");

    let user = PlayerRecord::new("user", "User", "town_square");
    store.put_player(user).expect("save user");

    // require_admin should succeed for admin
    assert!(store.require_admin("admin").is_ok());

    // require_admin should fail for regular user
    assert!(store.require_admin("user").is_err());
}
