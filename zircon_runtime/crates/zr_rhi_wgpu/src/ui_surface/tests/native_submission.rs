use super::*;

use std::sync::mpsc;

use super::super::geometry::ImageVertex;

#[test]
fn wgpu_ui_surface_render_pass_coalesces_contiguous_non_text_ops() {
    let source = include_str!("../render_pass.rs");
    let compact = source.split_whitespace().collect::<String>();

    assert!(
        compact.contains("letrun_end=non_text_run_end(draw_ops,op_index);"),
        "render recording must find the complete contiguous solid/image run"
    );
    assert!(
        compact.contains("forrun_indexinop_index..run_end{"),
        "one render pass must record every op in the contiguous non-text run"
    );
}

#[test]
fn wgpu_ui_surface_marks_the_complete_present_submission_for_renderdoc() {
    let source = include_str!("../presentation.rs");

    assert!(
        source.contains("encoder.push_debug_group(\"zircon::UI\");"),
        "the full UI submission must be grouped under the standard RenderDoc UI marker"
    );
    assert!(
        source.contains("encoder.pop_debug_group();"),
        "the UI debug group must close before command submission"
    );
}

#[test]
fn wgpu_ui_surface_shared_context_path_does_not_request_a_second_device() {
    let source = include_str!("../../ui_surface.rs");
    let renderer_impl = source
        .split("impl WgpuUiSurfaceRenderer {")
        .nth(1)
        .expect("native renderer implementation should remain explicit");
    let shared_constructor = renderer_impl
        .split("fn new_with_context(\n        descriptor: UiSurfaceDescriptor,\n        context: WgpuUiSurfaceContext,")
        .nth(1)
        .and_then(|source| source.split("fn from_surface(").next())
        .expect("native renderer should expose the shared-context construction path");

    assert!(shared_constructor.contains("create_surface(&context.instance, target)?"));
    assert!(!shared_constructor.contains("request_device"));
}

#[test]
fn wgpu_ui_surface_external_image_path_uses_the_shared_texture_without_cpu_upload() {
    let source = include_str!("../image_cache.rs");
    let external_prepare = source
        .split("fn prepare_external_image(")
        .nth(1)
        .and_then(|source| source.split("fn admit(").next())
        .expect("native renderer should prepare external images independently");

    assert!(external_prepare.contains("WgpuUiImageResource::from_external"));
    assert!(!external_prepare.contains("queue.write_texture"));
    assert!(external_prepare.contains("image_payload_layout"));
    assert!(external_prepare.contains("layout.expected_len as u64"));

    let presenter_source = include_str!("../presentation.rs");
    let confirm = presenter_source
        .split("self.image_cache.prepare(")
        .nth(1)
        .expect("native presenter should prepare image resources");
    assert!(confirm.contains("provider.confirm_resident"));
}

#[test]
fn wgpu_ui_surface_copies_shared_products_to_generation_stable_textures() {
    let source = include_str!("../../ui_surface.rs");
    let copy = source
        .split("fn copy_texture_for_external_image")
        .nth(1)
        .expect("shared WGPU context should expose a GPU-only product copy");

    assert!(copy.contains("create_texture"));
    assert!(copy.contains("copy_texture_to_texture"));
    assert!(copy.contains("self.queue.submit"));
    assert!(copy.contains("byte_space_sample_view_format"));
    assert!(copy.contains("view_formats"));
    assert!(copy.contains("WgpuUiExternalImageAlphaMode::Opaque"));
}

#[test]
fn wgpu_ui_surface_samples_srgb_products_as_byte_space_images() {
    assert_eq!(
        byte_space_sample_view_format(wgpu::TextureFormat::Rgba8UnormSrgb),
        Some(wgpu::TextureFormat::Rgba8Unorm)
    );
    assert_eq!(
        byte_space_sample_view_format(wgpu::TextureFormat::Bgra8UnormSrgb),
        Some(wgpu::TextureFormat::Bgra8Unorm)
    );
    assert_eq!(
        byte_space_sample_view_format(wgpu::TextureFormat::Rgba8Unorm),
        None
    );
}

#[test]
fn wgpu_ui_surface_presents_only_after_submission_and_retained_commit() {
    let source = include_str!("../presentation.rs");
    let render = source
        .split("fn render_draw_list_to_surface(")
        .nth(1)
        .and_then(|source| source.split("fn retryable_surface_presentation(").next())
        .expect("native surface render owner");
    let submit = render
        .find("self.queue.submit(Some(encoder.finish()));")
        .expect("native presentation must submit its encoder");
    let commit = render
        .find("RetainedCacheCommit::OrdinaryBaseline =>")
        .expect("retained cache must commit after submission");
    let present = render
        .find("surface_texture.present();")
        .expect("submitted surface texture must be presented");
    let success = render
        .find("Ok(WgpuUiSurfaceRenderStats")
        .expect("submitted presentation must return success");

    assert!(submit < commit);
    assert!(commit < present);
    assert!(present < success);
}

#[test]
fn wgpu_ui_surface_commits_retained_state_only_after_queue_submission() {
    let source = include_str!("../presentation.rs");
    let encode_readback = source
        .find(".encode_copies(&mut encoder, self.present_index)")
        .expect("readback copies must be encoded before submission");
    let submit = source
        .find("self.queue.submit(Some(encoder.finish()));")
        .expect("native presentation must submit its encoder");
    let commit = source
        .find("RetainedCacheCommit::OrdinaryBaseline =>")
        .expect("retained cache state must have an after-submit commit point");

    assert!(encode_readback < submit);
    assert!(submit < commit);
}

#[test]
fn wgpu_ui_surface_acquires_before_advancing_or_preparing_the_frame() {
    let source = include_str!("../presentation.rs");
    let present = source
        .split("pub(super) fn present(")
        .nth(1)
        .and_then(|source| source.split("fn resize_if_needed(").next())
        .expect("native presentation should expose a focused present owner");

    let acquire = present
        .find("self.acquire_surface_texture()?")
        .expect("surface acquisition should be explicit in present");
    let advance = present
        .find("self.present_index = self.present_index.saturating_add(1)")
        .expect("submitted presentation generation should advance explicitly");
    let batch = present
        .find(".compiled_batch_plan")
        .expect("submitted presentation should resolve its batch plan");
    let image = present
        .find("self.image_cache.prepare(")
        .expect("submitted presentation should prepare images");
    let text = present
        .find("self.text.prepare(")
        .expect("submitted presentation should prepare text");

    assert!(acquire < advance);
    assert!(acquire < batch);
    assert!(acquire < image);
    assert!(acquire < text);
}

#[test]
fn wgpu_ui_surface_does_not_fail_a_submitted_frame_when_timing_map_fails() {
    let source = include_str!("../presentation.rs");
    let submitted = source
        .split("self.queue.submit(Some(encoder.finish()));")
        .nth(1)
        .and_then(|source| source.split("Ok(WgpuUiSurfaceRenderStats").next())
        .expect("native presentation should finish submitted-frame bookkeeping");

    assert!(submitted.contains(".begin_map(self.present_index)"));
    assert!(submitted.contains("gpu_readback_queue.abort_frame(self.present_index)"));
    assert!(!submitted.contains("return Err"));
}

#[test]
fn wgpu_ui_surface_setup_production_path_is_panic_free() {
    let source = include_str!("../surface_setup.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(!source.contains(".expect("));
    assert!(!source.contains(".unwrap("));
    assert!(!source.contains("panic!("));
}

#[test]
fn wgpu_ui_linear_sampling_preserves_premultiplied_transparent_edges() {
    let Some((device, queue)) = offscreen_test_device() else {
        return;
    };

    for (source_rgba, expected) in [
        ([0, 0, 0, 0, 0, 0, 128, 128], [0, 0, 64, 64]),
        ([0, 0, 0, 0, 0, 0, 255, 255], [0, 0, 128, 128]),
    ] {
        let actual = render_linear_midpoint(&device, &queue, &source_rgba)
            .expect("offscreen image pipeline should return its midpoint pixel");
        assert_eq!(actual[0], 0, "transparent red must not bleed into the edge");
        assert_eq!(actual[1], 0, "transparent green must remain absent");
        assert_eq!(actual[2], actual[3], "filtered color must track alpha");
        assert!(
            actual[2].abs_diff(expected[2]) <= 1 && actual[3].abs_diff(expected[3]) <= 1,
            "UNORM filtering may round by one byte: actual={actual:?}, expected={expected:?}"
        );
    }
}

fn render_linear_midpoint(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source_rgba: &[u8; 8],
) -> Result<[u8; 4], String> {
    let source = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-ui-premultiplied-filter-source"),
        size: wgpu::Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        source.as_image_copy(),
        source_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
    );

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-ui-premultiplied-filter-target"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
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
    let source_view = source.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group_layout = create_image_bind_group_layout(device);
    let sampler = create_image_sampler(device);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-ui-premultiplied-filter-bind-group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&source_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let pipeline =
        create_image_pipeline(device, wgpu::TextureFormat::Rgba8Unorm, &bind_group_layout);
    let vertices = [
        ImageVertex {
            position: [-1.0, -1.0],
            uv: [0.0, 1.0],
        },
        ImageVertex {
            position: [1.0, -1.0],
            uv: [1.0, 1.0],
        },
        ImageVertex {
            position: [-1.0, 1.0],
            uv: [0.0, 0.0],
        },
        ImageVertex {
            position: [-1.0, 1.0],
            uv: [0.0, 0.0],
        },
        ImageVertex {
            position: [1.0, -1.0],
            uv: [1.0, 1.0],
        },
        ImageVertex {
            position: [1.0, 1.0],
            uv: [1.0, 0.0],
        },
    ];
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ui-premultiplied-filter-vertices"),
        size: std::mem::size_of_val(&vertices) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ui-premultiplied-filter-readback"),
        size: u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-ui-premultiplied-filter-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-ui-premultiplied-filter-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..6, 0..1);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
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

    let slice = readback.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| error.to_string())?;
    receiver
        .recv()
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let mapped = slice.get_mapped_range();
    let pixel = mapped
        .get(..4)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| "mapped WGPU pixel is shorter than four bytes".to_owned())?;
    drop(mapped);
    readback.unmap();
    Ok(pixel)
}

fn offscreen_test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-ui-premultiplied-filter-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
