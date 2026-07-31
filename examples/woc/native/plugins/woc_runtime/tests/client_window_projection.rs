use woc_protocol::EntityRef;
use woc_runtime::{
    ClientWindowProjection, EquippedBagProjection, HudAction, HudActionId, HudMeter, HudProjection,
    HudQuestObjective, HudQuestState, HudTrackedQuest, HudUnit, InventoryItemProjection,
    InventoryProjectionError, InventoryWindowProjection, QuestLogEntryProjection,
    QuestLogObjectiveProjection, QuestLogProjectionError, QuestLogWindowProjection,
    WindowProjectionError,
};

fn unit() -> HudUnit {
    HudUnit {
        entity: EntityRef {
            id: 1,
            generation: 1,
        },
        display_name: "Hero".into(),
        title_id: None,
        level: 4,
        health: HudMeter {
            current: 140.0,
            maximum: 160.0,
        },
        resource: None,
        absorb: 0.0,
        dead: false,
        hostile: false,
        elite: false,
        boss: false,
        cast: None,
    }
}

fn tracked_quest() -> HudTrackedQuest {
    HudTrackedQuest {
        quest_id: "q_wolves".into(),
        acceptance_order: 1,
        state: HudQuestState::Active,
        objectives: vec![HudQuestObjective {
            objective_index: 0,
            current: 2,
            required: 5,
        }],
    }
}

fn hud() -> HudProjection {
    HudProjection {
        player: unit(),
        target: None,
        target_of_target: None,
        combo_points: 0,
        actions: vec![HudAction {
            id: HudActionId::Attack,
            cooldown_remaining: 0.0,
            cooldown_total: 0.0,
            count: 1,
            usable: true,
            out_of_range: false,
            queued: false,
            proc_glow: false,
            empowered: false,
        }],
        tracked_quests: vec![tracked_quest()],
    }
}

fn inventory() -> InventoryWindowProjection {
    InventoryWindowProjection {
        backpack_slots: 16,
        capacity: 24,
        copper: 1234,
        bags: vec![
            Some(EquippedBagProjection {
                item_id: "small_pouch".into(),
                slots: 8,
            }),
            None,
            None,
            None,
        ],
        items: vec![
            InventoryItemProjection {
                inventory_index: 0,
                cell_hint: Some(0),
                item_id: "rusted_sword".into(),
                count: 1,
                instance_id: Some(41),
                soulbound: true,
            },
            InventoryItemProjection {
                inventory_index: 1,
                cell_hint: Some(7),
                item_id: "wolf_pelt".into(),
                count: 3,
                instance_id: None,
                soulbound: false,
            },
        ],
    }
}

fn quest_log() -> QuestLogWindowProjection {
    QuestLogWindowProjection {
        completed_count: 2,
        quests: vec![QuestLogEntryProjection {
            quest_id: "q_wolves".into(),
            acceptance_order: 1,
            state: HudQuestState::Active,
            suggested_players: None,
            objectives: vec![QuestLogObjectiveProjection {
                objective_index: 0,
                current: 2,
                required: 5,
            }],
            xp_reward: 250,
            copper_reward: 80,
            reward_item_id: Some("milepost_boots".into()),
            turn_in_npc_id: "npc_marshal_arden".into(),
        }],
    }
}

fn window_projection() -> ClientWindowProjection {
    ClientWindowProjection {
        inventory: inventory(),
        quest_log: quest_log(),
    }
}

#[test]
fn valid_windows_preserve_sparse_bag_cells_and_authoritative_quest_details() {
    let windows = window_projection();
    windows.validate_against(&hud()).expect("valid windows");

    assert_eq!(windows.inventory.items[1].cell_hint, Some(7));
    assert_eq!(windows.inventory.capacity, 24);
    assert_eq!(windows.quest_log.quests[0].xp_reward, 250);
    assert_eq!(
        windows.quest_log.quests[0].reward_item_id.as_deref(),
        Some("milepost_boots")
    );
}

#[test]
fn inventory_capacity_is_derived_from_backpack_and_four_bag_sockets() {
    let mut windows = window_projection();
    windows.inventory.bags.pop();
    assert_eq!(
        windows.validate_against(&hud()).expect_err("four sockets"),
        WindowProjectionError::Inventory(InventoryProjectionError::BagSocketCount {
            actual: 3,
            expected: 4,
        })
    );

    windows = window_projection();
    windows.inventory.capacity = 23;
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("derived capacity"),
        WindowProjectionError::Inventory(InventoryProjectionError::CapacityMismatch {
            actual: 23,
            expected: 24,
        })
    );
}

#[test]
fn inventory_items_require_dense_order_and_valid_stack_fields() {
    let mut windows = window_projection();
    windows.inventory.items[1].inventory_index = 0;
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("duplicate index"),
        WindowProjectionError::Inventory(InventoryProjectionError::ItemInventoryOrder {
            index: 1,
            actual: 0,
        })
    );

    windows = window_projection();
    windows.inventory.items[1].cell_hint = Some(0);
    windows
        .validate_against(&hud())
        .expect("duplicate legacy cell hints are laid out deterministically");

    windows = window_projection();
    windows.inventory.items[0].count = 0;
    assert_eq!(
        windows.validate_against(&hud()).expect_err("zero count"),
        WindowProjectionError::Inventory(InventoryProjectionError::InvalidItemField {
            index: 0,
            field: "count",
        })
    );
}

#[test]
fn quest_log_requires_canonical_quest_and_objective_order() {
    let mut windows = window_projection();
    windows.quest_log.quests[0].acceptance_order = 2;
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("quest acceptance order"),
        WindowProjectionError::QuestLog(QuestLogProjectionError::QuestAcceptanceOrder {
            index: 0,
            actual: 2,
        })
    );

    windows = window_projection();
    windows.quest_log.quests[0].objectives[0].objective_index = 1;
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("objective order"),
        WindowProjectionError::QuestLog(QuestLogProjectionError::ObjectiveOrder {
            quest_index: 0,
            objective_index: 0,
            actual: 1,
        })
    );
}

#[test]
fn tracked_quest_progress_must_match_the_window_projection() {
    let mut windows = window_projection();
    windows.quest_log.quests[0].objectives[0].current = 3;
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("tracker and log drift"),
        WindowProjectionError::TrackedQuestObjectiveMismatch {
            quest_index: 0,
            objective_index: 0,
        }
    );
}

#[test]
fn tracker_and_quest_log_are_the_same_ordered_active_quest_set() {
    let mut windows = window_projection();
    windows.quest_log.quests.clear();
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("tracker cannot outlive quest log"),
        WindowProjectionError::TrackedQuestCount {
            tracker: 1,
            quest_log: 0,
        }
    );

    windows = window_projection();
    windows.quest_log.quests[0].quest_id = "q_bandits".into();
    assert_eq!(
        windows
            .validate_against(&hud())
            .expect_err("tracker and log must share order"),
        WindowProjectionError::TrackedQuestIdMismatch {
            quest_index: 0,
            tracker_id: "q_wolves".into(),
            quest_log_id: "q_bandits".into(),
        }
    );
}
