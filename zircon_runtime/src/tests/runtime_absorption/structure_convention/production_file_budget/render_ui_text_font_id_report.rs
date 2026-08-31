use super::super::rust_source_view::production_code_view;
use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_text_font_id_report_is_child_owner() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let font_id_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/font_id_report.rs");
    let native_glyph_run =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/native_glyph_run.rs");
    let prepare_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/prepare_report.rs");
    let text_module = read_runtime_src("text/mod.rs");
    let render_state = read_runtime_src("text/render_state.rs");
    let parent_production = production_code_view(&parent);
    let font_id_report_production = production_code_view(&font_id_report);
    let render_state_production = production_code_view(&render_state);

    assert_contains_all(
        "screen-space UI text parent keeps native/SDF orchestration and child mount",
        &parent_production,
        &[
            "mod font_id_report;",
            "mod native_glyph_run;",
            "mod prepare_report;",
            "use self::font_id_report::ScreenSpaceUiTextFontIdReport;",
            "use self::native_glyph_run::native_bitmap_atlas_glyph_runs;",
            "ScreenSpaceUiTextSystem",
            "ScreenSpaceUiTextPrepareReport",
            "let native_font_id_report = self.native.prepare(",
            "font_ids: glyph_run_projection.font_ids",
        ],
    );
    for moved_owner in [
        "pub(crate) struct ScreenSpaceUiTextFontIdReport",
        "fn accumulate_resolved_glyph_faces(",
    ] {
        assert!(
            !parent_production.contains(moved_owner),
            "scene_renderer/ui/text.rs should delegate font-id report owner `{moved_owner}` to text/font_id_report.rs"
        );
        assert!(
            font_id_report_production.contains(moved_owner),
            "text/font_id_report.rs should own moved font-id report item `{moved_owner}`"
        );
    }
    assert_contains_all(
        "screen-space UI text font-id child owns canonical shaped-glyph reporting",
        &font_id_report_production,
        &[
            "pub(crate) struct ScreenSpaceUiTextFontIdReport",
            "pub(super) fn accumulate_resolved_glyph_faces",
            "faces: impl IntoIterator<Item = Option<FontFaceId>>",
            "let mut primary = None;",
            "Some(face) if primary.is_none()",
            "Some(_) => fallback_glyph_count += 1",
            "unmapped_glyph_count",
        ],
    );
    assert_contains_all(
        "native glyph-run projection resolves handles once and reuses them for diagnostics",
        &native_glyph_run,
        &[
            "resolve_font_handle_batch(",
            "accumulate_resolved_glyph_faces(font_ids, handles.iter().map(|(face, _)| *face));",
        ],
    );
    assert!(
        !parent_production.contains("text_state.font_database()"),
        "graphics production must not expose the Text-owned database for font-id reporting"
    );
    assert!(
        !font_id_report_production.contains("glyphon::"),
        "graphics font-id reporting must consume canonical glyph handles rather than glyphon"
    );
    for moved_report_owner in [
        "pub(crate) struct ScreenSpaceUiTextPrepareReport",
        "pub(crate) struct ScreenSpaceUiTextRasterUploadReport",
        "fn text_raster_upload_report(",
    ] {
        assert!(
            !parent.contains(moved_report_owner),
            "scene_renderer/ui/text.rs should delegate prepare-report owner `{moved_report_owner}`"
        );
        assert!(
            prepare_report.contains(moved_report_owner),
            "text/prepare_report.rs should own `{moved_report_owner}`"
        );
    }
    for forbidden_bridge in ["shape_horizontal_range", "annotate_fallback_font_ids"] {
        assert!(
            !font_id_report.contains(forbidden_bridge),
            "text/font_id_report.rs must not retain post-shape bridge `{forbidden_bridge}`"
        );
    }
    assert!(
        !text_module.contains("NativeTextFontIdReport"),
        "text/mod.rs must not retain the removed NativeTextFontIdReport re-export"
    );
    assert!(
        !render_state_production.contains("pub(crate) fn font_database(&self) -> &FontDatabase"),
        "the full Text font database accessor must remain test-only"
    );

    for (path, source) in [
        ("graphics/scene/scene_renderer/ui/text.rs", parent.as_str()),
        (
            "graphics/scene/scene_renderer/ui/text/font_id_report.rs",
            font_id_report.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/ui/text/prepare_report.rs",
            prepare_report.as_str(),
        ),
        ("text/mod.rs", text_module.as_str()),
        ("text/render_state.rs", render_state.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}
