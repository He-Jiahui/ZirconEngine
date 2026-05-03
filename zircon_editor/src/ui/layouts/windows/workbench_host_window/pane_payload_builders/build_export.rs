use slint::Model;

use super::super::pane_payload::{BuildExportPanePayload, BuildExportTargetPayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let Some(data) = context.build_export else {
        return PanePayload::BuildExportV1(BuildExportPanePayload {
            diagnostics: String::new(),
            targets: Vec::new(),
        });
    };

    PanePayload::BuildExportV1(BuildExportPanePayload {
        diagnostics: data.diagnostics.to_string(),
        targets: (0..data.targets.row_count())
            .filter_map(|row| data.targets.row_data(row))
            .map(|target| BuildExportTargetPayload {
                profile_name: target.profile_name.to_string(),
                platform: target.platform.to_string(),
                target_mode: target.target_mode.to_string(),
                strategies: target.strategies.to_string(),
                status: target.status.to_string(),
                enabled_plugins: target.enabled_plugins.to_string(),
                linked_runtime_crates: target.linked_runtime_crates.to_string(),
                native_dynamic_packages: target.native_dynamic_packages.to_string(),
                generated_files: target.generated_files.to_string(),
                diagnostics: target.diagnostics.to_string(),
                fatal: target.fatal,
            })
            .collect(),
    })
}
