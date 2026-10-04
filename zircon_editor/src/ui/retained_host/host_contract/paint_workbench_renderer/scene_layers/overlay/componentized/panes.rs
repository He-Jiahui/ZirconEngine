use super::*;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    authored_hierarchy, authored_panes, is_authored_tree_row,
};

struct OrdinaryPaneSubtree(ExtensionWorkspaceSubtree);
impl TemplateNodePaintTransform for OrdinaryPaneSubtree {
    fn stream_row_visit_indices(
        &self,
        row_count: usize,
        clip: &FrameRect,
        visit: &mut dyn FnMut(usize),
    ) -> bool {
        self.0.stream_row_visit_indices(row_count, clip, visit)
    }
    fn transform_row(
        &self,
        _: usize,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        (!is_authored_tree_row(&node)).then_some((node, clip))
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
        let subtree = OrdinaryPaneSubtree(ExtensionWorkspaceSubtree::from_presentation(
            presentation,
            pane.root.node_id.as_str(),
            Some(pane.root_row),
            index.clone(),
        ));
        draw_template_nodes_with_transform(
            frame,
            &presentation.workbench_window_nodes,
            &zero_origin(),
            &clip,
            Some(&focus),
            Some(&subtree),
        );
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
