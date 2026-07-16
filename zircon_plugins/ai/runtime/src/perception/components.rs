use std::ops::{BitOr, BitOrAssign};

use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::ecs::{Component, StorageType};
use zircon_runtime::scene::World;

pub const AI_PERCEPTION_SOURCE_COMPONENT_TYPE: &str = "ai.perception_source";
pub const AI_PERCEPTION_RECEIVER_COMPONENT_TYPE: &str = "ai.perception_receiver";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AiPerceptionChannels(u8);

impl AiPerceptionChannels {
    pub const NONE: Self = Self(0);
    pub const SIGHT: Self = Self(1 << 0);
    pub const HEARING: Self = Self(1 << 1);
    pub const ALL: Self = Self(Self::SIGHT.0 | Self::HEARING.0);

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & Self::ALL.0)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn contains(self, channel: Self) -> bool {
        self.0 & channel.0 == channel.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for AiPerceptionChannels {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for AiPerceptionChannels {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiPerceptionSource {
    pub channels: AiPerceptionChannels,
    pub strength: Real,
}

impl Default for AiPerceptionSource {
    fn default() -> Self {
        Self {
            channels: AiPerceptionChannels::SIGHT,
            strength: 1.0,
        }
    }
}

impl Component for AiPerceptionSource {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiPerceptionReceiver {
    pub sight_fov_degrees: Real,
    pub sight_range: Real,
    pub hearing_radius: Real,
    pub forget_seconds: Real,
}

impl Default for AiPerceptionReceiver {
    fn default() -> Self {
        Self {
            sight_fov_degrees: 90.0,
            sight_range: 30.0,
            hearing_radius: 20.0,
            forget_seconds: 5.0,
        }
    }
}

impl Component for AiPerceptionReceiver {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

pub fn ai_perception_component_descriptors() -> Vec<ComponentTypeDescriptor> {
    vec![
        ComponentTypeDescriptor::new(
            AI_PERCEPTION_SOURCE_COMPONENT_TYPE,
            crate::PLUGIN_ID,
            "AI Perception Source",
        )
        .with_property("channels", "integer", true)
        .with_property("strength", "scalar", true),
        ComponentTypeDescriptor::new(
            AI_PERCEPTION_RECEIVER_COMPONENT_TYPE,
            crate::PLUGIN_ID,
            "AI Perception Receiver",
        )
        .with_property("sight_fov_degrees", "scalar", true)
        .with_property("sight_range", "scalar", true)
        .with_property("hearing_radius", "scalar", true)
        .with_property("forget_seconds", "scalar", true),
    ]
}

pub(crate) fn perception_source(world: &World, entity: EntityId) -> Option<AiPerceptionSource> {
    if let Some(object) = world
        .dynamic_component(entity, AI_PERCEPTION_SOURCE_COMPONENT_TYPE)
        .and_then(|value| value.as_object())
    {
        let mut source = AiPerceptionSource::default();
        if let Some(value) = object.get("channels") {
            source.channels = AiPerceptionChannels::from_bits(u8::try_from(value.as_u64()?).ok()?);
        }
        if let Some(value) = object.get("strength") {
            source.strength = finite_real(value.as_f64())?;
        }
        return Some(source);
    }
    world.get::<AiPerceptionSource>(entity).copied()
}

pub(crate) fn perception_receiver(world: &World, entity: EntityId) -> Option<AiPerceptionReceiver> {
    if let Some(object) = world
        .dynamic_component(entity, AI_PERCEPTION_RECEIVER_COMPONENT_TYPE)
        .and_then(|value| value.as_object())
    {
        let mut receiver = AiPerceptionReceiver::default();
        if let Some(value) = object.get("sight_fov_degrees") {
            receiver.sight_fov_degrees = finite_real(value.as_f64())?;
        }
        if let Some(value) = object.get("sight_range") {
            receiver.sight_range = finite_real(value.as_f64())?;
        }
        if let Some(value) = object.get("hearing_radius") {
            receiver.hearing_radius = finite_real(value.as_f64())?;
        }
        if let Some(value) = object.get("forget_seconds") {
            receiver.forget_seconds = finite_real(value.as_f64())?;
        }
        return Some(receiver);
    }
    world.get::<AiPerceptionReceiver>(entity).copied()
}

fn finite_real(value: Option<f64>) -> Option<Real> {
    let value = value? as Real;
    value.is_finite().then_some(value)
}
