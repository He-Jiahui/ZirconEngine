use super::super::{
    RandomAlgorithmId, RandomServiceCheckpoint, RandomServiceCheckpointError, RandomServiceState,
    RandomState, RandomStreamCheckpoint,
};
use super::key;

const SERVICE_GENERATION: u64 = 3;

fn stream(id: u64, draw_index: u64) -> RandomStreamCheckpoint {
    stream_for_generation(id, draw_index, SERVICE_GENERATION)
}

fn stream_for_generation(
    id: u64,
    draw_index: u64,
    master_seed_generation: u64,
) -> RandomStreamCheckpoint {
    RandomStreamCheckpoint::new(
        key(id),
        RandomState::new(RandomAlgorithmId::Pcg32XshRrV1, id, 5, draw_index)
            .expect("odd increment is valid"),
        master_seed_generation,
    )
}

#[test]
fn checkpoint_round_trip_preserves_version_authority_and_canonical_stream_order() {
    let service = RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 17, SERVICE_GENERATION);
    let checkpoint = RandomServiceCheckpoint::try_new(service, vec![stream(4, 9), stream(8, 13)])
        .expect("strictly ordered streams should form a checkpoint");
    let encoded = serde_json::to_string(&checkpoint).expect("checkpoint should serialize");
    let restored: RandomServiceCheckpoint =
        serde_json::from_str(&encoded).expect("canonical checkpoint should deserialize");

    assert_eq!(restored, checkpoint);
    assert_eq!(restored.format_version(), 2);
    assert_eq!(restored.service_state(), service);
    assert!(
        restored
            .streams()
            .iter()
            .all(|stream| stream.master_seed_generation() == SERVICE_GENERATION)
    );
}

#[test]
fn checkpoint_rejects_duplicate_or_descending_stream_keys() {
    let service = RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 17, SERVICE_GENERATION);
    assert_eq!(
        RandomServiceCheckpoint::try_new(service, vec![stream(4, 9), stream(4, 13)]),
        Err(RandomServiceCheckpointError::NonCanonicalStreamOrder { index: 1 })
    );
    assert_eq!(
        RandomServiceCheckpoint::try_new(service, vec![stream(8, 9), stream(4, 13)]),
        Err(RandomServiceCheckpointError::NonCanonicalStreamOrder { index: 1 })
    );
}

#[test]
fn checkpoint_rejects_streams_from_another_authority_generation() {
    let service = RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 17, SERVICE_GENERATION);
    assert_eq!(
        RandomServiceCheckpoint::try_new(
            service,
            vec![stream_for_generation(4, 9, SERVICE_GENERATION - 1)],
        ),
        Err(
            RandomServiceCheckpointError::StreamAuthorityGenerationMismatch {
                index: 0,
                service_generation: SERVICE_GENERATION,
                stream_generation: SERVICE_GENERATION - 1,
            }
        )
    );

    let checkpoint =
        RandomServiceCheckpoint::try_new(service, vec![stream(4, 9)]).expect("valid checkpoint");
    let mut encoded = serde_json::to_value(checkpoint).expect("checkpoint should serialize");
    encoded["streams"][0]["master_seed_generation"] =
        serde_json::Value::from(SERVICE_GENERATION - 1);
    assert!(serde_json::from_value::<RandomServiceCheckpoint>(encoded).is_err());
}

#[test]
fn checkpoint_deserialization_hard_cuts_version_one_and_unknown_versions() {
    let service = RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 17, SERVICE_GENERATION);
    let checkpoint =
        RandomServiceCheckpoint::try_new(service, vec![stream(4, 9)]).expect("valid checkpoint");
    let mut version_one = serde_json::to_value(&checkpoint).expect("checkpoint should serialize");
    version_one["format_version"] = serde_json::Value::from(1);
    version_one["streams"][0]
        .as_object_mut()
        .expect("stream wire object")
        .remove("master_seed_generation");
    assert!(serde_json::from_value::<RandomServiceCheckpoint>(version_one).is_err());

    let mut unknown = serde_json::to_value(checkpoint).expect("checkpoint should serialize");
    unknown["format_version"] = serde_json::Value::from(3);
    assert!(serde_json::from_value::<RandomServiceCheckpoint>(unknown).is_err());
}
