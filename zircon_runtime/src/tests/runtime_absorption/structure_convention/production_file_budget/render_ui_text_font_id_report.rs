use super::super::rust_source_view::production_code_view;
use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_text_font_id_report_is_child_owner() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/text.rs");
    let font_id_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/font_id_report.rs");
    let prepare_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/prepare_report.rs");
    let text_module = read_runtime_src("text/mod.rs");
    let native_buffer = read_runtime_src("text/native_buffer.rs");
    let render_state = read_runtime_src("text/render_state.rs");
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let graphics_text_doc = read_repo("docs/zircon_runtime/graphics/text.md");
    let ui_text_doc = read_repo("docs/zircon_runtime/ui/text.md");
    let parent_production = production_code_view(&parent);
    let font_id_report_production = production_code_view(&font_id_report);
    let render_state_production = production_code_view(&render_state);

    assert_contains_all(
        "screen-space UI text parent keeps native/SDF orchestration and child mount",
        &parent_production,
        &[
            "mod font_id_report;",
            "mod prepare_report;",
            "use self::font_id_report::{",
            "ScreenSpaceUiTextSystem",
            "ScreenSpaceUiTextPrepareReport",
            "let native_font_id_report = self.native.prepare(",
            "accumulate_text_font_id_report(",
            "native_buffer.primary_face",
            "|backend| text_state.font_face_id(backend)",
            "native_text_align(",
        ],
    );
    for moved_owner in [
        "pub(crate) struct ScreenSpaceUiTextFontIdReport",
        "fn accumulate_text_font_id_report(",
        "fn accumulate_backend_glyphs(",
        "resolve_face: impl FnMut(glyphon::fontdb::ID) -> Option<FontFaceId>",
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
        "screen-space UI text font-id child owns actual backend reporting",
        &font_id_report_production,
        &[
            "pub(crate) struct ScreenSpaceUiTextFontIdReport",
            "pub(super) fn accumulate_text_font_id_report",
            "primary: Option<FontFaceId>",
            "resolve_face: impl FnMut(glyphon::fontdb::ID) -> Option<FontFaceId>",
            "buffer.layout_runs()",
            "match (resolve_face(glyph.font_id), primary)",
            "(Some(_), _) => fallback_glyph_count += 1",
            "unmapped_glyph_count",
        ],
    );
    assert!(
        !parent_production.contains("text_state.font_database()"),
        "graphics production must resolve backend font IDs through the narrow TextRenderState query"
    );
    assert!(
        !font_id_report_production.contains("FontDatabase"),
        "graphics font-id reporting must receive a narrow resolver instead of the Text-owned database"
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
    for duplicate_owner in ["NativeTextFontIdReport", "fn font_id_report(", ".font_ids"] {
        assert!(
            !native_buffer.contains(duplicate_owner),
            "text/native_buffer.rs must not duplicate render-only font-id reporting owner `{duplicate_owner}`"
        );
    }
    assert!(
        !text_module.contains("NativeTextFontIdReport"),
        "text/mod.rs must not retain the removed NativeTextFontIdReport re-export"
    );
    assert_contains_all(
        "code text resolves the same default family installed as the glyphon monospace family",
        &parent,
        &[
            "fn resolve_family_name(",
            "code: bool",
            "if code {",
            "DEFAULT_FONT_ASSET,",
            "record.family.clone()",
        ],
    );
    assert_contains_all(
        "native text preparation exposes only canonical primary-face metadata to the render report owner",
        &native_buffer,
        &[
            "pub(crate) primary_face: Option<FontFaceId>",
            "fn native_font_query(",
            "if request.code {",
            "Family::Monospace",
            "style: if request.emphasis",
            "requested_weight.max(FontWeight::BOLD)",
        ],
    );
    assert_contains_all(
        "text render state exposes a narrow font-face query to the render-only report owner",
        &render_state_production,
        &[
            "pub(crate) fn set_default_ui_family(&mut self, family: &str)",
            "mutate_shared_font_database(|database| database.set_default_ui_family(family))",
            "pub(crate) fn font_face_id(&self, backend: glyphon::fontdb::ID)",
            "self.font_database.font_face_id(backend)",
        ],
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
        ("text/native_buffer.rs", native_buffer.as_str()),
        ("text/render_state.rs", render_state.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}
