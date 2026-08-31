use std::{
    cmp::Ordering,
    collections::{btree_map::Entry, BTreeMap},
};

use thiserror::Error;

use serde::Serialize;

use crate::{EntityRef, MovementFrame};

/// The target clears held movement only after a 750 ms silent stream. At 20 Hz,
/// equality remains live and input becomes stale after more than fifteen ticks.
pub const MOVEMENT_INPUT_STALE_AFTER_TICKS: u64 = 15;
pub const MAX_MOVEMENT_FRAMES_PER_TICK: usize = 65_536;
pub const MAX_MOVEMENT_FACING_MAGNITUDE: f64 = 1_000.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct MovementInputFlags {
    pub forward: bool,
    pub back: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
    pub jump: bool,
}

impl MovementInputFlags {
    pub const fn is_held(self) -> bool {
        self.forward
            || self.back
            || self.turn_left
            || self.turn_right
            || self.strafe_left
            || self.strafe_right
            || self.jump
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MovementFrameBatch {
    frames: Vec<MovementFrame>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetainedMovementInput {
    pub actor: EntityRef,
    pub acknowledgement: u32,
    pub flags: MovementInputFlags,
    pub facing: Option<f64>,
    pub accepted_tick: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementFrameDisposition {
    Applied {
        actor: EntityRef,
        acknowledgement: u32,
    },
    Duplicate {
        actor: EntityRef,
        acknowledgement: u32,
    },
    Stale {
        actor: EntityRef,
        acknowledgement: u32,
    },
}

#[derive(Debug, Error, PartialEq)]
pub enum MovementInputError {
    #[error("movement actor id {id} must be nonzero")]
    InvalidActor { id: u64 },
    #[error("movement sequence must be positive")]
    InvalidSequence { actor: EntityRef },
    #[error("movement facing {value} must be finite and within +/-{maximum}")]
    InvalidFacing { value: f64, maximum: f64 },
    #[error("movement batch contains duplicate actor {actor_id}:{generation}")]
    DuplicateActor { actor_id: u64, generation: u32 },
    #[error(
        "movement batch actor order regressed from {previous_actor_id}:{previous_generation} to {actual_actor_id}:{actual_generation}"
    )]
    NonCanonicalActorOrder {
        previous_actor_id: u64,
        previous_generation: u32,
        actual_actor_id: u64,
        actual_generation: u32,
    },
    #[error("movement batch has {actual} frames, maximum is {maximum}")]
    TooManyFrames { actual: usize, maximum: usize },
    #[error("movement tick regressed from {previous} to {actual}")]
    TickRegression { previous: u64, actual: u64 },
}

impl MovementFrame {
    pub fn validate(self) -> Result<Self, MovementInputError> {
        if self.actor.id == 0 {
            return Err(MovementInputError::InvalidActor { id: self.actor.id });
        }
        if self.sequence == 0 {
            return Err(MovementInputError::InvalidSequence { actor: self.actor });
        }
        if let Some(facing) = self.facing {
            if !facing.is_finite() || facing.abs() > MAX_MOVEMENT_FACING_MAGNITUDE {
                return Err(MovementInputError::InvalidFacing {
                    value: facing,
                    maximum: MAX_MOVEMENT_FACING_MAGNITUDE,
                });
            }
        }
        Ok(self)
    }
}

impl MovementFrameBatch {
    pub fn new(mut frames: Vec<MovementFrame>) -> Result<Self, MovementInputError> {
        validate_frames(&frames)?;
        frames.sort_by_key(|frame| actor_key(frame.actor));
        validate_canonical_order(&frames)?;
        Ok(Self { frames })
    }

    pub fn from_canonical(frames: Vec<MovementFrame>) -> Result<Self, MovementInputError> {
        validate_frames(&frames)?;
        validate_canonical_order(&frames)?;
        Ok(Self { frames })
    }

    pub fn frames(&self) -> &[MovementFrame] {
        &self.frames
    }
}

/// Network transport state for movement frames. It owns sequencing and stale
/// input clearing, never player position, collision, movement speed, or combat.
#[derive(Default)]
pub struct MovementInputRelay {
    inputs: BTreeMap<(u64, u32), RetainedMovementInput>,
    last_observed_tick: Option<u64>,
}

impl MovementInputRelay {
    pub fn apply_batch(
        &mut self,
        tick: u64,
        batch: &MovementFrameBatch,
    ) -> Result<Vec<MovementFrameDisposition>, MovementInputError> {
        self.observe_tick(tick)?;
        let mut dispositions = Vec::with_capacity(batch.frames.len());
        for frame in batch.frames() {
            let key = actor_key(frame.actor);
            let disposition = match self.inputs.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(RetainedMovementInput {
                        actor: frame.actor,
                        acknowledgement: frame.sequence,
                        flags: frame.flags,
                        facing: frame.facing,
                        accepted_tick: tick,
                    });
                    MovementFrameDisposition::Applied {
                        actor: frame.actor,
                        acknowledgement: frame.sequence,
                    }
                }
                Entry::Occupied(mut occupied) => {
                    let input = occupied.get_mut();
                    match frame.sequence.cmp(&input.acknowledgement) {
                        Ordering::Greater => {
                            input.acknowledgement = frame.sequence;
                            input.flags = frame.flags;
                            if let Some(facing) = frame.facing {
                                input.facing = Some(facing);
                            }
                            input.accepted_tick = tick;
                            MovementFrameDisposition::Applied {
                                actor: frame.actor,
                                acknowledgement: frame.sequence,
                            }
                        }
                        Ordering::Equal => MovementFrameDisposition::Duplicate {
                            actor: frame.actor,
                            acknowledgement: input.acknowledgement,
                        },
                        Ordering::Less => MovementFrameDisposition::Stale {
                            actor: frame.actor,
                            acknowledgement: input.acknowledgement,
                        },
                    }
                }
            };
            dispositions.push(disposition);
        }
        Ok(dispositions)
    }

    /// Clears held directional state after the target's stale-stream window.
    /// The last valid facing is intentionally retained, matching server behavior.
    pub fn clear_stale(&mut self, tick: u64) -> Result<Vec<EntityRef>, MovementInputError> {
        self.observe_tick(tick)?;
        let mut cleared = Vec::new();
        for input in self.inputs.values_mut() {
            if tick.saturating_sub(input.accepted_tick) > MOVEMENT_INPUT_STALE_AFTER_TICKS
                && input.flags.is_held()
            {
                input.flags = MovementInputFlags::default();
                cleared.push(input.actor);
            }
        }
        Ok(cleared)
    }

    pub fn acknowledgement(&self, actor: EntityRef) -> Option<u32> {
        self.inputs
            .get(&actor_key(actor))
            .map(|input| input.acknowledgement)
    }

    pub fn input(&self, actor: EntityRef) -> Option<&RetainedMovementInput> {
        self.inputs.get(&actor_key(actor))
    }

    fn observe_tick(&mut self, tick: u64) -> Result<(), MovementInputError> {
        if let Some(previous) = self.last_observed_tick {
            if tick < previous {
                return Err(MovementInputError::TickRegression {
                    previous,
                    actual: tick,
                });
            }
        }
        self.last_observed_tick = Some(tick);
        Ok(())
    }
}

fn actor_key(actor: EntityRef) -> (u64, u32) {
    (actor.id, actor.generation)
}

fn validate_frames(frames: &[MovementFrame]) -> Result<(), MovementInputError> {
    if frames.len() > MAX_MOVEMENT_FRAMES_PER_TICK {
        return Err(MovementInputError::TooManyFrames {
            actual: frames.len(),
            maximum: MAX_MOVEMENT_FRAMES_PER_TICK,
        });
    }
    for frame in frames {
        frame.validate()?;
    }
    Ok(())
}

fn validate_canonical_order(frames: &[MovementFrame]) -> Result<(), MovementInputError> {
    for pair in frames.windows(2) {
        let previous = actor_key(pair[0].actor);
        let actual = actor_key(pair[1].actor);
        if previous == actual {
            return Err(MovementInputError::DuplicateActor {
                actor_id: pair[0].actor.id,
                generation: pair[0].actor.generation,
            });
        }
        if previous > actual {
            return Err(MovementInputError::NonCanonicalActorOrder {
                previous_actor_id: pair[0].actor.id,
                previous_generation: pair[0].actor.generation,
                actual_actor_id: pair[1].actor.id,
                actual_generation: pair[1].actor.generation,
            });
        }
    }
    Ok(())
}
