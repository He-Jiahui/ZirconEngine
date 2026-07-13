use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::{Real, Vec3};

use super::{AnimationIkCommandError, AnimationTargetId};

/// Stable, skeleton-independent IK work submitted by components or scripts.
///
/// The animation runtime resolves target IDs to skeleton-scoped dense slots
/// immediately before the post-process pass. Targets and poles are expressed
/// in skeleton model space.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnimationIkCommand {
    TwoBone(AnimationTwoBoneIkCommand),
    LookAt(AnimationLookAtCommand),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTwoBoneIkCommand {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub root: AnimationTargetId,
    pub mid: AnimationTargetId,
    pub tip: AnimationTargetId,
    pub target: Vec3,
    pub pole: Option<Vec3>,
    pub weight: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationLookAtCommand {
    pub world: WorldHandle,
    pub entity: EntityId,
    pub bone: AnimationTargetId,
    pub target: Vec3,
    pub axis: Vec3,
    pub clamp_degrees: Real,
    pub weight: Real,
}

impl AnimationIkCommand {
    pub const fn world(&self) -> WorldHandle {
        match self {
            Self::TwoBone(command) => command.world,
            Self::LookAt(command) => command.world,
        }
    }

    pub const fn entity(&self) -> EntityId {
        match self {
            Self::TwoBone(command) => command.entity,
            Self::LookAt(command) => command.entity,
        }
    }

    pub fn validate(&self) -> Result<(), AnimationIkCommandError> {
        let world = self.world();
        let entity = self.entity();
        let (finite, weight) = match self {
            Self::TwoBone(command) => (
                command.target.is_finite() && command.pole.is_none_or(|pole| pole.is_finite()),
                command.weight,
            ),
            Self::LookAt(command) => {
                if command.axis.length_squared() <= Real::EPSILON {
                    return Err(AnimationIkCommandError::DegenerateAxis { world, entity });
                }
                (
                    command.target.is_finite()
                        && command.axis.is_finite()
                        && command.clamp_degrees.is_finite(),
                    command.weight,
                )
            }
        };
        if !finite {
            return Err(AnimationIkCommandError::NonFiniteInput { world, entity });
        }
        if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
            return Err(AnimationIkCommandError::InvalidWeight { world, entity });
        }
        Ok(())
    }
}
