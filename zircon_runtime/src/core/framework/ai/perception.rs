use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

/// 行为树感知条件与编辑器覆盖层共用的感知通道标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiPerceptionSense {
    Sight,
    Hearing,
    Damage,
    Touch,
    Custom,
}

/// 听觉事件的来源标签；声音、动画和自定义生产者通过同一中立事件通道接入。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiHearingStimulusOrigin {
    SoundPlayback,
    AnimationEvent,
    Custom,
}

/// Neutral bus event that sound, animation, or gameplay plugins can emit without depending on a
/// concrete AI runtime implementation.
/// 事件先由 AI perception adapter 按接收者和预算转换成感知 stimulus；生产者不持有代理状态。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiHearingStimulusEvent {
    pub source: EntityId,
    pub position: Vec3,
    pub strength: Real,
    pub max_range: Option<Real>,
    pub origin: AiHearingStimulusOrigin,
    pub age_seconds: Real,
}

impl AiHearingStimulusEvent {
    pub fn sound_playback(source: EntityId, position: Vec3, strength: Real) -> Self {
        Self {
            source,
            position,
            strength,
            max_range: None,
            origin: AiHearingStimulusOrigin::SoundPlayback,
            age_seconds: 0.0,
        }
    }

    pub fn animation_event(source: EntityId, position: Vec3, strength: Real) -> Self {
        Self {
            source,
            position,
            strength,
            max_range: None,
            origin: AiHearingStimulusOrigin::AnimationEvent,
            age_seconds: 0.0,
        }
    }

    pub fn with_max_range(mut self, max_range: Real) -> Self {
        self.max_range = Some(max_range);
        self
    }

    pub fn with_age_seconds(mut self, age_seconds: Real) -> Self {
        self.age_seconds = age_seconds;
        self
    }
}

/// 一条已采样的感知结果，供行为树条件匹配并在编辑器中绘制来源、强度和年龄。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiPerceptionStimulus {
    pub source: EntityId,
    pub sense: AiPerceptionSense,
    pub position: Vec3,
    pub strength: Real,
    pub age_seconds: Real,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 某个代理在一次感知采样后的输入快照；传入管理器时 agent 必须与目标 entity 一致。
pub struct AiPerceptionSnapshot {
    pub agent: EntityId,
    pub stimuli: Vec<AiPerceptionStimulus>,
}
