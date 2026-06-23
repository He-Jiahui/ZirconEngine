pub const PLUGIN_ID: &str = "editor_build_export_desktop";
pub const CAPABILITY: &str = "editor.extension.build_export_desktop";
pub const DIAGNOSTICS_CAPABILITY: &str = "editor.extension.build_export_desktop.diagnostics";
pub const NATIVE_DYNAMIC_REPORT_CAPABILITY: &str =
    "editor.extension.build_export_desktop.native_dynamic_report";

pub const EDITOR_CAPABILITIES: &[&str] = &[
    CAPABILITY,
    DIAGNOSTICS_CAPABILITY,
    NATIVE_DYNAMIC_REPORT_CAPABILITY,
];
