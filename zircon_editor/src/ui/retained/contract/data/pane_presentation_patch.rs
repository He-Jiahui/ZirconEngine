use std::collections::BTreeMap;

use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::{HostWindowPresentationData, PaneData, TemplatePaneNodeData};

#[derive(Clone)]
pub(crate) enum HostPanePresentationLocation {
    LeftDock,
    DocumentDock,
    DocumentLeaf { surface_key: SharedString },
    RightDock,
    BottomDock,
    Floating { row: usize, window_id: SharedString },
    NativeFloating { row: usize, window_id: SharedString },
}

pub(crate) struct HostPanePresentationPatch {
    replacements: Vec<HostPanePresentationReplacement>,
}

pub(crate) struct HostPresentationPatch {
    panes: HostPanePresentationPatch,
    workbench_nodes: Option<HostWorkbenchNodeReplacement>,
}

struct HostWorkbenchNodeReplacement {
    nodes: ModelRc<TemplatePaneNodeData>,
    changed_rows: Vec<usize>,
}

struct HostPanePresentationReplacement {
    location: HostPanePresentationLocation,
    expected_id: SharedString,
    expected_kind: SharedString,
    expected_nodes: Option<ModelRc<TemplatePaneNodeData>>,
    next_pane: PaneData,
}

impl HostPanePresentationPatch {
    pub(crate) fn new() -> Self {
        Self {
            replacements: Vec::new(),
        }
    }

    pub(crate) fn single(
        location: HostPanePresentationLocation,
        previous: &PaneData,
        next: PaneData,
    ) -> Self {
        let mut patch = Self::new();
        patch.push(location, previous, next);
        patch
    }

    pub(crate) fn push(
        &mut self,
        location: HostPanePresentationLocation,
        previous: &PaneData,
        next: PaneData,
    ) {
        self.replacements.push(HostPanePresentationReplacement {
            location,
            expected_id: previous.id.clone(),
            expected_kind: previous.kind.clone(),
            expected_nodes: paint_nodes(previous).cloned(),
            next_pane: next,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.replacements.is_empty()
    }

    pub(crate) fn replacement_count(&self) -> usize {
        self.replacements.len()
    }

    pub(crate) fn paint_model_replacements(
        &self,
        presentation: &HostWindowPresentationData,
    ) -> Option<Vec<(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)>> {
        if self.replacements.is_empty() {
            return None;
        }

        let mut model_occurrences = None;
        let mut model_replacements = Vec::with_capacity(self.replacements.len());
        for (index, replacement) in self.replacements.iter().enumerate() {
            if self.replacements[..index]
                .iter()
                .any(|previous| previous.location.same_target(&replacement.location))
            {
                return None;
            }
            let current = replacement.location.pane(presentation)?;
            if current.id != replacement.expected_id
                || current.kind != replacement.expected_kind
                || replacement.next_pane.id != replacement.expected_id
                || replacement.next_pane.kind != replacement.expected_kind
                || !same_optional_nodes(paint_nodes(current), replacement.expected_nodes.as_ref())
            {
                return None;
            }

            match (paint_nodes(current), paint_nodes(&replacement.next_pane)) {
                (None, None) => {}
                (Some(previous), Some(next)) if previous.shares_values_with(next) => {}
                (Some(previous), Some(next)) => {
                    let occurrences = model_occurrences
                        .get_or_insert_with(|| presentation.paint_node_model_occurrences());
                    if model_occurrence_count(occurrences, next) != 0 {
                        return None;
                    }
                    if !push_unique_model_replacement(&mut model_replacements, previous, next) {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        if let Some(occurrences) = model_occurrences.as_deref() {
            if model_replacements.iter().any(|(previous, _)| {
                let replaced_occurrences = self
                    .replacements
                    .iter()
                    .filter(|replacement| {
                        replacement
                            .expected_nodes
                            .as_ref()
                            .is_some_and(|nodes| nodes.shares_values_with(previous))
                    })
                    .count();
                model_occurrence_count(occurrences, previous) != replaced_occurrences
            }) {
                return None;
            }
        }
        Some(model_replacements)
    }

    pub(crate) fn apply(self, presentation: &mut HostWindowPresentationData) {
        let mut left = None;
        let mut document = None;
        let mut right = None;
        let mut bottom = None;
        let mut floating_rows = BTreeMap::new();
        let mut native_floating_rows = BTreeMap::new();

        for replacement in self.replacements {
            match replacement.location {
                HostPanePresentationLocation::LeftDock => left = Some(replacement.next_pane),
                HostPanePresentationLocation::DocumentDock => {
                    document = Some(replacement.next_pane)
                }
                HostPanePresentationLocation::DocumentLeaf { surface_key } => {
                    let leaf = presentation
                        .host_scene_data
                        .document_leaves
                        .iter_mut()
                        .find(|leaf| leaf.surface_key == surface_key)
                        .expect("validated document leaf must remain present");
                    leaf.pane = replacement.next_pane;
                }
                HostPanePresentationLocation::RightDock => right = Some(replacement.next_pane),
                HostPanePresentationLocation::BottomDock => bottom = Some(replacement.next_pane),
                HostPanePresentationLocation::Floating { row, .. } => {
                    let mut window = presentation
                        .host_scene_data
                        .floating_layer
                        .floating_windows
                        .get(row)
                        .expect("validated floating pane row must remain present")
                        .clone();
                    window.active_pane = replacement.next_pane;
                    floating_rows.insert(row, window);
                }
                HostPanePresentationLocation::NativeFloating { row, .. } => {
                    let mut window = presentation
                        .native_floating_surface_data
                        .floating_windows
                        .get(row)
                        .expect("validated native floating pane row must remain present")
                        .clone();
                    window.active_pane = replacement.next_pane;
                    native_floating_rows.insert(row, window);
                }
            }
        }

        if let Some(pane) = left {
            presentation.host_scene_data.left_dock.pane = pane;
        }
        if let Some(pane) = document {
            presentation.host_scene_data.document_dock.pane = pane;
        }
        if let Some(pane) = right {
            presentation.host_scene_data.right_dock.pane = pane;
        }
        if let Some(pane) = bottom {
            presentation.host_scene_data.bottom_dock.pane = pane;
        }
        if !floating_rows.is_empty() {
            presentation.host_scene_data.floating_layer.floating_windows = presentation
                .host_scene_data
                .floating_layer
                .floating_windows
                .with_row_patches(floating_rows);
        }
        if !native_floating_rows.is_empty() {
            presentation.native_floating_surface_data.floating_windows = presentation
                .native_floating_surface_data
                .floating_windows
                .with_row_patches(native_floating_rows);
        }
    }
}

impl HostPresentationPatch {
    pub(crate) const fn new(panes: HostPanePresentationPatch) -> Self {
        Self {
            panes,
            workbench_nodes: None,
        }
    }

    pub(crate) fn with_workbench_nodes(
        mut self,
        nodes: ModelRc<TemplatePaneNodeData>,
        changed_rows: Vec<usize>,
    ) -> Self {
        self.workbench_nodes = Some(HostWorkbenchNodeReplacement {
            nodes,
            changed_rows,
        });
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.panes.is_empty() && self.workbench_nodes.is_none()
    }

    pub(crate) fn pane_replacement_count(&self) -> usize {
        self.panes.replacement_count()
    }

    pub(crate) fn workbench_nodes(&self) -> Option<(&ModelRc<TemplatePaneNodeData>, &[usize])> {
        self.workbench_nodes
            .as_ref()
            .map(|replacement| (&replacement.nodes, replacement.changed_rows.as_slice()))
    }

    pub(crate) fn paint_model_replacements(
        &self,
        presentation: &HostWindowPresentationData,
    ) -> Option<Vec<(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)>> {
        if self.panes.is_empty() {
            Some(Vec::new())
        } else {
            self.panes.paint_model_replacements(presentation)
        }
    }

    pub(crate) fn apply(self, presentation: &mut HostWindowPresentationData) {
        if let Some(replacement) = self.workbench_nodes {
            presentation.workbench_window_nodes = replacement.nodes;
        }
        self.panes.apply(presentation);
    }
}

impl HostPanePresentationLocation {
    fn pane<'a>(&self, presentation: &'a HostWindowPresentationData) -> Option<&'a PaneData> {
        match self {
            Self::LeftDock => Some(&presentation.host_scene_data.left_dock.pane),
            Self::DocumentDock => Some(&presentation.host_scene_data.document_dock.pane),
            Self::DocumentLeaf { surface_key } => presentation
                .host_scene_data
                .document_leaves
                .iter()
                .find(|leaf| &leaf.surface_key == surface_key)
                .map(|leaf| &leaf.pane),
            Self::RightDock => Some(&presentation.host_scene_data.right_dock.pane),
            Self::BottomDock => Some(&presentation.host_scene_data.bottom_dock.pane),
            Self::Floating { row, window_id } => presentation
                .host_scene_data
                .floating_layer
                .floating_windows
                .get(*row)
                .filter(|window| window.window_id.as_str() == window_id.as_str())
                .map(|window| &window.active_pane),
            Self::NativeFloating { row, window_id } => presentation
                .native_floating_surface_data
                .floating_windows
                .get(*row)
                .filter(|window| window.window_id.as_str() == window_id.as_str())
                .map(|window| &window.active_pane),
        }
    }

    fn same_target(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::DocumentLeaf { surface_key: left },
                Self::DocumentLeaf { surface_key: right },
            ) => left == right,
            (Self::LeftDock, Self::LeftDock)
            | (Self::DocumentDock, Self::DocumentDock)
            | (Self::RightDock, Self::RightDock)
            | (Self::BottomDock, Self::BottomDock) => true,
            (
                Self::Floating {
                    row: left_row,
                    window_id: left_id,
                },
                Self::Floating {
                    row: right_row,
                    window_id: right_id,
                },
            )
            | (
                Self::NativeFloating {
                    row: left_row,
                    window_id: left_id,
                },
                Self::NativeFloating {
                    row: right_row,
                    window_id: right_id,
                },
            ) => left_row == right_row && left_id == right_id,
            _ => false,
        }
    }
}

fn paint_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    pane.template_nodes().filter(|nodes| nodes.row_count() > 0)
}

fn model_occurrence_count(
    occurrences: &[(ModelRc<TemplatePaneNodeData>, usize)],
    expected: &ModelRc<TemplatePaneNodeData>,
) -> usize {
    occurrences
        .iter()
        .find(|(nodes, _)| nodes.shares_values_with(expected))
        .map_or(0, |(_, count)| *count)
}

fn same_optional_nodes(
    left: Option<&ModelRc<TemplatePaneNodeData>>,
    right: Option<&ModelRc<TemplatePaneNodeData>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.shares_values_with(right),
        _ => false,
    }
}

fn push_unique_model_replacement(
    replacements: &mut Vec<(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)>,
    previous: &ModelRc<TemplatePaneNodeData>,
    next: &ModelRc<TemplatePaneNodeData>,
) -> bool {
    if let Some((_, existing_next)) = replacements
        .iter()
        .find(|(existing_previous, _)| existing_previous.shares_values_with(previous))
    {
        return existing_next.shares_values_with(next);
    }
    if replacements
        .iter()
        .any(|(_, existing_next)| existing_next.shares_values_with(next))
    {
        return false;
    }
    replacements.push((previous.clone(), next.clone()));
    true
}
