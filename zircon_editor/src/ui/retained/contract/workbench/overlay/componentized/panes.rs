use super::*;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    authored_hierarchy, authored_level_viewport, authored_panes, is_authored_tree_row,
    module_tools_menu_spec,
};

struct OrdinaryPaneSubtree {
    subtree: ExtensionWorkspaceSubtree,
    excluded_rows: HashSet<usize>,
    suppress_viewport_fallback: bool,
}
impl TemplateNodePaintTransform for OrdinaryPaneSubtree {
    fn stream_row_visit_indices(
        &self,
        row_count: usize,
        clip: &FrameRect,
        visit: &mut dyn FnMut(usize),
    ) -> bool {
        self.subtree
            .stream_row_visit_indices(row_count, clip, visit)
    }
    fn filter_row(
        &self,
        row: usize,
        node: &TemplatePaneNodeData,
        clip: &FrameRect,
    ) -> Option<FrameRect> {
        (!is_authored_tree_row(&node)
            && module_tools_menu_spec(node.control_id.as_str()).is_none()
            && !self.excluded_rows.contains(&row)
            && !(self.suppress_viewport_fallback && is_viewport_fallback_scene_node(&node)))
        .then(|| clip.clone())
    }
}

pub(super) fn draw_authored_panes(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    bounds: &FrameRect,
) {
    let index = paint_workbench_hit_index(&presentation.workbench_window_nodes);
    let focus = paint_text_input_focus(presentation);
    for pane in authored_panes(presentation) {
        let Some(clip) = intersect_rect(&pane.frame, bounds) else {
            continue;
        };
        let is_viewport = pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel";
        let viewport_document = (is_viewport && pane.module_workspace.is_none())
            .then(|| {
                presentation
                    .host_scene_data
                    .document_surfaces()
                    .iter()
                    .find(|document| {
                        authored_level_viewport(presentation, document)
                            .is_some_and(|viewport| viewport.root.node_id == pane.root.node_id)
                    })
            })
            .flatten();
        if is_viewport && viewport_document.is_none() && pane.module_workspace.is_none() {
            continue;
        }
        let viewport_toolbar = (is_viewport && pane.module_workspace.is_none())
            .then(|| authored_viewport_toolbar(presentation, pane.root.node_id.as_str()))
            .flatten();
        let excluded_rows = viewport_toolbar
            .map(|(_, toolbar)| subtree_rows(presentation, toolbar.node_id.as_str()))
            .unwrap_or_default();
        let subtree = OrdinaryPaneSubtree {
            subtree: ExtensionWorkspaceSubtree::from_presentation(
                presentation,
                pane.root.node_id.as_str(),
                Some(pane.root_row),
                index.clone(),
            ),
            excluded_rows,
            suppress_viewport_fallback: is_viewport,
        };
        draw_template_nodes_with_transform(
            frame,
            &presentation.workbench_window_nodes,
            &zero_origin(),
            &clip,
            Some(&focus),
            Some(&subtree),
        );
        if let Some(document) = viewport_document {
            draw_authored_viewport_media(frame, presentation, document, &clip);
            if let Some((toolbar_row, toolbar)) = viewport_toolbar {
                let toolbar_subtree = ExtensionWorkspaceSubtree::from_presentation(
                    presentation,
                    toolbar.node_id.as_str(),
                    Some(toolbar_row),
                    index.clone(),
                );
                draw_template_nodes_with_transform(
                    frame,
                    &presentation.workbench_window_nodes,
                    &zero_origin(),
                    &clip,
                    Some(&focus),
                    Some(&toolbar_subtree),
                );
            }
        }
    }
    let Some(hierarchy) = authored_hierarchy(presentation) else {
        return;
    };
    let interaction = paint_pane_interaction_state(presentation);
    let Some(clip) = intersect_rect(&hierarchy.viewport, bounds) else {
        return;
    };
    let pitch = hierarchy.metrics.row_height + hierarchy.metrics.row_gap;
    if pitch <= 0.0 {
        return;
    }
    let scroll = interaction.hierarchy_scroll_px.max(0.0);
    let start = ((scroll - hierarchy.metrics.row_y) / pitch)
        .floor()
        .max(0.0) as usize;
    let end = ((scroll + hierarchy.viewport.height - hierarchy.metrics.row_y) / pitch)
        .ceil()
        .max(0.0) as usize;
    let mut rows = Vec::new();
    for row_index in start..end.min(hierarchy.pane.hierarchy.hierarchy_nodes.row_count()) {
        let Some(entity) = hierarchy.pane.hierarchy.hierarchy_nodes.get(row_index) else {
            continue;
        };
        let mut row = hierarchy.prototype.clone();
        // Discard cached runtime commands: this physical row now renders a different entity.
        row.surface_render_command_ref = None;
        row.node_id = format!("{}/entity/{}", hierarchy.prototype.node_id, entity.id).into();
        row.control_id = format!("NativeHierarchyEntity:{}", entity.id).into();
        row.text = entity.name.clone();
        row.label_text = entity.name.clone();
        row.value_text = Default::default();
        row.pressed = false;
        row.focused = false;
        if focus.control_id.as_str()
            == crate::ui::retained_host::app::hierarchy_rename::HIERARCHY_INLINE_RENAME_CONTROL_ID
            && crate::ui::retained_host::app::hierarchy_rename::hierarchy_inline_rename_target_id(
                focus.dispatch_kind.as_str(),
            ) == Some(entity.id.as_str())
        {
            row.text = focus.value_text.clone();
            row.label_text = focus.value_text.clone();
        }
        row.selected = entity.selected;
        row.hovered = interaction.hovered_hierarchy_index == row_index as i32;
        row.tree_depth = entity.depth;
        // Zero selects the same shared TreeRow indentation tokens as authored rows.
        row.tree_indent_px = 0.0;
        let geometry = crate::ui::retained_host::host_contract::paint_workbench_renderer::native_panes::hierarchy_row_frame(&hierarchy.viewport, row_index, scroll, hierarchy.metrics);
        row.frame = crate::ui::retained_host::host_contract::data::TemplateNodeFrameData {
            x: geometry.x,
            y: geometry.y,
            width: geometry.width,
            height: geometry.height,
        };
        row.has_clip_frame = false;
        rows.push(row);
    }
    draw_template_nodes(
        frame,
        &ModelRc::from(std::rc::Rc::new(
            crate::ui::retained_host::primitives::VecModel::from(rows),
        )),
        &zero_origin(),
        &clip,
        Some(&focus),
    );
    crate::ui::retained_host::host_contract::paint_workbench_renderer::native_panes::draw_hierarchy_scrollbar(frame, hierarchy.pane, &hierarchy.viewport, &clip, &interaction, hierarchy.metrics);
}

fn subtree_rows(presentation: &HostWindowPresentationData, root_id: &str) -> HashSet<usize> {
    let index = node_index(&presentation.workbench_window_nodes);
    presentation
        .workbench_window_nodes
        .iter()
        .enumerate()
        .filter_map(|(row, node)| {
            reaches_subtree_root(node.node_id.as_str(), root_id, &index).then_some(row)
        })
        .collect()
}

fn authored_viewport_toolbar<'a>(
    presentation: &'a HostWindowPresentationData,
    viewport_root_id: &str,
) -> Option<(usize, &'a TemplatePaneNodeData)> {
    let index = node_index(&presentation.workbench_window_nodes);
    presentation
        .workbench_window_nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.control_id.as_str() == "UnrealViewportToolbar"
                && reaches_subtree_root(node.node_id.as_str(), viewport_root_id, &index)
        })
}

fn draw_authored_viewport_media(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    document: &crate::ui::retained_host::host_contract::data::HostDocumentDockSurfaceData,
    pane_clip: &FrameRect,
) {
    let Some(viewport) = authored_level_viewport(presentation, document) else {
        return;
    };
    let Some(clip) = intersect_rect(&viewport.clip, pane_clip) else {
        return;
    };
    let images = paint_viewport_images(presentation);
    frame.with_render_source_frame(viewport.media.source_surface_frame.as_ref(), |frame| {
        frame.with_render_source_command(viewport.media.surface_render_command_ref, |frame| {
            crate::ui::retained_host::host_contract::paint_workbench_renderer::native_panes::draw_viewport_image_for_surface(
                frame, &document.pane, &viewport.frame, &clip, &images, Some(document.surface_key.as_str()),
            );
        });
    });
}
