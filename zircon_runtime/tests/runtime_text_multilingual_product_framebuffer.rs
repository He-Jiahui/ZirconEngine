use glyphon::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, SwashContent};
use std::{path::PathBuf, sync::Arc};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
    RenderFrameExtract, RenderFramework, RenderOverlayExtract, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec4};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::ui::surface::layout_text;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRenderMode, UiTextWrap,
    UiTextWritingMode,
};

#[path = "runtime_text_multilingual_product_framebuffer/product_project_fixture.rs"]
mod product_project_fixture;
#[path = "runtime_text_multilingual_product_framebuffer/proof_assertions.rs"]
mod proof_assertions;
#[path = "runtime_text_multilingual_product_framebuffer/proof_commands.rs"]
mod proof_commands;
mod support;
use product_project_fixture::product_fixture_asset_manager;
use proof_commands::{
    proof_background, proof_bbcode_text, proof_horizontal_rich_table,
    proof_msdf_sharp_corner_sample, proof_native_sdf_parity, proof_rich_text,
    proof_rich_text_with_direction, proof_rich_text_with_overflow, proof_rich_text_with_wrap,
    proof_text, proof_vertical_bbcode_paragraphs, proof_vertical_rich_table,
    proof_vertical_rich_text, proof_vertical_text,
};

#[cfg(target_os = "windows")]
#[test]
#[ignore = "exports an explicit runtime WGPU multilingual text framebuffer proof"]
fn export_runtime_multilingual_text_product_framebuffer_png() {
    assert_color_emoji_backend_raster_contract();
    assert_arabic_mark_cluster_backend_face_contract();

    let viewport_size = UVec2::new(1080, 1840);
    let (asset_manager, fixture_root) = product_fixture_asset_manager();
    let background = proof_background(viewport_size);
    let mut samples = vec![
        proof_rich_text(
            101,
            UiFrame::new(42.0, 34.0, 876.0, 52.0),
            "Zircon <b>Text</b> <img src=\"res://ui/rich-inline-checker.png\" width=\"28\" height=\"28\" baseline=\"center\"> · AV fi café — Runtime Layout",
        ),
        proof_rich_text_with_wrap(
            111,
            UiFrame::new(42.0, 104.0, 360.0, 116.0),
            "Word wrap alpha beta gamma <img src=\"res://ui/rich-inline-checker.png\" width=\"36\" height=\"36\" baseline=\"center\"> omega",
            UiTextWrap::Word,
        ),
        proof_rich_text_with_wrap(
            112,
            UiFrame::new(480.0, 104.0, 410.0, 116.0),
            "WordSmart prefix segment <img src=\"res://ui/rich-inline-checker.png\" width=\"36\" height=\"36\" baseline=\"center\"> suffix",
            UiTextWrap::WordSmart,
        ),
        proof_text(
            102,
            UiFrame::new(42.0, 244.0, 876.0, 52.0),
            "中文排版：引擎文本与布局",
            UiTextDirection::LeftToRight,
            Some("zh-Hans"),
            UiTextRenderMode::Native,
        ),
        proof_text(
            103,
            UiFrame::new(42.0, 314.0, 876.0, 52.0),
            "العَرَبِيَّةُ — مرحبًا بالعالم",
            UiTextDirection::RightToLeft,
            Some("ar"),
            UiTextRenderMode::Native,
        ),
        proof_text(
            104,
            UiFrame::new(42.0, 384.0, 876.0, 52.0),
            "עברית RTL — שלום עולם — 2026",
            UiTextDirection::RightToLeft,
            Some("he"),
            UiTextRenderMode::Native,
        ),
        proof_text(
            105,
            UiFrame::new(42.0, 454.0, 876.0, 52.0),
            "Emoji 😀  🧑‍🚀  ❤️  🎮  ✨",
            UiTextDirection::LeftToRight,
            Some("en"),
            UiTextRenderMode::Native,
        ),
        proof_text(
            106,
            UiFrame::new(42.0, 524.0, 876.0, 52.0),
            "Mixed Serbian: Latin aб — 构建 — مرحبًا — FPS 60",
            UiTextDirection::LeftToRight,
            Some("sr"),
            UiTextRenderMode::Native,
        ),
        proof_text(
            107,
            UiFrame::new(42.0, 594.0, 410.0, 58.0),
            "A/M/W · SDF 尖角",
            UiTextDirection::LeftToRight,
            Some("en"),
            UiTextRenderMode::Sdf,
        ),
        proof_text(
            108,
            UiFrame::new(42.0, 664.0, 410.0, 52.0),
            "骨直示辺",
            UiTextDirection::LeftToRight,
            Some("zh-Hans"),
            UiTextRenderMode::Native,
        ),
        proof_text(
            109,
            UiFrame::new(508.0, 664.0, 410.0, 52.0),
            "骨直示辺",
            UiTextDirection::LeftToRight,
            Some("ja"),
            UiTextRenderMode::Native,
        ),
        proof_vertical_text(
            110,
            UiFrame::new(982.0, 34.0, 76.0, 240.0),
            "竖排「标点」。第二列，验证。",
        ),
        proof_rich_text_with_direction(
            113,
            UiFrame::new(42.0, 734.0, 876.0, 72.0),
            "RTL rich: שלום <img src=\"res://ui/rich-inline-checker.png\" width=\"36\" height=\"36\" baseline=\"center\"> עולם",
            UiTextDirection::RightToLeft,
            Some("he"),
        ),
        proof_rich_text_with_overflow(
            114,
            UiFrame::new(42.0, 814.0, 220.0, 52.0),
            "Ellipsis A <img src=\"res://ui/rich-inline-checker.png\" width=\"36\" height=\"36\" baseline=\"center\"> trailing content",
            UiTextOverflow::Ellipsis,
        ),
        proof_vertical_rich_text(
            115,
            UiFrame::new(934.0, 330.0, 124.0, 100.0),
            "竖<img src=\"res://ui/rich-inline-checker.png\" width=\"36\" height=\"40\" baseline=\"center\">排富文本",
        ),
        proof_bbcode_text(
            116,
            UiFrame::new(42.0, 884.0, 430.0, 220.0),
            "[p align=center]BBCode V1 [icon=★|Microsoft YaHei UI] :rocket:[/p][ul bullet=◆][li][b]Hanging list[/b] alpha beta gamma delta epsilon[/li][li]Nested[ol type=A][li]Alpha item[/li][li]Beta item[/li][/ol][/li][/ul]",
            UiTextWrap::WordSmart,
        ),
        proof_bbcode_text(
            117,
            UiFrame::new(520.0, 884.0, 500.0, 220.0),
            "[p align=right indent=28][color=#64d8ff]Paragraph container[/color][/p][indent]Logical indent wraps through the shared text layout owner instead of manufactured spaces.[/indent]",
            UiTextWrap::WordSmart,
        ),
        proof_vertical_bbcode_paragraphs(),
        proof_horizontal_rich_table(),
        proof_vertical_rich_table(),
    ];
    samples.extend(proof_native_sdf_parity());
    samples.push(proof_msdf_sharp_corner_sample());
    samples.extend(proof_commands::proof_variable_font_instance_samples());
    let vertical_layout = samples[11]
        .text_layout
        .as_ref()
        .expect("VerticalRl product proof must consume a resolved layout");
    assert_eq!(vertical_layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_eq!(
        vertical_layout.lines.len(),
        2,
        "the product proof must exercise two real VerticalRl columns"
    );
    assert!(
        vertical_layout
            .lines
            .windows(2)
            .all(|columns| columns[0].frame.x > columns[1].frame.x),
        "VerticalRl product columns must be laid out from right to left"
    );
    proof_assertions::assert_bbcode_block_layouts(&samples);
    proof_assertions::assert_vertical_bbcode_paragraph_layout(&samples);
    proof_assertions::assert_native_sdf_parity_layout(&samples);
    proof_assertions::assert_bbcode_table_layout(&samples);
    proof_assertions::assert_vertical_bbcode_table_layout(&samples);
    let mut commands = vec![background.clone()];
    commands.extend(samples.iter().cloned());
    let (capture, stats) = render_ui_extract_frame(
        UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.text.multilingual.product"),
            list: UiRenderList { commands },
        },
        viewport_size,
        asset_manager.clone(),
    );
    let (background_capture, _) = render_ui_extract_frame(
        UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.text.multilingual.product.background"),
            list: UiRenderList {
                commands: vec![background],
            },
        },
        viewport_size,
        asset_manager,
    );

    assert_eq!(stats.last_ui_text_payload_count, samples.len());
    for sample in &samples {
        let changed = count_changed_pixels_in_frame(
            &capture.rgba,
            &background_capture.rgba,
            capture.width,
            capture.height,
            sample.frame,
            10,
        );
        assert!(
            changed > 48,
            "every product row must contain real framebuffer glyph deltas; node={:?}, changed={changed}",
            sample.node_id
        );
    }
    proof_assertions::assert_native_sdf_parity_pixels(&samples, &capture, &background_capture);
    proof_assertions::assert_msdf_sharp_corner_pixels(&samples, &capture, &background_capture);
    proof_assertions::assert_variable_font_instance_pixels(&samples, &capture, &background_capture);
    let locale_variant_delta = count_relative_pixel_differences(
        &capture.rgba,
        capture.width,
        capture.height,
        samples[9].frame,
        samples[10].frame,
        10,
    );
    assert!(
        locale_variant_delta > 48,
        "zh-Hans and ja must not collapse the same Han codepoints onto one rendered face; changed={locale_variant_delta}"
    );
    let checker_channels = dominant_checker_channel_counts(
        &capture.rgba,
        capture.width,
        capture.height,
        samples[0].frame,
    );
    assert!(
        checker_channels.iter().all(|count| *count > 20),
        "rich inline image must sample the imported checker texture, not a solid placeholder: {checker_channels:?}"
    );
    for (label, sample_index) in [("Word", 1_usize), ("WordSmart", 2_usize)] {
        let channels = dominant_checker_channel_counts(
            &capture.rgba,
            capture.width,
            capture.height,
            samples[sample_index].frame,
        );
        assert!(
            channels.iter().all(|count| *count > 20),
            "{label} inline image must survive wrapping and sample the imported checker texture: {channels:?}"
        );
    }
    let rtl_rich_channels = dominant_checker_channel_counts(
        &capture.rgba,
        capture.width,
        capture.height,
        samples[12].frame,
    );
    assert!(
        rtl_rich_channels.iter().all(|count| *count > 20),
        "RTL rich inline image must follow visual order and sample the imported checker texture: {rtl_rich_channels:?}"
    );
    let ellipsis_layout = samples[13]
        .text_layout
        .as_ref()
        .expect("rich ellipsis product proof must consume resolved layout");
    assert!(ellipsis_layout.lines[0].ellipsized);
    assert!(ellipsis_layout.lines[0].text.ends_with('…'));
    assert!(ellipsis_layout.lines[0].text.contains('\u{fffc}'));
    let ellipsis_checker_frame = checker_frame_from_layout(&samples[13]);
    let ellipsis_rich_channels = dominant_checker_channel_counts(
        &capture.rgba,
        capture.width,
        capture.height,
        ellipsis_checker_frame,
    );
    assert!(
        ellipsis_rich_channels.iter().all(|count| *count > 20),
        "ellipsized rich inline image must retain its real sampled texture: channels={ellipsis_rich_channels:?}, frame={ellipsis_checker_frame:?}, line={:?}, advances={:?}",
        ellipsis_layout.lines[0].text,
        ellipsis_layout.lines[0].glyph_advances,
    );
    let vertical_rich_layout = samples[14]
        .text_layout
        .as_ref()
        .expect("vertical rich product proof must consume resolved layout");
    assert_eq!(
        vertical_rich_layout.writing_mode,
        UiTextWritingMode::VerticalRl
    );
    assert!(vertical_rich_layout.lines.len() >= 2);
    let vertical_inline_line = vertical_rich_layout
        .lines
        .iter()
        .find(|line| line.text.contains('\u{fffc}'))
        .expect("vertical rich product proof must retain inline image");
    let inline_index = vertical_inline_line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("vertical rich inline grapheme");
    assert!((vertical_inline_line.glyph_advances[inline_index] - 40.0).abs() < 0.01);
    let vertical_checker_frame = checker_frame_from_layout(&samples[14]);
    let vertical_rich_channels = dominant_checker_channel_counts(
        &capture.rgba,
        capture.width,
        capture.height,
        vertical_checker_frame,
    );
    assert!(
        vertical_rich_channels.iter().all(|count| *count > 20),
        "VerticalRl rich inline image must advance on y and retain sampled texture: channels={vertical_rich_channels:?}, frame={vertical_checker_frame:?}"
    );
    let vertical_bounds = changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background_capture.rgba,
        capture.width,
        capture.height,
        samples[11].frame,
        10,
    )
    .expect("VerticalRl SDF row must contain real glyph pixels");
    let vertical_width = vertical_bounds.2 - vertical_bounds.0 + 1;
    let vertical_height = vertical_bounds.3 - vertical_bounds.1 + 1;
    assert!(
        vertical_height >= vertical_width * 2,
        "VerticalRl product glyphs must form tall columns; bounds={vertical_bounds:?}"
    );
    for (index, column) in vertical_layout.lines.iter().enumerate() {
        let changed = count_changed_pixels_in_frame(
            &capture.rgba,
            &background_capture.rgba,
            capture.width,
            capture.height,
            column.frame,
            10,
        );
        assert!(
            changed > 48,
            "VerticalRl product column {index} must contain real glyph pixels; changed={changed}"
        );
    }

    let output = proof_path();
    std::fs::create_dir_all(output.parent().expect("proof output directory"))
        .expect("create runtime text proof directory");
    image::save_buffer(
        &output,
        &capture.rgba,
        capture.width,
        capture.height,
        image::ColorType::Rgba8,
    )
    .expect("save runtime multilingual text product framebuffer");
    assert!(output.is_file());
    assert!(
        output
            .components()
            .all(|component| component.as_os_str() != "target"),
        "runtime text proof must not be written into target: {}",
        output.display()
    );
    eprintln!(
        "runtime multilingual product framebuffer={}",
        output.display()
    );
    let _ = std::fs::remove_dir_all(fixture_root);
}

fn assert_color_emoji_backend_raster_contract() {
    let mut font_system = FontSystem::new();
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(48.0, 60.0));
    buffer.set_size(&mut font_system, Some(128.0), Some(80.0));
    buffer.set_text(
        &mut font_system,
        "😀",
        &Attrs::new(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let glyph = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .find(|glyph| glyph.glyph_id != 0)
        .cloned()
        .expect("system fallback must shape the color emoji");
    let selected_family = font_system
        .db()
        .face(glyph.font_id)
        .and_then(|face| face.families.first())
        .map(|family| family.0.clone())
        .expect("backend-selected emoji face must remain queryable");
    let mut swash_cache = SwashCache::new();
    let image = swash_cache
        .get_image_uncached(&mut font_system, glyph.physical((0.0, 0.0), 1.0).cache_key)
        .expect("backend-selected emoji glyph must rasterize");

    assert_eq!(
        image.content,
        SwashContent::Color,
        "selected emoji face must use the native color-glyph path: {selected_family}"
    );
    assert_eq!(
        image.data.len(),
        image.placement.width as usize * image.placement.height as usize * 4,
        "SwashContent::Color must provide one RGBA texel per glyph pixel"
    );
    eprintln!(
        "runtime color emoji backend face={selected_family}, rgba_bytes={}",
        image.data.len()
    );
}

fn assert_arabic_mark_cluster_backend_face_contract() {
    let text = "نَ";
    let mut font_system = FontSystem::new();
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(48.0, 60.0));
    buffer.set_size(&mut font_system, Some(256.0), Some(80.0));
    buffer.set_text(
        &mut font_system,
        text,
        &Attrs::new().family(Family::Name("Zircon Missing Primary")),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let glyphs = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .filter(|glyph| glyph.start < text.len() && glyph.end <= text.len())
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        glyphs.len() >= 2,
        "Arabic base plus fatha must reach the backend as a multi-glyph grapheme cluster: {glyphs:?}"
    );
    let face = glyphs[0].font_id;
    assert!(
        glyphs.iter().all(|glyph| glyph.font_id == face),
        "one Arabic grapheme cluster must stay on one actual backend face: {glyphs:?}"
    );
    let selected_family = font_system
        .db()
        .face(face)
        .and_then(|face| face.families.first())
        .map(|family| family.0.as_str())
        .unwrap_or("<unknown>");
    eprintln!(
        "runtime Arabic mark cluster backend face={selected_family}, glyphs={}",
        glyphs.len()
    );
}

fn render_ui_extract_frame(
    ui: UiRenderExtract,
    viewport_size: UVec2,
    asset_manager: Arc<ProjectAssetManager>,
) -> (
    zircon_runtime::core::framework::render::CapturedFrame,
    zircon_runtime::core::framework::render::RenderStats,
) {
    let asset_runtime = support::ProjectAssetTestRuntime::new(asset_manager);
    let server = WgpuRenderFramework::new(asset_runtime.access()).expect("headless WGPU renderer");
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("headless viewport");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-multilingual-text")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .expect("text proof quality profile");
    let settle_frame_count = if ui
        .list
        .commands
        .iter()
        .any(|command| command.text.is_some())
    {
        24_u64
    } else {
        2_u64
    };
    for frame_index in 0..settle_frame_count {
        server
            .submit_frame_extract_with_ui(
                viewport,
                empty_extract(viewport_size, frame_index + 1),
                Some(ui.clone()),
            )
            .expect("submit multilingual text settle frame");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    let stats = server.query_stats().expect("text proof render stats");
    let capture = server
        .capture_frame(viewport)
        .expect("capture multilingual text frame")
        .expect("submitted frame must be capturable");
    (capture, stats)
}

fn dominant_checker_channel_counts(
    rgba: &[u8],
    width: u32,
    height: u32,
    frame: UiFrame,
) -> [usize; 3] {
    let mut counts = [0usize; 3];
    let left = frame.x.max(0.0).floor() as u32;
    let top = frame.y.max(0.0).floor() as u32;
    let right = frame.right().max(0.0).ceil().min(width as f32) as u32;
    let bottom = frame.bottom().max(0.0).ceil().min(height as f32) as u32;
    for y in top..bottom {
        for x in left..right {
            let index = ((y * width + x) * 4) as usize;
            let pixel = &rgba[index..index + 4];
            if pixel[0] > 180
                && pixel[0] > pixel[1].saturating_add(60)
                && pixel[0] > pixel[2].saturating_add(60)
            {
                counts[0] += 1;
            }
            if pixel[1] > 180
                && pixel[1] > pixel[0].saturating_add(60)
                && pixel[1] > pixel[2].saturating_add(60)
            {
                counts[1] += 1;
            }
            if pixel[2] > 180
                && pixel[2] > pixel[0].saturating_add(60)
                && pixel[2] > pixel[1].saturating_add(60)
            {
                counts[2] += 1;
            }
        }
    }
    counts
}

fn checker_frame_from_layout(command: &UiRenderCommand) -> UiFrame {
    let layout = command
        .text_layout
        .as_ref()
        .expect("checker proof must carry resolved layout");
    let line = layout
        .lines
        .iter()
        .find(|line| line.text.contains('\u{fffc}'))
        .expect("checker proof must retain an inline placeholder");
    let inline_index = line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("inline placeholder grapheme index");
    let main_offset = line
        .glyph_advances
        .iter()
        .take(inline_index)
        .copied()
        .sum::<f32>();
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        UiFrame::new(
            line.frame.x - 4.0,
            line.frame.y + main_offset - 2.0,
            line.frame.width + 8.0,
            44.0,
        )
    } else {
        UiFrame::new(
            line.frame.x + main_offset - 2.0,
            line.frame.y - 4.0,
            40.0,
            44.0,
        )
    }
}

fn empty_extract(viewport_size: UVec2, snapshot_id: u64) -> RenderFrameExtract {
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
        RenderWorldSnapshotHandle::new(snapshot_id),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: EnvironmentExtract::default(),
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

fn count_relative_pixel_differences(
    rgba: &[u8],
    width: u32,
    height: u32,
    lhs: UiFrame,
    rhs: UiFrame,
    threshold: u8,
) -> usize {
    let width = width as usize;
    let height = height as usize;
    let lhs_left = lhs.x.max(0.0).floor() as usize;
    let lhs_top = lhs.y.max(0.0).floor() as usize;
    let rhs_left = rhs.x.max(0.0).floor() as usize;
    let rhs_top = rhs.y.max(0.0).floor() as usize;
    let compare_width = lhs.width.min(rhs.width).max(0.0).ceil() as usize;
    let compare_height = lhs.height.min(rhs.height).max(0.0).ceil() as usize;
    let mut changed = 0usize;
    for dy in 0..compare_height {
        let lhs_y = lhs_top + dy;
        let rhs_y = rhs_top + dy;
        if lhs_y >= height || rhs_y >= height {
            break;
        }
        for dx in 0..compare_width {
            let lhs_x = lhs_left + dx;
            let rhs_x = rhs_left + dx;
            if lhs_x >= width || rhs_x >= width {
                break;
            }
            let lhs_index = (lhs_y * width + lhs_x) * 4;
            let rhs_index = (rhs_y * width + rhs_x) * 4;
            let delta = rgba[lhs_index..lhs_index + 4]
                .iter()
                .zip(&rgba[rhs_index..rhs_index + 4])
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

fn changed_pixel_bounds_in_frame(
    lhs: &[u8],
    rhs: &[u8],
    width: u32,
    height: u32,
    frame: UiFrame,
    threshold: u8,
) -> Option<(usize, usize, usize, usize, usize)> {
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let right = frame.right().max(0.0).ceil() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let width = width as usize;
    let height = height as usize;
    let mut bounds = (usize::MAX, usize::MAX, 0usize, 0usize, 0usize);
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
                bounds.0 = bounds.0.min(x);
                bounds.1 = bounds.1.min(y);
                bounds.2 = bounds.2.max(x);
                bounds.3 = bounds.3.max(y);
                bounds.4 += 1;
            }
        }
    }
    (bounds.4 > 0).then_some(bounds)
}

fn proof_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join("runtime_text_variable_font_instances_product_framebuffer_20260714.png")
}
