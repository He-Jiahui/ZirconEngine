use woc_protocol::{EntityRef, MovementFrame, MovementInputFlags};

use super::ClientMovementInputError;

/// Owns the one locally controlled actor's held input and its independent
/// positive movement sequence. Sampling is side-effect-free so a failed VM
/// transaction can retry exactly the same authoritative frame.
pub struct ClientMovementStream {
    actor: EntityRef,
    next_sequence: Option<u32>,
    flags: MovementInputFlags,
    facing: Option<f64>,
}

impl ClientMovementStream {
    pub fn new(actor: EntityRef, next_sequence: u32) -> Result<Self, ClientMovementInputError> {
        MovementFrame {
            actor,
            sequence: next_sequence,
            flags: MovementInputFlags::default(),
            facing: None,
        }
        .validate()?;
        Ok(Self {
            actor,
            next_sequence: Some(next_sequence),
            flags: MovementInputFlags::default(),
            facing: None,
        })
    }

    /// Validates before replacing the held state so malformed host input cannot
    /// erase the last valid direction or facing value.
    pub fn set_input(
        &mut self,
        flags: MovementInputFlags,
        facing: Option<f64>,
    ) -> Result<(), ClientMovementInputError> {
        self.frame_with(flags, facing)?.validate()?;
        self.flags = flags;
        self.facing = facing;
        Ok(())
    }

    pub fn frame(&self) -> Result<MovementFrame, ClientMovementInputError> {
        self.frame_with(self.flags, self.facing)
    }

    /// Advances only after the surrounding authority and presentation
    /// transaction commits, preserving retry identity on every failure path.
    pub fn commit(&mut self) {
        let sequence = self
            .next_sequence
            .expect("movement sequences commit only after a sampled frame");
        self.next_sequence = sequence.checked_add(1);
    }

    fn frame_with(
        &self,
        flags: MovementInputFlags,
        facing: Option<f64>,
    ) -> Result<MovementFrame, ClientMovementInputError> {
        let sequence = self
            .next_sequence
            .ok_or(ClientMovementInputError::SequenceExhausted)?;
        Ok(MovementFrame {
            actor: self.actor,
            sequence,
            flags,
            facing,
        })
    }
}
