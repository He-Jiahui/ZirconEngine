use std::sync::Arc;

use zr_contracts::random::{RandomSequenceId, RandomState, RandomStreamKey};

use super::authority::RandomAuthority;
use super::{RandomStream, RandomStreamError};

/// Exclusive mutable ownership of one registered deterministic stream.
/// 同一稳定键只能有一个租约；抽样在本地推进，归还时才把进度写回注册表。
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
    /// 需要在 checkpoint 或 reseed 前解除活跃租约时，应显式归还并处理所得状态。
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

// TODO: [CR-RUNTIME-RANDOM-0001] 核查未来固定步调用链的 commit/abort：若失败步持有租约，析构仍会提交抽样进度。
// 证据：release 和 Drop 均调用 commit；当前生产调用链尚未接入固定步抽样。
impl Drop for RandomStreamLease {
    fn drop(&mut self) {
        self.commit();
    }
}
