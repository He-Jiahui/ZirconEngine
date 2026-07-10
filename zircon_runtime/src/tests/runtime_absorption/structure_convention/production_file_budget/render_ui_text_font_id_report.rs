use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_text_font_id_report_is_child_owner() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let font_id_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/font_id_report.rs");
    let prepare_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/prepare_report.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let graphics_text_doc = read_repo("docs/zircon_runtime/graphics/text.md");
    let ui_text_doc = read_repo("docs/zircon_runtime/ui/text.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "screen-space UI text parent keeps native/SDF orchestration and child mount",
        &parent,
        &[
            "mod font_id_report;",
            "mod prepare_report;",
            "use self::font_id_report::{",
            "ScreenSpaceUiTextSystem",
            "ScreenSpaceUiTextPrepareReport",
            "let native_font_id_report = self.native.prepare(",
            "accumulate_text_font_id_report(",
            "native_text_align(",
        ],
    );
    for moved_owner in [
        "pub(super) struct ScreenSpaceUiTextFontIdReport",
        "fn resolved_style_for_text_batch(",
        "fn accumulate_text_font_id_report(",
        "fn accumulate_backend_glyphs(",
        "font_database.font_face_id(glyph.font_id)",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene_renderer/ui/text.rs should delegate font-id report owner `{moved_owner}` to text/font_id_report.rs"
        );
        assert!(
            font_id_report.contains(moved_owner),
            "text/font_id_report.rs should own moved font-id report item `{moved_owner}`"
        );
    }
    assert_contains_all(
        "screen-space UI text font-id child owns actual backend reporting",
        &font_id_report,
        &[
            "pub(super) struct ScreenSpaceUiTextFontIdReport",
            "pub(super) fn accumulate_text_font_id_report",
            "UiResolvedStyle",
            "font_query_for_style",
            "buffer.layout_runs()",
            "font_database.font_face_id(glyph.font_id)",
            "unmapped_glyph_count",
            "resolve_text_render_mode(UiTextRenderMode::Native, None)",
        ],
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
    for forbidden_bridge in ["shape_horizontal_line", "annotate_fallback_font_ids"] {
        assert!(
            !font_id_report.contains(forbidden_bridge),
            "text/font_id_report.rs must not retain post-shape bridge `{forbidden_bridge}`"
        );
    }

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
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("graphics text doc", graphics_text_doc.as_str()),
        ("UI text doc", ui_text_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 screen-space UI text font-id report owner split",
                "runtime_15_screen_space_ui_text_font_id_report_owner_split_static_passed_cargo_deferred",
                "graphics/scene/scene_renderer/ui/text.rs",
                "graphics/scene/scene_renderer/ui/text/font_id_report.rs",
                "runtime_15_screen_space_ui_text_font_id_report_is_child_owner",
            ],
        );
    }
}
