mod target_rows;

#[cfg(test)]
mod tests;

use self::target_rows::build_export_target_row_nodes;
use super::build_export_wizard_panel::{
    build_export_pane_supports_wizard_projection, build_export_wizard_panel_nodes,
};
use super::model_projection::map_model_rc;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportTargetViewData, PaneContentSize, PaneData,
};
use crate::ui::retained_host as host_contract;

pub(crate) fn to_host_contract_build_export_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::BuildExportPaneData {
    let native = &data.native_body.build_export;
    let nodes = if build_export_pane_supports_wizard_projection(data) {
        build_export_wizard_panel_nodes(native, content_size).unwrap_or_else(|| {
            let mut nodes =
                build_export_template_projection(data, content_size).unwrap_or_default();
            nodes.extend(build_export_target_row_nodes(native, &nodes, content_size));
            nodes
        })
    } else {
        let mut nodes = build_export_template_projection(data, content_size).unwrap_or_default();
        nodes.extend(build_export_target_row_nodes(native, &nodes, content_size));
        nodes
    };

    host_contract::BuildExportPaneData {
        nodes: model_rc(nodes),
        targets: map_model_rc(&native.targets, to_host_contract_build_export_target),
        diagnostics: native.diagnostics.clone(),
    }
}

fn to_host_contract_build_export_target(
    data: BuildExportTargetViewData,
) -> host_contract::BuildExportTargetData {
    host_contract::BuildExportTargetData {
        profile_name: data.profile_name,
        platform: data.platform,
        target_mode: data.target_mode,
        strategies: data.strategies,
        status: data.status,
        enabled_plugins: data.enabled_plugins,
        linked_runtime_crates: data.linked_runtime_crates,
        native_dynamic_packages: data.native_dynamic_packages,
        generated_files: data.generated_files,
        diagnostics: data.diagnostics,
        fatal: data.fatal,
    }
}

fn build_export_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::BuildExportV1(_)
    ) {
        return None;
    }

    super::project_pane_template_nodes(&presentation.body, content_size)
}
