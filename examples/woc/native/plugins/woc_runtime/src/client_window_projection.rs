use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::client_hud_projection::{HudProjection, HudQuestState};

pub const INVENTORY_BAG_SOCKET_COUNT: usize = 4;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EquippedBagProjection {
    pub item_id: String,
    pub slots: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct InventoryItemProjection {
    /// Dense authoritative inventory order. Commands address this index, while
    /// `cell_hint` only controls the manual bag layout.
    pub inventory_index: u16,
    /// Optional fixed bag cell. Duplicate and out-of-range legacy hints remain
    /// valid; the client lays those stacks into the first free cells.
    pub cell_hint: Option<u16>,
    pub item_id: String,
    pub count: u32,
    pub instance_id: Option<u64>,
    pub soulbound: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct InventoryWindowProjection {
    pub backpack_slots: u16,
    pub capacity: u16,
    pub copper: u64,
    pub bags: Vec<Option<EquippedBagProjection>>,
    pub items: Vec<InventoryItemProjection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct QuestLogObjectiveProjection {
    pub objective_index: u16,
    pub current: u32,
    pub required: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct QuestLogEntryProjection {
    pub quest_id: String,
    pub acceptance_order: u16,
    pub state: HudQuestState,
    pub suggested_players: Option<u16>,
    pub objectives: Vec<QuestLogObjectiveProjection>,
    pub xp_reward: u32,
    pub copper_reward: u64,
    pub reward_item_id: Option<String>,
    pub turn_in_npc_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct QuestLogWindowProjection {
    pub completed_count: u32,
    pub quests: Vec<QuestLogEntryProjection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ClientWindowProjection {
    pub inventory: InventoryWindowProjection,
    pub quest_log: QuestLogWindowProjection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryProjectionError {
    InvalidBackpackSlots,
    BagSocketCount { actual: usize, expected: usize },
    InvalidBagField { socket: usize, field: &'static str },
    CapacityOverflow,
    CapacityMismatch { actual: u16, expected: u16 },
    ItemInventoryOrder { index: usize, actual: u16 },
    InvalidItemField { index: usize, field: &'static str },
    DuplicateInstanceId { index: usize, instance_id: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestLogProjectionError {
    QuestAcceptanceOrder {
        index: usize,
        actual: u16,
    },
    DuplicateQuestId {
        index: usize,
        quest_id: String,
    },
    InvalidQuestField {
        index: usize,
        field: &'static str,
    },
    ObjectiveOrder {
        quest_index: usize,
        objective_index: usize,
        actual: u16,
    },
    ObjectiveRequiredZero {
        quest_index: usize,
        objective_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowProjectionError {
    Inventory(InventoryProjectionError),
    QuestLog(QuestLogProjectionError),
    TrackedQuestCount {
        tracker: usize,
        quest_log: usize,
    },
    TrackedQuestIdMismatch {
        quest_index: usize,
        tracker_id: String,
        quest_log_id: String,
    },
    TrackedQuestStateMismatch {
        quest_index: usize,
    },
    TrackedQuestObjectiveCount {
        quest_index: usize,
        tracker: usize,
        quest_log: usize,
    },
    TrackedQuestObjectiveMismatch {
        quest_index: usize,
        objective_index: usize,
    },
}

impl ClientWindowProjection {
    pub fn validate_against(&self, hud: &HudProjection) -> Result<(), WindowProjectionError> {
        self.inventory
            .validate()
            .map_err(WindowProjectionError::Inventory)?;
        self.quest_log
            .validate()
            .map_err(WindowProjectionError::QuestLog)?;
        self.validate_tracked_quests(hud)
    }

    fn validate_tracked_quests(&self, hud: &HudProjection) -> Result<(), WindowProjectionError> {
        if hud.tracked_quests.len() != self.quest_log.quests.len() {
            return Err(WindowProjectionError::TrackedQuestCount {
                tracker: hud.tracked_quests.len(),
                quest_log: self.quest_log.quests.len(),
            });
        }
        for (quest_index, (tracked, logged)) in hud
            .tracked_quests
            .iter()
            .zip(&self.quest_log.quests)
            .enumerate()
        {
            if tracked.quest_id != logged.quest_id {
                return Err(WindowProjectionError::TrackedQuestIdMismatch {
                    quest_index,
                    tracker_id: tracked.quest_id.clone(),
                    quest_log_id: logged.quest_id.clone(),
                });
            }
            if logged.state != tracked.state {
                return Err(WindowProjectionError::TrackedQuestStateMismatch { quest_index });
            }
            if logged.objectives.len() != tracked.objectives.len() {
                return Err(WindowProjectionError::TrackedQuestObjectiveCount {
                    quest_index,
                    tracker: tracked.objectives.len(),
                    quest_log: logged.objectives.len(),
                });
            }
            for (objective_index, (tracker, quest_log)) in tracked
                .objectives
                .iter()
                .zip(&logged.objectives)
                .enumerate()
            {
                if tracker.objective_index != quest_log.objective_index
                    || tracker.current != quest_log.current
                    || tracker.required != quest_log.required
                {
                    return Err(WindowProjectionError::TrackedQuestObjectiveMismatch {
                        quest_index,
                        objective_index,
                    });
                }
            }
        }
        Ok(())
    }
}

impl InventoryWindowProjection {
    fn validate(&self) -> Result<(), InventoryProjectionError> {
        if self.backpack_slots == 0 {
            return Err(InventoryProjectionError::InvalidBackpackSlots);
        }
        if self.bags.len() != INVENTORY_BAG_SOCKET_COUNT {
            return Err(InventoryProjectionError::BagSocketCount {
                actual: self.bags.len(),
                expected: INVENTORY_BAG_SOCKET_COUNT,
            });
        }

        let mut expected_capacity = self.backpack_slots;
        for (socket, bag) in self.bags.iter().enumerate() {
            let Some(bag) = bag else {
                continue;
            };
            if bag.item_id.is_empty() {
                return Err(InventoryProjectionError::InvalidBagField {
                    socket,
                    field: "item_id",
                });
            }
            if bag.slots == 0 {
                return Err(InventoryProjectionError::InvalidBagField {
                    socket,
                    field: "slots",
                });
            }
            expected_capacity = expected_capacity
                .checked_add(bag.slots)
                .ok_or(InventoryProjectionError::CapacityOverflow)?;
        }
        if self.capacity != expected_capacity {
            return Err(InventoryProjectionError::CapacityMismatch {
                actual: self.capacity,
                expected: expected_capacity,
            });
        }

        let mut instance_ids = HashSet::new();
        for (index, item) in self.items.iter().enumerate() {
            if usize::from(item.inventory_index) != index {
                return Err(InventoryProjectionError::ItemInventoryOrder {
                    index,
                    actual: item.inventory_index,
                });
            }
            if item.item_id.is_empty() {
                return Err(InventoryProjectionError::InvalidItemField {
                    index,
                    field: "item_id",
                });
            }
            if item.count == 0 {
                return Err(InventoryProjectionError::InvalidItemField {
                    index,
                    field: "count",
                });
            }
            if let Some(instance_id) = item.instance_id {
                if instance_id == 0 {
                    return Err(InventoryProjectionError::InvalidItemField {
                        index,
                        field: "instance_id",
                    });
                }
                if !instance_ids.insert(instance_id) {
                    return Err(InventoryProjectionError::DuplicateInstanceId {
                        index,
                        instance_id,
                    });
                }
            }
        }
        Ok(())
    }
}

impl QuestLogWindowProjection {
    fn validate(&self) -> Result<(), QuestLogProjectionError> {
        let mut quest_ids = HashSet::new();
        for (quest_index, quest) in self.quests.iter().enumerate() {
            if usize::from(quest.acceptance_order) != quest_index + 1 {
                return Err(QuestLogProjectionError::QuestAcceptanceOrder {
                    index: quest_index,
                    actual: quest.acceptance_order,
                });
            }
            if quest.quest_id.is_empty() {
                return Err(QuestLogProjectionError::InvalidQuestField {
                    index: quest_index,
                    field: "quest_id",
                });
            }
            if !quest_ids.insert(quest.quest_id.as_str()) {
                return Err(QuestLogProjectionError::DuplicateQuestId {
                    index: quest_index,
                    quest_id: quest.quest_id.clone(),
                });
            }
            if quest
                .suggested_players
                .is_some_and(|suggested_players| suggested_players == 0)
            {
                return Err(QuestLogProjectionError::InvalidQuestField {
                    index: quest_index,
                    field: "suggested_players",
                });
            }
            if quest.reward_item_id.as_ref().is_some_and(String::is_empty) {
                return Err(QuestLogProjectionError::InvalidQuestField {
                    index: quest_index,
                    field: "reward_item_id",
                });
            }
            if quest.turn_in_npc_id.is_empty() {
                return Err(QuestLogProjectionError::InvalidQuestField {
                    index: quest_index,
                    field: "turn_in_npc_id",
                });
            }
            for (objective_index, objective) in quest.objectives.iter().enumerate() {
                if usize::from(objective.objective_index) != objective_index {
                    return Err(QuestLogProjectionError::ObjectiveOrder {
                        quest_index,
                        objective_index,
                        actual: objective.objective_index,
                    });
                }
                if objective.required == 0 {
                    return Err(QuestLogProjectionError::ObjectiveRequiredZero {
                        quest_index,
                        objective_index,
                    });
                }
            }
        }
        Ok(())
    }
}
