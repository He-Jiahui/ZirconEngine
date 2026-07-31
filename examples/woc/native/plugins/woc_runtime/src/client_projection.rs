use std::cmp::Ordering;
use std::f32::consts::{PI, TAU};

use serde::{Deserialize, Serialize};
use woc_protocol::EntityRef;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PresentationVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ActorTransform {
    pub translation: PresentationVec3,
    pub facing_radians: f32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ActorAnimationInput {
    pub horizontal_speed: f32,
    pub moving: bool,
    pub running: bool,
    pub airborne: bool,
    pub backwards: bool,
    pub reverse_backpedal: bool,
    pub dead: bool,
    pub casting: bool,
    pub spinning: bool,
    pub swimming: bool,
    pub sitting: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct ActorAppearance {
    pub skin_variant: u16,
    pub mainhand_item_id: Option<String>,
    pub offhand_item_id: Option<String>,
    pub weapon_skin_id: Option<String>,
    pub weapon_stowed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ActorPresentation {
    pub entity: EntityRef,
    pub template_id: String,
    pub transform: ActorTransform,
    pub animation: ActorAnimationInput,
    pub appearance: ActorAppearance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BulkPresentationProjection {
    pub viewer: EntityRef,
    pub actors: Vec<ActorPresentation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresentationProjectionError {
    ActorsNotStrictlySorted {
        index: usize,
    },
    ViewerMissing {
        viewer: EntityRef,
    },
    NonFiniteActorField {
        entity: EntityRef,
        field: &'static str,
    },
    NegativeHorizontalSpeed {
        entity: EntityRef,
    },
    NonFiniteAlpha,
}

impl BulkPresentationProjection {
    pub fn validate(&self) -> Result<(), PresentationProjectionError> {
        for (index, pair) in self.actors.windows(2).enumerate() {
            if compare_entity_ref(pair[0].entity, pair[1].entity) != Ordering::Less {
                return Err(PresentationProjectionError::ActorsNotStrictlySorted {
                    index: index + 1,
                });
            }
        }

        for actor in &self.actors {
            validate_actor(actor)?;
        }

        if self.actor(self.viewer).is_none() {
            return Err(PresentationProjectionError::ViewerMissing {
                viewer: self.viewer,
            });
        }
        Ok(())
    }

    pub fn actor(&self, entity: EntityRef) -> Option<&ActorPresentation> {
        self.actors
            .binary_search_by(|actor| compare_entity_ref(actor.entity, entity))
            .ok()
            .map(|index| &self.actors[index])
    }

    pub fn visit_interpolated_from(
        &self,
        previous: &Self,
        alpha: f32,
        mut visitor: impl FnMut(&ActorPresentation, ActorTransform),
    ) -> Result<(), PresentationProjectionError> {
        if !alpha.is_finite() {
            return Err(PresentationProjectionError::NonFiniteAlpha);
        }
        let alpha = alpha.clamp(0.0, 1.0);
        let mut previous_index = 0;

        for actor in &self.actors {
            while previous_index < previous.actors.len()
                && compare_entity_ref(previous.actors[previous_index].entity, actor.entity)
                    == Ordering::Less
            {
                previous_index += 1;
            }
            let transform = if previous_index < previous.actors.len()
                && previous.actors[previous_index].entity == actor.entity
            {
                interpolate_transform(
                    previous.actors[previous_index].transform,
                    actor.transform,
                    alpha,
                )
            } else {
                actor.transform
            };
            visitor(actor, transform);
        }
        Ok(())
    }
}

fn validate_actor(actor: &ActorPresentation) -> Result<(), PresentationProjectionError> {
    for (field, value) in [
        ("transform.translation.x", actor.transform.translation.x),
        ("transform.translation.y", actor.transform.translation.y),
        ("transform.translation.z", actor.transform.translation.z),
        ("transform.facing_radians", actor.transform.facing_radians),
        (
            "animation.horizontal_speed",
            actor.animation.horizontal_speed,
        ),
    ] {
        if !value.is_finite() {
            return Err(PresentationProjectionError::NonFiniteActorField {
                entity: actor.entity,
                field,
            });
        }
    }
    if actor.animation.horizontal_speed < 0.0 {
        return Err(PresentationProjectionError::NegativeHorizontalSpeed {
            entity: actor.entity,
        });
    }
    Ok(())
}

fn compare_entity_ref(left: EntityRef, right: EntityRef) -> Ordering {
    left.id
        .cmp(&right.id)
        .then_with(|| left.generation.cmp(&right.generation))
}

fn interpolate_transform(from: ActorTransform, to: ActorTransform, alpha: f32) -> ActorTransform {
    if alpha <= 0.0 {
        return from;
    }
    if alpha >= 1.0 {
        return to;
    }

    ActorTransform {
        translation: PresentationVec3 {
            x: lerp(from.translation.x, to.translation.x, alpha),
            y: lerp(from.translation.y, to.translation.y, alpha),
            z: lerp(from.translation.z, to.translation.z, alpha),
        },
        facing_radians: from.facing_radians + shortest_angle_delta(from, to) * alpha,
    }
}

fn shortest_angle_delta(from: ActorTransform, to: ActorTransform) -> f32 {
    let mut delta = (to.facing_radians - from.facing_radians).rem_euclid(TAU);
    if delta > PI {
        delta -= TAU;
    }
    delta
}

fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}
