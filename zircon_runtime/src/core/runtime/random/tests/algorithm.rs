use zr_contracts::random::{RandomAlgorithmId, RandomSequenceId, RandomState, RandomStateError};

use super::super::{RandomStream, RandomStreamError};

#[test]
fn pcg32_xsh_rr_matches_published_seed_and_stream_vectors() {
    let sequence = RandomSequenceId::new(54).expect("known vector sequence should be valid");
    let mut stream = RandomStream::from_seed(RandomAlgorithmId::Pcg32XshRrV1, 42, sequence);

    let draws = (0..4)
        .map(|_| stream.try_next_u32().expect("known stream draw"))
        .collect::<Vec<_>>();

    assert_eq!(draws, [0xa15c_02b7, 0x7b47_f409, 0xba1d_3330, 0x83d2_f293]);
    assert_eq!(stream.draw_index(), 4);
}

#[test]
fn random_stream_rejects_even_increment_and_draw_index_exhaustion() {
    assert_eq!(
        RandomState::new(RandomAlgorithmId::Pcg32XshRrV1, 0, 2, 0),
        Err(RandomStateError::EvenIncrement)
    );
    let exhausted = RandomState::new(RandomAlgorithmId::Pcg32XshRrV1, 0, 1, u64::MAX)
        .expect("odd increment is valid");
    let mut stream = RandomStream::from_state(exhausted).expect("valid exhausted snapshot");
    assert_eq!(
        stream.try_next_u32(),
        Err(RandomStreamError::DrawIndexExhausted)
    );
}
