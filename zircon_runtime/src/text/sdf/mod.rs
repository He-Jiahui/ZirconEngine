//! Renderer-neutral signed-distance-field generation and decode contracts.

#[path = "decode.rs"]
mod decode;
#[path = "fdsm_gen.rs"]
mod fdsm_gen;
#[path = "font_bake.rs"]
pub(crate) mod font_bake;
#[path = "generation_error.rs"]
mod generation_error;
#[path = "generation_scheduler.rs"]
mod generation_scheduler;
#[path = "generation_source.rs"]
mod generation_source;
#[path = "geometry_preprocess.rs"]
mod geometry_preprocess;
#[path = "glyph_data.rs"]
mod glyph_data;
#[path = "mode.rs"]
mod mode;
#[path = "offline/mod.rs"]
mod offline;
#[path = "params.rs"]
mod params;

pub(crate) use generation_error::SdfGlyphGenerationError;
pub(crate) use generation_scheduler::{
    SdfGenerationBudgetSnapshot, SdfGenerationCompletion, SdfGenerationCompletionDrainBudget,
    SdfGenerationInactiveWorkOutcome, SdfGenerationScheduler, SdfGenerationSchedulerDiagnostics,
    SdfGenerationSchedulerOptions, SdfGenerationSubmitError, SdfGenerationWorkId,
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
#[path = "tests/cases.rs"]
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
