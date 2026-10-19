use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn post_process_shader_exposes_split_ssr_resolve_entry_point() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_resolve"));
    assert!(POST_PROCESS_SHADER.contains("-> @location(0) vec4<f32>"));
    assert!(POST_PROCESS_SHADER.contains("resolve_screen_space_reflection_history"));
}

fn contract_305_device() -> (wgpu::Device, wgpu::Queue) {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("this rendering regression requires a native GPU adapter");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("render-contract-305"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .expect("this rendering regression requires a native GPU device")
}

fn collect_305_observations(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    output: &wgpu::Buffer,
    size: u64,
) -> Vec<[f32; 4]> {
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("render-contract-305-readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(output, 0, &readback, 0, size);
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
        .expect("GPU submission must finish within the aggregate deadline");
    receiver
        .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
        .expect("GPU readback callback must finish within the aggregate deadline")
        .expect("GPU observation mapping must succeed");
    let mapped = slice.get_mapped_range();
    let observations = mapped
        .chunks_exact(16)
        .map(|row| {
            std::array::from_fn(|i| f32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap()))
        })
        .collect();
    drop(mapped);
    readback.unmap();
    observations
}

#[test]
fn ssr_native_shared_hzb_ranges_nearer_hits_and_orthographic_directions_match_production_contracts()
{
    use crate::graphics::scene::scene_renderer::post_process::params::{
        hzb_params::HzbParams, post_process_params::PostProcessParams,
    };
    let (device, queue) = contract_305_device();
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ssr-actual-depth-input"),
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&Default::default());
    let hzb = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ssr-real-shared-hzb-two-mips"),
        size: wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 2,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let dummy = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("unused-first-level-parent"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let dummy_view = dummy.create_view(&Default::default());
    let hzb_views: Vec<_> = (0..2)
        .map(|mip| {
            hzb.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            })
        })
        .collect();
    let full_hzb_view = hzb.create_view(&Default::default());
    // Only the input-depth raster is test data. Both HZB reductions below use the actual production factory.
    let depth_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ssr-depth-fixture-raster"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
    return vec4<f32>(p[i],0.0,1.0);
}
@fragment fn fs(@builtin(position) p: vec4<f32>) -> @builtin(frag_depth) f32 {
    return select(0.5,0.75,p.x >= 2.0);
}
"#
            .into(),
        ),
    });
    let depth_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ssr-depth-fixture-raster"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &depth_shader,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &depth_shader,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Always),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let hzb_layout = super::super::super::bind_group_layouts::hzb(&device);
    let hzb_pipeline = super::super::hzb_pipeline::hzb_pipeline(&device, &hzb_layout);
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ssr-real-depth-fixture"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&depth_pipeline);
        pass.draw(0..3, 0..1);
    }
    for mip in 0..2 {
        let params = HzbParams {
            target_size: [2 >> mip, 2 >> mip],
            target_mip_level: mip,
            _pad0: 0,
        };
        let uniform = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("actual-hzb-params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let parent = if mip == 0 { &dummy_view } else { &hzb_views[0] };
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("actual-hzb-build-binding"),
            layout: &hzb_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(parent),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&hzb_views[mip as usize]),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("actual-hzb-build"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&hzb_pipeline);
        pass.set_bind_group(0, &binding, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    let source = format!(
        "{}\n{}",
        POST_PROCESS_SHADER,
        r#"
@group(0) @binding(29) var<storage,read_write> contract_305: array<vec4<f32>>;
@compute @workgroup_size(1) fn observe_ssr_contract_305() {
    let base = u32(params.effect_depth.w) * 8u;
    let coarse = load_screen_space_reflection_depth_pyramid_cell_at_mip(vec2<u32>(0u),1u);
    let left = load_screen_space_reflection_depth_pyramid_cell_at_mip(vec2<u32>(0u),0u);
    let right = load_screen_space_reflection_depth_pyramid_cell_at_mip(vec2<u32>(1u,0u),0u);
    contract_305[base] = vec4<f32>(coarse,left);
    contract_305[base+1u] = vec4<f32>(right,
        screen_space_reflection_depth_pyramid_visibility((coarse.x+coarse.y)*0.5,coarse),
        screen_space_reflection_depth_pyramid_visibility(coarse.y+10.0,coarse));
    contract_305[base+2u] = vec4<f32>(
        screen_space_reflection_hit_visibility(8.4,8.4,10.0,2.0,100.0,vec2<f32>(31.5),vec2<u32>(64u)),
        screen_space_reflection_hit_visibility(10.0,10.0,10.0,0.0,100.0,vec2<f32>(31.5),vec2<u32>(64u)),
        screen_space_reflection_hit_visibility(10.0,10.0,10.0,-1.0,100.0,vec2<f32>(31.5),vec2<u32>(64u)),
        screen_space_reflection_hit_visibility(8.4,6.0,10.0,2.0,100.0,vec2<f32>(31.5),vec2<u32>(64u)));
    contract_305[base+3u] = vec4<f32>(screen_space_reflection_view_direction(vec3<f32>(-4.0,2.0,-10.0)),0.0);
    contract_305[base+4u] = vec4<f32>(screen_space_reflection_view_direction(vec3<f32>(4.0,2.0,-10.0)),0.0);
}
"#
    );
    let parsed = naga::front::wgsl::parse_str(&source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&parsed)
    .expect("full production post shader with observation entry must validate");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("full-production-post-ssr-observation"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let observer = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("actual-ssr-helper-observation"),
        layout: None,
        module: &shader,
        entry_point: Some("observe_ssr_contract_305"),
        compilation_options: Default::default(),
        cache: None,
    });
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ssr-observations"),
        size: 256,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    for perspective in [false, true] {
        let mut params: PostProcessParams = bytemuck::Zeroable::zeroed();
        params.viewport_and_clusters = [64, 64, 0, 0];
        params.effect_depth = [1.0, 17.0, 1.0 / 16.0, if perspective { 1.0 } else { 0.0 }];
        params.effect_dither_ssr[3] = 0.1;
        params.effect_ssr_limits = [100.0, 2.0, 0.0, 0.0];
        let uniform = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("actual-post-process-params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssr-real-full-mip-chain-consumer"),
            layout: &observer.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 23,
                    resource: wgpu::BindingResource::TextureView(&full_hzb_view),
                },
                wgpu::BindGroupEntry {
                    binding: 29,
                    resource: output.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("actual-ssr-consumer"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&observer);
        pass.set_bind_group(0, &binding, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    let actual = collect_305_observations(&device, &queue, encoder, &output, 256);
    for (base, near, far) in [(0, 9.0_f32, 13.0_f32), (8, 17.0 / 9.0, 3.4)] {
        for (value, expected) in actual[base].into_iter().zip([near, far, near, near]) {
            assert!((value - expected).abs() <= 1.0e-4, "{actual:?}");
        }
        assert!((actual[base + 1][0] - far).abs() <= 1.0e-4);
        assert!((actual[base + 1][1] - far).abs() <= 1.0e-4);
        assert_eq!(&actual[base + 1][2..], &[1.0, 0.0]);
        assert!(actual[base + 2][0] > 0.97 && actual[base + 2][0] <= 1.0);
        assert_eq!(&actual[base + 2][1..], &[0.0; 3]);
    }
    assert_eq!(&actual[3][..3], &[0.0, 0.0, -1.0]);
    assert_eq!(actual[3], actual[4]);
    assert!(actual[11][0] < -0.3 && actual[12][0] > 0.3);
    for i in [11, 12] {
        let direction = crate::core::math::Vec3::from_array(actual[i][..3].try_into().unwrap());
        assert!(direction.is_finite() && (direction.length() - 1.0).abs() <= 1.0e-5);
    }
}
