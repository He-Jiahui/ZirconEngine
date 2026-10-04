use super::*;

#[test]
fn copy_offsets_keep_wgpu_row_alignment() {
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    assert_eq!(aligned_copy_offset(0), Some(0));
    assert_eq!(aligned_copy_offset(1), Some(alignment));
    assert_eq!(aligned_copy_offset(alignment), Some(alignment));
}

#[test]
fn prepared_upload_batch_encodes_into_the_caller_frame_encoder() {
    let product = include_str!("../upload_batch.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("product source precedes tests");

    assert!(product.contains("encoder: &mut wgpu::CommandEncoder"));
    assert!(product.contains("frame_uploads: &mut WgpuBufferUploadBatch"));
    assert!(product.contains("frame_uploads.push("));
    assert!(product.contains("return Err(CubemapUploadStagingError::MissingTarget)"));
    assert!(!product.contains("cubemap upload target was validated before batch publication"));
    assert!(!product.contains("queue.submit("));
    assert!(!product.contains("queue.write_buffer("));
    assert!(!product.contains("encoder.finish()"));
}

#[test]
fn staging_statistics_reset_last_sample_and_preserve_bounded_history() {
    let committed = crate::core::framework::render::SourceCubemapUploadKey {
        source_revision: 11,
        source_hash: [1; 4],
        pmrem_hash: [2; 4],
        irradiance_cube_hash: [3; 4],
    };
    let pending = crate::core::framework::render::SourceCubemapUploadKey {
        source_revision: 12,
        source_hash: [4; 4],
        pmrem_hash: [5; 4],
        irradiance_cube_hash: [6; 4],
    };
    let mut statistics = CubemapUploadStagingStatistics::default();

    statistics.begin_observation();
    statistics.record_host_capacity_growth(0, 4_096);
    statistics.record_gpu_capacity_growth(0, 65_536);
    statistics.record_scheduled_batch(4_096, 12);
    let scheduled = statistics.report(committed, Some(pending), 4_096, 65_536, 48, 96, 48, 192);

    assert_eq!(scheduled.observation_epoch, 1);
    assert_eq!(scheduled.committed_upload_key, committed);
    assert_eq!(scheduled.pending_upload_key, Some(pending));
    assert_eq!(scheduled.last_scheduled_upload_bytes, 4_096);
    assert_eq!(scheduled.last_scheduled_copy_count, 12);
    assert_eq!(scheduled.peak_scheduled_upload_bytes, 4_096);
    assert_eq!(scheduled.peak_scheduled_copy_count, 12);
    assert_eq!(scheduled.cumulative_scheduled_upload_bytes, 4_096);
    assert_eq!(scheduled.scheduled_upload_batch_count, 1);
    assert_eq!(scheduled.host_staging_capacity_bytes, 4_096);
    assert_eq!(scheduled.gpu_staging_capacity_bytes, 65_536);
    assert_eq!(scheduled.host_staging_growth_batch_count, 1);
    assert_eq!(scheduled.gpu_staging_growth_batch_count, 1);

    statistics.begin_observation();
    statistics.record_host_capacity_growth(4_096, 4_096);
    statistics.record_gpu_capacity_growth(65_536, 65_536);
    let steady = statistics.report(committed, None, 4_096, 65_536, 48, 96, 48, 192);

    assert_eq!(steady.observation_epoch, 2);
    assert_eq!(steady.last_scheduled_upload_bytes, 0);
    assert_eq!(steady.last_scheduled_copy_count, 0);
    assert_eq!(steady.peak_scheduled_upload_bytes, 4_096);
    assert_eq!(steady.peak_scheduled_copy_count, 12);
    assert_eq!(steady.cumulative_scheduled_upload_bytes, 4_096);
    assert_eq!(steady.scheduled_upload_batch_count, 1);
    assert_eq!(steady.host_staging_growth_batch_count, 1);
    assert_eq!(steady.gpu_staging_growth_batch_count, 1);
    assert_eq!(steady.resident_texture_bytes, 192);
}

#[test]
fn staging_statistics_use_saturating_counters() {
    let mut statistics = CubemapUploadStagingStatistics {
        observation_epoch: u64::MAX,
        cumulative_scheduled_upload_bytes: u64::MAX,
        scheduled_upload_batch_count: u64::MAX,
        host_staging_growth_batch_count: u64::MAX,
        gpu_staging_growth_batch_count: u64::MAX,
        ..CubemapUploadStagingStatistics::default()
    };

    statistics.begin_observation();
    statistics.record_host_capacity_growth(0, 1);
    statistics.record_gpu_capacity_growth(0, 1);
    statistics.record_scheduled_batch(1, 1);
    let report = statistics.report(
        crate::core::framework::render::SourceCubemapUploadKey::default(),
        None,
        1,
        1,
        48,
        96,
        48,
        192,
    );

    assert_eq!(report.observation_epoch, u64::MAX);
    assert_eq!(report.cumulative_scheduled_upload_bytes, u64::MAX);
    assert_eq!(report.scheduled_upload_batch_count, u64::MAX);
    assert_eq!(report.host_staging_growth_batch_count, u64::MAX);
    assert_eq!(report.gpu_staging_growth_batch_count, u64::MAX);
}
