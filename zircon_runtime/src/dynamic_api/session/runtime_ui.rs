use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use zircon_runtime_interface::ui::accessibility::{
    UiAccessibilityActionRequest, UiAccessibilityDiagnostic, UiAccessibilityTreeSnapshot,
};
use zircon_runtime_interface::ui::dispatch::{
    UiAccessibilityInputEvent, UiInputEvent, UiInputEventMetadata, UiInputSequence, UiPointerEvent,
    UiPointerInputEvent, UiPointerSource,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiPointerButton, UiPointerEventKind, UiRenderExtract, UiRenderList,
};
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::super::bounded_json::BoundedJsonError;

use crate::asset::project::ProjectManager;
use crate::asset::{AssetKind, AssetUri, ImportedAsset};
use crate::ui::dispatch::UiInputManager;
use crate::ui::surface::UiSurface;
use crate::ui::v2::{UiV2PrototypeStore, UiV2PrototypeStoreBuilder, UiV2SurfaceBuilder};

use super::error::{RuntimeProjectError, RuntimeProjectResult};

const RUNTIME_PROJECT_UI_TREE_ID: &str = "zircon-runtime-project-ui";
const NODE_ID_SURFACE_SHIFT: u32 = 48;
const NODE_ID_LOCAL_MASK: u64 = (1_u64 << NODE_ID_SURFACE_SHIFT) - 1;

#[derive(Default)]
pub(super) struct RuntimeUiSurfaceSet {
    surfaces: Vec<RuntimeUiSurface>,
    input_sequence: u64,
    pointer_capture_surfaces: BTreeMap<Option<u64>, usize>,
}

struct RuntimeUiSurface {
    surface: UiSurface,
    input: UiInputManager,
}

impl RuntimeUiSurfaceSet {
    pub(super) fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    pub(super) fn load(project: &ProjectManager, roots: &[AssetUri]) -> RuntimeProjectResult<Self> {
        if roots.is_empty() {
            return Ok(Self::default());
        }
        let prototype_store = project_ui_prototype_store(project, roots)?;
        let mut surfaces = Vec::with_capacity(roots.len());
        for (surface_index, root) in roots.iter().enumerate() {
            let root_key = root.to_string();
            let Some(document) = prototype_store.get(&root_key) else {
                return Err(RuntimeProjectError::BuildRuntimeUiRoot {
                    root: root_key,
                    detail: "runtime UI root is absent from the project UI prototype store"
                        .to_string(),
                });
            };
            if !matches!(
                document.asset.kind,
                zircon_runtime_interface::ui::v2::UiV2AssetKind::View
            ) {
                return Err(RuntimeProjectError::BuildRuntimeUiRoot {
                    root: root.to_string(),
                    detail: "runtime UI roots must resolve to .zui view assets".to_string(),
                });
            }
            let tree_id = UiTreeId::new(format!("{RUNTIME_PROJECT_UI_TREE_ID}:{surface_index}"));
            let mut surface = UiV2SurfaceBuilder::build_surface_with_prototype_store(
                tree_id,
                document.as_ref(),
                &prototype_store,
            )
            .map_err(|source| RuntimeProjectError::BuildRuntimeUiRoot {
                root: root.to_string(),
                detail: source.to_string(),
            })?;
            surface
                .compute_layout(UiSize::new(1280.0, 720.0))
                .map_err(|source| RuntimeProjectError::BuildRuntimeUiRoot {
                    root: root.to_string(),
                    detail: source.to_string(),
                })?;
            surfaces.push(RuntimeUiSurface {
                surface,
                input: UiInputManager::default(),
            });
        }
        Ok(Self {
            surfaces,
            input_sequence: 0,
            pointer_capture_surfaces: BTreeMap::new(),
        })
    }

    pub(super) fn render_extract(
        &mut self,
        viewport_size: crate::core::math::UVec2,
    ) -> Result<Option<UiRenderExtract>, UiTreeError> {
        if self.surfaces.is_empty() {
            return Ok(None);
        }
        let root_size = ui_size(viewport_size);
        let mut commands = Vec::new();
        let mut raster_scale = 1.0;
        for (surface_index, runtime_surface) in self.surfaces.iter_mut().enumerate() {
            runtime_surface.surface.rebuild_dirty(root_size)?;
            raster_scale = runtime_surface.surface.render_extract.raster_scale;
            commands.extend(
                runtime_surface
                    .surface
                    .render_extract
                    .list
                    .commands
                    .iter()
                    .cloned()
                    .map(|mut command| {
                        command.node_id = global_node_id(surface_index, command.node_id);
                        command
                    }),
            );
        }
        Ok(Some(UiRenderExtract {
            tree_id: UiTreeId::new(RUNTIME_PROJECT_UI_TREE_ID),
            list: UiRenderList { commands },
            raster_scale,
        }))
    }

    pub(super) fn accessibility_snapshot(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        limit: zircon_runtime_interface::ZrRuntimePayloadLimitV1,
    ) -> Result<Option<UiAccessibilityTreeSnapshot>, BoundedJsonError> {
        if self.surfaces.is_empty() {
            return Ok(None);
        }
        let source_nodes = self.surfaces.iter().fold(0_usize, |count, surface| {
            count.saturating_add(surface.surface.accessibility_source_node_count())
        });
        if source_nodes > limit.max_items {
            return Err(BoundedJsonError::Items {
                observed: source_nodes,
                limit: limit.max_items,
            });
        }
        let started = Instant::now();
        let root_size = ui_size(viewport_size);
        let mut budget = crate::ui::accessibility::AccessibilityBuildBudget::new(limit);
        let mut snapshot = UiAccessibilityTreeSnapshot {
            tree_id: UiTreeId::new(RUNTIME_PROJECT_UI_TREE_ID),
            ..UiAccessibilityTreeSnapshot::default()
        };
        budget
            .observe_value(&snapshot, 0)
            .map_err(|error| accessibility_budget_error(error, limit))?;
        for (surface_index, runtime_surface) in self.surfaces.iter_mut().enumerate() {
            let elapsed = started.elapsed();
            let processing_limit = Duration::from_micros(limit.max_processing_time_micros);
            if elapsed > processing_limit {
                return Err(BoundedJsonError::ProcessingTime {
                    limit_micros: limit.max_processing_time_micros,
                });
            }
            runtime_surface
                .surface
                .rebuild_dirty(root_size)
                .map_err(|error| BoundedJsonError::Json(error.to_string()))?;
            let local = runtime_surface
                .surface
                .accessibility_snapshot_bounded(&mut budget)
                .map_err(|error| accessibility_budget_error(error, limit))?;
            let local = globalize_accessibility_snapshot(surface_index, local, &mut budget)
                .map_err(|error| accessibility_budget_error(error, limit))?;
            snapshot.roots.extend(local.roots);
            snapshot.nodes.extend(local.nodes);
            snapshot.diagnostics.extend(local.diagnostics);
            if let Some(focused) = local.focused {
                // Later manifest roots render and receive input above earlier roots.
                snapshot.focused = Some(focused);
            }
        }
        budget
            .validate_payload(&snapshot)
            .map_err(|error| accessibility_budget_error(error, limit))?;
        Ok(Some(snapshot))
    }

    pub(super) fn dispatch_accessibility_action(
        &mut self,
        mut request: UiAccessibilityActionRequest,
    ) -> Result<bool, UiTreeError> {
        let Some((surface_index, local_node_id)) = split_global_node_id(request.target) else {
            return Ok(false);
        };
        let metadata = self.next_input_metadata();
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            return Ok(false);
        };
        request.target = local_node_id;
        Ok(runtime_surface
            .surface
            .dispatch_input_event_with_manager(
                &mut runtime_surface.input,
                UiInputEvent::Accessibility(UiAccessibilityInputEvent { metadata, request }),
            )
            .map(|result| result.reply.stops_propagation())?)
    }

    pub(super) fn dispatch_input(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        event: UiInputEvent,
    ) -> Result<bool, UiTreeError> {
        let root_size = ui_size(viewport_size);
        for runtime_surface in self.surfaces.iter_mut().rev() {
            runtime_surface.surface.rebuild_dirty(root_size)?;
            let result = runtime_surface
                .surface
                .dispatch_input_event_with_manager(&mut runtime_surface.input, event.clone())?;
            if result.reply.stops_propagation() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn dispatch_pointer(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        kind: UiPointerEventKind,
        point: UiPoint,
        button: Option<UiPointerButton>,
        pointer_id: Option<u64>,
        pointer_source: UiPointerSource,
        scroll_delta: f32,
    ) -> Result<bool, UiTreeError> {
        let mut event = UiPointerEvent::new(kind, point).with_scroll_delta(scroll_delta);
        if let Some(button) = button {
            event = event.with_button(button);
        }
        let mut metadata = self.next_input_metadata();
        metadata.pointer_id =
            pointer_id.map(zircon_runtime_interface::ui::dispatch::UiPointerId::new);
        metadata.pointer_source = pointer_source;
        self.dispatch_pointer_input(
            viewport_size,
            pointer_id,
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata,
                event,
                precise_scroll: None,
            }),
        )
    }

    fn dispatch_pointer_input(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        pointer_id: Option<u64>,
        event: UiInputEvent,
    ) -> Result<bool, UiTreeError> {
        let UiInputEvent::Pointer(pointer) = &event else {
            unreachable!("runtime pointer dispatch requires a pointer event");
        };
        let kind = pointer.event.kind;
        let root_size = ui_size(viewport_size);
        if kind == UiPointerEventKind::Down {
            self.pointer_capture_surfaces.remove(&pointer_id);
        }
        if let Some(surface_index) =
            capture_surface_for_event(&self.pointer_capture_surfaces, pointer_id, kind)
        {
            return self.dispatch_pointer_to_surface(
                surface_index,
                root_size,
                pointer_id,
                kind,
                event,
            );
        }
        for surface_index in (0..self.surfaces.len()).rev() {
            let result = self.dispatch_pointer_to_surface(
                surface_index,
                root_size,
                pointer_id,
                kind,
                event.clone(),
            )?;
            if result {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn dispatch_pointer_to_surface(
        &mut self,
        surface_index: usize,
        root_size: UiSize,
        pointer_id: Option<u64>,
        kind: UiPointerEventKind,
        event: UiInputEvent,
    ) -> Result<bool, UiTreeError> {
        let runtime_surface = self
            .surfaces
            .get_mut(surface_index)
            .expect("runtime UI capture surface index must remain valid");
        runtime_surface.surface.rebuild_dirty(root_size)?;
        let result = runtime_surface
            .surface
            .dispatch_input_event_with_manager(&mut runtime_surface.input, event)?;
        update_capture_surface(
            &mut self.pointer_capture_surfaces,
            pointer_id,
            surface_index,
            kind,
            result.diagnostics.route_trace.capture_target.is_some(),
        );
        Ok(result.reply.stops_propagation())
    }

    pub(super) fn next_input_metadata(&mut self) -> UiInputEventMetadata {
        self.input_sequence = self.input_sequence.saturating_add(1);
        UiInputEventMetadata::new(
            zircon_runtime_interface::ui::dispatch::UiInputTimestamp::default(),
            UiInputSequence::new(self.input_sequence),
        )
    }
}

fn globalize_accessibility_snapshot(
    surface_index: usize,
    mut snapshot: UiAccessibilityTreeSnapshot,
    budget: &mut crate::ui::accessibility::AccessibilityBuildBudget,
) -> Result<UiAccessibilityTreeSnapshot, crate::ui::accessibility::AccessibilitySnapshotBudgetError>
{
    for root in &mut snapshot.roots {
        let global = global_node_id(surface_index, *root);
        budget.observe_replacement(root, &global, 2)?;
        *root = global;
    }
    for node in &mut snapshot.nodes {
        let global = global_node_id(surface_index, node.node_id);
        budget.observe_replacement(&node.node_id, &global, 3)?;
        node.node_id = global;
        for child in &mut node.children {
            let global = global_node_id(surface_index, *child);
            budget.observe_replacement(child, &global, 4)?;
            *child = global;
        }
        let labelled_by = node
            .labelled_by
            .map(|node_id| global_node_id(surface_index, node_id));
        budget.observe_replacement(&node.labelled_by, &labelled_by, 3)?;
        node.labelled_by = labelled_by;
        let label_for = node
            .label_for
            .map(|node_id| global_node_id(surface_index, node_id));
        budget.observe_replacement(&node.label_for, &label_for, 3)?;
        node.label_for = label_for;
        let node_path = node.node_path.take();
        let global_path = node_path
            .as_ref()
            .map(|path| UiNodePath::new(format!("surface-{surface_index}:{}", path.0)));
        budget.observe_replacement(&node_path, &global_path, 3)?;
        node.node_path = global_path;
    }
    for diagnostic in &mut snapshot.diagnostics {
        let node_id = diagnostic
            .node_id
            .map(|node_id| global_node_id(surface_index, node_id));
        budget.observe_replacement(&diagnostic.node_id, &node_id, 3)?;
        diagnostic.node_id = node_id;
    }
    let focused = snapshot
        .focused
        .map(|node_id| global_node_id(surface_index, node_id));
    budget.observe_replacement(&snapshot.focused, &focused, 2)?;
    snapshot.focused = focused;
    Ok(snapshot)
}

fn accessibility_budget_error(
    error: crate::ui::accessibility::AccessibilitySnapshotBudgetError,
    limit: zircon_runtime_interface::ZrRuntimePayloadLimitV1,
) -> BoundedJsonError {
    match error {
        crate::ui::accessibility::AccessibilitySnapshotBudgetError::EncodedBytes {
            observed,
            ..
        } => BoundedJsonError::EncodedBytes {
            observed,
            limit: limit.max_encoded_bytes,
        },
        crate::ui::accessibility::AccessibilitySnapshotBudgetError::Items { observed, .. } => {
            BoundedJsonError::Items {
                observed,
                limit: limit.max_items,
            }
        }
        crate::ui::accessibility::AccessibilitySnapshotBudgetError::ProcessingTime { .. } => {
            BoundedJsonError::ProcessingTime {
                limit_micros: limit.max_processing_time_micros,
            }
        }
        crate::ui::accessibility::AccessibilitySnapshotBudgetError::NestingDepth {
            observed,
            ..
        } => BoundedJsonError::NestingDepth {
            observed,
            limit: limit.max_nesting_depth,
        },
        crate::ui::accessibility::AccessibilitySnapshotBudgetError::Json(message) => {
            BoundedJsonError::Json(message)
        }
    }
}

/// Builds one immutable lookup table for all project and mounted-package `.zui`
/// artifacts before retained runtime surfaces are created. URI aliases retain the
/// authored `res://` import contract while the document id remains canonical for
/// compiler diagnostics and cross-document component references.
fn project_ui_prototype_store(
    project: &ProjectManager,
    roots: &[AssetUri],
) -> RuntimeProjectResult<UiV2PrototypeStore> {
    let mut builder = UiV2PrototypeStoreBuilder::new();
    for entry in project.asset_registry().entries() {
        if !matches!(
            entry.type_marker(),
            AssetKind::UiLayout | AssetKind::UiWidget | AssetKind::UiStyle
        ) {
            continue;
        }
        let uri = entry.path();
        let artifact = project.load_artifact(uri).map_err(|source| {
            RuntimeProjectError::LoadRuntimeUiRoot {
                root: uri.to_string(),
                source,
            }
        })?;
        let document = match artifact {
            ImportedAsset::UiV2View(asset) => asset.document,
            ImportedAsset::UiV2Component(asset) => asset.document,
            ImportedAsset::UiV2Style(asset) => asset.document,
            _ => continue,
        };
        let _ = builder.insert_with_aliases(document, [uri.to_string()]);
    }
    builder
        .build_for_roots(roots.iter().map(ToString::to_string))
        .map_err(|source| RuntimeProjectError::BuildRuntimeUiRoot {
            root: "project UI prototype store".to_string(),
            detail: source.to_string(),
        })
}

fn ui_size(viewport_size: crate::core::math::UVec2) -> UiSize {
    UiSize::new(viewport_size.x.max(1) as f32, viewport_size.y.max(1) as f32)
}

fn global_node_id(surface_index: usize, node_id: UiNodeId) -> UiNodeId {
    let surface = u64::try_from(surface_index + 1).expect("runtime UI surface index fits u64");
    UiNodeId::new((surface << NODE_ID_SURFACE_SHIFT) | (node_id.0 & NODE_ID_LOCAL_MASK))
}

fn split_global_node_id(node_id: UiNodeId) -> Option<(usize, UiNodeId)> {
    let surface = node_id.0 >> NODE_ID_SURFACE_SHIFT;
    let surface_index = usize::try_from(surface.checked_sub(1)?).ok()?;
    Some((surface_index, UiNodeId::new(node_id.0 & NODE_ID_LOCAL_MASK)))
}

fn capture_surface_for_event(
    capture_surfaces: &BTreeMap<Option<u64>, usize>,
    pointer_id: Option<u64>,
    kind: UiPointerEventKind,
) -> Option<usize> {
    matches!(
        kind,
        UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
    )
    .then(|| capture_surfaces.get(&pointer_id).copied())
    .flatten()
}

fn update_capture_surface(
    capture_surfaces: &mut BTreeMap<Option<u64>, usize>,
    pointer_id: Option<u64>,
    surface_index: usize,
    kind: UiPointerEventKind,
    captures_pointer: bool,
) {
    if matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel) {
        capture_surfaces.remove(&pointer_id);
    } else if captures_pointer {
        capture_surfaces.insert(pointer_id, surface_index);
    } else if capture_surfaces.get(&pointer_id) == Some(&surface_index) {
        capture_surfaces.remove(&pointer_id);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        capture_surface_for_event, global_node_id, split_global_node_id, update_capture_surface,
    };
    use zircon_runtime_interface::ui::event_ui::UiNodeId;
    use zircon_runtime_interface::ui::surface::UiPointerEventKind;

    #[test]
    fn global_node_ids_keep_same_local_ids_distinct_across_runtime_ui_surfaces() {
        let local = UiNodeId::new(41);
        let first = global_node_id(0, local);
        let second = global_node_id(1, local);

        assert_ne!(first, second);
        assert_eq!(split_global_node_id(first), Some((0, local)));
        assert_eq!(split_global_node_id(second), Some((1, local)));
    }

    #[test]
    fn pointer_capture_routes_follow_up_events_back_to_the_owning_surface() {
        let mut captures = BTreeMap::new();
        update_capture_surface(&mut captures, Some(7), 0, UiPointerEventKind::Down, true);

        assert_eq!(
            capture_surface_for_event(&captures, Some(7), UiPointerEventKind::Move),
            Some(0)
        );
        assert_eq!(
            capture_surface_for_event(&captures, Some(7), UiPointerEventKind::Up),
            Some(0)
        );

        update_capture_surface(&mut captures, Some(7), 0, UiPointerEventKind::Up, false);
        assert_eq!(
            capture_surface_for_event(&captures, Some(7), UiPointerEventKind::Move),
            None
        );
    }
}
