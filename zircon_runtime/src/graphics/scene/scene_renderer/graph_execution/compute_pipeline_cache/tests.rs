use super::{
    ComputePipelineBindingLayout, ComputePipelineCache, ComputePipelineCacheBucketKey,
    ComputePipelineCacheEntry, ComputePipelineCacheKey, compute_entry_point_workgroup_size,
    validate_compute_workgroup_limits, validate_expected_workgroup_size,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassDeviceEpoch;
use crate::render_graph::{
    RenderGraphComputePipelineFallbackPolicy, RenderGraphComputePipelineResolutionStatus,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

mod mru_hash_bypass;

#[test]
fn last_good_resolution_requires_matching_family_interface_abi_and_device_epoch() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute-last-good-empty-scene-layout"),
                entries: &[],
            });
    let mut cache = ComputePipelineCache::default();
    let policy = RenderGraphComputePipelineFallbackPolicy::last_good("ao.evaluate", 2);
    let valid_source = "@compute @workgroup_size(1) fn cs_main() {}";
    let invalid_candidate = "@compute @workgroup_size(1) fn other_entry() {}";

    let ready = cache
        .resolve(
            &backend.device,
            &scene_bind_group_layout,
            "ao-evaluate",
            valid_source,
            "cs_main",
            [1, 1, 1],
            &[],
            &policy,
            Some(RenderPassDeviceEpoch::new(7, 3)),
        )
        .expect("valid candidate should publish the family");
    assert_eq!(
        ready.resolution.status,
        RenderGraphComputePipelineResolutionStatus::Ready
    );

    let fallback = cache
        .resolve(
            &backend.device,
            &scene_bind_group_layout,
            "ao-evaluate",
            invalid_candidate,
            "cs_main",
            [1, 1, 1],
            &[],
            &policy,
            Some(RenderPassDeviceEpoch::new(7, 3)),
        )
        .expect("failed compatible candidate should resolve the published family");
    assert_eq!(
        fallback.resolution.status,
        RenderGraphComputePipelineResolutionStatus::UsingLastGood
    );
    assert_ne!(
        fallback.resolution.candidate_artifact_fingerprint,
        fallback.resolution.resolved_artifact_fingerprint
    );
    assert!(fallback.resolution.candidate_failure.is_some());

    let changed_interface = RenderGraphComputePipelineFallbackPolicy::last_good("ao.evaluate", 3);
    assert!(
        cache
            .resolve(
                &backend.device,
                &scene_bind_group_layout,
                "ao-evaluate",
                invalid_candidate,
                "cs_main",
                [1, 1, 1],
                &[],
                &changed_interface,
                Some(RenderPassDeviceEpoch::new(7, 3)),
            )
            .is_err()
    );
    assert!(
        cache
            .resolve(
                &backend.device,
                &scene_bind_group_layout,
                "ao-evaluate",
                invalid_candidate,
                "cs_main",
                [1, 1, 1],
                &[],
                &policy,
                Some(RenderPassDeviceEpoch::new(7, 4)),
            )
            .is_err()
    );
}

#[test]
fn compute_pipeline_cache_keeps_native_epoch_typed_until_report_projection() {
    let source = include_str!("../compute_pipeline_cache.rs");

    assert!(source.contains("active_device_epoch: Option<RenderPassDeviceEpoch>"));
    assert!(source.contains("device_epoch: Option<RenderPassDeviceEpoch>"));
    assert!(source.contains("device_epoch.map(RenderPassDeviceEpoch::raw_parts)"));
    assert!(source.contains("device_epoch.raw_parts()"));
    assert!(!source.contains("active_device_epoch: Option<(u64, u64)>"));
    assert!(!source.contains("device_epoch: Option<(u64, u64)>"));
}

#[test]
fn pipeline_cache_key_requires_full_schema_equality_after_bucket_selection() {
    let key = ComputePipelineCacheKey::new(
        "@compute @workgroup_size(1) fn cs_main() {}",
        "cs_main",
        &[],
    );

    assert!(key.matches(
        "@compute @workgroup_size(1) fn cs_main() {}",
        "cs_main",
        &[]
    ));
    assert!(!key.matches(
        "@compute @workgroup_size(2) fn cs_main() {}",
        "cs_main",
        &[]
    ));
    assert!(!key.matches(
        "@compute @workgroup_size(1) fn cs_main() {}",
        "alternate_entry",
        &[]
    ));
}

#[test]
fn pipeline_cache_evicts_least_recently_used_entry_at_capacity() {
    let mut cache = ComputePipelineCache::with_capacity(2);
    insert_failed_entry(&mut cache, "first");
    insert_failed_entry(&mut cache, "second");
    assert_eq!(cache.entry_count(), 2);

    assert!(touch_entry(&mut cache, "first"));
    insert_failed_entry(&mut cache, "third");

    assert_eq!(cache.entry_count(), 2);
    assert!(touch_entry(&mut cache, "first"));
    assert!(!touch_entry(&mut cache, "second"));
    assert!(touch_entry(&mut cache, "third"));
}

fn insert_failed_entry(cache: &mut ComputePipelineCache, source: &str) {
    let key = ComputePipelineCacheKey::new(source, "cs_main", &[]);
    let bucket_key = ComputePipelineCacheBucketKey::new(source, "cs_main", &[]);
    let use_counter = cache.next_use_counter();
    cache.insert_entry(
        bucket_key,
        key,
        ComputePipelineCacheEntry::Failed("test failure".to_string()),
        use_counter,
    );
}

fn touch_entry(cache: &mut ComputePipelineCache, source: &str) -> bool {
    let bucket_key = ComputePipelineCacheBucketKey::new(source, "cs_main", &[]);
    let use_counter = cache.next_use_counter();
    cache
        .matching_entry(&bucket_key, source, "cs_main", &[], use_counter)
        .is_some()
}

#[test]
fn compute_pipeline_binding_layout_maps_supported_schema_resources() {
    let color_texture = TextureDesc::new(
        "compute-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    );
    let depth_texture = TextureDesc::new(
        "compute-depth",
        16,
        16,
        TextureFormat::Depth32Float,
        TextureUsage::SAMPLED,
    );
    let multisampled_depth_texture = TextureDesc::new(
        "compute-depth-msaa",
        16,
        16,
        TextureFormat::Depth32Float,
        TextureUsage::SAMPLED,
    )
    .with_sample_count(4);

    assert!(matches!(
        ComputePipelineBindingLayout::uniform_buffer(0).wgpu_binding_type(),
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            ..
        }
    ));
    assert!(matches!(
        ComputePipelineBindingLayout::storage_buffer_read(1).wgpu_binding_type(),
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            ..
        }
    ));
    assert!(matches!(
        ComputePipelineBindingLayout::storage_buffer_read_write(2).wgpu_binding_type(),
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            ..
        }
    ));
    assert!(matches!(
        ComputePipelineBindingLayout::sampled_texture(3, &color_texture)
            .expect("color texture layout should be supported")
            .wgpu_binding_type(),
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            ..
        }
    ));
    assert!(matches!(
        ComputePipelineBindingLayout::sampled_texture(4, &depth_texture)
            .expect("depth texture layout should be supported")
            .wgpu_binding_type(),
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            ..
        }
    ));
    assert!(matches!(
        ComputePipelineBindingLayout::sampled_texture(5, &multisampled_depth_texture)
            .expect("multisampled depth texture layout should be supported")
            .wgpu_binding_type(),
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            multisampled: true,
            ..
        }
    ));
    assert!(matches!(
        ComputePipelineBindingLayout::storage_texture_write(6, &color_texture)
            .expect("storage texture layout should be supported")
            .wgpu_binding_type(),
        wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba8Unorm,
            ..
        }
    ));
}

#[test]
fn compute_pipeline_reflects_compute_entry_point_workgroup_size() {
    let workgroup_size = compute_entry_point_workgroup_size(
        "reflect-workgroup-size",
        "@compute @workgroup_size(8, 4, 2) fn cs_main() {}",
        "cs_main",
    )
    .expect("compute entry point should be reflected");

    assert_eq!(workgroup_size, [8, 4, 2]);
}

#[test]
fn compute_pipeline_reflection_rejects_non_compute_entry_point() {
    let error = compute_entry_point_workgroup_size(
            "reflect-workgroup-size",
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }",
            "vs_main",
        )
        .expect_err("vertex entry point must not be accepted for compute dispatch");

    assert!(error.contains("does not define a compute entry point `vs_main`"));
}

#[test]
fn compute_pipeline_rejects_workgroups_outside_device_limits() {
    let limits = wgpu::Limits {
        max_compute_invocations_per_workgroup: 64,
        max_compute_workgroup_size_x: 8,
        max_compute_workgroup_size_y: 8,
        max_compute_workgroup_size_z: 2,
        ..wgpu::Limits::default()
    };

    let dimension_error =
        validate_compute_workgroup_limits("limited-workgroup", [9, 1, 1], &limits)
            .expect_err("workgroup axis exceeding the device limit must fail");
    assert!(dimension_error.contains("exceeds device dimension limits"));

    let zero_dimension_error =
        validate_compute_workgroup_limits("limited-workgroup", [0, 1, 1], &limits)
            .expect_err("zero-sized workgroup dimensions must fail");
    assert!(zero_dimension_error.contains("must have positive dimensions"));

    let invocation_error =
        validate_compute_workgroup_limits("limited-workgroup", [8, 8, 2], &limits)
            .expect_err("workgroup invocation count exceeding the device limit must fail");
    assert!(invocation_error.contains("has 128 invocations"));
}

#[test]
fn compute_pipeline_rejects_workgroup_size_mismatches() {
    let error =
        validate_expected_workgroup_size("mismatched-workgroup", "cs_main", [8, 8, 1], [16, 8, 1])
            .expect_err("graph and WGSL workgroup sizes must match");

    assert!(error.contains("declares workgroup size [8, 8, 1]"));
    assert!(error.contains("declares [16, 8, 1]"));
}

#[test]
fn compute_pipeline_cache_reuses_same_source_entry() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute-cache-empty-scene-layout"),
                entries: &[],
            });
    let mut cache = ComputePipelineCache::default();
    let source = "@compute @workgroup_size(1) fn cs_main() {}";
    let (first_pipeline, _) = cache
        .get_or_create(
            &backend.device,
            &scene_bind_group_layout,
            "compute-cache-reuse",
            source,
            "cs_main",
            [1, 1, 1],
            &[],
        )
        .expect("first pipeline creation should succeed");
    assert_eq!(cache.entry_count(), 1);

    let (second_pipeline, _) = cache
        .get_or_create(
            &backend.device,
            &scene_bind_group_layout,
            "compute-cache-reuse",
            source,
            "cs_main",
            [1, 1, 1],
            &[],
        )
        .expect("cache hit should succeed");

    assert_eq!(cache.entry_count(), 1);
    assert_eq!(first_pipeline, second_pipeline);

    let mismatch = cache.get_or_create(
        &backend.device,
        &scene_bind_group_layout,
        "compute-cache-reuse",
        source,
        "cs_main",
        [8, 1, 1],
        &[],
    );
    let error = match mismatch {
        Ok(_) => panic!("workgroup-size mismatch must be rejected before dispatch recording"),
        Err(error) => error,
    };
    assert!(error.contains("declares workgroup size [1, 1, 1]"));
    assert!(error.contains("declares [8, 1, 1]"));
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn compute_pipeline_cache_does_not_memoize_workload_mismatches() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute-cache-mismatch-empty-scene-layout"),
                entries: &[],
            });
    let mut cache = ComputePipelineCache::default();
    let source = "@compute @workgroup_size(1) fn cs_main() {}";

    let mismatch = match cache.get_or_create(
        &backend.device,
        &scene_bind_group_layout,
        "compute-cache-mismatch",
        source,
        "cs_main",
        [8, 1, 1],
        &[],
    ) {
        Ok(_) => panic!("mismatched workload must fail before pipeline creation"),
        Err(error) => error,
    };
    assert!(mismatch.contains("declares [8, 1, 1]"));
    assert_eq!(cache.entry_count(), 0);

    cache
        .get_or_create(
            &backend.device,
            &scene_bind_group_layout,
            "compute-cache-mismatch",
            source,
            "cs_main",
            [1, 1, 1],
            &[],
        )
        .expect("valid workload must still create the pipeline after a mismatch");
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn compute_pipeline_cache_rebuilds_after_scene_layout_changes() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let first_scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute-cache-first-scene-layout"),
                entries: &[],
            });
    let second_scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute-cache-second-scene-layout"),
                entries: &[],
            });
    let mut cache = ComputePipelineCache::default();
    let source = "@compute @workgroup_size(1) fn cs_main() {}";

    cache
        .get_or_create(
            &backend.device,
            &first_scene_bind_group_layout,
            "compute-cache-scene-layout",
            source,
            "cs_main",
            [1, 1, 1],
            &[],
        )
        .expect("first scene layout should create a pipeline");
    assert_eq!(cache.entry_count(), 1);

    cache
        .get_or_create(
            &backend.device,
            &second_scene_bind_group_layout,
            "compute-cache-scene-layout",
            source,
            "cs_main",
            [1, 1, 1],
            &[],
        )
        .expect("changed scene layout should rebuild the cached pipeline");
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn compute_pipeline_cache_memoizes_invalid_wgsl() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute-cache-invalid-empty-scene-layout"),
                entries: &[],
            });
    let mut cache = ComputePipelineCache::default();
    let source = "@compute @workgroup_size(1) fn cs_main() { let invalid =; }";

    let first_error = match cache.get_or_create(
        &backend.device,
        &scene_bind_group_layout,
        "compute-cache-invalid",
        source,
        "cs_main",
        [1, 1, 1],
        &[],
    ) {
        Ok(_) => panic!("invalid WGSL must fail before creating a pipeline"),
        Err(error) => error,
    };
    assert_eq!(cache.entry_count(), 1);

    let second_error = match cache.get_or_create(
        &backend.device,
        &scene_bind_group_layout,
        "compute-cache-invalid",
        source,
        "cs_main",
        [1, 1, 1],
        &[],
    ) {
        Ok(_) => panic!("cached invalid WGSL must fail without retrying reflection"),
        Err(error) => error,
    };

    assert_eq!(second_error, first_error);
    assert_eq!(cache.entry_count(), 1);
}
