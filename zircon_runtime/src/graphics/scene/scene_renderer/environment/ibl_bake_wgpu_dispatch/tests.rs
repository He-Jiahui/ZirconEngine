use std::sync::{Arc, mpsc};

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, IblBakeArtifactContents, IblBakeArtifactRequest,
    ProceduralSkyParams, RenderFrameExtract, RenderPluginRendererOutputs,
    RenderWorldSnapshotHandle, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::core::math::UVec2;
use crate::graphics::ViewportRenderFrame;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassExecutorId, TransientResourcePool,
};
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::render_graph::{QueueLane, RenderGraphBuilder};
use crate::scene::world::World;

use super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::super::ibl_bake_wgpu_binding::{
    IblBakeWgpuBindGroupLayouts, IblBakeWgpuOutputBindingResource, create_ibl_bake_wgpu_bind_group,
    create_ibl_bake_wgpu_params_buffer, create_ibl_bake_wgpu_source_sampler,
};
use super::super::ibl_bake_wgpu_command_plan::{
    IblBakeWgpuCommandPlan, IblBakeWgpuOutputPlan, ibl_bake_wgpu_command_plan_for_request,
};
use super::super::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use super::*;

#[path = "tests/irradiance_parity.rs"]
mod irradiance_parity;
#[path = "tests/reference_parity.rs"]
mod reference_parity;

#[test]
fn compute_pipeline_encodes_storage_texture_and_storage_buffer_dispatches() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let device = &backend.device;
    let queue = &backend.queue;
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let source_texture = create_source_cubemap_texture(device);
    let source_view = source_texture.create_view(&source_cubemap_view_descriptor());
    let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);

    let pmrem = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem { mip_level: 0 },
    );
    let pmrem_output = create_storage_output_texture(
        device,
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let pmrem_output_view = pmrem_output.create_view(&storage_texture_descriptor(pmrem));
    let pmrem_params = create_ibl_bake_wgpu_params_buffer(device, pmrem);
    let pmrem_bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        pmrem,
        &pmrem_params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageTexture2DArray(&pmrem_output_view),
    )
    .expect("PMREM bind group should be valid");
    let pmrem_pipeline = create_ibl_bake_wgpu_compute_pipeline(
        device,
        pmrem,
        layouts.layout(pmrem.bind_group_layout_kind),
    );

    let sh9 = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9);
    let sh9_output = create_sh9_output_buffer(device);
    let sh9_params = create_ibl_bake_wgpu_params_buffer(device, sh9);
    let sh9_bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        sh9,
        &sh9_params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageBuffer(&sh9_output),
    )
    .expect("SH9 bind group should be valid");
    let sh9_pipeline = create_ibl_bake_wgpu_compute_pipeline(
        device,
        sh9,
        layouts.layout(sh9.bind_group_layout_kind),
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-dispatch-test-encoder"),
    });
    let pmrem_record = encode_ibl_bake_wgpu_compute_dispatch(
        &mut encoder,
        pmrem,
        &pmrem_pipeline,
        &pmrem_bind_group,
    )
    .expect("PMREM dispatch should encode");
    let sh9_record =
        encode_ibl_bake_wgpu_compute_dispatch(&mut encoder, sh9, &sh9_pipeline, &sh9_bind_group)
            .expect("SH9 dispatch should encode");

    let pmrem_base_dispatch = [
        SOURCE_CUBEMAP_PMREM_FACE_SIZE.div_ceil(IBL_BAKE_WORKGROUP_SIZE[0]),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE.div_ceil(IBL_BAKE_WORKGROUP_SIZE[1]),
        6,
    ];
    assert_eq!(pmrem_record.dispatch_groups, pmrem_base_dispatch);
    assert_eq!(sh9_record.dispatch_groups, [1, 1, 1]);
    queue.submit(std::iter::once(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("IBL bake test dispatches should finish");
}

#[test]
fn final_pmrem_mip_writes_common_six_face_average() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let device = &backend.device;
    let queue = &backend.queue;
    let layouts = IblBakeWgpuBindGroupLayouts::new(device);
    let sampler = create_ibl_bake_wgpu_source_sampler(device);
    let request = request(16, 5, IblBakeArtifactContents::PMREM);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let final_mip_level = SOURCE_CUBEMAP_PMREM_MIP_COUNT - 1;
    let final_mip = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem {
            mip_level: final_mip_level,
        },
    );
    let source_texture = create_asymmetric_source_cubemap_texture(device, queue, 16, 5);
    let source_view = source_texture.create_view(&source_cubemap_mip_view_descriptor(5));
    let pmrem_output = create_storage_output_texture(
        device,
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let pmrem_output_view = pmrem_output.create_view(&storage_texture_descriptor(final_mip));
    let params = create_ibl_bake_wgpu_params_buffer(device, final_mip);
    let bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        &layouts,
        final_mip,
        &params,
        &source_view,
        &sampler,
        IblBakeWgpuOutputBindingResource::StorageTexture2DArray(&pmrem_output_view),
    )
    .expect("final PMREM bind group should be valid");
    let pipeline = create_ibl_bake_wgpu_compute_pipeline(
        device,
        final_mip,
        layouts.layout(final_mip.bind_group_layout_kind),
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-final-mip-average-test-encoder"),
    });

    let record =
        encode_ibl_bake_wgpu_compute_dispatch(&mut encoder, final_mip, &pipeline, &bind_group)
            .expect("final PMREM dispatch should encode");
    assert_eq!(record.dispatch_groups, [1, 1, 6]);
    queue.submit(std::iter::once(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("final PMREM dispatch should finish");

    let bytes = read_rgba16float_mip_faces(device, queue, &pmrem_output, final_mip_level, 1, 6);
    let first = rgba16float_face_color(&bytes, 0);
    assert!(
        first[0] + first[1] + first[2] > 0.05,
        "final PMREM average should contain source radiance, got {first:?}"
    );
    for face in 1..6 {
        let actual = rgba16float_face_color(&bytes, face);
        assert_vec4_near(
            actual,
            first,
            0.001,
            &format!("final PMREM face {face} should match face 0"),
        );
    }
}

#[test]
fn dispatch_encoder_rejects_zero_dispatch_groups_before_wgpu_pass() {
    let request = request(16, 5, IblBakeArtifactContents::SH9);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let mut command =
        command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9).clone();
    command.dispatch_groups = [0, 4, 6];
    let result = validate_dispatch_groups(&command);

    assert!(result.is_err());
    assert!(
        result
            .err()
            .unwrap()
            .contains("invalid zero dispatch groups")
    );
}

#[test]
fn graph_context_records_pmrem_wgpu_dispatch_from_materialized_resources() {
    let request = request(16, 5, IblBakeArtifactContents::PMREM);
    let pass_name = super::super::ibl_bake_graph_plan::ibl_bake_pmrem_pass_name(0);
    let Some((encoded, records, executed_pass_name)) =
        record_graph_context_dispatch(&request, &pass_name)
    else {
        return;
    };

    let pmrem_base_dispatch = [
        SOURCE_CUBEMAP_PMREM_FACE_SIZE.div_ceil(IBL_BAKE_WORKGROUP_SIZE[0]),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE.div_ceil(IBL_BAKE_WORKGROUP_SIZE[1]),
        6,
    ];
    assert_eq!(encoded.dispatch_groups, pmrem_base_dispatch);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].pass_name, executed_pass_name);
    assert_eq!(records[0].dispatch_groups, pmrem_base_dispatch);
    assert_eq!(
        records[0].storage_write_resources,
        [super::super::ibl_bake_graph_plan::IBL_BAKE_PMREM_RESOURCE.to_string()]
    );
}

#[test]
fn graph_context_records_sh9_wgpu_dispatch_from_materialized_resources() {
    let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9);
    let Some((encoded, records, executed_pass_name)) = record_graph_context_dispatch(
        &request,
        super::super::ibl_bake_graph_plan::IBL_BAKE_IRRADIANCE_SH9_PASS,
    ) else {
        return;
    };

    assert_eq!(encoded.dispatch_groups, [1, 1, 1]);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].pass_name, executed_pass_name);
    assert_eq!(records[0].dispatch_groups, [1, 1, 1]);
    assert_eq!(
        records[0].storage_write_resources,
        [super::super::ibl_bake_graph_plan::IBL_BAKE_IRRADIANCE_SH9_RESOURCE.to_string()]
    );
}

#[test]
fn graph_context_records_iem_wgpu_dispatch_from_materialized_resources() {
    let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9_IEM);
    let Some((encoded, records, executed_pass_name)) = record_graph_context_dispatch(
        &request,
        super::super::ibl_bake_graph_plan::IBL_BAKE_IRRADIANCE_CUBE_PASS,
    ) else {
        return;
    };

    assert_eq!(encoded.dispatch_groups, [4, 4, 6]);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].pass_name, executed_pass_name);
    assert_eq!(records[0].dispatch_groups, [4, 4, 6]);
    assert_eq!(
        records[0].storage_write_resources,
        [super::super::ibl_bake_graph_plan::IBL_BAKE_IRRADIANCE_CUBE_RESOURCE.to_string()]
    );
}

fn record_graph_context_dispatch(
    request: &IblBakeArtifactRequest,
    pass_name: &str,
) -> Option<(
    IblBakeWgpuEncodedDispatch,
    Vec<RenderGraphComputeDispatchRecord>,
    String,
)> {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return None;
    };
    let mut builder = RenderGraphBuilder::new("ibl-bake-wgpu-context-test");
    super::super::ibl_bake_graph_plan::append_ibl_bake_artifact_graph_plan(&mut builder, request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("IBL bake graph should compile");
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .unwrap_or_else(|| panic!("IBL bake pass `{pass_name}` should exist"));
    let mut resources = RenderGraphExecutionResources::new();
    let mut transient_pool = TransientResourcePool::default();
    transient_pool.begin_frame();
    resources
        .materialize_transient_resources_with_pool(&backend.device, &graph, &mut transient_pool)
        .expect("IBL transient outputs should materialize");
    let source_texture = create_source_cubemap_texture(&backend.device);
    resources.import_texture_view(
        super::super::ibl_bake_graph_plan::IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        source_texture.create_view(&source_cubemap_view_descriptor()),
    );
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ibl-bake-wgpu-context-test-encoder"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ibl-bake-wgpu-context-test-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ibl-bake-wgpu-context-test-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut ibl_bake_pipeline_cache = IblBakeWgpuPipelineCache::new(&backend.device);

    let (encoded, records) = {
        let gpu = RenderPassGpuExecutionContext::new_for_test(
            &backend.device,
            &backend.queue,
            &mut encoder,
            &frame,
            &scene_bind_group_layout,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Depth32Float,
            &scene_bind_group,
            &mut resources,
            &mut plugin_outputs,
            &mut screen_space_ui_renderer,
        )
        .with_ibl_bake_pipeline_cache(&mut ibl_bake_pipeline_cache);
        let mut context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            )
            .with_resource_resolver(&graph, pass.id)
            .with_gpu(gpu);

        let encoded = record_ibl_bake_wgpu_pass_for_request(&mut context, request)
            .expect("IBL WGPU pass should encode from graph resources");
        let records = context.gpu_mut().unwrap().take_compute_dispatches();
        (encoded, records)
    };

    backend.queue.submit(std::iter::once(encoder.finish()));
    backend
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("IBL graph-context dispatch should finish");

    Some((encoded, records, pass.name.clone()))
}

#[test]
fn graph_context_rejects_pmrem_pass_without_mip_suffix_before_gpu_lookup() {
    let request = request(16, 5, IblBakeArtifactContents::PMREM);
    let mut context = RenderPassExecutionContext::with_declared_graph_metadata_and_resources(
        "env.ibl_prefilter",
        RenderPassExecutorId::new(super::super::ibl_bake_graph_plan::IBL_BAKE_PMREM_EXECUTOR_ID),
        QueueLane::AsyncCompute,
        QueueLane::AsyncCompute,
        Default::default(),
        Vec::new(),
    );

    let error = record_ibl_bake_wgpu_pass_for_request(&mut context, &request).unwrap_err();

    assert!(error.contains("no IBL bake WGPU command matches pass"));
}

fn request(
    face_size: u32,
    mip_count: u32,
    contents: IblBakeArtifactContents,
) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        face_size,
        mip_count,
    )
    .with_required_contents(contents)
}

fn command_for_kind(
    commands: &[IblBakeWgpuCommandPlan],
    kind: IblBakeComputeKernelKind,
) -> &IblBakeWgpuCommandPlan {
    commands
        .iter()
        .find(|command| command.kind == kind)
        .expect("requested command should be present")
}

fn create_source_cubemap_texture(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-dispatch-test-source-cubemap"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn source_cubemap_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
    source_cubemap_mip_view_descriptor(1)
}

fn source_cubemap_mip_view_descriptor(mip_count: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("ibl-bake-dispatch-test-source-cubemap-view"),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(mip_count.max(1)),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}

fn create_asymmetric_source_cubemap_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    face_size: u32,
    mip_count: u32,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-final-mip-average-source-cubemap"),
        size: wgpu::Extent3d {
            width: face_size,
            height: face_size,
            depth_or_array_layers: 6,
        },
        mip_level_count: mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let face_colors = [
        [255_u8, 16, 16, 255],
        [32, 255, 32, 255],
        [32, 32, 255, 255],
        [255, 255, 32, 255],
        [255, 32, 255, 255],
        [32, 255, 255, 255],
    ];

    for mip_level in 0..mip_count {
        let mip_size = (face_size >> mip_level).max(1);
        for (face, color) in face_colors.iter().enumerate() {
            let rgba = repeated_rgba(*color, mip_size);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * mip_size),
                    rows_per_image: Some(mip_size),
                },
                wgpu::Extent3d {
                    width: mip_size,
                    height: mip_size,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    texture
}

fn repeated_rgba(color: [u8; 4], size: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(size as usize * size as usize * 4);
    for _ in 0..size * size {
        rgba.extend_from_slice(&color);
    }
    rgba
}

fn create_storage_output_texture(
    device: &wgpu::Device,
    face_size: u32,
    mip_count: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-dispatch-test-storage-output"),
        size: wgpu::Extent3d {
            width: face_size,
            height: face_size,
            depth_or_array_layers: 6,
        },
        mip_level_count: mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn storage_texture_descriptor(
    command: &IblBakeWgpuCommandPlan,
) -> wgpu::TextureViewDescriptor<'static> {
    let IblBakeWgpuOutputPlan::StorageTexture { view, .. } = &command.output else {
        panic!("command should write a storage texture")
    };
    (*view).to_wgpu_descriptor()
}

fn create_sh9_output_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ibl-bake-dispatch-test-sh9-output"),
        size: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn read_rgba16float_mip_faces(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    mip_level: u32,
    mip_size: u32,
    face_count: u32,
) -> Vec<u8> {
    let bytes_per_pixel = 8_u32;
    let unpadded_bytes_per_row = mip_size * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer_size = u64::from(padded_bytes_per_row) * u64::from(mip_size) * u64::from(face_count);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ibl-bake-final-mip-average-readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ibl-bake-final-mip-average-readback-encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(mip_size),
            },
        },
        wgpu::Extent3d {
            width: mip_size,
            height: mip_size,
            depth_or_array_layers: face_count,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("final PMREM readback poll should finish");
    receiver
        .recv()
        .expect("final PMREM readback callback should run")
        .expect("final PMREM readback should map");

    let mapped = slice.get_mapped_range();
    let mut rgba = vec![0_u8; face_count as usize * mip_size as usize * mip_size as usize * 8];
    for face in 0..face_count as usize {
        for row in 0..mip_size as usize {
            let source_offset = face * mip_size as usize * padded_bytes_per_row as usize
                + row * padded_bytes_per_row as usize;
            let target_offset = face * mip_size as usize * unpadded_bytes_per_row as usize
                + row * unpadded_bytes_per_row as usize;
            rgba[target_offset..target_offset + unpadded_bytes_per_row as usize].copy_from_slice(
                &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
            );
        }
    }
    drop(mapped);
    buffer.unmap();

    rgba
}

fn rgba16float_face_color(bytes: &[u8], face: usize) -> [f32; 4] {
    let offset = face * 8;
    [
        f16_to_f32(u16::from_le_bytes([bytes[offset], bytes[offset + 1]])),
        f16_to_f32(u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]])),
        f16_to_f32(u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]])),
        f16_to_f32(u16::from_le_bytes([bytes[offset + 6], bytes[offset + 7]])),
    ]
}

fn assert_vec4_near(actual: [f32; 4], expected: [f32; 4], tolerance: f32, context: &str) {
    for index in 0..4 {
        assert!(
            (actual[index] - expected[index]).abs() <= tolerance,
            "{context}: component {index} actual={actual:?} expected={expected:?}"
        );
    }
}

fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exponent = (bits >> 10) & 0x1f;
    let fraction = bits & 0x03ff;
    match exponent {
        0 => {
            if fraction == 0 {
                f32::from_bits(sign)
            } else {
                let mut normalized_fraction = fraction;
                let mut exponent_value = -14_i32;
                while normalized_fraction & 0x0400 == 0 {
                    normalized_fraction <<= 1;
                    exponent_value -= 1;
                }
                normalized_fraction &= 0x03ff;
                f32::from_bits(
                    sign | (((exponent_value + 127) as u32) << 23)
                        | ((normalized_fraction as u32) << 13),
                )
            }
        }
        0x1f => f32::from_bits(sign | 0x7f80_0000 | ((fraction as u32) << 13)),
        _ => f32::from_bits(
            sign | ((((exponent as i32) - 15 + 127) as u32) << 23) | ((fraction as u32) << 13),
        ),
    }
}
