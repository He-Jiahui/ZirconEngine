use super::*;

use std::sync::mpsc;

use super::super::geometry::{draw_items, DrawItem, ImageVertex, SolidVertex};

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
fn wgpu_ui_surface_reports_the_configured_adapter_and_device_source_once() {
    assert_eq!(
        WgpuUiSurfaceCompletionOwner::Local.device_source_label(),
        "independent"
    );
    assert_eq!(
        WgpuUiSurfaceCompletionOwner::External.device_source_label(),
        "shared"
    );

    let source = include_str!("../../ui_surface.rs");
    let renderer = source
        .split_once("impl WgpuUiSurfaceRenderer {")
        .map(|(_, tail)| tail)
        .expect("native renderer implementation");
    let owned_constructor = renderer
        .split_once("fn new_owned(")
        .and_then(|(_, tail)| tail.split_once("fn new_with_context("))
        .map(|(body, _)| body)
        .expect("owned native surface constructor");
    assert!(owned_constructor.contains("with_local_completion_owner()"));

    let shared_constructor = renderer
        .split_once("fn new_with_context(")
        .and_then(|(_, tail)| tail.split_once("fn from_surface("))
        .map(|(body, _)| body)
        .expect("shared native surface constructor");
    assert!(!shared_constructor.contains("with_local_completion_owner()"));

    let from_surface = renderer
        .split_once("fn from_surface(")
        .and_then(|(_, tail)| tail.split_once("fn resize("))
        .map(|(body, _)| body)
        .expect("common native surface constructor");
    let configured = from_surface
        .split_once("configure_surface(&surface, &context.adapter, &context.device, size)?;")
        .map(|(_, tail)| tail)
        .expect("adapter must configure the actual native surface");
    assert!(configured.contains("context.adapter.get_info()"));
    assert!(configured.contains("context.completion_owner.device_source_label()"));
    assert!(configured.contains("ui_surface_adapter"));
    for field in [
        "device_source=",
        "name=",
        "backend=",
        "vendor_id=",
        "device_id=",
    ] {
        assert!(
            configured.contains(field),
            "missing product adapter field {field}"
        );
    }
    assert_eq!(configured.matches("ui_surface_adapter").count(), 1);
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
    let copy = include_str!("../external_image_copy.rs");

    assert!(copy.contains("create_texture"));
    assert!(copy.contains("copy_texture_to_texture"));
    assert!(copy.contains("begin_native_recording(RenderQueueClass::Graphics)"));
    assert!(copy.contains("submit_native_recording_packet(packet)"));
    assert!(!copy.contains("self.queue.submit"));
    assert!(!copy.contains("device.poll"));
    assert!(copy.contains("WgpuUiExternalImage::new_opaque"));
    assert!(copy.contains("WgpuUiExternalImageCopyReceipt"));
    assert!(copy.contains("WgpuUiExternalImageCopyTarget"));
    assert!(copy.contains("prepare_texture_for_external_image"));
    assert!(copy.contains("target.encode_copy(encoder, source)"));
    assert!(copy.contains("target.complete(submission)"));
    assert!(copy.contains("submission: SubmissionTicket"));
    assert!(!copy.contains("byte_space_sample_view_format"));
}

#[test]
fn wgpu_ui_external_images_sample_through_their_native_transfer_function() {
    let source = include_str!("../../ui_surface.rs");

    assert!(
        !source.contains("byte_space_sample_view_format"),
        "external sRGB products must retain hardware sRGB decode for linear-light composition"
    );
    assert!(!source.contains("sample_view_format"));
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
        .find("self.submit_present_command_buffer(encoder.finish(), image_allocation_pins)?")
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
fn wgpu_ui_surface_commits_retained_state_only_after_owner_submission() {
    let source = include_str!("../presentation.rs");
    let encode_readback = source
        .find(".encode_copies(&mut encoder, self.present_index)")
        .expect("readback copies must be encoded before submission");
    let submit = source
        .find("self.submit_present_command_buffer(encoder.finish(), image_allocation_pins)?")
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
        .split("self.submit_present_command_buffer(encoder.finish(), image_allocation_pins)?")
        .nth(1)
        .and_then(|source| source.split("Ok(WgpuUiSurfaceRenderStats").next())
        .expect("native presentation should finish submitted-frame bookkeeping");

    assert!(submitted.contains(".begin_map(self.present_index)"));
    assert!(submitted.contains("readback_queue.abort_frame(self.present_index)"));
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

#[test]
fn wgpu_ui_rounded_solid_readback_contains_fractional_edge_coverage() {
    let Some((device, queue)) = offscreen_test_device() else {
        return;
    };

    let pixels = render_rounded_solid(&device, &queue)
        .expect("offscreen rounded-solid pipeline should return its pixels");
    let pixel = |x: usize, y: usize| pixels[y * 8 + x];

    assert_eq!(pixel(0, 0), [0, 0, 0, 0], "outside must remain clear");
    assert_eq!(pixel(4, 4), [255, 255, 255, 255], "center must stay opaque");

    let fractional = pixels
        .iter()
        .filter(|pixel| pixel[3] > 0 && pixel[3] < 255)
        .collect::<Vec<_>>();
    assert!(
        !fractional.is_empty(),
        "analytic rounded coverage must produce fractional edge pixels: {pixels:?}"
    );
    assert!(
        fractional.iter().all(|pixel| {
            pixel[0].abs_diff(pixel[3]) <= 1
                && pixel[1].abs_diff(pixel[3]) <= 1
                && pixel[2].abs_diff(pixel[3]) <= 1
        }),
        "white premultiplied edge RGB must track coverage alpha: {fractional:?}"
    );
}

#[test]
fn wgpu_ui_rounded_box_readback_does_not_double_cover_the_outer_edge() {
    let Some((device, queue)) = offscreen_test_device() else {
        return;
    };

    let frame = UiSurfaceRect::new(1.0, 1.0, 6.0, 6.0);
    let fill = UiSurfaceCommandKind::Quad {
        color: [255, 255, 255, 255],
        corner_radius: 3.0,
    };
    let fill_pixels = render_analytic_solid(&device, &queue, frame, fill.clone())
        .expect("single rounded fill should render");
    let box_pixels = render_analytic_solid_commands(
        &device,
        &queue,
        frame,
        vec![
            fill,
            UiSurfaceCommandKind::Border {
                color: [255, 255, 255, 255],
                width: 1.0,
                corner_radius: 3.0,
            },
        ],
    )
    .expect("combined rounded box should render as one analytic item");

    assert_eq!(
        box_pixels, fill_pixels,
        "same-color fill and border must preserve one outer coverage instead of blending it twice"
    );
}

#[test]
fn wgpu_ui_rounded_box_readback_partitions_fill_and_border_colors() {
    let Some((device, queue)) = offscreen_test_device() else {
        return;
    };

    let pixels = render_analytic_solid_commands(
        &device,
        &queue,
        UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0),
        vec![
            UiSurfaceCommandKind::Quad {
                color: [255, 0, 0, 255],
                corner_radius: 2.0,
            },
            UiSurfaceCommandKind::Border {
                color: [0, 255, 0, 255],
                width: 2.0,
                corner_radius: 2.0,
            },
        ],
    )
    .expect("distinct rounded fill and border should render as one analytic item");
    let pixel = |x: usize, y: usize| pixels[y * 8 + x];

    assert_eq!(pixel(3, 3), [255, 0, 0, 255], "center uses fill color");
    assert_eq!(
        pixel(3, 1),
        [0, 255, 0, 255],
        "ring interior uses border color"
    );
    let outer_corner = pixel(0, 0);
    assert!(
        outer_corner[3] > 0 && outer_corner[3] < 255,
        "rounded outer corner must retain fractional coverage: {outer_corner:?}"
    );
    assert_eq!(outer_corner[0], 0, "outer coverage must not leak fill red");
    assert_eq!(outer_corner[2], 0, "outer coverage must not leak fill blue");
    assert!(
        outer_corner[1].abs_diff(outer_corner[3]) <= 1,
        "opaque green border must remain premultiplied at the outer edge: {outer_corner:?}"
    );
}

#[test]
fn wgpu_ui_fractional_square_border_readback_preserves_subpixel_width() {
    let Some((device, queue)) = offscreen_test_device() else {
        return;
    };

    let frame = UiSurfaceRect::new(1.25, 1.25, 5.5, 5.5);
    let render = |width| {
        render_analytic_solid(
            &device,
            &queue,
            frame,
            UiSurfaceCommandKind::Border {
                color: [255, 255, 255, 255],
                width,
                corner_radius: 0.0,
            },
        )
    };
    let thin = render(0.625).expect("fractional square border should render");
    let one_pixel = render(1.0).expect("one-pixel square border should render");
    let alpha_sum =
        |pixels: &[[u8; 4]]| pixels.iter().map(|pixel| u32::from(pixel[3])).sum::<u32>();

    assert!(
        thin.iter().any(|pixel| (1..=254).contains(&pixel[3])),
        "a 0.625-pixel square border must retain fractional coverage: {thin:?}"
    );
    assert_eq!(
        thin[4 * 8 + 4],
        [0, 0, 0, 0],
        "the analytic square outline must not fill its center"
    );
    assert!(
        alpha_sum(&thin) < alpha_sum(&one_pixel),
        "the fractional border must not be clamped to a one-pixel outline: thin={thin:?}, one_pixel={one_pixel:?}"
    );
}

#[test]
fn wgpu_ui_srgb_target_encodes_linear_light_alpha_at_an_interior_sample() {
    let (device, queue) = offscreen_test_device()
        .expect("the managed sRGB qualification requires an offscreen WGPU device");

    let pixel = render_flat_solid_sample(
        &device,
        &queue,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        [1.0, 1.0, 1.0, 0.5],
    )
    .expect("offscreen sRGB solid pipeline should return its pixel");

    assert!(
        (187..=189).contains(&pixel[0])
            && (187..=189).contains(&pixel[1])
            && (187..=189).contains(&pixel[2]),
        "linear 50% white must encode near sRGB 188 instead of gamma-space 128: {pixel:?}"
    );
    assert!((127..=128).contains(&pixel[3]));
}

fn render_flat_solid_sample(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_format: wgpu::TextureFormat,
    color: [f32; 4],
) -> Result<[u8; 4], String> {
    const PADDED_BYTES_PER_ROW: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-ui-linear-light-solid-target"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let pipeline = create_solid_pipeline(device, target_format);
    let vertex = |position| SolidVertex {
        position,
        color,
        local_position: position,
        // Keep the analytic shape well beyond the single-pixel target so this test isolates
        // linear-light color encoding from the rounded-coverage derivative at a 1x1 edge.
        half_extent: [4.0, 4.0],
        corner_radius: 0.0,
        border_width: 0.0,
        fill_color: [0.0; 4],
    };
    let vertices = [
        vertex([-1.0, 1.0]),
        vertex([1.0, 1.0]),
        vertex([-1.0, -1.0]),
        vertex([-1.0, -1.0]),
        vertex([1.0, 1.0]),
        vertex([1.0, -1.0]),
    ];
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ui-linear-light-solid-vertices"),
        size: std::mem::size_of_val(&vertices) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ui-linear-light-solid-readback"),
        size: u64::from(PADDED_BYTES_PER_ROW),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-ui-linear-light-solid-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-ui-linear-light-solid-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..6, 0..1);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(PADDED_BYTES_PER_ROW),
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
    let pixel = mapped[..4]
        .try_into()
        .expect("linear-light readback pixel is exactly four bytes");
    drop(mapped);
    readback.unmap();
    Ok(pixel)
}

#[test]
fn wgpu_ui_rounded_solid_readback_preserves_subpixel_positioning() {
    let Some((device, queue)) = offscreen_test_device() else {
        return;
    };

    let aligned = render_rounded_solid_at(&device, &queue, [1.0, 1.0])
        .expect("aligned rounded solid should render");
    let shifted = render_rounded_solid_at(&device, &queue, [1.25, 1.25])
        .expect("fractionally shifted rounded solid should render");
    let late_phase = render_rounded_solid_at(&device, &queue, [1.75, 1.75])
        .expect("late-phase rounded solid should render");

    assert_ne!(
        aligned, shifted,
        "a quarter-pixel translation must reach the analytic shader without integer snapping"
    );
    for (label, pixels) in [
        ("aligned", aligned),
        ("shifted", shifted),
        ("late_phase", late_phase.clone()),
    ] {
        assert!(
            pixels.iter().any(|pixel| pixel[3] > 0 && pixel[3] < 255),
            "{label} rounded edges must retain fractional coverage: {pixels:?}"
        );
    }
    assert!(
        late_phase[4 * 8 + 1][3] > 0,
        "the analytic raster envelope must shade the first partially covered pixel outside the shape bounds: {late_phase:?}"
    );
}

fn render_rounded_solid(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<Vec<[u8; 4]>, String> {
    render_rounded_solid_at(device, queue, [1.0, 1.0])
}

fn render_rounded_solid_at(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    origin: [f32; 2],
) -> Result<Vec<[u8; 4]>, String> {
    render_analytic_solid(
        device,
        queue,
        UiSurfaceRect::new(origin[0], origin[1], 6.0, 6.0),
        UiSurfaceCommandKind::Quad {
            color: [255, 255, 255, 255],
            corner_radius: 3.0,
        },
    )
}

fn render_analytic_solid(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame: UiSurfaceRect,
    kind: UiSurfaceCommandKind,
) -> Result<Vec<[u8; 4]>, String> {
    render_analytic_solid_commands(device, queue, frame, vec![kind])
}

fn render_analytic_solid_commands(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame: UiSurfaceRect,
    kinds: Vec<UiSurfaceCommandKind>,
) -> Result<Vec<[u8; 4]>, String> {
    const EDGE: u32 = 8;
    const PADDED_BYTES_PER_ROW: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-ui-analytic-solid-target"),
        size: wgpu::Extent3d {
            width: EDGE,
            height: EDGE,
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
    let pipeline = create_solid_pipeline(device, wgpu::TextureFormat::Rgba8Unorm);
    let commands = kinds
        .into_iter()
        .enumerate()
        .map(|(index, kind)| UiSurfaceCommand {
            z_index: index as i32,
            frame,
            clip: None,
            kind,
        })
        .collect();
    let draw_list = UiSurfaceDrawList::new((EDGE, EDGE), None, commands);
    let mut solid_items = draw_items(&draw_list)
        .into_iter()
        .filter_map(|item| match item {
            DrawItem::Solid(item) => Some(item),
            DrawItem::Image(_) | DrawItem::Text(_) => None,
        })
        .collect::<Vec<_>>();
    if solid_items.len() != 1 {
        return Err(format!(
            "analytic readback requires one fused solid item, got {}",
            solid_items.len()
        ));
    }
    let vertices = solid_items
        .pop()
        .expect("one solid item was checked above")
        .vertices()
        .to_vec();
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ui-analytic-solid-vertices"),
        size: (vertices.len() * std::mem::size_of_val(&vertices[0])) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(vertices.as_slice()));

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-ui-analytic-solid-readback"),
        size: u64::from(PADDED_BYTES_PER_ROW) * u64::from(EDGE),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-ui-analytic-solid-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-ui-analytic-solid-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(PADDED_BYTES_PER_ROW),
                rows_per_image: Some(EDGE),
            },
        },
        wgpu::Extent3d {
            width: EDGE,
            height: EDGE,
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
    let mut pixels = Vec::with_capacity((EDGE * EDGE) as usize);
    for y in 0..EDGE as usize {
        let row_start = y * PADDED_BYTES_PER_ROW as usize;
        for x in 0..EDGE as usize {
            let offset = row_start + x * 4;
            pixels.push(
                mapped[offset..offset + 4]
                    .try_into()
                    .expect("analytic readback pixel is exactly four bytes"),
            );
        }
    }
    drop(mapped);
    readback.unmap();
    Ok(pixels)
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

#[test]
fn normal_readback_preserves_the_actual_surface_encoder_ticket_and_image_pins() {
    let owner = include_str!("../../ui_surface.rs");
    let render = include_str!("../presentation.rs");
    let readback = include_str!("../frame_readback.rs");
    let prepare = render
        .find("self.prepare_frame_readback(draw_list, &mut encoder)")
        .expect("same normal encoder must prepare its readback tail");
    let diagnostic_submit = render
        .find("self.submit_present_command_buffer_with_diagnostics(")
        .expect("normal readback must submit its actual packet");
    let ordinary_submit = render
        .find("self.submit_present_command_buffer(encoder.finish(), image_allocation_pins)?")
        .expect("ordinary normal present must retain its existing packet owner");
    let commit = render
        .find("retained_cache.mark_ordinary_baseline_ready()")
        .expect("retained baseline must commit only after actual submission");
    let present = render
        .find("surface_texture.present()")
        .expect("actual window texture must present after successful owner submission");
    assert!(
        prepare < diagnostic_submit
            && diagnostic_submit < commit
            && ordinary_submit < commit
            && commit < present
    );
    let branch = owner
        .split("fn submit_present_command_buffer_with_diagnostics(")
        .nth(1)
        .expect("typed frame owner must be explicit");
    assert!(branch.contains("packet.retain_ui_image_pins(pins)"));
    assert!(
        branch.contains(".submit_native_recording_packet_with_diagnostics(packet, Some(frame))")
    );
    assert_eq!(
        branch
            .matches("abort_prepared_native_diagnostic_readback_frame(frame)")
            .count(),
        2,
        "both construction failures must terminate their owned unbound frame"
    );
    assert!(!branch.contains(".queue.submit("));
    let prepare_owner = readback
        .split_once("pub(super) fn prepare_frame_readback(")
        .unwrap()
        .1;
    let register = prepare_owner
        .find("pending.window.register(owner)")
        .unwrap();
    let register_image = prepare_owner.find("image.slot.register(owner)").unwrap();
    let encode = prepare_owner
        .find(".prepare_native_diagnostic_readback_frame(")
        .unwrap();
    assert!(
        register < encode && register_image < encode,
        "the existing delivery router must own the request before encoding/submission"
    );
    assert!(!readback.contains("abort_diagnostic_readback_frame("));
}

fn viewport_media_request(image: bool) -> zr_rhi::UiSurfaceFrameReadbackRequest {
    zr_rhi::UiSurfaceFrameReadbackRequest {
        nonce: "scene-only-test".into(),
        physical_size: (3, 2),
        expected_draw_generation: None,
        deadline_ms: 5000,
        image: image.then(|| zr_rhi::UiSurfaceImageReadbackRequest {
            command_index: 1,
            resource_key: "viewport:1:1".into(),
            resource_generation: 7,
            physical_size: (3, 2),
            expected_pixel_format: None,
        }),
    }
}

fn viewport_media_draw_list() -> UiSurfaceDrawList {
    UiSurfaceDrawList::with_compact_styles(
        (3, 2),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 3.0, 2.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [10, 20, 30, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(0.0, 0.0, 3.0, 2.0),
                clip: Some(UiSurfaceRect::new(1.0, 0.5, 1.0, 1.0)),
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "viewport:1:1".into(),
                        resource_generation: 7,
                        width: 3,
                        height: 2,
                        upload_bytes: 0,
                        rgba: None,
                        atlas_uv: None,
                    },
                },
            },
        ],
    )
}

fn viewport_media_tracker(max_requests: usize) -> zr_rhi::DiagnosticReadbackTracker {
    zr_rhi::DiagnosticReadbackTracker::new(
        zr_rhi::DeviceId::new(451),
        zr_rhi::DeviceGeneration::initial(),
        zr_rhi::DiagnosticReadbackBudget::new(max_requests, 4, 4096, 8192, 8192, 8),
    )
}

fn viewport_media_ticket(sequence: u64) -> zr_rhi::SubmissionTicket {
    zr_rhi::SubmissionTicket::new(
        zr_rhi::DeviceId::new(451),
        zr_rhi::DeviceGeneration::initial(),
        zr_rhi::RenderQueueClass::Graphics,
        sequence,
    )
}

fn viewport_media_admitted(
    admission: zr_rhi::DiagnosticReadbackAdmission,
) -> zr_rhi::DiagnosticReadbackRequestId {
    let zr_rhi::DiagnosticReadbackAdmission::Admitted(id) = admission else {
        panic!("expected admission");
    };
    id
}

#[test]
fn normal_viewport_media_command_requires_exact_index_resource_generation_extent_and_full_uv() {
    use super::super::frame_readback::image_command_geometry;
    let draw_list = viewport_media_draw_list();
    let request = viewport_media_request(true).image.unwrap();
    assert_eq!(
        image_command_geometry(&draw_list, &request).unwrap(),
        (draw_list.commands[1].frame, draw_list.commands[1].clip)
    );
    for wrong in [
        zr_rhi::UiSurfaceImageReadbackRequest {
            command_index: 0,
            ..request.clone()
        },
        zr_rhi::UiSurfaceImageReadbackRequest {
            command_index: 2,
            ..request.clone()
        },
        zr_rhi::UiSurfaceImageReadbackRequest {
            resource_key: "viewport:2:1".into(),
            ..request.clone()
        },
        zr_rhi::UiSurfaceImageReadbackRequest {
            resource_generation: 8,
            ..request.clone()
        },
        zr_rhi::UiSurfaceImageReadbackRequest {
            physical_size: (6, 2),
            ..request.clone()
        },
    ] {
        assert!(image_command_geometry(&draw_list, &wrong).is_err());
    }
    let mut atlas = draw_list.clone();
    let UiSurfaceCommandKind::Image { payload } = &mut atlas.commands[1].kind else {
        unreachable!()
    };
    payload.atlas_uv = Some(UiSurfaceImageUvRect {
        min: [0.0, 0.0],
        max: [1.0, 1.0],
    });
    assert!(
        image_command_geometry(&atlas, &request).is_err(),
        "even identity UV must be explicitly whole-resource"
    );
}

#[test]
fn normal_viewport_media_command_rejects_hidden_invalid_or_cpu_backed_resources() {
    use super::super::frame_readback::image_command_geometry;
    let request = viewport_media_request(true).image.unwrap();
    for frame in [
        UiSurfaceRect::new(20.0, 20.0, 6.0, 4.0),
        UiSurfaceRect::new(f32::NAN, 3.0, 6.0, 4.0),
        UiSurfaceRect::new(2.0, 3.0, 0.0, 4.0),
    ] {
        let mut draw_list = viewport_media_draw_list();
        draw_list.commands[1].frame = frame;
        assert!(image_command_geometry(&draw_list, &request).is_err());
    }
    let mut draw_list = viewport_media_draw_list();
    draw_list.commands[1].clip = Some(UiSurfaceRect::new(10.0, 8.0, 1.0, 1.0));
    assert!(image_command_geometry(&draw_list, &request).is_err());
    let mut draw_list = viewport_media_draw_list();
    let UiSurfaceCommandKind::Image { payload } = &mut draw_list.commands[1].kind else {
        unreachable!()
    };
    payload.rgba = Some(vec![255; 24]);
    assert!(image_command_geometry(&draw_list, &request).is_err());
    let compact =
        UiSurfaceDrawList::with_compact_styles(draw_list.surface_size, None, draw_list.commands);
    assert!(compact
        .image_resource(&request.resource_key, request.resource_generation)
        .is_some());
    assert!(
        image_command_geometry(&compact, &request).is_err(),
        "compacting CPU bytes cannot make them an external resource"
    );
}

#[test]
fn normal_viewport_media_texture_reports_actual_format_alpha_and_rejects_unreadable_generations() {
    use super::super::frame_readback::image_texture_format;
    let Some((device, _queue)) = offscreen_test_device() else {
        return;
    };
    let request = viewport_media_request(true).image.unwrap();
    for (format, expected) in [
        (
            wgpu::TextureFormat::Rgba8UnormSrgb,
            zr_rhi::UiSurfaceFramePixelFormat::Rgba8Srgb,
        ),
        (
            wgpu::TextureFormat::Bgra8UnormSrgb,
            zr_rhi::UiSurfaceFramePixelFormat::Bgra8Srgb,
        ),
        (
            wgpu::TextureFormat::Rgba8Unorm,
            zr_rhi::UiSurfaceFramePixelFormat::Rgba8Linear,
        ),
        (
            wgpu::TextureFormat::Bgra8Unorm,
            zr_rhi::UiSurfaceFramePixelFormat::Bgra8Linear,
        ),
    ] {
        let texture = viewport_media_texture(&device, format, wgpu::TextureUsages::COPY_SRC);
        let image = WgpuUiExternalImage::new_premultiplied(texture.clone(), 3, 2, 7);
        assert_eq!(
            image_texture_format(&image, &request).unwrap(),
            (expected, zr_rhi::UiSurfaceImageAlphaMode::Premultiplied)
        );
        let opaque = WgpuUiExternalImage::new_opaque(texture.clone(), 3, 2, 7);
        assert_eq!(
            image_texture_format(&opaque, &request).unwrap().1,
            zr_rhi::UiSurfaceImageAlphaMode::Opaque
        );
        let wrong = zr_rhi::UiSurfaceImageReadbackRequest {
            expected_pixel_format: Some(
                if expected == zr_rhi::UiSurfaceFramePixelFormat::Rgba8Srgb {
                    zr_rhi::UiSurfaceFramePixelFormat::Bgra8Linear
                } else {
                    zr_rhi::UiSurfaceFramePixelFormat::Rgba8Srgb
                },
            ),
            ..request.clone()
        };
        assert!(image_texture_format(&image, &wrong).is_err());
        assert!(image_texture_format(
            &WgpuUiExternalImage::new_opaque(texture.clone(), 3, 2, 8),
            &request
        )
        .is_err());
        assert!(
            image_texture_format(&WgpuUiExternalImage::new_opaque(texture, 2, 2, 7), &request)
                .is_err()
        );
    }
    for (format, usage) in [
        (
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
        ),
        (
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::COPY_SRC,
        ),
    ] {
        let image = WgpuUiExternalImage::new_opaque(
            viewport_media_texture(&device, format, usage),
            3,
            2,
            7,
        );
        assert!(image_texture_format(&image, &request).is_err());
    }
}

#[test]
fn normal_viewport_media_none_preserves_window_only_bytes_and_submission() {
    use super::super::frame_readback::PendingFrameReadback;
    let mut tracker = viewport_media_tracker(4);
    tracker.begin_frame(1).unwrap();
    let admission = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let id = viewport_media_admitted(admission);
    let mut pending = PendingFrameReadback::new(viewport_media_request(false), 12);
    pending.prepare_window(zr_rhi::UiSurfaceFramePixelFormat::Bgra8Linear, 19);
    assert!(pending.window.admit(admission));
    let ticket = viewport_media_ticket(1);
    tracker.bind_active_frame(ticket).unwrap();
    let receipt = tracker
        .terminalize(id, zr_rhi::DiagnosticReadbackTerminal::Succeeded)
        .unwrap();
    let bytes = [1, 2, 3, 255].repeat(6);
    pending.window.deliver(receipt, Some(bytes.clone()));
    assert!(
        !pending.ready(),
        "delivered bytes cannot substitute for the actual bound ticket"
    );
    pending.bind_submission(ticket);
    assert!(pending.ready());
    let result = pending.finish("Opaque".into(), None);
    assert_eq!(result.submission, Some(ticket));
    assert_eq!(result.present_index, 19);
    assert_eq!(result.surface_epoch, 12);
    assert_eq!(result.color8, Some(bytes));
    assert_eq!(
        result.pixel_format,
        Some(zr_rhi::UiSurfaceFramePixelFormat::Bgra8Linear)
    );
    assert!(result.image.is_none() && result.failure.is_none());
}

#[test]
fn normal_viewport_media_partial_admission_still_binds_and_waits_for_admitted_window() {
    use super::super::frame_readback::PendingFrameReadback;
    let mut tracker = viewport_media_tracker(1);
    tracker.begin_frame(1).unwrap();
    let first = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let id = viewport_media_admitted(first);
    let rejected = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let mut pending = PendingFrameReadback::new(viewport_media_request(true), 1);
    assert!(pending.window.admit(first));
    assert!(!pending.image.as_mut().unwrap().slot.admit(rejected));
    pending.fail("image readback quota rejected".into());
    assert!(
        !pending.ready(),
        "partial failure must not retire unbound admitted work"
    );
    let ticket = viewport_media_ticket(1);
    tracker.bind_active_frame(ticket).unwrap();
    pending.bind_submission(ticket);
    assert!(
        !pending.ready(),
        "partial failure must still await the admitted receipt"
    );
    let receipt = tracker
        .terminalize(id, zr_rhi::DiagnosticReadbackTerminal::Succeeded)
        .unwrap();
    pending.window.deliver(receipt, Some(vec![255; 24]));
    assert!(pending.ready());
    let result = pending.finish("Opaque".into(), None);
    assert_eq!(result.submission, Some(ticket));
    assert_eq!(result.receipt, Some(receipt));
    let image = result.image.unwrap();
    assert_eq!(
        image.receipt.unwrap().terminal(),
        zr_rhi::DiagnosticReadbackTerminal::OverBudget
    );
    assert!(result.failure.is_some() && result.color8.is_none() && image.color8.is_none());
}

#[test]
fn normal_viewport_media_both_admitted_require_both_terminal_deliveries() {
    use super::super::frame_readback::PendingFrameReadback;
    let mut tracker = viewport_media_tracker(4);
    tracker.begin_frame(1).unwrap();
    let first = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let second = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let mut pending = PendingFrameReadback::new(viewport_media_request(true), 1);
    pending.window.admit(first);
    pending.image.as_mut().unwrap().slot.admit(second);
    let ticket = viewport_media_ticket(1);
    tracker.bind_active_frame(ticket).unwrap();
    pending.bind_submission(ticket);
    let receipt = tracker
        .terminalize(
            viewport_media_admitted(first),
            zr_rhi::DiagnosticReadbackTerminal::Succeeded,
        )
        .unwrap();
    pending.window.deliver(receipt, Some(vec![255; 24]));
    assert!(!pending.ready());
    let failed = tracker
        .terminalize(
            viewport_media_admitted(second),
            zr_rhi::DiagnosticReadbackTerminal::MapFailed,
        )
        .unwrap();
    pending.image.as_ref().unwrap().slot.deliver(failed, None);
    assert!(pending.ready());
    let result = pending.finish("Opaque".into(), None);
    assert_eq!(result.image.as_ref().unwrap().receipt, Some(failed));
    assert!(result.failure.is_some() && result.color8.is_none());
}

#[test]
fn normal_viewport_media_rejects_cross_request_cross_submission_and_wrong_byte_extent() {
    use super::super::frame_readback::PendingFrameReadback;
    for defect in ["request", "submission", "extent"] {
        let mut tracker = viewport_media_tracker(4);
        tracker.begin_frame(1).unwrap();
        let first = tracker
            .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
            .unwrap();
        let ticket = viewport_media_ticket(1);
        tracker.bind_active_frame(ticket).unwrap();
        let window = tracker
            .terminalize(
                viewport_media_admitted(first),
                zr_rhi::DiagnosticReadbackTerminal::Succeeded,
            )
            .unwrap();
        tracker.begin_frame(2).unwrap();
        let second = tracker
            .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
            .unwrap();
        tracker
            .bind_active_frame(if defect == "submission" {
                viewport_media_ticket(2)
            } else {
                ticket
            })
            .unwrap();
        let image = tracker
            .terminalize(
                viewport_media_admitted(second),
                zr_rhi::DiagnosticReadbackTerminal::Succeeded,
            )
            .unwrap();
        let mut pending = PendingFrameReadback::new(viewport_media_request(true), 1);
        pending.window.admit(first);
        pending.image.as_mut().unwrap().slot.admit(second);
        pending.bind_submission(ticket);
        pending.window.deliver(window, Some(vec![255; 24]));
        pending.image.as_ref().unwrap().slot.deliver(
            if defect == "request" { window } else { image },
            Some(vec![255; if defect == "extent" { 512 } else { 24 }]),
        );
        assert!(pending.ready());
        let result = pending.finish("Opaque".into(), None);
        let failure = result.image.as_ref().unwrap().failure.as_deref().unwrap();
        assert!(
            failure.contains(if defect == "extent" {
                "payload extent"
            } else {
                "receipt"
            }),
            "{defect}: {failure}"
        );
        assert!(
            result.failure.is_some()
                && result.color8.is_none()
                && result.image.unwrap().color8.is_none()
        );
    }
}

#[derive(Default)]
struct ViewportMediaCompletionOwner {
    callbacks: std::sync::Mutex<
        HashMap<zr_rhi::DiagnosticReadbackRequestId, WgpuUiSurfaceDiagnosticReceiptCallback>,
    >,
    cancelled: std::sync::Mutex<Vec<zr_rhi::DiagnosticReadbackRequestId>>,
}
impl WgpuUiSurfaceDiagnosticCompletionOwner for ViewportMediaCompletionOwner {
    fn register_receipt_callback(
        &self,
        admission: zr_rhi::DiagnosticReadbackAdmission,
        callback: WgpuUiSurfaceDiagnosticReceiptCallback,
    ) -> Result<bool, zr_rhi::RhiError> {
        self.callbacks
            .lock()
            .unwrap()
            .insert(viewport_media_admitted(admission), callback);
        Ok(true)
    }
    fn cancel_receipt_callback(&self, id: zr_rhi::DiagnosticReadbackRequestId) {
        self.callbacks.lock().unwrap().remove(&id);
        self.cancelled.lock().unwrap().push(id);
    }
    fn drain_once(&self) -> Result<zr_rhi::SubmissionPollReceipt, zr_rhi::RhiError> {
        unreachable!("cancellation does not poll or cancel native GPU work")
    }
}

#[test]
fn normal_viewport_media_cancel_detaches_both_routes_without_fabricating_native_terminals() {
    use super::super::frame_readback::PendingFrameReadback;
    let mut tracker = viewport_media_tracker(4);
    tracker.begin_frame(1).unwrap();
    let first = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let second = tracker
        .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Texture, 512)
        .unwrap();
    let mut pending = PendingFrameReadback::new(viewport_media_request(true), 1);
    pending.window.admit(first);
    pending.image.as_mut().unwrap().slot.admit(second);
    let owner = ViewportMediaCompletionOwner::default();
    pending.window.register(&owner).unwrap();
    pending
        .image
        .as_ref()
        .unwrap()
        .slot
        .register(&owner)
        .unwrap();
    assert_eq!(owner.callbacks.lock().unwrap().len(), 2);
    let ticket = viewport_media_ticket(1);
    let frame_key = tracker.bind_active_frame(ticket).unwrap();
    pending.bind_submission(ticket);
    pending.cancel("actual surface resized", Some(&owner));
    assert!(pending.ready());
    assert!(owner.callbacks.lock().unwrap().is_empty());
    assert_eq!(
        *owner.cancelled.lock().unwrap(),
        vec![
            viewport_media_admitted(first),
            viewport_media_admitted(second)
        ]
    );
    let result = pending.finish("Opaque".into(), None);
    assert_eq!(result.submission, Some(ticket));
    assert!(
        result.failure.is_some()
            && result.receipt.is_none()
            && result.image.unwrap().receipt.is_none()
    );
    assert_eq!(
        tracker
            .terminalize_frame(frame_key, zr_rhi::DiagnosticReadbackTerminal::Cancelled)
            .len(),
        2,
        "callback detachment leaves native lifecycle terminalization with its original owner"
    );
}

fn viewport_media_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("normal-viewport-media-test"),
        size: wgpu::Extent3d {
            width: 3,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn viewport_media_device(
    budget: zr_rhi::DiagnosticReadbackBudget,
) -> Option<Arc<crate::WgpuRenderDevice>> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let request = crate::wgpu_device_request(
        adapter.features(),
        &zr_rhi::RenderDeviceRequestPolicy::mvp_baseline(),
    )
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("normal-viewport-media-owner-test"),
        required_features: request.requested_features(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()?;
    let profile = zr_rhi::RenderDeviceProfile::new(
        crate::next_wgpu_device_id(),
        zr_rhi::DeviceGeneration::initial(),
        crate::wgpu_adapter_facts(&adapter.get_info(), adapter.features()),
        request.feature_negotiation().clone(),
        crate::wgpu_device_limits(&device.limits()),
        zr_rhi::RenderDeviceQueueTopology::single_serialized_queue(),
        zr_rhi::RenderDeviceBudgets {
            memory_budget: zr_rhi::GpuMemoryBudget::reference_1080p_mid(),
            submission_limits: zr_rhi::SubmissionLimits::default(),
            diagnostic_readback_budget: budget,
        },
    );
    Some(Arc::new(
        crate::WgpuRenderDevice::new(
            crate::WgpuRenderDeviceContext::new(instance, adapter, device, queue),
            profile,
        )
        .unwrap(),
    ))
}

fn viewport_media_native_batch(reject_image: bool) {
    use super::super::frame_readback::PendingFrameReadback;
    use zr_rhi::RenderDevice;
    let budget = zr_rhi::DiagnosticReadbackBudget::new(
        if reject_image { 1 } else { 4 },
        8,
        4096,
        8192,
        16384,
        8,
    );
    let Some(device) = viewport_media_device(budget) else {
        return;
    };
    let context = device.ui_surface_context();
    let usage = wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST;
    let window = viewport_media_texture(&context.device, wgpu::TextureFormat::Rgba8Unorm, usage);
    let scene = viewport_media_texture(&context.device, wgpu::TextureFormat::Bgra8UnormSrgb, usage);
    let window_bytes = [11, 22, 33, 255].repeat(6);
    let scene_bytes = [
        3, 20, 80, 255, 5, 30, 90, 255, 7, 40, 100, 255, 9, 50, 110, 255, 12, 60, 120, 255, 15, 70,
        130, 255,
    ]
    .to_vec();
    for (texture, bytes) in [(&window, &window_bytes), (&scene, &scene_bytes)] {
        context.queue.write_texture(
            texture.as_image_copy(),
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(12),
                rows_per_image: Some(2),
            },
            wgpu::Extent3d {
                width: 3,
                height: 2,
                depth_or_array_layers: 1,
            },
        );
    }
    let external = context
        .copy_texture_for_external_image(&scene, 3, 2, scene.format(), 7)
        .unwrap()
        .into_image();
    let mut pending = PendingFrameReadback::new(viewport_media_request(true), 12);
    pending.prepare_window(zr_rhi::UiSurfaceFramePixelFormat::Rgba8Linear, 29);
    pending
        .image
        .as_mut()
        .unwrap()
        .prepare(&viewport_media_draw_list(), external.clone())
        .unwrap();
    device.begin_diagnostic_readback_frame(29).unwrap();
    assert!(pending.window.admit(
        device
            .enqueue_native_diagnostic_texture_color8_readback(&window, 3, 2)
            .unwrap()
    ));
    let admission = device
        .enqueue_native_diagnostic_texture_color8_readback(&external.texture, 3, 2)
        .unwrap();
    assert_eq!(
        pending.image.as_mut().unwrap().slot.admit(admission),
        !reject_image
    );
    if reject_image {
        pending.fail("image readback quota rejected".into());
    }
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("normal-viewport-media-same-encoder"),
        });
    let frame = device
        .prepare_native_diagnostic_readback_frame("normal-viewport-media-same-batch", &mut encoder)
        .unwrap();
    assert!(
        frame.is_some(),
        "partial failure must keep the admitted window's submission"
    );
    let mut recording = device
        .begin_native_recording(zr_rhi::RenderQueueClass::Graphics)
        .unwrap();
    recording.extend_recorded_command_buffers([encoder.finish()]);
    let ticket = device
        .submit_native_recording_packet_with_diagnostics(recording.finish().unwrap(), frame)
        .unwrap();
    pending.bind_submission(ticket);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        device.poll_submissions().unwrap();
        pending.collect_local(&device);
        if pending.ready() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "both admitted readbacks must reach actual terminal delivery"
        );
        std::thread::yield_now();
    }
    let result = pending.finish("Opaque".into(), None);
    assert_eq!(result.submission, Some(ticket));
    let receipt = result.receipt.unwrap();
    assert_eq!(
        receipt.terminal(),
        zr_rhi::DiagnosticReadbackTerminal::Succeeded
    );
    assert_eq!(receipt.frame_key().unwrap().submission(), ticket);
    let image = result.image.unwrap();
    if reject_image {
        assert!(result.failure.is_some() && result.color8.is_none() && image.color8.is_none());
        assert_eq!(
            image.receipt.unwrap().terminal(),
            zr_rhi::DiagnosticReadbackTerminal::OverBudget
        );
    } else {
        assert!(result.failure.is_none() && image.failure.is_none());
        assert_eq!(result.color8, Some(window_bytes));
        assert_eq!(image.color8, Some(scene_bytes));
        assert_eq!(
            image.pixel_format,
            Some(zr_rhi::UiSurfaceFramePixelFormat::Bgra8Srgb)
        );
        assert_eq!(
            image.alpha_mode,
            Some(zr_rhi::UiSurfaceImageAlphaMode::Opaque)
        );
        assert_eq!(image.opaque_pixels, Some(true));
        assert_eq!(
            image.receipt.unwrap().frame_key().unwrap().submission(),
            ticket
        );
        assert_eq!(
            image.receipt.unwrap().byte_len(),
            512,
            "quota uses padded rows but bytes retain only the original texels"
        );
        assert_eq!(
            image.command_frame,
            Some(viewport_media_draw_list().commands[1].frame)
        );
        assert_eq!(
            image.command_clip,
            viewport_media_draw_list().commands[1].clip
        );
    }
    assert_eq!(device.retained_diagnostic_readback_delivery_bytes(), 0);
}

#[test]
fn normal_viewport_media_native_batch_returns_original_scene_bytes_with_window_submission() {
    viewport_media_native_batch(false);
}

#[test]
fn normal_viewport_media_native_partial_quota_rejection_still_settles_the_window_copy() {
    viewport_media_native_batch(true);
}
