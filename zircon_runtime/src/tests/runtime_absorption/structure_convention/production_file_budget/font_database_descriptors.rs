use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_font_database_descriptor_helpers_are_child_owner() {
    let font_mod = read_runtime_src("graphics/text/font/mod.rs");
    let database = read_runtime_src("graphics/text/font/database.rs");
    let descriptors = read_runtime_src("graphics/text/font/descriptors.rs");
    let matching = read_runtime_src("graphics/text/font/matching.rs");
    let asset_registration = read_runtime_src("graphics/text/font/asset_registration.rs");
    let fallback = read_runtime_src("graphics/text/font/fallback.rs");
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
        "font module mounts descriptor child beside database owner",
        &font_mod,
        &[
            "mod asset_registration;",
            "mod database;",
            "mod descriptors;",
            "mod matching;",
            "pub(crate) use database::{FontDatabase, SystemFontPolicy};",
        ],
    );
    assert_contains_all(
        "font database keeps storage, indexes, matching, and system registration orchestration",
        &database,
        &[
            "use super::descriptors::{",
            "use super::matching::{",
            "descriptor_from_font_bytes",
            "descriptor_from_fontdb_face",
            "source_key_from_fontdb_source",
            "pub(super) struct FontSourceKey",
            "source_face_index: HashMap<FontSourceKey, FontFaceId>",
            "fn register_system_face(",
            "fn match_face_in_family_order(",
        ],
    );
    for moved_descriptor_owner in [
        "fn family_from_source_path(",
        "pub(super) fn descriptor_from_font_bytes(",
        "\nfn face_family_name(",
        "fn ttf_name_by_id(",
        "fn style_from_ttf(",
        "pub(super) fn stretch_from_ttf_width_class(",
        "pub(super) fn descriptor_from_fontdb_face(",
        "fn style_from_fontdb(",
        "pub(super) fn source_key_from_fontdb_source(",
    ] {
        assert!(
            !database.contains(moved_descriptor_owner),
            "graphics/text/font/database.rs should delegate descriptor helper `{moved_descriptor_owner}` to descriptors.rs"
        );
        assert!(
            descriptors.contains(moved_descriptor_owner),
            "graphics/text/font/descriptors.rs should own moved descriptor helper `{moved_descriptor_owner}`"
        );
    }
    for moved_matching_owner in [
        "pub(super) fn dedupe_families(",
        "pub(super) fn weight_distance(",
        "pub(super) fn stretch_distance(",
        "pub(super) fn style_distance(",
    ] {
        assert!(
            !database.contains(moved_matching_owner),
            "graphics/text/font/database.rs should delegate matching helper `{moved_matching_owner}` to matching.rs"
        );
        assert!(
            matching.contains(moved_matching_owner),
            "graphics/text/font/matching.rs should own moved matching helper `{moved_matching_owner}`"
        );
    }
    assert_contains_all(
        "font descriptor child owns TTF/fontdb projection",
        &descriptors,
        &[
            "use ttf_parser::{name_id, Face, Style as TtfStyle};",
            "use super::database::FontSourceKey;",
            "Face::parse(bytes, face_index)",
            "FontFaceDescriptor {",
            "VariationCoords::default()",
            "fontdb::Source::SharedFile(path, _)",
            "FontSourceKey::from_path(path, face_index)",
        ],
    );
    assert_contains_all(
        "font matching child owns family dedupe and query distance helpers",
        &matching,
        &[
            "pub(super) fn dedupe_families(",
            "pub(super) fn weight_distance(",
            "pub(super) fn stretch_distance(",
            "pub(super) fn style_distance(",
            "normalized_family_key(family.as_str())",
        ],
    );
    assert_contains_all(
        "asset registration consumes descriptor child through narrow imports",
        &asset_registration,
        &[
            "use super::database::{canonical_source_key, normalized_family_key};",
            "use super::descriptors::{descriptor_from_font_bytes, stretch_from_ttf_width_class};",
            "descriptor_from_font_asset_member(",
            "variation_coords_from_font_asset(",
        ],
    );
    assert_contains_all(
        "fallback resolver consumes family dedupe through matching child",
        &fallback,
        &[
            "use super::matching::dedupe_families;",
            "FallbackResolver",
            "candidates_for_codepoint",
        ],
    );

    for (path, source) in [
        ("graphics/text/font/database.rs", database.as_str()),
        ("graphics/text/font/descriptors.rs", descriptors.as_str()),
        ("graphics/text/font/matching.rs", matching.as_str()),
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
                "Runtime 15 M4 font database descriptor helper owner split",
                "runtime_15_font_database_descriptor_helper_owner_split_static_passed_cargo_deferred",
                "graphics/text/font/database.rs",
                "graphics/text/font/descriptors.rs",
                "graphics/text/font/matching.rs",
                "runtime_15_font_database_descriptor_helpers_are_child_owner",
            ],
        );
    }
}
