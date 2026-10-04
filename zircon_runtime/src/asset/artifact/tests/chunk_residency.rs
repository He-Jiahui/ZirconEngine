use super::*;

#[test]
fn external_lease_tracker_reuses_a_slot_for_an_expired_address_record() {
    let bytes: Arc<[u8]> = Arc::from([1_u8, 2, 3]);
    let _consumer = Arc::clone(&bytes);
    let payload_identity = Arc::as_ptr(&bytes).cast::<u8>() as usize;
    let expired_lease = {
        let released: Arc<[u8]> = Arc::from([9_u8]);
        RetiredArtifactChunkLease {
            bytes: Arc::downgrade(&released),
            byte_size: released.len(),
            slot_index: 0,
        }
    };
    let mut state = ArtifactChunkResidencyState::default();
    state
        .retired_external_lease_slots
        .push(Some(payload_identity));
    state
        .retired_external_leases
        .insert(payload_identity, expired_lease);

    state.track_external_lease(&bytes);

    let tracked = state
        .retired_external_leases
        .get(&payload_identity)
        .unwrap();
    assert_eq!(tracked.slot_index, 0);
    assert_eq!(tracked.bytes.upgrade().as_deref(), Some(bytes.as_ref()));
    assert_eq!(tracked.byte_size, bytes.len());
}

#[test]
fn external_lease_tracker_overwrites_one_live_record_at_its_metadata_cap() {
    let leases = (0..MAX_RETIRED_EXTERNAL_LEASES)
        .map(|_| Arc::<[u8]>::from([1_u8]))
        .collect::<Vec<_>>();
    let _consumers = leases.iter().map(Arc::clone).collect::<Vec<_>>();
    let first_identity = Arc::as_ptr(&leases[0]).cast::<u8>() as usize;
    let mut state = ArtifactChunkResidencyState::default();
    for lease in &leases {
        state.track_external_lease(lease);
    }

    let replacement: Arc<[u8]> = Arc::from([2_u8]);
    let _replacement_consumer = Arc::clone(&replacement);
    let replacement_identity = Arc::as_ptr(&replacement).cast::<u8>() as usize;
    state.track_external_lease(&replacement);

    assert_eq!(
        state.retired_external_leases.len(),
        MAX_RETIRED_EXTERNAL_LEASES
    );
    assert!(!state.retired_external_leases.contains_key(&first_identity));
    assert!(state
        .retired_external_leases
        .contains_key(&replacement_identity));
    assert_eq!(state.external_lease_tracking_overflows, 1);
}

#[test]
fn external_lease_tracker_skips_a_cache_owned_payload_without_a_consumer() {
    let bytes: Arc<[u8]> = Arc::from([1_u8, 2, 3]);
    let mut state = ArtifactChunkResidencyState::default();

    state.track_external_lease(&bytes);

    assert!(state.retired_external_leases.is_empty());
    assert!(state.retired_external_lease_slots.is_empty());

    let _consumer = Arc::clone(&bytes);
    state.track_external_lease(&bytes);

    assert_eq!(state.retired_external_leases.len(), 1);
    assert_eq!(state.retired_external_lease_slots.len(), 1);
}

#[test]
fn eviction_index_skips_stale_accesses_and_selects_the_oldest_resident_key() {
    let first = Arc::new(test_cache_key("first"));
    let second = Arc::new(test_cache_key("second"));
    let mut state = ArtifactChunkResidencyState::default();
    state.entries.insert(
        Arc::clone(&first),
        test_resident_chunk(Arc::clone(&first), 1, 1),
    );
    state.entries.insert(
        Arc::clone(&second),
        test_resident_chunk(Arc::clone(&second), 2, 2),
    );
    state.record_eviction_candidate(Arc::clone(&first), 1);
    state.record_eviction_candidate(Arc::clone(&second), 2);

    state.entries.get_mut(&first).unwrap().last_access = 3;
    state.record_eviction_candidate(Arc::clone(&first), 3);

    assert_eq!(state.pop_oldest_resident_key(), Some(second));
}

#[test]
fn eviction_index_reuses_the_resident_cache_key_allocation() {
    let key = Arc::new(test_cache_key("shared-key"));
    let mut state = ArtifactChunkResidencyState::default();
    state.entries.insert(
        Arc::clone(&key),
        test_resident_chunk(Arc::clone(&key), 1, 1),
    );

    state.record_eviction_candidate(Arc::clone(&key), 1);

    let Reverse((_, candidate_key)) = state
        .eviction_candidates
        .peek()
        .expect("resident entry should add one eviction candidate");
    assert!(Arc::ptr_eq(candidate_key, &key));
}

#[test]
fn eviction_index_stays_bounded_during_hot_cache_hits() {
    let key = Arc::new(test_cache_key("hot"));
    let mut state = ArtifactChunkResidencyState::default();
    state.entries.insert(
        Arc::clone(&key),
        test_resident_chunk(Arc::clone(&key), 1, 0),
    );

    for access in 1..=16 {
        state.entries.get_mut(&key).unwrap().last_access = access;
        state.record_eviction_candidate(Arc::clone(&key), access);
    }

    assert!(state.eviction_candidates.len() <= MAX_EVICTION_INDEX_CANDIDATES_PER_RESIDENT_ENTRY);
    assert!(state.eviction_index_rebuilds > 0);
}

fn test_cache_key(content_hash: &str) -> ArtifactChunkCacheKey {
    ArtifactChunkCacheKey {
        chunk_root: Arc::new(PathBuf::from("artifact-cache-test")),
        content_hash: Arc::from(content_hash),
    }
}

fn test_resident_chunk(
    cache_key: Arc<ArtifactChunkCacheKey>,
    byte: u8,
    last_access: u64,
) -> ResidentArtifactChunk {
    ResidentArtifactChunk {
        cache_key,
        bytes: Arc::from([byte]),
        last_access,
    }
}
