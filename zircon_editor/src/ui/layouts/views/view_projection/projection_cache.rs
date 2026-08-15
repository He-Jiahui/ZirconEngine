#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus};
use zircon_runtime_interface::ui::v2::UiV2CompiledDocument;
use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens, event_ui::UiNodeId, layout::UiSize,
};

use super::super::{ViewTemplateFrameData, ViewTemplateNodeData};
use super::retained_binding::{ViewTemplateTextBinding, ViewTemplateTextOverrideSemantics};
use super::{
    ViewTemplateNodeMaterialization, ViewTemplateNodePatch, ViewTemplateProjectionError,
    ViewTemplateProjectionRowSignature,
};

pub(super) struct CachedProjection {
    pub(super) base_rows: Rc<Vec<Rc<ViewTemplateNodeData>>>,
    pub(super) row_patches: Rc<BTreeMap<usize, Rc<ViewTemplateNodeData>>>,
}

enum ProjectionCacheUpdate {
    Ready(CachedProjection),
    TopologyChanged,
}

struct ProjectionCacheEntry {
    width_bits: u32,
    height_bits: u32,
    compiled: Arc<UiV2CompiledDocument>,
    design_tokens: Arc<EditorDesignTokens>,
    font_database_generation: u64,
    surface: UiSurface,
    text_override_semantics: Rc<Vec<ViewTemplateTextOverrideSemantics>>,
    text_bindings: BTreeMap<String, ViewTemplateTextBinding>,
    control_rows: Rc<BTreeMap<String, Vec<usize>>>,
    frame_source_node_ids: Rc<Vec<UiNodeId>>,
    frame_source_rows: BTreeMap<UiNodeId, Vec<usize>>,
    row_signatures: Rc<Vec<ViewTemplateProjectionRowSignature>>,
    surface_topology_signatures: Rc<Vec<ViewTemplateProjectionRowSignature>>,
    authored_node_patches: Rc<Vec<ViewTemplateNodePatch>>,
    last_text_overrides: BTreeMap<String, String>,
    last_node_patches: BTreeMap<String, ViewTemplateNodePatch>,
    base_rows: Rc<Vec<Rc<ViewTemplateNodeData>>>,
    row_patches: Rc<BTreeMap<usize, Rc<ViewTemplateNodeData>>>,
}

pub(super) fn projected_nodes<F>(
    document_tree_id: &str,
    width: f32,
    height: f32,
    compiled: &Arc<UiV2CompiledDocument>,
    design_tokens: &Arc<EditorDesignTokens>,
    text_overrides: &BTreeMap<String, String>,
    node_patches: &BTreeMap<String, ViewTemplateNodePatch>,
    build: F,
) -> Result<CachedProjection, ViewTemplateProjectionError>
where
    F: FnOnce()
        -> Result<(UiSurface, ViewTemplateNodeMaterialization), ViewTemplateProjectionError>,
{
    let width_bits = width.to_bits();
    let height_bits = height.to_bits();
    let font_database_generation = UiSurface::shared_font_database_generation();
    let mut build = Some(build);

    loop {
        let cache_matches = PROJECTION_CACHE.with(|cache| {
            cache.borrow().get(document_tree_id).is_some_and(|entry| {
                projection_resource_identity_matches(
                    &entry.compiled,
                    &entry.design_tokens,
                    compiled,
                    design_tokens,
                    entry.font_database_generation,
                    font_database_generation,
                )
            })
        });
        if !cache_matches {
            let build = build
                .take()
                .expect("projection builder is available until the cache is materialized");
            let (surface, materialization) = build()?;
            install_projection_cache_entry(
                document_tree_id,
                width_bits,
                height_bits,
                compiled,
                design_tokens,
                font_database_generation,
                surface,
                materialization,
            );
        }

        let update = PROJECTION_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let entry = cache
                .get_mut(document_tree_id)
                .expect("projection cache entry was inserted above");
            let size_changed = !projection_size_matches(
                entry.width_bits,
                entry.height_bits,
                width_bits,
                height_bits,
            );
            if size_changed
                || entry.last_text_overrides != *text_overrides
                || entry.last_node_patches != *node_patches
            {
                let changed_text_controls = entry
                    .last_text_overrides
                    .keys()
                    .chain(text_overrides.keys())
                    .filter(|control_id| {
                        entry.last_text_overrides.get(*control_id)
                            != text_overrides.get(*control_id)
                    })
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>();
                let changed_patch_controls = entry
                    .last_node_patches
                    .keys()
                    .chain(node_patches.keys())
                    .filter(|control_id| {
                        entry.last_node_patches.get(*control_id) != node_patches.get(*control_id)
                    })
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>();
                let mut surface_changed = false;
                for control_id in &changed_text_controls {
                    let Some(binding) = entry.text_bindings.get(control_id) else {
                        continue;
                    };
                    for (property, value) in binding
                        .requested_mutations(text_overrides.get(control_id).map(String::as_str))
                    {
                        let report =
                            entry
                                .surface
                                .mutate_property(UiPropertyMutationRequest::new(
                                    binding.node_id,
                                    property.clone(),
                                    value,
                                ))?;
                        match report.status {
                            UiPropertyMutationStatus::Accepted => {
                                surface_changed = true;
                                record_property_mutation_for_tests();
                            }
                            UiPropertyMutationStatus::Unchanged => {}
                            UiPropertyMutationStatus::Rejected => {
                                return Err(ViewTemplateProjectionError::BindingMutationRejected {
                                    control_id: control_id.clone(),
                                    property,
                                    detail: report.message.unwrap_or_else(|| {
                                        "runtime rejected the property value".to_string()
                                    }),
                                });
                            }
                        }
                    }
                }
                for control_id in &changed_patch_controls {
                    let Some(row) = entry
                        .control_rows
                        .get(control_id)
                        .and_then(|rows| rows.first())
                        .copied()
                    else {
                        continue;
                    };
                    let Some(binding) = entry.text_bindings.get(control_id) else {
                        continue;
                    };
                    let authored = &entry.authored_node_patches[row];
                    let previous = entry
                        .last_node_patches
                        .get(control_id)
                        .unwrap_or(authored)
                        .resolved_against(authored);
                    let desired = node_patches
                        .get(control_id)
                        .unwrap_or(authored)
                        .resolved_against(authored);
                    for (property, value) in desired.changed_properties(&previous) {
                        let report =
                            entry
                                .surface
                                .mutate_property(UiPropertyMutationRequest::new(
                                    binding.node_id,
                                    property,
                                    value,
                                ))?;
                        match report.status {
                            UiPropertyMutationStatus::Accepted => {
                                surface_changed = true;
                                record_property_mutation_for_tests();
                            }
                            UiPropertyMutationStatus::Unchanged => {}
                            UiPropertyMutationStatus::Rejected => {
                                return Err(ViewTemplateProjectionError::BindingMutationRejected {
                                    control_id: control_id.clone(),
                                    property: property.to_string(),
                                    detail: report.message.unwrap_or_else(|| {
                                        "runtime rejected the property value".to_string()
                                    }),
                                });
                            }
                        }
                    }
                }
                let mut changed_geometry = BTreeMap::new();
                if surface_changed || size_changed {
                    let rebuild = {
                        zircon_runtime::profile_scope!(
                            "editor",
                            "template_projection",
                            "rebuild_retained_surface"
                        );
                        entry.surface.rebuild_dirty(UiSize::new(width, height))?
                    };
                    record_incremental_rebuild_for_tests();
                    zircon_runtime::profile_counter!(
                        "editor",
                        "ui.template_projection.surface_rebuild_count",
                        1,
                    );
                    zircon_runtime::profile_counter!(
                        "editor",
                        "ui.template_projection.layout_visited_node_count",
                        rebuild.layout_visited_node_count,
                    );
                    zircon_runtime::profile_counter!(
                        "editor",
                        "ui.template_projection.geometry_changed_node_count",
                        rebuild.layout_geometry_changed_node_count,
                    );
                    let rebound_frame_source_node_ids = if surface_changed {
                        entry.surface_topology_signatures = Rc::new(
                            super::view_template_projection_row_signatures(&entry.surface),
                        );
                        std::collections::BTreeSet::new()
                    } else if size_changed {
                        let Some(rebound) = sync_projection_row_topology(document_tree_id, entry)
                        else {
                            zircon_runtime::profile_counter!(
                                "editor",
                                "ui.template_projection.topology_fallback_count",
                                1,
                            );
                            return Ok(ProjectionCacheUpdate::TopologyChanged);
                        };
                        zircon_runtime::profile_counter!(
                            "editor",
                            "ui.template_projection.frame_source_rebind_count",
                            rebound.len(),
                        );
                        rebound
                    } else {
                        std::collections::BTreeSet::new()
                    };
                    let changed_node_ids = entry.surface.last_layout_geometry_changed_node_ids();
                    for command in &entry.surface.render_extract.list.commands {
                        if !(changed_node_ids.contains(&command.node_id)
                            || rebound_frame_source_node_ids.contains(&command.node_id))
                            || !entry.frame_source_rows.contains_key(&command.node_id)
                        {
                            continue;
                        }
                        let frame = ViewTemplateFrameData {
                            x: command.frame.x,
                            y: command.frame.y,
                            width: command.frame.width,
                            height: command.frame.height,
                        };
                        let replace = changed_geometry
                            .get(&command.node_id)
                            .is_none_or(|current| frame_area(&frame) > frame_area(current));
                        if replace {
                            changed_geometry.insert(command.node_id, frame);
                        }
                    }
                    zircon_runtime::profile_counter!(
                        "editor",
                        "ui.template_projection.geometry_command_visit_count",
                        entry.surface.render_extract.list.commands.len(),
                    );
                    zircon_runtime::profile_counter!(
                        "editor",
                        "ui.template_projection.geometry_patch_node_count",
                        changed_geometry.len(),
                    );
                }
                entry.width_bits = width_bits;
                entry.height_bits = height_bits;

                let mut row_patches = entry.row_patches.as_ref().clone();
                let mut rows_changed = false;
                let mut changed_row_indices = std::collections::BTreeSet::new();
                for control_id in changed_text_controls
                    .iter()
                    .chain(changed_patch_controls.iter())
                {
                    let Some(control_rows) = entry.control_rows.get(control_id) else {
                        continue;
                    };
                    changed_row_indices.extend(control_rows.iter().copied());
                }
                for node_id in changed_geometry.keys() {
                    if let Some(frame_rows) = entry.frame_source_rows.get(node_id) {
                        changed_row_indices.extend(frame_rows.iter().copied());
                    }
                }
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.template_projection.row_patch_count",
                    changed_row_indices.len(),
                );
                for row in changed_row_indices {
                    let current = row_patches
                        .get(&row)
                        .or_else(|| entry.base_rows.get(row))
                        .expect("projection row must resolve");
                    let mut node = current.as_ref().clone();
                    let control_id = node.control_id.to_string();
                    if changed_text_controls.contains(&control_id) {
                        let requested_text = text_overrides.get(&control_id).map(String::as_str);
                        let current_value_number =
                            entry.text_bindings.get(&control_id).and_then(|binding| {
                                surface_value_number(&entry.surface, binding.node_id)
                            });
                        entry.text_override_semantics[row].apply(
                            &mut node,
                            requested_text,
                            current_value_number,
                        );
                    }
                    if changed_patch_controls.contains(&control_id) {
                        let authored = &entry.authored_node_patches[row];
                        node_patches
                            .get(&control_id)
                            .unwrap_or(authored)
                            .resolved_against(authored)
                            .apply(&mut node);
                    }
                    if let Some(frame) = changed_geometry.get(&entry.frame_source_node_ids[row]) {
                        node.frame = frame.clone();
                    }
                    if node == **current {
                        continue;
                    }
                    record_node_clone_for_tests(estimated_node_clone_owned_bytes(&node));
                    if node == *entry.base_rows[row] {
                        row_patches.remove(&row);
                    } else {
                        row_patches.insert(row, Rc::new(node));
                    }
                    rows_changed = true;
                }
                for control_id in &changed_text_controls {
                    if let Some(value) = text_overrides.get(control_id) {
                        entry
                            .last_text_overrides
                            .insert(control_id.clone(), value.clone());
                    } else {
                        entry.last_text_overrides.remove(control_id);
                    }
                }
                for control_id in &changed_patch_controls {
                    if let Some(value) = node_patches.get(control_id) {
                        entry
                            .last_node_patches
                            .insert(control_id.clone(), value.clone());
                    } else {
                        entry.last_node_patches.remove(control_id);
                    }
                }
                if rows_changed {
                    entry.row_patches = Rc::new(row_patches);
                }
            }

            Ok(ProjectionCacheUpdate::Ready(CachedProjection {
                base_rows: Rc::clone(&entry.base_rows),
                row_patches: Rc::clone(&entry.row_patches),
            }))
        })?;

        match update {
            ProjectionCacheUpdate::Ready(projection) => return Ok(projection),
            ProjectionCacheUpdate::TopologyChanged => {
                let build = build
                    .take()
                    .expect("size topology fallback retains the projection builder");
                let (surface, materialization) = build()?;
                install_projection_cache_entry(
                    document_tree_id,
                    width_bits,
                    height_bits,
                    compiled,
                    design_tokens,
                    font_database_generation,
                    surface,
                    materialization,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn install_projection_cache_entry(
    document_tree_id: &str,
    width_bits: u32,
    height_bits: u32,
    compiled: &Arc<UiV2CompiledDocument>,
    design_tokens: &Arc<EditorDesignTokens>,
    font_database_generation: u64,
    surface: UiSurface,
    materialization: ViewTemplateNodeMaterialization,
) {
    let ViewTemplateNodeMaterialization {
        nodes,
        text_override_semantics,
        text_bindings,
        frame_source_node_ids,
        row_signatures,
    } = materialization;
    debug_assert_eq!(nodes.len(), text_override_semantics.len());
    debug_assert_eq!(nodes.len(), row_signatures.len());
    let mut control_rows = BTreeMap::<String, Vec<usize>>::new();
    let mut frame_source_rows = BTreeMap::<UiNodeId, Vec<usize>>::new();
    let authored_node_patches = nodes
        .iter()
        .map(ViewTemplateNodePatch::authored)
        .collect::<Vec<_>>();
    for (row, node) in nodes.iter().enumerate() {
        if !node.control_id.is_empty() {
            control_rows
                .entry(node.control_id.to_string())
                .or_default()
                .push(row);
        }
        frame_source_rows
            .entry(frame_source_node_ids[row])
            .or_default()
            .push(row);
    }
    let control_rows = Rc::new(control_rows);
    let rows: Rc<Vec<Rc<ViewTemplateNodeData>>> = Rc::new(nodes.into_iter().map(Rc::new).collect());
    zircon_runtime::profile_counter!(
        "editor",
        "ui.template_projection.full_materialization_count",
        1,
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.template_projection.full_materialized_node_count",
        rows.len(),
    );
    PROJECTION_CACHE.with(|cache| {
        cache.borrow_mut().insert(
            document_tree_id.to_string(),
            ProjectionCacheEntry {
                width_bits,
                height_bits,
                compiled: Arc::clone(compiled),
                design_tokens: Arc::clone(design_tokens),
                font_database_generation,
                surface,
                text_override_semantics: Rc::new(text_override_semantics),
                text_bindings,
                control_rows,
                frame_source_node_ids: Rc::new(frame_source_node_ids),
                frame_source_rows,
                surface_topology_signatures: Rc::new(row_signatures.clone()),
                row_signatures: Rc::new(row_signatures),
                authored_node_patches: Rc::new(authored_node_patches),
                last_text_overrides: BTreeMap::new(),
                last_node_patches: BTreeMap::new(),
                base_rows: rows,
                row_patches: Rc::new(BTreeMap::new()),
            },
        );
    });
    #[cfg(test)]
    SURFACE_MATERIALIZATION_COUNT.set(SURFACE_MATERIALIZATION_COUNT.get() + 1);
}

fn sync_projection_row_topology(
    document_tree_id: &str,
    entry: &mut ProjectionCacheEntry,
) -> Option<std::collections::BTreeSet<UiNodeId>> {
    let current = super::view_template_projection_row_signatures(&entry.surface);
    if current.len() != entry.surface_topology_signatures.len() {
        zircon_runtime::profile_counter!(
            "editor",
            "ui.template_projection.topology_length_mismatch_count",
            1,
        );
        record_projection_topology_owner(document_tree_id);
        return None;
    }
    if current
        .iter()
        .zip(entry.surface_topology_signatures.iter())
        .any(|(current, cached)| {
            current.node_id != cached.node_id || current.command_kind != cached.command_kind
        })
    {
        let mut current_identities = current
            .iter()
            .map(projection_row_identity)
            .collect::<Vec<_>>();
        let mut cached_identities = entry
            .surface_topology_signatures
            .iter()
            .map(projection_row_identity)
            .collect::<Vec<_>>();
        current_identities.sort_unstable();
        cached_identities.sort_unstable();
        if current_identities == cached_identities {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.template_projection.topology_order_mismatch_count",
                1,
            );
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.template_projection.topology_identity_mismatch_count",
                1,
            );
        }
        record_projection_topology_owner(document_tree_id);
        return None;
    }

    entry.surface_topology_signatures = Rc::new(current.clone());
    if current.len() != entry.row_signatures.len()
        || current
            .iter()
            .zip(entry.row_signatures.iter())
            .any(|(current, cached)| {
                current.node_id != cached.node_id || current.command_kind != cached.command_kind
            })
    {
        return Some(std::collections::BTreeSet::new());
    }

    let rebound_frame_source_node_ids = current
        .iter()
        .zip(entry.row_signatures.iter())
        .filter_map(|(current, cached)| {
            (current.frame_source_node_id != cached.frame_source_node_id)
                .then_some(current.frame_source_node_id)
        })
        .collect::<std::collections::BTreeSet<_>>();
    if rebound_frame_source_node_ids.is_empty() {
        return Some(rebound_frame_source_node_ids);
    }

    let frame_source_node_ids = current
        .iter()
        .map(|signature| signature.frame_source_node_id)
        .collect::<Vec<_>>();
    let mut frame_source_rows = BTreeMap::<UiNodeId, Vec<usize>>::new();
    for (row, node_id) in frame_source_node_ids.iter().copied().enumerate() {
        frame_source_rows.entry(node_id).or_default().push(row);
    }
    entry.frame_source_node_ids = Rc::new(frame_source_node_ids);
    entry.frame_source_rows = frame_source_rows;
    entry.row_signatures = Rc::new(current);
    Some(rebound_frame_source_node_ids)
}

fn projection_row_identity(signature: &ViewTemplateProjectionRowSignature) -> (UiNodeId, u8) {
    let command_kind = match signature.command_kind {
        zircon_runtime_interface::ui::surface::UiRenderCommandKind::Group => 0,
        zircon_runtime_interface::ui::surface::UiRenderCommandKind::Quad => 1,
        zircon_runtime_interface::ui::surface::UiRenderCommandKind::Text => 2,
        zircon_runtime_interface::ui::surface::UiRenderCommandKind::Image => 3,
    };
    (signature.node_id, command_kind)
}

fn record_projection_topology_owner(document_tree_id: &str) {
    let _counter = if document_tree_id.contains("hierarchy") {
        "ui.template_projection.topology_fallback_hierarchy_count"
    } else if document_tree_id.contains("inspector") {
        "ui.template_projection.topology_fallback_inspector_count"
    } else if document_tree_id.contains("document") {
        "ui.template_projection.topology_fallback_document_count"
    } else {
        "ui.template_projection.topology_fallback_other_count"
    };
    zircon_runtime::profile_counter!("editor", _counter, 1);
}

fn frame_area(frame: &ViewTemplateFrameData) -> f32 {
    frame.width.max(0.0) * frame.height.max(0.0)
}

fn surface_value_number(surface: &UiSurface, node_id: UiNodeId) -> Option<f32> {
    surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .attributes
        .get("value")
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
}

#[allow(clippy::too_many_arguments)]
fn projection_resource_identity_matches(
    cached_compiled: &Arc<UiV2CompiledDocument>,
    cached_design_tokens: &Arc<EditorDesignTokens>,
    compiled: &Arc<UiV2CompiledDocument>,
    design_tokens: &Arc<EditorDesignTokens>,
    cached_font_database_generation: u64,
    font_database_generation: u64,
) -> bool {
    Arc::ptr_eq(cached_compiled, compiled)
        && Arc::ptr_eq(cached_design_tokens, design_tokens)
        && cached_font_database_generation == font_database_generation
}

fn projection_size_matches(
    cached_width_bits: u32,
    cached_height_bits: u32,
    width_bits: u32,
    height_bits: u32,
) -> bool {
    cached_width_bits == width_bits && cached_height_bits == height_bits
}

thread_local! {
    static PROJECTION_CACHE: RefCell<BTreeMap<String, ProjectionCacheEntry>> =
        RefCell::new(BTreeMap::new());
    #[cfg(test)]
    static SURFACE_MATERIALIZATION_COUNT: Cell<u64> = const { Cell::new(0) };
    #[cfg(test)]
    static NODE_CLONE_COUNT: Cell<u64> = const { Cell::new(0) };
    #[cfg(test)]
    static NODE_CLONE_OWNED_BYTES: Cell<u64> = const { Cell::new(0) };
    #[cfg(test)]
    static LEGACY_FULL_CLONE_COUNT: Cell<u64> = const { Cell::new(0) };
    #[cfg(test)]
    static PROPERTY_MUTATION_COUNT: Cell<u64> = const { Cell::new(0) };
    #[cfg(test)]
    static INCREMENTAL_REBUILD_COUNT: Cell<u64> = const { Cell::new(0) };
}

fn estimated_node_clone_owned_bytes(node: &ViewTemplateNodeData) -> usize {
    std::mem::size_of::<ViewTemplateNodeData>()
        + node.node_id.len()
        + node.control_id.len()
        + node.role.len()
        + node.text.len()
        + node.component_role.len()
        + node.component_variant.len()
        + node.value_text.len()
        + node.dispatch_kind.len()
        + node.action_id.len()
        + node.binding_id.len()
        + node.edit_action_id.len()
        + node.commit_action_id.len()
        + node.surface_variant.len()
        + node.text_tone.len()
        + node.button_variant.len()
        + node.text_align.len()
        + node.overflow.len()
        + node.transition_kind.len()
        + node.transition_easing.len()
        + node.transition_direction.len()
        + node.media_source.len()
        + node.icon_name.len()
}

#[cfg(test)]
fn record_node_clone_for_tests(owned_bytes: usize) {
    NODE_CLONE_COUNT.set(NODE_CLONE_COUNT.get() + 1);
    NODE_CLONE_OWNED_BYTES.set(
        NODE_CLONE_OWNED_BYTES
            .get()
            .saturating_add(owned_bytes as u64),
    );
}

#[cfg(not(test))]
fn record_node_clone_for_tests(_owned_bytes: usize) {}

#[cfg(test)]
pub(super) fn record_legacy_full_clone_for_tests() {
    LEGACY_FULL_CLONE_COUNT.set(LEGACY_FULL_CLONE_COUNT.get() + 1);
}

fn record_property_mutation_for_tests() {
    #[cfg(test)]
    PROPERTY_MUTATION_COUNT.set(PROPERTY_MUTATION_COUNT.get() + 1);
}

fn record_incremental_rebuild_for_tests() {
    #[cfg(test)]
    INCREMENTAL_REBUILD_COUNT.set(INCREMENTAL_REBUILD_COUNT.get() + 1);
}

#[cfg(test)]
pub(super) fn clear_for_tests() {
    PROJECTION_CACHE.with(|cache| cache.borrow_mut().clear());
    SURFACE_MATERIALIZATION_COUNT.set(0);
    NODE_CLONE_COUNT.set(0);
    NODE_CLONE_OWNED_BYTES.set(0);
    LEGACY_FULL_CLONE_COUNT.set(0);
    PROPERTY_MUTATION_COUNT.set(0);
    INCREMENTAL_REBUILD_COUNT.set(0);
}

#[cfg(test)]
pub(super) fn materialization_count_for_tests() -> u64 {
    surface_materialization_count_for_tests()
}

#[cfg(test)]
pub(super) fn surface_materialization_count_for_tests() -> u64 {
    SURFACE_MATERIALIZATION_COUNT.get()
}

#[cfg(test)]
pub(super) fn node_clone_count_for_tests() -> u64 {
    NODE_CLONE_COUNT.get()
}

#[cfg(test)]
pub(super) fn node_clone_owned_bytes_for_tests() -> u64 {
    NODE_CLONE_OWNED_BYTES.get()
}

#[cfg(test)]
pub(super) fn legacy_full_clone_count_for_tests() -> u64 {
    LEGACY_FULL_CLONE_COUNT.get()
}

#[cfg(test)]
pub(super) fn property_mutation_count_for_tests() -> u64 {
    PROPERTY_MUTATION_COUNT.get()
}

#[cfg(test)]
pub(super) fn incremental_rebuild_count_for_tests() -> u64 {
    INCREMENTAL_REBUILD_COUNT.get()
}

#[cfg(test)]
pub(super) fn surface_generation_for_tests(document_tree_id: &str) -> Option<u64> {
    PROJECTION_CACHE.with(|cache| {
        cache
            .borrow()
            .get(document_tree_id)
            .map(|entry| entry.surface.invalidation_generations().generation)
    })
}

#[cfg(test)]
pub(super) fn surface_string_property_for_tests(
    document_tree_id: &str,
    control_id: &str,
    property: &str,
) -> Option<String> {
    PROJECTION_CACHE.with(|cache| {
        cache
            .borrow()
            .get(document_tree_id)?
            .surface
            .tree
            .nodes
            .values()
            .find_map(|node| {
                let metadata = node.template_metadata.as_ref()?;
                (metadata.control_id.as_deref() == Some(control_id))
                    .then(|| {
                        metadata
                            .attributes
                            .get(property)?
                            .as_str()
                            .map(str::to_string)
                    })
                    .flatten()
            })
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime_interface::ui::v2::{UiV2ComponentGraph, UiV2NodeArena};

    use super::*;

    #[test]
    fn projection_resource_identity_ignores_size_but_rejects_resource_replacement() {
        let compiled = Arc::new(UiV2CompiledDocument {
            asset_id: "projection-cache-test".to_string(),
            arena: UiV2NodeArena::default(),
            node_handles: BTreeMap::new(),
            component_graph: UiV2ComponentGraph::default(),
        });
        let replacement_compiled = Arc::new(compiled.as_ref().clone());
        let design_tokens = Arc::new(EditorDesignTokens::workbench_dark());
        let replacement_design_tokens = Arc::new(design_tokens.as_ref().clone());
        let width_bits = 640.0_f32.to_bits();
        let height_bits = 480.0_f32.to_bits();

        assert!(projection_resource_identity_matches(
            &compiled,
            &design_tokens,
            &compiled,
            &design_tokens,
            7,
            7,
        ));
        assert_ne!(width_bits, 800.0_f32.to_bits());
        assert_eq!(height_bits, 480.0_f32.to_bits());
        assert!(!projection_resource_identity_matches(
            &compiled,
            &design_tokens,
            &replacement_compiled,
            &design_tokens,
            7,
            7,
        ));
        assert!(!projection_resource_identity_matches(
            &compiled,
            &design_tokens,
            &compiled,
            &replacement_design_tokens,
            7,
            7,
        ));
        assert!(!projection_resource_identity_matches(
            &compiled,
            &design_tokens,
            &compiled,
            &design_tokens,
            7,
            8,
        ));
    }
}
