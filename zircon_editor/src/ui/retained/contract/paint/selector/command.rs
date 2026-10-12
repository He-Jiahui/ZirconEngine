//! 编译与资源导入动作共享视觉角色分类，按钮和图标按钮据同一身份选择特例样式；这里只识别外观，不派发命令。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WorkbenchCommandVisualRole {
    None,
    MutedProminent,
    PrimaryImport,
}

pub(super) fn workbench_command_visual_role(
    node: &TemplatePaneNodeData,
) -> WorkbenchCommandVisualRole {
    if is_asset_import_command(node) {
        WorkbenchCommandVisualRole::PrimaryImport
    } else if is_muted_prominent_command(node) {
        WorkbenchCommandVisualRole::MutedProminent
    } else {
        WorkbenchCommandVisualRole::None
    }
}

fn is_asset_import_command(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "ImportModel" | "WorkbenchAssetsImportButton"
    ) || matches!(
        node.action_id.as_str(),
        "workbench.asset.import_model"
            | "workbench.module.assets.import.invoke"
            | "workbench.module.assets.import_now"
    )
}

fn is_muted_prominent_command(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "WorkbenchModuleCompile" | "WorkbenchToolbarCompile"
    ) || matches!(
        node.action_id.as_str(),
        "workbench.module.compile" | "workbench.toolbar.compile"
    )
}
