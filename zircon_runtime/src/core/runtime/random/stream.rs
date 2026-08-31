use zr_contracts::random::{
    assembly::{random_state_from_sequence, random_state_with_progress},
    RandomAlgorithmId, RandomSequenceId, RandomState, RandomStateError,
};

use super::RandomStreamError;

const PCG32_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const UNIT_F32_DENOMINATOR: f32 = 16_777_216.0;

/// Stateful deterministic stream derived by `RandomService` or restored from a snapshot.
#[derive(Debug, PartialEq, Eq)]
pub struct RandomStream {
    state: RandomState,
}

impl RandomStream {
    pub fn from_state(state: RandomState) -> Result<Self, RandomStateError> {
        let state = RandomState::new(
            state.algorithm(),
            state.generator_state(),
            state.increment(),
            state.draw_index(),
        )?;
        Ok(Self { state })
    }

    pub(crate) const fn from_valid_state(state: RandomState) -> Self {
        Self { state }
    }

    pub const fn snapshot(&self) -> RandomState {
        self.state
    }

    pub const fn draw_index(&self) -> u64 {
        self.state.draw_index()
    }

    pub const fn sequence_id(&self) -> RandomSequenceId {
        self.state.sequence_id()
    }

    pub fn try_next_u32(&mut self) -> Result<u32, RandomStreamError> {
        let next_draw_index = self
            .state
            .draw_index()
            .checked_add(1)
            .ok_or(RandomStreamError::DrawIndexExhausted)?;
        let previous_state = self.state.generator_state();
        let next_state = advance_pcg_state(previous_state, self.state.increment());
        let output = match self.state.algorithm() {
            RandomAlgorithmId::Pcg32XshRrV1 => pcg32_xsh_rr(previous_state),
        };
        self.state = random_state_with_progress(self.state, next_state, next_draw_index);
        Ok(output)
    }

    /// Draws a uniformly distributed value in `[0, upper_exclusive)`.
    pub fn try_next_bounded_u32(
        &mut self,
        upper_exclusive: u32,
    ) -> Result<Option<u32>, RandomStreamError> {
        if upper_exclusive == 0 {
            return Ok(None);
        }
        let rejection_threshold = upper_exclusive.wrapping_neg() % upper_exclusive;
        loop {
            let draw = self.try_next_u32()?;
            if draw >= rejection_threshold {
                return Ok(Some(draw % upper_exclusive));
            }
        }
    }

    /// Draws a reproducible 24-bit fraction in `[0, 1)` without platform math state.
    pub fn try_next_unit_f32(&mut self) -> Result<f32, RandomStreamError> {
        let draw = self.try_next_u32()? >> 8;
        Ok(draw as f32 / UNIT_F32_DENOMINATOR)
    }

    pub(crate) fn from_seed(
        algorithm: RandomAlgorithmId,
        seed: u64,
        sequence: RandomSequenceId,
    ) -> Self {
        let increment = (sequence.value() << 1) | 1;
        let first_advance = advance_pcg_state(0, increment);
        let seeded_state = first_advance.wrapping_add(seed);
        let initialized_state = advance_pcg_state(seeded_state, increment);
        Self {
            state: random_state_from_sequence(algorithm, initialized_state, sequence, 0),
        }
    }
}

fn advance_pcg_state(state: u64, increment: u64) -> u64 {
    state.wrapping_mul(PCG32_MULTIPLIER).wrapping_add(increment)
}

fn pcg32_xsh_rr(previous_state: u64) -> u32 {
    let xorshifted = (((previous_state >> 18) ^ previous_state) >> 27) as u32;
    xorshifted.rotate_right((previous_state >> 59) as u32)
}
