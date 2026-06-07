use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use super::super::data::{FrameRect, HostWindowPresentationData, PaneData, TemplatePaneNodeData};
use super::super::frame_geometry::contains_point;
use super::super::painter::frame_from_template;
use super::super::template_component_family::{template_component_family, TemplateComponentFamily};
use super::super::template_geometry::template_nodes_bounds;
use super::super::template_popup_layout::{
    dropdown_option_popup_frame_within, dropdown_option_row_frame_within, menu_item_row_frame,
};
use super::surface_frame::hit_test_host_surface_frame;

#[derive(Clone)]
pub(crate) struct TemplateNodePointerHit {
    pub(crate) control_id: SharedString,
    pub(crate) action_id: SharedString,
    pub(crate) binding_id: SharedString,
    pub(crate) dispatch_kind: SharedString,
    pub(crate) component_role: SharedString,
    pub(crate) component_family: Option<TemplateComponentFamily>,
    pub(crate) value_text: SharedString,
    pub(crate) edit_action_id: SharedString,
    pub(crate) commit_action_id: SharedString,
    pub(crate) frame: FrameRect,
}

pub(crate) fn hit_test_pane_template_node(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = pane_template_nodes(pane)?;
    let surface_frame = pane.body_surface_frame.as_ref()?;
    hit_test_template_nodes(nodes, surface_frame, body, x, y)
}

pub(crate) fn hit_test_workbench_window_template_node(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = &presentation.workbench_window_nodes;
    let bounds = template_nodes_bounds(nodes)?;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: bounds.width.max(bounds.x + bounds.width).max(1.0),
        height: bounds.height.max(bounds.y + bounds.height).max(1.0),
    };
    let surface_frame =
        template_nodes_surface_frame(nodes, UiSize::new(origin.width, origin.height));
    hit_test_template_nodes(nodes, &surface_frame, &origin, x, y)
}

pub(crate) fn build_pane_template_surface_frame(
    pane: &PaneData,
    surface_size: UiSize,
) -> Option<UiSurfaceFrame> {
    let nodes = pane_template_nodes(pane)?;
    let has_dispatchable = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .any(|node| is_dispatchable(&node));
    has_dispatchable.then(|| template_nodes_surface_frame(nodes, surface_size))
}

fn pane_template_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    match pane.kind.as_str() {
        "Hierarchy" => Some(&pane.hierarchy.nodes),
        "Inspector" => Some(&pane.inspector.nodes),
        "Console" => Some(&pane.console.nodes),
        "Assets" => Some(&pane.assets_activity.nodes),
        "AssetBrowser" => Some(&pane.asset_browser.nodes),
        "Welcome" => Some(&pane.welcome.nodes),
        "Project" | "UiComponentShowcase" => Some(&pane.project_overview.nodes),
        "RuntimeDiagnostics" => Some(&pane.runtime_diagnostics.nodes),
        "PerformanceTimeline" => Some(&pane.performance_timeline.nodes),
        "ModulePlugins" => Some(&pane.module_plugins.nodes),
        "BuildExport" => Some(&pane.build_export.nodes),
        "GeneratedBottom" => Some(&pane.generated_bottom.nodes),
        "UiAssetEditor" => Some(&pane.ui_asset.nodes),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&pane.animation.nodes),
        _ => None,
    }
}

fn hit_test_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_frame: &UiSurfaceFrame,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    match hit_test_template_popup_rows(nodes, origin, x, y) {
        Some(TemplatePopupRowHit::Hit(hit)) => return Some(hit),
        Some(TemplatePopupRowHit::Blocked) => return None,
        None => {}
    }

    let hit = hit_test_host_surface_frame(surface_frame, origin, x, y)?;
    let row = hit.node_id.0.checked_sub(2)? as usize;
    let node = nodes.row_data(row)?;
    let frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let component_family = template_component_family(&node);
    Some(TemplateNodePointerHit {
        control_id: node.control_id,
        action_id: node.action_id,
        binding_id: node.binding_id,
        dispatch_kind: node.dispatch_kind,
        component_role: node.component_role,
        component_family,
        value_text: node.value_text,
        edit_action_id: node.edit_action_id,
        commit_action_id: node.commit_action_id,
        frame,
    })
}

fn hit_test_template_popup_rows(
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    for row in (0..nodes.row_count()).rev() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !node.popup_open || node.disabled || node.control_id.is_empty() {
            continue;
        }
        if let Some(hit) = hit_test_template_menu_rows(&node, origin, x, y) {
            return Some(hit);
        }
        if let Some(hit) = hit_test_template_option_rows(&node, origin, x, y) {
            return Some(hit);
        }
    }
    None
}

enum TemplatePopupRowHit {
    Hit(TemplateNodePointerHit),
    Blocked,
}

fn hit_test_template_option_rows(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        return None;
    }
    let action_id = if node.edit_action_id.is_empty() {
        node.action_id.clone()
    } else {
        node.edit_action_id.clone()
    };
    if action_id.is_empty() {
        return None;
    }

    let local = frame_from_template(&node.frame);
    let control_frame = FrameRect {
        x: origin.x + local.x,
        y: origin.y + local.y,
        width: local.width,
        height: local.height,
    };
    let popup_frame = dropdown_option_popup_frame_within(&control_frame, row_count, origin)?;
    for row in 0..row_count {
        let option = node.structured_options.row_data(row)?;
        if option.disabled {
            continue;
        }
        let row_frame = dropdown_option_row_frame_within(&control_frame, row_count, row, origin)?;
        if contains_point(&row_frame, x, y) {
            return Some(TemplatePopupRowHit::Hit(template_popup_row_hit(
                node,
                row_frame,
                "workbench_option",
                action_id,
                option.id,
            )));
        }
    }
    if contains_point(&popup_frame, x, y) {
        return Some(TemplatePopupRowHit::Blocked);
    }
    None
}

fn hit_test_template_menu_rows(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    let row_count = node.structured_menu_items.row_count();
    if row_count == 0 {
        return None;
    }

    let local = frame_from_template(&node.frame);
    let menu_frame = FrameRect {
        x: origin.x + local.x,
        y: origin.y + local.y,
        width: local.width,
        height: local.height,
    };
    for row in 0..row_count {
        let item = node.structured_menu_items.row_data(row)?;
        if item.disabled || item.separator || item.action_id.is_empty() {
            continue;
        }
        let row_frame = menu_item_row_frame(&menu_frame, row_count, row)?;
        if contains_point(&row_frame, x, y) {
            return Some(TemplatePopupRowHit::Hit(template_popup_row_hit(
                node,
                row_frame,
                "workbench_menu_item",
                normalized_menu_row_action_id(item.action_id.as_str(), item.label.as_str()),
                item.label.clone(),
            )));
        }
    }
    if contains_point(&menu_frame, x, y) {
        return Some(TemplatePopupRowHit::Blocked);
    }
    None
}

fn template_popup_row_hit(
    node: &TemplatePaneNodeData,
    frame: FrameRect,
    dispatch_kind: &str,
    action_id: SharedString,
    value_text: SharedString,
) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        control_id: node.control_id.clone(),
        action_id,
        binding_id: String::new(),
        dispatch_kind: dispatch_kind.to_string(),
        component_role: node.component_role.clone(),
        component_family: template_component_family(node),
        value_text,
        edit_action_id: node.edit_action_id.clone(),
        commit_action_id: node.commit_action_id.clone(),
        frame,
    }
}

fn normalized_menu_row_action_id(action_id: &str, label: &str) -> SharedString {
    if action_id.starts_with("menu.item.") {
        return action_id.into();
    }
    menu_item_action_id(if label.is_empty() { action_id } else { label }).into()
}

fn menu_item_action_id(label: &str) -> String {
    format!("menu.item.{}", label_to_action_segment(label))
}

fn label_to_action_segment(label: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

fn is_dispatchable(node: &TemplatePaneNodeData) -> bool {
    let family = template_component_family(node);
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty()
            || !node.edit_action_id.is_empty()
            || !node.commit_action_id.is_empty()
            || family == Some(TemplateComponentFamily::TextInput))
}

fn template_nodes_surface_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_size: UiSize,
) -> UiSurfaceFrame {
    let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.template_nodes.hit"));
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        surface_size.width.max(1.0),
        surface_size.height.max(1.0),
    );
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("template_nodes/root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !is_dispatchable(&node) {
            continue;
        }
        let component = if node.component_role.is_empty() {
            template_component_family(&node)
                .map(TemplateComponentFamily::as_str)
                .unwrap_or_default()
                .to_string()
        } else {
            node.component_role.to_string()
        };
        let metadata = UiTemplateNodeMetadata {
            component,
            control_id: Some(node.control_id.to_string()),
            ..Default::default()
        };
        let mut tree_node = UiTreeNode::new(
            UiNodeId::new(row as u64 + 2),
            UiNodePath::new(format!("template_nodes/{}", node.node_id)),
        )
        .with_frame(UiFrame::new(
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
        ))
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: !node.disabled,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: node.pressed,
            checked: node.checked,
            dirty: false,
        })
        .with_input_policy(UiInputPolicy::Receive)
        .with_template_metadata(metadata);
        tree_node.layout_cache.clip_frame = template_node_clip_frame(&node);
        let _ = surface.tree.insert_child(UiNodeId::new(1), tree_node);
    }

    surface.rebuild();
    surface.surface_frame()
}

fn template_node_clip_frame(node: &TemplatePaneNodeData) -> Option<UiFrame> {
    node.has_clip_frame.then(|| {
        UiFrame::new(
            node.clip_frame.x,
            node.clip_frame.y,
            node.clip_frame.width,
            node.clip_frame.height,
        )
    })
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use crate::ui::retained_host::host_contract::data::{
        HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneMenuItemData,
        TemplatePaneNodeData, TemplatePaneOptionData,
    };
    use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};
    use crate::ui::retained_host::to_host_contract_workbench_window_nodes;
    use zircon_runtime_interface::ui::binding::UiEventKind;
    use zircon_runtime_interface::ui::layout::UiSize;

    use super::hit_test_workbench_window_template_node;

    #[test]
    fn workbench_hit_test_routes_open_dropdown_option_rows() {
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: model(vec![TemplatePaneNodeData {
                node_id: "dropdown".into(),
                control_id: "WorkbenchInputDropdown".into(),
                role: "Dropdown".into(),
                component_role: "dropdown".into(),
                edit_action_id: "component_lab.input_dropdown.select".into(),
                popup_open: true,
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 20.0,
                    width: 120.0,
                    height: 32.0,
                },
                structured_options: model(vec![
                    option("dropdown", false),
                    option("option_a", false),
                    option("option_b", true),
                ]),
                ..TemplatePaneNodeData::default()
            }]),
            ..HostWindowPresentationData::default()
        };

        let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 96.0)
            .expect("open dropdown option row should be hit-tested");

        assert_eq!(hit.control_id.as_str(), "WorkbenchInputDropdown");
        assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
        assert_eq!(
            hit.action_id.as_str(),
            "component_lab.input_dropdown.select"
        );
        assert_eq!(hit.value_text.as_str(), "option_a");
        assert_eq!(hit.frame.y, 88.0);
    }

    #[test]
    fn workbench_hit_test_routes_componentized_text_input_center() {
        let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
            .expect("componentized workbench template should project");
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
                bridge.host_projection(),
            )),
            ..HostWindowPresentationData::default()
        };
        let input = workbench_node(&presentation, "WorkbenchInputText");
        let hit = hit_test_workbench_window_template_node(
            &presentation,
            input.frame.x + input.frame.width * 0.5,
            input.frame.y + input.frame.height * 0.5,
        )
        .expect("input center should hit a componentized workbench node");

        assert_eq!(
            hit.control_id.as_str(),
            "WorkbenchInputText",
            "input center routed to {} with kind {} and role {}",
            hit.control_id,
            hit.dispatch_kind,
            hit.component_role
        );
        assert_eq!(hit.edit_action_id.as_str(), "component_lab.input_text.edit");
    }

    #[test]
    fn text_field_family_without_legacy_input_role_is_hit_tested() {
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: model(vec![TemplatePaneNodeData {
                node_id: "text".into(),
                control_id: "GenericTextField".into(),
                role: "TextField".into(),
                component_category: "input".into(),
                component_role: "text-field".into(),
                component_layout_role: "leaf".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 12.0,
                    width: 120.0,
                    height: 28.0,
                },
                ..TemplatePaneNodeData::default()
            }]),
            ..HostWindowPresentationData::default()
        };

        let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 20.0)
            .expect("TextInput component family should enter the template hit surface");

        assert_eq!(hit.control_id.as_str(), "GenericTextField");
        assert_eq!(
            hit.component_family,
            Some(TemplateComponentFamily::TextInput)
        );
    }

    #[test]
    fn workbench_hit_test_ignores_decorative_viewport_scene_layers() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
                .expect("componentized workbench template should project");
        bridge
            .dispatch_control_state("WorkbenchModuleScene", UiEventKind::Click)
            .expect("scene module state dispatch should succeed")
            .expect("scene module should expose a preview binding");
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
                bridge.host_projection(),
            )),
            ..HostWindowPresentationData::default()
        };
        let scene_layer = workbench_node(&presentation, "WorkbenchViewportFloorGrateRight");
        let x = scene_layer.frame.x + scene_layer.frame.width * 0.5;
        let y = scene_layer.frame.y + scene_layer.frame.height * 0.5;

        let hit = hit_test_workbench_window_template_node(&presentation, x, y);

        assert!(
            hit.is_none(),
            "decorative viewport scene layer should not capture pointer hit, routed to {:?}",
            hit.as_ref().map(|hit| hit.control_id.to_string())
        );
    }

    #[test]
    fn workbench_hit_test_routes_dropdown_option_rows_above_control_when_bottom_clipped() {
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: model(vec![
                TemplatePaneNodeData {
                    node_id: "root".into(),
                    control_id: "WorkbenchRoot".into(),
                    role: "Panel".into(),
                    frame: TemplateNodeFrameData {
                        x: 0.0,
                        y: 0.0,
                        width: 160.0,
                        height: 160.0,
                    },
                    ..TemplatePaneNodeData::default()
                },
                TemplatePaneNodeData {
                    node_id: "dropdown".into(),
                    control_id: "WorkbenchInputDropdown".into(),
                    role: "Dropdown".into(),
                    component_role: "dropdown".into(),
                    edit_action_id: "component_lab.input_dropdown.select".into(),
                    popup_open: true,
                    frame: TemplateNodeFrameData {
                        x: 20.0,
                        y: 120.0,
                        width: 100.0,
                        height: 28.0,
                    },
                    structured_options: model(vec![
                        option("dropdown", false),
                        option("option_a", false),
                        option("option_b", false),
                    ]),
                    ..TemplatePaneNodeData::default()
                },
            ]),
            ..HostWindowPresentationData::default()
        };

        let hit = hit_test_workbench_window_template_node(&presentation, 28.0, 74.0)
            .expect("clipped dropdown option row should be hit-tested above the control");

        assert_eq!(hit.control_id.as_str(), "WorkbenchInputDropdown");
        assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
        assert_eq!(hit.value_text.as_str(), "option_a");
        assert_eq!(hit.frame.y, 60.0);
    }

    #[test]
    fn workbench_hit_test_routes_open_popup_menu_rows() {
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: model(vec![TemplatePaneNodeData {
                node_id: "popup".into(),
                control_id: "WorkbenchPopupMenu".into(),
                role: "Menu".into(),
                component_role: "menu".into(),
                popup_open: true,
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 20.0,
                    width: 140.0,
                    height: 120.0,
                },
                structured_menu_items: model(vec![
                    menu_item("New", false, false),
                    menu_item("Open", false, false),
                    menu_item("Save", false, false),
                    menu_item("", true, true),
                    menu_item("Delete", false, false),
                ]),
                ..TemplatePaneNodeData::default()
            }]),
            ..HostWindowPresentationData::default()
        };

        let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 128.0)
            .expect("open popup menu item row should be hit-tested");

        assert_eq!(hit.control_id.as_str(), "WorkbenchPopupMenu");
        assert_eq!(hit.dispatch_kind.as_str(), "workbench_menu_item");
        assert_eq!(hit.action_id.as_str(), "menu.item.delete");
        assert_eq!(hit.value_text.as_str(), "Delete");
        assert_eq!(hit.frame.y, 116.0);
    }

    #[test]
    fn workbench_hit_test_blocks_popup_menu_separator_row() {
        let presentation = HostWindowPresentationData {
            workbench_window_nodes: model(vec![TemplatePaneNodeData {
                node_id: "popup".into(),
                control_id: "WorkbenchPopupMenu".into(),
                role: "Menu".into(),
                component_role: "menu".into(),
                action_id: "workbench.component.menu.open".into(),
                popup_open: true,
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 20.0,
                    width: 140.0,
                    height: 120.0,
                },
                structured_menu_items: model(vec![
                    menu_item("New", false, false),
                    menu_item("Open", false, false),
                    menu_item("Save", false, false),
                    menu_item("", true, true),
                    menu_item("Delete", false, false),
                ]),
                ..TemplatePaneNodeData::default()
            }]),
            ..HostWindowPresentationData::default()
        };

        assert!(
            hit_test_workbench_window_template_node(&presentation, 24.0, 104.0).is_none(),
            "separator rows should block parent/underlay hit fallback while staying inside the popup"
        );
    }

    fn option(id: &str, disabled: bool) -> TemplatePaneOptionData {
        TemplatePaneOptionData {
            id: id.into(),
            label: id.into(),
            disabled,
            ..TemplatePaneOptionData::default()
        }
    }

    fn menu_item(action_id: &str, disabled: bool, separator: bool) -> TemplatePaneMenuItemData {
        TemplatePaneMenuItemData {
            action_id: action_id.into(),
            label: action_id.into(),
            disabled,
            separator,
            ..TemplatePaneMenuItemData::default()
        }
    }

    fn workbench_node(
        presentation: &HostWindowPresentationData,
        control_id: &str,
    ) -> TemplatePaneNodeData {
        (0..presentation.workbench_window_nodes.row_count())
            .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
    }

    fn model<T: Clone>(values: Vec<T>) -> ModelRc<T> {
        Rc::new(VecModel::from(values)).into()
    }
}
