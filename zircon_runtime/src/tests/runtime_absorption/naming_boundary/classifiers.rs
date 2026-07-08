pub(super) fn allowed_server_context(relative_path: &str, line: &str) -> bool {
    let lower_line = line.to_ascii_lowercase();
    if is_test_path(relative_path) {
        return true;
    }
    if relative_path.contains("/net/")
        || relative_path.contains("/network")
        || relative_path.contains("/net_features/")
    {
        return true;
    }
    if lower_line.contains("serverruntime")
        || lower_line.contains("runtimeprofileid::server")
        || lower_line.contains("target-server")
        || lower_line.contains("target_server")
        || lower_line.contains("headless_server")
        || lower_line.contains("headless server")
        || lower_line.contains("dedicatedserver")
        || lower_line.contains("listenserver")
        || lower_line.contains("server_client_targets")
    {
        return true;
    }
    if matches!(
        relative_path,
        "src/plugin/runtime_profile/descriptor.rs"
            | "src/plugin/runtime_profile/defaults.rs"
            | "src/plugin/export_build_plan/default_profile.rs"
    ) {
        return true;
    }
    if relative_path == "src/plugin/export_build_plan/platform_host_files/browser.rs"
        && (lower_line.contains("server config")
            || lower_line.contains("dev server")
            || lower_line.contains("server:"))
    {
        return true;
    }
    if relative_path == "src/ui/component/catalog/material_foundation/mui_x.rs"
        && line.contains("\"server\"")
        && line.contains("mui_enum_prop")
    {
        return true;
    }
    if matches!(
        relative_path,
        "src/ui/component/state_reducer/table.rs"
            | "src/ui/surface/surface/default_interactions/table/mod.rs"
            | "src/ui/surface/surface/default_interactions/table/columns.rs"
    ) && line.contains("Some(\"server\")")
    {
        return true;
    }
    if relative_path == "src/platform/capability/matrix/mod.rs"
        && (lower_line.contains("server/headless") || lower_line.contains("server runtime"))
    {
        return true;
    }
    false
}

pub(super) fn classify_server_reference(_relative_path: &str, _line: &str) -> Option<&'static str> {
    None
}

pub(super) fn classify_editor_reference(relative_path: &str) -> Option<&'static str> {
    if is_test_path(relative_path) {
        return Some("test-fixture");
    }
    if relative_path.starts_with("src/plugin/")
        || relative_path.starts_with("src/builtin/runtime_modules/")
    {
        return Some("runtime-profile-editor-host-target");
    }
    if relative_path.starts_with("src/dynamic_api/") {
        return Some("dynamic-api-editor-host-mode");
    }
    if relative_path.starts_with("src/ui/component/catalog/")
        || relative_path.starts_with("src/ui/component/state_reducer/")
        || relative_path.starts_with("src/ui/v2/surface_tree/")
    {
        return Some("runtime-ui-component-catalog-editor-controls");
    }
    if relative_path.starts_with("src/ui/template/") {
        return Some("runtime-ui-template-editor-profile");
    }
    if relative_path.starts_with("src/asset/") {
        return Some("runtime-asset-editor-metadata");
    }
    if relative_path.starts_with("src/core/framework/")
        || relative_path.starts_with("src/core/runtime/diagnostics/")
    {
        return Some("framework-editor-facing-descriptor");
    }
    if relative_path.starts_with("src/graphics/") {
        return Some("graphics-editor-facing-metadata");
    }
    if relative_path.starts_with("src/platform/") {
        return Some("platform-editor-target-diagnostic");
    }
    if relative_path.starts_with("src/rhi") {
        return Some("rhi-editor-surface-label");
    }
    if relative_path.starts_with("src/scene/reflect/")
        || relative_path.starts_with("src/scene/inspection/")
    {
        return Some("scene-reflection-editor-visible-metadata");
    }
    if matches!(
        relative_path,
        "src/diagnostic_log/sink.rs" | "src/prelude.rs"
    ) {
        return Some("curated-runtime-facade-editor-reference");
    }
    None
}

pub(super) fn classify_legacy_reference(relative_path: &str) -> Option<&'static str> {
    if is_test_path(relative_path) {
        return Some("test-fixture");
    }
    if relative_path.starts_with("src/ui/surface/input/")
        || relative_path == "src/ui/surface/property_mutation.rs"
        || relative_path == "src/ui/surface/surface/default_interactions.rs"
    {
        return Some("legacy-runtime-ui-input-debt");
    }
    if relative_path == "src/ui/surface/render/collection_rows/table.rs" {
        return Some("legacy-runtime-ui-render-table-debt");
    }
    if relative_path.starts_with("src/graphics/")
        || relative_path.starts_with("src/core/framework/render/")
    {
        return Some("legacy-runtime-graphics-debt");
    }
    if relative_path == "src/asset/assets/texture/upload_support/dds.rs" {
        return Some("legacy-runtime-dds-container-policy");
    }
    if relative_path.starts_with("src/ui/template/") {
        return Some("legacy-runtime-ui-template-schema-debt");
    }
    if relative_path.starts_with("src/ui/layout/") {
        return Some("legacy-runtime-ui-layout-debt");
    }
    if relative_path.starts_with("src/input/")
        || relative_path.starts_with("src/core/framework/input/")
    {
        return Some("legacy-runtime-input-event-debt");
    }
    if relative_path.starts_with("src/asset/") {
        return Some("legacy-runtime-asset-schema-debt");
    }
    if relative_path.starts_with("src/dynamic_api/") {
        return Some("legacy-dynamic-api-migration-debt");
    }
    if relative_path.starts_with("src/scene/") {
        return Some("legacy-scene-schema-render-debt");
    }
    if matches!(
        relative_path,
        "src/prelude.rs" | "src/ui/accessibility/extract.rs"
    ) {
        return Some("curated-runtime-facade-legacy-reference");
    }
    None
}

fn is_test_path(relative_path: &str) -> bool {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    relative_path.split('/').any(|part| part == "tests")
        || file_name == "tests.rs"
        || file_name.ends_with("_tests.rs")
}
