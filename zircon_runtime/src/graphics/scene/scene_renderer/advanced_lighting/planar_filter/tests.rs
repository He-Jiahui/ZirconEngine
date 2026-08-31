use std::sync::mpsc;

use super::*;

const BASE_SIZE: u32 = 8;
const MIP_COUNT: u32 = 4;
const READBACK_BYTES_PER_ROW: u32 = 256;

#[test]
fn render_planar_filter_plugin_workload_uses_the_pipeline_owned_contract() {
    let workload = planar_reflection_filter_compute_workload();
    assert_eq!(workload.pipeline_label, PLANAR_FILTER_PIPELINE_LABEL);
    assert_eq!(workload.workgroup_size, PLANAR_FILTER_WORKGROUP_SIZE);
    assert_eq!(
        workload.dispatch_extent,
        crate::render_graph::RenderGraphComputeDispatchExtent::PerPixel {
            target: PLANAR_REFLECTION_TEXTURE_RESOURCE.to_string(),
            local_size: [
                PLANAR_FILTER_WORKGROUP_SIZE[0],
                PLANAR_FILTER_WORKGROUP_SIZE[1]
            ],
        }
    );
}

#[test]
fn render_planar_filter_shader_parses_and_owns_roughness_mip_contract() {
    let module = naga::front::wgsl::parse_str(PLANAR_FILTER_SHADER)
        .expect("planar reflection filter shader should parse");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .expect("planar reflection filter shader should validate");
    assert!(PLANAR_FILTER_SHADER.contains("texture_storage_2d<rgba16float, write>"));
    assert!(PLANAR_FILTER_SHADER.contains("@workgroup_size(8, 8, 1)"));
    assert!(PLANAR_FILTER_SHADER.contains("params.input_dimensions / params.output_dimensions"));
}

#[test]
fn render_planar_filter_rejects_invalid_extent_and_mip_count() {
    assert!(validate_filter_request(
        wgpu::Extent3d {
            width: 0,
            height: 8,
            depth_or_array_layers: 1,
        },
        1,
    )
    .is_err());
    assert!(validate_filter_request(test_extent(), 0).is_err());
    assert!(validate_filter_request(test_extent(), MIP_COUNT + 1).is_err());
    assert!(validate_filter_request(test_extent(), MIP_COUNT).is_ok());
}

#[test]
fn render_planar_filter_routes_native_creates_through_pass_capability() {
    let recording =
        include_str!("../../graph_execution/render_pass_execution_context/gpu/native.rs");
    let pipeline = include_str!("mod.rs");
    let executor = include_str!("executor.rs");

    assert!(pipeline.contains("RenderPassGpuResourceFactory"));
    assert!(pipeline.contains("RenderPassGpuRecordingContext"));
    assert!(recording.contains("trait RenderPassGpuRecordingContext"));
    assert!(recording.contains("type ResourceFactory: RenderPassGpuResourceFactory + ?Sized"));
    assert!(pipeline.contains("fn new<F: RenderPassGpuResourceFactory + ?Sized>(factory: &F)"));
    assert!(pipeline.contains("fn encode<C: RenderPassGpuRecordingContext>"));
    assert!(!pipeline.contains("device.create_buffer_init"));
    assert!(!pipeline.contains("device.create_bind_group("));
    assert!(!pipeline.contains("device.create_bind_group_layout("));
    assert!(!pipeline.contains("device.create_shader_module("));
    assert!(!pipeline.contains("device.create_pipeline_layout("));
    assert!(!pipeline.contains("device.create_compute_pipeline("));

    assert!(executor.contains("let mut native = gpu.native_context()"));
    assert!(executor.contains("PlanarReflectionFilterPipeline::new("));
    assert!(executor.contains("native.resource_factory()"));
    assert!(executor.contains("pipeline.encode("));
    assert!(executor.contains("&mut native"));
    let native_scope = executor
        .find("let report = {")
        .expect("planar filter must scope native capability and pipeline lock");
    let dispatch_recording = executor
        .find("gpu.record_compute_dispatch_with_uploaded_bytes(")
        .expect("planar filter must record dispatches after encoding");
    assert!(native_scope < dispatch_recording);
    assert!(!executor.contains("PlanarReflectionFilterPipeline::new(gpu.device)"));
    assert!(!executor.contains("pipeline.encode(\n            gpu.device"));
}

#[test]
fn render_planar_filter_wgpu_builds_blurred_rgba16f_mip_chain() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let source = source_texture(&device, &queue);
    let source_view = source.create_view(&wgpu::TextureViewDescriptor::default());
    let output = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-planar-filter-test-output"),
        size: test_extent(),
        mip_level_count: MIP_COUNT,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-planar-filter-test-readback"),
        size: u64::from(READBACK_BYTES_PER_ROW),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let pipeline = PlanarReflectionFilterPipeline::new(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-planar-filter-test-encoder"),
    });
    let mut recording = (&device, &mut encoder);
    let report = pipeline
        .encode(
            &mut recording,
            &source_view,
            &output,
            test_extent(),
            MIP_COUNT,
        )
        .expect("valid planar filter request");
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &output,
            mip_level: MIP_COUNT - 1,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(READBACK_BYTES_PER_ROW),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    assert_eq!(report.mip_count, MIP_COUNT);
    assert_eq!(report.dispatches, vec![[1, 1]; MIP_COUNT as usize]);
    assert_eq!(report.uploaded_bytes, 32 * u64::from(MIP_COUNT));
    let rgba = read_rgba16f(&device, &readback);
    assert!(rgba[0] > 0.2, "red should survive mip filtering: {rgba:?}");
    assert!(rgba[2] > 0.2, "blue should survive mip filtering: {rgba:?}");
    assert!(rgba[1] < 0.05, "green should remain absent: {rgba:?}");
    assert!(rgba[3] > 0.9, "alpha should remain opaque: {rgba:?}");
}

fn source_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-planar-filter-test-source"),
        size: test_extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut rgba = Vec::with_capacity((BASE_SIZE * BASE_SIZE * 4) as usize);
    for _y in 0..BASE_SIZE {
        for x in 0..BASE_SIZE {
            rgba.extend_from_slice(if x < BASE_SIZE / 2 {
                &[255, 0, 0, 255]
            } else {
                &[0, 0, 255, 255]
            });
        }
    }
    queue.write_texture(
        texture.as_image_copy(),
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(BASE_SIZE * 4),
            rows_per_image: Some(BASE_SIZE),
        },
        test_extent(),
    );
    texture
}

fn read_rgba16f(device: &wgpu::Device, buffer: &wgpu::Buffer) -> [f32; 4] {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll should complete planar filter readback");
    receiver
        .recv()
        .expect("planar filter readback callback should run")
        .expect("planar filter readback mapping should succeed");
    let mapped = slice.get_mapped_range();
    let words = bytemuck::cast_slice::<u8, u16>(&mapped[..8]);
    let rgba = [words[0], words[1], words[2], words[3]].map(f16_bits_to_f32);
    drop(mapped);
    buffer.unmap();
    rgba
}

fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x03ff);
    let f32_bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut normalized = mantissa;
            let mut shift = 0_u32;
            while normalized & 0x0400 == 0 {
                normalized <<= 1;
                shift += 1;
            }
            sign | ((113_u32.saturating_sub(shift)) << 23) | ((normalized & 0x03ff) << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(f32_bits)
}

fn test_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: BASE_SIZE,
        height: BASE_SIZE,
        depth_or_array_layers: 1,
    }
}

fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-planar-filter-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
