//! 工作台容器的稳定 control_id 映射到集中 chrome 样式；内容面板后缀只在 Workbench 命名空间内认领。

use super::super::super::data::TemplatePaneNodeData;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use super::super::style_selector::WorkbenchChromeKind as ShellPanelKind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shell_panel_kind(
    node: &TemplatePaneNodeData,
) -> Option<ShellPanelKind> {
    match node.control_id.as_str() {
        "WorkbenchWindowRoot" => Some(ShellPanelKind::WindowRoot),
        "WorkbenchWindowTopToolbar" | "WorkbenchWindowTopToolbarRegion" => {
            Some(ShellPanelKind::TopToolbar)
        }
        "WorkbenchMainBand" | "WorkbenchWindowMainBandRegion" => Some(ShellPanelKind::MainBand),
        "WorkbenchWindowActivityRail" | "WorkbenchMainBandActivityRail" => {
            Some(ShellPanelKind::ActivityRail)
        }
        "WorkbenchSceneTreePanel" | "WorkbenchMainBandSceneTreePanel" => {
            Some(ShellPanelKind::ScenePanel)
        }
        "WorkbenchViewportPanel" | "WorkbenchMainBandViewportPanel" => {
            Some(ShellPanelKind::ViewportPanel)
        }
        "WorkbenchInspectorPanel" | "WorkbenchMainBandInspectorPanel" => {
            Some(ShellPanelKind::InspectorPanel)
        }
        "WorkbenchComponentDrawer" | "WorkbenchWindowComponentDrawerRegion" => {
            Some(ShellPanelKind::ComponentDrawer)
        }
        "WorkbenchComponentDrawerBody" | "WorkbenchComponentDrawerConsoleBody" => {
            Some(ShellPanelKind::DrawerBody)
        }
        "WorkbenchComponentInputs"
        | "WorkbenchComponentSelection"
        | "WorkbenchComponentFeedback"
        | "WorkbenchComponentList" => Some(ShellPanelKind::DrawerColumn),
        "WorkbenchWindowStatusBar" | "WorkbenchWindowStatusBarRegion" => {
            Some(ShellPanelKind::StatusBar)
        }
        "WorkbenchSceneTabs" | "WorkbenchInspectorTabs" | "WorkbenchComponentDrawerTabs" => {
            Some(ShellPanelKind::TabsBand)
        }
        "WorkbenchInspectorTransform" | "WorkbenchInspectorMesh" => {
            Some(ShellPanelKind::InspectorSection)
        }
        _ if is_workbench_content_panel_id(node.control_id.as_str()) => {
            Some(ShellPanelKind::ContentPanel)
        }
        _ => None,
    }
}

fn is_workbench_content_panel_id(control_id: &str) -> bool {
    control_id.starts_with("Workbench")
        && matches!(
            content_panel_suffix(control_id),
            Some("LeftPanel" | "CenterPanel" | "RightPanel")
        )
}

fn content_panel_suffix(control_id: &str) -> Option<&'static str> {
    let stem = control_id.strip_suffix("Panel")?;
    if stem.ends_with("Left") {
        Some("LeftPanel")
    } else if stem.ends_with("Center") {
        Some("CenterPanel")
    } else if stem.ends_with("Right") {
        Some("RightPanel")
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/identity_optimization_tests.rs"]
mod optimization_tests;
