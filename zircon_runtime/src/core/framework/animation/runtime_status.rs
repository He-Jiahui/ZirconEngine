use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;
use crate::core::resource::AssetReference;

use super::{AnimationAvatarMask, AnimationGpuSkinningReadiness, AnimationTickReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPlayerKind {
    Clip,
    Sequence,
    Graph,
    StateMachine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPlayerRuntimeState {
    Stopped,
    Playing,
    Paused,
    WaitingForAsset,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlayerRuntimeStatus {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub kind: AnimationPlayerKind,
    pub state: AnimationPlayerRuntimeState,
    pub source: Option<AssetReference>,
    pub active_state: Option<String>,
    #[serde(
        serialize_with = "serialize_sanitized_non_negative_real",
        deserialize_with = "deserialize_sanitized_non_negative_real"
    )]
    pub time_seconds: Real,
    #[serde(
        serialize_with = "serialize_sanitized_non_negative_real",
        deserialize_with = "deserialize_sanitized_non_negative_real"
    )]
    pub playback_speed: Real,
    #[serde(
        serialize_with = "serialize_normalized_real",
        deserialize_with = "deserialize_normalized_real"
    )]
    pub weight: Real,
    pub looping: bool,
    pub diagnostics: Vec<String>,
}

impl AnimationPlayerRuntimeStatus {
    pub fn new(world: WorldHandle, entity: EntityId, kind: AnimationPlayerKind) -> Self {
        Self {
            world,
            entity,
            kind,
            state: AnimationPlayerRuntimeState::Stopped,
            source: None,
            active_state: None,
            time_seconds: 0.0,
            playback_speed: 1.0,
            weight: 1.0,
            looping: false,
            diagnostics: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, AnimationPlayerRuntimeState::Playing)
    }

    pub fn sanitized_time_seconds(&self) -> Real {
        if self.time_seconds.is_finite() {
            self.time_seconds.max(0.0)
        } else {
            0.0
        }
    }

    pub fn sanitized_playback_speed(&self) -> Real {
        if self.playback_speed.is_finite() {
            self.playback_speed.max(0.0)
        } else {
            0.0
        }
    }

    pub fn normalized_weight(&self) -> Real {
        if self.weight.is_finite() {
            self.weight.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn with_source(mut self, source: AssetReference) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }

    pub fn sanitized_snapshot(&self) -> Self {
        let mut snapshot = self.clone();
        snapshot.time_seconds = self.sanitized_time_seconds();
        snapshot.playback_speed = self.sanitized_playback_speed();
        snapshot.weight = self.normalized_weight();
        snapshot
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationRigRuntimeStatus {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub skeleton: Option<AssetReference>,
    pub bone_count: u32,
    pub posed_bone_count: u32,
    pub avatar_mask: Option<AnimationAvatarMask>,
    pub gpu_skinning: AnimationGpuSkinningReadiness,
    pub missing_targets: Vec<String>,
    pub diagnostics: Vec<String>,
}

impl AnimationRigRuntimeStatus {
    pub fn new(world: WorldHandle, entity: EntityId) -> Self {
        Self {
            world,
            entity,
            skeleton: None,
            bone_count: 0,
            posed_bone_count: 0,
            avatar_mask: None,
            gpu_skinning: AnimationGpuSkinningReadiness::default(),
            missing_targets: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn ready_for_pose(&self) -> bool {
        self.skeleton.is_some() && self.bone_count > 0 && self.missing_targets.is_empty()
    }

    pub fn pose_coverage(&self) -> Real {
        if self.bone_count == 0 {
            return 0.0;
        }
        (self.posed_bone_count as Real / self.bone_count as Real).clamp(0.0, 1.0)
    }

    pub fn with_skeleton(mut self, skeleton: AssetReference, bone_count: u32) -> Self {
        self.skeleton = Some(skeleton);
        self.bone_count = bone_count;
        self
    }

    pub fn with_missing_target(mut self, target: impl Into<String>) -> Self {
        self.missing_targets.push(target.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationRuntimeStatus {
    pub world: WorldHandle,
    pub players: Vec<AnimationPlayerRuntimeStatus>,
    pub rigs: Vec<AnimationRigRuntimeStatus>,
    pub last_tick: AnimationTickReport,
    pub diagnostics: Vec<String>,
}

impl AnimationRuntimeStatus {
    pub fn new(world: WorldHandle) -> Self {
        Self {
            world,
            players: Vec::new(),
            rigs: Vec::new(),
            last_tick: AnimationTickReport::new(world),
            diagnostics: Vec::new(),
        }
    }

    pub fn active_player_count(&self) -> usize {
        self.players
            .iter()
            .filter(|player| player.is_active())
            .count()
    }

    pub fn posed_rig_count(&self) -> usize {
        self.rigs
            .iter()
            .filter(|rig| rig.posed_bone_count > 0)
            .count()
    }

    pub fn gpu_ready_rig_count(&self) -> usize {
        self.rigs
            .iter()
            .filter(|rig| rig.gpu_skinning.ready_for_gpu_skinning())
            .count()
    }

    pub fn has_runtime_work(&self) -> bool {
        self.last_tick.has_runtime_work()
            || self.active_player_count() > 0
            || self.posed_rig_count() > 0
    }

    pub fn with_player(mut self, player: AnimationPlayerRuntimeStatus) -> Self {
        self.players.push(player);
        self
    }

    pub fn with_rig(mut self, rig: AnimationRigRuntimeStatus) -> Self {
        self.rigs.push(rig);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }

    pub fn sanitized_snapshot(&self) -> Self {
        let mut snapshot = self.clone();
        snapshot.players = self
            .players
            .iter()
            .map(AnimationPlayerRuntimeStatus::sanitized_snapshot)
            .collect();
        snapshot
    }
}

fn serialize_sanitized_non_negative_real<S>(value: &Real, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f32(sanitize_non_negative_real(*value))
}

fn deserialize_sanitized_non_negative_real<'de, D>(deserializer: D) -> Result<Real, D::Error>
where
    D: Deserializer<'de>,
{
    Real::deserialize(deserializer).map(sanitize_non_negative_real)
}

fn serialize_normalized_real<S>(value: &Real, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f32(normalize_real(*value))
}

fn deserialize_normalized_real<'de, D>(deserializer: D) -> Result<Real, D::Error>
where
    D: Deserializer<'de>,
{
    Real::deserialize(deserializer).map(normalize_real)
}

fn sanitize_non_negative_real(value: Real) -> Real {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn normalize_real(value: Real) -> Real {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
