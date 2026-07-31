use std::cmp::Ordering;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use woc_protocol::EntityRef;

use crate::client_projection::{BulkPresentationProjection, PresentationProjectionError};
use crate::client_window_projection::{ClientWindowProjection, WindowProjectionError};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct HudMeter {
    pub current: f32,
    pub maximum: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HudResourceKind {
    Mana,
    Rage,
    Energy,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct HudResource {
    pub kind: HudResourceKind,
    pub meter: HudMeter,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HudCast {
    pub ability_id: String,
    pub remaining: f32,
    pub total: f32,
    pub channeling: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HudUnit {
    pub entity: EntityRef,
    pub display_name: String,
    pub title_id: Option<String>,
    pub level: u16,
    pub health: HudMeter,
    pub resource: Option<HudResource>,
    pub absorb: f32,
    pub dead: bool,
    pub hostile: bool,
    pub elite: bool,
    pub boss: bool,
    pub cast: Option<HudCast>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "content_id", rename_all = "snake_case")]
pub enum HudActionId {
    Attack,
    Ability(String),
    Item(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HudAction {
    pub id: HudActionId,
    pub cooldown_remaining: f32,
    pub cooldown_total: f32,
    pub count: u32,
    pub usable: bool,
    pub out_of_range: bool,
    pub queued: bool,
    pub proc_glow: bool,
    pub empowered: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HudQuestState {
    Active,
    Ready,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct HudQuestObjective {
    pub objective_index: u16,
    pub current: u32,
    pub required: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct HudTrackedQuest {
    pub quest_id: String,
    pub acceptance_order: u16,
    pub state: HudQuestState,
    pub objectives: Vec<HudQuestObjective>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HudProjection {
    pub player: HudUnit,
    pub target: Option<HudUnit>,
    pub target_of_target: Option<HudUnit>,
    pub combo_points: u8,
    pub actions: Vec<HudAction>,
    pub tracked_quests: Vec<HudTrackedQuest>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ClientPresentationProjection {
    pub schema_version: u16,
    pub world: BulkPresentationProjection,
    pub hud: HudProjection,
    pub windows: ClientWindowProjection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudUnitRole {
    Player,
    Target,
    TargetOfTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HudProjectionError {
    PlayerIsNotViewer {
        player: EntityRef,
        viewer: EntityRef,
    },
    UnitActorMissing {
        role: HudUnitRole,
        entity: EntityRef,
    },
    TargetOfTargetWithoutTarget,
    InvalidUnitField {
        role: HudUnitRole,
        field: &'static str,
    },
    ComboPointsOutOfRange {
        actual: u8,
    },
    ActionsNotStrictlySorted {
        index: usize,
    },
    InvalidActionId {
        index: usize,
    },
    InvalidActionField {
        index: usize,
        field: &'static str,
    },
    QuestAcceptanceOrder {
        index: usize,
        actual: u16,
    },
    InvalidQuestId {
        index: usize,
    },
    DuplicateQuestId {
        index: usize,
        quest_id: String,
    },
    QuestObjectiveOrder {
        quest_index: usize,
        objective_index: usize,
        actual: u16,
    },
    QuestObjectiveRequiredZero {
        quest_index: usize,
        objective_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientProjectionError {
    UnsupportedSchemaVersion { actual: u16, expected: u16 },
    Actors(PresentationProjectionError),
    Hud(HudProjectionError),
    Windows(WindowProjectionError),
}

pub const CLIENT_PRESENTATION_SCHEMA_VERSION: u16 = 2;
pub const MAX_CLIENT_PRESENTATION_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientProjectionCodecError {
    PayloadTooLarge { actual: usize, maximum: usize },
    Json(String),
    Projection(ClientProjectionError),
}

impl ClientPresentationProjection {
    pub fn validate(&self) -> Result<(), ClientProjectionError> {
        if self.schema_version != CLIENT_PRESENTATION_SCHEMA_VERSION {
            return Err(ClientProjectionError::UnsupportedSchemaVersion {
                actual: self.schema_version,
                expected: CLIENT_PRESENTATION_SCHEMA_VERSION,
            });
        }
        self.world
            .validate()
            .map_err(ClientProjectionError::Actors)?;
        self.hud
            .validate_against(&self.world)
            .map_err(ClientProjectionError::Hud)?;
        self.windows
            .validate_against(&self.hud)
            .map_err(ClientProjectionError::Windows)
    }

    pub fn encode_json(&self) -> Result<Vec<u8>, ClientProjectionCodecError> {
        self.validate()
            .map_err(ClientProjectionCodecError::Projection)?;
        let encoded = serde_json::to_vec(self)
            .map_err(|error| ClientProjectionCodecError::Json(error.to_string()))?;
        require_payload_bound(encoded.len())?;
        Ok(encoded)
    }

    pub fn decode_json(bytes: &[u8]) -> Result<Self, ClientProjectionCodecError> {
        require_payload_bound(bytes.len())?;
        let projection: Self = serde_json::from_slice(bytes)
            .map_err(|error| ClientProjectionCodecError::Json(error.to_string()))?;
        projection
            .validate()
            .map_err(ClientProjectionCodecError::Projection)?;
        Ok(projection)
    }
}

fn require_payload_bound(actual: usize) -> Result<(), ClientProjectionCodecError> {
    if actual > MAX_CLIENT_PRESENTATION_BYTES {
        Err(ClientProjectionCodecError::PayloadTooLarge {
            actual,
            maximum: MAX_CLIENT_PRESENTATION_BYTES,
        })
    } else {
        Ok(())
    }
}

impl HudProjection {
    fn validate_against(
        &self,
        world: &BulkPresentationProjection,
    ) -> Result<(), HudProjectionError> {
        if self.player.entity != world.viewer {
            return Err(HudProjectionError::PlayerIsNotViewer {
                player: self.player.entity,
                viewer: world.viewer,
            });
        }

        validate_unit(&self.player, HudUnitRole::Player, world)?;
        if let Some(target) = &self.target {
            validate_unit(target, HudUnitRole::Target, world)?;
        }
        if let Some(target_of_target) = &self.target_of_target {
            if self.target.is_none() {
                return Err(HudProjectionError::TargetOfTargetWithoutTarget);
            }
            validate_unit(target_of_target, HudUnitRole::TargetOfTarget, world)?;
        }
        if self.combo_points > 5 {
            return Err(HudProjectionError::ComboPointsOutOfRange {
                actual: self.combo_points,
            });
        }
        validate_actions(&self.actions)?;
        validate_quests(&self.tracked_quests)
    }
}

fn validate_unit(
    unit: &HudUnit,
    role: HudUnitRole,
    world: &BulkPresentationProjection,
) -> Result<(), HudProjectionError> {
    if world.actor(unit.entity).is_none() {
        return Err(HudProjectionError::UnitActorMissing {
            role,
            entity: unit.entity,
        });
    }
    if unit.display_name.is_empty() {
        return Err(HudProjectionError::InvalidUnitField {
            role,
            field: "display_name",
        });
    }
    if unit.title_id.as_ref().is_some_and(String::is_empty) {
        return Err(HudProjectionError::InvalidUnitField {
            role,
            field: "title_id",
        });
    }
    validate_meter(unit.health, role, "health")?;
    if let Some(resource) = unit.resource {
        validate_meter(resource.meter, role, "resource")?;
    }
    if !unit.absorb.is_finite() || unit.absorb < 0.0 {
        return Err(HudProjectionError::InvalidUnitField {
            role,
            field: "absorb",
        });
    }
    if let Some(cast) = &unit.cast {
        if cast.ability_id.is_empty() {
            return Err(HudProjectionError::InvalidUnitField {
                role,
                field: "cast.ability_id",
            });
        }
        if !cast.remaining.is_finite() || cast.remaining < 0.0 {
            return Err(HudProjectionError::InvalidUnitField {
                role,
                field: "cast.remaining",
            });
        }
        if !cast.total.is_finite() || cast.total <= 0.0 {
            return Err(HudProjectionError::InvalidUnitField {
                role,
                field: "cast.total",
            });
        }
    }
    Ok(())
}

fn validate_meter(
    meter: HudMeter,
    role: HudUnitRole,
    field: &'static str,
) -> Result<(), HudProjectionError> {
    if !meter.current.is_finite() || meter.current < 0.0 {
        return Err(HudProjectionError::InvalidUnitField { role, field });
    }
    if !meter.maximum.is_finite() || meter.maximum <= 0.0 {
        return Err(HudProjectionError::InvalidUnitField { role, field });
    }
    Ok(())
}

fn validate_actions(actions: &[HudAction]) -> Result<(), HudProjectionError> {
    for (index, action) in actions.iter().enumerate() {
        if matches!(
            &action.id,
            HudActionId::Ability(id) | HudActionId::Item(id) if id.is_empty()
        ) {
            return Err(HudProjectionError::InvalidActionId { index });
        }
        for (field, value) in [
            ("cooldown_remaining", action.cooldown_remaining),
            ("cooldown_total", action.cooldown_total),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(HudProjectionError::InvalidActionField { index, field });
            }
        }
    }
    for (index, pair) in actions.windows(2).enumerate() {
        if compare_action_id(&pair[0].id, &pair[1].id) != Ordering::Less {
            return Err(HudProjectionError::ActionsNotStrictlySorted { index: index + 1 });
        }
    }
    Ok(())
}

fn compare_action_id(left: &HudActionId, right: &HudActionId) -> Ordering {
    match (left, right) {
        (HudActionId::Attack, HudActionId::Attack) => Ordering::Equal,
        (HudActionId::Attack, _) => Ordering::Less,
        (_, HudActionId::Attack) => Ordering::Greater,
        (HudActionId::Ability(left), HudActionId::Ability(right)) => left.cmp(right),
        (HudActionId::Ability(_), HudActionId::Item(_)) => Ordering::Less,
        (HudActionId::Item(_), HudActionId::Ability(_)) => Ordering::Greater,
        (HudActionId::Item(left), HudActionId::Item(right)) => left.cmp(right),
    }
}

fn validate_quests(quests: &[HudTrackedQuest]) -> Result<(), HudProjectionError> {
    let mut quest_ids = HashSet::new();
    for (quest_index, quest) in quests.iter().enumerate() {
        if usize::from(quest.acceptance_order) != quest_index + 1 {
            return Err(HudProjectionError::QuestAcceptanceOrder {
                index: quest_index,
                actual: quest.acceptance_order,
            });
        }
        if quest.quest_id.is_empty() {
            return Err(HudProjectionError::InvalidQuestId { index: quest_index });
        }
        if !quest_ids.insert(quest.quest_id.as_str()) {
            return Err(HudProjectionError::DuplicateQuestId {
                index: quest_index,
                quest_id: quest.quest_id.clone(),
            });
        }
        for (objective_index, objective) in quest.objectives.iter().enumerate() {
            if usize::from(objective.objective_index) != objective_index {
                return Err(HudProjectionError::QuestObjectiveOrder {
                    quest_index,
                    objective_index,
                    actual: objective.objective_index,
                });
            }
            if objective.required == 0 {
                return Err(HudProjectionError::QuestObjectiveRequiredZero {
                    quest_index,
                    objective_index,
                });
            }
        }
    }
    Ok(())
}
