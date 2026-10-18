import unittest
from pathlib import Path


class RuntimeRgA3GeometryReplayContractTests(unittest.TestCase):
    """Static guard for the bounded, revision-aware RenderScene replay boundary.

    The production implementation is currently waiting on the managed native gate.  Keep the
    ownership and transaction invariants executable in the repository's Python audit layer so a
    later refactor cannot silently discard revisions, acknowledge a partial chunk, or broaden
    supplemental event retention to unrelated resource kinds.
    """

    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.replay = (
            self.repo_root
            / "zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs"
        )
        self.residency = (
            self.repo_root
            / "zircon_runtime/src/graphics/scene/resources/resource_streamer/"
            "resource_streamer_residency.rs"
        )
        self.projector = (
            self.repo_root
            / "zircon_runtime/src/graphics/scene/render_scene/component_projector/projector.rs"
        )

    @staticmethod
    def _production(source: str) -> str:
        # Some owners expose small `#[cfg(test)]` helpers inside production impls.  Cut at the
        # dedicated test module instead of the first attribute so those helpers cannot hide the
        # replay fields or continuation methods we are auditing.
        return source.split("#[cfg(test)]\nmod tests", 1)[0]

    def test_event_cursor_filters_revisions_and_preserves_incomplete_work(self) -> None:
        source = self.replay.read_text(encoding="utf-8")
        production = self._production(source)

        for anchor in (
            "pending_revisions: HashMap<UntypedResourceHandle, u64>",
            "applied_revisions: HashMap<UntypedResourceHandle, u64>",
            "pub(super) fn drain(",
            "if limit == 0",
            "self.reconnect_required = true",
            "self.require_resync();",
            "pub(super) fn commit(&mut self, resync_complete: bool, replay_complete: bool)",
            "pub(super) fn retain_dependencies(",
            ".extend(self.pending_revisions.drain())",
        ):
            self.assertIn(anchor, production)

        self.assertLess(production.index("if limit == 0"), production.index("for _ in 0..limit"))
        commit_guard = production.index("if !replay_complete")
        commit_apply = production.index(".extend(self.pending_revisions.drain())", commit_guard)
        self.assertLess(commit_guard, commit_apply)
        self.assertIn("ResourceKind::Mesh | ResourceKind::Model", production)
        self.assertIn("ResourceEventKind::Added", production)
        self.assertIn("ResourceEventKind::Updated", production)
        self.assertIn("ResourceEventKind::Removed", production)
        self.assertIn("ResourceEventKind::ReloadFailed", production)
        self.assertIn(".is_some_and(|revision| *revision >= event.revision)", production)

    def test_residency_bridge_reconnects_and_commits_only_complete_bounded_chunks(self) -> None:
        source = self.residency.read_text(encoding="utf-8")
        production = self._production(source)

        for anchor in (
            "const MAX_GEOMETRY_REPLAY_PRIMITIVES: usize = 256;",
            "RenderSceneGeometryReplayState::new(Some(receiver))",
            "receiver_missing)",
            ".install_receiver(receiver);",
            ".drain(MAX_GEOMETRY_REPLAY_PRIMITIVES",
            "prepare_geometry_replay_with_revisions(",
            "finish_geometry_replay_selection(&selection);",
            ".commit(projector.geometry_resync_complete(), !selection.truncated());",
            ".retain_dependencies(|resource| projector.has_geometry_dependency(resource));",
        ):
            self.assertIn(anchor, production)

        classify = production.index("let artifact_may_reference_supplemental_geometry")
        drain = production.index(".drain(MAX_GEOMETRY_REPLAY_PRIMITIVES", classify)
        prepare = production.index("prepare_geometry_replay_with_revisions(", drain)
        finish = production.index("finish_geometry_replay_selection(&selection);", prepare)
        commit = production.index(".commit(projector.geometry_resync_complete()", finish)
        retain = production.index(".retain_dependencies(|resource|", commit)
        self.assertLess(classify, drain)
        self.assertLess(drain, prepare)
        self.assertLess(prepare, finish)
        self.assertLess(finish, commit)
        self.assertLess(commit, retain)
        self.assertIn(
            "matches!(resource.kind(), ResourceKind::Mesh | ResourceKind::Model)",
            production,
        )
        self.assertNotIn(".map(|(resource, _)| *resource)", production)

    def test_projector_uses_revision_scope_to_restart_stable_key_continuations(self) -> None:
        source = self.projector.read_text(encoding="utf-8")
        production = self._production(source)

        for anchor in (
            "geometry_replay_after: Option<u64>",
            "geometry_replay_scope: HashSet<UntypedResourceHandle>",
            "geometry_replay_revisions: HashMap<UntypedResourceHandle, u64>",
            "pub(crate) fn prepare_geometry_replay_with_revisions(",
            "pub(crate) fn finish_geometry_replay_selection(",
            "let revision_continuation_is_current =",
            "self.geometry_replay_scope == resource_scope",
            "targets.retain(|key| *key > after);",
            "targets.truncate(limit.max(1));",
        ):
            self.assertIn(anchor, production)

        finish = production[production.index("pub(crate) fn finish_geometry_replay_selection(") :]
        self.assertIn("if selection.resync", finish)
        self.assertIn("if selection.truncated && !selection.component_geometry_changed", finish)
        self.assertIn("self.geometry_replay_revisions = selection.resource_revisions.clone();", finish)
        self.assertIn("self.geometry_replay_revisions.clear();", finish)

        continuation = production[production.index("let revision_continuation_is_current =") :]
        self.assertLess(
            continuation.index("self.geometry_replay_scope == resource_scope"),
            continuation.index("targets.retain(|key| *key > after);"),
        )
        self.assertIn("target_count_before_limit > limit.max(1)", continuation)

    def test_regressions_cover_reconnect_same_frame_dependencies_and_revision_restart(self) -> None:
        replay = self.replay.read_text(encoding="utf-8")
        residency = self.residency.read_text(encoding="utf-8")

        for test_name in (
            "fn render_scene_resource_geometry_replay_restarts_continuation_for_new_revision()",
            "fn render_scene_resource_geometry_replay_keeps_same_frame_declared_and_supplemental_dependency_events(",
            "fn render_scene_resource_geometry_replay_continues_dependent_targets_after_budget()",
            "fn render_scene_resource_geometry_replay_failure_keeps_pending_revision()",
            "fn render_scene_resource_geometry_replay_same_revision_is_noop_after_commit()",
        ):
            self.assertIn(test_name, replay)
        self.assertIn("(0..300).map(|_| renderer.clone())", replay)
        self.assertIn("assert_eq!(second.target_keys(), first.target_keys());", replay)
        self.assertIn("assert_eq!(final_chunk.selected_primitive_count(), 44);", replay)
        self.assertIn("replay_state.commit(true, !final_chunk.truncated());", replay)
        self.assertIn(
            "fn render_scene_admission_reconnects_geometry_replay_and_processes_mesh_update()",
            residency,
        )
        self.assertIn("stale_manager.subscribe()", residency)
        self.assertIn("pre-reconnect mesh update must be recovered by bounded resync", residency)


if __name__ == "__main__":
    unittest.main()
