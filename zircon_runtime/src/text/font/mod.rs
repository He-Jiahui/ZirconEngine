//! Shared runtime font database used by text shaping and raster paths.

#[path = "asset_registration.rs"]
mod asset_registration;
#[path = "backend.rs"]
mod backend;
#[path = "composite_resolve.rs"]
mod composite_resolve;
#[path = "coverage.rs"]
mod coverage;
#[path = "database.rs"]
mod database;
#[path = "decoration_metrics.rs"]
mod decoration_metrics;
#[path = "default_families.rs"]
mod default_families;
#[path = "descriptors.rs"]
mod descriptors;
#[path = "face_metadata.rs"]
mod face_metadata;
#[path = "fallback.rs"]
mod fallback;
#[path = "fallback_cache.rs"]
mod fallback_cache;
#[path = "handle_registry.rs"]
mod handle_registry;
#[path = "instance.rs"]
mod instance;
#[path = "line_metrics.rs"]
mod line_metrics;
#[path = "matching.rs"]
mod matching;
#[path = "query.rs"]
mod query;
#[path = "runtime_asset.rs"]
mod runtime_asset;
#[path = "shared.rs"]
mod shared;
#[path = "source_manifest.rs"]
mod source_manifest;
#[cfg(test)]
#[path = "tests/test_font_fixtures.rs"]
mod test_font_fixtures;
#[path = "vertical_metrics.rs"]
mod vertical_metrics;

pub(crate) use database::{
    FontAssetUpdateReport, FontDatabase, FontDatabaseError, SystemFontPolicy,
};
pub(crate) use decoration_metrics::{
    text_decoration_frame, TextDecorationKind, TextDecorationMetrics, TextDecorationMetricsCache,
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
    font_handle_registry_report, font_handle_resolver_snapshot, register_font_face_handle,
    register_font_handle_batch, register_font_handle_batch_for_collection,
    register_font_handle_batch_with_report, register_font_handle_batch_with_report_for_collection,
    register_font_handles, register_font_instance_handle, resolve_font_face_handle,
    resolve_font_handle_batch, resolve_font_handle_batch_for_collection,
    resolve_font_handle_batch_from_snapshot, resolve_font_handles, resolve_font_instance_handle,
    FontHandleRegistrationBatchReport, FontHandleRegistryReport, FontHandleResolverSnapshot,
};
pub(crate) use line_metrics::{
    font_chain_line_metric_envelope, primary_face_covers_all_hard_line_content,
    SelectedFaceLineEnvelope, SelectedFaceLineExtents,
};
pub(crate) use query::font_query_for_text_style;
pub(crate) use runtime_asset::{
    prepare_runtime_font_asset_admission, RuntimeFontAssetAdmissionError,
    RuntimeFontAssetAdmissionReport, RuntimeFontAssetClaimScope, RuntimeFontAssetClaimUpdateReport,
};
#[cfg(test)]
pub(crate) use shared::{
    force_publish_shared_font_database, runtime_default_font_database_for_test,
    shared_font_database_test_read_guard, shared_font_database_test_serial_guard,
};
pub(crate) use shared::{
    shared_font_collection_handle, shared_font_collection_service, shared_font_collection_snapshot,
    shared_font_database_generation, shared_font_database_snapshot, FontCollectionRevision,
    FontCollectionService, FontCollectionSnapshot,
};
pub(crate) use source_manifest::{
    load_text_font_source, FontLoadError, FontLoadIoFailure, LoadedTextFontSource,
};
pub(crate) use vertical_metrics::FontVerticalMetrics;

pub(crate) const DEFAULT_UI_FONT_ASSET: &str = "res://fonts/default.font.toml";
