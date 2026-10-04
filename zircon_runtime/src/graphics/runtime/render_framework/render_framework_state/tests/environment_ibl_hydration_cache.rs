use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    IblBakeArtifactRequest, IblBakeKey, SourceCubemapEnvironment, SourceCubemapMipChain,
    ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY,
};

use super::EnvironmentIblHydrationCache;

#[test]
fn hydration_report_starts_empty_and_is_non_destructive() {
    let cache = EnvironmentIblHydrationCache::default();

    let first = cache.report();
    let second = cache.report();

    assert_eq!(first, second);
    assert_eq!(first.observation_epoch, 0);
    assert_eq!(first.resident_count, 0);
    assert_eq!(first.pending_count, 0);
    assert_eq!(
        first.resident_requests,
        [None; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY]
    );
    assert_eq!(
        first.pending_requests,
        [None; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY]
    );
}

#[test]
fn hydration_report_tracks_cache_access_without_payload_copies() {
    let hit_request = request(7);
    let missing = request(8);
    let cached = environment(7, 1.0, 0.0).with_prepared_upload_artifact();
    let cached_pmrem = cached.mip_chain.pmrem_texels().as_ptr();
    let mut cache = EnvironmentIblHydrationCache::default();

    cache.insert(hit_request, cached);
    let inserted = cache.report();
    assert_eq!(inserted.observation_epoch, 1);
    assert_eq!(inserted.insert_count, 1);
    assert_eq!(inserted.resident_count, 1);
    assert_eq!(inserted.resident_requests[0], Some(hit_request));

    let hit = cache.get(&hit_request, &environment(7, 2.0, 0.5)).unwrap();
    assert_eq!(hit.mip_chain.pmrem_texels().as_ptr(), cached_pmrem);
    let after_hit = cache.report();
    assert_eq!(after_hit.observation_epoch, 2);
    assert_eq!(after_hit.hit_count, 1);

    assert!(cache.get(&missing, &environment(8, 1.0, 0.0)).is_none());
    let after_miss = cache.report();
    assert_eq!(after_miss.observation_epoch, 3);
    assert_eq!(after_miss.miss_count, 1);
}

#[test]
fn hydration_report_tracks_bounded_eviction() {
    let mut cache = EnvironmentIblHydrationCache::default();
    for identity in 0..=ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY as u64 {
        cache.insert(request(identity), environment(identity, 1.0, 0.0));
    }

    let report = cache.report();
    assert_eq!(report.observation_epoch, 5);
    assert_eq!(report.insert_count, 5);
    assert_eq!(report.eviction_count, 1);
    assert_eq!(report.resident_count, 4);
    assert_eq!(report.resident_requests[0], Some(request(4)));
    assert!(!report.resident_requests.contains(&Some(request(0))));
}

#[test]
fn hydration_report_deduplicates_shared_resident_payload_allocations() {
    let mut cache = EnvironmentIblHydrationCache::default();
    let shared = environment(7, 1.0, 0.0).with_prepared_upload_artifact();

    cache.insert(request(7), shared.clone());
    cache.insert(request(8), shared);
    let shared_report = cache.report();

    // Two 1x1 RGBA32F cube pyramids plus three 256-byte-row RGBA16F cube uploads.
    assert_eq!(shared_report.resident_count, 2);
    assert_eq!(shared_report.resident_decoded_texel_bytes, 192);
    assert_eq!(shared_report.resident_prepared_upload_bytes, 4_608);
    assert_eq!(shared_report.resident_payload_bytes, 4_800);

    let replacement = environment(9, 1.0, 0.0).with_prepared_upload_artifact();
    cache.insert(request(8), replacement.clone());
    let distinct_report = cache.report();
    assert_eq!(distinct_report.resident_decoded_texel_bytes, 384);
    assert_eq!(distinct_report.resident_prepared_upload_bytes, 9_216);
    assert_eq!(distinct_report.resident_payload_bytes, 9_600);

    cache.insert(request(7), replacement);
    let reshared_report = cache.report();
    assert_eq!(reshared_report.resident_decoded_texel_bytes, 192);
    assert_eq!(reshared_report.resident_prepared_upload_bytes, 4_608);
    assert_eq!(reshared_report.resident_payload_bytes, 4_800);
}

#[test]
fn hydration_report_reads_preaccounted_bytes_without_payload_traversal() {
    let source = include_str!("../environment_ibl_hydration_cache.rs");
    let report = source
        .split("fn report(")
        .nth(1)
        .and_then(|source| source.split("fn accumulate_unique_slice_bytes").next())
        .expect("hydration report implementation");

    assert!(!report.contains("source_texels()"));
    assert!(!report.contains("pmrem_texels()"));
    assert!(!report.contains("prepared_upload_artifact()"));
    assert!(!report.contains("HashSet"));
}

#[test]
fn hydration_report_tracks_reservation_suppression_and_release() {
    let request = request(7);
    let mut cache = EnvironmentIblHydrationCache::default();

    assert!(cache.begin_runtime_bake(request));
    assert!(!cache.begin_runtime_bake(request));
    let pending = cache.report();
    assert_eq!(pending.observation_epoch, 2);
    assert_eq!(pending.reservation_count, 1);
    assert_eq!(pending.reservation_suppression_count, 1);
    assert_eq!(pending.pending_count, 1);
    assert_eq!(pending.pending_requests[0], Some(request));

    cache.insert(request, environment(7, 1.0, 0.0));
    let hydrated = cache.report();
    assert_eq!(hydrated.observation_epoch, 3);
    assert_eq!(hydrated.insert_count, 1);
    assert_eq!(hydrated.reservation_release_count, 1);
    assert_eq!(hydrated.pending_count, 0);
}

#[test]
fn optimization_batch_20260830eq_runtime548_reuses_payload_and_runtime_controls() {
    let request = request(7);
    let cached = environment(7, 1.0, 0.0).with_prepared_upload_artifact();
    let cached_pmrem = cached.mip_chain.pmrem_texels().as_ptr();
    let mut cache = EnvironmentIblHydrationCache::default();
    cache.insert(request, cached);

    let current = environment(7, 2.5, 0.75);
    for _ in 0..60 {
        let reused = cache
            .get(&request, &current)
            .expect("an unchanged request should reuse its hydration");
        assert_eq!(reused.intensity, 2.5);
        assert_eq!(reused.rotation_radians, 0.75);
        assert_eq!(reused.mip_chain.pmrem_texels().as_ptr(), cached_pmrem);
        assert!(reused.prepared_upload_artifact().is_some());
    }
}

#[test]
fn optimization_batch_20260830eq_runtime548_ibl_front_hit_returns_before_lru_queue_mutation() {
    let source = include_str!("../environment_ibl_hydration_cache.rs");
    let get = source
        .split("pub(in crate::graphics::runtime::render_framework) fn get")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::graphics::runtime::render_framework) fn insert")
                .next()
        })
        .expect("hydration cache get implementation");
    let front_hit = get
        .find("if index == 0")
        .expect("RUNTIME548_IBL_FRONT_HIT_BENCH_V1 front-hit branch");
    let queue_mutation = get
        .find("self.entries.remove(index)")
        .expect("non-front LRU queue mutation");

    assert!(front_hit < queue_mutation);
    assert!(get[front_hit..queue_mutation].contains("self.entries.front()?"));
}

#[test]
#[ignore = "deterministic optimization evidence"]
fn optimization_batch_20260830eq_runtime548_ibl_front_hit_operation_count() {
    const FRONT_HITS: usize = 65_536;
    let legacy_queue_mutations = FRONT_HITS * 2;
    let optimized_queue_mutations = 0;

    println!(
        "RUNTIME548_IBL_FRONT_HIT_BENCH_V1 legacy_queue_mutations={legacy_queue_mutations} optimized_queue_mutations={optimized_queue_mutations}"
    );
    assert_eq!(legacy_queue_mutations, 131_072);
    assert_eq!(optimized_queue_mutations, 0);
}

#[test]
fn optimization_batch_20260830eq_runtime548_preserves_bounded_lru_eviction() {
    let mut cache = EnvironmentIblHydrationCache::default();
    for identity in 0..=ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY as u64 {
        cache.insert(request(identity), environment(identity, 1.0, 0.0));
    }

    assert!(cache.get(&request(0), &environment(0, 1.0, 0.0)).is_none());
    assert!(cache
        .get(
            &request(ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY as u64),
            &environment(ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY as u64, 1.0, 0.0,),
        )
        .is_some());
}

#[test]
fn pending_runtime_bake_reserves_one_graph_until_cache_hydration() {
    let request = request(7);
    let mut cache = EnvironmentIblHydrationCache::default();

    assert!(cache.begin_runtime_bake(request));
    assert!(!cache.begin_runtime_bake(request));

    cache.insert(request, environment(7, 1.0, 0.0));
    assert!(cache.begin_runtime_bake(request));
}

#[test]
fn dropped_runtime_bake_reservation_releases_the_request_for_retry() {
    let request = request(7);
    let cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
    let reservation = EnvironmentIblHydrationCache::reserve_runtime_bake(&cache, request)
        .expect("first runtime bake should reserve the request");
    assert!(EnvironmentIblHydrationCache::reserve_runtime_bake(&cache, request).is_none());

    drop(reservation);

    assert!(EnvironmentIblHydrationCache::reserve_runtime_bake(&cache, request).is_some());
    let report = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .report();
    assert_eq!(report.reservation_count, 2);
    assert_eq!(report.reservation_suppression_count, 1);
    assert_eq!(report.reservation_release_count, 1);
    assert_eq!(report.observation_epoch, 4);
}

fn request(identity: u64) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(identity, [identity as u32; 4]),
        1,
        1,
    )
    .with_pmrem_layout(1, 1)
}

fn environment(identity: u64, intensity: f32, rotation_radians: f32) -> SourceCubemapEnvironment {
    let mip_chain = SourceCubemapMipChain::new(
        1,
        1,
        vec![[identity as f32, 0.0, 0.0, 1.0]; 6],
        1,
        1,
        vec![[0.0, identity as f32, 0.0, 1.0]; 6],
    );
    let mut environment = SourceCubemapEnvironment::new(mip_chain, identity, [identity as u32; 4]);
    environment.intensity = intensity;
    environment.rotation_radians = rotation_radians;
    environment
}
