from __future__ import annotations
from tools.tests.rust_test_files import read_rust_test_file

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def production(source: str) -> str:
    # Several production impls contain small `#[cfg(test)]` helpers before their later
    # production methods.  Only remove the trailing test module, never truncate the owner at
    # the first helper.
    return source.split("#[cfg(test)]\nmod tests", 1)[0]


class RuntimeRenderSceneReplayContractTests(unittest.TestCase):
    """Static guard for bounded, revision-aware RenderScene geometry replay."""

    def test_resource_replay_is_revision_aware_bounded_and_reconnect_safe(self) -> None:
        source = read(
            "zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs"
        )
        body = production(source)

        for anchor in (
            "pending_revisions: HashMap<UntypedResourceHandle, u64>",
            "applied_revisions: HashMap<UntypedResourceHandle, u64>",
            "reconnect_required: bool",
            "for _ in 0..limit",
            "ResourceEventTryRecvError::Lagged(_)",
            "ResourceEventTryRecvError::SequenceExhausted",
            "self.reconnect_required = true",
            "self.pending_revisions.insert(resource, event.revision)",
            "*revision >= event.revision",
            "if !replay_complete || self.reconnect_required",
        ):
            self.assertIn(anchor, body)

        self.assertIn("ResourceKind::Mesh | ResourceKind::Model", body)
        self.assertIn("ResourceEventKind::Added", body)
        self.assertIn("ResourceEventKind::Updated", body)
        self.assertIn("ResourceEventKind::Removed", body)
        self.assertIn("ResourceEventKind::ReloadFailed", body)

    def test_frame_admission_consumes_authoritative_cursor_and_stages_before_commit(self) -> None:
        source = read(
            "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs"
        )
        body = production(source)

        for anchor in (
            "const MAX_GEOMETRY_REPLAY_PRIMITIVES: usize = 256",
            "projector.is_exact_frame_replay(frame)?",
            "self.asset_manager()?.resource_manager().subscribe()",
            ".drain(MAX_GEOMETRY_REPLAY_PRIMITIVES",
            "prepare_geometry_replay_with_revisions(",
            "ensure_render_scene_projection_geometry_resources(",
            "project_frame_with_resource_geometry_staging(",
            "finish_geometry_replay_selection(&selection)",
            "retain_dependencies(|resource| projector.has_geometry_dependency(resource))",
        ):
            self.assertIn(anchor, body)

        exact = body.index("let exact_replay = projector.is_exact_frame_replay(frame)?")
        drain = body.index(".drain(MAX_GEOMETRY_REPLAY_PRIMITIVES")
        selection = body.index("prepare_geometry_replay_with_revisions(")
        staging = body.index("project_frame_with_resource_geometry_staging(")
        finish = body.index("finish_geometry_replay_selection(&selection)")
        self.assertLess(exact, drain)
        self.assertLess(drain, selection)
        self.assertLess(selection, staging)
        self.assertLess(staging, finish)
        self.assertNotIn("frame.meshes()", body)

    def test_projector_indexes_resource_dependencies_and_continues_only_current_revisions(self) -> None:
        source = read(
            "zircon_runtime/src/graphics/scene/render_scene/component_projector/projector.rs"
        )
        body = production(source)

        for anchor in (
            "geometry_primitive_dependencies: HashMap<u64, HashSet<UntypedResourceHandle>>",
            "geometry_primitive_keys: BTreeSet<u64>",
            "geometry_replay_revisions: HashMap<UntypedResourceHandle, u64>",
            "pub(crate) fn prepare_geometry_replay_with_revisions(",
            "let revision_continuation_is_current =",
            "self.geometry_replay_revisions.eq(resource_revisions)",
            "let truncated = !resync && target_count_before_limit > limit.max(1)",
            "apply_geometry_dependency_journal(&journal, resolver)",
            "stable_instance_key()",
            "required.extend(geometry_dependencies(primitive))",
        ):
            self.assertIn(anchor, body)

        revision_check = body.index("let revision_continuation_is_current =")
        truncation = body.index("let truncated = !resync")
        dependency_journal = body.index("fn apply_geometry_dependency_journal(")
        self.assertLess(revision_check, truncation)
        self.assertLess(dependency_journal, body.index("fn add_geometry_dependencies("))

    def test_regressions_cover_stale_geometry_atomicity_budget_and_stable_frames(self) -> None:
        replay_tests = read(
            "zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs"
        )
        projector_tests = read(
            "zircon_runtime/src/graphics/scene/render_scene/component_projector/tests/cases.rs"
        )
        residency_tests = read(
            "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_residency.rs"
        )

        for name in (
            "render_scene_resource_geometry_replay_disconnect_requires_resync_and_reconnect",
            "render_scene_resource_geometry_replay_same_revision_is_noop_after_commit",
            "render_scene_resource_geometry_replay_failure_keeps_pending_revision",
            "render_scene_resource_geometry_replay_keeps_latest_revision_until_final_chunk_commits",
            "render_scene_resource_geometry_replay_updates_shared_model_without_component_change",
            "render_scene_resource_geometry_replay_resync_is_bounded_and_continues",
            "render_scene_resource_geometry_replay_continues_dependent_targets_after_budget",
            "render_scene_resource_geometry_replay_restarts_continuation_for_new_revision",
            "render_scene_resource_geometry_replay_stable_frame_without_invalidations_resolves_nothing",
        ):
            self.assertIn(f"fn {name}", (read_rust_test_file("zircon_runtime/src/graphics/scene/resources/resource_streamer/tests/geometry_replay.rs") if f"fn {name}" in {"fn render_scene_resource_geometry_replay_continues_dependent_targets_after_budget", "fn render_scene_resource_geometry_replay_disconnect_requires_resync_and_reconnect", "fn render_scene_resource_geometry_replay_failure_keeps_pending_revision", "fn render_scene_resource_geometry_replay_keeps_latest_revision_until_final_chunk_commits", "fn render_scene_resource_geometry_replay_restarts_continuation_for_new_revision", "fn render_scene_resource_geometry_replay_resync_is_bounded_and_continues", "fn render_scene_resource_geometry_replay_same_revision_is_noop_after_commit", "fn render_scene_resource_geometry_replay_stable_frame_without_invalidations_resolves_nothing", "fn render_scene_resource_geometry_replay_updates_shared_model_without_component_change"} else replay_tests))

        for name in (
            "render_scene_component_projector_keeps_batch_atomic_while_geometry_is_pending",
            "render_scene_component_projector_retries_same_artifact_after_staging_rejection",
            "render_scene_component_projector_full_resync_removes_absent_primitive",
        ):
            self.assertIn(f"fn {name}", projector_tests)

        self.assertIn(
            "render_scene_admission_reconnects_geometry_replay_and_processes_mesh_update",
            read_rust_test_file("zircon_runtime/src/graphics/scene/resources/resource_streamer/tests/resource_streamer_residency.rs"),
        )
        self.assertIn("production_residency_bridge_requires_authoritative_delta_inputs", read_rust_test_file("zircon_runtime/src/graphics/scene/resources/resource_streamer/tests/resource_streamer_residency.rs"))


if __name__ == "__main__":
    unittest.main()
