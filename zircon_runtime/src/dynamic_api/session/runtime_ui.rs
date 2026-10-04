use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zircon_runtime_interface::ui::accessibility::{
    UiAccessibilityDiagnostic, UiAccessibilityTreeSnapshot,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::tree::UiTreeError;

use super::super::bounded_json::BoundedJsonError;

use crate::asset::project::ProjectManager;
use crate::asset::{AssetKind, AssetUri, ImportedAsset, ProjectAssetManager};
use crate::core::framework::render::{
    UiRenderNodeIdProjection, UiRenderSubmission, UiRenderSubmissionSegment,
};
use crate::text::font::RuntimeFontAssetClaimScope;
use crate::text::TextRuntimeContext;
use crate::ui::dispatch::UiInputManager;
use crate::ui::surface::UiSurface;
use crate::ui::v2::{
    source_path_identity_for_path, UiV2PrototypeStore, UiV2PrototypeStoreBuilder,
    UiV2SurfaceBuilder,
};
use zircon_runtime_interface::ui::dispatch::UiInputTimestamp;

use super::error::{RuntimeProjectError, RuntimeProjectResult};

mod action_requests;
mod font_admission;
mod host_request_drain;
mod host_requests;
mod input_publication;
mod input_routing;

use action_requests::RuntimeUiActionRequestQueue;
use host_requests::RuntimeUiHostRequestQueue;
use input_publication::RuntimeUiInputPublication;
use input_routing::{published_focused_surfaces, published_navigation_surface};

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
    pointer_capture_surfaces: HashMap<Option<u64>, usize>,
    pointer_positions: HashMap<Option<u64>, UiPoint>,
    action_requests: RuntimeUiActionRequestQueue,
    host_requests: RuntimeUiHostRequestQueue,
    render_cache: RuntimeUiAggregateRenderCache,
    input_clock: RuntimeUiInputClock,
    _font_claim_scope: Option<RuntimeFontAssetClaimScope>,
}

struct RuntimeUiInputClock {
    origin: Instant,
}

impl Default for RuntimeUiInputClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl RuntimeUiInputClock {
    fn now(&self) -> UiInputTimestamp {
        input_routing::ui_input_timestamp_at(self.origin, Instant::now())
    }
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

    pub(super) fn load(
        project: &ProjectManager,
        asset_manager: &ProjectAssetManager,
        roots: &[AssetUri],
        text_context: Arc<TextRuntimeContext>,
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
            let surface = UiV2SurfaceBuilder::build_surface_with_prototype_store_and_text_context(
                tree_id,
                document.as_ref(),
                &prototype_store,
                &text_context,
            )
            .map_err(|source| RuntimeProjectError::BuildRuntimeUiRoot {
                root: root.to_string(),
                detail: source.to_string(),
            })?;
            built_surfaces.push((root.to_string(), surface));
        }

        let font_collection = text_context.font_collection();
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
            pointer_capture_surfaces: HashMap::new(),
            pointer_positions: HashMap::new(),
            action_requests: RuntimeUiActionRequestQueue::default(),
            host_requests: RuntimeUiHostRequestQueue::default(),
            render_cache: RuntimeUiAggregateRenderCache::default(),
            input_clock: RuntimeUiInputClock::default(),
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
    for entry in project.asset_registry().entries_iter() {
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
        let source_path = project
            .source_path_for_uri(uri)
            .ok()
            .and_then(|path| source_path_identity_for_path(&path))
            .unwrap_or_else(|| uri.to_string());
        let _ =
            builder.insert_with_aliases_and_source_path(document, [uri.to_string()], source_path);
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

#[cfg(test)]
#[path = "runtime_ui/tests/cases.rs"]
mod tests;
