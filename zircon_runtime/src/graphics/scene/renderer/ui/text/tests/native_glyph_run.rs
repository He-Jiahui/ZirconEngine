use super::{
    aligned_device_text_start_x, native_bitmap_atlas_relative_baseline, vertical_subpixel_bin,
    UiFrame, UiTextAlign, UiTextDirection,
};

#[test]
fn native_bitmap_glyph_run_retains_four_vertical_raster_phases() {
    assert_eq!(vertical_subpixel_bin(12.0), 0);
    assert_eq!(vertical_subpixel_bin(12.26), 1);
    assert_eq!(vertical_subpixel_bin(12.51), 2);
    assert_eq!(vertical_subpixel_bin(12.76), 3);
}

#[test]
fn native_bitmap_glyph_run_keeps_logical_start_end_alignment_after_device_origin_snap() {
    let frame = UiFrame::new(10.0, 7.0, 100.0, 24.0);

    assert_eq!(
        aligned_device_text_start_x(
            frame,
            UiTextAlign::Start,
            UiTextDirection::LeftToRight,
            30.0,
        ),
        10.0
    );
    assert_eq!(
        aligned_device_text_start_x(
            frame,
            UiTextAlign::Center,
            UiTextDirection::LeftToRight,
            f32::NAN,
        ),
        60.0
    );
    assert_eq!(
        aligned_device_text_start_x(
            frame,
            UiTextAlign::Start,
            UiTextDirection::RightToLeft,
            30.0,
        ),
        80.0
    );
    assert_eq!(
        aligned_device_text_start_x(frame, UiTextAlign::End, UiTextDirection::LeftToRight, 30.0,),
        80.0
    );
    assert_eq!(
        aligned_device_text_start_x(frame, UiTextAlign::End, UiTextDirection::RightToLeft, 30.0,),
        10.0
    );
}

#[test]
fn native_bitmap_glyph_run_prefers_layout_decoration_baseline_and_sanitizes_invalid_input() {
    assert_eq!(
        native_bitmap_atlas_relative_baseline(Some(35.0), 20.0, 30.0, 20.0),
        15.0
    );
    assert_eq!(
        native_bitmap_atlas_relative_baseline(Some(f32::NAN), 20.0, 30.0, 20.0),
        21.0
    );
}

#[cfg(feature = "ui")]
#[test]
fn native_bitmap_glyph_run_resolves_runtime_context_menu_and_hud_artifacts() {
    use crate::core::math::UVec2;
    use crate::graphics::scene::scene_renderer::ui::render::native_text_batches_for_product_proof;
    use crate::text::{font::shared_font_collection_service, TextRuntimeContext};
    use crate::ui::surface::{
        resolve_text_layout_with_cache, UiTextLayoutRequest, UiTextMeasureCache,
    };
    use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
    use zircon_runtime_interface::ui::surface::{
        UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
        UiTextRenderMode,
    };

    let context = TextRuntimeContext::new().expect("runtime text context");
    assert_ne!(
        context.font_collection().collection_id(),
        shared_font_collection_service().collection_id(),
        "runtime products own a collection separate from standalone text"
    );
    let mut cache = UiTextMeasureCache::new_with_text_context(&context).unwrap();
    let viewport = UVec2::new(640, 360);
    let frame = UiFrame::new(24.0, 24.0, 560.0, 160.0);
    for (text, font_size) in [
        ("Start", 18.0),
        ("Collect 8 orbs in 60 s. P pauses.", 16.0),
        ("Score 0 / 8\nTime 60\nPAUSED", 15.0),
    ] {
        cache.begin_frame();
        let style = UiResolvedStyle {
            font_size,
            line_height: font_size + 5.0,
            text_render_mode: UiTextRenderMode::Native,
            ..UiResolvedStyle::default()
        };
        let layout = resolve_text_layout_with_cache(
            &UiTextLayoutRequest::new(text, &style, frame, Some(frame)),
            &mut cache,
        )
        .layout;
        let line_count = layout.lines.len();
        let batches = native_text_batches_for_product_proof(
            &UiRenderExtract {
                tree_id: UiTreeId::new("runtime.g41.native-text"),
                list: UiRenderList {
                    commands: vec![UiRenderCommand {
                        node_id: UiNodeId::new(1),
                        kind: UiRenderCommandKind::Text,
                        frame,
                        clip_frame: Some(frame),
                        z_index: 0,
                        style,
                        text_layout: Some(layout),
                        text: Some(text.to_string()),
                        image: None,
                        opacity: 1.0,
                    }],
                },
                raster_scale: 1.0,
            },
            viewport,
        );
        assert_eq!(batches.len(), line_count);
        assert!(batches
            .iter()
            .all(|batch| batch.glyph_artifact_line.is_some()));
        let projection = super::native_bitmap_atlas_glyph_runs(viewport, &batches);
        assert!(projection.font_ids.glyph_count > 0);
        assert_eq!(
            projection.font_ids.unmapped_glyph_count, 0,
            "{text:?}: native projection must resolve the runtime artifact's font handles"
        );
        assert_eq!(projection.glyph_runs.len(), line_count);
        assert!(projection
            .glyph_runs
            .iter()
            .all(|run| !run.glyphs.is_empty()));
        cache.finish_frame();
    }
}
