use super::super::super::super::shader_sources::UPSCALE_SHADER;

#[test]
fn upscale_shader_parses() {
    let module = naga::front::wgsl::parse_str(UPSCALE_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(UPSCALE_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error}"));
}

fn regression_device() -> (wgpu::Device, wgpu::Queue) {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("this regression requires a native GPU adapter");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("render-contract-regression"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .expect("this regression requires a native GPU device")
}

fn regression_texture(
    device: &wgpu::Device,
    width: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render-contract-regression"),
        size: wgpu::Extent3d {
            width,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn read_regression_row(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    width: u32,
    byte_count: usize,
) -> Vec<u8> {
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("render-contract-readback"),
        size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    queue.submit([encoder.finish()]);
    let slice = readback.slice(..);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(deadline.saturating_duration_since(std::time::Instant::now())),
        })
        .expect("GPU work must complete before the aggregate deadline");
    receiver
        .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
        .expect("readback callback must complete before the aggregate deadline")
        .expect("readback mapping must succeed");
    let mapped = slice.get_mapped_range();
    let bytes = mapped[..byte_count].to_vec();
    drop(mapped);
    readback.unmap();
    bytes
}

#[test]
fn upscale_clamps_to_logical_texel_centers_before_sampling_allocation_tail() {
    let (device, queue) = regression_device();
    let layout = super::super::super::bind_group_layouts::upscale(&device);
    let pipeline = super::upscale_pipeline(&device, &layout);
    let source = regression_texture(&device, 8, wgpu::TextureFormat::Rgba8Unorm);
    let output = regression_texture(
        &device,
        4,
        crate::graphics::scene::scene_renderer::post_process::POST_PROCESS_TONEMAPPED_FORMAT,
    );
    let mut source_pixels = [[0_u8, 255, 0, 255]; 8];
    source_pixels[0] = [255, 0, 0, 255];
    source_pixels[1] = [0, 0, 255, 255];
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &source,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&source_pixels),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(32),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 8,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let params = wgpu::util::DeviceExt::create_buffer_init(
        &device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("upscale-logical-two-to-four"),
            contents: bytemuck::cast_slice(&[2_u32, 1, 4, 1]),
            usage: wgpu::BufferUsages::UNIFORM,
        },
    );
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    });
    let source_view = source.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("logical-upscale-regression"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&source_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("production-upscale-regression"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &binding, &[]);
        pass.draw(0..3, 0..1);
    }
    let bytes = read_regression_row(&device, &queue, encoder, &output, 4, 16);
    assert_eq!(&bytes[..4], &[255, 0, 0, 255]);
    assert_eq!(&bytes[8..12], &[0, 0, 255, 255]);
    assert_eq!(&bytes[12..16], &[0, 0, 255, 255]);
    assert!(
        bytes
            .chunks_exact(4)
            .all(|pixel| pixel[1] == 0 && pixel[3] == 255),
        "green allocation-tail poison must never enter the logical output: {bytes:?}"
    );
    assert!((i16::from(bytes[4]) - 128).abs() <= 1);
    assert!((i16::from(bytes[6]) - 128).abs() <= 1);
}
