//! Shared runtime font database used by text shaping and raster paths.

mod asset_registration;
mod backend;
mod composite_resolve;
mod coverage;
mod database;
mod decoration_metrics;
mod default_families;
mod descriptors;
mod face_metadata;
mod fallback;
mod fallback_cache;
mod handle_registry;
mod instance;
mod matching;
mod shared;
mod source_manifest;
#[cfg(test)]
mod test_font_fixtures;
mod vertical_metrics;

pub(crate) use database::{FontAssetUpdateReport, FontDatabase, SystemFontPolicy};
pub(crate) use decoration_metrics::{
    text_decoration_frame, TextDecorationKind, TextDecorationMetrics, TextDecorationMetricsCache,
};
#[cfg(test)]
pub(crate) use default_families::default_runtime_font_families;
pub(crate) use fallback::MissingGlyphDiagnosticsReport;
pub(crate) use handle_registry::{
    font_handle_registry_report, register_font_face_handle, register_font_handle_batch,
    register_font_handles, register_font_instance_handle, resolve_font_face_handle,
    resolve_font_handle_batch, resolve_font_handles, resolve_font_instance_handle,
};
pub(crate) use shared::{
    mutate_shared_font_database, shared_font_database_generation, shared_font_database_snapshot,
};
#[cfg(test)]
pub(crate) use shared::{
    shared_font_database_test_read_guard, shared_font_database_test_serial_guard,
};
pub(crate) use source_manifest::{load_text_font_source, LoadedTextFontSource};
pub(crate) use vertical_metrics::FontVerticalMetrics;
