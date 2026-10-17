//! Only the six selected-module menus may leave their pane's inherited clip.
//! Paint and native hit testing share this exact source, frame and foreground clip proof.

use super::super::data::{FrameRect, HostWindowPresentationData, TemplatePaneNodeData};
use super::panes::authored_panes;
use super::{intersect_rect, visible_rect};
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, surface::UiSurfaceFrame, tree::UiVisibility,
};

pub(in crate::ui::retained_host) struct ModuleToolsMenuSpec {
    pub menu_control_id: &'static str,
    pub trigger_control_id: &'static str,
    pub root_control_id: &'static str,
    pub trigger_source_node_id: &'static str,
    pub popup_source_node_id: &'static str,
    pub source_suffix: &'static str,
}

impl ModuleToolsMenuSpec {
    /// Keep the primitive's original source; the caller belongs to the selected root's exact ancestry.
    pub(in crate::ui::retained_host) fn trigger_source_is_owned(
        &self,
        root_source_path: &str,
        root_instance_path: Option<
            &[zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep],
        >,
        trigger_source_path: &str,
        trigger_source_node_id: &str,
        trigger_instance_path: Option<
            &[zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep],
        >,
    ) -> bool {
        let (Some(root_instance_path), Some(trigger_instance_path)) =
            (root_instance_path, trigger_instance_path)
        else {
            return false;
        };
        let Some((caller, prefix)) = trigger_instance_path.split_last() else {
            return false;
        };
        let root_path = root_source_path.replace('\\', "/");
        let Some(editor_source_prefix) = root_path.strip_suffix(self.source_suffix) else {
            return false;
        };
        let primitive_source = format!(
            "{editor_source_prefix}/components/workbench/primitives/inputs/workbench_button.zui"
        );
        prefix == root_instance_path
            && caller.source_path == root_source_path
            && caller.source_node_id == self.trigger_source_node_id
            && trigger_source_node_id == "root"
            && trigger_source_path.replace('\\', "/") == primitive_source
    }
}

const MENUS: &[ModuleToolsMenuSpec] = &[
    ModuleToolsMenuSpec {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        trigger_control_id: "WorkbenchAssetsWorldTools",
        root_control_id: "WorkbenchModuleAssetsWorkspace",
        trigger_source_node_id: "assets_world_tools",
        popup_source_node_id: "assets_world_tools_menu",
        source_suffix: "/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
    },
    ModuleToolsMenuSpec {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        trigger_control_id: "WorkbenchAssetsGameplayTools",
        root_control_id: "WorkbenchModuleAssetsWorkspace",
        trigger_source_node_id: "assets_gameplay_tools",
        popup_source_node_id: "assets_gameplay_tools_menu",
        source_suffix: "/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
    },
    ModuleToolsMenuSpec {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        trigger_control_id: "WorkbenchAssetsProductionTools",
        root_control_id: "WorkbenchModuleAssetsWorkspace",
        trigger_source_node_id: "assets_production_tools",
        popup_source_node_id: "assets_production_tools_menu",
        source_suffix: "/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
    },
    ModuleToolsMenuSpec {
        menu_control_id: "WorkbenchAbilityAnimationToolsMenu",
        trigger_control_id: "WorkbenchAbilityAnimationTools",
        root_control_id: "WorkbenchModuleAbilityWorkspace",
        trigger_source_node_id: "ability_animation_tools_button",
        popup_source_node_id: "ability_animation_tools_menu",
        source_suffix:
            "/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui",
    },
    ModuleToolsMenuSpec {
        menu_control_id: "WorkbenchRenderToolsMenu",
        trigger_control_id: "WorkbenchRenderTools",
        root_control_id: "WorkbenchModuleRenderWorkspace",
        trigger_source_node_id: "render_tools_button",
        popup_source_node_id: "render_tools_menu",
        source_suffix:
            "/components/workbench/modules/core/rendering/workbench_render_workspace.zui",
    },
    ModuleToolsMenuSpec {
        menu_control_id: "WorkbenchHudToolsMenu",
        trigger_control_id: "WorkbenchHudTools",
        root_control_id: "WorkbenchModuleHudWorkspace",
        trigger_source_node_id: "hud_tools_button",
        popup_source_node_id: "hud_tools_menu",
        source_suffix: "/components/workbench/modules/core/ui/workbench_hud_workspace.zui",
    },
];

pub(in crate::ui::retained_host) fn module_tools_menu_spec(
    control_id: &str,
) -> Option<&'static ModuleToolsMenuSpec> {
    MENUS.iter().find(|menu| menu.menu_control_id == control_id)
}

pub(in crate::ui::retained_host) fn module_tools_menu_spec_for_trigger(
    control_id: &str,
) -> Option<&'static ModuleToolsMenuSpec> {
    MENUS
        .iter()
        .find(|menu| menu.trigger_control_id == control_id)
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host) struct ModuleToolsPopup {
    pub row: usize,
    pub frame: FrameRect,
    pub clip: FrameRect,
    pub paint_order: u64,
}

impl ModuleToolsPopup {
    pub(in crate::ui::retained_host::host_contract) fn visible_frame(&self) -> Option<FrameRect> {
        intersect_rect(&self.frame, &self.clip)
    }
}

/// Availability belongs to the same immutable source and its complete actual parent chain.
pub(in crate::ui::retained_host) fn module_tools_source_chain_is_available(
    source: &UiSurfaceFrame,
    node_id: UiNodeId,
    window_id: UiNodeId,
    allow_collapsed_node: bool,
) -> bool {
    let mut next = Some(node_id);
    for _ in 0..source.arranged_tree.nodes.len() {
        let Some(id) = next else {
            return false;
        };
        let Some(actual) = source.arranged_tree.get(id) else {
            return false;
        };
        let closed_popup =
            id == node_id && allow_collapsed_node && actual.visibility == UiVisibility::Collapsed;
        if !actual.enabled || (!actual.is_render_visible() && !closed_popup) {
            return false;
        }
        if id == window_id {
            return actual.control_id.as_deref() == Some("WorkbenchWindowRoot");
        }
        next = actual.parent;
    }
    false
}

/// A closed popup still needs its actual selected source and an enabled, visible anchor.
pub(in crate::ui::retained_host) fn module_tools_menu_admission_is_live(
    presentation: &HostWindowPresentationData,
    menu_control_id: &str,
) -> bool {
    trigger_source(presentation, menu_control_id).is_some()
}

pub(in crate::ui::retained_host) fn module_tools_popups(
    presentation: &HostWindowPresentationData,
    bounds: &FrameRect,
) -> Vec<ModuleToolsPopup> {
    let nodes = &presentation.workbench_window_nodes;
    let Some(window) = unique_control(presentation, "WorkbenchWindowRoot") else {
        return Vec::new();
    };
    let Some(mut clip) = intersect_rect(&node_frame(window), bounds) else {
        return Vec::new();
    };
    if window.has_clip_frame {
        let Some(window_clip) = intersect_rect(
            &clip,
            &FrameRect {
                x: window.clip_frame.x,
                y: window.clip_frame.y,
                width: window.clip_frame.width,
                height: window.clip_frame.height,
            },
        ) else {
            return Vec::new();
        };
        clip = window_clip;
    }
    nodes
        .iter()
        .enumerate()
        .filter_map(|(row, node)| {
            module_tools_menu_spec(node.control_id.as_str())?;
            if !node.popup_open || node.disabled {
                return None;
            }
            let (_, popup) = menu_source(presentation, node.control_id.as_str())?;
            let source = popup.source_surface_frame.as_ref()?;
            let arranged = source.arranged_tree.get(popup.surface_node_id?)?;
            if !arranged.enabled
                || !arranged.is_render_visible()
                || !matches!(
                    popup.component_role.as_str(),
                    "menu" | "popup-menu" | "context-menu"
                )
                || !finite_frame(&node_frame(popup))
            {
                return None;
            }
            let frame = node_frame(popup);
            intersect_rect(&frame, &clip)?;
            Some(ModuleToolsPopup {
                row,
                frame,
                clip: clip.clone(),
                paint_order: arranged.paint_order,
            })
        })
        .collect()
}

fn menu_source<'a>(
    presentation: &'a HostWindowPresentationData,
    control_id: &str,
) -> Option<(&'a TemplatePaneNodeData, &'a TemplatePaneNodeData)> {
    let spec = module_tools_menu_spec(control_id)?;
    let (root, trigger) = trigger_source(presentation, control_id)?;
    let popup = unique_control(presentation, spec.menu_control_id)?;
    let source = root.source_surface_frame.as_ref()?;
    let window = unique_control(presentation, "WorkbenchWindowRoot")?;
    // A layout-only root may have no paint anchor; its arranged membership is
    // still proved by this same live publication. Any supplied paint stamp must agree.
    if window
        .source_surface_frame
        .as_ref()
        .is_some_and(|window| !Arc::ptr_eq(window, source))
    {
        return None;
    }
    let window_node = source.arranged_tree.get(window.surface_node_id?)?;
    if window_node.control_id.as_deref() != Some("WorkbenchWindowRoot")
        || window_node.node_path.0 != window.node_id.as_str()
        || !window_node.enabled
        || !window_node.is_render_visible()
    {
        return None;
    }
    let popup_id = popup.surface_node_id?;
    if !module_tools_source_chain_is_available(source, popup_id, window.surface_node_id?, false) {
        return None;
    }
    let popup_node = source.arranged_tree.get(popup_id)?;
    (source_member(root, popup, spec.popup_source_node_id, spec.source_suffix)
        && popup.parent_node_id == root.node_id
        && popup_node.parent == root.surface_node_id)
        .then_some((trigger, popup))
}

fn trigger_source<'a>(
    presentation: &'a HostWindowPresentationData,
    control_id: &str,
) -> Option<(&'a TemplatePaneNodeData, &'a TemplatePaneNodeData)> {
    let spec = module_tools_menu_spec(control_id)?;
    let root = authored_panes(presentation)
        .into_iter()
        .filter_map(|pane| pane.module_workspace)
        .find(|root| root.control_id.as_str() == spec.root_control_id)?;
    let trigger = unique_control(presentation, spec.trigger_control_id)?;
    let source = root.source_surface_frame.as_ref()?;
    let root_id = root.surface_node_id?;
    let trigger_id = trigger.surface_node_id?;
    let window = unique_control(presentation, "WorkbenchWindowRoot")?;
    if !module_tools_source_chain_is_available(source, trigger_id, window.surface_node_id?, false)
        || !module_tools_source_chain_is_available(source, root_id, window.surface_node_id?, false)
    {
        return None;
    }
    let anchor = source.arranged_tree.get(trigger_id)?;
    if !trigger_source_member(root, trigger, spec)
        || !anchor.enabled
        || !anchor.is_render_visible()
        || trigger.disabled
        || !finite_frame(&node_frame(trigger))
    {
        return None;
    }
    let anchor_clip = FrameRect {
        x: anchor.clip_frame.x,
        y: anchor.clip_frame.y,
        width: anchor.clip_frame.width,
        height: anchor.clip_frame.height,
    };
    // Arranged frames are logical; projected host frames may be DPI-scaled.
    // Validate anchor visibility in its own published coordinate space.
    intersect_rect(
        &FrameRect {
            x: anchor.frame.x,
            y: anchor.frame.y,
            width: anchor.frame.width,
            height: anchor.frame.height,
        },
        &anchor_clip,
    )?;
    let mut parent = Some(trigger_id);
    for _ in 0..source.arranged_tree.nodes.len() {
        let id = parent?;
        if id == root_id {
            return Some((root, trigger));
        }
        let ancestor = source.arranged_tree.get(id)?;
        if !ancestor.enabled || !ancestor.is_render_visible() {
            return None;
        }
        parent = ancestor.parent;
    }
    None
}

fn source_member(
    root: &TemplatePaneNodeData,
    node: &TemplatePaneNodeData,
    source_node_id: &str,
    source_suffix: &str,
) -> bool {
    node.source_node_id.as_str() == source_node_id
        && node
            .source_path
            .as_str()
            .replace('\\', "/")
            .ends_with(source_suffix)
        && source_frame_member(root, node)
}

fn trigger_source_member(
    root: &TemplatePaneNodeData,
    trigger: &TemplatePaneNodeData,
    spec: &ModuleToolsMenuSpec,
) -> bool {
    type Step = zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep;
    let Ok(root_instance_path) = serde_json::from_str::<Vec<Step>>(root.instance_path.as_str())
    else {
        return false;
    };
    let Ok(trigger_instance_path) =
        serde_json::from_str::<Vec<Step>>(trigger.instance_path.as_str())
    else {
        return false;
    };
    spec.trigger_source_is_owned(
        root.source_path.as_str(),
        Some(&root_instance_path),
        trigger.source_path.as_str(),
        trigger.source_node_id.as_str(),
        Some(&trigger_instance_path),
    ) && source_frame_member(root, trigger)
}

fn source_frame_member(root: &TemplatePaneNodeData, node: &TemplatePaneNodeData) -> bool {
    root.source_surface_frame
        .as_ref()
        .zip(node.source_surface_frame.as_ref())
        .is_some_and(|(root, node)| Arc::ptr_eq(root, node))
        && node
            .surface_node_id
            .and_then(|id| root.source_surface_frame.as_ref()?.arranged_tree.get(id))
            .is_some_and(|actual| {
                actual.node_path.0 == node.node_id.as_str()
                    && actual.control_id.as_deref() == Some(node.control_id.as_str())
            })
}

fn unique_control<'a>(
    presentation: &'a HostWindowPresentationData,
    control_id: &str,
) -> Option<&'a TemplatePaneNodeData> {
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .filter(|node| node.control_id.as_str() == control_id);
    let node = nodes.next()?;
    nodes.next().is_none().then_some(node)
}

fn finite_frame(frame: &FrameRect) -> bool {
    visible_rect(frame)
        && [frame.x, frame.y, frame.width, frame.height]
            .into_iter()
            .all(f32::is_finite)
}

fn node_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

#[cfg(test)]
#[path = "../retained/contract/module_tools_popups/test_support.rs"]
mod test_support;
#[cfg(test)]
pub(in crate::ui::retained_host) use test_support::{
    assert_control_source_paint_and_hit, assert_module_tools_menu_paint_once,
    assert_retired_source_has_no_paint,
};
