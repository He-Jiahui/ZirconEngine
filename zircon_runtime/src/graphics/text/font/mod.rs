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
mod instance;
mod matching;
mod shared;
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
pub(crate) use shared::{
    publish_shared_font_database, shared_font_database_generation, shared_font_database_snapshot,
};
