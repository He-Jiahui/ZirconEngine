use super::*;

fn limits() -> EditorAssetImportAdmissionLimits {
    EditorAssetImportAdmissionLimits::new(8, usize::MAX, Duration::from_secs(60))
}

fn key(label: &str) -> ImportGenerationKey {
    ImportGenerationKey::new(
        AssetUuid::from_stable_label(label),
        Arc::new(AssetUri::parse(&format!("res://textures/{label}.png")).unwrap()),
        Arc::from(format!("digest-{label}")),
    )
}

#[test]
fn generated_mutex_groups_are_distinct_valid_submission_values() {
    let mut state = ImportFlowState::default();
    let first = state.allocate_mutex_group().unwrap();
    let second = state.allocate_mutex_group().unwrap();

    assert_eq!(first.as_str(), "asset_import_0000000000000000");
    assert_eq!(second.as_str(), "asset_import_0000000000000001");
    assert_ne!(first, second);
}

#[test]
fn reserve_rejects_new_flights_after_mutex_group_identity_exhaustion() {
    let mut state = ImportFlowState::default();
    state.next_mutex_group = u64::MAX;

    assert!(state
        .reserve(
            key("mutex-group-final"),
            EditorAssetImportReason::Manual,
            Instant::now(),
            limits(),
        )
        .is_ok());

    let overflow = state.reserve(
        key("mutex-group-overflow"),
        EditorAssetImportReason::Manual,
        Instant::now(),
        limits(),
    );
    assert!(matches!(
        overflow,
        Err(EditorAssetImportSubmitError::MutexGroupIdentityExhausted)
    ));
    assert_eq!(state.flights.len(), 1);
    assert_eq!(state.active_by_uuid.len(), 1);
    assert_eq!(state.next_flight_identity, 1);
}

#[test]
fn reserve_rejects_new_flights_after_flight_identity_exhaustion() {
    let mut state = ImportFlowState::default();
    state.next_flight_identity = u64::MAX;

    assert!(state
        .reserve(
            key("flight-final"),
            EditorAssetImportReason::Manual,
            Instant::now(),
            limits(),
        )
        .is_ok());

    let overflow = state.reserve(
        key("flight-overflow"),
        EditorAssetImportReason::Manual,
        Instant::now(),
        limits(),
    );
    assert!(matches!(
        overflow,
        Err(EditorAssetImportSubmitError::FlightIdentityExhausted)
    ));
    assert_eq!(state.flights.len(), 1);
    assert_eq!(state.active_by_uuid.len(), 1);
    assert_eq!(state.next_mutex_group, 1);
    assert_eq!(state.next_uuid_lifecycle, 1);
}

#[test]
fn reserve_rejects_new_flights_after_uuid_lifecycle_identity_exhaustion() {
    let mut state = ImportFlowState::default();
    state.next_uuid_lifecycle = u64::MAX;

    assert!(state
        .reserve(
            key("uuid-lifecycle-final"),
            EditorAssetImportReason::Manual,
            Instant::now(),
            limits(),
        )
        .is_ok());

    let overflow = state.reserve(
        key("uuid-lifecycle-overflow"),
        EditorAssetImportReason::Manual,
        Instant::now(),
        limits(),
    );
    assert!(matches!(
        overflow,
        Err(EditorAssetImportSubmitError::UuidLifecycleIdentityExhausted)
    ));
    assert_eq!(state.flights.len(), 1);
    assert_eq!(state.active_by_uuid.len(), 1);
    assert_eq!(state.next_mutex_group, 1);
    assert_eq!(state.next_flight_identity, 1);
}

#[test]
fn reserve_rejects_active_uuid_flight_count_exhaustion() {
    let mut state = ImportFlowState::default();
    let uuid = AssetUuid::from_stable_label("active-flight-count");
    let initial = ImportGenerationKey::new(
        uuid,
        Arc::new(AssetUri::parse("res://textures/active-flight-count-a.png").unwrap()),
        Arc::from("digest-a"),
    );
    let successor = ImportGenerationKey::new(
        uuid,
        Arc::new(AssetUri::parse("res://textures/active-flight-count-b.png").unwrap()),
        Arc::from("digest-b"),
    );
    let reservation = state
        .reserve(
            initial,
            EditorAssetImportReason::Manual,
            Instant::now(),
            limits(),
        )
        .unwrap();
    let token = match reservation {
        ReserveAttempt::Ready(ImportReservation::New {
            begin_uuid: Some(token),
            ..
        }) => token,
        _ => panic!("first UUID reservation must begin a lifecycle"),
    };
    assert!(state.mark_uuid_ready(token));
    state
        .active_by_uuid
        .get_mut(&uuid)
        .expect("test lifecycle must remain active")
        .active_count = usize::MAX;

    let overflow = state.reserve(
        successor,
        EditorAssetImportReason::Manual,
        Instant::now(),
        limits(),
    );

    assert!(matches!(
        overflow,
        Err(EditorAssetImportSubmitError::UuidActiveFlightCountExhausted {
            uuid: exhausted_uuid
        }) if exhausted_uuid == uuid
    ));
    assert_eq!(state.flights.len(), 1);
    assert_eq!(state.active_by_uuid.len(), 1);
}
