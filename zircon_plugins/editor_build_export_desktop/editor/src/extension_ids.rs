pub const EXPORT_VIEW_ID: &str = "editor.build_export_desktop";
pub const EXPORT_DRAWER_ID: &str = "editor_build_export_desktop.drawer";
pub const EXPORT_TEMPLATE_ID: &str = zircon_editor::EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID;
pub const SOURCE_TEMPLATE_REPORT_ID: &str = "editor_build_export_desktop.source_template_report";
pub const LIBRARY_EMBED_REPORT_ID: &str = "editor_build_export_desktop.library_embed_report";
pub const NATIVE_DYNAMIC_REPORT_ID: &str = "editor_build_export_desktop.native_dynamic_report";

pub const EXPORT_PANEL_TEMPLATE_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/panel.zui";
pub const SOURCE_TEMPLATE_REPORT_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/source_template_report.zui";
pub const LIBRARY_EMBED_REPORT_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/library_embed_report.zui";
pub const NATIVE_DYNAMIC_REPORT_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/native_dynamic_report.zui";
pub const EXPORT_PROFILE_TEMPLATE_DOCUMENT: &str =
    "asset://editor_build_export_desktop/templates/desktop_export_profile.toml";
pub const EXPORT_PROFILE_DRAWER_DOCUMENT: &str =
    "asset://editor_build_export_desktop/editor/export_profile_drawer.zui";

pub const EXPORT_OPERATION_GENERATE_PLAN: &str = "build_export.desktop.generate_plan";
pub const EXPORT_OPERATION_SOURCE_TEMPLATE: &str = "build_export.desktop.source_template";
pub const EXPORT_OPERATION_LIBRARY_EMBED: &str = "build_export.desktop.library_embed";
pub const EXPORT_OPERATION_NATIVE_DYNAMIC: &str = "build_export.desktop.native_dynamic";
pub const EXPORT_OPERATION_OPEN_DIAGNOSTICS: &str = "build_export.desktop.open_diagnostics";
pub const EXPORT_OPERATION_CREATE_PROFILE: &str = "build_export.desktop.create_profile";
pub const EXPORT_OPERATION_OPEN_PROFILE: &str = "build_export.desktop.open_profile";

pub const EXPORT_UI_TEMPLATE_DOCUMENTS: &[(&str, &str)] = &[
    (EXPORT_TEMPLATE_ID, EXPORT_PANEL_TEMPLATE_DOCUMENT),
    (SOURCE_TEMPLATE_REPORT_ID, SOURCE_TEMPLATE_REPORT_DOCUMENT),
    (LIBRARY_EMBED_REPORT_ID, LIBRARY_EMBED_REPORT_DOCUMENT),
    (NATIVE_DYNAMIC_REPORT_ID, NATIVE_DYNAMIC_REPORT_DOCUMENT),
];
pub const EXPORT_REPORT_TEMPLATE_DOCUMENTS: &[(&str, &str)] = &[
    (SOURCE_TEMPLATE_REPORT_ID, SOURCE_TEMPLATE_REPORT_DOCUMENT),
    (LIBRARY_EMBED_REPORT_ID, LIBRARY_EMBED_REPORT_DOCUMENT),
    (NATIVE_DYNAMIC_REPORT_ID, NATIVE_DYNAMIC_REPORT_DOCUMENT),
];

pub const EXPORT_PROFILE_COMPONENT: &str = "editor.build_export_desktop.ExportProfile";
pub const EXPORT_PROFILE_ASSET_KIND: &str = "build.export_profile";
