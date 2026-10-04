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
