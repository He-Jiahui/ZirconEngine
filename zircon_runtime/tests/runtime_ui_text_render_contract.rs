use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderOverlayExtract, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec4};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::ui::template::{UiAssetLoader, UiDocumentCompiler, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};

#[test]
fn native_runtime_ui_text_renders_sparse_glyph_pixels_without_placeholder_band() {
    let capture = render_text_frame(UiTextRenderMode::Native, "Native Glyphs");
    assert_sparse_text_footprint(&capture.rgba, capture.width, capture.height);
}

#[test]
fn sdf_runtime_ui_text_renders_sparse_glyph_pixels_without_placeholder_band() {
    let capture = render_text_frame(UiTextRenderMode::Sdf, "Sdf Glyphs");
    assert_sparse_text_footprint(&capture.rgba, capture.width, capture.height);
}

#[test]
fn sdf_runtime_ui_text_adds_sparse_real_glyph_delta_over_background() {
    let viewport_size = UVec2::new(320, 180);
    let sdf_command = text_command(UiTextRenderMode::Sdf, "AIO");
    let text_frame = sdf_command.frame;
    let (with_text, with_text_stats) = render_ui_extract_frame(
        UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.text.sdf.real_glyphs"),
            list: UiRenderList {
                commands: vec![sdf_command],
            },
        },
        "runtime.ui.text.sdf.real_glyphs",
        viewport_size,
    );
    assert_eq!(with_text_stats.last_ui_text_payload_count, 1);

    let mut background_command = text_command(UiTextRenderMode::Sdf, "AIO");
    background_command.text = None;
    let (background, background_stats) = render_ui_extract_frame(
        UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.text.sdf.real_glyphs.background"),
            list: UiRenderList {
                commands: vec![background_command],
            },
        },
        "runtime.ui.text.sdf.real_glyphs.background",
        viewport_size,
    );
    assert_eq!(background_stats.last_ui_text_payload_count, 0);

    let changed_pixels = count_changed_pixels_in_frame(
        &with_text.rgba,
        &background.rgba,
        with_text.width,
        with_text.height,
        text_frame,
        12,
    );
    let text_area = (text_frame.width * text_frame.height) as usize;

    assert_sparse_text_footprint(&with_text.rgba, with_text.width, with_text.height);
    assert!(
        changed_pixels > 60,
        "expected SDF glyph bake to add visible text deltas over the background; changed_pixels={changed_pixels}"
    );
    assert!(
        changed_pixels < text_area / 3,
        "expected SDF glyph delta to remain sparse instead of filling the old placeholder block; changed_pixels={changed_pixels}, text_area={text_area}"
    );
}

#[test]
fn clipped_runtime_ui_text_stays_inside_clip_frame() {
    let capture = render_custom_text_frame(
        UiRenderCommand {
            node_id: UiNodeId::new(2),
            kind: UiRenderCommandKind::Text,
            frame: UiFrame::new(20.0, 64.0, 280.0, 40.0),
            clip_frame: Some(UiFrame::new(20.0, 64.0, 48.0, 40.0)),
            z_index: 0,
            style: UiResolvedStyle {
                foreground_color: Some("#f3f7ff".to_string()),
                font: Some("res://fonts/default.font.toml".to_string()),
                font_size: 28.0,
                line_height: 32.0,
                text_align: UiTextAlign::Left,
                wrap: UiTextWrap::None,
                text_render_mode: UiTextRenderMode::Native,
                ..UiResolvedStyle::default()
            },
            text_layout: None,
            text: Some("Clip Me Hard".to_string()),
            image: None,
            opacity: 1.0,
        },
        "runtime.ui.text.clip",
    );

    let clipped_region = UiFrame::new(20.0, 64.0, 48.0, 40.0);
    let outside_region = UiFrame::new(76.0, 64.0, 220.0, 40.0);
    let lit_inside =
        count_lit_pixels_in_frame(&capture.rgba, capture.width, capture.height, clipped_region);
    let lit_outside =
        count_lit_pixels_in_frame(&capture.rgba, capture.width, capture.height, outside_region);

    assert!(
        lit_inside > 40,
        "expected clipped text to still leave visible glyph pixels inside the clip frame; lit_inside={lit_inside}"
    );
    assert!(
        lit_outside < 12,
        "expected glyph rasterization to honor clip bounds instead of bleeding across the full text band; lit_outside={lit_outside}"
    );
}

#[test]
fn wrapped_runtime_ui_text_spans_multiple_rows_inside_text_frame() {
    let capture = render_custom_text_frame(
        UiRenderCommand {
            node_id: UiNodeId::new(3),
            kind: UiRenderCommandKind::Text,
            frame: UiFrame::new(24.0, 40.0, 118.0, 92.0),
            clip_frame: None,
            z_index: 0,
            style: UiResolvedStyle {
                foreground_color: Some("#f3f7ff".to_string()),
                font: Some("res://fonts/default.font.toml".to_string()),
                font_size: 22.0,
                line_height: 28.0,
                text_align: UiTextAlign::Left,
                wrap: UiTextWrap::Word,
                text_render_mode: UiTextRenderMode::Native,
                ..UiResolvedStyle::default()
            },
            text_layout: None,
            text: Some("Wrap this runtime text into multiple rows".to_string()),
            image: None,
            opacity: 1.0,
        },
        "runtime.ui.text.wrap",
    );

    let upper_band = UiFrame::new(24.0, 40.0, 118.0, 36.0);
    let lower_band = UiFrame::new(24.0, 76.0, 118.0, 44.0);
    let lit_upper =
        count_lit_pixels_in_frame(&capture.rgba, capture.width, capture.height, upper_band);
    let lit_lower =
        count_lit_pixels_in_frame(&capture.rgba, capture.width, capture.height, lower_band);

    assert!(
        lit_upper > 80 && lit_lower > 80,
        "expected wrapped text to populate multiple row bands inside the text frame; upper={lit_upper}, lower={lit_lower}"
    );
}

#[test]
fn runtime_ui_text_opacity_modulates_glyph_intensity() {
    let opaque = render_custom_text_frame(
        text_command_with_opacity(1.0, "Opacity"),
        "runtime.ui.text.opacity.opaque",
    );
    let translucent = render_custom_text_frame(
        text_command_with_opacity(0.22, "Opacity"),
        "runtime.ui.text.opacity.translucent",
    );
    let bounds = UiFrame::new(20.0, 64.0, 280.0, 40.0);
    let opaque_luma = average_luma_in_frame(&opaque.rgba, opaque.width, opaque.height, bounds);
    let translucent_luma = average_luma_in_frame(
        &translucent.rgba,
        translucent.width,
        translucent.height,
        bounds,
    );

    // Capture bytes come back through the final sRGB render target, so this assertion
    // intentionally checks visible dimming instead of assuming linear alpha scaling.
    assert!(
        opaque_luma > translucent_luma * 1.5 && opaque_luma > translucent_luma + 4.0,
        "expected lower opacity glyphs to render dimmer than opaque glyphs in the captured frame; opaque_luma={opaque_luma:.2}, translucent_luma={translucent_luma:.2}"
    );
}

#[test]
fn template_surface_quad_text_renders_wrapped_glyphs_through_formal_ui_pipeline() {
    let viewport_size = UVec2::new(220, 180);
    let ui = render_extract_from_asset_source(
        &template_runtime_text_asset_source(1.0),
        "runtime.ui.template.wrap",
        viewport_size,
    );
    let text_frame = ui
        .list
        .commands
        .iter()
        .find(|command| command.text.is_some())
        .map(|command| command.frame)
        .expect("template text command frame");

    let (with_text, with_text_stats) = render_ui_extract_frame(
        ui.clone(),
        "runtime.ui.template.wrap.capture",
        viewport_size,
    );
    assert!(
        with_text_stats.last_ui_quad_count >= 1,
        "expected template-driven container text to keep its quad/background path alive"
    );
    assert_eq!(with_text_stats.last_ui_text_payload_count, 1);

    let mut background_only = ui;
    background_only.tree_id = UiTreeId::new("runtime.ui.template.wrap.background");
    for command in &mut background_only.list.commands {
        command.text = None;
    }
    let (without_text, without_text_stats) = render_ui_extract_frame(
        background_only,
        "runtime.ui.template.wrap.background.capture",
        viewport_size,
    );
    assert_eq!(without_text_stats.last_ui_text_payload_count, 0);

    let upper_band = UiFrame::new(text_frame.x, text_frame.y, text_frame.width, 36.0);
    let lower_band = UiFrame::new(
        text_frame.x,
        text_frame.y + 36.0,
        text_frame.width,
        (text_frame.height - 36.0).max(1.0),
    );
    let changed_upper = count_changed_pixels_in_frame(
        &with_text.rgba,
        &without_text.rgba,
        with_text.width,
        with_text.height,
        upper_band,
        12,
    );
    let changed_lower = count_changed_pixels_in_frame(
        &with_text.rgba,
        &without_text.rgba,
        with_text.width,
        with_text.height,
        lower_band,
        12,
    );

    assert!(
        changed_upper > 40 && changed_lower > 40,
        "expected template/surface-driven wrapped text to add glyph deltas across multiple row bands; upper={changed_upper}, lower={changed_lower}"
    );
}

#[test]
fn template_surface_text_opacity_modulates_glyph_delta_through_formal_ui_pipeline() {
    let viewport_size = UVec2::new(220, 180);
    let opaque_ui = render_extract_from_asset_source(
        &template_runtime_text_asset_source(1.0),
        "runtime.ui.template.opacity.opaque",
        viewport_size,
    );
    let text_frame = opaque_ui
        .list
        .commands
        .iter()
        .find(|command| command.text.is_some())
        .map(|command| command.frame)
        .expect("template text command frame");
    let translucent_ui = render_extract_from_asset_source(
        &template_runtime_text_asset_source(0.22),
        "runtime.ui.template.opacity.translucent",
        viewport_size,
    );

    let (opaque_capture, _) = render_ui_extract_frame(
        opaque_ui.clone(),
        "runtime.ui.template.opacity.opaque.capture",
        viewport_size,
    );
    let (opaque_background, _) = render_ui_extract_frame(
        text_stripped_ui_extract(opaque_ui, "runtime.ui.template.opacity.opaque.background"),
        "runtime.ui.template.opacity.opaque.background.capture",
        viewport_size,
    );
    let (translucent_capture, _) = render_ui_extract_frame(
        translucent_ui.clone(),
        "runtime.ui.template.opacity.translucent.capture",
        viewport_size,
    );
    let (translucent_background, _) = render_ui_extract_frame(
        text_stripped_ui_extract(
            translucent_ui,
            "runtime.ui.template.opacity.translucent.background",
        ),
        "runtime.ui.template.opacity.translucent.background.capture",
        viewport_size,
    );

    let opaque_delta = average_luma_in_frame(
        &opaque_capture.rgba,
        opaque_capture.width,
        opaque_capture.height,
        text_frame,
    ) - average_luma_in_frame(
        &opaque_background.rgba,
        opaque_background.width,
        opaque_background.height,
        text_frame,
    );
    let translucent_delta = average_luma_in_frame(
        &translucent_capture.rgba,
        translucent_capture.width,
        translucent_capture.height,
        text_frame,
    ) - average_luma_in_frame(
        &translucent_background.rgba,
        translucent_background.width,
        translucent_background.height,
        text_frame,
    );

    assert!(
        opaque_delta > translucent_delta * 1.5 && opaque_delta > translucent_delta + 4.0,
        "expected template/surface-driven opacity to dim glyph contribution above the same background; opaque_delta={opaque_delta:.2}, translucent_delta={translucent_delta:.2}"
    );
}

fn render_text_frame(
    text_render_mode: UiTextRenderMode,
    text: &str,
) -> zircon_runtime::core::framework::render::CapturedFrame {
    render_custom_text_frame(
        text_command(text_render_mode, text),
        "runtime.ui.text.contract",
    )
}

fn text_command(text_render_mode: UiTextRenderMode, text: &str) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(20.0, 64.0, 280.0, 40.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            foreground_color: Some("#f3f7ff".to_string()),
            font: Some("res://fonts/default.font.toml".to_string()),
            font_size: 28.0,
            line_height: 32.0,
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::None,
            text_render_mode,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}

fn render_custom_text_frame(
    command: UiRenderCommand,
    tree_id: &str,
) -> zircon_runtime::core::framework::render::CapturedFrame {
    let (capture, stats) = render_ui_extract_frame(
        UiRenderExtract {
            tree_id: UiTreeId::new(tree_id),
            list: UiRenderList {
                commands: vec![command],
            },
        },
        tree_id,
        UVec2::new(320, 180),
    );
    assert_eq!(stats.last_ui_command_count, 1);
    assert_eq!(stats.last_ui_quad_count, 0);
    assert_eq!(stats.last_ui_text_payload_count, 1);
    capture
}

fn render_ui_extract_frame(
    ui: UiRenderExtract,
    _profile_name: &str,
    viewport_size: UVec2,
) -> (
    zircon_runtime::core::framework::render::CapturedFrame,
    zircon_runtime::core::framework::render::RenderStats,
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-text")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();

    server
        .submit_frame_extract_with_ui(viewport, empty_extract(viewport_size), Some(ui))
        .unwrap();

    let stats = server.query_stats().unwrap();
    let capture = server
        .capture_frame(viewport)
        .unwrap()
        .expect("text submission should produce a capturable frame");
    (capture, stats)
}

fn render_extract_from_asset_source(
    source: &str,
    tree_id: &str,
    viewport_size: UVec2,
) -> UiRenderExtract {
    let document = UiAssetLoader::load_toml_str(source).expect("template asset document");
    let compiled = UiDocumentCompiler::default()
        .compile(&document)
        .expect("compiled template asset");
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(tree_id),
        &compiled,
    )
    .expect("template surface");
    surface
        .compute_layout(UiSize::new(viewport_size.x as f32, viewport_size.y as f32))
        .expect("template surface layout");
    surface.render_extract.clone()
}

fn text_stripped_ui_extract(mut ui: UiRenderExtract, tree_id: &str) -> UiRenderExtract {
    ui.tree_id = UiTreeId::new(tree_id);
    for command in &mut ui.list.commands {
        command.text = None;
    }
    ui
}

fn template_runtime_text_asset_source(opacity: f32) -> String {
    format!(
        r##"
stylesheets = []

[asset]
kind = "layout"
id = "runtime.ui.text.contract.asset"
version = 1
display_name = "Runtime Text Contract Asset"

[imports]
widgets = []
styles = []

[tokens]

[root]
node_id = "root"
kind = "native"
type = "Overlay"
control_id = "Root"
classes = []
bindings = []

[root.params]

[root.props]

[root.layout.container]
kind = "Overlay"

[root.layout.height]
stretch = "Stretch"

[root.layout.width]
stretch = "Stretch"

[root.style_overrides.self]

[root.style_overrides.slot]

[[root.children]]

[root.children.slot]

[root.children.node]
node_id = "card"
kind = "native"
type = "Container"
control_id = "Card"
classes = []
bindings = []
children = []

[root.children.node.params]

[root.children.node.props]
text = "Template driven glyph wrap path"
opacity = {opacity}
font = "res://fonts/default.font.toml"
font_size = 22.0
line_height = 28.0
text_align = "left"
wrap = "word"
text_render_mode = "native"

[root.children.node.props.background]
color = "#1b2635"

[root.children.node.props.foreground]
color = "#f2f7ff"

[root.children.node.props.border]
color = "#6fb7ff88"
radius = 8.0
width = 1.0

[root.children.node.layout.anchor]
x = 0.0
y = 0.0

[root.children.node.layout.position]
x = 24.0
y = 24.0

[root.children.node.layout.width]
min = 132.0
preferred = 132.0
max = 132.0
stretch = "Fixed"

[root.children.node.layout.height]
min = 96.0
preferred = 96.0
max = 96.0
stretch = "Fixed"

[root.children.node.style_overrides.self]

[root.children.node.style_overrides.slot]

[components]
"##
    )
}

fn empty_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: zircon_runtime::core::math::Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn assert_sparse_text_footprint(rgba: &[u8], width: u32, height: u32) {
    let text_bounds = UiFrame::new(20.0, 64.0, 280.0, 40.0);
    let lit_in_text_bounds = count_lit_pixels_in_frame(rgba, width, height, text_bounds);
    let left_margin = UiFrame::new(20.0, 64.0, 36.0, 40.0);
    let right_margin = UiFrame::new(264.0, 64.0, 36.0, 40.0);
    let lit_left_margin = count_lit_pixels_in_frame(rgba, width, height, left_margin);
    let lit_right_margin = count_lit_pixels_in_frame(rgba, width, height, right_margin);
    let area = (text_bounds.width * text_bounds.height) as usize;

    assert!(
        lit_in_text_bounds > 180,
        "expected runtime UI text to produce visible glyph pixels; lit_in_text_bounds={lit_in_text_bounds}"
    );
    assert!(
        lit_in_text_bounds < area / 3,
        "expected runtime UI text to stay sparse like glyphs instead of filling a placeholder band; lit_in_text_bounds={lit_in_text_bounds}, area={area}"
    );
    assert!(
        lit_left_margin < 40 && lit_right_margin < 40,
        "expected centered text to leave dark side margins instead of drawing a full-width placeholder band; left={lit_left_margin}, right={lit_right_margin}"
    );
}

fn text_command_with_opacity(opacity: f32, text: &str) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(4),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(20.0, 64.0, 280.0, 40.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            foreground_color: Some("#f3f7ff".to_string()),
            font: Some("res://fonts/default.font.toml".to_string()),
            font_size: 28.0,
            line_height: 32.0,
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::None,
            text_render_mode: UiTextRenderMode::Native,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity,
    }
}

fn count_lit_pixels_in_frame(rgba: &[u8], width: u32, height: u32, frame: UiFrame) -> usize {
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let right = frame.right().max(0.0).ceil() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let width = width as usize;
    let height = height as usize;

    let mut lit = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let index = (y * width + x) * 4;
            let pixel = &rgba[index..index + 4];
            if pixel[0] > 14 || pixel[1] > 14 || pixel[2] > 14 {
                lit += 1;
            }
        }
    }
    lit
}

fn count_changed_pixels_in_frame(
    lhs: &[u8],
    rhs: &[u8],
    width: u32,
    height: u32,
    frame: UiFrame,
    threshold: u8,
) -> usize {
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let right = frame.right().max(0.0).ceil() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let width = width as usize;
    let height = height as usize;

    let mut changed = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let index = (y * width + x) * 4;
            let delta = lhs[index..index + 4]
                .iter()
                .zip(&rhs[index..index + 4])
                .map(|(lhs, rhs)| lhs.abs_diff(*rhs))
                .max()
                .unwrap_or(0);
            if delta >= threshold {
                changed += 1;
            }
        }
    }
    changed
}

fn average_luma_in_frame(rgba: &[u8], width: u32, height: u32, frame: UiFrame) -> f32 {
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let right = frame.right().max(0.0).ceil() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let width = width as usize;
    let height = height as usize;

    let mut total = 0.0f32;
    let mut count = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let index = (y * width + x) * 4;
            let pixel = &rgba[index..index + 4];
            total += 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}
