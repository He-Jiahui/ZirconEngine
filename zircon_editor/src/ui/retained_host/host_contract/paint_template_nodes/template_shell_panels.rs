use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_chrome_style, WorkbenchChromeKind as ShellPanelKind};

mod separators;

pub(super) fn push_shell_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = shell_panel_kind(node) else {
        return false;
    };
    let rect = separators::pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let style = select_workbench_chrome_style(node, kind);
    if let Some(fill) = style.fill {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(fill),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    separators::push_shell_panel_separators(
        commands,
        kind,
        &style,
        &rect,
        clip,
        order + 1,
        opacity,
    );
    true
}

fn shell_panel_kind(node: &TemplatePaneNodeData) -> Option<ShellPanelKind> {
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
        _ => None,
    }
}

#[cfg(test)]
#[path = "template_shell_panels_tests.rs"]
mod tests;
