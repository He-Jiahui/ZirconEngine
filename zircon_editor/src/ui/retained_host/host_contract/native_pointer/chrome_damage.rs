use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplatePaneNodeData,
};

use super::ChromePointerRoute;

pub(super) fn chrome_press_damage_frame(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let frame = match route {
        ChromePointerRoute::ActivityRail { .. } => {
            presentation.host_layout.center_band_frame.clone()
        }
        ChromePointerRoute::DocumentTab { surface_key, .. }
            if surface_key.as_str() == "document"
                || surface_key.as_str() == scene.document_dock.surface_key.as_str() =>
        {
            scene.document_dock.region_frame.clone()
        }
        ChromePointerRoute::DocumentTab { surface_key, .. } => {
            let Some(window) =
                (0..scene.floating_layer.floating_windows.row_count()).find_map(|row| {
                    let window = scene.floating_layer.floating_windows.row_data(row)?;
                    (window.window_id.as_str() == surface_key.as_str()).then_some(window)
                })
            else {
                return None;
            };
            window.frame.clone()
        }
        ChromePointerRoute::DrawerHeaderTab { surface_key, .. } => match surface_key.as_str() {
            "left" => scene.left_dock.region_frame.clone(),
            "right" => scene.right_dock.region_frame.clone(),
            "bottom" => scene.bottom_dock.region_frame.clone(),
            _ => return None,
        },
        ChromePointerRoute::HostPageTab { .. } => {
            return host_page_tab_damage_frame(presentation);
        }
        ChromePointerRoute::Resize => return None,
        ChromePointerRoute::FloatingWindowHeader { window_id } => {
            let Some(target) =
                (0..scene.floating_layer.floating_windows.row_count()).find_map(|row| {
                    let window = scene.floating_layer.floating_windows.row_data(row)?;
                    (window.window_id.as_str() == window_id.as_str()).then_some(window.frame)
                })
            else {
                return None;
            };
            let mut damage = Some(target);
            for row in 0..scene.floating_layer.floating_windows.row_count() {
                let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
                    continue;
                };
                if !visible_frame(&window.frame) {
                    continue;
                }
                damage = Some(match damage {
                    Some(current) => union_frame(&current, &window.frame),
                    None => window.frame.clone(),
                });
            }
            damage?
        }
    };
    visible_frame(&frame).then_some(frame)
}

fn host_page_tab_damage_frame(presentation: &HostWindowPresentationData) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let page_chrome = &scene.page_chrome;
    let mut damage = None;

    // Page activation can update selected tab chrome, the active workbench body,
    // and the status text; keep the menu/title bar out of this repaint.
    damage = union_visible_frame(damage, page_chrome.tab_row_frame.clone());
    damage = union_visible_frame(damage, page_chrome.project_path_frame.clone());
    for row in 0..page_chrome.tab_frames.row_count() {
        let Some(tab) = page_chrome.tab_frames.row_data(row) else {
            continue;
        };
        damage = union_visible_frame(damage, tab.frame.clone());
        damage = union_visible_frame(damage, tab.close_frame.clone());
    }
    for row in 0..page_chrome.template_nodes.row_count() {
        let Some(node) = page_chrome.template_nodes.row_data(row) else {
            continue;
        };
        damage = union_visible_frame(damage, template_node_frame(&node));
    }

    damage = union_visible_frame(damage, presentation.host_layout.center_band_frame.clone());
    damage = union_visible_frame(damage, scene.layout.center_band_frame.clone());
    damage = union_visible_frame(damage, presentation.host_layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.status_bar.status_bar_frame.clone());
    damage
}

fn template_node_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

fn union_visible_frame(current: Option<FrameRect>, frame: FrameRect) -> Option<FrameRect> {
    if !visible_frame(&frame) {
        return current;
    }
    Some(match current {
        Some(current) => union_frame(&current, &frame),
        None => frame,
    })
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn union_frame(left: &FrameRect, right: &FrameRect) -> FrameRect {
    let x0 = left.x.min(right.x);
    let y0 = left.y.min(right.y);
    let x1 = (left.x + left.width).max(right.x + right.width);
    let y1 = (left.y + left.height).max(right.y + right.height);
    FrameRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    }
}
