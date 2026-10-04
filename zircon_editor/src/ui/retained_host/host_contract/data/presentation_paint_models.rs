use crate::ui::retained_host::primitives::ModelRc;

use super::{
    HostBottomDockSurfaceData, HostDockPresentationPatch, HostSideDockSurfaceData,
    HostWindowPresentationData, PaneData, TemplatePaneNodeData,
};

impl HostWindowPresentationData {
    pub(crate) fn paint_node_models(&self) -> Vec<ModelRc<TemplatePaneNodeData>> {
        let mut models = vec![self.workbench_window_nodes.clone()];
        self.visit_paint_node_models(|nodes| push_unique_model(&mut models, nodes));
        models
    }

    pub(crate) fn paint_node_model_occurrences(
        &self,
    ) -> Vec<(ModelRc<TemplatePaneNodeData>, usize)> {
        let mut occurrences = Vec::new();
        increment_model_occurrence(&mut occurrences, &self.workbench_window_nodes);
        self.visit_paint_node_models(|nodes| increment_model_occurrence(&mut occurrences, nodes));
        occurrences
    }

    fn visit_paint_node_models(&self, mut visit: impl FnMut(&ModelRc<TemplatePaneNodeData>)) {
        visit(&self.root_template_nodes);
        let scene = &self.host_scene_data;
        visit(&scene.menu_chrome.template_nodes);
        visit(&scene.page_chrome.template_nodes);
        visit(&scene.status_bar.template_nodes);
        for menu in scene.menu_chrome.menus.iter() {
            visit(&menu.popup_nodes);
        }

        visit(&scene.left_dock.rail_nodes);
        visit(&scene.left_dock.header_nodes);
        visit_pane(&scene.left_dock.pane, &mut visit);
        for leaf in scene.document_surfaces() {
            visit(&leaf.header_nodes);
            visit_pane(&leaf.pane, &mut visit);
        }
        visit(&scene.right_dock.rail_nodes);
        visit(&scene.right_dock.header_nodes);
        visit_pane(&scene.right_dock.pane, &mut visit);
        visit(&scene.bottom_dock.header_nodes);
        visit_pane(&scene.bottom_dock.pane, &mut visit);

        for window in scene.floating_layer.floating_windows.iter() {
            visit(&window.header_nodes);
            visit_pane(&window.active_pane, &mut visit);
        }
        for window in self.native_floating_surface_data.floating_windows.iter() {
            visit(&window.header_nodes);
            visit_pane(&window.active_pane, &mut visit);
        }
    }
}

impl HostDockPresentationPatch {
    pub(crate) fn previous_paint_node_models(
        &self,
        presentation: &HostWindowPresentationData,
    ) -> Vec<ModelRc<TemplatePaneNodeData>> {
        match self {
            Self::Left(_) => side_dock_paint_node_models(&presentation.host_scene_data.left_dock),
            Self::Right(_) => side_dock_paint_node_models(&presentation.host_scene_data.right_dock),
            Self::Bottom(_) => {
                bottom_dock_paint_node_models(&presentation.host_scene_data.bottom_dock)
            }
        }
    }

    pub(crate) fn paint_node_models(&self) -> Vec<ModelRc<TemplatePaneNodeData>> {
        match self {
            Self::Left(dock) | Self::Right(dock) => side_dock_paint_node_models(dock),
            Self::Bottom(dock) => bottom_dock_paint_node_models(dock),
        }
    }
}

fn side_dock_paint_node_models(
    dock: &HostSideDockSurfaceData,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = Vec::with_capacity(3);
    push_unique_model(&mut models, &dock.rail_nodes);
    push_unique_model(&mut models, &dock.header_nodes);
    push_pane_model(&mut models, &dock.pane);
    models
}

fn bottom_dock_paint_node_models(
    dock: &HostBottomDockSurfaceData,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = Vec::with_capacity(2);
    push_unique_model(&mut models, &dock.header_nodes);
    push_pane_model(&mut models, &dock.pane);
    models
}

fn visit_pane(pane: &PaneData, visit: &mut impl FnMut(&ModelRc<TemplatePaneNodeData>)) {
    if let Some(nodes) = pane.template_nodes() {
        visit(nodes);
    }
}

fn push_pane_model(models: &mut Vec<ModelRc<TemplatePaneNodeData>>, pane: &PaneData) {
    if let Some(nodes) = pane.template_nodes() {
        push_unique_model(models, nodes);
    }
}

fn push_unique_model(
    models: &mut Vec<ModelRc<TemplatePaneNodeData>>,
    nodes: &ModelRc<TemplatePaneNodeData>,
) {
    if nodes.row_count() == 0
        || models
            .iter()
            .any(|existing| existing.shares_values_with(nodes))
    {
        return;
    }
    models.push(nodes.clone());
}

fn increment_model_occurrence(
    occurrences: &mut Vec<(ModelRc<TemplatePaneNodeData>, usize)>,
    nodes: &ModelRc<TemplatePaneNodeData>,
) {
    if nodes.row_count() == 0 {
        return;
    }
    if let Some((_, count)) = occurrences
        .iter_mut()
        .find(|(existing, _)| existing.shares_values_with(nodes))
    {
        *count = count.saturating_add(1);
        return;
    }
    occurrences.push((nodes.clone(), 1));
}
