//! Renderer-neutral signed-distance-field generation and decode contracts.

mod decode;
mod fdsm_gen;
pub(crate) mod font_bake;
mod generation_error;
mod generation_scheduler;
mod generation_source;
mod geometry_preprocess;
mod glyph_data;
mod mode;
mod offline;
mod params;

pub(crate) use generation_error::SdfGlyphGenerationError;
pub(crate) use generation_scheduler::{
    SdfGenerationCompletion, SdfGenerationCompletionDrainBudget, SdfGenerationInactiveWorkOutcome,
    SdfGenerationScheduler, SdfGenerationSchedulerDiagnostics, SdfGenerationSchedulerOptions,
    SdfGenerationSubmitError, SdfGenerationWorkId,
};
pub(crate) use generation_source::{
    SdfGenerationBatch, SdfGenerationBatchGlyph, SdfGenerationBatchReport,
    SdfGenerationSourceContext, SdfGenerationSourceHandle, SdfGenerationSourceReport,
};
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
pub(crate) use font_bake::{
    scale_sdf_metrics_for_display, sdf_scalar_is_invisible_format, sdf_scalar_requires_atlas_slot,
    SdfAtlasBake, SdfAtlasBakeDirtyPage, SdfAtlasBakePage, SdfAtlasBakeReport,
    SdfAtlasGlyphGenerationFailure, SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot, SdfBakedGlyph,
    SdfFontBakeCache, SdfGlyphMetrics, SdfRunCpuPreparation, SdfShapedGlyphIdentity, SdfTextRun,
};
