import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


PANE_PIPELINE_STAGE_COUNT = 4
GEOMETRY_SEMANTIC_PANE_CLONE_SITES = 8

CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs",
        (
            "pub(crate) struct PaneData",
            "pub body_surface_frame: Option<Arc<UiSurfaceFrame>>",
            "pub body_template_hit_index: Option<Arc<HostPaneTemplateHitIndex>>",
            "pub inspector: InspectorPaneData",
            "pub ui_asset: UiAssetEditorPaneData",
            "pub(crate) fn template_nodes(&self)",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/data/"
        "pane_presentation_patch.rs",
        (
            "pub(crate) struct HostPanePresentationPatch",
            "pub(crate) fn paint_model_replacements(",
            "let mut model_occurrences = None;",
            "previous.location.same_target(&replacement.location)",
            "push_unique_model_replacement",
            ".with_row_patches(floating_rows)",
            ".with_row_patches(native_floating_rows)",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/data/"
        "presentation_paint_models.rs",
        (
            "pub(crate) fn paint_node_models(&self)",
            "pub(crate) fn paint_node_model_occurrences(",
            "increment_model_occurrence",
            "self.visit_paint_node_models",
            "pane.template_nodes()",
            "pub(crate) fn previous_paint_node_models(",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/data/"
        "presentation_generation.rs",
        (
            "pub(crate) struct HostPresentationGeneration",
            "self.structure_generation == other.structure_generation",
            "pub(crate) fn structure_generation(&self) -> u64",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs",
        (
            "pub(crate) struct HostWindowPresentationData",
            "pub(crate) struct HostWindowGeometryPresentationData",
            "current.host_scene_data.left_dock.pane.clone()",
            "current.host_scene_data.document_dock.pane.clone()",
            "current.host_scene_data.right_dock.pane.clone()",
            "current.host_scene_data.bottom_dock.pane.clone()",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/ui/apply_presentation/scene_conversion.rs",
        (
            "to_host_contract_host_scene_geometry_with_retained_panes",
            "current.left_dock.pane.clone()",
            "current.document_dock.pane.clone()",
            "current.right_dock.pane.clone()",
            "current.bottom_dock.pane.clone()",
            ".map(|candidate| candidate.active_pane.clone())",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs",
        (
            "Arc::make_mut(&mut self.host_presentation)",
            ".indexes_presentation(&self.host_presentation)",
            "HostWorkbenchHitIndex::from_presentation(",
            "fn advance_structure_generation(&mut self)",
            "pub(crate) fn replace_native_floating_window_presentation(",
            "ui.native_floating_presentation.commit_count",
            "rebind_presentation_dock_patch",
            "pub(crate) fn patch_host_presentation_panes(",
            ".rebind_paint_models(&model_replacements)",
            "ui.pane_presentation_transaction.paint_model_rebind_node_count",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs",
        (
            "pub(crate) fn patch_host_presentation_dock",
            "pub(crate) fn patch_host_presentation_panes<R>",
            "pub(crate) fn set_native_floating_window_presentation(",
            "state.patch_host_presentation_panes(patch)",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/ui/scoped_presentation.rs",
        (
            "ui.patch_host_presentation_panes(",
            "prepare_ui_asset_presentation_transaction",
            "HostPanePresentationLocation::NativeFloating",
            "rebuild_pane_template_hit_artifacts(",
            "result.floating_window_rows_visited += native.floating_windows.row_count()",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
        "template_node/pane_nodes.rs",
        ("pane.template_nodes()",),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
        "docks/pane/template_nodes/selection.rs",
        ("pane.template_nodes()",),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/"
        "geometry/pane_frames/template_nodes/source.rs",
        ("pane.template_nodes()",),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
        "template_node.rs",
        (
            "pub(crate) fn rebuild_pane_template_hit_artifacts",
            "pane.body_surface_frame = build_template_surface_frame(",
            "pane.body_template_hit_index = Some(Arc::new(HostPaneTemplateHitIndex::new(nodes)))",
            ".filter(|index| index.indexes_nodes(nodes))",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
        "template_node/popup_rows.rs",
        (
            "for row in index.popup_rows().iter().rev().copied()",
            "for node in nodes.iter().rev()",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
        "template_node/index.rs",
        (
            "pub(crate) fn rebind_paint_models",
            "pub(crate) fn indexes_presentation",
            "let models = presentation.paint_node_models();",
            "HostTemplateNodePaintIndex::new",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/ui/shell_content_presentation.rs",
        (
            "fn presentation_paint_models_for_target(",
            "fn push_pane_paint_model(",
            "if let Some(nodes) = pane.template_nodes()",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
        "surface_frames.rs",
        (
            "ui.patch_host_presentation_panes(",
            "HostPanePresentationPatch::new()",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
        "tests/surface_frames.rs",
        ("stable_toolbar_publication_does_not_advance_presentation_generations",),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
        "surface_frames/docked.rs",
        (
            "append_docked_viewport_toolbar_surface_frame_patches",
            "HostPanePresentationLocation::DocumentDock",
            "patch.push(location, pane, next);",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
        "surface_frames/floating.rs",
        (
            "append_floating_viewport_toolbar_surface_frame_patches",
            "HostPanePresentationLocation::Floating",
            ".iter()",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
        "surface_frames/pane_frame.rs",
        (
            "next_viewport_toolbar_surface_frame_pane",
            "same_surface_frame",
            "Arc::ptr_eq(left, right)",
            ".then(|| {",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/native_windows/presentation.rs",
        (
            "ui.set_native_floating_window_presentation(",
            "fn apply_native_floating_presentation_data(",
            "fn native_floating_presentation_matches(",
        ),
    ),
    (
        "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
        "SlateInvalidationRoot.cpp",
        (
            "void FSlateInvalidationRoot::InvalidateRootLayout",
            "bool FSlateInvalidationRoot::PaintFastPath",
            "bool FSlateInvalidationRoot::ProcessInvalidation",
            "FinalUpdateList",
        ),
    ),
    (
        "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
        "WidgetProxy.cpp",
        (
            "FWidgetProxy::ProcessLayoutInvalidation",
            "EInvalidateWidgetReason::Paint",
            "EWidgetUpdateFlags::NeedsRenderTransform",
        ),
    ),
    (
        "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/"
        "HittestGrid.cpp",
        (
            "void FHittestGrid::AddWidget",
            "GetPaintSpaceGeometry()",
            "GetRenderBoundingRect()",
            "RemoveWidget(InWidget)",
        ),
    ),
    (
        "dev/Fyrox/fyrox-ui/src/widget.rs",
        (
            "pub fn invalidate_layout(&self)",
            "self.measure_valid.set(false)",
            "self.arrange_valid.set(false)",
            "self.visual_valid.set(false)",
        ),
    ),
    (
        "dev/Fyrox/fyrox-ui/src/lib.rs",
        (
            "node.is_arrange_valid() && node.prev_arrange.get() == *final_rect",
            "node.is_measure_valid() && node.prev_measure.get() == available_size",
            "if message.need_perform_layout()",
        ),
    ),
    (
        "dev/slint/internal/core/partial_renderer.rs",
        (
            "pub struct CachedRenderingData",
            "pub struct DirtyRegion",
            "pub struct PartialRenderingState",
            "pub fn mark_dirty_region(&self, region: DirtyRegion)",
        ),
    ),
)

FORBIDDEN_SOURCE_TOKENS = {
    "zircon_editor/src/ui/retained_host/host_contract/data/"
    "presentation_generation.rs": (
        "semantic_structure: Arc<HostWindowPresentationData>",
    ),
    "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs": (
        "host_structure: Arc<HostWindowPresentationData>",
        "self.host_structure = Arc::clone(&self.host_presentation);",
        "semantic_structure_identity: Arc<HostPresentationSemanticIdentity>",
        "self.semantic_structure_identity = Arc::new(HostPresentationSemanticIdentity);",
    ),
    "zircon_editor/src/ui/retained_host/ui/scoped_presentation.rs": (
        "pane.body_surface_frame = build_pane_template_surface_frame(",
        "ui.update_host_presentation_if(",
    ),
    "zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs": (
        "pub(crate) fn update_host_presentation_if",
    ),
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/pane_nodes.rs": (
        "match pane.kind.as_str()",
    ),
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "docks/pane/template_nodes/selection.rs": (
        "match pane.kind.as_str()",
    ),
    "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/"
    "geometry/pane_frames/template_nodes/source.rs": (
        "match pane.kind.as_str()",
    ),
    "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
    "surface_frames.rs": (
        "ui.update_host_presentation(",
    ),
    "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
    "surface_frames/floating.rs": (
        "model_rc(floating_windows)",
        ".row_data(row)",
    ),
    "zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/"
    "surface_frames/docked.rs": (
        "&mut document_dock.pane",
        "&mut left_dock.pane",
        "&mut right_dock.pane",
        "&mut bottom_dock.pane",
    ),
    "zircon_editor/src/ui/retained_host/app/native_windows/presentation.rs": (
        "ui.get_host_presentation_generation(",
        "ui.update_host_presentation(",
    ),
}


class SourceContractError(RuntimeError):
    """Raised when the pressure model no longer describes current source."""


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest().upper()


def _git_output(repo_root: Path, *args: str) -> str | None:
    try:
        completed = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout.strip()


def source_binding_report(repo_root: Path) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    critical_sources = []
    source_set = hashlib.sha256()
    relative_paths = []
    for relative_path, required_tokens in CRITICAL_SOURCE_CONTRACTS:
        path = repo_root / relative_path
        try:
            payload = path.read_bytes()
        except OSError as error:
            raise SourceContractError(f"missing critical source: {relative_path}") from error
        source = payload.decode("utf-8")
        missing = [token for token in required_tokens if token not in source]
        if missing:
            raise SourceContractError(
                f"critical source contract changed: {relative_path}: {missing}"
            )
        obsolete = [
            token
            for token in FORBIDDEN_SOURCE_TOKENS.get(relative_path, ())
            if token in source
        ]
        if obsolete:
            raise SourceContractError(
                f"obsolete source contract returned: {relative_path}: {obsolete}"
            )
        digest = _sha256(payload)
        critical_sources.append(
            {
                "relative_path": relative_path,
                "sha256": digest,
                "byte_length": len(payload),
            }
        )
        relative_paths.append(relative_path)
        source_set.update(relative_path.encode("utf-8"))
        source_set.update(b"\0")
        source_set.update(bytes.fromhex(digest))

    dirty = _git_output(repo_root, "status", "--short", "--", *relative_paths) or ""
    return {
        "ready": True,
        "git_revision": _git_output(repo_root, "rev-parse", "HEAD") or "unavailable",
        "critical_source_dirty": bool(dirty),
        "critical_source_dirty_entry_count": len(dirty.splitlines()) if dirty else 0,
        "source_set_sha256": source_set.hexdigest().upper(),
        "critical_sources": critical_sources,
    }


def run(
    *,
    pane_count: int,
    nodes_per_pane: int,
    stable_update_count: int,
    changed_update_count: int,
    changed_panes_per_update: int,
    geometry_update_count: int = 0,
    pointer_event_count: int = 100_000,
    popup_candidate_count: int = 4,
    toolbar_route_update_count: int = 1_000,
    floating_window_count: int = 16,
    native_window_update_count: int = 1_000,
) -> dict[str, Any]:
    if pane_count <= 0:
        raise ValueError("pane_count must be positive")
    if nodes_per_pane <= 0:
        raise ValueError("nodes_per_pane must be positive")
    if (
        stable_update_count < 0
        or changed_update_count < 0
        or geometry_update_count < 0
        or pointer_event_count < 0
        or toolbar_route_update_count < 0
        or floating_window_count < 0
        or native_window_update_count < 0
    ):
        raise ValueError("update counts must be non-negative")
    if not 0 <= changed_panes_per_update <= pane_count:
        raise ValueError("changed_panes_per_update must be within pane_count")
    if not 0 <= popup_candidate_count <= nodes_per_pane:
        raise ValueError("popup_candidate_count must be within nodes_per_pane")
    if changed_update_count > 0 and changed_panes_per_update == 0:
        raise ValueError("changed updates must identify at least one changed pane")

    baseline_initial_surface_build_count = pane_count
    baseline_stable_surface_build_count = pane_count * stable_update_count
    baseline_changed_surface_build_count = pane_count * changed_update_count
    baseline_total_surface_build_count = (
        baseline_initial_surface_build_count
        + baseline_stable_surface_build_count
        + baseline_changed_surface_build_count
    )

    retained_initial_surface_build_count = pane_count
    retained_stable_surface_build_count = 0
    retained_changed_surface_build_count = (
        changed_update_count * changed_panes_per_update
    )
    retained_total_surface_build_count = (
        retained_initial_surface_build_count
        + retained_changed_surface_build_count
    )

    work_per_surface = nodes_per_pane * PANE_PIPELINE_STAGE_COUNT
    baseline_node_stage_visit_count = baseline_total_surface_build_count * work_per_surface
    retained_node_stage_visit_count = retained_total_surface_build_count * work_per_surface
    stable_surface_build_avoidance_count = (
        baseline_stable_surface_build_count - retained_stable_surface_build_count
    )
    stable_surface_build_avoidance_percent = (
        100.0
        if baseline_stable_surface_build_count > 0
        else 0.0
    )
    geometry_semantic_pane_clone_count = (
        geometry_update_count * GEOMETRY_SEMANTIC_PANE_CLONE_SITES
    )
    whole_presentation_pane_clone_count = changed_update_count * pane_count
    paint_model_identity_visit_count = changed_update_count * pane_count
    hit_node_rebuild_visit_count = (
        changed_update_count * pane_count * nodes_per_pane
    )
    pane_owned_hit_node_rebuild_visit_count = (
        changed_update_count * changed_panes_per_update * nodes_per_pane
    )
    hit_node_visit_reduction_ratio = (
        hit_node_rebuild_visit_count / pane_owned_hit_node_rebuild_visit_count
        if pane_owned_hit_node_rebuild_visit_count > 0
        else 0.0
    )
    rejected_frame_only_pointer_node_visit_count = pointer_event_count * nodes_per_pane
    current_publication_index_node_visit_count = (
        changed_update_count * changed_panes_per_update * nodes_per_pane
    )
    current_pointer_candidate_visit_count = pointer_event_count * popup_candidate_count
    current_popup_hit_total_node_visit_count = (
        current_publication_index_node_visit_count
        + current_pointer_candidate_visit_count
    )
    popup_hit_node_visit_reduction_ratio = (
        rejected_frame_only_pointer_node_visit_count
        / current_popup_hit_total_node_visit_count
        if current_popup_hit_total_node_visit_count > 0
        else 0.0
    )
    toolbar_publication_count = stable_update_count + toolbar_route_update_count
    legacy_toolbar_paint_model_identity_visit_count = toolbar_publication_count * pane_count
    legacy_toolbar_floating_row_clone_count = toolbar_publication_count * floating_window_count
    current_toolbar_changed_pane_clone_count = (
        toolbar_route_update_count * changed_panes_per_update
    )
    legacy_native_presentation_cow_clone_count = native_window_update_count
    legacy_native_paint_model_identity_visit_count = native_window_update_count * pane_count

    return {
        "schema": "zircon.editor.pane_surface_retention_pressure.v8",
        "model_scope": "deterministic pane pipeline operation counts; not elapsed time or memory",
        "pane_pipeline_stage_count": PANE_PIPELINE_STAGE_COUNT,
        "pane_count": pane_count,
        "nodes_per_pane": nodes_per_pane,
        "stable_update_count": stable_update_count,
        "changed_update_count": changed_update_count,
        "changed_panes_per_update": changed_panes_per_update,
        "geometry_update_count": geometry_update_count,
        "pointer_event_count": pointer_event_count,
        "popup_candidate_count": popup_candidate_count,
        "toolbar_route_update_count": toolbar_route_update_count,
        "floating_window_count": floating_window_count,
        "native_window_update_count": native_window_update_count,
        "baseline_initial_surface_build_count": baseline_initial_surface_build_count,
        "baseline_stable_surface_build_count": baseline_stable_surface_build_count,
        "baseline_changed_surface_build_count": baseline_changed_surface_build_count,
        "baseline_total_surface_build_count": baseline_total_surface_build_count,
        "retained_initial_surface_build_count": retained_initial_surface_build_count,
        "retained_stable_surface_build_count": retained_stable_surface_build_count,
        "retained_changed_surface_build_count": retained_changed_surface_build_count,
        "retained_total_surface_build_count": retained_total_surface_build_count,
        "retained_unchanged_pane_rebuild_count": 0,
        "stable_surface_build_avoidance_count": stable_surface_build_avoidance_count,
        "stable_surface_build_avoidance_percent": stable_surface_build_avoidance_percent,
        "baseline_node_stage_visit_count": baseline_node_stage_visit_count,
        "retained_node_stage_visit_count": retained_node_stage_visit_count,
        "eliminated_node_stage_visit_count": (
            baseline_node_stage_visit_count - retained_node_stage_visit_count
        ),
        "node_stage_visit_reduction_ratio": (
            baseline_node_stage_visit_count / retained_node_stage_visit_count
        ),
        "rejected_dual_arc_aggregate_presentation": {
            "scenario": "each semantic patch begins from a coherent generation where host_structure shares host_presentation",
            "whole_presentation_cow_clone_count": changed_update_count,
            "whole_presentation_pane_clone_count": whole_presentation_pane_clone_count,
            "complexity": "O(U * P) aggregate COW before patch work",
        },
        "current_semantic_identity_presentation": {
            "scenario": "no outstanding external generation snapshot; semantic identity is the existing structure generation",
            "geometry_semantic_pane_clone_sites": GEOMETRY_SEMANTIC_PANE_CLONE_SITES,
            "geometry_semantic_pane_clone_count": geometry_semantic_pane_clone_count,
            "state_owned_presentation_arc_count": 1,
            "semantic_identity_lifetime_allocation_count": 0,
            "semantic_identity_update_allocation_count": 0,
            "semantic_generation_increment_count": changed_update_count,
            "internal_bookkeeping_cow_clone_count": 0,
            "whole_presentation_pane_clone_count": 0,
            "paint_model_identity_visit_count": paint_model_identity_visit_count,
            "hit_node_rebuild_visit_count": hit_node_rebuild_visit_count,
            "external_snapshot_requires_cow": True,
            "complexity": "O(G * fixed_pane_clone_sites + U * (P + P * N)); external snapshots retain required COW",
        },
        "pane_owned_presentation_transaction": {
            "implementation_scope": "current UI Asset scoped presentation path; target for remaining generic pane updates",
            "geometry_semantic_pane_clone_count": 0,
            "whole_presentation_cow_clone_count": 0,
            "whole_presentation_pane_clone_count": 0,
            "pane_handle_swap_count": changed_update_count * changed_panes_per_update,
            "paint_model_rebind_visit_count": paint_model_identity_visit_count,
            "hit_node_rebuild_visit_count": pane_owned_hit_node_rebuild_visit_count,
            "unchanged_pane_hit_node_visit_count": 0,
            "complexity": "O(U * (P + C * N)); persistent paint-index directories can later reduce P",
        },
        "current_scoped_ui_asset_pane_transaction": {
            "scenario": "no outstanding external generation snapshot; stable pane identity and one-to-one paint-model replacement",
            "pane_clone_count": changed_update_count * changed_panes_per_update,
            "whole_presentation_pane_clone_count": 0,
            "full_hit_index_build_count": 0,
            "paint_model_directory_visit_count": paint_model_identity_visit_count,
            "paint_model_rebind_count": changed_update_count
            * changed_panes_per_update,
            "hit_node_rebuild_visit_count": pane_owned_hit_node_rebuild_visit_count,
            "unchanged_pane_hit_node_visit_count": 0,
            "persistent_floating_row_clone_upper_bound_count": changed_update_count
            * changed_panes_per_update,
            "typed_fallbacks_excluded": "stale locator, changed pane id/kind, empty/nonempty model cardinality change, shared-model merge/split, or hit-index rebind rejection",
            "complexity": "O(U * (P + C * N)); no full presentation paint-model enumeration or all-pane hit-product rebuild",
        },
        "implemented_semantic_identity_delta": {
            "avoided_internal_whole_presentation_cow_clone_count": changed_update_count,
            "avoided_internal_whole_presentation_pane_clone_count": whole_presentation_pane_clone_count,
        },
        "pane_transaction_remaining_delta": {
            "scope": "generic update_host_presentation callers not yet migrated to the pane transaction",
            "avoided_geometry_semantic_pane_clone_count": geometry_semantic_pane_clone_count,
            "avoided_hit_node_visit_count": (
                hit_node_rebuild_visit_count - pane_owned_hit_node_rebuild_visit_count
            ),
            "hit_node_visit_reduction_ratio": hit_node_visit_reduction_ratio,
        },
        "implemented_scoped_ui_asset_transaction_delta": {
            "avoided_full_hit_index_build_count": changed_update_count,
            "avoided_unchanged_pane_hit_node_visit_count": (
                hit_node_rebuild_visit_count - pane_owned_hit_node_rebuild_visit_count
            ),
            "hit_node_visit_reduction_ratio": hit_node_visit_reduction_ratio,
        },
        "scoped_pane_popup_hit_index": {
            "rejected_frame_only_pointer_node_visit_count": (
                rejected_frame_only_pointer_node_visit_count
            ),
            "rejected_event_hot_path_complexity": "O(pointer events * pane nodes)",
            "current_publication_index_node_visit_count": (
                current_publication_index_node_visit_count
            ),
            "current_pointer_candidate_visit_count": (
                current_pointer_candidate_visit_count
            ),
            "current_total_node_visit_count": current_popup_hit_total_node_visit_count,
            "current_event_hot_path_complexity": "O(pointer events * popup candidates)",
            "node_visit_reduction_ratio": popup_hit_node_visit_reduction_ratio,
        },
        "viewport_toolbar_pane_transaction": {
            "stable_publication_count": stable_update_count,
            "route_update_count": toolbar_route_update_count,
            "legacy_paint_model_directory_materialization_count": toolbar_publication_count,
            "legacy_paint_model_identity_visit_count": legacy_toolbar_paint_model_identity_visit_count,
            "legacy_floating_row_clone_count": legacy_toolbar_floating_row_clone_count,
            "legacy_floating_model_materialization_count": toolbar_publication_count,
            "current_stable_pane_clone_count": 0,
            "current_stable_generation_increment_count": 0,
            "current_paint_model_directory_materialization_count": 0,
            "current_paint_model_identity_visit_count": 0,
            "current_route_changed_pane_clone_count": current_toolbar_changed_pane_clone_count,
            "current_route_generation_increment_count": toolbar_route_update_count,
            "current_unchanged_floating_row_clone_count": 0,
            "complexity": "stable O(D + F) identity probes with no publication; route delta O(D + F + C) and no paint-model scan",
        },
        "native_floating_presentation_transaction": {
            "scenario": "bounds-only updates after stable native window identity and title",
            "update_count": native_window_update_count,
            "legacy_presentation_cow_clone_count": legacy_native_presentation_cow_clone_count,
            "legacy_paint_model_identity_visit_count": legacy_native_paint_model_identity_visit_count,
            "legacy_workbench_hit_index_rebuild_count": 0,
            "legacy_structure_generation_increment_count": native_window_update_count,
            "legacy_geometry_generation_increment_count": native_window_update_count,
            "current_metadata_commit_count": native_window_update_count,
            "current_presentation_cow_clone_count": 0,
            "current_paint_model_identity_visit_count": 0,
            "current_workbench_hit_index_rebuild_count": 0,
            "current_structure_generation_increment_count": 0,
            "current_geometry_generation_increment_count": native_window_update_count,
            "stable_repeat_commit_count": 0,
            "complexity": "metadata delta O(1) after field equality gate; no paint-model or workbench-index scan",
        },
        "interpretation": {
            "is_product_timing": False,
            "included": "source-proven clone sites, rejected internal COW, current semantic identity, paint-model rebuild visits, and pane popup hit-index visits",
            "excluded": "elapsed CPU/GPU time, allocator bytes, Arc payload depth, lock contention, RSS, and input-to-present latency",
            "required_product_evidence": "presentation COW clones, pane handle swaps, hit-index full builds/rebinds/node visits, damage area, CPU/allocation/RSS, and input-to-present p50/p95/p99",
        },
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.casefold() == "c:":
        raise ValueError("profile artifacts must not be written to the C drive")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pane-count", type=int, default=64)
    parser.add_argument("--nodes-per-pane", type=int, default=2_048)
    parser.add_argument("--stable-update-count", type=int, default=1_000)
    parser.add_argument("--changed-update-count", type=int, default=1_000)
    parser.add_argument("--changed-panes-per-update", type=int, default=1)
    parser.add_argument("--geometry-update-count", type=int, default=600)
    parser.add_argument("--pointer-event-count", type=int, default=100_000)
    parser.add_argument("--popup-candidate-count", type=int, default=4)
    parser.add_argument("--toolbar-route-update-count", type=int, default=1_000)
    parser.add_argument("--floating-window-count", type=int, default=16)
    parser.add_argument("--native-window-update-count", type=int, default=1_000)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        pane_count=args.pane_count,
        nodes_per_pane=args.nodes_per_pane,
        stable_update_count=args.stable_update_count,
        changed_update_count=args.changed_update_count,
        changed_panes_per_update=args.changed_panes_per_update,
        geometry_update_count=args.geometry_update_count,
        pointer_event_count=args.pointer_event_count,
        popup_candidate_count=args.popup_candidate_count,
        toolbar_route_update_count=args.toolbar_route_update_count,
        floating_window_count=args.floating_window_count,
        native_window_update_count=args.native_window_update_count,
    )
    result["source_binding"] = source_binding_report(Path(__file__).resolve().parents[4])
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
