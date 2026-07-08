#[path = "d1_capability_single_source/audit_surfaces.rs"]
mod audit_surfaces;
#[path = "d1_capability_single_source/runtime_roots.rs"]
mod runtime_roots;
#[path = "d1_capability_single_source/sdk_builder.rs"]
mod sdk_builder;
#[path = "d1_capability_single_source/split_layout.rs"]
mod split_layout;
#[path = "d1_capability_single_source/status_docs.rs"]
mod status_docs;
#[path = "d1_capability_single_source/support.rs"]
mod support;

const D1_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 plugin-importer D1 capability single-source guard folder-backed split";
const D1_FOLDER_BACKED_STATUS: &str =
    "runtime_15_plugin_importer_d1_capability_guard_folder_backed_static_passed_cargo_deferred";
const D1_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_plugin_importer_d1_capability_guard_folder_backed_static_passed_cargo_deferred";
const D1_FOLDER_BACKED_GUARD: &str =
    "runtime_15_plugin_importer_d1_capability_guard_is_folder_backed";

#[test]
fn review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror() {
    runtime_roots::assert_runtime_capability_roots_use_single_source();
    audit_surfaces::assert_capability_audit_surfaces_are_wired();
    sdk_builder::assert_sdk_builder_mirrors_capabilities();
    status_docs::assert_d1_status_docs_are_synced();
}
