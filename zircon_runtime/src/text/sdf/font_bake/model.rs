use std::sync::Arc;

use crate::core::framework::text::TextFontFaceHandle;
use crate::text::atlas::GlyphAtlasPageKey;
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::{SdfBakeParams, SdfGenerationSchedulerDiagnostics, SdfGlyphGenerationError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfAtlasRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SdfAtlasGlyphKey {
    pub(crate) glyph: char,
    pub(crate) glyph_id: Option<u32>,
    pub(crate) font_id: Option<TextFontFaceHandle>,
    pub(crate) font_instance_id: Option<TextFontFaceHandle>,
    pub(crate) font: Option<Arc<str>>,
    pub(crate) font_family: Option<Arc<str>>,
    pub(crate) language: Option<Arc<str>>,
    pub(crate) font_weight: u16,
    pub(crate) bake_params: SdfBakeParams,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasSlot {
    pub(crate) key: SdfAtlasGlyphKey,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) rect: SdfAtlasRect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfShapedGlyphIdentity {
    pub(crate) glyph_id: u32,
    pub(crate) font_id: Option<TextFontFaceHandle>,
    pub(crate) font_instance_id: Option<TextFontFaceHandle>,
}

pub(crate) trait SdfTextRun {
    fn font(&self) -> Option<&str>;
    fn font_family(&self) -> Option<&str>;
    fn language(&self) -> Option<&str>;
    fn font_weight(&self) -> u16;
    fn font_size(&self) -> f32;
    fn render_scalars(&self) -> Vec<char>;
    fn resolved_glyph_advances(&self) -> Option<Vec<f32>>;
    fn shaped_glyph(&self, glyph_index: usize) -> Option<SdfShapedGlyphIdentity>;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SdfRunCpuPreparation {
    pub(crate) glyph_metrics: Vec<SdfGlyphMetrics>,
    pub(crate) glyph_advances: Vec<f32>,
    pub(crate) decoration_metrics: TextDecorationMetrics,
}

#[derive(Clone, Debug)]
pub(crate) struct SdfAtlasBake {
    pub(crate) pages: Arc<[SdfAtlasBakePage]>,
    pub(crate) dirty_pages: Arc<[SdfAtlasBakeDirtyPage]>,
    pub(crate) glyphs: Arc<[SdfBakedGlyph]>,
    pub(crate) generation_failures: Arc<[SdfAtlasGlyphGenerationFailure]>,
    pub(crate) report: SdfAtlasBakeReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasBakePage {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) source_offset: usize,
    pub(crate) byte_len: usize,
    pub(crate) pixels: Arc<[u8]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasBakeDirtyPage {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) dirty_rect: SdfAtlasRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasGlyphGenerationFailure {
    pub(crate) slot_index: usize,
    pub(crate) key: SdfAtlasGlyphKey,
    pub(crate) error: SdfGlyphGenerationError,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfAtlasBakeReport {
    pub(crate) slot_count: usize,
    pub(crate) visible_glyph_count: usize,
    pub(crate) empty_glyph_count: usize,
    pub(crate) atlas_byte_len: usize,
    pub(crate) nonzero_pixel_count: usize,
    /// Materialized faces retained by the cache after this atlas build.
    pub(crate) resident_font_count: usize,
    /// Faces materialized by this atlas build rather than reused from the cache.
    pub(crate) loaded_font_count: usize,
    pub(crate) generation_failure_count: usize,
    pub(crate) resident_font_asset_error_count: usize,
    pub(crate) resident_font_asset_no_registered_faces_count: usize,
    pub(crate) r8_byte_len: usize,
    pub(crate) rgba_byte_len: usize,
    pub(crate) offline_glyph_count: usize,
    pub(crate) dynamic_glyph_count: usize,
    pub(crate) offline_resident_manifest_count: usize,
    pub(crate) offline_resident_artifact_identity_count: usize,
    pub(crate) offline_resident_artifact_byte_count: usize,
    pub(crate) offline_resident_glyph_bitmap_count: usize,
    pub(crate) offline_resident_glyph_bitmap_byte_count: usize,
    pub(crate) offline_manifest_parse_count: usize,
    pub(crate) offline_artifact_stat_count: usize,
    pub(crate) offline_artifact_read_count: usize,
    pub(crate) offline_artifact_read_byte_count: usize,
    pub(crate) offline_artifact_decode_count: usize,
    pub(crate) offline_pixel_copy_count: usize,
    pub(crate) offline_pixel_copy_byte_count: usize,
    pub(crate) offline_manifest_eviction_count: usize,
    pub(crate) offline_artifact_eviction_count: usize,
    pub(crate) offline_glyph_bitmap_eviction_count: usize,
    pub(crate) offline_oldest_artifact_idle_access_count: u64,
    pub(crate) offline_oldest_glyph_bitmap_idle_access_count: u64,
    pub(crate) resident_baked_glyph_count: usize,
    pub(crate) resident_baked_glyph_byte_count: usize,
    pub(crate) baked_glyph_eviction_count: usize,
    pub(crate) oldest_baked_glyph_idle_access_count: u64,
    pub(crate) resident_source_context_count: usize,
    pub(crate) resident_source_byte_count: usize,
    pub(crate) source_context_created_count: usize,
    pub(crate) source_context_eviction_count: usize,
    pub(crate) oldest_source_context_idle_access_count: u64,
    pub(crate) source_hash_count: usize,
    pub(crate) face_parse_count: usize,
    pub(crate) generation_batch_count: usize,
    pub(crate) generation_requested_glyph_count: usize,
    pub(crate) generation_unique_glyph_count: usize,
    pub(crate) generation_duplicate_glyph_count: usize,
    pub(crate) bitmap_clone_byte_count: usize,
    pub(crate) resident_atlas_page_count: usize,
    pub(crate) atlas_page_alloc_count: usize,
    pub(crate) atlas_page_zero_byte_count: usize,
    pub(crate) atlas_page_clear_count: usize,
    pub(crate) atlas_page_clear_byte_count: usize,
    pub(crate) atlas_page_write_count: usize,
    pub(crate) atlas_page_write_byte_count: usize,
    pub(crate) atlas_page_reused_slot_count: usize,
    pub(crate) atlas_full_page_scan_byte_count: usize,
    pub(crate) compiled_atlas_build_count: usize,
    pub(crate) compiled_atlas_reuse_count: usize,
    pub(crate) generation_scheduler: SdfGenerationSchedulerDiagnostics,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SdfBakedGlyph {
    pub(crate) metrics: SdfGlyphMetrics,
    pub(crate) visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SdfGlyphMetrics {
    pub(crate) bitmap_width: u32,
    pub(crate) bitmap_height: u32,
    pub(crate) bitmap_left: f32,
    pub(crate) bitmap_bottom: f32,
    pub(crate) advance: f32,
    pub(crate) ascent: f32,
}
