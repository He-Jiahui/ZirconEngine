use super::is_test_path;

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
            | "src/plugin/runtime_profile/assembly_presets.rs"
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
