use std::{fs, path::PathBuf, sync::mpsc};

use image::{ImageBuffer, ImageFormat, Rgba};
use wgpu::util::DeviceExt;

use super::super::buffer_helpers::{create_pod_storage_buffer, create_u32_storage_buffer};
use super::packing::{
    GlobalSdfGpuDispatchParams, GlobalSdfGpuMeshPayload, GlobalSdfGpuObject, GlobalSdfGpuPage,
    GLOBAL_SDF_PAGE_CELLS_PER_EDGE, GLOBAL_SDF_PAGE_VOXEL_COUNT,
};
use super::{GlobalSdfGpuResources, GlobalSdfGpuState};

const GLOBAL_SDF_BUILD_WGPU_PNG: &str = "plan18_hybrid_gi_m5_global_sdf_build_wgpu_20260813.png";

struct GlobalSdfBuildReadback {
    atlas: Vec<u32>,
    completion: Vec<u32>,
}

#[test]
fn global_sdf_build_shader_writes_mesh_sdf_distance_and_completion() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping Global SDF build Wgpu test because no adapter is available");
        return;
    };
    let readback = dispatch_sphere_global_sdf_build(&device, &queue);
    assert_sphere_global_sdf_readback(&readback);
}

#[test]
#[ignore = "requires a real Wgpu adapter and writes a Global SDF build PNG"]
fn export_global_sdf_build_wgpu_png() {
    let (device, queue) = test_device_with_backends(wgpu::Backends::DX12).expect(
        "Global SDF PNG export requires a DX12 Wgpu adapter for RenderDoc; do not accept a skipped run",
    );
    let readback = dispatch_sphere_global_sdf_build(&device, &queue);
    assert_sphere_global_sdf_readback(&readback);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_global_sdf_slice_png(output_dir.join(GLOBAL_SDF_BUILD_WGPU_PNG), &readback.atlas);
}

fn dispatch_sphere_global_sdf_build(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> GlobalSdfBuildReadback {
    let resources = GlobalSdfGpuResources::new(device);
    let state = GlobalSdfGpuState::new(device);
    let params = GlobalSdfGpuDispatchParams {
        page_count: 1,
        object_count: 1,
        payload_count: 1,
        candidate_count: 1,
    };
    let pages = [GlobalSdfGpuPage {
        world_min_and_cell_size: [-4.0, -4.0, -4.0, 1.0],
        atlas_slot: 0,
        candidate_offset: 0,
        candidate_count: 1,
        _padding: 0,
    }];
    let objects = [GlobalSdfGpuObject {
        world_min_and_mode: [-4.0, -4.0, -4.0, 1.0],
        world_max_and_padding: [4.0, 4.0, 4.0, 0.0],
        payload_offset: 0,
        payload_count: 1,
        _padding: [0; 2],
    }];
    let payloads = [GlobalSdfGpuMeshPayload {
        local_min_and_distance_min: [-4.0, -4.0, -4.0, -4.0],
        local_max_and_distance_max: [4.0, 4.0, 4.0, 4.0],
        dimensions_and_voxel_offset: [8, 8, 8, 0],
        world_to_local: zircon_runtime::core::math::Mat4::IDENTITY.to_cols_array_2d(),
        distance_scale_and_padding: [1.0, 0.0, 0.0, 0.0],
    }];
    let voxels = sphere_snorm16_voxels(1.25, 4.0);
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-global-sdf-build-test-params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let page_buffer = create_pod_storage_buffer(
        &device,
        "zircon-global-sdf-build-test-pages",
        &pages,
        wgpu::BufferUsages::STORAGE,
    );
    let object_buffer = create_pod_storage_buffer(
        &device,
        "zircon-global-sdf-build-test-objects",
        &objects,
        wgpu::BufferUsages::STORAGE,
    );
    let payload_buffer = create_pod_storage_buffer(
        &device,
        "zircon-global-sdf-build-test-payloads",
        &payloads,
        wgpu::BufferUsages::STORAGE,
    );
    let voxel_buffer = create_u32_storage_buffer(
        &device,
        "zircon-global-sdf-build-test-voxels",
        &voxels,
        wgpu::BufferUsages::STORAGE,
    );
    let candidate_buffer = create_u32_storage_buffer(
        &device,
        "zircon-global-sdf-build-test-candidates",
        &[0],
        wgpu::BufferUsages::STORAGE,
    );
    let completion_buffer = create_u32_storage_buffer(
        &device,
        "zircon-global-sdf-build-test-completion",
        &[0],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-global-sdf-build-test-bind-group"),
        layout: &resources.bind_group_layout,
        entries: &[
            binding(0, &params_buffer),
            binding(1, &page_buffer),
            binding(2, &object_buffer),
            binding(3, &payload_buffer),
            binding(4, &voxel_buffer),
            binding(5, &candidate_buffer),
            binding(6, &state.atlas_buffer),
            binding(7, &completion_buffer),
        ],
    });
    let atlas_readback = readback_buffer(
        &device,
        "zircon-global-sdf-build-test-atlas-readback",
        GLOBAL_SDF_PAGE_VOXEL_COUNT,
    );
    let completion_readback = readback_buffer(
        &device,
        "zircon-global-sdf-build-test-completion-readback",
        1,
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-global-sdf-build-test-encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("GlobalSdfBuildTestPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&resources.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((GLOBAL_SDF_PAGE_VOXEL_COUNT as u32).div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(
        &state.atlas_buffer,
        0,
        &atlas_readback,
        0,
        (GLOBAL_SDF_PAGE_VOXEL_COUNT * std::mem::size_of::<u32>()) as u64,
    );
    encoder.copy_buffer_to_buffer(
        &completion_buffer,
        0,
        &completion_readback,
        0,
        std::mem::size_of::<u32>() as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));

    let atlas = readback_u32s(device, &atlas_readback, GLOBAL_SDF_PAGE_VOXEL_COUNT);
    let completion = readback_u32s(device, &completion_readback, 1);
    GlobalSdfBuildReadback { atlas, completion }
}

fn assert_sphere_global_sdf_readback(readback: &GlobalSdfBuildReadback) {
    let inside = f32::from_bits(readback.atlas[global_sdf_voxel_index(3, 3, 3)]);
    let outside = f32::from_bits(readback.atlas[0]);
    assert!(inside < 0.0, "inside Mesh SDF distance should be negative");
    assert!(
        outside > 0.0,
        "outside Mesh SDF distance should be positive"
    );
    assert_eq!(readback.completion, vec![1]);
}

fn write_global_sdf_slice_png(path: PathBuf, atlas: &[u32]) {
    const DISPLAY_SCALE: u32 = 64;
    const SLICE_Z: usize = 3;
    let side = GLOBAL_SDF_PAGE_CELLS_PER_EDGE as u32 * DISPLAY_SCALE;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(side, side, |x, y| {
        let cell_x = (x / DISPLAY_SCALE) as usize;
        let cell_y = (y / DISPLAY_SCALE) as usize;
        let distance = f32::from_bits(atlas[global_sdf_voxel_index(cell_x, cell_y, SLICE_Z)]);
        let color = signed_distance_color(distance);
        if x % DISPLAY_SCALE == 0 || y % DISPLAY_SCALE == 0 {
            Rgba([24, 28, 34, 255])
        } else {
            color
        }
    });
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn global_sdf_voxel_index(x: usize, y: usize, z: usize) -> usize {
    let edge = GLOBAL_SDF_PAGE_CELLS_PER_EDGE;
    x + y * edge + z * edge * edge
}

fn signed_distance_color(distance: f32) -> Rgba<u8> {
    let normalized = (distance / 4.0).clamp(-1.0, 1.0);
    if normalized < 0.0 {
        let intensity = (-normalized * 255.0).round() as u8;
        Rgba([255, 64_u8.saturating_sub(intensity / 4), 0, 255])
    } else {
        let intensity = (normalized * 255.0).round() as u8;
        Rgba([0, 96_u8.saturating_add(intensity / 3), 160, 255])
    }
}

fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}

fn sphere_snorm16_voxels(radius: f32, distance_limit: f32) -> Vec<u32> {
    (0..8)
        .flat_map(|z| {
            (0..8).flat_map(move |y| {
                (0..8).map(move |x| {
                    let position = zircon_runtime::core::math::Vec3::new(
                        x as f32 - 3.5,
                        y as f32 - 3.5,
                        z as f32 - 3.5,
                    );
                    let encoded = (((position.length() - radius) / distance_limit).clamp(-1.0, 1.0)
                        * i16::MAX as f32)
                        .round() as i16;
                    i32::from(encoded) as u32
                })
            })
        })
        .collect()
}

fn binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    test_device_with_backends(wgpu::Backends::PRIMARY)
}

fn test_device_with_backends(backends: wgpu::Backends) -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = backends;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-global-sdf-build-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}

fn readback_buffer(device: &wgpu::Device, label: &'static str, word_count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (word_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn readback_u32s(device: &wgpu::Device, buffer: &wgpu::Buffer, word_count: usize) -> Vec<u32> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    receiver.recv().unwrap().unwrap();
    let bytes = slice.get_mapped_range();
    let words = bytemuck::cast_slice::<u8, u32>(&bytes)
        .iter()
        .copied()
        .take(word_count)
        .collect();
    drop(bytes);
    buffer.unmap();
    words
}
