//! Renderer-neutral signed-distance-field generation and decode contracts.

mod decode;
mod fdsm_gen;
mod generation_error;
mod geometry_preprocess;
mod glyph_data;
mod mode;
mod offline;
mod params;

pub(crate) use generation_error::SdfGlyphGenerationError;
pub(crate) use glyph_data::SdfGlyphData;
pub(crate) use mode::SdfMode;
pub(crate) use offline::{
    sdf_default_variation_hash, sdf_font_source_hash, sdf_offline_artifact_path,
    sdf_variation_hash, SdfOfflineArtifact, SdfOfflineArtifactError, SdfOfflineArtifactIdentity,
    SdfOfflineGlyph, SdfOfflineGlyphMetrics, SdfOfflinePage, SdfOfflineRect,
};
pub(crate) use params::SdfBakeParams;

#[cfg(test)]
mod tests;
pub(crate) use decode::{
    distance_field_coverage, median3, msdf_sample_distance, mtsdf_sample_true_distance,
};
pub(crate) use fdsm_gen::{
    generate_distance_field_glyph, generate_distance_field_glyph_with_variations,
};
