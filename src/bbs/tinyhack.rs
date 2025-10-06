//! TinyHack v1 - a compact, ASCII-only roguelike mini-game designed for ~230-char messages.
//!
//! This module implements a small turn-based dungeon crawler with per-user save files.
//! It renders a complete snapshot each turn and accepts terse one/two-token commands.
//!
//! Persistence: JSON at `<data_dir>/tinyhack/<username>.json` using atomic write+rename.
//!
//! Notes:
//! - ASCII only (no emoji). Keep all lines well under 230 chars; final render is trimmed.
//! - Deterministic procgen using a stored seed per-save; grid defaults to 6x6.
//! - Designed to be called by BBS command processor while in SessionState::TinyHack.

use fs2::FileExt;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

// Note: Chunking is handled centrally by the BBS server when sending DM replies.

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoomKind {
    Empty,
    Monster(MonsterKind),
    Chest,
    LockedDoor,
    Trap,
    Vendor,
    Shrine,
    Fountain,
    Stairs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MonsterKind {
    Rat,
    Goblin,
    Slime,
    Skeleton,
    Orc,
    Mimic,
    Boss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub kind: RoomKind,
    /// Remaining monster HP if present; None if cleared
    pub mon_hp: Option<i32>,
    /// One-shot markers: trap sprung, shrine used, fountain used, chest looted, door opened
    pub used: bool,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            kind: RoomKind::Empty,
            mon_hp: None,
            used: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub defn: i32,
    pub lvl: u8,
    pub xp: i32,
    pub gold: i32,
    pub x: usize,
    pub y: usize,
    pub keys: i32,
    pub potions: i32,
    pub bombs: i32,
    pub scrolls: i32,
    pub weapon_lvl: i32,
    pub armor_lvl: i32,
    /// Lockpicks improve chances to pick locked doors. Backward-compatible default: 0.
    #[serde(default)]
    pub lockpicks: i32,
}

impl Player {
    fn new() -> Self {
        Player {
            hp: 10,
            max_hp: 10,
            atk: 2,
            defn: 1,
            lvl: 1,
            xp: 0,
            gold: 0,
            x: 0,
            y: 0,
            keys: 0,
            potions: 1,
            bombs: 0,
            scrolls: 0,
            weapon_lvl: 0,
            armor_lvl: 0,
            lockpicks: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub gid: u32,
    pub turn: u32,
    pub seed: u64,
    pub w: usize,
    pub h: usize,
    pub player: Player,
    pub map: Vec<Room>, // row-major h*w
    /// Whether the one-time intro has been shown to this player.
    /// Defaults to true for backward compatibility with old saves; new games set false.
    #[serde(default = "intro_shown_default_true")]
    pub intro_shown: bool,
    /// First-encounter hint flags (one-time). Backward-compatible defaults: false.
    #[serde(default)]
    pub seen_monster: bool,
    #[serde(default)]
    pub seen_chest: bool,
    #[serde(default)]
    pub seen_vendor: bool,
    #[serde(default)]
    pub seen_door: bool,
    #[serde(default)]
    pub seen_trap: bool,
    /// Visited rooms for fog-of-war mini-map. Backward-compatible default: empty.
    #[serde(default)]
    pub visited: Vec<bool>,
}

fn intro_shown_default_true() -> bool {
    true
}

impl GameState {
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    pub fn room(&self, x: usize, y: usize) -> &Room {
        &self.map[self.idx(x, y)]
    }
    pub fn room_mut(&mut self, x: usize, y: usize) -> &mut Room {
        let i = self.idx(x, y);
        &mut self.map[i]
    }
}

fn d6(rng: &mut StdRng) -> i32 {
    rng.gen_range(1..=6)
}

fn choose<'a>(rng: &mut StdRng, opts: &[&'a str]) -> &'a str {
    let i = rng.gen_range(0..opts.len());
    opts[i]
}

fn monster_stats(k: MonsterKind) -> (i32, i32, i32, i32) {
    match k {
        MonsterKind::Rat => (4, 1, 0, 1),
        MonsterKind::Goblin => (6, 2, 1, 2),
        MonsterKind::Slime => (8, 1, 2, 2),
        MonsterKind::Skeleton => (7, 3, 1, 3),
        MonsterKind::Orc => (10, 4, 2, 4),
        MonsterKind::Mimic => (8, 3, 2, 4),
        MonsterKind::Boss => (14, 5, 3, 6),
    }
}

fn clamp_ascii(s: String) -> String {
    s
}

fn ensure_dir(path: &Path) {
    let _ = std::fs::create_dir_all(path);
}

fn save_path(base_dir: &str, username: &str) -> PathBuf {
    Path::new(base_dir).join("tinyhack").join(format!(
        "{}.json",
        crate::validation::safe_filename(username)
    ))
}

fn write_json_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    ensure_dir(path.parent().unwrap_or(Path::new(".")));
    // Take an exclusive lock on the target path (create if missing)
    let lock_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(path)?;
    lock_file.lock_exclusive()?;
    // Create temp file
    let dir = path.parent().unwrap_or(Path::new("."));
    let base = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("save.json");
    let mut counter = 0u32;
    let tmp_path = loop {
        let cand = dir.join(format!(".{}.tmp-{}-{}", base, std::process::id(), counter));
        match OpenOptions::new().write(true).create_new(true).open(&cand) {
            Ok(mut tmp) => {
                tmp.write_all(content.as_bytes())?;
                let _ = tmp.flush();
                let _ = tmp.sync_all();
                break cand;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                counter = counter.saturating_add(1);
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }
    };
    std::fs::rename(&tmp_path, path)?;
    if let Ok(dirf) = File::open(dir) {
        let _ = dirf.sync_all();
    }
    drop(lock_file);
    Ok(())
}

fn read_json(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

fn farthest_from_start(w: usize, h: usize, sx: usize, sy: usize) -> (usize, usize) {
    let mut best = (sx, sy);
    let mut bestd: i32 = -1;
    for y in 0..h {
        for x in 0..w {
            let d = (x as i32 - sx as i32).abs() + (y as i32 - sy as i32).abs();
            if d > bestd {
                bestd = d;
                best = (x, y);
            }
        }
    }
    best
}

fn is_occupied_for_special(kind: &RoomKind) -> bool {
    // A special can only be placed on truly empty tiles (not start/exit; enforced by caller)
    !matches!(kind, RoomKind::Empty)
}

fn place_world(rng: &mut StdRng, w: usize, h: usize, map: &mut [Room]) {
    // Start at (0,0). Exit farthest.
    let (sx, sy) = (0usize, 0usize);
    let (ex, ey) = farthest_from_start(w, h, sx, sy);
    map[ey * w + ex] = Room {
        kind: RoomKind::Stairs,
        mon_hp: None,
        used: false,
    };
    // Vendor, shrine, fountain somewhere near start (not (0,0), not exit), don't overwrite each other or other content
    let mut placed = 0;
    while placed < 3 {
        let x = rng.gen_range(0..w);
        let y = rng.gen_range(0..h);
        let idx = y * w + x;
        if (x, y) != (sx, sy) && (x, y) != (ex, ey) && !is_occupied_for_special(&map[idx].kind) {
            let k = match placed {
                0 => RoomKind::Vendor,
                1 => RoomKind::Shrine,
                _ => RoomKind::Fountain,
            };
            map[idx] = Room {
                kind: k,
                mon_hp: None,
                used: false,
            };
            placed += 1;
        }
    }
    // Chests and traps scattered
    for _ in 0..(w + h) {
        let x = rng.gen_range(0..w);
        let y = rng.gen_range(0..h);
        let idx = y * w + x;
        if (x, y) != (sx, sy) && matches!(map[idx].kind, RoomKind::Empty) {
            map[idx] = Room {
                kind: RoomKind::Chest,
                mon_hp: None,
                used: false,
            };
        }
    }
    for _ in 0..(w) {
        let x = rng.gen_range(0..w);
        let y = rng.gen_range(0..h);
        let idx = y * w + x;
        // Traps should not overwrite specials, chests, or any non-empty tile
        if (x, y) != (sx, sy) && matches!(map[idx].kind, RoomKind::Empty) {
            map[idx] = Room {
                kind: RoomKind::Trap,
                mon_hp: None,
                used: false,
            };
        }
    }
    // Boss near exit: prefer an empty neighbor; else pick any empty non-start/non-exit
    if let Some((bx, by)) = [
        (ex.wrapping_sub(1), ey),
        (ex + 1, ey),
        (ex, ey.wrapping_sub(1)),
        (ex, ey + 1),
    ]
    .into_iter()
    .filter_map(|(x, y)| if x < w && y < h { Some((x, y)) } else { None })
    .find(|(x, y)| matches!(map[y * w + x].kind, RoomKind::Empty))
    {
        map[by * w + bx] = Room {
            kind: RoomKind::Monster(MonsterKind::Boss),
            mon_hp: Some(monster_stats(MonsterKind::Boss).0),
            used: false,
        };
    } else {
        for y in 0..h {
            for x in 0..w {
                if (x, y) != (sx, sy)
                    && (x, y) != (ex, ey)
                    && matches!(map[y * w + x].kind, RoomKind::Empty)
                {
                    map[y * w + x] = Room {
                        kind: RoomKind::Monster(MonsterKind::Boss),
                        mon_hp: Some(monster_stats(MonsterKind::Boss).0),
                        used: false,
                    };
                    break;
                }
            }
        }
    }
    // Populate other monsters roughly by distance bands
    for y in 0..h {
        for x in 0..w {
            if (x, y) == (sx, sy)
                || matches!(
                    map[y * w + x].kind,
                    RoomKind::Stairs
                        | RoomKind::Vendor
                        | RoomKind::Shrine
                        | RoomKind::Fountain
                        | RoomKind::Chest
                        | RoomKind::Trap
                        | RoomKind::Monster(_)
                )
            {
                continue;
            }
            let d = (x as i32 - sx as i32).abs() + (y as i32 - sy as i32).abs();
            let mk = if d <= 2 {
                if rng.gen_bool(0.4) {
                    Some(MonsterKind::Rat)
                } else {
                    None
                }
            } else if d <= 4 {
                Some(if rng.gen_bool(0.5) {
                    MonsterKind::Goblin
                } else {
                    MonsterKind::Slime
                })
            } else {
                Some(if rng.gen_bool(0.5) {
                    MonsterKind::Skeleton
                } else {
                    MonsterKind::Orc
                })
            };
            if let Some(k) = mk {
                map[y * w + x] = Room {
                    kind: RoomKind::Monster(k),
                    mon_hp: Some(monster_stats(k).0),
                    used: false,
                };
            }
        }
    }
    // Sprinkle a few locked doors only on empty tiles; bounded attempts to avoid infinite loops
    let mut ld = 0;
    let mut attempts = w * h * 4;
    while ld < 3 && attempts > 0 {
        attempts -= 1;
        let x = rng.gen_range(0..w);
        let y = rng.gen_range(0..h);
        let idx = y * w + x;
        if (x, y) != (sx, sy) && matches!(map[idx].kind, RoomKind::Empty) {
            map[idx] = Room {
                kind: RoomKind::LockedDoor,
                mon_hp: None,
                used: false,
            };
            ld += 1;
        }
    }
}

fn new_game(seed: u64, w: usize, h: usize) -> GameState {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = vec![Room::default(); w * h];
    place_world(&mut rng, w, h, &mut map);
    let mut visited = vec![false; w * h];
    // Mark starting position as visited
    visited[0] = true;
    GameState {
        gid: rng.gen(),
        turn: 1,
        seed,
        w,
        h,
        player: Player::new(),
        map,
        intro_shown: false,
        seen_monster: false,
        seen_chest: false,
        seen_vendor: false,
        seen_door: false,
        seen_trap: false,
        visited,
    }
}

fn describe_room(gs: &GameState) -> String {
    let r = gs.room(gs.player.x, gs.player.y);
    match r.kind {
        RoomKind::Empty => "You stand in a dim corridor. Dust hangs in the air.".to_string(),
        RoomKind::Monster(k) => {
            let hp = r.mon_hp.unwrap_or(0);
            let name = match k {
                MonsterKind::Rat => "a giant rat",
                MonsterKind::Goblin => "a goblin",
                MonsterKind::Slime => "a slithering slime",
                MonsterKind::Skeleton => "a rattling skeleton",
                MonsterKind::Orc => "an orc bruiser",
                MonsterKind::Mimic => "a suspicious chest (mimic)",
                MonsterKind::Boss => "the dungeon boss",
            };
            format!(
                "You are not alone-{} lurks here (HP {}). It eyes you hungrily.",
                name, hp
            )
        }
        RoomKind::Chest => {
            "A dusty chest rests against the wall, its latch brittle with age.".to_string()
        }
        RoomKind::LockedDoor => {
            "A heavy locked door bars the way; something valuable may lie beyond.".to_string()
        }
        RoomKind::Trap => {
            if r.used {
                "Scorched marks and sprung mechanisms suggest a trap was here.".to_string()
            } else {
                "The floor tiles look uneven-a trap may be set in this hall.".to_string()
            }
        }
        RoomKind::Vendor => {
            "A robed vendor watches from a tiny stall, wares neatly arranged.".to_string()
        }
        RoomKind::Shrine => {
            if r.used {
                "An ancient shrine, its magic spent, stands silent.".to_string()
            } else {
                "An ancient shrine hums softly, inviting a humble approach.".to_string()
            }
        }
        RoomKind::Fountain => {
            if r.used {
                "A cracked fountain sits dry; its healing waters spent.".to_string()
            } else {
                "A cracked stone fountain trickles-its waters seem restorative.".to_string()
            }
        }
        RoomKind::Stairs => "A narrow stairwell leads out of this cursed place.".to_string(),
    }
}

fn exits(gs: &GameState) -> (bool, bool, bool, bool) {
    // N,S,W,E
    let (x, y) = (gs.player.x, gs.player.y);
    (y > 0, y + 1 < gs.h, x > 0, x + 1 < gs.w)
}

fn compute_options(gs: &GameState) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let r = gs.room(gs.player.x, gs.player.y);
    // movement
    let (n, s, w, e) = exits(gs);
    if n {
        v.push("N".into());
    }
    if s {
        v.push("S".into());
    }
    if w {
        v.push("W".into());
    }
    if e {
        v.push("E".into());
    }
    // context
    match r.kind {
        RoomKind::Monster(_) => {
            v.push("A".into());
            v.push("U P".into());
            v.push("C F".into());
        }
        RoomKind::Chest => {
            v.push("T".into());
        }
        RoomKind::LockedDoor => {
            v.push("O".into());
            v.push("PICK".into());
            v.push("U B".into());
        }
        RoomKind::Vendor => {
            v.push("BUY P".into());
            v.push("BUY B".into());
            v.push("BUY S".into());
            v.push("BUY K".into());
            v.push("BUY H".into());
            v.push("BUY L".into());
            v.push("UPG W".into());
            v.push("UPG A".into());
            v.push("MYST".into());
            v.push("LEAVE".into());
        }
        _ => {}
    }
    v.push("R".into());
    v.push("I".into());
    v.push("M".into());
    v.push("?".into());
    v.push("Q".into());
    v
}

fn status_line(gs: &GameState) -> String {
    // Compact status for top line (Option 5)
    let need = level_threshold(gs.player.lvl);
    format!(
        "L{} H{}/{} X{}/{} G{}",
        gs.player.lvl, gs.player.hp, gs.player.max_hp, gs.player.xp, need, gs.player.gold
    )
}

fn full_status_line(gs: &GameState) -> String {
    // More detailed status for Inspect: include coords and gear/inventory
    format!(
        "L{} H{}/{} X{}/{} G{} @{},{} ATK{} DEF{} Inv:P{} K{} B{} S{}",
        gs.player.lvl,
        gs.player.hp,
        gs.player.max_hp,
        gs.player.xp,
        level_threshold(gs.player.lvl),
        gs.player.gold,
        gs.player.x,
        gs.player.y,
        gs.player.atk,
        gs.player.defn,
        gs.player.potions,
        gs.player.keys,
        gs.player.bombs,
        gs.player.scrolls
    )
}

fn help_text() -> &'static str {
    "TinyHack Commands:\n\
N)orth S)outh E)ast W)est - move between rooms\n\
A)ttack - melee strike (crits possible); monster may retaliate\n\
U)se P)otion - heal; U)se B)omb - blast door/foe (6 dmg)\n\
C)ast F)ireball - burn a foe (5 dmg)\n\
T)ake - loot a chest (gold, keys, items)\n\
O)pen - unlock a locked door (needs a key)\n\
PICK - try to pick a locked door (chance↑ with lockpicks; some risk)\n\
R)est - recover a little HP (risk ambush)\n\
I)nspect - show status and options again\n\
M)ap - show mini-map with fog of war\n\
B)ack - return to BBS menu; Q)uit - leave TinyHack\n\
Vendor (at stall): BUY P/B/S/K/H/L, UPG W/A, MYST, LEAVE\n\
Tip: 'U P'/'UP' uses a potion; 'U B'/'UB' uses a bomb; 'C F'/'CF' casts Fireball.\n\
Also accepted: USE POTION/BOMB and CAST FIREBALL.\n\
Goal: Find the Stairs and escape the dungeon."
}

/// Compact, ASCII-only welcome message for first-time TinyHack start.
pub fn welcome_message() -> &'static str {
    "Welcome to TinyHack: a compact, turn-based dungeon crawl. Explore a maze of rooms, meet monsters, dodge traps, visit vendors, hoard treasure, grow stronger, and race for the Stairs."
}

pub fn render(gs: &GameState) -> String {
    let mut msg = String::new();
    msg.push_str(&status_line(gs));
    msg.push('\n');
    let mut room = describe_room(gs);
    // Append exits summary
    let (n, s, w, e) = exits(gs);
    let mut ex: Vec<&str> = Vec::new();
    if n {
        ex.push("N");
    }
    if s {
        ex.push("S");
    }
    if w {
        ex.push("W");
    }
    if e {
        ex.push("E");
    }
    if !ex.is_empty() {
        room.push_str(" Exits ");
        room.push_str(&ex.join(","));
        room.push('.');
    }
    msg.push_str(&room);
    msg.push('\n');
    let opts = compute_options(gs).join(" ");
    msg.push_str("Opts: ");
    msg.push_str(&opts);
    msg.push('\n');
    // Do not include an in-game prompt; the server appends the session prompt on the last chunk.
    // Do not locally truncate: server will chunk and append the DM prompt on the last part.
    msg
}

/// Render mini-map with fog of war. Compact ASCII, ~150-180 chars.
pub fn render_map(gs: &GameState) -> String {
    // Ensure visited vector is initialized (backward compatibility)
    let has_visited = !gs.visited.is_empty();

    let mut msg = String::new();
    // Compact status line
    msg.push_str(&format!(
        "L{} HP{}/{} G{} K{} P{}\n",
        gs.player.lvl,
        gs.player.hp,
        gs.player.max_hp,
        gs.player.gold,
        gs.player.keys,
        gs.player.potions
    ));

    // Build 6x6 grid
    for y in 0..gs.h {
        for x in 0..gs.w {
            let idx = gs.idx(x, y);
            let is_player = gs.player.x == x && gs.player.y == y;
            let is_visited = has_visited && idx < gs.visited.len() && gs.visited[idx];

            if is_player {
                msg.push('@');
            } else if !is_visited {
                msg.push('#');
            } else {
                // Show room type for visited rooms
                let room = &gs.map[idx];
                let symbol = match room.kind {
                    RoomKind::Empty => '.',
                    RoomKind::Monster(_) => {
                        if room.mon_hp.unwrap_or(0) > 0 {
                            'M'
                        } else {
                            'X' // defeated
                        }
                    }
                    RoomKind::Chest => {
                        if room.used {
                            '.'
                        } else {
                            'C'
                        }
                    }
                    RoomKind::LockedDoor => {
                        if room.used {
                            '.'
                        } else {
                            'D'
                        }
                    }
                    RoomKind::Trap => {
                        if room.used {
                            '.'
                        } else {
                            'T'
                        }
                    }
                    RoomKind::Vendor => 'V',
                    RoomKind::Shrine => {
                        if room.used {
                            '.'
                        } else {
                            'H'
                        }
                    }
                    RoomKind::Fountain => {
                        if room.used {
                            '.'
                        } else {
                            'F'
                        }
                    }
                    RoomKind::Stairs => 'S',
                };
                msg.push(symbol);
            }

            // Add space between columns except last
            if x < gs.w - 1 {
                msg.push(' ');
            }
        }
        msg.push('\n');
    }

    // Compact legend - only show symbols currently on map
    msg.push_str("@=You #=Fog .=Clear\n");
    msg.push_str("M=Mon X=Dead C=Chest\n");
    msg.push_str("D=Door V=Vendor S=Exit\n");

    msg
}

fn level_threshold(lvl: u8) -> i32 {
    match lvl {
        1 => 5,
        2 => 12,
        3 => 20,
        4 => 30,
        _ => 42 + ((lvl as i32 - 5) * 12),
    }
}

fn try_level_up(p: &mut Player, rng: &mut StdRng) {
    let need = level_threshold(p.lvl);
    if p.xp >= need {
        p.lvl += 1;
        p.max_hp += 2;
        p.hp = p.max_hp;
        if rng.gen_bool(0.5) {
            p.atk += 1;
        } else {
            p.defn += 1;
        }
    }
}

fn do_attack(gs: &mut GameState, rng: &mut StdRng) -> String {
    let (x, y) = (gs.player.x, gs.player.y);
    // Copy needed player stats before mutable borrow
    let p_atk = gs.player.atk;
    let p_def = gs.player.defn;
    let r = gs.room_mut(x, y);
    let (mk, mon_hp) = match r.kind {
        RoomKind::Monster(k) => (k, r.mon_hp.unwrap_or(0)),
        _ => {
            return "No target.\n".into();
        }
    };
    if mon_hp <= 0 {
        r.mon_hp = None;
        r.kind = RoomKind::Empty;
        return "No target.\n".into();
    }
    // Player hit
    let (_, matk, mdef, _mxp) = monster_stats(mk);
    let mut pd = (p_atk + d6(rng) - mdef).max(1);
    let mut crit = false;
    if pd >= p_atk + 6 - mdef {
        pd += 2;
        crit = true;
    } // simple crit on high roll
    let mhp = mon_hp - pd;
    r.mon_hp = Some(mhp);
    let verb = choose(rng, &["strike", "slash", "jab", "smash", "lunge"]);
    let mut out = format!("You {} for {}. ", verb, pd);
    if crit {
        out.push_str("Critical! ");
    }
    if mhp <= 0 {
        r.mon_hp = None;
        r.kind = RoomKind::Empty;
        let xp = monster_stats(mk).3;
        let g = rng.gen_range(1..=3);
        gs.player.xp += xp;
        gs.player.gold += g;
        try_level_up(&mut gs.player, rng);
        let kp = choose(
            rng,
            &[
                "The foe collapses.",
                "The enemy falls.",
                "Down it goes.",
                "You prevail.",
                "The monster is defeated.",
            ],
        );
        out.push_str(kp);
        out.push_str(&format!(" (+{} XP, +{}g). ", xp, g));
    } else {
        // Monster retaliates
        let mut md = (matk + d6(rng) - p_def).max(1);
        if md >= matk + 6 - p_def {
            md += 2;
        }
        gs.player.hp -= md;
        let ret = choose(
            rng,
            &[
                "It retaliates for {}.",
                "It strikes back for {}.",
                "It claws you for {}.",
                "It bites for {}.",
                "It counters for {}.",
            ],
        );
        let line = ret.replacen("{}", &md.to_string(), 1);
        out.push_str(&line);
    }
    if gs.player.hp <= 0 {
        return death_text(gs);
    }
    out.push('\n');
    out
}

fn use_potion(gs: &mut GameState) -> String {
    if gs.player.potions <= 0 {
        return "No potions.\n".into();
    }
    gs.player.potions -= 1;
    gs.player.hp = (gs.player.hp + 6).min(gs.player.max_hp);
    "You quaff a potion; warmth spreads as wounds knit.\n".into()
}

fn use_bomb(gs: &mut GameState, rng: &mut StdRng) -> String {
    if gs.player.bombs <= 0 {
        return "No bombs.\n".into();
    }
    gs.player.bombs -= 1;
    let x = gs.player.x;
    let y = gs.player.y;
    let r = gs.room_mut(x, y);
    match r.kind {
        RoomKind::LockedDoor => {
            r.kind = RoomKind::Empty;
            r.used = true;
            gs.player.gold += 3;
            gs.player.xp += 1;
            "You set the charge-wood splinters; behind it, a small stash (+3g, +1 XP).\n".into()
        }
        RoomKind::Monster(mk) => {
            let hp = r.mon_hp.unwrap_or(0) - 6;
            if hp <= 0 {
                r.kind = RoomKind::Empty;
                r.mon_hp = None;
                let xp = monster_stats(mk).3;
                let g = rng.gen_range(1..=3);
                gs.player.xp += xp;
                gs.player.gold += g;
                try_level_up(&mut gs.player, rng);
                let msg = choose(
                    rng,
                    &[
                        "The blast hurls the foe aside.",
                        "The explosion ends it.",
                        "Shrapnel finishes the monster.",
                    ],
                );
                format!("{} (+{} XP, +{}g).\n", msg, xp, g)
            } else {
                r.mon_hp = Some(hp);
                let msg = choose(
                    rng,
                    &[
                        "The blast staggers your foe.",
                        "Shrapnel tears into it.",
                        "The explosion rocks it.",
                    ],
                );
                format!("{}\n", msg)
            }
        }
        _ => "The fuse sputters out harmlessly.\n".into(),
    }
}

fn cast_fireball(gs: &mut GameState, rng: &mut StdRng) -> String {
    if gs.player.scrolls <= 0 {
        return "No scrolls.\n".into();
    }
    let x = gs.player.x;
    let y = gs.player.y;
    // Inspect first without holding mutable borrow
    let mk_opt = match gs.room(x, y).kind {
        RoomKind::Monster(mk) => Some(mk),
        _ => None,
    };
    if mk_opt.is_none() {
        return "Nothing to burn.\n".into();
    }
    gs.player.scrolls -= 1;
    let r = gs.room_mut(x, y);
    // Safe: r.kind is Monster
    let hp = r.mon_hp.unwrap_or(0) - 5;
    if hp <= 0 {
        r.kind = RoomKind::Empty;
        r.mon_hp = None;
        let mk = mk_opt.unwrap();
        let xp = monster_stats(mk).3;
        let g = rng.gen_range(1..=3);
        gs.player.xp += xp;
        gs.player.gold += g;
        try_level_up(&mut gs.player, rng);
        let msg = choose(
            rng,
            &[
                "You conjure flame-your foe is reduced to ash.",
                "An inferno engulfs the monster.",
                "Searing fire ends the fight.",
            ],
        );
        format!("{} (+{} XP, +{}g).\n", msg, xp, g)
    } else {
        r.mon_hp = Some(hp);
        let msg = choose(
            rng,
            &[
                "A gout of flame scorches your foe.",
                "Fire sears the enemy.",
                "Your spell burns it.",
            ],
        );
        format!("{}\n", msg)
    }
}

fn do_take(gs: &mut GameState, rng: &mut StdRng) -> String {
    let r = gs.room_mut(gs.player.x, gs.player.y);
    match r.kind {
        RoomKind::Chest => {
            r.kind = RoomKind::Empty;
            let roll = rng.gen_range(0..6);
            match roll {
                0 | 1 => {
                    gs.player.gold += 4;
                    "You pry the chest open and pocket 4 gold.\n".into()
                }
                2 => {
                    gs.player.keys += 1;
                    "Inside: a small brass key.\n".into()
                }
                3 => {
                    gs.player.potions += 1;
                    "A stoppered vial-one healing potion.\n".into()
                }
                4 => {
                    gs.player.bombs += 1;
                    "You find a compact bomb, carefully wrapped.\n".into()
                }
                _ => {
                    gs.player.scrolls += 1;
                    "A brittle scroll crackles with latent fire.\n".into()
                }
            }
        }
        _ => "There is nothing here to take.\n".into(),
    }
}

fn do_open(gs: &mut GameState) -> String {
    let x = gs.player.x;
    let y = gs.player.y;
    let is_locked = matches!(gs.room(x, y).kind, RoomKind::LockedDoor);
    if !is_locked {
        return "Nothing to open.\n".into();
    }
    if gs.player.keys <= 0 {
        return "No keys.\n".into();
    }
    gs.player.keys -= 1;
    let r = gs.room_mut(x, y);
    r.kind = RoomKind::Empty;
    r.used = true;
    gs.player.xp += 1;
    gs.player.gold += 3;
    "You turn the key; the door swings free. Behind it, a small stash (+3g, +1 XP).\n".into()
}

fn do_rest(gs: &mut GameState, rng: &mut StdRng) -> String {
    // 25% chance of ambush if a monster exists here (wander in)
    let mut out = String::new();
    gs.player.hp = (gs.player.hp + 3).min(gs.player.max_hp);
    if rng.gen_bool(0.25) {
        // Create a small rat ambush
        let (mhp, matk, _mdef, _mxp) = monster_stats(MonsterKind::Rat);
        let mut md = (matk + d6(rng) - gs.player.defn).max(1);
        if md >= matk + 6 - gs.player.defn {
            md += 2;
        }
        gs.player.hp -= md;
        out.push_str(&format!("An ambush! A rat nips you for {}. ", md));
        if gs.player.hp <= 0 {
            return death_text(gs);
        }
        // Place a rat in room
        let r = gs.room_mut(gs.player.x, gs.player.y);
        r.kind = RoomKind::Monster(MonsterKind::Rat);
        r.mon_hp = Some(mhp);
    }
    out.push_str("You catch your breath and tend your gear.\n");
    out
}

fn do_pick_lock(gs: &mut GameState, rng: &mut StdRng) -> String {
    let x = gs.player.x;
    let y = gs.player.y;
    let is_locked = matches!(gs.room(x, y).kind, RoomKind::LockedDoor);
    if !is_locked {
        return "Nothing to pick.\n".into();
    }
    // Base 40% + level*5% + lockpicks*10%, capped at 90%
    let mut chance = 0.40 + (gs.player.lvl as f64) * 0.05 + (gs.player.lockpicks as f64) * 0.10;
    if chance > 0.90 {
        chance = 0.90;
    }
    let roll: f64 = rng.gen();
    if gs.player.lockpicks > 0 {
        gs.player.lockpicks -= 1;
    }
    if roll < chance {
        let r = gs.room_mut(x, y);
        r.kind = RoomKind::Empty;
        r.used = true;
        gs.player.gold += 2;
        gs.player.xp += 1;
        "You finesse the tumblers-click. The door yields (+2g, +1 XP).\n".into()
    } else if rng.gen_bool(0.5) {
        let dmg = rng.gen_range(1..=3);
        gs.player.hp -= dmg;
        if gs.player.hp <= 0 {
            return death_text(gs);
        }
        format!("A needle trap snaps! You take {}.\n", dmg)
    } else {
        let (mhp, _matk, _mdef, _mxp) = monster_stats(MonsterKind::Rat);
        let r = gs.room_mut(x, y);
        r.kind = RoomKind::Monster(MonsterKind::Rat);
        r.mon_hp = Some(mhp);
        "Your clumsy attempt draws a rat.\n".into()
    }
}

fn do_move(gs: &mut GameState, dir: char) -> String {
    let (x, y) = (gs.player.x, gs.player.y);
    let (nx, ny) = match dir {
        'N' if y > 0 => (x, y - 1),
        'S' if y + 1 < gs.h => (x, y + 1),
        'W' if x > 0 => (x - 1, y),
        'E' if x + 1 < gs.w => (x + 1, y),
        _ => {
            return "Can't move.\n".into();
        }
    };
    if matches!(gs.room(nx, ny).kind, RoomKind::LockedDoor) {
        return "A locked door bars your way. Try O (key), PICK, or U B.\n".into();
    }
    gs.player.x = nx;
    gs.player.y = ny;

    // Mark new room as visited
    let idx = gs.idx(nx, ny);
    if !gs.visited.is_empty() && idx < gs.visited.len() {
        gs.visited[idx] = true;
    }

    String::new()
}

fn vendor_prices() -> (i32, i32, i32) {
    (6, 8, 10)
} // P,B,S

fn handle_vendor(gs: &mut GameState, cmd: &str) -> Option<String> {
    if !matches!(gs.room(gs.player.x, gs.player.y).kind, RoomKind::Vendor) {
        return None;
    }
    let up = cmd.trim().to_uppercase();
    if up.starts_with("BUY ") {
        let item = up.split_whitespace().nth(1).unwrap_or("");
        let (pp, bp, sp) = vendor_prices();
        match item {
            "P" => {
                if gs.player.gold >= pp {
                    gs.player.gold -= pp;
                    gs.player.potions += 1;
                    return Some("Bought potion.\n".into());
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            "B" => {
                if gs.player.gold >= bp {
                    gs.player.gold -= bp;
                    gs.player.bombs += 1;
                    return Some("Bought bomb.\n".into());
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            "S" => {
                if gs.player.gold >= sp {
                    gs.player.gold -= sp;
                    gs.player.scrolls += 1;
                    return Some("Bought scroll.\n".into());
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            "K" => {
                let cost = 6;
                if gs.player.gold >= cost {
                    gs.player.gold -= cost;
                    gs.player.keys += 1;
                    return Some("Bought key.\n".into());
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            "H" => {
                let cost = 6;
                if gs.player.gold >= cost {
                    gs.player.gold -= cost;
                    gs.player.hp = gs.player.max_hp;
                    return Some("Bought bandages; you feel restored.\n".into());
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            "L" => {
                let cost = 5;
                if gs.player.gold >= cost {
                    gs.player.gold -= cost;
                    gs.player.lockpicks += 1;
                    return Some("Bought lockpick.\n".into());
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            _ => {
                return Some("Usage: BUY P|B|S|K|H|L\n".into());
            }
        }
    } else if up.starts_with("UPG ") {
        let what = up.split_whitespace().nth(1).unwrap_or("");
        match what {
            "W" => {
                let cost = 10 + gs.player.weapon_lvl * 4;
                if gs.player.gold >= cost {
                    gs.player.gold -= cost;
                    gs.player.weapon_lvl += 1;
                    gs.player.atk += 1;
                    return Some(format!(
                        "Upgraded weapon to +{} (+1 ATK) for {}g.\n",
                        gs.player.weapon_lvl, cost
                    ));
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            "A" => {
                let cost = 10 + gs.player.armor_lvl * 4;
                if gs.player.gold >= cost {
                    gs.player.gold -= cost;
                    gs.player.armor_lvl += 1;
                    gs.player.defn += 1;
                    return Some(format!(
                        "Upgraded armor to +{} (+1 DEF) for {}g.\n",
                        gs.player.armor_lvl, cost
                    ));
                } else {
                    return Some("Not enough gold.\n".into());
                }
            }
            _ => {
                return Some("Usage: UPG W|A\n".into());
            }
        }
    } else if up == "MYST" {
        let cost = 8;
        if gs.player.gold < cost {
            return Some("Not enough gold.\n".into());
        }
        gs.player.gold -= cost;
        // Mystery grab bag
        let roll = rand::random::<u8>() % 7;
        let msg = match roll {
            0 => {
                gs.player.potions += 1;
                "Mystery: a potion!\n"
            }
            1 => {
                gs.player.scrolls += 1;
                "Mystery: a scroll!\n"
            }
            2 => {
                gs.player.bombs += 1;
                "Mystery: a bomb!\n"
            }
            3 => {
                gs.player.keys += 1;
                "Mystery: a small brass key!\n"
            }
            4 => {
                gs.player.lockpicks += 1;
                "Mystery: a lockpick!\n"
            }
            5 => {
                gs.player.xp += 1;
                "Mystery: a training tip (+1 XP).\n"
            }
            _ => {
                gs.player.gold += 5;
                "Mystery: a handful of coins (+5g).\n"
            }
        };
        return Some(msg.into());
    } else if up == "LEAVE" {
        return Some("You leave the vendor.\n".into());
    }
    None
}

fn win_text(gs: &GameState) -> String {
    clamp_ascii(format!(
        "TH g{} WIN t{} LVL{} XP{} G{}\nYou escape the Tiny Dungeon. Congrats!\n",
        gs.gid, gs.turn, gs.player.lvl, gs.player.xp, gs.player.gold
    ))
}

fn death_text(gs: &GameState) -> String {
    clamp_ascii(format!(
        "TH g{} RIP t{} L{},{}\nYou fall in battle. Final LVL{} XP{} G{}.\n",
        gs.gid, gs.turn, gs.player.x, gs.player.y, gs.player.lvl, gs.player.xp, gs.player.gold
    ))
}

fn on_enter_tile(gs: &mut GameState, rng: &mut StdRng) -> Option<String> {
    let (x, y) = (gs.player.x, gs.player.y);
    let (kind, used) = {
        let r = gs.room(x, y);
        (r.kind, r.used)
    };
    let mut hint: Option<String> = None;
    match kind {
        RoomKind::Monster(_) if !gs.seen_monster => {
            gs.seen_monster = true;
            hint = Some("Hint: A)ttack, U P potion, C F fireball.\n".into());
        }
        RoomKind::Chest if !gs.seen_chest => {
            gs.seen_chest = true;
            hint = Some("Hint: T)ake to loot chests-keys, gear, or gold.\n".into());
        }
        RoomKind::Vendor if !gs.seen_vendor => {
            gs.seen_vendor = true;
            hint = Some("Hint: At vendor: BUY P|B|S|K|H|L, UPG W|A, MYST.\n".into());
        }
        RoomKind::LockedDoor if !gs.seen_door => {
            gs.seen_door = true;
            hint =
                Some("Hint: Doors: O)pen with key, PICK to lockpick (risk), or U B bomb.\n".into());
        }
        RoomKind::Trap if !gs.seen_trap && !used => { /* will hint after trigger */ }
        _ => {}
    }
    match (kind, used) {
        (RoomKind::Trap, false) => {
            let dmg = rng.gen_range(1..=4);
            gs.player.hp -= dmg;
            {
                let r = gs.room_mut(x, y);
                r.used = true;
            }
            if gs.player.hp <= 0 {
                return Some(death_text(gs));
            }
            let mut s = format!("A hidden mechanism snaps-needles bite for {}.\n", dmg);
            if !gs.seen_trap {
                gs.seen_trap = true;
                s.push_str("Hint: Watch for uneven tiles; R)est to recover a bit.\n");
            }
            Some(s)
        }
        (RoomKind::Shrine, false) => {
            {
                let r = gs.room_mut(x, y);
                r.used = true;
            }
            if rand::random::<bool>() {
                gs.player.atk += 1;
                Some("You kneel; a warm light steels your arm (+1 ATK).\n".into())
            } else {
                gs.player.defn += 1;
                Some("You whisper a prayer; resolve hardens (+1 DEF).\n".into())
            }
        }
        (RoomKind::Fountain, false) => {
            {
                let r = gs.room_mut(x, y);
                r.used = true;
            }
            gs.player.hp = gs.player.max_hp;
            Some("You drink; cool water mends every ache.\n".into())
        }
        (RoomKind::Stairs, _) => Some(win_text(gs)),
        _ => hint,
    }
}

fn parse_cmd(raw: &str) -> (String, String) {
    let up = raw.trim().to_uppercase();
    // Normalize a variety of question mark glyphs to ASCII '?'
    // Accept if the input is solely one or more question marks (any supported glyph) and/or whitespace
    let mut only_qmarks = true;
    for ch in up.chars() {
        if !(ch.is_whitespace()
            || ch == '?'
            || ch == '\u{FF1F}' /* FULLWIDTH QUESTION MARK */
            || ch == '\u{00BF}' /* INVERTED QUESTION MARK */
            || ch == '\u{FE56}' /* SMALL QUESTION MARK */
            || ch == '\u{061F}'/* ARABIC QUESTION MARK */)
        {
            only_qmarks = false;
            break;
        }
    }
    if up == "?" || only_qmarks {
        return ("?".into(), String::new());
    }
    // Some clients might send stray punctuation; accept a single '?' token even if surrounded by spaces
    if up.contains('?') && up.chars().all(|c| c == '?' || c.is_whitespace()) {
        return ("?".into(), String::new());
    }

    // Tokenize by whitespace, but also support compact forms (e.g., "UP", "UB", "CF") and common full words
    let tokens: Vec<&str> = up.split_whitespace().collect();
    let merged = tokens.join("");

    // Compact and full-word aliases for common two-letter commands
    match merged.as_str() {
        // Use Potion
        "UP" | "USEPOTION" | "POTION" | "DRINK" => return ("U".into(), "P".into()),
        // Use Bomb
        "UB" | "USEBOMB" | "BOMB" => return ("U".into(), "B".into()),
        // Cast Fireball
        "CF" | "CASTF" | "CASTFIREBALL" | "FIREBALL" => return ("C".into(), "F".into()),
        _ => {}
    }

    // Two-token variants like "USE POTION", "USE BOMB", "CAST FIREBALL"
    if tokens.len() >= 2 {
        let t0 = tokens[0];
        let t1 = tokens[1];
        if (t0 == "U" || t0 == "USE") && (t1 == "P" || t1 == "POTION") {
            return ("U".into(), "P".into());
        }
        if (t0 == "U" || t0 == "USE") && (t1 == "B" || t1 == "BOMB") {
            return ("U".into(), "B".into());
        }
        if (t0 == "C" || t0 == "CAST") && (t1 == "F" || t1 == "FIREBALL") {
            return ("C".into(), "F".into());
        }
    }

    // Default: first token is op, second token (if any) is arg
    let mut it = up.split_whitespace();
    let op = it.next().unwrap_or("").to_string();
    let arg = it.next().unwrap_or("").to_string();
    (op, arg)
}

pub fn handle_turn(mut gs: GameState, cmd: &str) -> (GameState, String) {
    let mut rng = StdRng::seed_from_u64(
        gs.seed
            .wrapping_add(gs.turn as u64)
            .wrapping_add((gs.player.x as u64) << 8)
            .wrapping_add((gs.player.y as u64) << 16),
    );
    let (op, arg) = parse_cmd(cmd);
    let mut out = String::new();
    match op.as_str() {
        "N" | "S" | "E" | "W" => {
            out.push_str(&do_move(&mut gs, op.chars().next().unwrap()));
            if let Some(extra) = on_enter_tile(&mut gs, &mut rng) {
                out.push_str(&extra);
            }
        }
        "A" => {
            out.push_str(&do_attack(&mut gs, &mut rng));
        }
        "U" if arg == "P" => {
            out.push_str(&use_potion(&mut gs));
        }
        "U" if arg == "B" => {
            out.push_str(&use_bomb(&mut gs, &mut rng));
        }
        "C" if arg == "F" => {
            out.push_str(&cast_fireball(&mut gs, &mut rng));
        }
        "T" => {
            out.push_str(&do_take(&mut gs, &mut rng));
        }
        "O" => {
            out.push_str(&do_open(&mut gs));
        }
        "PICK" => {
            out.push_str(&do_pick_lock(&mut gs, &mut rng));
        }
        "R" => {
            out.push_str(&do_rest(&mut gs, &mut rng));
        }
        "I" => {
            let view = format!(
                "{}\n{}\nOpts: {}\n",
                full_status_line(&gs),
                describe_room(&gs),
                compute_options(&gs).join(" ")
            );
            return (gs, view);
        }
        "M" => {
            // Show mini-map (non-action, doesn't increment turn)
            let map_view = render_map(&gs);
            return (gs, map_view);
        }
        "?" => {
            let view = format!(
                "{}\n{}\n{}\n",
                status_line(&gs),
                describe_room(&gs),
                help_text()
            );
            return (gs, view);
        }
        other
            if other.starts_with("BUY")
                || other == "LEAVE"
                || other == "UPG"
                || other == "MYST" =>
        {
            if let Some(txt) = handle_vendor(&mut gs, cmd) {
                out.push_str(&txt);
            } else {
                out.push_str("No vendor here.\n");
            }
        }
        "Q" => {
            return (gs, "Quit.\n".into());
        }
        _ => {
            out.push_str("Bad cmd. Use ? for help.\n");
        }
    }
    gs.turn = gs.turn.saturating_add(1);
    let dead = gs.player.hp <= 0;
    if dead {
        return (gs.clone(), death_text(&gs));
    }
    // If an event produced a full-screen message (e.g., WIN/RIP), return it directly
    let trimmed = out.trim_start();
    if trimmed.starts_with("TH g") {
        return (gs.clone(), out);
    }
    // Otherwise, compose the normal view and append the action/hint text if it fits
    let mut view = String::new();
    view.push_str(&status_line(&gs));
    view.push('\n');
    let mut room = describe_room(&gs);
    let (n, s, w, e) = exits(&gs);
    let mut ex: Vec<&str> = Vec::new();
    if n {
        ex.push("N");
    }
    if s {
        ex.push("S");
    }
    if w {
        ex.push("W");
    }
    if e {
        ex.push("E");
    }
    if !ex.is_empty() {
        room.push_str(" Exits ");
        room.push_str(&ex.join(","));
        room.push('.');
    }
    view.push_str(&room);
    view.push('\n');
    view.push_str("Opts: ");
    view.push_str(&compute_options(&gs).join(" "));
    view.push('\n');
    if !out.is_empty() {
        let extra = out.as_str();
        view.push_str(extra);
        if !extra.ends_with('\n') {
            view.push('\n');
        }
    }
    (gs.clone(), view)
}

/// Load or create a save for the given user; returns the state and the rendered snapshot, plus is_new flag.
pub fn load_or_new_with_flag(base_dir: &str, username: &str) -> (GameState, String, bool) {
    let path = save_path(base_dir, username);
    if path.exists() {
        if let Ok(s) = read_json(&path) {
            if let Ok(mut gs) = serde_json::from_str::<GameState>(&s) {
                // Clamp position within bounds if sizes changed
                if gs.player.x >= gs.w {
                    gs.player.x = gs.w.saturating_sub(1);
                }
                if gs.player.y >= gs.h {
                    gs.player.y = gs.h.saturating_sub(1);
                }
                // Backward compatibility: initialize visited vector if missing
                if gs.visited.is_empty() {
                    gs.visited = vec![false; gs.w * gs.h];
                    // Mark current position as visited
                    let idx = gs.idx(gs.player.x, gs.player.y);
                    if idx < gs.visited.len() {
                        gs.visited[idx] = true;
                    }
                }
                // Render view; do not change intro_shown here for old saves defaulting to true.
                return (gs.clone(), render(&gs), false);
            }
        }
    }
    let seed = rand::thread_rng().gen::<u64>();
    let mut gs = new_game(seed, 6, 6);
    // First render does not include inline intro; welcome is sent separately. Mark intro shown afterwards.
    let view = render(&gs);
    gs.intro_shown = true;
    (gs.clone(), view, true)
}

/// Backwards-compatible wrapper: load or new and render, ignoring the new flag.
pub fn load_or_new_and_render(base_dir: &str, username: &str) -> (GameState, String) {
    let (gs, view, _is_new) = load_or_new_with_flag(base_dir, username);
    (gs, view)
}

/// Apply a command to the current save, persist, and return the new rendered snapshot.
pub fn apply_and_save(base_dir: &str, username: &str, gs: GameState, cmd: &str) -> String {
    // Apply command to the provided state and persist atomically.
    let (ngs, out) = handle_turn(gs, cmd);
    let path = save_path(base_dir, username);
    if let Ok(json) = serde_json::to_string_pretty(&ngs) {
        let _ = write_json_atomic(&path, &json);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn mk_gs_wh(w: usize, h: usize) -> GameState {
        GameState {
            gid: 1,
            turn: 1,
            seed: 42,
            w,
            h,
            player: Player::new(),
            map: vec![Room::default(); w * h],
            intro_shown: true,
            seen_monster: false,
            seen_chest: false,
            seen_vendor: false,
            seen_door: false,
            seen_trap: false,
            visited: vec![false; w * h],
        }
    }

    fn set_room(
        gs: &mut GameState,
        x: usize,
        y: usize,
        kind: RoomKind,
        mon_hp: Option<i32>,
        used: bool,
    ) {
        let idx = gs.idx(x, y);
        gs.map[idx] = Room { kind, mon_hp, used };
    }

    #[test]
    fn render_includes_intro_and_level_progress() {
        // Use a temp dir to avoid touching real data; create a fresh game view.
        let td = tempfile::tempdir().unwrap();
        let base = td.path().to_string_lossy().to_string();
        let (_gs, view, _is_new) = load_or_new_with_flag(&base, "legend_tester");
        assert!(
            view.starts_with("L"),
            "status line should start compactly with L.. got: {}",
            view
        );
        assert!(view.contains(" H"), "status should include HP: {}", view);
        assert!(
            view.contains(" X"),
            "status should include XP progress: {}",
            view
        );
        // Welcome text is sent as a separate first message by the main menu handler, not inline here.
    }

    #[test]
    fn question_mark_shows_help() {
        let mut gs = new_game(123, 6, 6);
        gs.intro_shown = true; // don't include intro to simplify assertion
        let (_ngs, view) = handle_turn(gs, "?");
        assert!(
            view.contains("TinyHack Commands:"),
            "help expected, got: {}",
            view
        );
        assert!(
            !view.contains("Bad cmd"),
            "should not show bad cmd on help: {}",
            view
        );
    }

    #[test]
    fn parse_cmd_accepts_unicode_question_marks() {
        for inp in [
            "?", "  ?  ", "？？", " ¿ ", "\u{FF1F}", "\u{FE56}", "\u{061F}",
        ] {
            let (op, arg) = parse_cmd(inp);
            assert_eq!(op, "?", "input {:?} should map to help", inp);
            assert!(arg.is_empty());
        }
    }

    #[test]
    fn compute_options_context_specific() {
        let mut gs = mk_gs_wh(2, 2);
        // Empty: base options
        let opts = compute_options(&gs).join(" ");
        assert!(opts.contains("R") && opts.contains("I") && opts.contains("?"));
        // Monster
        set_room(
            &mut gs,
            0,
            0,
            RoomKind::Monster(MonsterKind::Rat),
            Some(3),
            false,
        );
        let opts_m = compute_options(&gs).join(" ");
        assert!(opts_m.contains("A") && opts_m.contains("U P") && opts_m.contains("C F"));
        // LockedDoor
        set_room(&mut gs, 0, 0, RoomKind::LockedDoor, None, false);
        let opts_d = compute_options(&gs).join(" ");
        assert!(opts_d.contains("O") && opts_d.contains("PICK") && opts_d.contains("U B"));
        // Chest
        set_room(&mut gs, 0, 0, RoomKind::Chest, None, false);
        let opts_c = compute_options(&gs).join(" ");
        assert!(opts_c.contains("T"));
    }

    #[test]
    fn movement_boundaries_and_blocks() {
        let mut gs = mk_gs_wh(2, 2);
        // At (0,0), moving North/West should fail
        let msg_n = do_move(&mut gs, 'N');
        assert!(msg_n.contains("Can't move"));
        assert_eq!((gs.player.x, gs.player.y), (0, 0));
        let msg_w = do_move(&mut gs, 'W');
        assert!(msg_w.contains("Can't move"));
        // Place locked door to the east to block
        set_room(&mut gs, 1, 0, RoomKind::LockedDoor, None, false);
        let msg_e = do_move(&mut gs, 'E');
        assert!(msg_e.contains("locked door bars your way"));
        assert_eq!((gs.player.x, gs.player.y), (0, 0));
        // South should be fine
        let msg_s = do_move(&mut gs, 'S');
        assert!(msg_s.is_empty());
        assert_eq!((gs.player.x, gs.player.y), (0, 1));
    }

    #[test]
    fn vendor_buy_and_upgrade() {
        let mut gs = mk_gs_wh(2, 2);
        set_room(&mut gs, 0, 0, RoomKind::Vendor, None, false);
        gs.player.gold = 20;
        // Buy potion
        let resp = handle_vendor(&mut gs, "BUY P").expect("vendor here");
        assert!(resp.contains("Bought potion"));
        assert_eq!(gs.player.potions, 2);
        // Upgrade weapon
        let resp2 = handle_vendor(&mut gs, "UPG W").expect("vendor here");
        assert!(resp2.contains("Upgraded weapon"));
        assert_eq!(gs.player.weapon_lvl, 1);
        assert_eq!(gs.player.atk, 3);
    }

    #[test]
    fn use_potion_and_bomb() {
        let mut gs = mk_gs_wh(2, 2);
        // Potion
        gs.player.hp = 4;
        gs.player.potions = 1;
        let pmsg = use_potion(&mut gs);
        assert!(pmsg.contains("quaff a potion"));
        assert!(gs.player.hp > 4);
        assert_eq!(gs.player.potions, 0);
        // Bomb on door
        set_room(&mut gs, 0, 0, RoomKind::LockedDoor, None, false);
        gs.player.bombs = 1;
        let mut rng = StdRng::seed_from_u64(7);
        let bmsg = use_bomb(&mut gs, &mut rng);
        assert!(bmsg.contains("charge") || bmsg.contains("splinters"));
        assert!(matches!(gs.room(0, 0).kind, RoomKind::Empty));
        assert!(gs.player.gold >= 3);
        assert!(gs.player.xp >= 1);
    }

    #[test]
    fn cast_fireball_kills_rat() {
        let mut gs = mk_gs_wh(2, 2);
        set_room(
            &mut gs,
            0,
            0,
            RoomKind::Monster(MonsterKind::Rat),
            Some(4),
            false,
        );
        gs.player.scrolls = 1;
        let mut rng = StdRng::seed_from_u64(9);
        let msg = cast_fireball(&mut gs, &mut rng);
        assert!(
            msg.contains("inferno")
                || msg.contains("reduced to ash")
                || msg.contains("ends the fight")
                || msg.contains("(+")
        );
        assert!(matches!(gs.room(0, 0).kind, RoomKind::Empty));
        assert_eq!(gs.player.scrolls, 0);
        assert!(gs.player.gold >= 1 && gs.player.gold <= 3);
        assert!(gs.player.xp >= 1);
    }

    fn find_seed_for_threshold(thresh: f64, want_below: bool) -> u64 {
        for s in 0u64..10_000u64 {
            let mut rng = StdRng::seed_from_u64(s);
            let v: f64 = rng.gen();
            if want_below && v < thresh {
                return s;
            }
            if !want_below && v >= thresh {
                return s;
            }
        }
        0
    }

    #[test]
    fn pick_lock_success_and_failure() {
        let mut gs = mk_gs_wh(2, 2);
        set_room(&mut gs, 0, 0, RoomKind::LockedDoor, None, false);
        gs.player.lvl = 1;
        gs.player.lockpicks = 1;
        let chance = 0.40 + 0.05 + 0.10; // 0.55
        let seed_ok = find_seed_for_threshold(chance, true);
        let mut rng_ok = StdRng::seed_from_u64(seed_ok);
        let msg_ok = do_pick_lock(&mut gs, &mut rng_ok);
        assert!(msg_ok.contains("click") || msg_ok.contains("yields"));
        assert!(matches!(gs.room(0, 0).kind, RoomKind::Empty));
        assert!(gs.player.gold >= 2);
        // Reset for failure case
        set_room(&mut gs, 0, 0, RoomKind::LockedDoor, None, false);
        gs.player.lockpicks = 1;
        let seed_bad = find_seed_for_threshold(chance, false);
        let mut rng_bad = StdRng::seed_from_u64(seed_bad);
        let msg_bad = do_pick_lock(&mut gs, &mut rng_bad);
        assert!(msg_bad.contains("trap") || msg_bad.contains("rat") || msg_bad.contains("clumsy"));
        assert_eq!(gs.player.lockpicks, 0);
    }

    #[test]
    fn render_shows_intro_when_first_time() {
        // New behavior: welcome is a separate message; render() does not include it.
        let (gs, view, is_new) = load_or_new_with_flag("/tmp", "unit_intro_test");
        assert!(is_new, "expected new-game flag true on first load");
        assert!(
            view.starts_with("L"),
            "screen should start with status line"
        );
        // Welcome message is provided separately and should be concise (<=200 chars target)
        let welcome = welcome_message();
        assert!(
            welcome.len() <= 200,
            "welcome should be ~200 chars or less; got {}",
            welcome.len()
        );
        // gs.intro_shown should be true after first load
        assert!(gs.intro_shown);
    }

    #[test]
    fn quit_command_short_circuits() {
        let gs = mk_gs_wh(2, 2);
        let (_ngs, view) = handle_turn(gs, "Q");
        assert!(view.starts_with("Quit"));
    }

    #[test]
    fn stairs_win_text_on_enter() {
        let mut gs = mk_gs_wh(2, 2);
        set_room(&mut gs, 0, 0, RoomKind::Stairs, None, false);
        let mut rng = StdRng::seed_from_u64(1);
        let maybe = on_enter_tile(&mut gs, &mut rng);
        assert!(maybe.unwrap_or_default().starts_with("TH g"));
    }
}
