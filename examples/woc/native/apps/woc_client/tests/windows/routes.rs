use woc_client::{
    decode_inventory_filter, ClientGameplayIntent, ClientWindowController, ClientWindowId,
    ClientWindowRouteEffect, ClientWindowRouteError, InventoryCategory, InventoryInteractionMode,
    InventoryItemKind, InventoryItemPresentation, InventoryItemView, InventoryPrimaryAction,
    InventoryQuality, InventorySort, OptionsPanelId,
};
use woc_runtime::{
    ClientWindowProjection, EquippedBagProjection, HudQuestState, InventoryItemProjection,
    InventoryWindowProjection, QuestLogEntryProjection, QuestLogObjectiveProjection,
    QuestLogWindowProjection,
};

fn quest(quest_id: &str, order: u16, state: HudQuestState) -> QuestLogEntryProjection {
    QuestLogEntryProjection {
        quest_id: quest_id.into(),
        acceptance_order: order,
        state,
        suggested_players: None,
        objectives: vec![QuestLogObjectiveProjection {
            objective_index: 0,
            current: if state == HudQuestState::Ready { 3 } else { 1 },
            required: 3,
        }],
        xp_reward: 100,
        copper_reward: 25,
        reward_item_id: None,
        turn_in_npc_id: "npc_marshal_redbrook".into(),
    }
}

fn windows() -> ClientWindowProjection {
    ClientWindowProjection {
        inventory: InventoryWindowProjection {
            backpack_slots: 16,
            capacity: 24,
            copper: 99,
            bags: vec![
                Some(EquippedBagProjection {
                    item_id: "small_pouch".into(),
                    slots: 8,
                }),
                None,
                None,
                None,
            ],
            items: Vec::new(),
        },
        quest_log: QuestLogWindowProjection {
            completed_count: 2,
            quests: vec![
                quest("q_wolves", 1, HudQuestState::Active),
                quest("q_bandits", 2, HudQuestState::Ready),
            ],
        },
    }
}

#[test]
fn inventory_routes_mutate_only_local_filter_state_and_emit_persistence() {
    let windows = windows();
    let before_projection = windows.clone();
    let mut controller = ClientWindowController::new(Some(
        r#"{"category":"armor","sort":"name","search":"iron"}"#,
    ));
    controller.open_inventory();

    let effect = controller
        .handle_route(&windows, "woc.window.inventory.filter.weapon", None)
        .expect("weapon filter route");
    let ClientWindowRouteEffect::PersistInventoryFilter(encoded) = effect else {
        panic!("filter route must request persistence");
    };
    let stored = decode_inventory_filter(Some(&encoded));
    assert_eq!(stored.category, InventoryCategory::Weapon);
    assert_eq!(stored.sort, InventorySort::Name);
    assert_eq!(stored.search, "iron");

    controller
        .handle_route(&windows, "woc.window.inventory.sort.quality", None)
        .expect("quality sort route");
    controller
        .handle_route(&windows, "woc.window.inventory.search", Some("  wolf  "))
        .expect("search route");
    assert_eq!(controller.inventory_filter().sort, InventorySort::Quality);
    assert_eq!(controller.inventory_filter().search, "  wolf  ");
    assert_eq!(windows, before_projection);
}

#[test]
fn equipped_bag_socket_routes_emit_authority_intent_and_empty_sockets_refuse() {
    let windows = windows();
    let mut controller = ClientWindowController::new(None);
    controller.open_inventory();

    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.inventory.bag_socket.0", None,)
            .expect("equipped bag"),
        ClientWindowRouteEffect::Authority(ClientGameplayIntent::UnequipBag { socket: 0 })
    );
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.inventory.bag_socket.1", None,)
            .expect_err("empty socket"),
        ClientWindowRouteError::EmptyBagSocket { socket: 1 }
    );
}

#[test]
fn quest_routes_use_the_resolved_selection_and_keep_abandon_behind_confirmation() {
    let windows = windows();
    let mut controller = ClientWindowController::new(None);
    let view = controller.open_quest_log(&windows.quest_log, Some("q_bandits"));
    assert_eq!(view.selected_quest_id.as_deref(), Some("q_bandits"));

    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.quest_log.share", None)
            .expect("share route"),
        ClientWindowRouteEffect::ShareQuest {
            quest_id: "q_bandits".into(),
        }
    );
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.quest_log.abandon", None)
            .expect("abandon route"),
        ClientWindowRouteEffect::ConfirmAbandonQuest {
            quest_id: "q_bandits".into(),
        }
    );

    let mut changed = windows.clone();
    changed.quest_log.quests.pop();
    let view = controller.quest_log_view(&changed.quest_log);
    assert_eq!(view.selected_quest_id.as_deref(), Some("q_wolves"));
    assert_eq!(controller.selected_quest_id(), Some("q_wolves"));
}

#[test]
fn settings_panel_open_gate_and_back_reset_close_effects_match_target_navigation() {
    let windows = windows();
    let mut controller = ClientWindowController::new(None);
    assert_eq!(
        controller
            .open_settings(OptionsPanelId::BugReport, false)
            .expect_err("offline bug report gate"),
        ClientWindowRouteError::UnavailableSettingsPanel(OptionsPanelId::BugReport)
    );
    assert!(!controller.is_open(ClientWindowId::Settings));

    controller
        .open_settings(OptionsPanelId::Graphics, false)
        .expect("graphics panel");
    assert_eq!(controller.settings_panel(), Some(OptionsPanelId::Graphics));
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.settings.reset", None)
            .expect("settings reset"),
        ClientWindowRouteEffect::ResetSettings
    );
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.settings.back", None)
            .expect("settings back"),
        ClientWindowRouteEffect::ShowOptionsMenu
    );
    assert!(!controller.is_open(ClientWindowId::Settings));

    controller
        .open_settings(OptionsPanelId::BugReport, true)
        .expect("online bug report");
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.settings.close", None)
            .expect("settings close"),
        ClientWindowRouteEffect::Closed(ClientWindowId::Settings)
    );
}

#[test]
fn invalid_routes_and_missing_values_are_atomic() {
    let windows = windows();
    let mut controller = ClientWindowController::new(None);
    controller.open_inventory();
    let before = controller.clone();
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.inventory.search", None)
            .expect_err("search value required"),
        ClientWindowRouteError::MissingTextValue
    );
    assert_eq!(controller, before);
    assert_eq!(
        controller
            .handle_route(&windows, "woc.window.inventory.filter.unknown", None)
            .expect_err("unknown route"),
        ClientWindowRouteError::UnknownRoute("woc.window.inventory.filter.unknown".into())
    );
    assert_eq!(controller, before);
}

#[test]
fn dynamic_item_activation_revalidates_the_snapshot_before_deciding_an_action() {
    let mut windows = windows();
    windows.inventory.items.push(InventoryItemProjection {
        inventory_index: 0,
        cell_hint: Some(2),
        item_id: "small_pouch".into(),
        count: 1,
        instance_id: Some(77),
        soulbound: false,
    });
    let clicked = InventoryItemView {
        inventory_index: 0,
        cell_hint: Some(2),
        item_id: "small_pouch".into(),
        count: 1,
        instance_id: Some(77),
        soulbound: false,
        presentation: Some(InventoryItemPresentation::new(
            "small_pouch",
            "Small Pouch",
            InventoryItemKind::Bag,
            Some(InventoryQuality::Common),
        )),
    };
    let mut controller = ClientWindowController::new(None);
    controller.open_inventory();
    assert_eq!(
        controller
            .activate_inventory_item(
                &windows.inventory,
                &clicked,
                InventoryInteractionMode::default()
            )
            .expect("fresh bag click"),
        ClientWindowRouteEffect::InventoryItemAction {
            inventory_index: 0,
            action: InventoryPrimaryAction::EquipBag,
        }
    );

    windows.inventory.items[0].count = 2;
    assert_eq!(
        controller
            .activate_inventory_item(
                &windows.inventory,
                &clicked,
                InventoryInteractionMode::default()
            )
            .expect_err("stale snapshot"),
        ClientWindowRouteError::StaleInventoryItem { inventory_index: 0 }
    );
}
