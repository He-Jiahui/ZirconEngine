use super::super::data::HostWindowPresentationData;
use super::{chrome, tabs, template, viewport_toolbar};

pub(in crate::ui::retained_host::host_contract) fn route_contains_profile_frame(
    presentation: &HostWindowPresentationData,
    kind: &str,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    if let Some(hit) = authored_control_route_hit(presentation, kind, id, surface, x, y) {
        return hit;
    }
    let scene = &presentation.host_scene_data;
    match kind {
        "resize_splitter" => chrome::resize_splitter_route_hit(scene, surface, x, y),
        "document_tab" => tabs::document_tab_route_hit(scene, id, x, y),
        "drawer_tab" => tabs::drawer_tab_route_hit(scene, id, surface, x, y),
        "floating_tab" => tabs::floating_tab_route_hit(scene, id, surface, x, y),
        "host_page_tab" => tabs::host_page_tab_route_hit(scene, id, x, y),
        "activity_rail_button" => chrome::activity_rail_route_hit(scene, id, surface, x, y),
        "viewport_toolbar_control" => viewport_toolbar::viewport_toolbar_route_hit(scene, id, x, y),
        "template_control" => template::template_route_hit(scene, id, x, y),
        _ => false,
    }
}

fn authored_control_route_hit(
    presentation: &HostWindowPresentationData,
    kind: &str,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> Option<bool> {
    if !super::super::componentized_workbench_regions::owns_ordinary_panes(presentation) {
        return None;
    }
    let prefix = match kind {
        "template_control" => "template",
        "viewport_toolbar_control" => "viewport_toolbar_control",
        _ => return None,
    };
    let control_id = super::identity::profile_control_id(id, prefix, surface)?;
    let nodes = &presentation.workbench_window_nodes;
    let node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)?;
    let scene = &presentation.host_scene_data;
    let owner_matches = super::super::componentized_workbench_regions::authored_panes(presentation)
        .into_iter()
        .any(|pane| {
            if !node_descends_from(presentation, node, pane.root.node_id.as_str()) {
                return false;
            }
            if pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel" {
                let Some(document) = scene.document_surfaces().iter().find(|document| {
                    super::super::componentized_workbench_regions::authored_level_viewport(
                        presentation,
                        document,
                    )
                    .is_some_and(|viewport| viewport.root.node_id == pane.root.node_id)
                }) else {
                    return false;
                };
                let expected_surface = if scene.document_leaves.is_empty() {
                    "document"
                } else {
                    document.surface_key.as_str()
                };
                let is_toolbar = nodes
                    .iter()
                    .find(|candidate| {
                        candidate.control_id.as_str() == "UnrealViewportToolbar"
                            && node_descends_from(
                                presentation,
                                candidate,
                                pane.root.node_id.as_str(),
                            )
                    })
                    .is_some_and(|toolbar| {
                        node_descends_from(presentation, node, toolbar.node_id.as_str())
                    });
                surface == expected_surface && ((kind == "viewport_toolbar_control") == is_toolbar)
            } else {
                let expected_surface =
                    if pane.root.control_id.as_str() == "WorkbenchMainBandActivityRail" {
                        "left"
                    } else {
                        "right"
                    };
                surface == expected_surface && kind == "template_control"
            }
        });
    if !owner_matches {
        return Some(false);
    }
    use super::super::surface_hit_test::{
        hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
    };
    let cached =
        super::super::data::paint_workbench_hit_index(&presentation.workbench_window_nodes);
    let owned;
    let index = if let Some(index) = cached.as_deref() {
        index
    } else {
        owned = HostWorkbenchHitIndex::from_presentation(presentation);
        &owned
    };
    Some(
        hit_test_workbench_window_template_node_with_index(presentation, index, x, y)
            .is_some_and(|hit| hit.control_id.as_str() == control_id),
    )
}

fn node_descends_from(
    presentation: &HostWindowPresentationData,
    node: &super::super::data::TemplatePaneNodeData,
    root_id: &str,
) -> bool {
    let nodes = &presentation.workbench_window_nodes;
    let mut current = node;
    for _ in 0..nodes.row_count() {
        if current.node_id.as_str() == root_id {
            return true;
        }
        if current.parent_node_id.is_empty() {
            return false;
        }
        let Some(parent) = nodes
            .iter()
            .find(|candidate| candidate.node_id == current.parent_node_id)
        else {
            return false;
        };
        current = parent;
    }
    false
}
