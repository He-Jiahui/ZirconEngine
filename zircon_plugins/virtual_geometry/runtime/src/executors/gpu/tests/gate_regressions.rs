use std::borrow::Cow;

use wgpu::util::DeviceExt;

use super::super::storage_buffer_layout_entry;
use super::{read_u32_words, RenderBackend};

#[test]
fn virtual_geometry_graph_gpu_empty_selection_rejects_malformed_visbuffer() {
    let backend = RenderBackend::new_offscreen();
    for vis_count in [0_u32, 1] {
        let indices = gpu_test_words(&backend, &[0]);
        let clusters = gpu_test_words(&backend, &[0]);
        let authors = gpu_test_words(&backend, &[0]);
        let selected = gpu_test_words(&backend, &[0]);
        let vis = gpu_test_words(&backend, &[vis_count, 0, 0]);
        let decision = gpu_test_words(&backend, &[0x5647_5331, 0, 0, 0, 0, 0, 1, 0]);
        let args = gpu_test_words(&backend, &[0]);
        let submission = gpu_test_words(&backend, &[0]);
        let hardware = gpu_test_words(&backend, &[0]);
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        dispatch_gpu_test_shader(
            &backend,
            &mut encoder,
            super::super::super::indirect::SELECTED_SHADER,
            &[
                (&indices, true),
                (&clusters, true),
                (&authors, true),
                (&selected, false),
                (&vis, true),
            ],
        );
        dispatch_gpu_test_shader(
            &backend,
            &mut encoder,
            super::super::super::selection::FINAL_PUBLICATION_SHADER,
            &[
                (&decision, false),
                (&args, false),
                (&submission, false),
                (&selected, false),
                (&hardware, false),
            ],
        );
        backend.queue.submit([encoder.finish()]);
        let selected_words = read_gpu_test_words(&backend, &selected, 1);
        let decision_words = read_gpu_test_words(&backend, &decision, 8);
        let hardware_words = read_gpu_test_words(&backend, &hardware, 1);
        assert_eq!(read_gpu_test_words(&backend, &args, 1), [0]);
        assert_eq!(read_gpu_test_words(&backend, &submission, 1), [0]);
        if vis_count == 0 {
            assert_eq!(selected_words, [0], "a valid empty selection stays empty");
            assert_eq!(hardware_words, [0]);
            assert_eq!(decision_words[7], 1, "a valid empty stream is published");
        } else {
            assert_eq!(selected_words, [u32::MAX]);
            assert_eq!(hardware_words, [u32::MAX]);
            assert_eq!(decision_words[7], 2, "malformed VisBuffer cannot publish");
        }
    }
}

#[test]
fn virtual_geometry_graph_gpu_raw_count_parent_cycles_fail_closed() {
    let backend = RenderBackend::new_offscreen();
    for (parents, valid) in [
        (vec![u32::MAX], true),
        (vec![9], false),
        (vec![10, 9], false),
    ] {
        let count = u32::try_from(parents.len()).expect("small GPU test input");
        let mut source = vec![0_u32; 64];
        source[..14].copy_from_slice(&[1, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0, u32::MAX, 0, 0]);
        source[14] = 0x5647_5331;
        source[15] = 2;
        source[16] = count;
        source[17] = count;
        source[20] = count;
        source[21] = 1;
        source[22] = 64;
        for (index, parent) in parents.iter().copied().enumerate() {
            let index = u32::try_from(index).expect("small GPU test index");
            source.extend_from_slice(&[
                7,
                0,
                9 + index,
                17 + index,
                0,
                parent,
                u32::MAX,
                0,
                0,
                0,
                0,
                0,
            ]);
        }
        source[23] = u32::try_from(source.len()).expect("small GPU test offset");
        for index in 0..count {
            source.extend_from_slice(&[0, 7, 0, 9 + index, index, 17 + index, 0, 0, index, 1]);
        }
        source[24] = u32::try_from(source.len()).expect("small GPU test offset");
        source[25] = source[24];
        source[26] = source[24];
        for index in 0..count {
            source.extend_from_slice(&[17 + index, 1]);
        }
        let pages = gpu_test_words(&backend, &[1, 17]);
        let source = gpu_test_words(&backend, &source);
        let visible = gpu_test_words(&backend, &[0, 0]);
        let candidates = gpu_test_words(&backend, &[0, 0]);
        let requests = gpu_test_words(&backend, &[0, 0]);
        let vis = gpu_test_words(&backend, &[0, 0, 0]);
        let startup = gpu_test_words(&backend, &[0, 7, 0, 0, 1, 0, 0, 1, 1, u32::MAX]);
        let decisions = gpu_test_words(&backend, &vec![0; 8 + 4096 * 33]);
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        dispatch_gpu_test_shader(
            &backend,
            &mut encoder,
            include_str!("../../node_cluster_cull.wgsl"),
            &[
                (&pages, true),
                (&source, true),
                (&visible, false),
                (&candidates, false),
                (&requests, false),
                (&vis, false),
                (&startup, true),
                (&decisions, false),
            ],
        );
        backend.queue.submit([encoder.finish()]);
        let decision = read_gpu_test_words(&backend, &decisions, 8);
        assert_eq!(decision[0], 0x5647_5331);
        let visible = read_gpu_test_words(&backend, &visible, 2);
        let candidates = read_gpu_test_words(&backend, &candidates, 2);
        let vis = read_gpu_test_words(&backend, &vis, 3);
        if valid {
            assert_eq!(decision[1], 0);
            assert_eq!(decision[2], 1);
            assert_eq!(visible, [1, 9]);
            assert_eq!(candidates, [1, 0]);
            assert_eq!(vis[0], 1);
        } else {
            assert_ne!(decision[1], 0, "full raw-count parent cycles are invalid");
            assert_eq!(visible[0], 0);
            assert_eq!(candidates[0], 0);
            assert_eq!(vis[0], 0);
        }
        assert_eq!(read_gpu_test_words(&backend, &requests, 2)[0], 0);
    }
}

fn gpu_test_words(backend: &RenderBackend, words: &[u32]) -> wgpu::Buffer {
    backend
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-gate-regression-words"),
            contents: bytemuck::cast_slice(words),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        })
}

fn dispatch_gpu_test_shader(
    backend: &RenderBackend,
    encoder: &mut wgpu::CommandEncoder,
    source: &str,
    buffers: &[(&wgpu::Buffer, bool)],
) {
    let shader = backend
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-vg-gate-regression-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        });
    let entries = buffers
        .iter()
        .enumerate()
        .map(|(binding, (_, read_only))| {
            storage_buffer_layout_entry(
                u32::try_from(binding).expect("bounded GPU test binding"),
                *read_only,
                wgpu::ShaderStages::COMPUTE,
            )
        })
        .collect::<Vec<_>>();
    let layout = backend
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-vg-gate-regression-layout"),
            entries: &entries,
        });
    let pipeline_layout = backend
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-vg-gate-regression-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
    let pipeline = backend
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-vg-gate-regression-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
    let entries = buffers
        .iter()
        .enumerate()
        .map(|(binding, (buffer, _))| wgpu::BindGroupEntry {
            binding: u32::try_from(binding).expect("bounded GPU test binding"),
            resource: buffer.as_entire_binding(),
        })
        .collect::<Vec<_>>();
    let bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-vg-gate-regression-bind-group"),
            layout: &layout,
            entries: &entries,
        });
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("zircon-vg-gate-regression-pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
}

fn read_gpu_test_words(backend: &RenderBackend, source: &wgpu::Buffer, count: usize) -> Vec<u32> {
    let bytes = u64::try_from(count).expect("bounded GPU test readback") * 4;
    let staging = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-gate-regression-readback"),
        size: bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(source, 0, &staging, 0, bytes);
    backend.queue.submit([encoder.finish()]);
    read_u32_words(&backend.device, &staging, count)
}
