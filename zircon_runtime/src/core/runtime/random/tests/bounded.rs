use zr_contracts::random::{RandomAlgorithmId, RandomState};

use super::super::{RandomService, RandomStream};
use super::key;

#[test]
fn bounded_draws_cover_range_zero_bound_and_rejection_accounting() {
    let service = RandomService::new(23);
    let mut stream = service.acquire_stream(key()).expect("stream admission");
    let before = stream.snapshot();
    assert_eq!(stream.try_next_bounded_u32(0), Ok(None));
    assert_eq!(stream.snapshot(), before);
    for _ in 0..128 {
        assert!(
            stream
                .try_next_bounded_u32(17)
                .expect("bounded draw")
                .expect("non-zero bound must produce a value")
                < 17
        );
    }

    let rejection_state =
        RandomState::new(RandomAlgorithmId::Pcg32XshRrV1, 0, 1, 0).expect("odd increment is valid");
    let mut rejection_stream =
        RandomStream::from_state(rejection_state).expect("valid rejection-path state");
    assert_eq!(rejection_stream.try_next_bounded_u32(10), Ok(Some(8)));
    assert_eq!(rejection_stream.draw_index(), 3);
}

#[test]
fn draw_execution_source_has_no_registry_synchronization_or_key_hashing() {
    let stream_source = include_str!("../stream.rs");

    assert!(!stream_source.contains("Mutex"));
    assert!(!stream_source.contains("RandomStreamKey"));
    assert!(!stream_source.contains("blake3"));
}
