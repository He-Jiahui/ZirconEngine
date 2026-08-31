//! Fsync'd immutable intent plus append-only transitions.
//!
//! The root owns the crate-private journal surface. Each child owns one part of
//! the write-ahead-log protocol so framing, durable publication, and restart
//! recovery do not accumulate in a single source file.

mod append;
mod frame_codec;
mod intent;
mod recovery;
#[cfg(test)]
mod tests;

pub(super) use append::{
    CommitPointRecord, record_commit_point, record_phase, record_prepared, record_state,
};
pub(super) use frame_codec::MAX_JOURNAL_BYTES;
#[cfg(test)]
pub(super) use frame_codec::encode_frame;
#[cfg(test)]
pub(super) use intent::create_intent;
pub(super) use intent::{persist_intent, plan_intent};
#[cfg(any(test, feature = "test-support"))]
pub(super) use recovery::decode_journal;
pub(super) use recovery::{decode_journal_with_valid_len, truncate_torn_tail};
