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
mod tests {
    use std::sync::Arc;
    use std::thread;

    use super::*;

    #[test]
    fn maximum_identity_is_issued_once_before_terminal_exhaustion() {
        let sequence = ArtifactSequence::starting_at(u64::MAX - 1);

        assert_eq!(sequence.next().unwrap().get(), u64::MAX - 1);
        assert_eq!(sequence.next().unwrap().get(), u64::MAX);
        assert_eq!(sequence.next(), Err(ArtifactIdentityExhausted));
        assert_eq!(sequence.next(), Err(ArtifactIdentityExhausted));
    }

    #[test]
    fn concurrent_boundary_allocation_never_duplicates_or_wraps() {
        const AVAILABLE: u64 = 16;
        const CALLERS: usize = 32;
        let sequence = Arc::new(ArtifactSequence::starting_at(u64::MAX - AVAILABLE + 1));
        let mut callers = Vec::with_capacity(CALLERS);
        for _ in 0..CALLERS {
            let sequence = Arc::clone(&sequence);
            callers.push(thread::spawn(move || sequence.next().map(NonZeroU64::get)));
        }

        let mut issued = Vec::with_capacity(AVAILABLE as usize);
        let mut exhausted = 0;
        for caller in callers {
            match caller.join().unwrap() {
                Ok(identity) => issued.push(identity),
                Err(ArtifactIdentityExhausted) => exhausted += 1,
            }
        }
        issued.sort_unstable();

        assert_eq!(
            issued,
            ((u64::MAX - AVAILABLE + 1)..=u64::MAX).collect::<Vec<_>>()
        );
        assert_eq!(exhausted, CALLERS - AVAILABLE as usize);
    }
}
