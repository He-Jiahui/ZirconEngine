use woc_runtime::{HudQuestState, QuestLogWindowProjection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestLogSummary {
    pub active: usize,
    pub completed: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestLogItemView {
    pub quest_id: String,
    pub ready: bool,
    pub selected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestObjectiveView {
    pub objective_index: u16,
    pub current: u32,
    pub required: u32,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestDetailView {
    pub quest_id: String,
    pub suggested_players: Option<u16>,
    pub objectives: Vec<QuestObjectiveView>,
    pub xp_reward: u32,
    pub copper_reward: u64,
    pub reward_item_id: Option<String>,
    pub turn_in_npc_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestLogView {
    pub summary: QuestLogSummary,
    pub items: Vec<QuestLogItemView>,
    pub selected_quest_id: Option<String>,
    pub detail: Option<QuestDetailView>,
    pub empty: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuestLogWindowState {
    selected_quest_id: Option<String>,
}

pub fn build_quest_log_view(
    projection: &QuestLogWindowProjection,
    selected_quest_id: Option<&str>,
) -> QuestLogView {
    let selected_quest_id = selected_quest_id
        .filter(|selected| {
            projection
                .quests
                .iter()
                .any(|quest| quest.quest_id == *selected)
        })
        .map(str::to_owned)
        .or_else(|| {
            projection
                .quests
                .first()
                .map(|quest| quest.quest_id.clone())
        });

    let items = projection
        .quests
        .iter()
        .map(|quest| QuestLogItemView {
            quest_id: quest.quest_id.clone(),
            ready: quest.state == HudQuestState::Ready,
            selected: selected_quest_id.as_deref() == Some(quest.quest_id.as_str()),
        })
        .collect();
    let detail = selected_quest_id.as_deref().and_then(|selected| {
        projection
            .quests
            .iter()
            .find(|quest| quest.quest_id == selected)
            .map(|quest| QuestDetailView {
                quest_id: quest.quest_id.clone(),
                suggested_players: quest.suggested_players,
                objectives: quest
                    .objectives
                    .iter()
                    .map(|objective| QuestObjectiveView {
                        objective_index: objective.objective_index,
                        current: objective.current,
                        required: objective.required,
                        done: objective.current >= objective.required,
                    })
                    .collect(),
                xp_reward: quest.xp_reward,
                copper_reward: quest.copper_reward,
                reward_item_id: quest.reward_item_id.clone(),
                turn_in_npc_id: quest.turn_in_npc_id.clone(),
            })
    });

    QuestLogView {
        summary: QuestLogSummary {
            active: projection.quests.len(),
            completed: projection.completed_count,
        },
        items,
        selected_quest_id,
        detail,
        empty: projection.quests.is_empty(),
    }
}

impl QuestLogWindowState {
    pub fn selected_quest_id(&self) -> Option<&str> {
        self.selected_quest_id.as_deref()
    }

    pub fn select(&mut self, projection: &QuestLogWindowProjection, quest_id: &str) -> bool {
        if !projection
            .quests
            .iter()
            .any(|quest| quest.quest_id == quest_id)
        {
            return false;
        }
        self.selected_quest_id = Some(quest_id.to_owned());
        true
    }

    pub fn open_with_quest(&mut self, quest_id: impl Into<String>) {
        self.selected_quest_id = Some(quest_id.into());
    }

    pub fn render(&mut self, projection: &QuestLogWindowProjection) -> QuestLogView {
        let view = build_quest_log_view(projection, self.selected_quest_id.as_deref());
        self.selected_quest_id.clone_from(&view.selected_quest_id);
        view
    }
}
