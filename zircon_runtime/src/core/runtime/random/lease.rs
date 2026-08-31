use std::sync::Arc;

use zr_contracts::random::{RandomSequenceId, RandomState, RandomStreamKey};

use super::authority::RandomAuthority;
use super::{RandomStream, RandomStreamError};

/// Exclusive mutable ownership of one registered deterministic stream.
#[derive(Debug)]
pub struct RandomStreamLease {
    key: RandomStreamKey,
    stream: Option<RandomStream>,
    authority: Arc<RandomAuthority>,
}

impl RandomStreamLease {
    pub(crate) fn new(
        key: RandomStreamKey,
        stream: RandomStream,
        authority: Arc<RandomAuthority>,
    ) -> Self {
        Self {
            key,
            stream: Some(stream),
            authority,
        }
    }

    pub const fn key(&self) -> RandomStreamKey {
        self.key
    }

    pub fn snapshot(&self) -> RandomState {
        self.stream_ref().snapshot()
    }

    pub fn draw_index(&self) -> u64 {
        self.stream_ref().draw_index()
    }

    pub fn sequence_id(&self) -> RandomSequenceId {
        self.stream_ref().sequence_id()
    }

    pub fn try_next_u32(&mut self) -> Result<u32, RandomStreamError> {
        self.stream_mut().try_next_u32()
    }

    pub fn try_next_bounded_u32(
        &mut self,
        upper_exclusive: u32,
    ) -> Result<Option<u32>, RandomStreamError> {
        self.stream_mut().try_next_bounded_u32(upper_exclusive)
    }

    pub fn try_next_unit_f32(&mut self) -> Result<f32, RandomStreamError> {
        self.stream_mut().try_next_unit_f32()
    }

    /// Commits this lease immediately and returns the committed stream state.
    pub fn release(mut self) -> RandomState {
        let state = self.stream_ref().snapshot();
        self.commit();
        state
    }

    fn stream_ref(&self) -> &RandomStream {
        self.stream
            .as_ref()
            .expect("a live random stream lease always owns its stream")
    }

    fn stream_mut(&mut self) -> &mut RandomStream {
        self.stream
            .as_mut()
            .expect("a live random stream lease always owns its stream")
    }

    fn commit(&mut self) {
        if let Some(stream) = self.stream.take() {
            self.authority.registry().release(self.key, stream);
        }
    }
}

impl Drop for RandomStreamLease {
    fn drop(&mut self) {
        self.commit();
    }
}
