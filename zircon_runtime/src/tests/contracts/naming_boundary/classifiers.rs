#[path = "classifiers/server_context.rs"]
mod server_context;

pub(super) fn allowed_server_context(relative_path: &str, line: &str) -> bool {
    server_context::allowed_server_context(relative_path, line)
}

pub(super) fn classify_server_reference(_relative_path: &str, _line: &str) -> Option<&'static str> {
    None
}

pub(super) fn classify_editor_reference(relative_path: &str) -> Option<&'static str> {
    if is_test_path(relative_path) {
        return Some("test-fixture");
    }
    if relative_path.starts_with("src/plugin/")
        || relative_path.starts_with("src/builtin/")
        || relative_path == "src/core/runtime/lifecycle.rs"
    {
        return Some("runtime-profile-editor-host-target");
    }
    if relative_path.starts_with("src/dynamic_api/") {
        return Some("dynamic-api-editor-host-mode");
    }
    if relative_path.starts_with("src/ui/catalog/")
        || relative_path.starts_with("src/ui/component/state_reducer/")
        || relative_path.starts_with("src/ui/v2/surface_tree/")
    {
        return Some("runtime-ui-component-catalog-editor-controls");
    }
    if relative_path.starts_with("src/ui/") {
        return Some("runtime-ui-template-editor-profile");
    }
    if relative_path.starts_with("src/ui/surface/render/")
        || matches!(
            relative_path,
            "src/ui/surface/surface.rs"
                | "src/ui/surface/text_artifact.rs"
                | "src/ui/text/measure_cache.rs"
                | "src/ui/v2/cache.rs"
                | "src/ui/v2/compiler.rs"
                | "src/ui/v2/style.rs"
        )
    {
        return Some("runtime-ui-editor-host-contract");
    }
    if relative_path.starts_with("src/asset/") {
        return Some("runtime-asset-editor-metadata");
    }
    if relative_path.starts_with("src/core/") || relative_path.starts_with("src/core/runtime/") {
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
        "src/scene/level_system.rs" | "src/scene/world/derived_state.rs"
    ) {
        return Some("runtime-scene-editor-facing-projection");
    }
    if matches!(
        relative_path,
        "src/core/runtime/time.rs"
            | "src/core/runtime/time/product_policy.rs"
            | "src/scene/world_time/snapshot.rs"
    ) {
        return Some("runtime-time-editor-preview-domain");
    }
    if matches!(
        relative_path,
        "src/text/layout_session.rs" | "src/text/shaping/mod.rs"
    ) {
        return Some("runtime-text-editor-host-contract");
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
    if relative_path.starts_with("src/ui/") {
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
    if relative_path.starts_with("src/render_graph/") {
        return Some("legacy-runtime-render-graph-access-debt");
    }
    if relative_path.starts_with("src/text/") {
        return Some("legacy-runtime-text-compatibility-debt");
    }
    if matches!(
        relative_path,
        "src/builtin/runtime_modules/assembly.rs"
            | "src/builtin/runtime_modules/composition/identity.rs"
    ) {
        return Some("legacy-runtime-module-composition-debt");
    }
    if relative_path == "src/core/framework/window/window_command/header.rs" {
        return Some("curated-runtime-legacy-rejection-guard");
    }
    if relative_path.starts_with("src/core/runtime/") {
        return Some("curated-runtime-facade-legacy-reference");
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
