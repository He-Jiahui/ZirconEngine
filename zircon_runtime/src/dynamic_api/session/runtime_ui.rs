use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zircon_runtime_interface::ui::accessibility::{
    UiAccessibilityActionRequest, UiAccessibilityDiagnostic, UiAccessibilityTreeSnapshot,
};
use zircon_runtime_interface::ui::dispatch::{
    UiAccessibilityInputEvent, UiClipboardInputEvent, UiInputEvent, UiInputEventMetadata,
    UiInputSequence, UiPointerEvent, UiPointerInputEvent, UiPointerSource,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::surface::{UiHitTestQuery, UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::super::bounded_json::BoundedJsonError;

use crate::asset::project::ProjectManager;
use crate::asset::{AssetKind, AssetUri, ImportedAsset, ProjectAssetManager};
use crate::core::framework::render::{
    UiRenderNodeIdProjection, UiRenderSubmission, UiRenderSubmissionSegment,
};
use crate::text::font::{FontCollectionService, RuntimeFontAssetClaimScope};
use crate::ui::dispatch::UiInputManager;
use crate::ui::surface::UiSurface;
use crate::ui::v2::{UiV2PrototypeStore, UiV2PrototypeStoreBuilder, UiV2SurfaceBuilder};

use super::error::{RuntimeProjectError, RuntimeProjectResult};

mod action_requests;
mod font_admission;
mod host_request_drain;
mod host_requests;
mod input_publication;

use action_requests::RuntimeUiActionRequestQueue;
use host_requests::RuntimeUiHostRequestQueue;
use input_publication::{
    RuntimeUiInputPublication, RuntimeUiInputQueryAdmission, RuntimeUiInputQueryRejectReason,
};

const RUNTIME_PROJECT_UI_TREE_ID: &str = "zircon-runtime-project-ui";
const NODE_ID_SURFACE_SHIFT: u32 = 48;
const NODE_ID_LOCAL_MASK: u64 = (1_u64 << NODE_ID_SURFACE_SHIFT) - 1;

#[derive(Default)]
pub(super) struct RuntimeUiSurfaceSet {
    surfaces: Vec<RuntimeUiSurface>,
    input_sequence: u64,
    focused_surfaces: BTreeSet<usize>,
    focused_surface: Option<usize>,
    navigation_surface: Option<usize>,
    input_publication: RuntimeUiInputPublication,
    pointer_capture_surfaces: BTreeMap<Option<u64>, usize>,
    pointer_positions: BTreeMap<Option<u64>, UiPoint>,
    action_requests: RuntimeUiActionRequestQueue,
    host_requests: RuntimeUiHostRequestQueue,
    render_cache: RuntimeUiAggregateRenderCache,
    _font_claim_scope: Option<RuntimeFontAssetClaimScope>,
}

struct RuntimeUiSurface {
    surface: UiSurface,
    input: UiInputManager,
}

impl RuntimeUiSurface {
    fn rebuild_dirty(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        self.input
            .synchronize_text_document_owners(&mut self.surface);
        self.surface.rebuild_dirty(root_size).map(|_| ())
    }
}

#[derive(Default)]
struct RuntimeUiAggregateRenderCache {
    viewport_size: Option<crate::core::math::UVec2>,
    render_generations: Vec<u64>,
    submission: Option<Arc<UiRenderSubmission>>,
}

impl RuntimeUiSurfaceSet {
    pub(super) fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    pub(super) fn dispatch_clipboard_result(
        &mut self,
        target_surface: u32,
        transfer_id: zircon_runtime_interface::ui::dispatch::UiClipboardTransferId,
        owner: UiNodeId,
        outcome: zircon_runtime_interface::ui::dispatch::UiClipboardTransferOutcome,
    ) -> Result<bool, UiTreeError> {
        let metadata = self.next_input_metadata();
        let Ok(target_surface) = usize::try_from(target_surface) else {
            return Ok(false);
        };
        let Some(runtime_surface) = self.surfaces.get_mut(target_surface) else {
            return Ok(false);
        };
        let focus_before = runtime_surface.surface.focus.focused;
        let result = runtime_surface.surface.dispatch_input_event_with_manager(
            &mut runtime_surface.input,
            UiInputEvent::Clipboard(UiClipboardInputEvent {
                metadata,
                transfer_id,
                owner,
                outcome,
            }),
        )?;
        let focus_after = runtime_surface.surface.focus.focused;
        self.update_focused_surface_after_dispatch(target_surface, focus_before, focus_after);
        self.record_dispatch_outputs(target_surface, &result);
        Ok(true)
    }

    pub(super) fn load(
        project: &ProjectManager,
        asset_manager: &ProjectAssetManager,
        roots: &[AssetUri],
        font_collection: Arc<FontCollectionService>,
    ) -> RuntimeProjectResult<Self> {
        if roots.is_empty() {
            return Ok(Self::default());
        }
        let prototype_store = project_ui_prototype_store(project, roots)?;
        let mut built_surfaces = Vec::with_capacity(roots.len());
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
            let surface =
                UiV2SurfaceBuilder::build_surface_with_prototype_store_and_font_collection(
                    tree_id,
                    document.as_ref(),
                    &prototype_store,
                    Arc::clone(&font_collection),
                )
                .map_err(|source| RuntimeProjectError::BuildRuntimeUiRoot {
                    root: root.to_string(),
                    detail: source.to_string(),
                })?;
            built_surfaces.push((root.to_string(), surface));
        }

        let mut font_claim_scope = font_collection.runtime_font_asset_claim_scope();
        font_admission::admit_surface_font_dependencies(
            built_surfaces.iter().map(|(_, surface)| surface),
            asset_manager,
            &mut font_claim_scope,
        );

        let mut surfaces = Vec::with_capacity(built_surfaces.len());
        for (root, mut surface) in built_surfaces {
            surface
                .compute_layout(UiSize::new(1280.0, 720.0))
                .map_err(|source| RuntimeProjectError::BuildRuntimeUiRoot {
                    root,
                    detail: source.to_string(),
                })?;
            surfaces.push(RuntimeUiSurface {
                surface,
                input: UiInputManager::summary(),
            });
        }
        let focused_surfaces = published_focused_surfaces(&surfaces);
        let focused_surface = focused_surfaces.last().copied();
        let navigation_surface = published_navigation_surface(&surfaces, focused_surface);
        Ok(Self {
            surfaces,
            input_sequence: 0,
            focused_surfaces,
            focused_surface,
            navigation_surface,
            input_publication: RuntimeUiInputPublication::default(),
            pointer_capture_surfaces: BTreeMap::new(),
            action_requests: RuntimeUiActionRequestQueue::default(),
            host_requests: RuntimeUiHostRequestQueue::default(),
            render_cache: RuntimeUiAggregateRenderCache::default(),
            _font_claim_scope: Some(font_claim_scope),
        })
    }

    pub(super) fn render_submission(
        &mut self,
        viewport_size: crate::core::math::UVec2,
    ) -> Result<Option<Arc<UiRenderSubmission>>, UiTreeError> {
        if self.surfaces.is_empty() {
            return Ok(None);
        }
        let root_size = ui_size(viewport_size);
        for runtime_surface in &mut self.surfaces {
            runtime_surface.rebuild_dirty(root_size)?;
        }
        self.refresh_input_owners_from_publication();
        self.publish_input_authority(viewport_size);
        let cache_hit = self.render_cache.viewport_size == Some(viewport_size)
            && self.render_cache.render_generations.len() == self.surfaces.len()
            && self
                .render_cache
                .render_generations
                .iter()
                .zip(&self.surfaces)
                .all(|(generation, runtime_surface)| {
                    *generation == runtime_surface.surface.invalidation_generations().render
                });
        if cache_hit {
            crate::profile_counter!("runtime", "ui.project_extract.cache_hit", 1);
            crate::profile_counter!("runtime", "ui.project_extract.rebuild_count", 0);
            return Ok(self.render_cache.submission.as_ref().map(Arc::clone));
        }

        let mut segments = Vec::with_capacity(self.surfaces.len());
        let mut command_count = 0_usize;
        for (surface_index, runtime_surface) in self.surfaces.iter().enumerate() {
            let surface = &runtime_surface.surface;
            let segment = UiRenderSubmissionSegment::projected(
                surface.render_frame_extract(),
                UiTreeId::new(RUNTIME_PROJECT_UI_TREE_ID),
                runtime_surface_node_id_projection(surface_index),
            );
            command_count = command_count.saturating_add(segment.command_count());
            segments.push(segment);
        }
        self.render_cache.viewport_size = Some(viewport_size);
        self.render_cache.render_generations.clear();
        self.render_cache.render_generations.extend(
            self.surfaces
                .iter()
                .map(|runtime_surface| runtime_surface.surface.invalidation_generations().render),
        );
        let submission = UiRenderSubmission::from_submission_segments(segments);
        crate::profile_counter!("runtime", "ui.project_extract.cache_hit", 0);
        crate::profile_counter!("runtime", "ui.project_extract.rebuild_count", 1);
        crate::profile_counter!(
            "runtime",
            "ui.project_extract.segment_handle_count",
            submission.segments().len()
        );
        crate::profile_counter!("runtime", "ui.project_extract.command_clone_count", 0);
        debug_assert_eq!(submission.command_count(), command_count);
        self.render_cache.submission = Some(Arc::clone(&submission));
        Ok(Some(submission))
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
        self.refresh_input_owners_from_publication();
        self.publish_input_authority(viewport_size);
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
        let focus_before = runtime_surface.surface.focus.focused;
        let result = runtime_surface.surface.dispatch_input_event_with_manager(
            &mut runtime_surface.input,
            UiInputEvent::Accessibility(UiAccessibilityInputEvent { metadata, request }),
        )?;
        let focus_after = runtime_surface.surface.focus.focused;
        let handled = result.reply.stops_propagation();
        self.update_focused_surface_after_dispatch(surface_index, focus_before, focus_after);
        self.record_dispatch_outputs(surface_index, &result);
        Ok(handled)
    }

    pub(super) fn dispatch_input(
        &mut self,
        viewport_size: crate::core::math::UVec2,
        event: UiInputEvent,
    ) -> Result<bool, UiTreeError> {
        if matches!(&event, UiInputEvent::MouseMotion(_)) {
            crate::profile_counter!("runtime", "ui.surface_set.input.unrouted_reject_count", 1);
            return Ok(false);
        }
        let root_size = ui_size(viewport_size);
        if input_requires_focus_owner(&event) {
            let Some(surface_index) = self.focused_surface.filter(|surface_index| {
                self.surfaces
                    .get(*surface_index)
                    .is_some_and(|surface| surface.surface.focus.focused.is_some())
            }) else {
                if let Some(stale_surface) = self.focused_surface.take() {
                    self.focused_surfaces.remove(&stale_surface);
                }
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_set.input.focus_owner_miss_count",
                    1
                );
                return Ok(false);
            };
            crate::profile_counter!(
                "runtime",
                "ui.surface_set.input.focus_direct_route_count",
                1
            );
            return self.dispatch_input_to_surface(surface_index, root_size, event, false);
        }
        if input_requires_navigation_owner(&event) {
            let Some(surface_index) = self.navigation_surface.filter(|surface_index| {
                self.surfaces.get(*surface_index).is_some_and(|surface| {
                    surface.surface.focus.focused.is_some()
                        || surface.surface.has_navigation_candidate()
                })
            }) else {
                self.navigation_surface = None;
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_set.input.navigation_owner_miss_count",
                    1
                );
                return Ok(false);
            };
            crate::profile_counter!(
                "runtime",
                "ui.surface_set.input.navigation_direct_route_count",
                1
            );
            return self.dispatch_input_to_surface(surface_index, root_size, event, false);
        }
        let mut event = Some(event);
        for surface_index in (0..self.surfaces.len()).rev() {
            let Some(event) = input_event_for_surface(&mut event, surface_index == 0) else {
                return Ok(false);
            };
            if self.dispatch_input_to_surface(surface_index, root_size, event, true)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn dispatch_input_to_surface(
        &mut self,
        surface_index: usize,
        root_size: UiSize,
        event: UiInputEvent,
        rebuild_before_dispatch: bool,
    ) -> Result<bool, UiTreeError> {
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            self.focused_surfaces.remove(&surface_index);
            if self.focused_surface == Some(surface_index) {
                self.focused_surface = self.focused_surfaces.last().copied();
            }
            return Ok(false);
        };
        let focus_before = runtime_surface.surface.focus.focused;
        if rebuild_before_dispatch {
            runtime_surface.rebuild_dirty(root_size)?;
        }
        let result = runtime_surface
            .surface
            .dispatch_input_event_with_manager(&mut runtime_surface.input, event)?;
        let focus_after = runtime_surface.surface.focus.focused;
        let handled = result.reply.stops_propagation();
        self.update_focused_surface_after_dispatch(surface_index, focus_before, focus_after);
        self.record_dispatch_outputs(surface_index, &result);
        Ok(handled)
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
            return Ok(false);
        };
        let kind = pointer.event.kind;
        let point = pointer.event.point;
        let previous_point = if kind == UiPointerEventKind::Down {
            point
        } else {
            self.pointer_positions
                .get(&pointer_id)
                .copied()
                .unwrap_or(point)
        };
        let root_size = ui_size(viewport_size);
        let query_admission = self
            .input_publication
            .query(viewport_size, point, previous_point);
        if kind == UiPointerEventKind::Down {
            self.pointer_capture_surfaces.remove(&pointer_id);
        }
        let published_query = match query_admission {
            RuntimeUiInputQueryAdmission::Published(query) => Some(query),
            RuntimeUiInputQueryAdmission::Unpublished => None,
            RuntimeUiInputQueryAdmission::Rejected(reason) => {
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_set.input.invalid_pointer_reject_count",
                    1
                );
                match reason {
                    RuntimeUiInputQueryRejectReason::NonFinitePointer => crate::profile_counter!(
                        "runtime",
                        "ui.surface_set.input.non_finite_pointer_reject_count",
                        1
                    ),
                    RuntimeUiInputQueryRejectReason::DegenerateViewport => crate::profile_counter!(
                        "runtime",
                        "ui.surface_set.input.degenerate_viewport_reject_count",
                        1
                    ),
                    RuntimeUiInputQueryRejectReason::AffineProjectionOverflow => {
                        crate::profile_counter!(
                            "runtime",
                            "ui.surface_set.input.affine_projection_reject_count",
                            1
                        )
                    }
                }
                if matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel) {
                    self.pointer_positions.remove(&pointer_id);
                    self.pointer_capture_surfaces.remove(&pointer_id);
                }
                return Ok(false);
            }
        };
        if pointer_id.is_some()
            && matches!(kind, UiPointerEventKind::Up | UiPointerEventKind::Cancel)
        {
            self.pointer_positions.remove(&pointer_id);
        } else {
            self.pointer_positions.insert(pointer_id, point);
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
                published_query.map(|query| query.hit_test_query()),
                false,
            );
        }
        if let Some(query) = published_query {
            crate::profile_counter!(
                "runtime",
                "ui.surface_set.input.candidate_surface_count",
                query.candidate_count()
            );
            let mut event = Some(event);
            for candidate_offset in 0..query.candidate_count() {
                let Some(surface_index) = self
                    .input_publication
                    .candidate_surface(query, candidate_offset)
                else {
                    break;
                };
                let Some(event) = input_event_for_surface(
                    &mut event,
                    candidate_offset.saturating_add(1) == query.candidate_count(),
                ) else {
                    return Ok(false);
                };
                if self.dispatch_pointer_to_surface(
                    surface_index,
                    root_size,
                    pointer_id,
                    kind,
                    event,
                    Some(query.hit_test_query()),
                    false,
                )? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_unavailable_fallback_count",
            1
        );
        let mut event = Some(event);
        for surface_index in (0..self.surfaces.len()).rev() {
            let Some(event) = input_event_for_surface(&mut event, surface_index == 0) else {
                return Ok(false);
            };
            let result = self.dispatch_pointer_to_surface(
                surface_index,
                root_size,
                pointer_id,
                kind,
                event,
                None,
                true,
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
        pointer_query: Option<UiHitTestQuery>,
        rebuild_before_dispatch: bool,
    ) -> Result<bool, UiTreeError> {
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            self.pointer_capture_surfaces.remove(&pointer_id);
            return Ok(false);
        };
        let focus_before = runtime_surface.surface.focus.focused;
        if rebuild_before_dispatch {
            runtime_surface.rebuild_dirty(root_size)?;
        }
        let result = runtime_surface.input.dispatch_input_event_with_query(
            &mut runtime_surface.surface,
            event,
            pointer_query,
        )?;
        let focus_after = runtime_surface.surface.focus.focused;
        self.update_focused_surface_after_dispatch(surface_index, focus_before, focus_after);
        update_capture_surface(
            &mut self.pointer_capture_surfaces,
            pointer_id,
            surface_index,
            kind,
            result
                .pointer_routing
                .as_ref()
                .and_then(|routing| routing.capture_target)
                .is_some(),
        );
        let handled = result.reply.stops_propagation();
        self.record_dispatch_outputs(surface_index, &result);
        Ok(handled)
    }

    fn refresh_input_owners_from_publication(&mut self) {
        self.focused_surfaces = published_focused_surfaces(&self.surfaces);
        if !self
            .focused_surface
            .is_some_and(|surface_index| self.focused_surfaces.contains(&surface_index))
        {
            self.focused_surface = self.focused_surfaces.last().copied();
        }
        self.navigation_surface =
            published_navigation_surface(&self.surfaces, self.focused_surface);
    }

    fn publish_input_authority(&mut self, viewport_size: crate::core::math::UVec2) {
        let report = self.input_publication.publish(
            viewport_size,
            self.surfaces.len(),
            self.surfaces
                .iter()
                .map(|surface| surface.surface.surface_frame()),
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_full_rebuild_count",
            report.full_rebuild as usize
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_patch_surface_count",
            report.patched_surface_count
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_visited_entry_count",
            report.visited_entry_count
        );
        crate::profile_counter!(
            "runtime",
            "ui.surface_set.input.publication_cell_membership_count",
            report.cell_membership_count
        );
    }

    fn update_focused_surface_after_dispatch(
        &mut self,
        surface_index: usize,
        focus_before: Option<UiNodeId>,
        focus_after: Option<UiNodeId>,
    ) {
        if focus_before != focus_after {
            if focus_after.is_some() {
                self.focused_surfaces.insert(surface_index);
                self.focused_surface = Some(surface_index);
                self.navigation_surface = Some(surface_index);
            } else {
                self.focused_surfaces.remove(&surface_index);
                if self.focused_surface == Some(surface_index) {
                    self.focused_surface = self.focused_surfaces.last().copied();
                }
                if self.navigation_surface == Some(surface_index)
                    && !self
                        .surfaces
                        .get(surface_index)
                        .is_some_and(|surface| surface.surface.has_navigation_candidate())
                {
                    self.navigation_surface = None;
                }
            }
        }
    }

    pub(super) fn next_input_metadata(&mut self) -> UiInputEventMetadata {
        self.input_sequence = self.input_sequence.saturating_add(1);
        UiInputEventMetadata::new(
            zircon_runtime_interface::ui::dispatch::UiInputTimestamp::default(),
            UiInputSequence::new(self.input_sequence),
        )
    }

    fn record_dispatch_outputs(
        &mut self,
        surface_index: usize,
        result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    ) {
        let Some(runtime_surface) = self.surfaces.get(surface_index) else {
            return;
        };
        let Ok(target_surface) = u32::try_from(surface_index) else {
            return;
        };
        let tree_id = runtime_surface.surface.tree.tree_id.clone();
        self.host_requests
            .record_result(target_surface, &tree_id, result);
        let revoked = self
            .action_requests
            .record_result(target_surface, &tree_id, result);
        if revoked.is_empty() {
            return;
        }
        let Some(runtime_surface) = self.surfaces.get_mut(surface_index) else {
            return;
        };
        for reference in revoked {
            runtime_surface.surface.revoke_secure_text_value(&reference);
        }
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
    runtime_surface_node_id_projection(surface_index).project(node_id)
}

fn runtime_surface_node_id_projection(surface_index: usize) -> UiRenderNodeIdProjection {
    let surface = (surface_index as u64).saturating_add(1);
    UiRenderNodeIdProjection::new(surface << NODE_ID_SURFACE_SHIFT, NODE_ID_LOCAL_MASK)
}

fn input_event_for_surface(
    event: &mut Option<UiInputEvent>,
    last_surface: bool,
) -> Option<UiInputEvent> {
    if last_surface {
        event.take()
    } else {
        event.as_ref().cloned()
    }
}

fn input_requires_focus_owner(event: &UiInputEvent) -> bool {
    matches!(
        event,
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_)
    )
}

fn input_requires_navigation_owner(event: &UiInputEvent) -> bool {
    matches!(event, UiInputEvent::Navigation(_) | UiInputEvent::Analog(_))
}

fn published_focused_surfaces(surfaces: &[RuntimeUiSurface]) -> BTreeSet<usize> {
    surfaces
        .iter()
        .enumerate()
        .filter_map(|(surface_index, surface)| {
            surface
                .surface
                .focus
                .focused
                .is_some()
                .then_some(surface_index)
        })
        .collect()
}

fn published_navigation_surface(
    surfaces: &[RuntimeUiSurface],
    focused_surface: Option<usize>,
) -> Option<usize> {
    focused_surface.or_else(|| {
        surfaces
            .iter()
            .enumerate()
            .rev()
            .find_map(|(surface_index, surface)| {
                surface
                    .surface
                    .has_navigation_candidate()
                    .then_some(surface_index)
            })
    })
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
    use std::sync::Arc;

    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        capture_surface_for_event, global_node_id, split_global_node_id, update_capture_surface,
        RuntimeUiSurface, RuntimeUiSurfaceSet,
    };
    use crate::ui::dispatch::UiInputManager;
    use crate::ui::surface::UiSurface;
    use zircon_runtime_interface::ui::dispatch::{
        UiInputEvent, UiInputEventMetadata, UiKeyboardInputEvent, UiKeyboardInputState,
        UiMouseMotionInputEvent, UiNavigationInputEvent,
    };
    use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{UiNavigationEventKind, UiPointerEventKind};
    use zircon_runtime_interface::ui::tree::{
        UiDirtyFlags, UiInputPolicy, UiStateFlags, UiTreeNode,
    };

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
    fn stable_project_ui_submission_reuses_the_same_allocation() {
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![RuntimeUiSurface {
                surface: UiSurface::new(zircon_runtime_interface::ui::event_ui::UiTreeId::new(
                    "test-runtime-ui",
                )),
                input: UiInputManager::default(),
            }],
            ..RuntimeUiSurfaceSet::default()
        };
        let viewport = crate::core::math::UVec2::new(640, 360);

        let first = surfaces.render_submission(viewport).unwrap().unwrap();
        let second = surfaces.render_submission(viewport).unwrap().unwrap();

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn local_surface_change_reuses_unchanged_segment_allocation() {
        let changed_node = UiNodeId::new(2);
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![
                test_runtime_surface("test-runtime-ui:first", UiNodeId::new(1)),
                test_runtime_surface("test-runtime-ui:second", changed_node),
            ],
            ..RuntimeUiSurfaceSet::default()
        };
        let viewport = crate::core::math::UVec2::new(640, 360);

        let first = surfaces.render_submission(viewport).unwrap().unwrap();
        surfaces.surfaces[1]
            .surface
            .mark_node_dirty(
                changed_node,
                UiDirtyFlags {
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )
            .unwrap();
        let second = surfaces.render_submission(viewport).unwrap().unwrap();

        assert!(!Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(
            first.segments()[0].extract(),
            second.segments()[0].extract()
        ));
        assert!(!Arc::ptr_eq(
            first.segments()[1].extract(),
            second.segments()[1].extract()
        ));
    }

    fn test_runtime_surface(tree_id: &str, node_id: UiNodeId) -> RuntimeUiSurface {
        let mut surface = UiSurface::new(UiTreeId::new(tree_id));
        surface.tree.insert_root(
            UiTreeNode::new(node_id, UiNodePath::new("root"))
                .with_frame(UiFrame::new(0.0, 0.0, 100.0, 40.0)),
        );
        RuntimeUiSurface {
            surface,
            input: UiInputManager::default(),
        }
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

    #[test]
    fn raw_mouse_motion_does_not_rebuild_or_dispatch_surfaces() {
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![
                test_runtime_surface("test-runtime-ui:bottom", UiNodeId::new(1)),
                test_runtime_surface("test-runtime-ui:top", UiNodeId::new(2)),
            ],
            ..RuntimeUiSurfaceSet::default()
        };
        let dirty_before = surfaces
            .surfaces
            .iter()
            .map(|surface| surface.surface.dirty_flags())
            .collect::<Vec<_>>();
        assert!(dirty_before.iter().any(|dirty| {
            dirty.layout || dirty.hit_test || dirty.render || dirty.style || dirty.text
        }));

        let handled = surfaces
            .dispatch_input(
                crate::core::math::UVec2::new(640, 360),
                UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
                    metadata: UiInputEventMetadata::default(),
                    delta_x: 2.0,
                    delta_y: -1.0,
                }),
            )
            .unwrap();

        assert!(!handled);
        assert_eq!(
            surfaces
                .surfaces
                .iter()
                .map(|surface| surface.surface.dirty_flags())
                .collect::<Vec<_>>(),
            dirty_before
        );
    }

    #[test]
    fn resized_pointer_directory_skips_non_candidate_dirty_surfaces() {
        let published_viewport = crate::core::math::UVec2::new(640, 360);
        let resized_viewport = crate::core::math::UVec2::new(1280, 720);
        let bottom_node = UiNodeId::new(5);
        let top_node = UiNodeId::new(6);
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![
                test_pointer_runtime_surface(
                    "test-runtime-ui:pointer-bottom",
                    bottom_node,
                    UiFrame::new(0.0, 0.0, 100.0, 40.0),
                ),
                test_pointer_runtime_surface(
                    "test-runtime-ui:pointer-top",
                    top_node,
                    UiFrame::new(300.0, 100.0, 100.0, 40.0),
                ),
            ],
            ..RuntimeUiSurfaceSet::default()
        };
        surfaces.render_submission(published_viewport).unwrap();
        let query = surfaces.input_publication.query(
            resized_viewport,
            zircon_runtime_interface::ui::layout::UiPoint::new(24.0, 24.0),
            zircon_runtime_interface::ui::layout::UiPoint::new(20.0, 20.0),
        );
        let RuntimeUiInputQueryAdmission::Published(query) = query else {
            panic!("published pointer query expected");
        };
        assert_eq!(query.candidate_count(), 1);
        assert_eq!(
            surfaces.input_publication.candidate_surface(query, 0),
            Some(0)
        );

        surfaces.surfaces[1]
            .surface
            .mark_node_dirty(
                top_node,
                UiDirtyFlags {
                    hit_test: true,
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )
            .unwrap();
        let top_dirty_before = surfaces.surfaces[1].surface.dirty_flags();

        surfaces
            .dispatch_pointer(
                resized_viewport,
                UiPointerEventKind::Move,
                zircon_runtime_interface::ui::layout::UiPoint::new(24.0, 24.0),
                None,
                None,
                zircon_runtime_interface::ui::dispatch::UiPointerSource::Mouse,
                0.0,
            )
            .unwrap();

        assert_eq!(surfaces.surfaces[1].surface.dirty_flags(), top_dirty_before);
    }

    #[test]
    fn invalid_pointer_input_does_not_probe_dirty_surfaces() {
        let viewport = crate::core::math::UVec2::new(640, 360);
        let node = UiNodeId::new(7);
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![test_pointer_runtime_surface(
                "test-runtime-ui:invalid-pointer",
                node,
                UiFrame::new(0.0, 0.0, 100.0, 40.0),
            )],
            ..RuntimeUiSurfaceSet::default()
        };
        surfaces.render_submission(viewport).unwrap();
        surfaces.surfaces[0]
            .surface
            .mark_node_dirty(
                node,
                UiDirtyFlags {
                    hit_test: true,
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )
            .unwrap();
        let dirty_before = surfaces.surfaces[0].surface.dirty_flags();

        assert!(!surfaces
            .dispatch_pointer(
                viewport,
                UiPointerEventKind::Move,
                UiPoint::new(f32::NAN, 12.0),
                None,
                None,
                zircon_runtime_interface::ui::dispatch::UiPointerSource::Mouse,
                0.0,
            )
            .unwrap());
        assert_eq!(surfaces.surfaces[0].surface.dirty_flags(), dirty_before);
    }

    #[test]
    fn focused_keyboard_input_dispatches_only_to_the_published_owner() {
        let mut bottom =
            test_focusable_runtime_surface("test-runtime-ui:focus-bottom", UiNodeId::new(11));
        bottom.surface.focus_node(UiNodeId::new(11)).unwrap();
        let top = test_runtime_surface("test-runtime-ui:focus-top", UiNodeId::new(22));
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![bottom, top],
            ..RuntimeUiSurfaceSet::default()
        };
        surfaces.refresh_input_owners_from_publication();
        let top_dirty_before = surfaces.surfaces[1].surface.dirty_flags();

        let handled = surfaces
            .dispatch_input(crate::core::math::UVec2::new(640, 360), keyboard_input())
            .unwrap();

        assert!(!handled);
        assert_eq!(surfaces.focused_surface, Some(0));
        assert_eq!(surfaces.surfaces[1].surface.dirty_flags(), top_dirty_before);
    }

    #[test]
    fn focused_input_without_a_published_owner_does_not_probe_surfaces() {
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![
                test_runtime_surface("test-runtime-ui:no-focus-bottom", UiNodeId::new(31)),
                test_runtime_surface("test-runtime-ui:no-focus-top", UiNodeId::new(32)),
            ],
            ..RuntimeUiSurfaceSet::default()
        };
        let dirty_before = surfaces
            .surfaces
            .iter()
            .map(|surface| surface.surface.dirty_flags())
            .collect::<Vec<_>>();

        assert!(!surfaces
            .dispatch_input(crate::core::math::UVec2::new(640, 360), keyboard_input(),)
            .unwrap());
        assert_eq!(
            surfaces
                .surfaces
                .iter()
                .map(|surface| surface.surface.dirty_flags())
                .collect::<Vec<_>>(),
            dirty_before
        );
    }

    #[test]
    fn a_focus_transition_selects_the_actual_surface_over_stack_order() {
        let mut bottom = test_focusable_runtime_surface(
            "test-runtime-ui:focus-transition-bottom",
            UiNodeId::new(41),
        );
        let mut top = test_focusable_runtime_surface(
            "test-runtime-ui:focus-transition-top",
            UiNodeId::new(42),
        );
        top.surface.focus_node(UiNodeId::new(42)).unwrap();
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![bottom, top],
            ..RuntimeUiSurfaceSet::default()
        };
        surfaces.refresh_input_owners_from_publication();
        assert_eq!(surfaces.focused_surface, Some(1));

        surfaces.surfaces[0]
            .surface
            .focus_node(UiNodeId::new(41))
            .unwrap();
        surfaces.update_focused_surface_after_dispatch(0, None, Some(UiNodeId::new(41)));

        assert_eq!(surfaces.focused_surface, Some(0));
        assert_eq!(
            surfaces.focused_surfaces,
            BTreeSet::from([0_usize, 1_usize])
        );
        surfaces.refresh_input_owners_from_publication();
        assert_eq!(surfaces.focused_surface, Some(0));
    }

    #[test]
    fn navigation_dispatches_only_to_the_published_eligible_surface() {
        let viewport = crate::core::math::UVec2::new(640, 360);
        let bottom_node = UiNodeId::new(51);
        let top_node = UiNodeId::new(52);
        let mut surfaces = RuntimeUiSurfaceSet {
            surfaces: vec![
                test_focusable_runtime_surface("test-runtime-ui:navigation-bottom", bottom_node),
                test_runtime_surface("test-runtime-ui:navigation-top", top_node),
            ],
            ..RuntimeUiSurfaceSet::default()
        };
        surfaces.render_submission(viewport).unwrap();
        assert_eq!(surfaces.navigation_surface, Some(0));

        surfaces.surfaces[1]
            .surface
            .mark_node_dirty(
                top_node,
                UiDirtyFlags {
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )
            .unwrap();
        let top_dirty_before = surfaces.surfaces[1].surface.dirty_flags();

        let handled = surfaces
            .dispatch_input(
                viewport,
                UiInputEvent::Navigation(UiNavigationInputEvent {
                    metadata: UiInputEventMetadata::default(),
                    kind: UiNavigationEventKind::Next,
                }),
            )
            .unwrap();

        assert!(handled);
        assert_eq!(surfaces.focused_surface, Some(0));
        assert_eq!(surfaces.navigation_surface, Some(0));
        assert_eq!(
            surfaces.surfaces[0].surface.focus.focused,
            Some(bottom_node)
        );
        assert_eq!(surfaces.surfaces[1].surface.dirty_flags(), top_dirty_before);
    }

    fn test_focusable_runtime_surface(tree_id: &str, node_id: UiNodeId) -> RuntimeUiSurface {
        let mut runtime_surface = test_runtime_surface(tree_id, node_id);
        runtime_surface
            .surface
            .tree
            .node_mut(node_id)
            .unwrap()
            .state_flags = UiStateFlags {
            visible: true,
            enabled: true,
            focusable: true,
            ..UiStateFlags::default()
        };
        runtime_surface
    }

    fn test_pointer_runtime_surface(
        tree_id: &str,
        node_id: UiNodeId,
        frame: UiFrame,
    ) -> RuntimeUiSurface {
        let mut surface = UiSurface::new(UiTreeId::new(tree_id));
        let mut node = UiTreeNode::new(node_id, UiNodePath::new("root"))
            .with_frame(frame)
            .with_input_policy(UiInputPolicy::Receive);
        node.state_flags.clickable = true;
        node.state_flags.hoverable = true;
        surface.tree.insert_root(node);
        RuntimeUiSurface {
            surface,
            input: UiInputManager::default(),
        }
    }

    fn keyboard_input() -> UiInputEvent {
        UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata: UiInputEventMetadata::default(),
            state: UiKeyboardInputState::Pressed,
            key_code: 65,
            scan_code: Some(30),
            physical_key: "KeyA".to_string(),
            logical_key: "A".to_string(),
            text: Some("a".to_string()),
        })
    }
}
