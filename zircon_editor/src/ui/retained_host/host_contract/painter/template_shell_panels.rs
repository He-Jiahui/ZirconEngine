use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

const ROOT_BG: [u8; 4] = [8, 11, 14, 255];
const TOPBAR_BG: [u8; 4] = [14, 18, 22, 255];
const MAIN_BG: [u8; 4] = [10, 13, 16, 255];
const RAIL_BG: [u8; 4] = [12, 16, 20, 255];
const PANEL_BG: [u8; 4] = [15, 20, 24, 255];
const VIEWPORT_FRAME_BG: [u8; 4] = [9, 12, 15, 255];
const DRAWER_BG: [u8; 4] = [13, 18, 22, 255];
const DRAWER_BODY_BG: [u8; 4] = [12, 16, 20, 255];
const STATUS_BG: [u8; 4] = [12, 17, 21, 255];
const TAB_BG: [u8; 4] = [14, 19, 23, 255];
const SEPARATOR: [u8; 4] = [29, 38, 44, 255];
const STRONG_SEPARATOR: [u8; 4] = [38, 49, 56, 255];
const SOFT_SEPARATOR: [u8; 4] = [24, 31, 36, 255];
const DRAWER_COLUMN_SEPARATOR_OFFSET: f32 = -6.0;

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
    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    if let Some(fill) = shell_panel_fill(kind) {
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
    push_shell_panel_separators(commands, kind, &rect, clip, order + 1, opacity);
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShellPanelKind {
    WindowRoot,
    TopToolbar,
    MainBand,
    ActivityRail,
    ScenePanel,
    ViewportPanel,
    InspectorPanel,
    ComponentDrawer,
    DrawerBody,
    DrawerColumn,
    StatusBar,
    TabsBand,
    InspectorSection,
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

fn shell_panel_fill(kind: ShellPanelKind) -> Option<[u8; 4]> {
    match kind {
        ShellPanelKind::WindowRoot => Some(ROOT_BG),
        ShellPanelKind::TopToolbar => Some(TOPBAR_BG),
        ShellPanelKind::MainBand => Some(MAIN_BG),
        ShellPanelKind::ActivityRail => Some(RAIL_BG),
        ShellPanelKind::ScenePanel
        | ShellPanelKind::InspectorPanel
        | ShellPanelKind::InspectorSection => Some(PANEL_BG),
        ShellPanelKind::ViewportPanel => Some(VIEWPORT_FRAME_BG),
        ShellPanelKind::ComponentDrawer => Some(DRAWER_BG),
        ShellPanelKind::DrawerBody => Some(DRAWER_BODY_BG),
        ShellPanelKind::DrawerColumn => None,
        ShellPanelKind::StatusBar => Some(STATUS_BG),
        ShellPanelKind::TabsBand => Some(TAB_BG),
    }
}

fn push_shell_panel_separators(
    commands: &mut Vec<HostPaintCommand>,
    kind: ShellPanelKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    match kind {
        ShellPanelKind::TopToolbar => {
            push_bottom_line(commands, rect, clip, order, STRONG_SEPARATOR, opacity);
        }
        ShellPanelKind::ActivityRail | ShellPanelKind::ScenePanel => {
            push_right_line(commands, rect, clip, order, STRONG_SEPARATOR, opacity);
        }
        ShellPanelKind::ViewportPanel => {
            push_left_line(commands, rect, clip, order, SOFT_SEPARATOR, opacity);
            push_right_line(commands, rect, clip, order, SOFT_SEPARATOR, opacity);
        }
        ShellPanelKind::InspectorPanel => {
            push_left_line(commands, rect, clip, order, STRONG_SEPARATOR, opacity);
        }
        ShellPanelKind::ComponentDrawer | ShellPanelKind::StatusBar => {
            push_top_line(commands, rect, clip, order, STRONG_SEPARATOR, opacity);
        }
        ShellPanelKind::TabsBand | ShellPanelKind::InspectorSection => {
            push_bottom_line(commands, rect, clip, order, SEPARATOR, opacity);
        }
        ShellPanelKind::DrawerColumn => {
            push_vertical_line(
                commands,
                rect.x + DRAWER_COLUMN_SEPARATOR_OFFSET,
                rect.y,
                rect.height,
                clip,
                order,
                SOFT_SEPARATOR,
                opacity,
            );
        }
        ShellPanelKind::WindowRoot | ShellPanelKind::MainBand | ShellPanelKind::DrawerBody => {}
    }
}

fn push_top_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_horizontal_line(commands, rect, rect.y, clip, order, color, opacity);
}

fn push_bottom_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_horizontal_line(
        commands,
        rect,
        rect.y + rect.height - 1.0,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_left_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_vertical_line(
        commands,
        rect.x,
        rect.y,
        rect.height,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_right_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_vertical_line(
        commands,
        rect.x + rect.width - 1.0,
        rect.y,
        rect.height,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_horizontal_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    y: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: y.round(),
            width: rect.width,
            height: 1.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_vertical_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    height: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x.round(),
            y: y.round(),
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn workbench_shell_panels_match_only_container_ids() {
        assert_eq!(
            shell_panel_kind(&panel_node(
                "WorkbenchWindowTopToolbar",
                0.0,
                0.0,
                120.0,
                40.0
            )),
            Some(ShellPanelKind::TopToolbar)
        );
        assert_eq!(
            shell_panel_kind(&panel_node(
                "WorkbenchInspectorPanel",
                0.0,
                0.0,
                120.0,
                40.0
            )),
            Some(ShellPanelKind::InspectorPanel)
        );
        assert_eq!(
            shell_panel_kind(&panel_node(
                "WorkbenchComponentInputs",
                0.0,
                0.0,
                120.0,
                40.0
            )),
            Some(ShellPanelKind::DrawerColumn)
        );
        assert_eq!(
            shell_panel_kind(&panel_node("WorkbenchViewportMode", 0.0, 0.0, 120.0, 40.0)),
            None
        );
    }

    #[test]
    fn top_toolbar_paints_surface_and_bottom_separator() {
        let bytes = paint_template_nodes_for_test(
            150,
            64,
            model_rc(vec![panel_node(
                "WorkbenchWindowTopToolbar",
                8.0,
                6.0,
                128.0,
                40.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 150, 24, 18), TOPBAR_BG);
        assert_eq!(pixel_at(&bytes, 150, 24, 45), STRONG_SEPARATOR);
    }

    #[test]
    fn side_panels_paint_directional_separators() {
        let bytes = paint_template_nodes_for_test(
            240,
            90,
            model_rc(vec![
                panel_node("WorkbenchSceneTreePanel", 8.0, 10.0, 86.0, 56.0),
                panel_node("WorkbenchInspectorPanel", 130.0, 10.0, 92.0, 56.0),
            ]),
        );

        assert_eq!(pixel_at(&bytes, 240, 30, 32), PANEL_BG);
        assert_eq!(pixel_at(&bytes, 240, 93, 32), STRONG_SEPARATOR);
        assert_eq!(pixel_at(&bytes, 240, 130, 32), STRONG_SEPARATOR);
        assert_eq!(pixel_at(&bytes, 240, 170, 32), PANEL_BG);
    }

    #[test]
    fn drawer_and_status_bar_paint_top_separators() {
        let bytes = paint_template_nodes_for_test(
            180,
            90,
            model_rc(vec![
                panel_node("WorkbenchComponentDrawer", 10.0, 12.0, 140.0, 34.0),
                panel_node("WorkbenchWindowStatusBar", 10.0, 56.0, 140.0, 28.0),
            ]),
        );

        assert_eq!(pixel_at(&bytes, 180, 20, 12), STRONG_SEPARATOR);
        assert_eq!(pixel_at(&bytes, 180, 20, 24), DRAWER_BG);
        assert_eq!(pixel_at(&bytes, 180, 20, 56), STRONG_SEPARATOR);
        assert_eq!(pixel_at(&bytes, 180, 20, 68), STATUS_BG);
    }

    #[test]
    fn drawer_column_paints_gap_separator_without_surface_fill() {
        let bytes = paint_template_nodes_for_test(
            160,
            70,
            model_rc(vec![panel_node(
                "WorkbenchComponentInputs",
                72.0,
                10.0,
                70.0,
                42.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 160, 66, 24), SOFT_SEPARATOR);
        assert_eq!(pixel_at(&bytes, 160, 96, 24), [0, 0, 0, 255]);
    }

    fn panel_node(
        control_id: &'static str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "VerticalGroup".into(),
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }
}
