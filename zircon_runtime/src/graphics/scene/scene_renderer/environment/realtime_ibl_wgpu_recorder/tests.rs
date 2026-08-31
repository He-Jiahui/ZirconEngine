use std::time::Instant;

use super::*;
use crate::core::framework::render::{
    SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::core::runtime::diagnostics::profiling;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::ibl_bake_pmrem_dispatch_groups_for_face_range;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::{
    append_realtime_ibl_graph_plan, RealtimeIblGraphPassKind,
};
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_profile_test_support::start_manual_cpu_profile_capture;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::{
    RealtimeIblCompletion, RealtimeIblOperation, RealtimeIblPrefilterDispatchSlice,
    RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
};
use crate::render_graph::RenderGraphBuilder;

#[test]
fn invalid_pmrem_slice_reports_its_exact_bounds() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let slice = RealtimeIblPrefilterDispatchSlice {
        mip_level: request.pmrem_mip_count() as u8,
        first_face: 5,
        face_count: 2,
    };

    let error = prefilter_command(&request, slice).expect_err("slice must be rejected");

    assert!(error.contains("mip=8"), "{error}");
    assert!(error.contains("first_face=5"), "{error}");
    assert!(error.contains("face_count=2"), "{error}");
}

#[test]
fn realtime_ibl_dispatch_parity_accepts_the_canonical_sh9_extent() {
    validate_graph_command_dispatch_groups(
        RealtimeIblWgpuBindingCommandKey::ProjectDiffuseSh9,
        [1, 1, 1],
        [1, 1, 1],
    )
    .expect("canonical SH9 graph and command dispatches must agree");
}

#[test]
fn realtime_ibl_dispatch_parity_rejects_the_legacy_graph_extent() {
    let error = validate_graph_command_dispatch_groups(
        RealtimeIblWgpuBindingCommandKey::ProjectDiffuseSh9,
        [4, 4, 6],
        [1, 1, 1],
    )
    .expect_err("a drifted graph dispatch must fail before WGPU encoding");

    assert_eq!(
        error,
        "realtime IBL graph/command dispatch mismatch for ProjectDiffuseSh9: graph=[4, 4, 6], command=[1, 1, 1]"
    );
}

#[test]
fn realtime_ibl_binding_cache_validates_dispatch_parity_before_gpu_work() {
    let source = include_str!("../realtime_ibl_wgpu_recorder.rs");
    let record_start = source
        .find("fn record(")
        .expect("realtime IBL binding-cache record function");
    let record = &source[record_start..];

    let hit_validation = record
        .find("validate_graph_command_dispatch_groups(\n                key,\n                graph_dispatch_groups,\n                entry.command.dispatch_groups,\n            )?;")
        .expect("cache-hit dispatch parity validation");
    let hit_encode = record
        .find("encode_ibl_bake_wgpu_compute_dispatch(\n                encoder,\n                &entry.command,")
        .expect("cache-hit WGPU encode");
    assert!(hit_validation < hit_encode);

    let command = record
        .find("let command = create_command()?;")
        .expect("cache-miss command creation");
    let miss_record = &record[command..];
    let miss_validation = miss_record
        .find("validate_graph_command_dispatch_groups(\n            key,\n            graph_dispatch_groups,\n            command.dispatch_groups,\n        )?;")
        .expect("cache-miss dispatch parity validation");
    let pipeline = miss_record
        .find("let pipeline = pipeline_cache.ensure_compute_pipeline(device, &command);")
        .expect("cache-miss pipeline creation");
    let params = miss_record
        .find("let params = create_ibl_bake_wgpu_params_buffer(device, &command);")
        .expect("cache-miss parameter-buffer creation");
    assert!(miss_validation < pipeline);
    assert!(miss_validation < params);
}

#[test]
fn realtime_ibl_timestamp_encoding_requires_explicit_gpu_timing() {
    let source = include_str!("../realtime_ibl_wgpu_recorder.rs");
    let normalized = source.split_whitespace().collect::<String>();
    let start = source
        .find("pub(in crate::graphics) fn record_graph_plan")
        .expect("realtime IBL recorder entrypoint");
    let record_graph_plan = &source[start..];

    assert!(record_graph_plan.contains("gpu_timing_enabled: bool"));
    assert!(record_graph_plan.contains("Self::timestamp_recorder("));
    assert!(record_graph_plan.contains("timestamp_recorder.map("));
    assert!(source.contains("timestamps: None"));
    assert!(normalized.contains("gpu_timing_enabled&&timestamps.is_none()"));
}

#[test]
fn realtime_ibl_stage_binding_profile_emits_adapter_identity() {
    let source = include_str!("tests.rs");
    let profile_function = [
        "fn profile_realtime_ibl_capture_and_source_mip_binding_encoding",
        "()",
    ]
    .concat();
    let profile = source
        .split(profile_function.as_str())
        .nth(1)
        .and_then(|body| body.split("#[test]").next())
        .expect("realtime IBL stage-binding profile source");

    assert!(profile.contains("let RenderBackend {"));
    assert!(profile.contains("adapter, device, .."));
    assert!(profile.contains("let adapter_info = adapter.get_info();"));
    for field in [
        "adapter_name={}",
        "adapter_backend={}",
        "adapter_vendor_id={}",
        "adapter_device_id={}",
        "adapter_type={:?}",
    ] {
        assert!(
            profile.contains(field),
            "realtime IBL stage-binding profile must emit `{field}`"
        );
    }
}

#[test]
fn realtime_ibl_stage_binding_profile_exports_to_an_explicit_non_c_root() {
    let source = include_str!("tests.rs");
    let profile_function = [
        "fn profile_realtime_ibl_capture_and_source_mip_binding_encoding",
        "()",
    ]
    .concat();
    let profile = source
        .split(profile_function.as_str())
        .nth(1)
        .and_then(|body| body.split("#[test]").next())
        .expect("realtime IBL stage-binding profile source");

    let support = include_str!("../realtime_ibl_profile_test_support.rs");
    assert!(profile.contains("profile_capture.finish_and_export();"));
    assert!(profile.contains("ProfileFrameScope::enter"));
    assert!(profile.contains("ProfileScope::enter"));
    assert!(support.contains("ZIRCON_PROFILE_OUTPUT_ROOT"));
    assert!(support.contains("absolute non-C"));
}

#[test]
fn realtime_ibl_binding_cache_profile_accumulates_separate_creation_buckets() {
    let mut total = RealtimeIblWgpuBindingCacheStats {
        command_plan_creation_micros: 3,
        pipeline_ensure_micros: 5,
        binding_creation_micros: 7,
        ..Default::default()
    };

    total.record(RealtimeIblWgpuBindingCacheStats {
        command_plan_creation_micros: 11,
        pipeline_ensure_micros: 13,
        binding_creation_micros: 17,
        ..Default::default()
    });

    assert_eq!(total.command_plan_creation_micros, 14);
    assert_eq!(total.pipeline_ensure_micros, 18);
    assert_eq!(total.binding_creation_micros, 24);
}

#[test]
fn realtime_ibl_binding_cache_profile_times_resource_creation_after_command_and_pipeline() {
    let source = include_str!("../realtime_ibl_wgpu_recorder.rs");
    let cache_record_start = source
        .find("fn record(")
        .expect("realtime IBL binding-cache record function");
    let cache_record = &source[cache_record_start..];

    let command_started = cache_record
        .find("let command_plan_started = cpu_timing_enabled.then(Instant::now);")
        .expect("command-plan timing boundary");
    let command = cache_record
        .find("let command = create_command()?;")
        .expect("command-plan creation");
    let command_elapsed = cache_record
        .find("let command_plan_creation_micros = elapsed_micros(command_plan_started);")
        .expect("command-plan elapsed metric");
    let pipeline_started = cache_record
        .find("let pipeline_ensure_started = cpu_timing_enabled.then(Instant::now);")
        .expect("pipeline timing boundary");
    let pipeline = cache_record
        .find("let pipeline = pipeline_cache.ensure_compute_pipeline(device, &command);")
        .expect("pipeline cache ensure");
    let pipeline_elapsed = cache_record
        .find("let pipeline_ensure_micros = elapsed_micros(pipeline_ensure_started);")
        .expect("pipeline elapsed metric");
    let binding_started = cache_record
        .find("let binding_creation_started = cpu_timing_enabled.then(Instant::now);")
        .expect("resource-binding timing boundary");
    let params = cache_record
        .find("let params = create_ibl_bake_wgpu_params_buffer(device, &command);")
        .expect("parameter-buffer creation");
    let bind_group = cache_record
        .find("let bind_group = create_ibl_bake_wgpu_bind_group(")
        .expect("bind-group creation");
    let binding_elapsed = cache_record
        .find("let binding_creation_micros = elapsed_micros(binding_creation_started);")
        .expect("resource-binding elapsed metric");

    assert!(command_started < command);
    assert!(command < command_elapsed);
    assert!(command_elapsed < pipeline_started);
    assert!(pipeline_started < pipeline);
    assert!(pipeline < pipeline_elapsed);
    assert!(pipeline_elapsed < binding_started);
    assert!(binding_started < params);
    assert!(params < bind_group);
    assert!(bind_group < binding_elapsed);
}

#[test]
fn realtime_ibl_binding_cache_does_not_count_an_empty_layout_change_as_a_reset() {
    let sky = ProceduralSkyParams::default_gradient();
    let initial_request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let changed_request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 32, 6);
    let mut cache = RealtimeIblWgpuBindingCache::default();

    assert!(!cache.prepare_for_request(&initial_request));
    assert!(cache.entries.is_empty());
    assert!(
        !cache.prepare_for_request(&changed_request),
        "a layout change cannot reset a cache with no binding entries"
    );
    assert!(cache.entries.is_empty());
}

#[test]
fn realtime_sh9_graph_workload_matches_the_encoded_sh9_command() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());

    let batch = (1..64)
        .find_map(|frame| {
            let batch = scheduler.begin_frame(frame).expect("ticket batch");
            if matches!(
                batch.operations(),
                [RealtimeIblOperation::ProjectDiffuseSh9]
            ) {
                return Some(batch);
            }

            assert_eq!(
                scheduler.complete_frame(batch.token(), true),
                RealtimeIblCompletion::Advanced
            );
            None
        })
        .expect("terminal SH9 ticket");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-sh9-command-parity");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");

    assert_eq!(plan.passes.len(), 1);
    assert_eq!(graph.passes().len(), 1);
    let graph_dispatch = fixed_dispatch_groups(&plan.passes[0].workload.dispatch_extent)
        .expect("terminal SH9 graph uses a fixed dispatch");
    let encoded_command = sh9_command(&request).expect("encoded SH9 command");

    assert_eq!(graph_dispatch, [1, 1, 1]);
    assert_eq!(graph_dispatch, encoded_command.dispatch_groups);
}

#[test]
fn realtime_pmrem_graph_workloads_match_encoded_commands_for_every_scheduled_slice() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());

    let mut checked_slices = 0;
    let mut terminal_average_slices = 0;
    let mut published = false;
    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("ticket batch");
        if !batch.prefilter_dispatch_slices().is_empty() {
            let mut builder = RenderGraphBuilder::new("realtime-ibl-pmrem-command-parity");
            let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
                .expect("realtime IBL graph plan");
            let graph = builder.compile().expect("realtime graph compiles");

            assert_eq!(graph.passes().len(), plan.passes.len());
            for pass in &plan.passes {
                let RealtimeIblGraphPassKind::Prefilter(slice) = pass.kind else {
                    continue;
                };
                let graph_dispatch = fixed_dispatch_groups(&pass.workload.dispatch_extent)
                    .expect("PMREM graph uses a fixed dispatch");
                let encoded_command =
                    prefilter_command(&request, slice).expect("encoded PMREM command");
                let shared_dispatch = ibl_bake_pmrem_dispatch_groups_for_face_range(
                    request.pmrem_face_size(),
                    request.pmrem_mip_count(),
                    u32::from(slice.mip_level),
                    u32::from(slice.first_face),
                    u32::from(slice.face_count),
                )
                .expect("scheduled PMREM slice must be valid");

                assert_eq!(
                    graph_dispatch, shared_dispatch,
                    "PMREM dispatch mismatch for {slice:?}"
                );
                assert_eq!(shared_dispatch, encoded_command.dispatch_groups);
                checked_slices += 1;
                terminal_average_slices +=
                    usize::from(encoded_command.dispatch_groups == [1, 1, 1]);
            }
        }

        match scheduler.complete_frame(batch.token(), true) {
            RealtimeIblCompletion::Advanced => {}
            RealtimeIblCompletion::Published => {
                published = true;
                break;
            }
            other => panic!("unexpected realtime IBL completion: {other:?}"),
        }
    }

    assert!(published, "the scheduled ticket must reach publication");
    assert_eq!(checked_slices, 7);
    assert_eq!(terminal_average_slices, 1);
}

#[test]
#[ignore = "time-sliced realtime IBL WGPU product validation"]
fn capture_ticket_records_and_submits_without_wgpu_validation_errors() {
    let Ok(RenderBackend { device, queue, .. }) = RenderBackend::new_offscreen() else {
        return;
    };
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let batch = scheduler.begin_frame(1).expect("initial batch");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-wgpu-product");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
        .expect("realtime graph plan");
    let resources = RealtimeIblGpuResources::new(&device, &request);
    let mut recorder = RealtimeIblWgpuRecorder::new(&device);
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-realtime-ibl-wgpu-product"),
    });
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    let result = recorder
        .record_graph_plan(
            &device,
            &mut encoder,
            true,
            false,
            &request,
            &sky,
            &plan,
            &plan.passes,
            &resources,
            &mut pipeline_cache,
        )
        .expect("realtime IBL WGPU recording");
    queue.submit([encoder.finish()]);

    let validation_error = pollster::block_on(error_scope.pop());
    assert!(validation_error.is_none(), "{validation_error:?}");
    assert_eq!(result.report.pass_count, plan.passes.len());
    assert_eq!(result.report.dispatch_count, plan.passes.len());
    assert_eq!(result.report.dispatch_groups.last(), Some(&[2, 2, 2]));
    assert_eq!(
        result.timestamp_readback.is_some(),
        device.features().contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
        )
    );
}

#[test]
#[ignore = "manual CPU profile for realtime IBL PMREM binding encoding"]
fn profile_realtime_ibl_pmrem_binding_encoding() {
    const PROFILE_ITERATIONS: usize = 512;

    let Ok(RenderBackend { device, .. }) = RenderBackend::new_offscreen() else {
        return;
    };
    let profile_capture = start_manual_cpu_profile_capture("realtime-ibl-pmrem-bindings");
    let profile_frame =
        profiling::ProfileFrameScope::enter("realtime_ibl_cpu_profile", "pmrem_binding_encoding");
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let batch = (1..64)
        .find_map(|frame| {
            let batch = scheduler.begin_frame(frame).expect("ticket batch");
            if batch
                .operations()
                .iter()
                .any(|operation| matches!(operation, RealtimeIblOperation::Prefilter { .. }))
            {
                Some(batch)
            } else {
                assert_eq!(
                    scheduler.complete_frame(batch.token(), true),
                    RealtimeIblCompletion::Advanced
                );
                None
            }
        })
        .expect("default realtime ticket contains PMREM work");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-pmrem-binding-profile");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
        .expect("realtime graph plan");
    let recording_passes = plan
        .passes
        .iter()
        .filter(|pass| matches!(pass.kind, RealtimeIblGraphPassKind::Prefilter(_)))
        .cloned()
        .collect::<Vec<_>>();
    assert!(!recording_passes.is_empty());

    let resources = RealtimeIblGpuResources::new(&device, &request);
    let mut recorder = RealtimeIblWgpuRecorder::new(&device);
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);
    let started = Instant::now();
    let mut encoded_dispatches = 0;
    let mut binding_cache_hits = 0;
    let mut binding_cache_misses = 0;
    let mut params_buffer_creations = 0;
    let mut bind_group_creations = 0;
    let mut command_plan_creation_micros = 0;
    let mut pipeline_ensure_micros = 0;
    let mut binding_creation_micros = 0;
    for _ in 0..PROFILE_ITERATIONS {
        let _profile_scope =
            profiling::ProfileScope::enter("realtime_ibl_cpu_profile", "recording", "pmrem_warm");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-realtime-ibl-pmrem-binding-profile"),
        });
        let result = recorder
            .record_graph_plan(
                &device,
                &mut encoder,
                false,
                true,
                &request,
                &sky,
                &plan,
                &recording_passes,
                &resources,
                &mut pipeline_cache,
            )
            .expect("realtime IBL PMREM profile recording");
        encoded_dispatches += result.report.dispatch_count;
        binding_cache_hits += result.report.binding_cache_hits;
        binding_cache_misses += result.report.binding_cache_misses;
        params_buffer_creations += result.report.params_buffer_creations;
        bind_group_creations += result.report.bind_group_creations;
        command_plan_creation_micros += result.report.command_plan_creation_micros;
        pipeline_ensure_micros += result.report.pipeline_ensure_micros;
        binding_creation_micros += result.report.binding_creation_micros;
        drop(encoder.finish());
    }

    let elapsed = started.elapsed();
    eprintln!(
        "realtime_ibl_pmrem_binding_profile iterations={PROFILE_ITERATIONS} passes_per_iteration={} encoded_dispatches={encoded_dispatches} binding_cache_hits={binding_cache_hits} binding_cache_misses={binding_cache_misses} params_buffer_creations={params_buffer_creations} bind_group_creations={bind_group_creations} command_plan_creation_micros={command_plan_creation_micros} pipeline_ensure_micros={pipeline_ensure_micros} binding_creation_micros={binding_creation_micros} elapsed_ms={:.3}",
        recording_passes.len(),
        elapsed.as_secs_f64() * 1000.0,
    );
    assert_eq!(
        encoded_dispatches,
        PROFILE_ITERATIONS * recording_passes.len()
    );
    assert_eq!(
        binding_cache_hits,
        (PROFILE_ITERATIONS - 1) * recording_passes.len()
    );
    assert_eq!(binding_cache_misses, recording_passes.len());
    assert_eq!(params_buffer_creations, recording_passes.len());
    assert_eq!(bind_group_creations, recording_passes.len());
    drop(profile_frame);
    profile_capture.finish_and_export();
}

#[test]
#[ignore = "manual CPU profile for realtime IBL capture and source-mip binding encoding"]
fn profile_realtime_ibl_capture_and_source_mip_binding_encoding() {
    const PROFILE_ITERATIONS: usize = 256;

    let RenderBackend {
        adapter, device, ..
    } = RenderBackend::new_offscreen()
        .expect("realtime IBL stage-binding profile requires an offscreen WGPU backend");
    let profile_capture = start_manual_cpu_profile_capture("realtime-ibl-dynamic-bindings");
    let profile_frame = profiling::ProfileFrameScope::enter(
        "realtime_ibl_cpu_profile",
        "capture_source_mip_binding_encoding",
    );
    let adapter_info = adapter.get_info();
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(
        sky.ibl_bake_key(),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(SOURCE_CUBEMAP_PMREM_MIP_COUNT as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let mut recording_batches = Vec::new();
    let mut published = false;
    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("ticket batch");
        let mut builder = RenderGraphBuilder::new("realtime-ibl-stage-binding-profile");
        let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
            .expect("realtime IBL graph plan");
        recording_batches.push((plan.clone(), plan.passes));

        match scheduler.complete_frame(batch.token(), true) {
            RealtimeIblCompletion::Advanced => {}
            RealtimeIblCompletion::Published => {
                published = true;
                break;
            }
            completion => panic!("expected a successful realtime IBL ticket, got {completion:?}"),
        }
    }
    assert!(published, "default realtime ticket must reach publication");
    assert_eq!(
        recording_batches
            .iter()
            .map(|(_, passes)| passes.len())
            .sum::<usize>(),
        21
    );

    let resources = RealtimeIblGpuResources::new(&device, &request);
    let mut recorder = RealtimeIblWgpuRecorder::new(&device);
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);
    let cold_started = Instant::now();
    let cold = {
        let _profile_scope =
            profiling::ProfileScope::enter("realtime_ibl_cpu_profile", "recording", "cold_ticket");
        record_realtime_ibl_ticket(
            &device,
            &mut recorder,
            &request,
            &sky,
            &recording_batches,
            &resources,
            &mut pipeline_cache,
        )
    };
    let cold_elapsed = cold_started.elapsed();
    let warm_started = Instant::now();
    let mut warm = empty_recording_total();
    for _ in 1..PROFILE_ITERATIONS {
        let _profile_scope =
            profiling::ProfileScope::enter("realtime_ibl_cpu_profile", "recording", "warm_ticket");
        accumulate_recording_total(
            &mut warm,
            record_realtime_ibl_ticket(
                &device,
                &mut recorder,
                &request,
                &sky,
                &recording_batches,
                &resources,
                &mut pipeline_cache,
            ),
        );
    }
    let warm_elapsed = warm_started.elapsed();

    eprintln!(
        "realtime_ibl_stage_binding_profile adapter_name={} adapter_backend={} adapter_vendor_id={} adapter_device_id={} adapter_type={:?} iterations={PROFILE_ITERATIONS} cold_ticket_elapsed_ms={:.3} cold_capture_params_buffer_creations={} cold_capture_bind_group_creations={} cold_capture_binding_creation_micros={} cold_source_mip_params_buffer_creations={} cold_source_mip_bind_group_creations={} cold_source_mip_binding_creation_micros={} cold_pmrem_sh9_binding_cache_hits={} cold_pmrem_sh9_binding_cache_misses={} cold_pmrem_sh9_params_buffer_creations={} cold_pmrem_sh9_bind_group_creations={} cold_pmrem_sh9_command_plan_creation_micros={} cold_pmrem_sh9_pipeline_ensure_micros={} cold_pmrem_sh9_binding_creation_micros={} warm_ticket_average_elapsed_ms={:.3} warm_capture_params_buffer_creations={} warm_capture_bind_group_creations={} warm_capture_binding_creation_micros={} warm_source_mip_params_buffer_creations={} warm_source_mip_bind_group_creations={} warm_source_mip_binding_creation_micros={} warm_pmrem_sh9_binding_cache_hits={} warm_pmrem_sh9_binding_cache_misses={} warm_pmrem_sh9_params_buffer_creations={} warm_pmrem_sh9_bind_group_creations={} warm_pmrem_sh9_command_plan_creation_micros={} warm_pmrem_sh9_pipeline_ensure_micros={} warm_pmrem_sh9_binding_creation_micros={}",
        adapter_info.name,
        adapter_info.backend.to_str(),
        adapter_info.vendor,
        adapter_info.device,
        adapter_info.device_type,
        cold_elapsed.as_secs_f64() * 1000.0,
        cold.capture_params_buffer_creations,
        cold.capture_bind_group_creations,
        cold.capture_binding_creation_micros,
        cold.source_mip_params_buffer_creations,
        cold.source_mip_bind_group_creations,
        cold.source_mip_binding_creation_micros,
        cold.binding_cache_hits,
        cold.binding_cache_misses,
        cold.params_buffer_creations,
        cold.bind_group_creations,
        cold.command_plan_creation_micros,
        cold.pipeline_ensure_micros,
        cold.binding_creation_micros,
        warm_elapsed.as_secs_f64() * 1000.0 / (PROFILE_ITERATIONS - 1) as f64,
        warm.capture_params_buffer_creations,
        warm.capture_bind_group_creations,
        warm.capture_binding_creation_micros,
        warm.source_mip_params_buffer_creations,
        warm.source_mip_bind_group_creations,
        warm.source_mip_binding_creation_micros,
        warm.binding_cache_hits,
        warm.binding_cache_misses,
        warm.params_buffer_creations,
        warm.bind_group_creations,
        warm.command_plan_creation_micros,
        warm.pipeline_ensure_micros,
        warm.binding_creation_micros,
    );
    assert_eq!(cold.dispatch_count, 21);
    assert_eq!(cold.capture_params_buffer_creations, 3);
    assert_eq!(cold.capture_bind_group_creations, 3);
    assert_eq!(cold.source_mip_params_buffer_creations, 7);
    assert_eq!(cold.source_mip_bind_group_creations, 7);
    assert_eq!(cold.binding_cache_misses, 11);
    assert_eq!(cold.params_buffer_creations, 11);
    assert_eq!(cold.bind_group_creations, 11);
    assert_eq!(cold.binding_cache_hits, 0);
    assert_eq!(warm.dispatch_count, (PROFILE_ITERATIONS - 1) * 21);
    assert_eq!(
        warm.capture_params_buffer_creations,
        (PROFILE_ITERATIONS - 1) * 3
    );
    assert_eq!(
        warm.capture_bind_group_creations,
        (PROFILE_ITERATIONS - 1) * 3
    );
    assert_eq!(
        warm.source_mip_params_buffer_creations,
        (PROFILE_ITERATIONS - 1) * 7
    );
    assert_eq!(
        warm.source_mip_bind_group_creations,
        (PROFILE_ITERATIONS - 1) * 7
    );
    assert_eq!(warm.binding_cache_hits, (PROFILE_ITERATIONS - 1) * 11);
    assert_eq!(warm.binding_cache_misses, 0);
    assert_eq!(warm.params_buffer_creations, 0);
    assert_eq!(warm.bind_group_creations, 0);
    drop(profile_frame);
    profile_capture.finish_and_export();
}

#[test]
fn realtime_ibl_recorder_reuses_pmrem_and_sh9_binding_templates_for_a_stable_work_slot() {
    let RenderBackend { device, .. } = RenderBackend::new_offscreen()
        .expect("stable-slot binding-cache regression requires an offscreen WGPU backend");
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(
        sky.ibl_bake_key(),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(SOURCE_CUBEMAP_PMREM_MIP_COUNT as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let mut recording_batches = Vec::new();
    let mut published = false;
    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("ticket batch");
        let mut builder = RenderGraphBuilder::new("realtime-ibl-binding-template-cache");
        let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
            .expect("realtime graph plan");
        recording_batches.push((plan.clone(), plan.passes));

        match scheduler.complete_frame(batch.token(), true) {
            RealtimeIblCompletion::Advanced => {}
            RealtimeIblCompletion::Published => {
                published = true;
                break;
            }
            completion => panic!("expected a successful realtime IBL ticket, got {completion:?}"),
        }
    }
    assert!(
        published,
        "default realtime ticket must reach its terminal SH9 pass"
    );
    let cacheable_pass_count = recording_batches
        .iter()
        .flat_map(|(_, passes)| passes)
        .filter(|pass| {
            matches!(
                pass.kind,
                RealtimeIblGraphPassKind::Prefilter(_)
                    | RealtimeIblGraphPassKind::ProjectDiffuseSh9
            )
        })
        .count();
    let capture_pass_count = recording_batches
        .iter()
        .flat_map(|(_, passes)| passes)
        .filter(|pass| matches!(pass.kind, RealtimeIblGraphPassKind::CaptureSky(_)))
        .count();
    let source_mip_pass_count = recording_batches
        .iter()
        .flat_map(|(_, passes)| passes)
        .filter(|pass| {
            matches!(
                pass.kind,
                RealtimeIblGraphPassKind::GenerateSourceMip { .. }
            )
        })
        .count();
    let recorded_pass_count = recording_batches
        .iter()
        .map(|(_, passes)| passes.len())
        .sum::<usize>();
    let prefilter_pass_count = recording_batches
        .iter()
        .flat_map(|(_, passes)| passes)
        .filter(|pass| matches!(pass.kind, RealtimeIblGraphPassKind::Prefilter(_)))
        .count();
    assert!(
        prefilter_pass_count > 0,
        "default realtime ticket contains PMREM work"
    );
    assert_eq!(prefilter_pass_count, 10);
    assert_eq!(cacheable_pass_count, 11);
    assert_eq!(capture_pass_count, 3);
    assert_eq!(source_mip_pass_count, 7);
    assert_eq!(recorded_pass_count, 21);

    let resources = RealtimeIblGpuResources::new(&device, &request);
    let mut recorder = RealtimeIblWgpuRecorder::new(&device);
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);

    let first = recording_totals(recording_batches.iter().map(|(plan, recording_passes)| {
        record_realtime_ibl_passes(
            &device,
            &mut recorder,
            &request,
            &sky,
            plan,
            recording_passes,
            &resources,
            &mut pipeline_cache,
        )
    }));
    assert_eq!(first.binding_cache_hits, 0);
    assert_eq!(first.binding_cache_misses, cacheable_pass_count);
    assert_eq!(first.params_buffer_creations, cacheable_pass_count);
    assert_eq!(first.bind_group_creations, cacheable_pass_count);
    assert_eq!(first.binding_cache_resets, 0);
    assert_eq!(first.capture_params_buffer_creations, capture_pass_count);
    assert_eq!(first.capture_bind_group_creations, capture_pass_count);
    assert_eq!(
        first.source_mip_params_buffer_creations,
        source_mip_pass_count
    );
    assert_eq!(first.source_mip_bind_group_creations, source_mip_pass_count);

    let second = recording_totals(recording_batches.iter().map(|(plan, recording_passes)| {
        record_realtime_ibl_passes(
            &device,
            &mut recorder,
            &request,
            &sky,
            plan,
            recording_passes,
            &resources,
            &mut pipeline_cache,
        )
    }));
    assert_eq!(second.binding_cache_hits, cacheable_pass_count);
    assert_eq!(second.binding_cache_misses, 0);
    assert_eq!(second.params_buffer_creations, 0);
    assert_eq!(second.bind_group_creations, 0);
    assert_eq!(second.binding_cache_resets, 0);
    assert_eq!(second.capture_params_buffer_creations, capture_pass_count);
    assert_eq!(second.capture_bind_group_creations, capture_pass_count);
    assert_eq!(
        second.source_mip_params_buffer_creations,
        source_mip_pass_count
    );
    assert_eq!(
        second.source_mip_bind_group_creations,
        source_mip_pass_count
    );
}

fn recording_totals(
    reports: impl IntoIterator<Item = RealtimeIblWgpuRecordReport>,
) -> RealtimeIblWgpuRecordReport {
    let mut total = empty_recording_total();
    for report in reports {
        accumulate_recording_total(&mut total, report);
    }
    total
}

#[allow(clippy::too_many_arguments)]
fn record_realtime_ibl_ticket(
    device: &wgpu::Device,
    recorder: &mut RealtimeIblWgpuRecorder,
    request: &IblBakeArtifactRequest,
    sky: &ProceduralSkyParams,
    recording_batches: &[(
        super::realtime_ibl_graph_plan::RealtimeIblGraphPlan,
        Vec<super::realtime_ibl_graph_plan::RealtimeIblGraphPass>,
    )],
    resources: &RealtimeIblGpuResources,
    pipeline_cache: &mut IblBakeWgpuPipelineCache,
) -> RealtimeIblWgpuRecordReport {
    recording_totals(recording_batches.iter().map(|(plan, recording_passes)| {
        record_realtime_ibl_passes(
            device,
            recorder,
            request,
            sky,
            plan,
            recording_passes,
            resources,
            pipeline_cache,
        )
    }))
}

fn empty_recording_total() -> RealtimeIblWgpuRecordReport {
    RealtimeIblWgpuRecordReport {
        pass_count: 0,
        dispatch_count: 0,
        dispatch_groups: Vec::new(),
        binding_cache_hits: 0,
        binding_cache_misses: 0,
        params_buffer_creations: 0,
        bind_group_creations: 0,
        binding_cache_resets: 0,
        command_plan_creation_micros: 0,
        pipeline_ensure_micros: 0,
        binding_creation_micros: 0,
        capture_params_buffer_creations: 0,
        capture_bind_group_creations: 0,
        capture_binding_creation_micros: 0,
        source_mip_params_buffer_creations: 0,
        source_mip_bind_group_creations: 0,
        source_mip_binding_creation_micros: 0,
    }
}

fn accumulate_recording_total(
    total: &mut RealtimeIblWgpuRecordReport,
    report: RealtimeIblWgpuRecordReport,
) {
    total.pass_count += report.pass_count;
    total.dispatch_count += report.dispatch_count;
    total.dispatch_groups.extend(report.dispatch_groups);
    total.binding_cache_hits += report.binding_cache_hits;
    total.binding_cache_misses += report.binding_cache_misses;
    total.params_buffer_creations += report.params_buffer_creations;
    total.bind_group_creations += report.bind_group_creations;
    total.binding_cache_resets += report.binding_cache_resets;
    total.command_plan_creation_micros += report.command_plan_creation_micros;
    total.pipeline_ensure_micros += report.pipeline_ensure_micros;
    total.binding_creation_micros += report.binding_creation_micros;
    total.capture_params_buffer_creations += report.capture_params_buffer_creations;
    total.capture_bind_group_creations += report.capture_bind_group_creations;
    total.capture_binding_creation_micros += report.capture_binding_creation_micros;
    total.source_mip_params_buffer_creations += report.source_mip_params_buffer_creations;
    total.source_mip_bind_group_creations += report.source_mip_bind_group_creations;
    total.source_mip_binding_creation_micros += report.source_mip_binding_creation_micros;
}

#[allow(clippy::too_many_arguments)]
fn record_realtime_ibl_passes(
    device: &wgpu::Device,
    recorder: &mut RealtimeIblWgpuRecorder,
    request: &IblBakeArtifactRequest,
    sky: &ProceduralSkyParams,
    plan: &super::realtime_ibl_graph_plan::RealtimeIblGraphPlan,
    recording_passes: &[super::realtime_ibl_graph_plan::RealtimeIblGraphPass],
    resources: &RealtimeIblGpuResources,
    pipeline_cache: &mut IblBakeWgpuPipelineCache,
) -> RealtimeIblWgpuRecordReport {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-realtime-ibl-binding-template-cache"),
    });
    let result = recorder
        .record_graph_plan(
            device,
            &mut encoder,
            false,
            profiling::capture_active(),
            request,
            sky,
            plan,
            recording_passes,
            resources,
            pipeline_cache,
        )
        .expect("realtime IBL binding template recording");
    drop(encoder.finish());
    result.report
}
