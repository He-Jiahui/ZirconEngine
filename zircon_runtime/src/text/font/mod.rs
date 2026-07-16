//! Shared runtime font database used by text shaping and raster paths.

mod asset_registration;
mod backend;
mod composite_resolve;
mod coverage;
mod database;
mod decoration_metrics;
mod default_families;
mod descriptors;
mod fallback;
mod handle_registry;
mod instance;
mod matching;
mod shared;
mod source_manifest;
#[cfg(test)]
mod test_font_fixtures;
mod vertical_metrics;

pub(crate) use database::{FontDatabase, SystemFontPolicy};
pub(crate) use decoration_metrics::{
    text_decoration_frame, TextDecorationKind, TextDecorationMetrics, TextDecorationMetricsCache,
};
#[cfg(test)]
pub(crate) use default_families::default_runtime_font_families;
pub(crate) use fallback::MissingGlyphDiagnosticsReport;
pub(crate) use handle_registry::{
    register_font_face_handle, register_font_instance_handle, resolve_font_face_handle,
    resolve_font_instance_handle,
};
pub(crate) use shared::{
    publish_shared_font_database, shared_font_database_generation, shared_font_database_snapshot,
};
pub(crate) use source_manifest::{load_text_font_source, LoadedTextFontSource};
