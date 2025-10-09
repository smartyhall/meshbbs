use chrono::{DateTime, Utc};

use crate::tmush::types::{
    AchievementCategory, AchievementRecord, AchievementTrigger, Direction, RoomFlag, RoomRecord,
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
        "A quiet gazebo overlooking Old Towne Mesh.",
        "Soft mesh lanterns line the gazebo railing while onboarding scripts hum to life.
Wizards finalize character details here before stepping into the public square.",
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
        CompanionRecord::new("gentle_mare", "Gentle Mare", CompanionType::Horse, "south_market")
            .with_description("A gentle brown mare with kind eyes. She seems eager for a rider."),
    );

    // A loyal dog at the town square
    companions.push(
        CompanionRecord::new("loyal_hound", "Loyal Hound", CompanionType::Dog, REQUIRED_START_LOCATION_ID)
            .with_description("A friendly dog with alert eyes. He wags his tail hopefully."),
    );

    // A mysterious cat near the museum
    companions.push(
        CompanionRecord::new("shadow_cat", "Shadow Cat", CompanionType::Cat, "mesh_museum")
            .with_description("A sleek black cat with piercing green eyes. She watches you intently."),
    );

    companions
}
