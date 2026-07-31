use woc_client::{build_quest_log_view, QuestLogWindowState};
use woc_runtime::{
    HudQuestState, QuestLogEntryProjection, QuestLogObjectiveProjection, QuestLogWindowProjection,
};

fn quest(
    quest_id: &str,
    acceptance_order: u16,
    state: HudQuestState,
    current: u32,
    required: u32,
) -> QuestLogEntryProjection {
    QuestLogEntryProjection {
        quest_id: quest_id.into(),
        acceptance_order,
        state,
        suggested_players: Some(2),
        objectives: vec![QuestLogObjectiveProjection {
            objective_index: 0,
            current,
            required,
        }],
        xp_reward: 250,
        copper_reward: 80,
        reward_item_id: Some("milepost_boots".into()),
        turn_in_npc_id: "npc_marshal_redbrook".into(),
    }
}

fn projection() -> QuestLogWindowProjection {
    QuestLogWindowProjection {
        completed_count: 7,
        quests: vec![
            quest("q_wolves", 1, HudQuestState::Active, 2, 5),
            quest("q_bandits", 2, HudQuestState::Ready, 4, 4),
        ],
    }
}

#[test]
fn absent_or_stale_selection_falls_back_to_the_first_accepted_quest() {
    let projection = projection();
    for selected in [None, Some("missing")].map(|value| value.map(str::to_string)) {
        let view = build_quest_log_view(&projection, selected.as_deref());
        assert_eq!(view.selected_quest_id.as_deref(), Some("q_wolves"));
        assert_eq!(
            view.items
                .iter()
                .map(|item| item.selected)
                .collect::<Vec<_>>(),
            vec![true, false]
        );
    }
}

#[test]
fn list_summary_and_selected_detail_are_projection_only_and_deterministic() {
    let projection = projection();
    let first = build_quest_log_view(&projection, Some("q_bandits"));
    let second = build_quest_log_view(&projection, Some("q_bandits"));

    assert_eq!(first, second);
    assert_eq!(first.summary.active, 2);
    assert_eq!(first.summary.completed, 7);
    assert!(!first.empty);
    assert!(!first.items[0].ready);
    assert!(first.items[1].ready);
    assert!(first.items[1].selected);

    let detail = first.detail.expect("selected quest detail");
    assert_eq!(detail.quest_id, "q_bandits");
    assert_eq!(detail.suggested_players, Some(2));
    assert!(detail.objectives[0].done);
    assert_eq!(detail.xp_reward, 250);
    assert_eq!(detail.copper_reward, 80);
    assert_eq!(detail.reward_item_id.as_deref(), Some("milepost_boots"));
    assert_eq!(detail.turn_in_npc_id, "npc_marshal_redbrook");
}

#[test]
fn empty_log_has_no_selection_or_detail_but_keeps_completed_summary() {
    let view = build_quest_log_view(
        &QuestLogWindowProjection {
            completed_count: 3,
            quests: Vec::new(),
        },
        Some("stale"),
    );

    assert!(view.empty);
    assert!(view.items.is_empty());
    assert_eq!(view.summary.active, 0);
    assert_eq!(view.summary.completed, 3);
    assert_eq!(view.selected_quest_id, None);
    assert_eq!(view.detail, None);
}

#[test]
fn window_state_preserves_valid_selection_and_repairs_it_after_projection_change() {
    let mut state = QuestLogWindowState::default();
    let projection = projection();
    assert!(state.select(&projection, "q_bandits"));
    assert_eq!(state.selected_quest_id(), Some("q_bandits"));
    assert!(!state.select(&projection, "missing"));
    assert_eq!(state.selected_quest_id(), Some("q_bandits"));

    let mut changed = projection.clone();
    changed.quests.pop();
    let view = state.render(&changed);
    assert_eq!(view.selected_quest_id.as_deref(), Some("q_wolves"));
    assert_eq!(state.selected_quest_id(), Some("q_wolves"));

    state.open_with_quest("q_bandits");
    let view = state.render(&projection);
    assert_eq!(view.selected_quest_id.as_deref(), Some("q_bandits"));
}
