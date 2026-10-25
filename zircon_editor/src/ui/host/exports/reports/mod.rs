#[path = "editor_plugin_enable_report.rs"]
mod editor_plugin_enable_report;
#[path = "../../../reports/editor_plugin_feature_selection_update_report.rs"]
mod editor_plugin_feature_selection_update_report;
#[path = "../../reports/editor_plugin_selection_update_report.rs"]
mod editor_plugin_selection_update_report;
#[path = "editor_plugin_status.rs"]
mod editor_plugin_status;
#[path = "editor_plugin_status_report.rs"]
mod editor_plugin_status_report;

pub use self::editor_plugin_enable_report::{
    EditorPluginEnableReport, PluginEnablementApplied, PluginEnablementCause,
    PluginEnablementCauseKind, PluginEnablementDurable, PluginEnablementOperationReport,
    PluginEnablementPhase, PluginEnablementRecovery,
};
pub use self::editor_plugin_feature_selection_update_report::EditorPluginFeatureSelectionUpdateReport;
pub use self::editor_plugin_selection_update_report::EditorPluginSelectionUpdateReport;
pub use self::editor_plugin_status::{
    EditorPluginFeatureDependencyStatus, EditorPluginFeatureStatus, EditorPluginPackageMetadata,
    EditorPluginStatus,
};
pub use self::editor_plugin_status_report::EditorPluginStatusReport;

pub use editor_plugin_enable_report::{PluginEnablementAdmission, PluginEnablementRecoveryCommand};
