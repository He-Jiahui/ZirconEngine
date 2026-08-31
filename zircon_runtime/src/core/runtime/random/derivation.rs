use zr_contracts::random::{
    assembly::sequence_id_from_uniform_u64, RandomAlgorithmId, RandomStreamKey,
};

use super::RandomStream;

const RANDOM_DERIVATION_DOMAIN: &[u8] = b"zircon.random.stream.v1";

pub(super) fn derive_stream(
    algorithm: RandomAlgorithmId,
    master_seed: u64,
    master_seed_generation: u64,
    key: RandomStreamKey,
) -> RandomStream {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RANDOM_DERIVATION_DOMAIN);
    hasher.update(&algorithm.stable_id().to_le_bytes());
    hasher.update(&master_seed.to_le_bytes());
    hasher.update(&master_seed_generation.to_le_bytes());
    append_stream_key(&mut hasher, key);
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    let seed = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let sequence = sequence_id_from_uniform_u64(u64::from_le_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]));
    RandomStream::from_seed(algorithm, seed, sequence)
}

fn append_stream_key(hasher: &mut blake3::Hasher, key: RandomStreamKey) {
    let world = key.world();
    hasher.update(&world.id().to_le_bytes());
    hasher.update(&world.generation().to_le_bytes());
    match key.entity() {
        Some(entity) => {
            hasher.update(&[1]);
            hasher.update(&entity.id().to_le_bytes());
            hasher.update(&entity.generation().to_le_bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
    hasher.update(&key.system().value().to_le_bytes());
    hasher.update(&key.purpose().value().to_le_bytes());
    hasher.update(&key.authoring_seed().to_le_bytes());
}
