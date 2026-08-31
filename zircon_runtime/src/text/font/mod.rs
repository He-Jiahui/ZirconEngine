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
mod line_metrics;
mod matching;
mod query;
mod runtime_asset;
mod shared;
mod source_manifest;
#[cfg(test)]
mod test_font_fixtures;
mod vertical_metrics;

pub(crate) use database::{
    FontAssetUpdateReport, FontDatabase, FontDatabaseError, SystemFontPolicy,
};
pub(crate) use decoration_metrics::{
    TextDecorationKind, TextDecorationMetrics, TextDecorationMetricsCache, text_decoration_frame,
};
#[cfg(test)]
pub(crate) use default_families::default_runtime_font_families;
#[cfg(test)]
pub(crate) use fallback::FallbackResolutionSource;
pub(crate) use fallback::{FallbackResolution, MissingGlyphDiagnosticsReport};
#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
pub(crate) use fallback_cache::FallbackCacheRequestProfile;
#[cfg(test)]
pub(crate) use handle_registry::current_thread_font_handle_registration_batch_count;
pub(crate) use handle_registry::{
    FontHandleRegistrationBatchReport, FontHandleRegistryReport, FontHandleResolverSnapshot,
    font_handle_registry_report, font_handle_resolver_snapshot, register_font_face_handle,
    register_font_handle_batch, register_font_handle_batch_for_collection,
    register_font_handle_batch_with_report, register_font_handle_batch_with_report_for_collection,
    register_font_handles, register_font_instance_handle, resolve_font_face_handle,
    resolve_font_handle_batch, resolve_font_handle_batch_for_collection,
    resolve_font_handle_batch_from_snapshot, resolve_font_handles, resolve_font_instance_handle,
};
pub(crate) use line_metrics::{
    SelectedFaceLineEnvelope, SelectedFaceLineExtents, font_chain_line_metric_envelope,
    primary_face_covers_all_hard_line_content,
};
pub(crate) use query::font_query_for_text_style;
pub(crate) use runtime_asset::{
    RuntimeFontAssetAdmissionError, RuntimeFontAssetAdmissionReport, RuntimeFontAssetClaimScope,
    RuntimeFontAssetClaimUpdateReport, prepare_runtime_font_asset_admission,
};
pub(crate) use shared::{
    FontCollectionRevision, FontCollectionService, FontCollectionSnapshot,
    shared_font_collection_handle, shared_font_collection_service, shared_font_collection_snapshot,
    shared_font_database_generation, shared_font_database_snapshot,
};
#[cfg(test)]
pub(crate) use shared::{
    force_publish_shared_font_database, runtime_default_font_database_for_test,
    shared_font_database_test_read_guard, shared_font_database_test_serial_guard,
};
pub(crate) use source_manifest::{
    FontLoadError, FontLoadIoFailure, LoadedTextFontSource, load_text_font_source,
};
pub(crate) use vertical_metrics::FontVerticalMetrics;

pub(crate) const DEFAULT_UI_FONT_ASSET: &str = "res://fonts/default.font.toml";
