use std::any::Any;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::{
    AssetSurfaceMode, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::resource::ResourceKind;

use super::super::ViewTemplateNodeData;
use super::ViewTemplateNodeProjection;

#[cfg(test)]
mod hash_index_tests;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetWorkspaceProjectionGeneration {
    workspace_generation: u64,
    selected_resource_revision: Option<u64>,
    surface_mode: AssetSurfaceMode,
    view_mode: AssetViewMode,
    utility_tab: AssetUtilityTab,
    kind_filter: Option<ResourceKind>,
    search_query_fingerprint: u64,
    selection_fingerprint: u64,
    visible_folder_count: usize,
    visible_asset_count: usize,
}

impl AssetWorkspaceProjectionGeneration {
    pub(crate) fn from_snapshot(snapshot: &AssetWorkspaceSnapshot) -> Self {
        Self {
            workspace_generation: snapshot.catalog_revision,
            selected_resource_revision: snapshot.selection.resource_revision,
            surface_mode: snapshot.surface_mode,
            view_mode: snapshot.view_mode,
            utility_tab: snapshot.utility_tab,
            kind_filter: snapshot.kind_filter,
            search_query_fingerprint: fingerprint(&snapshot.search_query),
            selection_fingerprint: fingerprint(&(
                &snapshot.selected_folder_id,
                &snapshot.selected_asset_uuid,
                &snapshot.selection.uuid,
                &snapshot.selection.display_name,
                &snapshot.selection.locator,
                &snapshot.selection.preview_artifact_path,
                &snapshot.selection.meta_path,
                &snapshot.selection.toolkit_view_id,
            )),
            visible_folder_count: snapshot.visible_folders.len(),
            visible_asset_count: snapshot.visible_assets.len(),
        }
    }
}

fn fingerprint(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

struct ProjectionCompositionEntry {
    source_projection: ViewTemplateNodeProjection,
    source_output_rows: Rc<Vec<Option<usize>>>,
    generation: Box<dyn Any>,
    base_rows: Rc<Vec<Rc<ViewTemplateNodeData>>>,
    row_patches: Rc<BTreeMap<usize, Rc<ViewTemplateNodeData>>>,
    metadata: Rc<dyn Any>,
    model: ModelRc<ViewTemplateNodeData>,
}

pub(super) fn compose_model<K, M, F>(
    composition_id: &str,
    projection: ViewTemplateNodeProjection,
    generation: &K,
    compose: F,
) -> ModelRc<ViewTemplateNodeData>
where
    K: Any + Clone + PartialEq,
    M: Any,
    F: FnOnce(&mut Vec<ViewTemplateNodeData>) -> M,
{
    if let Some(model) = COMPOSITION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let entry = cache.get_mut(composition_id)?;
        let generation_matches = entry
            .generation
            .downcast_ref::<K>()
            .is_some_and(|cached| cached == generation);
        if !generation_matches {
            return None;
        }
        if entry.source_projection.shares_rows_with(&projection) {
            return Some(entry.model.clone());
        }
        patch_composed_binding_rows(entry, &projection)
    }) {
        return model;
    }

    let source_projection = projection.clone();
    let mut nodes = projection.iter().cloned().collect::<Vec<_>>();
    record_full_materialization_for_tests(nodes.len());
    let metadata = compose(&mut nodes);
    let source_output_rows = Rc::new(source_output_rows(&source_projection, &nodes));
    let base_rows = Rc::new(nodes.into_iter().map(Rc::new).collect::<Vec<_>>());
    let row_patches = Rc::new(BTreeMap::new());
    let metadata: Rc<dyn Any> = Rc::new(metadata);
    let model = ModelRc::from_shared_rows_overlay_with_metadata(
        Rc::clone(&base_rows),
        Rc::clone(&row_patches),
        Rc::clone(&metadata),
    );
    COMPOSITION_CACHE.with(|cache| {
        cache.borrow_mut().insert(
            composition_id.to_string(),
            ProjectionCompositionEntry {
                source_projection,
                source_output_rows,
                generation: Box::new(generation.clone()),
                base_rows,
                row_patches,
                metadata,
                model: model.clone(),
            },
        );
    });
    model
}

fn patch_composed_binding_rows(
    entry: &mut ProjectionCompositionEntry,
    next_source_projection: &ViewTemplateNodeProjection,
) -> Option<ModelRc<ViewTemplateNodeData>> {
    let changed_rows = next_source_projection.changed_rows_from(&entry.source_projection)?;

    let mut next_patches = entry.row_patches.as_ref().clone();
    let mut patched = false;
    for row in changed_rows {
        let previous_source = entry.source_projection.row_rc(row)?;
        let next_source = next_source_projection.row_rc(row)?;
        if previous_source.node_id != next_source.node_id
            || previous_source.control_id != next_source.control_id
        {
            return None;
        }
        let output_row = entry.source_output_rows.get(row).copied().flatten()?;
        let previous_composed = entry.model.get(output_row)?;
        let next_composed = merge_projected_binding_delta(
            previous_source.as_ref(),
            next_source.as_ref(),
            previous_composed,
        )?;
        if next_composed != *previous_composed {
            if next_composed == *entry.base_rows[output_row] {
                next_patches.remove(&output_row);
            } else {
                next_patches.insert(output_row, Rc::new(next_composed));
            }
            record_incremental_row_patch_for_tests();
            patched = true;
        }
    }

    entry.source_projection = next_source_projection.clone();
    if !patched {
        return Some(entry.model.clone());
    }
    entry.row_patches = Rc::new(next_patches);
    entry.model = ModelRc::from_shared_rows_overlay_with_metadata(
        Rc::clone(&entry.base_rows),
        Rc::clone(&entry.row_patches),
        Rc::clone(&entry.metadata),
    );
    Some(entry.model.clone())
}

fn source_output_rows(
    source_projection: &ViewTemplateNodeProjection,
    composed_nodes: &[ViewTemplateNodeData],
) -> Vec<Option<usize>> {
    let mut first_output_by_identity = HashMap::with_capacity(composed_nodes.len());
    for (row, node) in composed_nodes.iter().enumerate() {
        first_output_by_identity
            .entry((node.node_id.as_str(), node.control_id.as_str()))
            .or_insert(row);
    }

    source_projection
        .iter()
        .map(|source| {
            first_output_by_identity
                .get(&(source.node_id.as_str(), source.control_id.as_str()))
                .copied()
        })
        .collect()
}

fn merge_projected_binding_delta(
    previous_source: &ViewTemplateNodeData,
    next_source: &ViewTemplateNodeData,
    previous_composed: &ViewTemplateNodeData,
) -> Option<ViewTemplateNodeData> {
    let mut expected_source = previous_source.clone();
    apply_projected_binding_delta(&mut expected_source, previous_source, next_source);
    if expected_source != *next_source {
        return None;
    }

    let mut next_composed = previous_composed.clone();
    apply_projected_binding_delta(&mut next_composed, previous_source, next_source);
    Some(next_composed)
}

fn apply_projected_binding_delta(
    target: &mut ViewTemplateNodeData,
    previous: &ViewTemplateNodeData,
    next: &ViewTemplateNodeData,
) {
    if previous.text != next.text {
        target.text = next.text.clone();
    }
    if previous.value_text != next.value_text {
        target.value_text = next.value_text.clone();
    }
    if previous.value_number != next.value_number {
        target.value_number = next.value_number;
    }
    if previous.selected != next.selected {
        target.selected = next.selected;
    }
    if previous.focused != next.focused {
        target.focused = next.focused;
    }
    if previous.surface_variant != next.surface_variant {
        target.surface_variant = next.surface_variant.clone();
    }
    if previous.text_tone != next.text_tone {
        target.text_tone = next.text_tone.clone();
    }
    if previous.frame != next.frame {
        target.frame = next.frame.clone();
    }
}

thread_local! {
    static COMPOSITION_CACHE: RefCell<BTreeMap<String, ProjectionCompositionEntry>> =
        RefCell::new(BTreeMap::new());
    #[cfg(test)]
    static FULL_MATERIALIZATION_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    #[cfg(test)]
    static FULL_MATERIALIZED_NODE_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    #[cfg(test)]
    static INCREMENTAL_ROW_PATCH_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn clear_for_tests() {
    COMPOSITION_CACHE.with(|cache| cache.borrow_mut().clear());
    FULL_MATERIALIZATION_COUNT.with(|count| count.set(0));
    FULL_MATERIALIZED_NODE_COUNT.with(|count| count.set(0));
    INCREMENTAL_ROW_PATCH_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
fn record_full_materialization_for_tests(node_count: usize) {
    FULL_MATERIALIZATION_COUNT.with(|count| count.set(count.get() + 1));
    FULL_MATERIALIZED_NODE_COUNT
        .with(|count| count.set(count.get().saturating_add(node_count as u64)));
}

#[cfg(not(test))]
fn record_full_materialization_for_tests(_: usize) {}

#[cfg(test)]
fn record_incremental_row_patch_for_tests() {
    INCREMENTAL_ROW_PATCH_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(not(test))]
fn record_incremental_row_patch_for_tests() {}

#[cfg(test)]
fn composition_counts_for_tests() -> (u64, u64, u64) {
    (
        FULL_MATERIALIZATION_COUNT.with(std::cell::Cell::get),
        FULL_MATERIALIZED_NODE_COUNT.with(std::cell::Cell::get),
        INCREMENTAL_ROW_PATCH_COUNT.with(std::cell::Cell::get),
    )
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn same_generation_text_delta_patches_only_the_composed_target_row() {
        clear_for_tests();
        let compose_calls = Cell::new(0_u64);
        let stable = Rc::new(ViewTemplateNodeData {
            node_id: "stable".into(),
            control_id: "Stable".into(),
            text: "stable".into(),
            ..ViewTemplateNodeData::default()
        });
        let changing = Rc::new(ViewTemplateNodeData {
            node_id: "changing".into(),
            control_id: "Changing".into(),
            text: "before".into(),
            ..ViewTemplateNodeData::default()
        });
        let first_rows = Rc::new(vec![Rc::clone(&stable), Rc::clone(&changing)]);
        let first = compose_model(
            "composition.incremental_text",
            ViewTemplateNodeProjection {
                base_rows: Rc::clone(&first_rows),
                row_patches: Rc::new(BTreeMap::new()),
                source_frame: None,
            },
            &7_u64,
            |nodes| {
                compose_calls.set(compose_calls.get() + 1);
                nodes[1].surface_variant = "composed".into();
            },
        );

        let changed = Rc::new(ViewTemplateNodeData {
            text: "after".into(),
            ..changing.as_ref().clone()
        });
        let next = compose_model(
            "composition.incremental_text",
            ViewTemplateNodeProjection {
                base_rows: Rc::clone(&first_rows),
                row_patches: Rc::new(BTreeMap::from([(1, changed)])),
                source_frame: None,
            },
            &7_u64,
            |_| panic!("same-generation text patch must not rerun full composition"),
        );

        assert_eq!(compose_calls.get(), 1);
        assert_eq!(composition_counts_for_tests(), (1, 2, 1));
        assert!(first.shares_row_with(&next, 0));
        assert!(!first.shares_row_with(&next, 1));
        assert_eq!(next.get(1).map(|node| node.text.as_str()), Some("after"));
        assert_eq!(
            next.get(1).map(|node| node.surface_variant.as_str()),
            Some("composed")
        );
    }

    #[test]
    fn topology_change_reruns_full_composition() {
        clear_for_tests();
        let compose_calls = Cell::new(0_u64);
        let stable = Rc::new(ViewTemplateNodeData {
            node_id: "stable".into(),
            control_id: "Stable".into(),
            ..ViewTemplateNodeData::default()
        });
        let _ = compose_model(
            "composition.topology",
            ViewTemplateNodeProjection {
                base_rows: Rc::new(vec![Rc::clone(&stable)]),
                row_patches: Rc::new(BTreeMap::new()),
                source_frame: None,
            },
            &11_u64,
            |_| compose_calls.set(compose_calls.get() + 1),
        );

        let appended = Rc::new(ViewTemplateNodeData {
            node_id: "appended".into(),
            control_id: "Appended".into(),
            ..ViewTemplateNodeData::default()
        });
        let next = compose_model(
            "composition.topology",
            ViewTemplateNodeProjection {
                base_rows: Rc::new(vec![stable, appended]),
                row_patches: Rc::new(BTreeMap::new()),
                source_frame: None,
            },
            &11_u64,
            |_| compose_calls.set(compose_calls.get() + 1),
        );

        assert_eq!(compose_calls.get(), 2);
        assert_eq!(next.row_count(), 2);
        assert_eq!(composition_counts_for_tests(), (2, 3, 0));
    }
}
