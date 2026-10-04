use std::{borrow::Cow, sync::mpsc};

use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryPagePayload, RenderVirtualGeometrySelectedCluster,
};

use super::{
    cluster_candidates, create_cluster_triangle_pipeline, resident_page_ids,
    storage_buffer_layout_entry, ClusterTriangleTarget, ClusterVertex,
    RenderVirtualGeometryDebugSnapshot,
};

#[test]
fn virtual_geometry_graph_gpu_execution_shaders_validate_on_wgpu_device() {
    let backend = RenderBackend::new_offscreen();
    let error_scope = backend
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let _cull = backend
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-virtual-geometry-graph-cull-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/node_cluster_cull.wgsl"
            ))),
        });
    let cull_layout = backend
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-virtual-geometry-graph-cull-layout"),
            entries: &[
                storage_buffer_layout_entry(0, true, wgpu::ShaderStages::COMPUTE),
                storage_buffer_layout_entry(1, true, wgpu::ShaderStages::COMPUTE),
                storage_buffer_layout_entry(2, false, wgpu::ShaderStages::COMPUTE),
            ],
        });
    let cull_pipeline_layout =
        backend
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("zircon-test-virtual-geometry-graph-cull-pipeline-layout"),
                bind_group_layouts: &[Some(&cull_layout)],
                immediate_size: 0,
            });
    let _cull_pipeline = backend
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-test-virtual-geometry-graph-cull-pipeline"),
            layout: Some(&cull_pipeline_layout),
            module: &_cull,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
    let _raster = backend
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-virtual-geometry-graph-raster-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/cluster_triangles.wgsl"
            ))),
        });
    let error = pollster::block_on(error_scope.pop());

    assert!(
        error.is_none(),
        "virtual geometry graph shaders should pass WGPU validation: {error:?}"
    );
}

#[test]
fn virtual_geometry_graph_gpu_execution_keeps_real_encoding_operations() {
    let source = include_str!("../../gpu.rs");

    assert!(source.contains("buffer_upload_recorder().write_buffer("));
    assert!(source.contains("begin_compute_pass("));
    assert!(source.contains("dispatch_workgroups("));
    assert!(source.contains("copy_buffer_to_buffer("));
    assert!(source.contains("begin_render_pass("));
    assert!(source.contains("pass.draw("));
    assert!(source.contains("page.cluster_ranges"));
    assert!(source.contains("clip_from_world * transform"));
}

#[test]
fn virtual_geometry_graph_gpu_execution_uses_resident_pages_and_cluster_candidates() {
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        resident_page_payloads: vec![RenderVirtualGeometryPagePayload::new(17, Vec::new())],
        selected_clusters: vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: 5,
            cluster_id: 9,
            cluster_ordinal: 0,
            page_id: 17,
            lod_level: 0,
            state: zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
        }],
        ..Default::default()
    };

    assert_eq!(resident_page_ids(Some(&snapshot)), vec![17]);
    let candidates = cluster_candidates(Some(&snapshot));
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].cluster_id, 9);
    assert_eq!(candidates[0].page_id, 17);
}

#[test]
fn virtual_geometry_graph_gpu_execution_culls_nonresident_pages_and_reads_back_ids() {
    let backend = RenderBackend::new_offscreen();
    let page_requests = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-vg-page-requests"),
            contents: bytemuck::cast_slice(&[2_u32, 17, 21]),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let candidate_words = [3_u32, 0, 9, 17, 10, 44, 11, 21];
    let candidates = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-vg-cluster-candidates"),
            contents: bytemuck::cast_slice(&candidate_words),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let visible = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-vg-visible-clusters"),
            contents: bytemuck::cast_slice(&[0_u32; 4]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
    let readback = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-vg-visible-readback"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let shader = backend
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-vg-node-cull-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/node_cluster_cull.wgsl"
            ))),
        });
    let layout = backend
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-vg-node-cull-layout"),
            entries: &[
                storage_buffer_layout_entry(0, true, wgpu::ShaderStages::COMPUTE),
                storage_buffer_layout_entry(1, true, wgpu::ShaderStages::COMPUTE),
                storage_buffer_layout_entry(2, false, wgpu::ShaderStages::COMPUTE),
            ],
        });
    let pipeline_layout = backend
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-test-vg-node-cull-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
    let pipeline = backend
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-test-vg-node-cull-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
    let bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-test-vg-node-cull-bind-group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: page_requests.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: candidates.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visible.as_entire_binding(),
                },
            ],
        });
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-test-vg-node-cull-encoder"),
        });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-test-vg-node-cull"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&visible, 0, &readback, 0, 16);
    backend.queue.submit([encoder.finish()]);

    let mut words = read_u32_words(&backend.device, &readback, 4);
    assert_eq!(words[0], 2);
    words[1..3].sort_unstable();
    assert_eq!(&words[1..3], &[9, 11]);
}

#[test]
fn virtual_geometry_graph_gpu_execution_pulls_cluster_vertices_into_color_pixels() {
    const EXTENT: u32 = 8;
    const ROW_BYTES: u32 = 256;
    let backend = RenderBackend::new_offscreen();
    assert_eq!(std::mem::size_of::<ClusterVertex>(), 32);
    let visible = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-vg-raster-visible"),
            contents: bytemuck::cast_slice(&[1_u32, 9]),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let vertices = [
        ClusterVertex {
            clip_position: [-0.75, -0.75, 0.0, 1.0],
            cluster_id: 9,
            _padding: [0; 3],
        },
        ClusterVertex {
            clip_position: [0.75, -0.75, 0.0, 1.0],
            cluster_id: 9,
            _padding: [0; 3],
        },
        ClusterVertex {
            clip_position: [0.0, 0.75, 0.0, 1.0],
            cluster_id: 9,
            _padding: [0; 3],
        },
    ];
    let vertex_buffer = backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-vg-raster-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let target = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-test-vg-raster-target"),
        size: wgpu::Extent3d {
            width: EXTENT,
            height: EXTENT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-vg-raster-readback"),
        size: u64::from(ROW_BYTES * EXTENT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let state = create_cluster_triangle_pipeline(
        &backend.device,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
        ClusterTriangleTarget::Color(&target_view),
    );
    let bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-test-vg-raster-bind-group"),
            layout: &state.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: visible.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vertex_buffer.as_entire_binding(),
                },
            ],
        });
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-test-vg-raster-encoder"),
        });
    {
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &target_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-test-vg-raster-pass"),
            color_attachments: &attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&state.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ROW_BYTES),
                rows_per_image: Some(EXTENT),
            },
        },
        wgpu::Extent3d {
            width: EXTENT,
            height: EXTENT,
            depth_or_array_layers: 1,
        },
    );
    backend.queue.submit([encoder.finish()]);
    let bytes = read_bytes(&backend.device, &readback);
    assert!(
        bytes.chunks_exact(4).any(|pixel| pixel[3] != 0),
        "vertex-pulled virtual geometry triangle should produce color pixels"
    );
}

struct RenderBackend {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RenderBackend {
    fn new_offscreen() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()))
            .into_iter()
            .find(|adapter| adapter.get_info().name.contains("RTX 3060"))
            .expect("RTX 3060 virtual geometry validation requires an enumerated RTX 3060 adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("zircon-virtual-geometry-graph-executor-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .expect("RTX 3060 virtual geometry validation device should initialize");
        Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
        }
    }
}

fn read_u32_words(device: &wgpu::Device, buffer: &wgpu::Buffer, word_count: usize) -> Vec<u32> {
    let slice = buffer.slice(..(word_count * std::mem::size_of::<u32>()) as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .expect("device poll should complete virtual geometry readback mapping");
    receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("virtual geometry readback callback should run")
        .expect("virtual geometry readback mapping should succeed");
    let mapped = slice.get_mapped_range();
    let words = bytemuck::cast_slice(&mapped).to_vec();
    drop(mapped);
    buffer.unmap();
    words
}

fn read_bytes(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<u8> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .expect("device poll should complete virtual geometry pixel readback");
    receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("virtual geometry pixel readback callback should run")
        .expect("virtual geometry pixel readback mapping should succeed");
    let mapped = slice.get_mapped_range();
    let bytes = mapped.to_vec();
    drop(mapped);
    buffer.unmap();
    bytes
}
