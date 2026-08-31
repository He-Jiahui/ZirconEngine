use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_font_database_descriptor_helpers_are_child_owner() {
    let font_mod = read_runtime_src("text/font/mod.rs");
    let database = read_runtime_src("text/font/database.rs");
    let descriptors = read_runtime_src("text/font/descriptors.rs");
    let face_metadata = read_runtime_src("text/font/face_metadata.rs");
    let coverage = read_runtime_src("text/font/coverage.rs");
    let matching = read_runtime_src("text/font/matching.rs");
    let asset_registration = read_runtime_src("text/font/asset_registration.rs");
    let system_fonts = read_runtime_src("text/font/database/system_fonts.rs");
    let face_access = read_runtime_src("text/font/database/face_access.rs");
    let face_matching = read_runtime_src("text/font/database/face_matching.rs");
    let fallback_queries = read_runtime_src("text/font/database/fallback_queries.rs");
    let sdf_distance_field = read_runtime_src("text/sdf/font_bake/distance_field.rs");
    let sdf_font_bake = read_runtime_src("text/sdf/font_bake.rs");
    let sdf_glyph_metrics = read_runtime_src("text/sdf/font_bake/glyph_metrics.rs");
    let sdf_offline_source = read_runtime_src("text/sdf/font_bake/offline_source.rs");
    let native_bitmap_atlas = read_runtime_src("text/native_bitmap_atlas.rs");
    let native_bitmap_glyph_run =
        read_runtime_src("graphics/scene/scene_renderer/ui/text/native_glyph_run.rs");
    let native_bitmap_source_image = read_runtime_src("text/native_bitmap_atlas/source_image.rs");
    let composite_resolve = read_runtime_src("text/font/composite_resolve.rs");
    let fallback = read_runtime_src("text/font/fallback.rs");
    let fallback_cache = read_runtime_src("text/font/fallback_cache.rs");

    assert_contains_all(
        "font module mounts metadata and fallback cache beside database orchestration",
        &font_mod,
        &[
            "mod asset_registration;",
            "mod database;",
            "mod descriptors;",
            "mod face_metadata;",
            "mod fallback_cache;",
            "mod matching;",
            "pub(crate) use database::{FontDatabase, SystemFontPolicy};",
        ],
    );
    assert_contains_all(
        "font database keeps generation storage and delegates focused owners",
        &database,
        &[
            "use super::descriptors::descriptor_from_font_metadata;",
            "use super::face_metadata::FontFaceMetadata;",
            "use super::fallback_cache::{",
            "mod error;",
            "mod face_access;",
            "mod face_matching;",
            "mod fallback_queries;",
            "mod instances;",
            "mod system_fonts;",
            "metadata: Arc<OnceLock<FontFaceMetadata>>",
            "source_bytes: Arc<OnceLock<Arc<[u8]>>>",
            "standalone_bytes: Arc<OnceLock<Arc<[u8]>>>",
            "fallback_family_identities: HashSet<FontFamilyIdentity>",
            "effective_instances: EffectiveInstanceCache",
            "fallback_caches: FallbackCaches",
            "fn detach_face_dependent_caches(&mut self)",
            "self.fallback_caches = FallbackCaches::default();",
        ],
    );
    assert!(
        database.lines().count() <= 800,
        "text/font/database.rs must stay within the production soft budget"
    );
    assert_contains_all(
        "face matching and its bounded cache are a focused database child owner",
        &face_matching,
        &[
            "pub(super) struct FontMatchCacheKey",
            "const MAX_FACE_MATCH_CACHE_ENTRIES: usize = 64;",
            "pub(crate) fn match_face(&self, query: &FontQuery)",
            "fn family_candidates(",
            "pub(in crate::text::font) fn family_candidates_for_codepoint(",
        ],
    );
    assert_contains_all(
        "fallback queries and cache adapters are a focused database child owner",
        &fallback_queries,
        &[
            "pub(crate) struct FontShapingFaceResolver",
            "pub(crate) fn begin_shaping_face_resolution",
            "pub(crate) fn fallback_cache_report",
            "pub(in crate::text::font) fn fallback_composite_index",
            "pub(in crate::text::font) fn cache_fallback_resolution",
        ],
    );
    for child_owned_behavior in [
        "pub(crate) struct FontShapingFaceResolver",
        "pub(crate) fn match_face(&self, query: &FontQuery)",
        "pub(crate) fn fallback_cache_report",
    ] {
        assert!(
            !database.contains(child_owned_behavior),
            "text/font/database.rs should delegate `{child_owned_behavior}`"
        );
    }
    for child_owned_helper in [
        "fn family_from_source_path(",
        "pub(super) fn descriptor_from_font_metadata(",
        "pub(super) fn stretch_from_ttf_width_class(",
        "pub(super) fn descriptor_from_fontdb_face(",
        "pub(super) fn source_key_from_fontdb_source(",
    ] {
        assert!(
            !database.contains(child_owned_helper),
            "text/font/database.rs should delegate `{child_owned_helper}`"
        );
        assert!(
            descriptors.contains(child_owned_helper),
            "text/font/descriptors.rs should own `{child_owned_helper}`"
        );
    }
    assert_contains_all(
        "face metadata is the single SFNT parse and projection owner",
        &face_metadata,
        &[
            "use ttf_parser::{name_id, Face, GlyphId, Style as TtfStyle};",
            "pub(super) struct FontFaceMetadata",
            "Face::parse(bytes, face_index)",
            "let glyph_map = FontGlyphMap::from_face(&face);",
            "coverage: FontCoverage::from_codepoint_values(glyph_map.codepoints())",
            "fn vertical_advances(",
            "fn face_metrics(",
            "fn face_family_name(",
        ],
    );
    assert!(
        !descriptors.contains("Face::parse("),
        "descriptor projection must consume owned face metadata instead of reparsing SFNT bytes"
    );
    assert_contains_all(
        "descriptor and coverage children consume the metadata parse boundary",
        &descriptors,
        &[
            "use super::face_metadata::FontFaceMetadata;",
            "descriptor_from_font_metadata(",
            "fontdb::Source::SharedFile(path, _)",
        ],
    );
    assert_contains_all(
        "coverage compacts the metadata owner's projected codepoints",
        &coverage,
        &[
            "pub(super) fn from_codepoint_values(mut codepoints: Vec<u32>)",
            "compact_codepoint_ranges",
        ],
    );
    assert_contains_all(
        "asset registration builds one metadata artifact per declared face",
        &asset_registration,
        &[
            "use super::descriptors::{descriptor_from_font_metadata, stretch_from_ttf_width_class};",
            "let metadata = FontFaceMetadata::from_sfnt_bytes",
            "FontAssetFaceRegistration {",
        ],
    );
    assert_contains_all(
        "system font registration remains a focused lazy child",
        &system_fonts,
        &[
            "fn register_system_face(",
            "descriptor_from_fontdb_face(info)",
            "Arc::new(OnceLock::new())",
        ],
    );
    assert_contains_all(
        "face access exposes metadata and coverage only to font siblings",
        &face_access,
        &[
            "pub(in crate::text::font) fn face_metadata(",
            "pub(in crate::text::font) fn face_covers_all(",
            "pub(in crate::text::font) fn face_covers_codepoint(",
            "pub(in crate::text::font) fn face_coverage_count(",
            "pub(in crate::text) fn face_glyph_id(",
            "stored.source_bytes.get()",
            "stored.standalone_bytes.get()",
        ],
    );
    assert!(
        !face_access.contains("pub(crate) fn face_metadata("),
        "face metadata access must not widen beyond the text::font owner"
    );
    assert!(
        !sdf_distance_field.contains("Face::parse(")
            && !sdf_offline_source.contains("Face::parse("),
        "SDF font-bake consumers must use generation-owned glyph metadata without reparsing"
    );
    assert!(
        sdf_font_bake.lines().count() <= 800,
        "text/sdf/font_bake.rs must stay within the production soft budget"
    );
    assert_contains_all(
        "SDF display metric projection is a focused font-bake child owner",
        &sdf_glyph_metrics,
        &[
            "pub(super) fn glyph_metrics(",
            "pub(crate) fn scale_sdf_metrics_for_display(",
            "pub(super) fn fallback_metrics(",
        ],
    );
    assert!(
        sdf_font_bake.contains("mod glyph_metrics;")
            && !sdf_font_bake.contains("fn scale_bitmap_dimension("),
        "font_bake root must delegate display metric projection"
    );
    assert_contains_all(
        "SDF offline source caches manifest hits and misses for the font generation",
        &sdf_offline_source,
        &[
            "manifests: HashMap<String, Option<LoadedTextFontSource>>",
            "fn load_manifest_cached(",
        ],
    );
    assert!(
        !sdf_offline_source.contains("register_loaded_font_manifest("),
        "offline glyph lookup must not reload and re-register the same font asset"
    );
    assert!(
        native_bitmap_atlas.lines().count() <= 800,
        "text/native_bitmap_atlas.rs must stay within the production soft budget"
    );
    assert_contains_all(
        "native bitmap stable raster identity projects shaped glyphs before atlas ownership",
        &native_bitmap_glyph_run,
        &[
            "pub(in crate::graphics::scene::scene_renderer::ui) fn native_bitmap_atlas_glyph_runs(",
            "GlyphRasterKey::from_request",
            "glyph_artifact_line",
            "vertical_subpixel_bin",
        ],
    );
    assert!(
        native_bitmap_atlas.contains("mod glyph_run;")
            && !native_bitmap_atlas.contains("GlyphRasterKey::from_request"),
        "native bitmap atlas root must consume prepared raster identities"
    );
    assert_contains_all(
        "native bitmap source-image projection is a focused child owner",
        &native_bitmap_source_image,
        &[
            "pub(super) struct NativeBitmapGlyphImage",
            "pub(super) fn native_bitmap_atlas_source_from_image(",
            "pub(super) fn native_bitmap_atlas_format(",
        ],
    );
    assert!(
        native_bitmap_atlas.contains("mod source_image;")
            && !native_bitmap_atlas.contains("struct NativeBitmapGlyphImage")
            && !native_bitmap_atlas.contains("fn native_bitmap_atlas_source_from_image("),
        "native bitmap atlas root must delegate source-image projection"
    );
    assert_contains_all(
        "matching owns stable family identity and linear dedupe",
        &matching,
        &[
            "pub(super) struct FontFamilyIdentity",
            "pub(super) fn font_family_identity(",
            "let mut identities = HashSet::new();",
            "pub(super) fn dedupe_families(",
        ],
    );
    assert_contains_all(
        "composite resolution uses a generation-owned compiled lookup index",
        &composite_resolve,
        &[
            "pub(super) struct CompositeFontIndex",
            "pub(super) fn compile(composite: &CompositeFontDescriptor)",
            ".partition_point(|range| range.start <= codepoint)",
            "dedupe_families(families)",
        ],
    );
    assert_contains_all(
        "fallback cache keys use fixed identities and bounded storage",
        &fallback_cache,
        &[
            "pub(super) struct CompositeFontIdentity",
            "pub(super) struct FallbackQueryIdentity",
            "struct BoundedCache<K, V>",
            "normalization_allocation_count: u64",
            "face_visit_count: u64",
            "pub(super) fn fallback_query_identity(",
            "pub(super) fn fallback_candidate_cache_key(",
        ],
    );
    assert_contains_all(
        "fallback resolver consumes compiled composite candidates",
        &fallback,
        &[
            "candidate_faces_for_cluster",
            "CompositeFontIndex",
            "query_identity: FallbackQueryIdentity",
            "FallbackResolver",
            "candidates_for_codepoint",
        ],
    );
}
