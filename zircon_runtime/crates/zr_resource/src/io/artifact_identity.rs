//! 为临时产物提供进程内并发唯一的非零序号；路径创建仍必须使用排他创建保证跨进程安全。
//! 最大值只发放一次，耗尽后保持终止状态，避免回绕复用仍存活的产物名。

use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("durable I/O artifact identity space is exhausted")]
pub struct ArtifactIdentityExhausted;

#[derive(Debug)]
pub(crate) struct ArtifactSequence {
    next: AtomicU64,
}

impl ArtifactSequence {
    pub(crate) const fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    #[cfg(test)]
    pub(crate) const fn starting_at(next: u64) -> Self {
        Self {
            next: AtomicU64::new(next),
        }
    }

    pub(crate) fn next(&self) -> Result<NonZeroU64, ArtifactIdentityExhausted> {
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            let identity = NonZeroU64::new(current).ok_or(ArtifactIdentityExhausted)?;
            let successor = current.checked_add(1).unwrap_or(0);
            match self.next.compare_exchange_weak(
                current,
                successor,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(identity),
                Err(observed) => current = observed,
            }
        }
    }
}

#[cfg(test)]
static TEST_OUTPUT_SEQUENCE: ArtifactSequence = ArtifactSequence::new();

#[cfg(test)]
pub(crate) fn next_test_output_id() -> u64 {
    TEST_OUTPUT_SEQUENCE
        .next()
        .expect("test output identity space has capacity")
        .get()
}

#[cfg(test)]
#[path = "tests/artifact_identity.rs"]
mod tests;
