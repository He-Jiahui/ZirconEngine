//! Workspace-only constructors for already-validated random invariants.

use super::{RandomAlgorithmId, RandomSequenceId, RandomState};

/// Projects a uniform derivation word into PCG's 63-bit stream identity space.
///
/// Runtime execution owns the derivation algorithm; this function only keeps
/// the contract type's representation private across the crate boundary.
pub const fn sequence_id_from_uniform_u64(value: u64) -> RandomSequenceId {
    RandomSequenceId::from_uniform_u64(value)
}

/// Assembles an initial state from a validated PCG stream identity.
pub const fn random_state_from_sequence(
    algorithm: RandomAlgorithmId,
    state: u64,
    sequence: RandomSequenceId,
    draw_index: u64,
) -> RandomState {
    let increment = (sequence.value() << 1) | 1;
    RandomState::from_valid_parts(algorithm, state, increment, draw_index)
}

/// Advances execution progress while preserving a validated stream identity.
pub const fn random_state_with_progress(
    current: RandomState,
    generator_state: u64,
    draw_index: u64,
) -> RandomState {
    RandomState::from_valid_parts(
        current.algorithm(),
        generator_state,
        current.increment(),
        draw_index,
    )
}
