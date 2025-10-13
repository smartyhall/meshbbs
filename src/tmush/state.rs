use chrono::{DateTime, Utc};

use crate::tmush::types::{
    AchievementCategory, AchievementRecord, AchievementTrigger, CurrencyAmount, Direction,
    ObjectOwner, ObjectRecord, ObjectTrigger, RoomFlag, RoomRecord, OBJECT_SCHEMA_VERSION,
};

/// Required landing location where new characters are staged before entering the world.
pub const REQUIRED_LANDING_LOCATION_ID: &str = "gazebo_landing";

/// Required starting location where new characters enter after creation completes.
pub const REQUIRED_START_LOCATION_ID: &str = "town_square";

/// Old Towne Mesh sample world locations that ship as a reference implementation.
///
/// Only `REQUIRED_LANDING_LOCATION_ID` and `REQUIRED_START_LOCATION_ID` are
/// mandatory system requirements. The remaining locations are instructional
/// set dressing that operators are free to replace with their own layouts.
pub const OLD_TOWNE_WORLD_ROOM_IDS: &[&str] = &[
    REQUIRED_LANDING_LOCATION_ID,
    REQUIRED_START_LOCATION_ID,
    "city_hall_lobby",
    "mayor_office",
    "mesh_museum",
    "north_gate",
    "south_market",
];

/// Build the canonical "Old Towne Mesh" starter rooms that Phase 1 seeds into the database.
///
/// The timestamps for each room are deterministic based on the `now` provided so tests can
/// supply a fixed value. Callers typically pass `Utc::now()` in production paths.
pub fn canonical_world_seed(now: DateTime<Utc>) -> Vec<RoomRecord> {
    let mut rooms = Vec::new();

    let landing = RoomRecord::world(
        REQUIRED_LANDING_LOCATION_ID,
        "Landing Gazebo",
        "A welcoming gazebo where new arrivals first materialize.",
        "You stand in an octagonal gazebo with polished wooden railings. Soft mesh lanterns \
cast a warm glow, and a carved wooden sign reads 'Welcome to Old Towne Mesh!' \
Through the northern archway, you can see the bustling Town Square. This is a \
safe place to learn the basics - try typing LOOK to examine your surroundings, \
INVENTORY to check what you're carrying, or HELP to see available commands. \
When ready, head NORTH to begin your adventure!",
    )
    .with_created_at(now)
    .with_exit(Direction::North, REQUIRED_START_LOCATION_ID)
    .with_flag(RoomFlag::Safe)
    .with_flag(RoomFlag::Indoor);
    rooms.push(landing);

    let town_square = RoomRecord::world(
        REQUIRED_START_LOCATION_ID,
        "Old Towne Square",
        "A tidy plaza centered around the Mesh beacon.",
        "Stone paths radiate from the beacon in the square's center. Mesh terminals hum
quietly while townsfolk trade stories about far-off packet relays.",
    )
    .with_created_at(now)
    .with_exit(Direction::North, "city_hall_lobby")
    .with_exit(Direction::East, "mesh_museum")
    .with_exit(Direction::West, "north_gate")
    .with_exit(Direction::South, "south_market")
    .with_exit(Direction::Down, REQUIRED_LANDING_LOCATION_ID)
    .with_capacity(25)
    .with_flag(RoomFlag::Safe)
    .with_flag(RoomFlag::Indoor);
    rooms.push(town_square);

    let city_hall = RoomRecord::world(
        "city_hall_lobby",
        "City Hall Lobby",
        "Sunlight filters through tall windows onto polished floors.",
        "Clerks shuffle reports about network outages while a patient queue waits to
register new callsigns. A portrait of the mayor watches over the proceedings.",
    )
    .with_created_at(now)
    .with_exit(Direction::South, REQUIRED_START_LOCATION_ID)
    .with_exit(Direction::North, "mayor_office")
    .with_exit(Direction::East, "mesh_museum")
    .with_flag(RoomFlag::Indoor)
    .with_flag(RoomFlag::Moderated);
    rooms.push(city_hall);

    let mayor_office = RoomRecord::world(
        "mayor_office",
        "Mayor's Office",
        "A well-appointed office with oak desk and mesh maps on walls.",
        "Mayor Thompson sits behind a sturdy oak desk, reviewing network topology
maps. Framed certificates line the walls alongside charts tracking mesh
uptime metrics. A window overlooks the town square.",
    )
    .with_created_at(now)
    .with_exit(Direction::South, "city_hall_lobby")
    .with_flag(RoomFlag::Safe)
    .with_flag(RoomFlag::Indoor)
    .with_flag(RoomFlag::QuestLocation);
    rooms.push(mayor_office);

    let museum = RoomRecord::world(
        "mesh_museum",
        "Mesh Museum",
        "Glass cases showcase legendary mesh hardware.",
        "Plaques describe pioneering nodes that kept the network alive during winter storms.
Interactive exhibits let visitors replay famous packet traces.",
    )
    .with_created_at(now)
    .with_exit(Direction::West, REQUIRED_START_LOCATION_ID)
    .with_exit(Direction::North, "north_gate")
    .with_flag(RoomFlag::Indoor)
    .with_flag(RoomFlag::QuestLocation);
    rooms.push(museum);

    let north_gate = RoomRecord::world(
        "north_gate",
        "North Gate",
        "A sturdy archway opens toward pine forests and repeater towers.",
        "Guards wave as traders arrive with crates of modules bound for the workshop. Beyond the
path a narrow trail promises adventure past the ridge.",
    )
    .with_created_at(now)
    .with_exit(Direction::South, REQUIRED_START_LOCATION_ID)
    .with_exit(Direction::East, "mesh_museum")
    .with_flag(RoomFlag::PlayerCreated)
    .with_flag(RoomFlag::QuestLocation);
    rooms.push(north_gate);

    let south_market = RoomRecord::world(
        "south_market",
        "South Market",
        "Stalls overflow with gadgets, produce, and hand-built antennas.",
        "Vendors haggle over packet quotas while musicians keep the tempo upbeat. The scent of
fresh bread mingles with solder flux drifting from a repair booth.",
    )
    .with_created_at(now)
    .with_exit(Direction::North, REQUIRED_START_LOCATION_ID)
    .with_flag(RoomFlag::Shop)
    .with_flag(RoomFlag::Indoor)
    .with_capacity(18);
    rooms.push(south_market);

    rooms
}

/// Sample starter quests for Phase 6 Week 2
///
/// These quests demonstrate the quest system functionality and provide
/// new players with guided activities to explore Old Towne Mesh.
pub fn seed_starter_quests() -> Vec<crate::tmush::types::QuestRecord> {
    use crate::tmush::types::{CurrencyAmount, ObjectiveType, QuestObjective, QuestRecord};

    let mut quests = Vec::new();

    // Quest 1: Welcome to Old Towne (beginner)
    let welcome_quest = QuestRecord::new(
        "welcome_towne",
        "Welcome to Old Towne",
        "Explore the basics of Old Towne Mesh and meet important NPCs.",
        "mayor_thompson",
        1,
    )
    .with_objective(QuestObjective::new(
        "Visit the Town Square",
        ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        },
        1,
    ))
    .with_objective(QuestObjective::new(
        "Visit the Mesh Museum",
        ObjectiveType::VisitLocation {
            room_id: "mesh_museum".to_string(),
        },
        1,
    ))
    .with_objective(QuestObjective::new(
        "Talk to Mayor Thompson",
        ObjectiveType::TalkToNpc {
            npc_id: "mayor_thompson".to_string(),
        },
        1,
    ))
    .with_reward_currency(CurrencyAmount::Decimal { minor_units: 1000 }) // $10 or 100cp
    .with_reward_experience(50)
    .with_reward_item("town_badge");
    quests.push(welcome_quest);

    // Quest 2: Explore the Markets (requires quest 1)
    let market_quest = QuestRecord::new(
        "market_exploration",
        "Market Exploration",
        "Visit the South Market and learn about trading.",
        "mayor_thompson",
        2,
    )
    .with_objective(QuestObjective::new(
        "Visit South Market",
        ObjectiveType::VisitLocation {
            room_id: "south_market".to_string(),
        },
        1,
    ))
    .with_objective(QuestObjective::new(
        "Return to Town Square",
        ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        },
        1,
    ))
    .with_prerequisite("welcome_towne")
    .with_reward_currency(CurrencyAmount::Decimal { minor_units: 1500 }) // $15 or 150cp
    .with_reward_experience(75);
    quests.push(market_quest);

    // Quest 3: Network Explorer (advanced, requires quest 2)
    let explorer_quest = QuestRecord::new(
        "network_explorer",
        "Network Explorer",
        "Chart the full extent of Old Towne Mesh.",
        "mayor_thompson",
        3,
    )
    .with_objective(QuestObjective::new(
        "Visit all 7 locations",
        ObjectiveType::VisitLocation {
            room_id: "north_gate".to_string(),
        },
        1,
    ))
    .with_prerequisite("market_exploration")
    .with_reward_currency(CurrencyAmount::Decimal { minor_units: 2500 }) // $25 or 250cp
    .with_reward_experience(150)
    .with_reward_item("explorer_compass");
    quests.push(explorer_quest);

    quests
}

/// Generate starter achievements for Old Towne Mesh (Phase 6 Week 3)
pub fn seed_starter_achievements() -> Vec<AchievementRecord> {
    use AchievementCategory::*;
    use AchievementTrigger::*;

    let mut achievements = Vec::new();

    // Combat achievements
    achievements.push(
        AchievementRecord::new(
            "first_blood",
            "First Blood",
            "Defeat your first enemy",
            Combat,
            KillCount { required: 1 },
        )
        .with_title("the Brave"),
    );

    achievements.push(AchievementRecord::new(
        "veteran",
        "Veteran",
        "Defeat 100 enemies",
        Combat,
        KillCount { required: 100 },
    ));

    achievements.push(
        AchievementRecord::new(
            "legendary",
            "Legendary Warrior",
            "Defeat 1000 enemies",
            Combat,
            KillCount { required: 1000 },
        )
        .with_title("the Legendary")
        .as_hidden(),
    );

    // Exploration achievements
    achievements.push(
        AchievementRecord::new(
            "wanderer",
            "Wanderer",
            "Visit 10 unique rooms",
            Exploration,
            RoomVisits { required: 10 },
        )
        .with_title("the Wanderer"),
    );

    achievements.push(AchievementRecord::new(
        "explorer",
        "Explorer",
        "Visit 50 unique rooms",
        Exploration,
        RoomVisits { required: 50 },
    ));

    achievements.push(
        AchievementRecord::new(
            "cartographer",
            "Cartographer",
            "Visit all rooms in Old Towne",
            Exploration,
            RoomVisits { required: 100 },
        )
        .with_title("the Cartographer"),
    );

    // Social achievements
    achievements.push(AchievementRecord::new(
        "friendly",
        "Friendly",
        "Make 5 friends",
        Social,
        FriendCount { required: 5 },
    ));

    achievements.push(
        AchievementRecord::new(
            "popular",
            "Popular",
            "Make 20 friends",
            Social,
            FriendCount { required: 20 },
        )
        .with_title("the Popular"),
    );

    achievements.push(AchievementRecord::new(
        "chatterbox",
        "Chatterbox",
        "Send 1000 messages",
        Social,
        MessagesSent { required: 1000 },
    ));

    // Economic achievements
    achievements.push(
        AchievementRecord::new(
            "merchant",
            "Merchant",
            "Complete 50 trades",
            Economic,
            TradeCount { required: 50 },
        )
        .with_title("the Merchant"),
    );

    achievements.push(AchievementRecord::new(
        "wealthy",
        "Wealthy",
        "Earn 100000 currency",
        Economic,
        CurrencyEarned { amount: 100000 },
    ));

    // Quest achievements
    achievements.push(
        AchievementRecord::new(
            "quest_beginner",
            "Quest Beginner",
            "Complete your first quest",
            Quest,
            QuestCompletion { required: 1 },
        )
        .with_title("the Questor"),
    );

    achievements.push(AchievementRecord::new(
        "quest_veteran",
        "Quest Veteran",
        "Complete 25 quests",
        Quest,
        QuestCompletion { required: 25 },
    ));

    // Special/hidden achievements
    achievements.push(
        AchievementRecord::new(
            "town_founder",
            "Town Founder",
            "Discover the founder's secret",
            Special,
            VisitLocation {
                room_id: "mayor_office".to_string(),
            },
        )
        .with_title("Town Founder")
        .as_hidden(),
    );

    achievements.push(
        AchievementRecord::new(
            "network_pioneer",
            "Network Pioneer",
            "Complete the Network Explorer quest",
            Special,
            CompleteQuest {
                quest_id: "network_explorer".to_string(),
            },
        )
        .with_title("Network Pioneer"),
    );

    achievements
}

/// Seed starter companions for Old Towne Mesh (Phase 6 Week 4)
pub fn seed_starter_companions() -> Vec<crate::tmush::types::CompanionRecord> {
    use crate::tmush::types::{CompanionRecord, CompanionType};

    let mut companions = Vec::new();

    // A gentle horse available at the stable
    companions.push(
        CompanionRecord::new(
            "gentle_mare",
            "Gentle Mare",
            CompanionType::Horse,
            "south_market",
        )
        .with_description("A gentle brown mare with kind eyes. She seems eager for a rider."),
    );

    // A loyal dog at the town square
    companions.push(
        CompanionRecord::new(
            "loyal_hound",
            "Loyal Hound",
            CompanionType::Dog,
            REQUIRED_START_LOCATION_ID,
        )
        .with_description("A friendly dog with alert eyes. He wags his tail hopefully."),
    );

    // A mysterious cat near the museum
    companions.push(
        CompanionRecord::new(
            "shadow_cat",
            "Shadow Cat",
            CompanionType::Cat,
            "mesh_museum",
        )
        .with_description("A sleek black cat with piercing green eyes. She watches you intently."),
    );

    companions
}

/// Seed starter NPCs for Old Towne Mesh
pub fn seed_starter_npcs() -> Vec<crate::tmush::types::NpcRecord> {
    use crate::tmush::types::NpcRecord;

    let mut npcs = Vec::new();

    // Mayor Thompson - Tutorial completion NPC
    let mayor = NpcRecord::new(
        "mayor_thompson",
        "Mayor Thompson",
        "Mayor of Old Towne Mesh",
        "A distinguished figure in formal attire. Mayor Thompson greets visitors with \
a warm smile and firm handshake. Years of network administration show in the \
confident way he discusses mesh topology.",
        "mayor_office",
    )
    .with_dialog(
        "greeting",
        "Welcome to Old Towne Mesh! I'm Mayor Thompson. \
I oversee the network operations here. How can I help you today?",
    )
    .with_dialog(
        "tutorial_complete",
        "Excellent work completing the tutorial! You've learned the basics of navigating \
our world. Here's a small reward to get you started on your adventures. \
Welcome to Old Towne Mesh, citizen!",
    )
    .with_dialog(
        "quest_welcome",
        "Looking for something to do? I have some tasks that could use a capable \
adventurer like yourself. Check back when you're ready for a challenge!",
    );

    npcs.push(mayor);

    // City Hall Clerk - Administrative help
    let clerk = NpcRecord::new(
        "city_clerk",
        "City Clerk",
        "Administrative Clerk",
        "A busy clerk with wire-rimmed glasses shuffles through papers. She looks up \
with a professional smile, ready to assist with administrative matters.",
        "city_hall_lobby",
    )
    .with_dialog(
        "greeting",
        "Welcome to City Hall! I handle administrative matters. If you need help \
understanding how things work around here, just ask!",
    )
    .with_dialog(
        "help",
        "Old Towne Mesh operates on a mesh network. You can explore the town, \
complete quests, trade with others, and even claim housing! Type HELP for commands.",
    )
    .with_flag(crate::tmush::types::NpcFlag::TutorialNpc);

    npcs.push(clerk);

    // Gate Guard - Security and direction
    let guard = NpcRecord::new(
        "gate_guard",
        "Gate Guard",
        "North Gate Guard",
        "A weathered guard in practical gear stands watch. Years of patrol have given \
her a keen eye for travelers. She nods in acknowledgment as you approach.",
        "north_gate",
    )
    .with_dialog(
        "greeting",
        "Greetings, traveler. I keep watch over the northern approach. Beyond lies \
the wilderness - beautiful but dangerous for the unprepared.",
    )
    .with_dialog(
        "warning",
        "If you venture beyond the gate, be sure you're properly equipped. The mesh \
signal weakens past the ridge, and you'll be on your own out there.",
    )
    .with_flag(crate::tmush::types::NpcFlag::Guard);

    npcs.push(guard);

    // Market Vendor - Trading and commerce
    let vendor = NpcRecord::new(
        "market_vendor",
        "Mira the Vendor",
        "South Market Trader",
        "A cheerful vendor behind a stall piled with mesh hardware and local goods. \
Mira has a reputation for fair prices and interesting stories about the items she sells.",
        "south_market",
    )
    .with_dialog(
        "greeting",
        "Welcome to my stall! I've got the finest mesh components and supplies in \
Old Towne. Looking for anything particular?",
    )
    .with_dialog(
        "wares",
        "I stock everything from basic antennas to rare modules salvaged from old nodes. \
Right now I'm running low on inventory, but check back soon - I restock regularly!",
    )
    .with_dialog(
        "story",
        "This stall has been in my family for three generations. My grandmother was one \
of the original mesh pioneers. These goods carry that legacy!",
    )
    .with_flag(crate::tmush::types::NpcFlag::Vendor);

    npcs.push(vendor);

    // Museum Curator - Lore and history
    let curator = NpcRecord::new(
        "museum_curator",
        "Dr. Reeves",
        "Museum Curator",
        "An elderly scholar with kind eyes and countless stories. Dr. Reeves has \
dedicated decades to preserving the history of the mesh network.",
        "mesh_museum",
    )
    .with_dialog(
        "greeting",
        "Ah, welcome to the Mesh Museum! I'm Dr. Reeves, curator and historian. \
Every artifact here tells a story of our network's resilience.",
    )
    .with_dialog(
        "history",
        "This museum chronicles the early days of the mesh - the pioneers who kept \
packets flowing through blizzards and power outages. Each node had a story, \
each relay a hero behind it.",
    )
    .with_dialog(
        "exhibit",
        "That display there? That's the original relay from Winter Storm '19. \
It ran for 72 hours on backup power, keeping the southern district connected. \
Legendary piece of equipment.",
    )
    .with_flag(crate::tmush::types::NpcFlag::TutorialNpc);

    npcs.push(curator);

    npcs
}

/// Seed full dialogue trees for all NPCs (called after seed_npcs_if_needed)
pub fn seed_npc_dialogues_if_needed(
    store: &crate::tmush::storage::TinyMushStore,
) -> Result<usize, crate::tmush::errors::TinyMushError> {
    use crate::tmush::types::{DialogChoice, DialogNode};
    use std::collections::HashMap;

    let mut updated = 0;

    // Mayor Thompson - Tutorial and town info
    match store.get_npc("mayor_thompson") {
        Ok(mut mayor) => {
            if mayor.dialog_tree.is_empty() {
                let mut tree = HashMap::new();

                // Root greeting node
                tree.insert("greeting".to_string(), DialogNode::new(
                "Welcome to Old Towne Mesh! I'm Mayor Thompson, and I oversee operations here. \
                Whether you're just starting out or looking for what to do next, I'm here to help!"
            )
            .with_choice(DialogChoice::new("Tell me about the tutorial").goto("tutorial"))
            .with_choice(DialogChoice::new("What is there to do in town?").goto("town_info"))
            .with_choice(DialogChoice::new("Do you have any quests?").goto("quest_check"))
            .with_choice(DialogChoice::new("Thanks, I'll look around").exit()));

                // Tutorial branch
                tree.insert("tutorial".to_string(), DialogNode::new(
                "The tutorial will teach you the basics of navigating our world - movement, talking to NPCs, \
                using items, and more. It's a great way to get your bearings!"
            )
            .with_choice(DialogChoice::new("How do I start the tutorial?").goto("tutorial_start"))
            .with_choice(DialogChoice::new("What commands should I know?").goto("tutorial_commands"))
            .with_choice(DialogChoice::new("I'd rather skip it").goto("skip_tutorial"))
            .with_choice(DialogChoice::new("Let me think about it").exit()));

                tree.insert("tutorial_start".to_string(), DialogNode::new(
                "Just type START TUTORIAL and you'll begin! The tutorial will guide you through each step. \
                When you complete it, come back and I'll have a small reward for you."
            )
            .with_choice(DialogChoice::new("Thanks, I'll try it!").exit())
            .with_choice(DialogChoice::new("What else is there?").goto("town_info")));

                tree.insert("tutorial_commands".to_string(), DialogNode::new(
                "Basic commands: LOOK (examine surroundings), GO <direction> (move), TALK <npc> (chat), \
                INVENTORY (check items), HELP (full command list). The tutorial covers all of these!"
            )
            .with_choice(DialogChoice::new("Got it!").exit())
            .with_choice(DialogChoice::new("Tell me about town").goto("town_info")));

                tree.insert("skip_tutorial".to_string(), DialogNode::new(
                "That's fine! You can always start it later with START TUTORIAL. Feel free to explore \
                on your own - the City Clerk can help with questions."
            )
            .with_choice(DialogChoice::new("Thanks").exit()));

                // Town info branch
                tree.insert("town_info".to_string(), DialogNode::new(
                "Old Towne Mesh has everything you need! We have a museum of mesh history, a marketplace, \
                City Hall for administrative help, and gates leading to the wilderness."
            )
            .with_choice(DialogChoice::new("Tell me about housing").goto("housing_info"))
            .with_choice(DialogChoice::new("What about quests?").goto("quest_check"))
            .with_choice(DialogChoice::new("Thanks!").exit()));

                tree.insert("housing_info".to_string(), DialogNode::new(
                "You can claim housing through City Hall! Talk to the City Clerk about available options. \
                Having your own place lets you store items and customize your space."
            )
            .with_choice(DialogChoice::new("Sounds great!").exit())
            .with_choice(DialogChoice::new("What else?").goto("town_info")));

                // Quest branch
                tree.insert("quest_check".to_string(), DialogNode::new(
                "I have some tasks that need doing, but I like to make sure folks are ready first. \
                Complete the tutorial and get familiar with town, then come back!"
            )
            .with_choice(DialogChoice::new("Will do!").exit())
            .with_choice(DialogChoice::new("Tell me about town").goto("town_info")));

                mayor.dialog_tree = tree;
                store.put_npc(mayor)?;
                updated += 1;
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not load mayor_thompson for dialogue seeding: {}",
                e
            );
        }
    }

    // City Clerk - Administrative help
    match store.get_npc("city_clerk") {
        Ok(mut clerk) => {
            if clerk.dialog_tree.is_empty() {
                let mut tree = HashMap::new();

                // Root greeting node
                tree.insert("greeting".to_string(), DialogNode::new(
                    "Welcome to City Hall! I'm here to help with administrative matters. Whether you need \
                    information about housing, quests, or town services, I can point you in the right direction."
                )
                .with_choice(DialogChoice::new("Tell me about housing").goto("housing_help"))
                .with_choice(DialogChoice::new("What quests are available?").goto("quest_help"))
                .with_choice(DialogChoice::new("What services does City Hall offer?").goto("services"))
                .with_choice(DialogChoice::new("Thanks, I'm all set").exit()));

                // Housing branch
                tree.insert("housing_help".to_string(), DialogNode::new(
                    "City Hall manages the housing system! You can claim an apartment or larger space \
                    depending on what you can afford. Each housing unit is yours to customize and store items."
                )
                .with_choice(DialogChoice::new("How much does housing cost?").goto("housing_cost"))
                .with_choice(DialogChoice::new("What else can you tell me?").goto("services"))
                .with_choice(DialogChoice::new("I'll think about it").exit()));

                tree.insert("housing_cost".to_string(), DialogNode::new(
                    "We have several options! Studio apartments start at 100 credits to claim, with a small \
                    recurring fee. Larger apartments with multiple rooms cost more but give you more space. \
                    Check the housing board for current availability."
                )
                .with_choice(DialogChoice::new("Sounds good!").exit())
                .with_choice(DialogChoice::new("Tell me about other services").goto("services")));

                // Quest branch
                tree.insert("quest_help".to_string(), DialogNode::new(
                    "Various people around town have tasks they need help with. Mayor Thompson coordinates \
                    most quest activities. You can also check the bulletin boards for opportunities!"
                )
                .with_choice(DialogChoice::new("Where should I look?").goto("quest_locations"))
                .with_choice(DialogChoice::new("What else can you help with?").goto("services"))
                .with_choice(DialogChoice::new("Thanks!").exit()));

                tree.insert("quest_locations".to_string(), DialogNode::new(
                    "The Mayor's office is a great place to start. The museum curator, market vendors, and \
                    even the gate guards sometimes have work available. Keep your eyes open!"
                )
                .with_choice(DialogChoice::new("Got it!").exit())
                .with_choice(DialogChoice::new("Tell me about housing").goto("housing_help")));

                // Services branch
                tree.insert("services".to_string(), DialogNode::new(
                    "City Hall handles housing claims, maintains public records, and coordinates with \
                    various town departments. We also keep bulletin boards updated with community news and \
                    opportunities. If you ever have questions about how things work, come find me!"
                )
                .with_choice(DialogChoice::new("Very helpful, thanks!").exit())
                .with_choice(DialogChoice::new("About housing...").goto("housing_help"))
                .with_choice(DialogChoice::new("About quests...").goto("quest_help")));

                clerk.dialog_tree = tree;
                store.put_npc(clerk)?;
                updated += 1;
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not load city_clerk for dialogue seeding: {}",
                e
            );
        }
    }

    // Gate Guard - Security and warnings
    match store.get_npc("gate_guard") {
        Ok(mut guard) => {
            if guard.dialog_tree.is_empty() {
                let mut tree = HashMap::new();

                // Root greeting node
                tree.insert("greeting".to_string(), DialogNode::new(
                    "Greetings, traveler. I keep watch over the northern approach. Beyond this gate lies \
                    the wilderness - beautiful but dangerous for those unprepared."
                )
                .with_choice(DialogChoice::new("What's out there?").goto("looking"))
                .with_choice(DialogChoice::new("What dangers should I know about?").goto("dangers"))
                .with_choice(DialogChoice::new("Any advice for exploring?").goto("advice"))
                .with_choice(DialogChoice::new("Just passing through").exit()));

                // Looking branch
                tree.insert("looking".to_string(), DialogNode::new(
                    "The wilderness holds many opportunities - rare resources, ancient relics, and \
                    untamed beauty. But it's also home to hostile creatures and environmental hazards. \
                    The mesh signal weakens the farther you go."
                )
                .with_choice(DialogChoice::new("What kind of dangers?").goto("dangers"))
                .with_choice(DialogChoice::new("Should I go out there?").goto("adventure_warning"))
                .with_choice(DialogChoice::new("I'll be careful").exit()));

                // Dangers branch
                tree.insert("dangers".to_string(), DialogNode::new(
                    "Wild animals, unstable terrain, signal dead zones... The wilderness doesn't forgive \
                    mistakes. Many venture out unprepared and need rescue. Some never come back."
                )
                .with_choice(DialogChoice::new("How should I prepare?").goto("equipment"))
                .with_choice(DialogChoice::new("Maybe I should stay in town").goto("adventure_warning"))
                .with_choice(DialogChoice::new("Thanks for the warning").exit()));

                tree.insert("adventure_warning".to_string(), DialogNode::new(
                    "Smart thinking. Build your skills in town first. Complete some quests, gather supplies, \
                    and learn your way around. The wilderness will still be here when you're ready."
                )
                .with_choice(DialogChoice::new("Good advice").exit())
                .with_choice(DialogChoice::new("Where's the market?").goto("market_direction")));

                // Equipment branch
                tree.insert("equipment".to_string(), DialogNode::new(
                    "At minimum, bring food, water, and basic tools. A reliable signal booster helps maintain \
                    connectivity. The market district has supplies - Mira's stall is your best bet."
                )
                .with_choice(DialogChoice::new("Where's the market?").goto("market_direction"))
                .with_choice(DialogChoice::new("Thanks, I'll gear up").exit()));

                tree.insert("market_direction".to_string(), DialogNode::new(
                    "Head south from the town square. You can't miss it - lots of stalls and activity. \
                    Mira runs a good operation, fair prices and quality gear."
                )
                .with_choice(DialogChoice::new("Thanks!").exit()));

                // Advice branch
                tree.insert("advice".to_string(), DialogNode::new(
                    "Never go out alone on your first trip. Stay within sight of the walls until you know \
                    what you're doing. And always tell someone where you're going. The rescue teams can only \
                    help if they know where to look."
                )
                .with_choice(DialogChoice::new("I'll remember that").exit())
                .with_choice(DialogChoice::new("Tell me about the dangers").goto("dangers")));

                guard.dialog_tree = tree;
                store.put_npc(guard)?;
                updated += 1;
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not load gate_guard for dialogue seeding: {}",
                e
            );
        }
    }

    // Market Vendor - Trading and commerce
    match store.get_npc("market_vendor") {
        Ok(mut vendor) => {
            if vendor.dialog_tree.is_empty() {
                let mut tree = HashMap::new();

                // Root greeting node
                tree.insert("greeting".to_string(), DialogNode::new(
                    "Welcome to my stall! I'm Mira, and I've got the finest mesh components and supplies \
                    in Old Towne. Everything from basic tools to rare salvaged tech!"
                )
                .with_choice(DialogChoice::new("What do you have for sale?").goto("wares"))
                .with_choice(DialogChoice::new("Tell me about yourself").goto("story"))
                .with_choice(DialogChoice::new("I'd like to buy something").goto("buying"))
                .with_choice(DialogChoice::new("Just browsing, thanks").exit()));

                // Wares branch
                tree.insert("wares".to_string(), DialogNode::new(
                    "I stock everything a traveler needs! Basic tools, signal boosters, mesh hardware, \
                    and the occasional rare find. My inventory changes as I acquire new goods, so check back often!"
                )
                .with_choice(DialogChoice::new("What's your best item?").goto("best_item"))
                .with_choice(DialogChoice::new("Can I buy something now?").goto("buying"))
                .with_choice(DialogChoice::new("Tell me your story").goto("story"))
                .with_choice(DialogChoice::new("Thanks!").exit()));

                tree.insert("best_item".to_string(), DialogNode::new(
                    "Right now? I've got a mesh signal booster that's perfect for wilderness exploration. \
                    Keeps you connected even in weak signal areas. 50 credits and it's yours!"
                )
                .with_choice(DialogChoice::new("I'll take it!").goto("buy_booster"))
                .with_choice(DialogChoice::new("Maybe later").exit()));

                // Story branch
                tree.insert("story".to_string(), DialogNode::new(
                    "This stall has been in my family for three generations! My grandmother was one of the \
                    original mesh pioneers. She taught me that quality goods and fair dealing build lasting networks."
                )
                .with_choice(DialogChoice::new("That's a great tradition!").exit())
                .with_choice(DialogChoice::new("What do you sell?").goto("wares"))
                .with_choice(DialogChoice::new("Tell me more history").goto("history_detail")));

                tree.insert("history_detail".to_string(), DialogNode::new(
                    "My grandmother kept the network running during the great storms of '19. Every piece \
                    of hardware here carries that legacy - built to last, tested in harsh conditions."
                )
                .with_choice(DialogChoice::new("Impressive!").exit())
                .with_choice(DialogChoice::new("Show me what you have").goto("wares")));

                // Buying branch
                tree.insert("buying".to_string(), DialogNode::new(
                    "Wonderful! I have some items ready for immediate purchase, or you can browse my full \
                    inventory at the shop interface."
                )
                .with_choice(DialogChoice::new("What's available now?").goto("prices"))
                .with_choice(DialogChoice::new("Never mind").exit()));

                tree.insert("prices".to_string(), DialogNode::new(
                    "I've got a few items on hand: A basic knife (15 credits) - handy for cutting rope and \
                    general tasks. A signal booster (50 credits) - essential for wilderness travel. More items \
                    available at the main shop!"
                )
                .with_choice(DialogChoice::new("I'll buy the knife").goto("purchase_knife"))
                .with_choice(DialogChoice::new("I'll buy the booster").goto("buy_booster"))
                .with_choice(DialogChoice::new("Just looking for now").exit()));

                // Note: These nodes would use DialogCondition::HasCurrency and DialogAction::TakeCurrency + GiveItem
                // but we're keeping them simple text for now. The @DIALOG EDIT command can add the actions.
                tree.insert("purchase_knife".to_string(), DialogNode::new(
                    "The knife is 15 credits. (This is a placeholder - use @DIALOG EDIT to add currency \
                    conditions and give_item actions)"
                )
                .with_choice(DialogChoice::new("Thanks!").exit())
                .with_choice(DialogChoice::new("What else do you have?").goto("prices")));

                tree.insert("buy_booster".to_string(), DialogNode::new(
                    "The signal booster is 50 credits. (This is a placeholder - use @DIALOG EDIT to add \
                    currency conditions and give_item actions)"
                )
                .with_choice(DialogChoice::new("Thanks!").exit())
                .with_choice(DialogChoice::new("What else?").goto("prices")));

                // Additional info branches
                tree.insert("valuable_items".to_string(), DialogNode::new(
                    "Looking for something special? I occasionally get rare mesh components from salvagers. \
                    Old relay modules, vintage antennas, that sort of thing. Collectors pay top credit!"
                )
                .with_choice(DialogChoice::new("How do I find those?").goto("finding_valuables"))
                .with_choice(DialogChoice::new("Interesting!").exit()));

                tree.insert("finding_valuables".to_string(), DialogNode::new(
                    "Explore the wilderness, check abandoned sites, complete quests. Bring me anything \
                    interesting you find - I'll give you a fair price or trade you something useful!"
                )
                .with_choice(DialogChoice::new("Will do!").exit())
                .with_choice(DialogChoice::new("Show me your current stock").goto("wares")));

                tree.insert("weapons".to_string(), DialogNode::new(
                    "I keep a few basic weapons for self-defense. Nothing fancy, but reliable. The knife is \
                    practical for everyday carry, and I sometimes have staves or other tools."
                )
                .with_choice(DialogChoice::new("I'll take the knife").goto("purchase_knife"))
                .with_choice(DialogChoice::new("What else do you have?").goto("wares"))
                .with_choice(DialogChoice::new("Maybe later").exit()));

                vendor.dialog_tree = tree;
                store.put_npc(vendor)?;
                updated += 1;
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not load market_vendor for dialogue seeding: {}",
                e
            );
        }
    }

    // Museum Curator - Lore and history
    match store.get_npc("museum_curator") {
        Ok(mut curator) => {
            if curator.dialog_tree.is_empty() {
                let mut tree = HashMap::new();

                // Root greeting node
                tree.insert("greeting".to_string(), DialogNode::new(
                    "Ah, welcome to the Mesh Museum! I'm Dr. Reeves, curator and historian. Every artifact \
                    here tells a story of our network's resilience and the people who built it."
                )
                .with_choice(DialogChoice::new("Tell me about the museum").goto("about_museum"))
                .with_choice(DialogChoice::new("What's the history here?").goto("history"))
                .with_choice(DialogChoice::new("Any special exhibits?").goto("exhibit"))
                .with_choice(DialogChoice::new("Just looking around").exit()));

                // Museum branch
                tree.insert("about_museum".to_string(), DialogNode::new(
                    "This museum preserves the history of the mesh network - from the first nodes to today. \
                    We document the technology, the people, and the challenges they overcame to keep packets flowing."
                )
                .with_choice(DialogChoice::new("Who founded it?").goto("founder"))
                .with_choice(DialogChoice::new("Tell me about the history").goto("history"))
                .with_choice(DialogChoice::new("Fascinating!").exit()));

                tree.insert("founder".to_string(), DialogNode::new(
                    "The museum was founded by the original mesh pioneers - people like Mira's grandmother. \
                    They wanted future generations to understand what it took to build this network from nothing."
                )
                .with_choice(DialogChoice::new("Amazing dedication").exit())
                .with_choice(DialogChoice::new("Tell me more history").goto("history_detail")));

                // History branch
                tree.insert("history".to_string(), DialogNode::new(
                    "The mesh network started small - just a few nodes connecting neighbors. Through \
                    determination and ingenuity, it grew into the robust system we have today. Every relay \
                    has a story, every node a guardian."
                )
                .with_choice(DialogChoice::new("Tell me more").goto("history_detail"))
                .with_choice(DialogChoice::new("What about the exhibits?").goto("exhibit"))
                .with_choice(DialogChoice::new("Impressive").exit()));

                tree.insert("history_detail".to_string(), DialogNode::new(
                    "Take the Winter Storm of '19 - power was out for days across the region. But the mesh \
                    stayed up! Battery backups, solar panels, people sharing generators... The network became \
                    a lifeline for the community."
                )
                .with_choice(DialogChoice::new("That's incredible!").exit())
                .with_choice(DialogChoice::new("Tell me about that relay").goto("relay_story"))
                .with_choice(DialogChoice::new("What else is here?").goto("exhibit")));

                tree.insert("relay_story".to_string(), DialogNode::new(
                    "That relay in the center display? It ran for 72 hours straight on backup power, keeping \
                    the southern district connected. The operator - Sarah Chen - stayed with it the entire time, \
                    making repairs, managing power. A true hero of the network."
                )
                .with_choice(DialogChoice::new("Amazing story").exit())
                .with_choice(DialogChoice::new("Are there other exhibits?").goto("other_exhibits")));

                // Exhibits branch
                tree.insert("exhibit".to_string(), DialogNode::new(
                    "Our centerpiece is the Winter Storm '19 relay - a legendary piece of equipment. We also \
                    have displays on early mesh protocols, pioneer stories, and technical evolution."
                )
                .with_choice(DialogChoice::new("Tell me about the storm").goto("relay_story"))
                .with_choice(DialogChoice::new("What about the pioneers?").goto("pioneers"))
                .with_choice(DialogChoice::new("Very interesting!").exit()));

                tree.insert("other_exhibits".to_string(), DialogNode::new(
                    "We have sections on early mesh protocols, the evolution of hardware, and profiles of \
                    key network architects. There's also a hands-on area where you can see how old equipment \
                    worked compared to modern systems."
                )
                .with_choice(DialogChoice::new("I'd like to see that").exit())
                .with_choice(DialogChoice::new("Tell me about the pioneers").goto("pioneers")));

                tree.insert("pioneers".to_string(), DialogNode::new(
                    "The mesh pioneers were ordinary people who saw a need and met it with creativity and \
                    determination. They didn't have perfect equipment or unlimited resources - just commitment \
                    to keeping the community connected."
                )
                .with_choice(DialogChoice::new("Inspiring!").exit())
                .with_choice(DialogChoice::new("Who were the heroes?").goto("heroes")));

                tree.insert("heroes".to_string(), DialogNode::new(
                    "Too many to list! Sarah Chen of Storm '19 fame, Marcus Webb who designed the resilient \
                    routing protocols we still use, the Tanaka family who donated land for relay towers... \
                    Each plaque here represents someone who put the network above themselves."
                )
                .with_choice(DialogChoice::new("I should learn more").exit())
                .with_choice(DialogChoice::new("Thanks for sharing").exit()));

                curator.dialog_tree = tree;
                store.put_npc(curator)?;
                updated += 1;
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not load museum_curator for dialogue seeding: {}",
                e
            );
        }
    }

    Ok(updated)
}

/// Create example trigger objects for Phase 9 testing
///
/// This creates 6 example objects demonstrating all trigger types:
/// - Healing Potion (OnUse with consume + heal)
/// - Ancient Key (OnLook with quest check, OnUse unlocks exit)
/// - Mystery Box (OnPoke with random chance)
/// - Quest Clue (OnLook with dynamic description)
/// - Teleport Stone (OnUse teleports player)
/// - Singing Mushroom (OnEnter ambient message)
pub fn create_example_trigger_objects(now: DateTime<Utc>) -> Vec<ObjectRecord> {
    let mut objects = Vec::new();

    // 1. Healing Potion - OnUse trigger with consume() and heal()
    let mut healing_potion = ObjectRecord {
        id: "example_healing_potion".to_string(),
        name: "Healing Potion".to_string(),
        description: "A crystal vial filled with glowing red liquid. When used, it heals wounds and vanishes.".to_string(),
        owner: ObjectOwner::World,
        created_at: now,
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 50,
        takeable: true,
        usable: true,
        actions: std::collections::HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: OBJECT_SCHEMA_VERSION,
    };
    healing_potion.actions.insert(
        ObjectTrigger::OnUse,
        "message(\"The potion glows brightly as you drink it!\") && heal(50) && consume()"
            .to_string(),
    );
    objects.push(healing_potion);

    // 2. Ancient Key - OnLook with quest check, OnUse unlocks exit
    let mut ancient_key = ObjectRecord {
        id: "example_ancient_key".to_string(),
        name: "Ancient Key".to_string(),
        description: "A tarnished brass key with mysterious runes etched along its length."
            .to_string(),
        owner: ObjectOwner::World,
        created_at: now,
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 100,
        takeable: true,
        usable: true,
        actions: std::collections::HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: OBJECT_SCHEMA_VERSION,
    };
    ancient_key.actions.insert(
        ObjectTrigger::OnLook,
        "has_quest(\"cryptkeeper_quest\") ? message(\"The runes glow faintly - you recognize them from your quest!\") : message(\"The runes are indecipherable.\")".to_string()
    );
    ancient_key.actions.insert(
        ObjectTrigger::OnUse,
        "message(\"The key turns in an invisible lock...\") && unlock_exit(\"north\")".to_string(),
    );
    objects.push(ancient_key);

    // 3. Mystery Box - OnPoke with random chance
    let mut mystery_box = ObjectRecord {
        id: "example_mystery_box".to_string(),
        name: "Mystery Box".to_string(),
        description: "A wooden box covered in question marks. It rattles when poked.".to_string(),
        owner: ObjectOwner::World,
        created_at: now,
        weight: 5,
        currency_value: CurrencyAmount::default(),
        value: 25,
        takeable: true,
        usable: false,
        actions: std::collections::HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: OBJECT_SCHEMA_VERSION,
    };
    mystery_box.actions.insert(
        ObjectTrigger::OnPoke,
        "random_chance(50) ? message(\"🎁 A small treat pops out!\") : message(\"💨 The box releases a puff of dust.\")".to_string()
    );
    objects.push(mystery_box);

    // 4. Quest Clue - OnLook with dynamic description
    let mut quest_clue = ObjectRecord {
        id: "example_quest_clue".to_string(),
        name: "Tattered Note".to_string(),
        description: "A yellowed piece of parchment with faded ink.".to_string(),
        owner: ObjectOwner::World,
        created_at: now,
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 5,
        takeable: true,
        usable: false,
        actions: std::collections::HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: OBJECT_SCHEMA_VERSION,
    };
    quest_clue.actions.insert(
        ObjectTrigger::OnLook,
        "message(\"The note reads: 'Meet me at the old gazebo when the moon is high. - M'\")"
            .to_string(),
    );
    objects.push(quest_clue);

    // 5. Teleport Stone - OnUse teleports player
    let mut teleport_stone = ObjectRecord {
        id: "example_teleport_stone".to_string(),
        name: "Teleport Stone".to_string(),
        description: "A smooth black stone that hums with magical energy.".to_string(),
        owner: ObjectOwner::World,
        created_at: now,
        weight: 2,
        currency_value: CurrencyAmount::default(),
        value: 500,
        takeable: true,
        usable: true,
        actions: std::collections::HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: OBJECT_SCHEMA_VERSION,
    };
    teleport_stone.actions.insert(
        ObjectTrigger::OnUse,
        "message(\"✨ The stone flashes brilliantly!\") && teleport(\"old_towne_square\")"
            .to_string(),
    );
    objects.push(teleport_stone);

    // 6. Singing Mushroom - OnEnter ambient message
    let mut singing_mushroom = ObjectRecord {
        id: "example_singing_mushroom".to_string(),
        name: "Singing Mushroom".to_string(),
        description: "A peculiar purple mushroom that hums softly to itself.".to_string(),
        owner: ObjectOwner::World,
        created_at: now,
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 10,
        takeable: true,
        usable: false,
        actions: std::collections::HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: OBJECT_SCHEMA_VERSION,
    };
    singing_mushroom.actions.insert(
        ObjectTrigger::OnEnter,
        "message(\"🍄 The mushroom chimes a cheerful tune as you enter the room!\")".to_string(),
    );
    objects.push(singing_mushroom);

    objects
}
