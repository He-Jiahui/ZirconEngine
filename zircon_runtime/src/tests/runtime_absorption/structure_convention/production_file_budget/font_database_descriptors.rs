use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_font_database_descriptor_helpers_are_child_owner() {
    let font_mod = read_runtime_src("text/font/mod.rs");
    let database = read_runtime_src("text/font/database.rs");
    let descriptors = read_runtime_src("text/font/descriptors.rs");
    let matching = read_runtime_src("text/font/matching.rs");
    let asset_registration = read_runtime_src("text/font/asset_registration.rs");
    let composite_resolve = read_runtime_src("text/font/composite_resolve.rs");
    let fallback = read_runtime_src("text/font/fallback.rs");

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
            "text/font/database.rs should delegate descriptor helper `{moved_descriptor_owner}` to descriptors.rs"
        );
        assert!(
            descriptors.contains(moved_descriptor_owner),
            "text/font/descriptors.rs should own moved descriptor helper `{moved_descriptor_owner}`"
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
            "text/font/database.rs should delegate matching helper `{moved_matching_owner}` to matching.rs"
        );
        assert!(
            matching.contains(moved_matching_owner),
            "text/font/matching.rs should own moved matching helper `{moved_matching_owner}`"
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
        "composite resolver consumes family dedupe through matching child",
        &composite_resolve,
        &[
            "use super::matching::dedupe_families;",
            "candidate_faces_for_cluster(",
            "dedupe_families(families)",
        ],
    );
    assert_contains_all(
        "fallback resolver consumes composite candidate projection",
        &fallback,
        &[
            "use super::composite_resolve::{candidate_faces_for_cluster, script_for_char};",
            "FallbackResolver",
            "candidates_for_codepoint",
        ],
    );
}
