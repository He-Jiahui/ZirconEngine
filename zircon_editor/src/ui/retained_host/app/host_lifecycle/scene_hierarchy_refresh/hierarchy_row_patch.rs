use std::collections::BTreeMap;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{
    FloatingWindowData, HostWindowPresentationData, PaneData, SceneNodeData,
};

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
    if row_patches.is_empty() {
        return true;
    }

    let Some(rows) = first_presented_hierarchy_rows(presentation) else {
        return true;
    };
    let row_count = rows.row_count();
    if row_patches.keys().any(|row_index| *row_index >= row_count)
        || !presented_hierarchy_models_match(presentation, &rows)
    {
        return false;
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
        return false;
    };
    let patched_rows = rows.with_row_patches(materialized_patches);
    replace_presented_hierarchy_rows(presentation, &patched_rows);
    true
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
mod tests {
    use super::*;
    use crate::ui::retained_host::primitives::ModelRc;

    fn hierarchy_rows(count: usize) -> ModelRc<SceneNodeData> {
        ModelRc::with_metadata(
            (0..count)
                .map(|index| SceneNodeData {
                    id: index.to_string().into(),
                    name: format!("Entity {index}").into(),
                    ..SceneNodeData::default()
                })
                .collect(),
            "hierarchy",
        )
    }

    #[test]
    fn sparse_native_row_patch_reuses_unchanged_model_storage() {
        let rows = hierarchy_rows(10_000);
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
        presentation.host_scene_data.left_dock.pane.hierarchy.hierarchy_nodes = rows.clone();
        presentation.host_scene_data.right_dock.pane.kind = "Hierarchy".into();
        presentation.host_scene_data.right_dock.pane.hierarchy.hierarchy_nodes = rows.clone();

        assert!(patch_presented_hierarchy_rows(
            &mut presentation,
            &BTreeMap::from([(
                9_999,
                PresentedHierarchyRowPatch::new(
                    Some(SceneNodeData {
                        id: "9999".into(),
                        name: "Renamed".into(),
                        selected: true,
                        ..SceneNodeData::default()
                    }),
                    true,
                ),
            )]),
        ));

        let patched = &presentation
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes;
        let mirrored = &presentation
            .host_scene_data
            .right_dock
            .pane
            .hierarchy
            .hierarchy_nodes;
        assert!(rows.shares_row_with(patched, 0));
        assert!(!rows.shares_row_with(patched, 9_999));
        assert_eq!(patched.get(9_999).unwrap().name.as_str(), "Renamed");
        assert!(patched.shares_values_with(mirrored));
    }

    #[test]
    fn full_reflow_shares_one_native_generation_across_presented_hierarchy_panes() {
        let rows = hierarchy_rows(128);
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
        presentation.host_scene_data.right_dock.pane.kind = "Hierarchy".into();

        replace_presented_hierarchy_rows(&mut presentation, &rows);

        assert!(rows.shares_values_with(
            &presentation
                .host_scene_data
                .left_dock
                .pane
                .hierarchy
                .hierarchy_nodes
        ));
        assert!(rows.shares_values_with(
            &presentation
                .host_scene_data
                .right_dock
                .pane
                .hierarchy
                .hierarchy_nodes
        ));
    }

    #[test]
    fn selection_only_patch_reuses_existing_row_content() {
        let rows = hierarchy_rows(128);
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
        presentation.host_scene_data.left_dock.pane.hierarchy.hierarchy_nodes = rows;

        assert!(patch_presented_hierarchy_rows(
            &mut presentation,
            &BTreeMap::from([(127, PresentedHierarchyRowPatch::new(None, true))]),
        ));

        let patched = presentation
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
            .get(127)
            .cloned()
            .unwrap();
        assert_eq!(patched.id.as_str(), "127");
        assert_eq!(patched.name.as_str(), "Entity 127");
        assert_eq!(patched.depth, 0);
        assert!(patched.selected);
    }

    #[test]
    fn sparse_patch_rejects_divergent_presented_generations() {
        let rows = hierarchy_rows(128);
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
        presentation.host_scene_data.left_dock.pane.hierarchy.hierarchy_nodes = rows.clone();
        presentation.host_scene_data.right_dock.pane.kind = "Hierarchy".into();
        presentation.host_scene_data.right_dock.pane.hierarchy.hierarchy_nodes =
            hierarchy_rows(128);

        assert!(!patch_presented_hierarchy_rows(
            &mut presentation,
            &BTreeMap::from([(
                100,
                PresentedHierarchyRowPatch::new(Some(SceneNodeData::default()), false),
            )]),
        ));
        assert!(rows.shares_values_with(
            &presentation
                .host_scene_data
                .left_dock
                .pane
                .hierarchy
                .hierarchy_nodes
        ));
    }
}
