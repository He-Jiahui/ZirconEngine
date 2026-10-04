use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{
    FloatingWindowData, HostPanePresentationLocation, HostPanePresentationPatch,
    HostWindowPresentationData, PaneData, SceneNodeData,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct PresentedHierarchyRowPatch {
    replacement: Option<SceneNodeData>,
    selected: bool,
}

impl PresentedHierarchyRowPatch {
    pub(super) const fn new(replacement: Option<SceneNodeData>, selected: bool) -> Self {
        Self {
            replacement,
            selected,
        }
    }
}

pub(super) fn replace_presented_hierarchy_rows(
    presentation: &mut HostWindowPresentationData,
    rows: &ModelRc<SceneNodeData>,
) {
    let scene = &mut presentation.host_scene_data;
    replace_hierarchy_pane(&mut scene.left_dock.pane, rows);
    replace_hierarchy_pane(&mut scene.right_dock.pane, rows);
    replace_hierarchy_pane(&mut scene.bottom_dock.pane, rows);
    replace_hierarchy_pane(&mut scene.document_dock.pane, rows);
    replace_floating_hierarchy_panes(&mut scene.floating_layer.floating_windows, rows);
    replace_floating_hierarchy_panes(
        &mut presentation.native_floating_surface_data.floating_windows,
        rows,
    );
}

pub(super) fn patch_presented_hierarchy_rows(
    presentation: &mut HostWindowPresentationData,
    row_patches: &BTreeMap<usize, PresentedHierarchyRowPatch>,
) -> bool {
    let Some(patch) = build_presented_hierarchy_pane_patch(presentation, row_patches) else {
        return false;
    };
    patch.apply(presentation);
    true
}

pub(super) fn build_presented_hierarchy_pane_patch(
    presentation: &HostWindowPresentationData,
    row_patches: &BTreeMap<usize, PresentedHierarchyRowPatch>,
) -> Option<HostPanePresentationPatch> {
    if row_patches.is_empty() {
        return Some(HostPanePresentationPatch::new());
    }

    let Some(rows) = first_presented_hierarchy_rows(presentation) else {
        return Some(HostPanePresentationPatch::new());
    };
    let row_count = rows.row_count();
    if row_patches.keys().any(|row_index| *row_index >= row_count)
        || !presented_hierarchy_models_match(presentation, &rows)
    {
        return None;
    }
    let Some(materialized_patches) = row_patches
        .iter()
        .map(|(row_index, patch)| {
            let mut next = patch
                .replacement
                .clone()
                .or_else(|| rows.get(*row_index).cloned())?;
            next.selected = patch.selected;
            Some((*row_index, next))
        })
        .collect::<Option<BTreeMap<_, _>>>()
    else {
        return None;
    };
    let patched_rows = rows.with_row_patches(materialized_patches);
    let mut patch = HostPanePresentationPatch::new();
    let scene = &presentation.host_scene_data;
    append_hierarchy_pane_patch(
        &mut patch,
        HostPanePresentationLocation::LeftDock,
        &scene.left_dock.pane,
        &patched_rows,
    );
    append_hierarchy_pane_patch(
        &mut patch,
        HostPanePresentationLocation::RightDock,
        &scene.right_dock.pane,
        &patched_rows,
    );
    append_hierarchy_pane_patch(
        &mut patch,
        HostPanePresentationLocation::BottomDock,
        &scene.bottom_dock.pane,
        &patched_rows,
    );
    append_hierarchy_pane_patch(
        &mut patch,
        HostPanePresentationLocation::DocumentDock,
        &scene.document_dock.pane,
        &patched_rows,
    );
    append_floating_hierarchy_pane_patches(
        &mut patch,
        &scene.floating_layer.floating_windows,
        &patched_rows,
        false,
    );
    append_floating_hierarchy_pane_patches(
        &mut patch,
        &presentation.native_floating_surface_data.floating_windows,
        &patched_rows,
        true,
    );
    Some(patch)
}

fn append_hierarchy_pane_patch(
    patch: &mut HostPanePresentationPatch,
    location: HostPanePresentationLocation,
    pane: &PaneData,
    rows: &ModelRc<SceneNodeData>,
) {
    if pane.kind.as_str() != "Hierarchy" {
        return;
    }
    let mut next = pane.clone();
    next.hierarchy.hierarchy_nodes = rows.clone();
    patch.push(location, pane, next);
}

fn append_floating_hierarchy_pane_patches(
    patch: &mut HostPanePresentationPatch,
    windows: &ModelRc<FloatingWindowData>,
    rows: &ModelRc<SceneNodeData>,
    native: bool,
) {
    for (row, window) in windows.iter().enumerate() {
        if window.active_pane.kind.as_str() != "Hierarchy" {
            continue;
        }
        let location = if native {
            HostPanePresentationLocation::NativeFloating {
                row,
                window_id: window.window_id.clone(),
            }
        } else {
            HostPanePresentationLocation::Floating {
                row,
                window_id: window.window_id.clone(),
            }
        };
        append_hierarchy_pane_patch(patch, location, &window.active_pane, rows);
    }
}

fn first_presented_hierarchy_rows(
    presentation: &HostWindowPresentationData,
) -> Option<ModelRc<SceneNodeData>> {
    let scene = &presentation.host_scene_data;
    for pane in [
        &scene.left_dock.pane,
        &scene.right_dock.pane,
        &scene.bottom_dock.pane,
        &scene.document_dock.pane,
    ] {
        if let Some(rows) = hierarchy_rows(pane) {
            return Some(rows.clone());
        }
    }
    first_floating_hierarchy_rows(&scene.floating_layer.floating_windows).or_else(|| {
        first_floating_hierarchy_rows(&presentation.native_floating_surface_data.floating_windows)
    })
}

fn first_floating_hierarchy_rows(
    windows: &ModelRc<FloatingWindowData>,
) -> Option<ModelRc<SceneNodeData>> {
    windows
        .iter()
        .find_map(|window| hierarchy_rows(&window.active_pane).cloned())
}

fn presented_hierarchy_models_match(
    presentation: &HostWindowPresentationData,
    expected: &ModelRc<SceneNodeData>,
) -> bool {
    let scene = &presentation.host_scene_data;
    [
        &scene.left_dock.pane,
        &scene.right_dock.pane,
        &scene.bottom_dock.pane,
        &scene.document_dock.pane,
    ]
    .into_iter()
    .filter_map(hierarchy_rows)
    .all(|rows| rows.shares_values_with(expected))
        && floating_hierarchy_models_match(&scene.floating_layer.floating_windows, expected)
        && floating_hierarchy_models_match(
            &presentation.native_floating_surface_data.floating_windows,
            expected,
        )
}

fn floating_hierarchy_models_match(
    windows: &ModelRc<FloatingWindowData>,
    expected: &ModelRc<SceneNodeData>,
) -> bool {
    windows
        .iter()
        .filter_map(|window| hierarchy_rows(&window.active_pane))
        .all(|rows| rows.shares_values_with(expected))
}

fn hierarchy_rows(pane: &PaneData) -> Option<&ModelRc<SceneNodeData>> {
    (pane.kind.as_str() == "Hierarchy").then_some(&pane.hierarchy.hierarchy_nodes)
}

fn replace_hierarchy_pane(pane: &mut PaneData, rows: &ModelRc<SceneNodeData>) {
    if pane.kind.as_str() == "Hierarchy" {
        pane.hierarchy.hierarchy_nodes = rows.clone();
    }
}

fn replace_floating_hierarchy_panes(
    windows: &mut ModelRc<FloatingWindowData>,
    rows: &ModelRc<SceneNodeData>,
) {
    let window_patches = windows
        .iter()
        .enumerate()
        .filter_map(|(window_index, window)| {
            (window.active_pane.kind.as_str() == "Hierarchy").then(|| {
                let mut next = window.clone();
                replace_hierarchy_pane(&mut next.active_pane, rows);
                (window_index, next)
            })
        })
        .collect::<BTreeMap<_, _>>();
    if !window_patches.is_empty() {
        *windows = windows.with_row_patches(window_patches);
    }
}

#[cfg(test)]
#[path = "tests/hierarchy_row_patch.rs"]
mod tests;
