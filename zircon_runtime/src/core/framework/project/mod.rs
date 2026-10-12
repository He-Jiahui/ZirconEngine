//! 导出配置和插件选择结果是 Runtime 与 Editor 共用的项目 DTO；目录查询通过调用方提供的 provider 完成。
#[path = "export_profile.rs"]
mod export_profile;
#[path = "plugin_selection_resolution.rs"]
mod plugin_selection_resolution;
#[path = "project_plugin_manifest/mod.rs"]
mod project_plugin_manifest;
#[path = "runtime_profile_id.rs"]
mod runtime_profile_id;

pub use export_profile::{
    ExportBuildMode, ExportPackagingStrategy, ExportPlatformHostKind, ExportPlatformPluginStrategy,
    ExportPlatformPolicy, ExportPlatformResourceStrategy, ExportProfile, ExportTargetPlatform,
};
pub use plugin_selection_resolution::{
    resolve_plugin_selections, PluginSelectionResolution, PluginSelectionResolutionReport,
    PluginSelectionResolutionStatus, RequiredPluginSelectionResolutionError,
};
pub use project_plugin_manifest::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};
pub use runtime_profile_id::RuntimeProfileId;
