use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::artifact::{
    resolve_ibl_bake_artifact_runtime_dispatch, write_ibl_bake_artifact_runtime_dispatch_readback,
    write_ibl_bake_artifact_runtime_readback, IblBakeArtifactCacheRead, IblBakeArtifactCacheStore,
    IblBakeArtifactRuntimeDispatchReadbackStatus, IblBakeArtifactRuntimeWritebackStatus,
    IBL_BAKE_RUNTIME_CACHE_DIRECTORY, IBL_BAKE_RUNTIME_CACHE_EXTENSION,
};
use zircon_runtime::core::framework::render::{
    resolve_ibl_bake_artifact_payload, select_ibl_bake_artifact,
    source_cubemap_environment_with_bake_artifact, source_cubemap_mip_chain_with_bake_artifact,
    source_cubemap_sample_count, IblBakeArtifactBlob, IblBakeArtifactBlobCandidate,
    IblBakeArtifactBlobError, IblBakeArtifactCandidate, IblBakeArtifactContents,
    IblBakeArtifactDescriptor, IblBakeArtifactHeader, IblBakeArtifactHeaderError,
    IblBakeArtifactPayload, IblBakeArtifactPayloadError, IblBakeArtifactReadbackError,
    IblBakeArtifactReadbackSectionKind, IblBakeArtifactReadbackSections, IblBakeArtifactRequest,
    IblBakeArtifactSource, ProceduralSkyParams, SourceCubemapBakeArtifactError,
    SourceCubemapEnvironment, SourceCubemapIrradianceCube, SourceCubemapMipChain,
    IBL_BAKE_ALGORITHM_VERSION, IBL_BAKE_ARTIFACT_HEADER_SIZE,
    IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
    SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};

#[test]
fn runtime_environment_ibl_bake_artifact_header_round_trips_current_algorithm_version() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let request = IblBakeArtifactRequest::new(key, 128, 8);
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);

    assert_eq!(descriptor.algorithm_version(), IBL_BAKE_ALGORITHM_VERSION);
    assert!(descriptor.is_current_for(&request));

    let header = IblBakeArtifactHeader::from_descriptor(descriptor);
    let encoded = header.encode();
    assert_eq!(encoded.len(), IBL_BAKE_ARTIFACT_HEADER_SIZE);

    let decoded = IblBakeArtifactHeader::decode(&encoded).expect("header should decode");
    assert_eq!(decoded.descriptor(), descriptor);

    let stale = descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION - 1);
    assert!(!stale.is_current_for(&request));
    assert_eq!(
        IblBakeArtifactHeader::decode(&encoded[..encoded.len() - 1]),
        Err(IblBakeArtifactHeaderError::InvalidLength)
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_selection_prefers_derived_before_cache_without_dispatch() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let request = IblBakeArtifactRequest::new(key, 128, 8);
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let stale_derived =
        descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let candidates = [
        IblBakeArtifactCandidate::runtime_cache(descriptor),
        IblBakeArtifactCandidate::asset_derived(stale_derived),
        IblBakeArtifactCandidate::asset_derived(descriptor),
    ];

    let selection = select_ibl_bake_artifact(&request, &candidates);

    assert_eq!(
        selection.source(),
        IblBakeArtifactSource::AssetDerivedArtifact
    );
    assert_eq!(selection.environment_compute_dispatch_count(), 0);
    assert!(!selection.requires_runtime_compute());
    assert_eq!(selection.rejected_candidate_count(), 1);
}

#[test]
fn runtime_environment_ibl_bake_artifact_miss_falls_back_to_runtime_compute_dispatches() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let request = IblBakeArtifactRequest::new(key, 128, 8)
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let wrong_layout =
        IblBakeArtifactDescriptor::current(key, 64, 7, IblBakeArtifactContents::PMREM_SH9_IEM);

    let selection = select_ibl_bake_artifact(
        &request,
        &[IblBakeArtifactCandidate::asset_derived(wrong_layout)],
    );

    assert_eq!(selection.source(), IblBakeArtifactSource::RuntimeCompute);
    assert!(selection.requires_runtime_compute());
    assert_eq!(selection.environment_compute_dispatch_count(), 3);
    assert_eq!(selection.rejected_candidate_count(), 1);
}

#[test]
fn runtime_environment_ibl_bake_artifact_payload_round_trips_pmrem_sh9_iem_bytes() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let face_size = 4;
    let mip_count = 3;
    let sample_count = source_cubemap_sample_count(face_size, mip_count);
    let pmrem_texels = (0..sample_count)
        .map(|index| {
            let gradient = index as f32 / sample_count as f32;
            [gradient, 0.5, 1.0, 2.0]
        })
        .collect::<Vec<_>>();
    let cubemap = shared_layout_mip_chain(face_size, mip_count, pmrem_texels);
    let iem_sample_count = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        * SOURCE_CUBEMAP_FACE_COUNT;
    let mut iem_texels = vec![[0.25, 0.5, 1.0]; iem_sample_count];
    iem_texels[0] = [2.0, 0.5, 0.25];
    iem_texels[iem_sample_count - 1] = [0.125, 0.25, 0.5];
    let irradiance_cube =
        SourceCubemapIrradianceCube::new(SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, iem_texels);
    let descriptor = IblBakeArtifactDescriptor::current(
        key,
        face_size,
        mip_count,
        IblBakeArtifactContents::PMREM_SH9_IEM,
    );

    let payload =
        IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, Some(&irradiance_cube))
            .expect("payload should encode");

    let pmrem_bytes = sample_count * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES;
    let sh9_bytes = IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES;
    let iem_bytes = iem_sample_count * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES;
    assert_eq!(payload.bytes().len(), pmrem_bytes + sh9_bytes + iem_bytes);
    assert_eq!(
        payload.bytes().len(),
        descriptor.expected_payload_size_bytes()
    );
    assert_eq!(payload.pmrem_rgba16f_byte_range(), Some(0..pmrem_bytes));
    assert_eq!(
        payload.irradiance_sh9_byte_range(),
        Some(pmrem_bytes..pmrem_bytes + sh9_bytes)
    );
    assert_eq!(
        payload.irradiance_cube_rgba16f_byte_range(),
        Some(pmrem_bytes + sh9_bytes..pmrem_bytes + sh9_bytes + iem_bytes)
    );

    let decoded =
        IblBakeArtifactPayload::decode(descriptor, payload.bytes()).expect("payload should decode");
    let decoded_pmrem = decoded.decode_pmrem_texels().expect("pmrem should decode");
    assert_eq!(decoded_pmrem.len(), sample_count);
    assert!((decoded_pmrem[0][1] - 0.5).abs() < 0.001);
    assert!((decoded_pmrem[0][2] - 1.0).abs() < 0.001);
    assert!((decoded_pmrem[0][3] - 2.0).abs() < 0.001);
    assert_eq!(
        decoded.decode_irradiance_sh9().expect("sh9 should decode"),
        *cubemap.irradiance_sh9()
    );
    let decoded_iem = decoded
        .decode_irradiance_cube_texels()
        .expect("iem should decode");
    assert_eq!(decoded_iem.len(), iem_sample_count);
    assert!((decoded_iem[0][0] - 2.0).abs() < 0.001);
    assert!((decoded_iem[0][2] - 0.25).abs() < 0.001);
    assert!((decoded_iem[iem_sample_count - 1][0] - 0.125).abs() < 0.001);
}

#[test]
fn runtime_environment_ibl_bake_artifact_payload_rejects_missing_iem_and_bad_length() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let face_size = 4;
    let mip_count = 3;
    let cubemap = shared_layout_mip_chain(
        face_size,
        mip_count,
        vec![[0.25, 0.5, 1.0, 1.0]; source_cubemap_sample_count(face_size, mip_count)],
    );
    let with_iem = IblBakeArtifactDescriptor::current(
        key,
        face_size,
        mip_count,
        IblBakeArtifactContents::PMREM_SH9_IEM,
    );

    assert_eq!(
        IblBakeArtifactPayload::from_source_cubemap(with_iem, &cubemap, None),
        Err(IblBakeArtifactPayloadError::MissingIrradianceCube)
    );
    assert_eq!(
        IblBakeArtifactPayload::decode(with_iem, &[0; 7]),
        Err(IblBakeArtifactPayloadError::InvalidPayloadLength {
            expected: with_iem.expected_payload_size_bytes(),
            actual: 7
        })
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_blob_round_trips_header_and_payload() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let face_size = 128;
    let mip_count = 8;
    let cubemap = shared_layout_mip_chain(
        face_size,
        mip_count,
        vec![[0.25, 0.5, 1.0, 1.0]; source_cubemap_sample_count(face_size, mip_count)],
    );
    let descriptor = IblBakeArtifactDescriptor::current(
        key,
        face_size,
        mip_count,
        IblBakeArtifactContents::PMREM_SH9,
    );
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, None)
        .expect("payload should encode");
    let blob = IblBakeArtifactBlob::from_payload(payload);

    let encoded = blob.encode();

    assert_eq!(encoded.len(), blob.encoded_len());
    assert_eq!(
        encoded.len(),
        IBL_BAKE_ARTIFACT_HEADER_SIZE + descriptor.expected_payload_size_bytes()
    );
    let decoded = IblBakeArtifactBlob::decode(&encoded).expect("blob should decode");
    assert_eq!(decoded.header(), blob.header());
    assert_eq!(decoded.descriptor(), descriptor);
    assert_eq!(decoded.payload().bytes(), blob.payload().bytes());

    let request = IblBakeArtifactRequest::new(key, face_size, mip_count);
    let current = IblBakeArtifactBlob::decode_current_for_request(&request, &encoded)
        .expect("current blob should decode");
    assert_eq!(current.descriptor(), descriptor);
}

#[test]
fn runtime_environment_ibl_bake_artifact_blob_rejects_truncated_payload_and_stale_descriptor() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let face_size = 128;
    let mip_count = 8;
    let cubemap = shared_layout_mip_chain(
        face_size,
        mip_count,
        vec![[0.125, 0.25, 0.5, 1.0]; source_cubemap_sample_count(face_size, mip_count)],
    );
    let descriptor = IblBakeArtifactDescriptor::current(
        key,
        face_size,
        mip_count,
        IblBakeArtifactContents::PMREM_SH9,
    );
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, None)
        .expect("payload should encode");
    let blob = IblBakeArtifactBlob::from_payload(payload);
    let mut encoded = blob.encode();

    assert_eq!(
        IblBakeArtifactBlob::decode(&encoded[..IBL_BAKE_ARTIFACT_HEADER_SIZE - 1]),
        Err(IblBakeArtifactBlobError::TruncatedHeader {
            expected: IBL_BAKE_ARTIFACT_HEADER_SIZE,
            actual: IBL_BAKE_ARTIFACT_HEADER_SIZE - 1
        })
    );

    encoded.pop();
    assert_eq!(
        IblBakeArtifactBlob::decode(&encoded),
        Err(IblBakeArtifactBlobError::Payload(
            IblBakeArtifactPayloadError::InvalidPayloadLength {
                expected: descriptor.expected_payload_size_bytes(),
                actual: descriptor.expected_payload_size_bytes() - 1
            }
        ))
    );

    let stale_descriptor =
        descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let stale_payload =
        IblBakeArtifactPayload::from_source_cubemap(stale_descriptor, &cubemap, None)
            .expect("stale payload should encode");
    let stale_blob = IblBakeArtifactBlob::from_payload(stale_payload).encode();
    let request = IblBakeArtifactRequest::new(key, face_size, mip_count);

    assert_eq!(
        IblBakeArtifactBlob::decode_current_for_request(&request, &stale_blob),
        Err(IblBakeArtifactBlobError::DescriptorNotCurrent {
            descriptor: stale_descriptor
        })
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_runtime_cache_store_reads_current_blob_as_candidate() {
    let cache_root = unique_cache_root("hit");
    let store = IblBakeArtifactCacheStore::new(cache_root.join(".zircon").join("cache"));
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let blob = pmrem_sh9_blob(descriptor, [0.25, 0.5, 1.0, 1.0]);
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count());

    let path = store
        .write_runtime_cache(&blob)
        .expect("runtime cache write should succeed");

    assert!(path.starts_with(store.cache_root().join(IBL_BAKE_RUNTIME_CACHE_DIRECTORY)));
    assert_eq!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(IBL_BAKE_RUNTIME_CACHE_EXTENSION)
    );

    let read = store
        .read_runtime_cache(&request)
        .expect("runtime cache read should succeed");
    assert!(read.is_hit());
    let candidate = read
        .candidate()
        .expect("runtime cache hit should yield candidate");
    assert_eq!(candidate.source(), IblBakeArtifactSource::RuntimeCache);
    assert_eq!(candidate.descriptor(), descriptor);

    let decoded = match read {
        IblBakeArtifactCacheRead::Hit(decoded) => decoded,
        other => panic!("expected runtime cache hit, got {other:?}"),
    };
    assert_eq!(decoded.descriptor(), descriptor);
    assert_eq!(decoded.payload().bytes(), blob.payload().bytes());

    let _ = fs::remove_dir_all(cache_root);
}

#[test]
fn runtime_environment_ibl_bake_artifact_runtime_cache_store_reports_missing_and_rejected_blobs() {
    let cache_root = unique_cache_root("stale");
    let store = IblBakeArtifactCacheStore::new(cache_root.join(".zircon").join("cache"));
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count());

    assert_eq!(
        store
            .read_runtime_cache(&request)
            .expect("missing cache read should not fail"),
        IblBakeArtifactCacheRead::Missing
    );

    let stale_descriptor =
        descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let stale_blob = pmrem_sh9_blob(stale_descriptor, [0.125, 0.25, 0.5, 1.0]);
    let stale_path = store.runtime_cache_path_for_descriptor(descriptor);
    fs::create_dir_all(stale_path.parent().expect("cache path should have parent"))
        .expect("cache directory should be created");
    fs::write(&stale_path, stale_blob.encode()).expect("stale blob should be written");

    assert_eq!(
        store
            .read_runtime_cache(&request)
            .expect("stale cache read should not fail"),
        IblBakeArtifactCacheRead::Rejected(IblBakeArtifactBlobError::DescriptorNotCurrent {
            descriptor: stale_descriptor
        })
    );

    fs::write(
        &stale_path,
        vec![0; IBL_BAKE_ARTIFACT_HEADER_SIZE.saturating_sub(1)],
    )
    .expect("truncated blob should be written");
    assert_eq!(
        store
            .read_runtime_cache(&request)
            .expect("truncated cache read should not fail"),
        IblBakeArtifactCacheRead::Rejected(IblBakeArtifactBlobError::TruncatedHeader {
            expected: IBL_BAKE_ARTIFACT_HEADER_SIZE,
            actual: IBL_BAKE_ARTIFACT_HEADER_SIZE - 1
        })
    );

    let _ = fs::remove_dir_all(cache_root);
}

#[test]
fn runtime_environment_ibl_bake_artifact_runtime_readback_sections_write_cache_blob() {
    let cache_root = unique_cache_root("runtime_writeback");
    let store = IblBakeArtifactCacheStore::new(cache_root.join(".zircon").join("cache"));
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count())
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let payload = pmrem_sh9_iem_payload(descriptor);

    let report = write_ibl_bake_artifact_runtime_readback(
        &store,
        &request,
        readback_sections_from_payload(&payload),
    )
    .expect("runtime readback should write a cache blob");

    assert_eq!(
        report.status(),
        IblBakeArtifactRuntimeWritebackStatus::Written
    );
    assert!(report.wrote_cache());
    assert_eq!(report.descriptor(), descriptor);
    assert_eq!(report.payload_len(), payload.bytes().len());
    assert_eq!(
        report.encoded_len(),
        IBL_BAKE_ARTIFACT_HEADER_SIZE + payload.bytes().len()
    );
    assert!(report
        .path()
        .expect("written report should include cache path")
        .starts_with(store.cache_root().join(IBL_BAKE_RUNTIME_CACHE_DIRECTORY)));

    let read = store
        .read_runtime_cache(&request)
        .expect("runtime cache read should succeed after writeback");
    let decoded = match read {
        IblBakeArtifactCacheRead::Hit(decoded) => decoded,
        other => panic!("expected runtime cache hit, got {other:?}"),
    };
    assert_eq!(decoded.descriptor(), descriptor);
    assert_eq!(decoded.payload().bytes(), payload.bytes());

    let _ = fs::remove_dir_all(cache_root);
}

#[test]
fn runtime_environment_ibl_bake_artifact_runtime_dispatch_writes_miss_readback_then_hits_cache() {
    let cache_root = unique_cache_root("runtime_dispatch_writeback");
    let store = IblBakeArtifactCacheStore::new(cache_root.join(".zircon").join("cache"));
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count())
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let payload = pmrem_sh9_iem_payload(descriptor);

    let first_dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("missing runtime cache should still resolve a dispatch report");

    assert_eq!(
        first_dispatch.cache_read(),
        &IblBakeArtifactCacheRead::Missing
    );
    assert_eq!(
        first_dispatch.source(),
        IblBakeArtifactSource::RuntimeCompute
    );
    assert!(first_dispatch.requires_runtime_compute());
    assert_eq!(first_dispatch.environment_compute_dispatch_count(), 3);
    assert!(first_dispatch.payload().is_none());

    let writeback = write_ibl_bake_artifact_runtime_dispatch_readback(
        &store,
        &request,
        &first_dispatch,
        readback_sections_from_payload(&payload),
    )
    .expect("runtime dispatch readback should write cache");

    assert_eq!(
        writeback.status(),
        IblBakeArtifactRuntimeDispatchReadbackStatus::Written
    );
    assert!(writeback.wrote_cache());
    assert_eq!(
        writeback
            .writeback()
            .expect("written readback should include writeback report")
            .status(),
        IblBakeArtifactRuntimeWritebackStatus::Written
    );

    let second_dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache should resolve after readback writeback");

    assert!(second_dispatch.cache_read().is_hit());
    assert_eq!(
        second_dispatch.source(),
        IblBakeArtifactSource::RuntimeCache
    );
    assert!(!second_dispatch.requires_runtime_compute());
    assert_eq!(second_dispatch.environment_compute_dispatch_count(), 0);
    assert_eq!(
        second_dispatch
            .payload()
            .expect("runtime cache hit should carry payload")
            .bytes(),
        payload.bytes()
    );

    let _ = fs::remove_dir_all(cache_root);
}

#[test]
fn runtime_environment_ibl_bake_artifact_runtime_dispatch_skips_readback_after_asset_hit() {
    let cache_root = unique_cache_root("runtime_dispatch_asset_hit");
    let store = IblBakeArtifactCacheStore::new(cache_root.join(".zircon").join("cache"));
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count())
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let payload = pmrem_sh9_iem_payload(descriptor);
    let asset_blob = IblBakeArtifactBlob::from_payload(payload.clone());

    let dispatch =
        resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[asset_blob.clone()])
            .expect("asset-derived blob should resolve a dispatch report");

    assert_eq!(
        dispatch.source(),
        IblBakeArtifactSource::AssetDerivedArtifact
    );
    assert_eq!(dispatch.environment_compute_dispatch_count(), 0);
    assert!(!dispatch.requires_runtime_compute());
    assert_eq!(
        dispatch
            .payload()
            .expect("asset hit should carry payload")
            .bytes(),
        asset_blob.payload().bytes()
    );

    let writeback = write_ibl_bake_artifact_runtime_dispatch_readback(
        &store,
        &request,
        &dispatch,
        readback_sections_from_payload(&payload),
    )
    .expect("asset hit should skip runtime readback without touching cache");

    assert_eq!(
        writeback.status(),
        IblBakeArtifactRuntimeDispatchReadbackStatus::SkippedRuntimeComputeNotRequired
    );
    assert!(!writeback.wrote_cache());
    assert!(writeback.writeback().is_none());
    assert_eq!(
        store
            .read_runtime_cache(&request)
            .expect("skipped readback should leave runtime cache missing"),
        IblBakeArtifactCacheRead::Missing
    );

    let _ = fs::remove_dir_all(cache_root);
}

#[test]
fn runtime_environment_ibl_bake_artifact_runtime_writeback_skips_stale_descriptor_without_file() {
    let cache_root = unique_cache_root("runtime_writeback_stale");
    let store = IblBakeArtifactCacheStore::new(cache_root.join(".zircon").join("cache"));
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let stale_descriptor =
        descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count());

    let report = write_ibl_bake_artifact_runtime_readback(
        &store,
        &request,
        IblBakeArtifactReadbackSections::new(stale_descriptor),
    )
    .expect("stale descriptor should be skipped without validating section bytes");

    assert_eq!(
        report.status(),
        IblBakeArtifactRuntimeWritebackStatus::SkippedDescriptorNotCurrent
    );
    assert!(!report.wrote_cache());
    assert_eq!(report.descriptor(), stale_descriptor);
    assert_eq!(report.path(), None);
    assert_eq!(report.payload_len(), 0);
    assert_eq!(report.encoded_len(), 0);
    assert_eq!(
        store
            .read_runtime_cache(&request)
            .expect("missing cache read should not fail"),
        IblBakeArtifactCacheRead::Missing
    );

    let _ = fs::remove_dir_all(cache_root);
}

#[test]
fn runtime_environment_ibl_bake_artifact_readback_sections_reject_missing_and_wrong_lengths() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 4, 3, IblBakeArtifactContents::PMREM_SH9_IEM);
    let payload = pmrem_sh9_iem_payload(descriptor);
    let pmrem = payload.pmrem_rgba16f_byte_range().expect("pmrem range");
    let sh9 = payload.irradiance_sh9_byte_range().expect("sh9 range");
    let mut short_pmrem = payload.bytes()[pmrem.clone()].to_vec();
    short_pmrem.pop();

    assert_eq!(
        IblBakeArtifactReadbackSections::new(descriptor)
            .with_pmrem_rgba16f_bytes(payload.bytes()[pmrem.clone()].to_vec())
            .with_irradiance_sh9_bytes(payload.bytes()[sh9].to_vec())
            .into_payload(),
        Err(IblBakeArtifactReadbackError::MissingSection {
            section: IblBakeArtifactReadbackSectionKind::IrradianceCube
        })
    );
    assert_eq!(
        IblBakeArtifactReadbackSections::new(descriptor)
            .with_pmrem_rgba16f_bytes(short_pmrem)
            .with_irradiance_sh9_bytes(
                payload.bytes()[payload.irradiance_sh9_byte_range().expect("sh9 range")].to_vec()
            )
            .with_irradiance_cube_rgba16f_bytes(
                payload.bytes()[payload
                    .irradiance_cube_rgba16f_byte_range()
                    .expect("iem range")]
                .to_vec()
            )
            .into_payload(),
        Err(IblBakeArtifactReadbackError::InvalidSectionLength {
            section: IblBakeArtifactReadbackSectionKind::Pmrem,
            expected: descriptor
                .expected_pmrem_rgba16f_size_bytes()
                .expect("descriptor should require pmrem"),
            actual: descriptor
                .expected_pmrem_rgba16f_size_bytes()
                .expect("descriptor should require pmrem")
                - 1
        })
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_resolved_payload_prefers_asset_derived_blob_before_cache()
{
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let stale_descriptor =
        descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count());
    let asset_blob = pmrem_sh9_blob(descriptor, [0.75, 0.5, 0.25, 1.0]);
    let cache_blob = pmrem_sh9_blob(descriptor, [0.125, 0.25, 0.5, 1.0]);
    let stale_asset_blob = pmrem_sh9_blob(stale_descriptor, [1.0, 0.0, 0.0, 1.0]);

    let resolved = resolve_ibl_bake_artifact_payload(
        &request,
        &[
            IblBakeArtifactBlobCandidate::runtime_cache(cache_blob.clone()),
            IblBakeArtifactBlobCandidate::asset_derived(stale_asset_blob),
            IblBakeArtifactBlobCandidate::asset_derived(asset_blob.clone()),
        ],
    );

    assert_eq!(
        resolved.source(),
        IblBakeArtifactSource::AssetDerivedArtifact
    );
    assert_eq!(resolved.descriptor(), Some(descriptor));
    assert_eq!(resolved.environment_compute_dispatch_count(), 0);
    assert!(!resolved.requires_runtime_compute());
    assert_eq!(resolved.rejected_candidate_count(), 1);
    assert_eq!(
        resolved
            .payload()
            .expect("asset payload should resolve")
            .bytes(),
        asset_blob.payload().bytes()
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_resolved_payload_uses_cache_when_asset_is_stale() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let stale_descriptor =
        descriptor.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION.saturating_sub(1));
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count());
    let cache_blob = pmrem_sh9_blob(descriptor, [0.25, 0.5, 1.0, 1.0]);
    let stale_asset_blob = pmrem_sh9_blob(stale_descriptor, [1.0, 0.0, 0.0, 1.0]);

    let resolved = resolve_ibl_bake_artifact_payload(
        &request,
        &[
            IblBakeArtifactBlobCandidate::asset_derived(stale_asset_blob),
            IblBakeArtifactBlobCandidate::runtime_cache(cache_blob.clone()),
        ],
    );

    assert_eq!(resolved.source(), IblBakeArtifactSource::RuntimeCache);
    assert_eq!(resolved.descriptor(), Some(descriptor));
    assert_eq!(resolved.environment_compute_dispatch_count(), 0);
    assert_eq!(resolved.rejected_candidate_count(), 1);
    assert_eq!(
        resolved
            .payload()
            .expect("cache payload should resolve")
            .bytes(),
        cache_blob.payload().bytes()
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_resolved_payload_miss_requires_compute_without_payload() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
    let request = IblBakeArtifactRequest::new(key, descriptor.face_size(), descriptor.mip_count())
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let cache_blob = pmrem_sh9_blob(descriptor, [0.25, 0.5, 1.0, 1.0]);

    let resolved = resolve_ibl_bake_artifact_payload(
        &request,
        &[IblBakeArtifactBlobCandidate::runtime_cache(cache_blob)],
    );

    assert_eq!(resolved.source(), IblBakeArtifactSource::RuntimeCompute);
    assert_eq!(resolved.descriptor(), None);
    assert!(resolved.requires_runtime_compute());
    assert_eq!(resolved.environment_compute_dispatch_count(), 3);
    assert_eq!(resolved.rejected_candidate_count(), 1);
    assert!(resolved.blob().is_none());
    assert!(resolved.payload().is_none());
}

#[test]
fn runtime_environment_ibl_bake_artifact_payload_applies_pmrem_sh9_without_replacing_source_mips() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 4, 3, IblBakeArtifactContents::PMREM_SH9);
    let source = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![
            [0.0, 0.25, 0.5, 1.0];
            source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
        ],
    );
    let baked = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![
            [0.75, 0.5, 0.25, 1.0];
            source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
        ],
    );
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &baked, None)
        .expect("payload should encode");

    let applied = source_cubemap_mip_chain_with_bake_artifact(&source, &payload)
        .expect("artifact payload should apply to matching source chain");

    assert_eq!(applied.source_texels(), source.source_texels());
    assert_eq!(applied.pmrem_texels(), baked.pmrem_texels());
    assert_eq!(applied.irradiance_sh9(), baked.irradiance_sh9());
}

#[test]
fn runtime_environment_ibl_bake_artifact_payload_applies_optional_iem_to_environment() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 4, 3, IblBakeArtifactContents::PMREM_SH9_IEM);
    let source = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![
            [0.0, 0.25, 0.5, 1.0];
            source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
        ],
    );
    let baked = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![
            [0.75, 0.5, 0.25, 1.0];
            source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
        ],
    );
    let mut iem_texels = vec![
        [0.125, 0.25, 0.5];
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_FACE_COUNT
    ];
    iem_texels[0] = [1.0, 0.5, 0.25];
    let irradiance_cube =
        SourceCubemapIrradianceCube::new(SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, iem_texels);
    let payload =
        IblBakeArtifactPayload::from_source_cubemap(descriptor, &baked, Some(&irradiance_cube))
            .expect("payload with IEM should encode");
    let mut environment = SourceCubemapEnvironment::new(source, 11, [9, 8, 7, 6]);
    environment.intensity = 1.7;
    environment.rotation_radians = 0.4;

    let applied = source_cubemap_environment_with_bake_artifact(environment, &payload)
        .expect("artifact payload should apply to matching source environment");

    assert_eq!(applied.mip_chain.source_texels()[0], [0.0, 0.25, 0.5, 1.0]);
    assert_eq!(applied.mip_chain.pmrem_texels()[0], [0.75, 0.5, 0.25, 1.0]);
    assert_eq!(applied.irradiance_sh9, *baked.irradiance_sh9());
    assert_eq!(applied.intensity, 1.7);
    assert_eq!(applied.rotation_radians, 0.4);
    let applied_iem = applied
        .irradiance_cube()
        .expect("IEM payload should attach an irradiance cube");
    assert_eq!(
        applied_iem.face_size(),
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE
    );
    assert_eq!(
        applied_iem.texel(
            zircon_runtime::core::framework::render::CubemapFace::PositiveX,
            0,
            0
        ),
        [1.0, 0.5, 0.25]
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_payload_apply_updates_upload_key_without_changing_bake_key(
) {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 4, 3, IblBakeArtifactContents::PMREM_SH9);
    let source = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![
            [0.0, 0.25, 0.5, 1.0];
            source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
        ],
    );
    let environment = SourceCubemapEnvironment::new(source, 11, [9, 8, 7, 6]);
    let source_bake_key = environment.ibl_bake_key();
    let source_upload_key = environment.texture_upload_key();
    let first_payload = IblBakeArtifactPayload::from_source_cubemap(
        descriptor,
        &shared_layout_mip_chain(
            descriptor.face_size(),
            descriptor.mip_count(),
            vec![
                [0.75, 0.5, 0.25, 1.0];
                source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
            ],
        ),
        None,
    )
    .expect("first payload should encode");
    let second_payload = IblBakeArtifactPayload::from_source_cubemap(
        descriptor,
        &shared_layout_mip_chain(
            descriptor.face_size(),
            descriptor.mip_count(),
            vec![
                [0.5, 0.25, 0.125, 1.0];
                source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
            ],
        ),
        None,
    )
    .expect("second payload should encode");

    let first_applied =
        source_cubemap_environment_with_bake_artifact(environment.clone(), &first_payload)
            .expect("first payload should apply");
    let first_applied_again =
        source_cubemap_environment_with_bake_artifact(environment.clone(), &first_payload)
            .expect("same payload should apply deterministically");
    let second_applied =
        source_cubemap_environment_with_bake_artifact(environment, &second_payload)
            .expect("second payload should apply");

    assert_eq!(first_applied.ibl_bake_key(), source_bake_key);
    assert_eq!(
        first_applied.source_revision,
        source_upload_key.source_revision
    );
    assert_eq!(first_applied.source_hash, source_upload_key.source_hash);
    assert_ne!(first_applied.texture_upload_key(), source_upload_key);
    assert_eq!(
        first_applied.texture_upload_key(),
        first_applied_again.texture_upload_key()
    );
    assert_ne!(
        first_applied.texture_upload_key(),
        second_applied.texture_upload_key()
    );
}

#[test]
fn runtime_environment_ibl_bake_artifact_payload_apply_rejects_layout_and_missing_content() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let source = shared_layout_mip_chain(
        4,
        3,
        vec![[0.0, 0.25, 0.5, 1.0]; source_cubemap_sample_count(4, 3)],
    );
    let wrong_layout_descriptor =
        IblBakeArtifactDescriptor::current(key, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let wrong_layout = shared_layout_mip_chain(
        2,
        2,
        vec![[0.75, 0.5, 0.25, 1.0]; source_cubemap_sample_count(2, 2)],
    );
    let wrong_layout_payload =
        IblBakeArtifactPayload::from_source_cubemap(wrong_layout_descriptor, &wrong_layout, None)
            .expect("wrong layout payload should encode");

    assert_eq!(
        source_cubemap_mip_chain_with_bake_artifact(&source, &wrong_layout_payload),
        Err(SourceCubemapBakeArtifactError::LayoutMismatch {
            expected_face_size: 4,
            actual_face_size: 2,
            expected_mip_count: 3,
            actual_mip_count: 2
        })
    );

    let pmrem_only_descriptor =
        IblBakeArtifactDescriptor::current(key, 4, 3, IblBakeArtifactContents::PMREM);
    let pmrem_only_payload =
        IblBakeArtifactPayload::from_source_cubemap(pmrem_only_descriptor, &source, None)
            .expect("pmrem-only payload should encode");

    assert_eq!(
        source_cubemap_mip_chain_with_bake_artifact(&source, &pmrem_only_payload),
        Err(SourceCubemapBakeArtifactError::MissingIrradianceSh9)
    );
}

fn pmrem_sh9_blob(descriptor: IblBakeArtifactDescriptor, texel: [f32; 4]) -> IblBakeArtifactBlob {
    let cubemap = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![texel; source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())],
    );
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, None)
        .expect("payload should encode");
    IblBakeArtifactBlob::from_payload(payload)
}

fn shared_layout_mip_chain(
    face_size: u32,
    mip_count: u32,
    texels: Vec<[f32; 4]>,
) -> SourceCubemapMipChain {
    SourceCubemapMipChain::new(
        face_size,
        mip_count,
        texels.clone(),
        face_size,
        mip_count,
        texels,
    )
}

fn pmrem_sh9_iem_payload(descriptor: IblBakeArtifactDescriptor) -> IblBakeArtifactPayload {
    let cubemap = shared_layout_mip_chain(
        descriptor.face_size(),
        descriptor.mip_count(),
        vec![
            [0.75, 0.5, 0.25, 1.0];
            source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
        ],
    );
    let mut irradiance_texels = vec![
        [0.125, 0.25, 0.5];
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_FACE_COUNT
    ];
    irradiance_texels[0] = [1.0, 0.5, 0.25];
    let irradiance_cube = SourceCubemapIrradianceCube::new(
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        irradiance_texels,
    );
    IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, Some(&irradiance_cube))
        .expect("PMREM/SH9/IEM payload should encode")
}

fn readback_sections_from_payload(
    payload: &IblBakeArtifactPayload,
) -> IblBakeArtifactReadbackSections {
    let pmrem = payload.pmrem_rgba16f_byte_range().expect("pmrem range");
    let sh9 = payload.irradiance_sh9_byte_range().expect("sh9 range");
    let iem = payload
        .irradiance_cube_rgba16f_byte_range()
        .expect("iem range");
    IblBakeArtifactReadbackSections::new(payload.descriptor())
        .with_pmrem_rgba16f_bytes(payload.bytes()[pmrem].to_vec())
        .with_irradiance_sh9_bytes(payload.bytes()[sh9].to_vec())
        .with_irradiance_cube_rgba16f_bytes(payload.bytes()[iem].to_vec())
}

fn unique_cache_root(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_ibl_bake_artifact_cache_{name}_{}_{}",
        std::process::id(),
        timestamp
    ))
}
