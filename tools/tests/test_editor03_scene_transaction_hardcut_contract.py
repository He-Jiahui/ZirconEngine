from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def section(text: str, start: str, end: str) -> str:
    return text.split(start, 1)[1].split(end, 1)[0]


class Editor03SceneTransactionHardcutTests(unittest.TestCase):
    def test_scene_commands_are_pure_transaction_commands(self) -> None:
        command = source("zircon_editor/src/core/editing/command.rs")

        for required in [
            "impl EditCommand for EditorCommand",
            "fn apply(&mut self, context: &mut dyn EditContext)",
            "fn revert(&mut self, context: &mut dyn EditContext)",
            "CreateNodeIntent",
            "already_applied",
        ]:
            self.assertIn(required, command)
        for removed in [
            "BatchEditorCommand",
            "Self::Batch",
            "selection_before:",
            "selection_after:",
            "previous_selected:",
        ]:
            self.assertNotIn(removed, command)

    def test_editor_state_uses_the_context_transaction_engine(self) -> None:
        state = source("zircon_editor/src/ui/workbench/state/editor_state.rs")
        construction = source(
            "zircon_editor/src/ui/workbench/startup/editor_state_construction.rs"
        )
        intents = source(
            "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs"
        )

        self.assertIn("context: Arc<EditorContext>", state)
        self.assertNotIn("EditorHistory", state)
        for required in [
            "HistoryContextId::Global",
            ".transactions()",
            ".begin(",
            "scope.push(",
            ".commit_after_apply(",
            "scene editing is disabled during play mode",
        ]:
            self.assertIn(required, intents)
        self.assertNotIn("self.history", intents)
        self.assertNotIn("scope.commit(", intents)
        self.assertIn("pub fn with_default_selection_with_context", construction)

    def test_cli_operation_injects_the_manager_context(self) -> None:
        app = source("zircon_app/src/entry/entry_runner/editor.rs")

        self.assertIn("EditorState::with_default_selection_with_context(", app)
        self.assertIn("manager.context().clone()", app)
        self.assertNotIn("EditorState::with_default_selection(\n", app)

    def test_old_scene_history_owner_is_physically_removed(self) -> None:
        self.assertFalse(
            (ROOT / "zircon_editor/src/core/editing/history.rs").exists(),
            "the old scene-only undo stack must be deleted, not wrapped",
        )
        editing_mod = source("zircon_editor/src/core/editing/mod.rs")
        self.assertNotIn("mod history", editing_mod)
        production_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (ROOT / "zircon_editor/src").rglob("*.rs")
            if "tests" not in path.parts
        )
        self.assertNotIn("EditorHistory", production_sources)
        self.assertIn("DelegatedToTransactionEngine", production_sources)

    def test_scene_context_and_pure_capture_regressions_are_present(self) -> None:
        context = source("zircon_editor/src/core/editing/context.rs")
        viewport = source(
            "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs"
        )
        tests = source("zircon_editor/src/tests/editing/history.rs")

        self.assertIn("gateway: EditorRuntimeGatewayHandle", context)
        self.assertIn("bind_scene", context)
        self.assertIn(".with_world(&mut", context)
        self.assertIn(".with_world_mut(&mut", context)
        self.assertNotIn("scene: Option<LevelSystem>", context)
        self.assertIn("capture_does_not_mutate", tests)
        self.assertIn("transaction_history", tests)
        self.assertIn("initial: Transform", viewport)
        self.assertIn("latest: Transform", viewport)
        self.assertNotIn("commands: Vec<EditorCommand>", viewport)
        self.assertIn("record_gizmo_transaction_step", viewport)
        self.assertIn("one_hundred_frames", tests)

    def test_gizmo_transaction_capture_has_a_matching_workbench_boundary(self) -> None:
        state = source("zircon_editor/src/ui/workbench/state/editor_state.rs")
        viewport = source(
            "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs"
        )

        self.assertIn(
            "pub(in crate::ui::workbench) gizmo_transaction: "
            "Option<GizmoTransactionCapture>",
            state,
        )
        self.assertIn(
            "pub(in crate::ui::workbench) struct GizmoTransactionCapture",
            viewport,
        )
        self.assertNotIn(
            "pub(crate) gizmo_transaction: Option<GizmoTransactionCapture>", state
        )
        self.assertNotIn("pub(super) struct GizmoTransactionCapture", viewport)

    def test_non_selection_commands_preserve_the_bound_selection(self) -> None:
        command = source("zircon_editor/src/core/editing/command.rs")

        update = section(command, "impl UpdateNodeCommand", "fn apply_node_state")
        reflected = section(
            command,
            "impl SetReflectedSceneFieldCommand",
            "fn ensure_reflected_field_editable",
        )
        self.assertNotIn("set_scene_selection", update)
        self.assertNotIn("set_scene_selection", reflected)

        state_tests = source("zircon_editor/src/tests/editing/state/selection.rs")
        reflected_tests = source("zircon_editor/src/tests/editing/reflected_command.rs")
        self.assertIn("non_selection_edit_preserves_active_multi_selection", state_tests)
        self.assertIn("reflected_edit_preserves_active_multi_selection", reflected_tests)

    def test_shared_scene_executor_rejects_play_mode_mutation(self) -> None:
        intents = source(
            "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs"
        )
        executor = section(
            intents,
            "pub(crate) fn execute_scene_commands",
            "pub(crate) fn capture_scene_command",
        )
        self.assertIn("self.is_playing()", executor)
        self.assertIn("disabled during play mode", executor)

        state_tests = source("zircon_editor/src/tests/editing/state/play_mode.rs")
        self.assertIn(
            "import_is_rejected_during_play_without_poisoning_edit_history",
            state_tests,
        )

    def test_viewport_is_the_only_fallible_gizmo_transaction_owner(self) -> None:
        viewport = source(
            "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs"
        )
        host = source(
            "zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs"
        )
        binding = source("zircon_editor/src/ui/binding_dispatch/viewport/apply.rs")

        self.assertIn("-> Result<ViewportFeedback, String>", viewport)
        self.assertIn("rollback_gizmo_transaction", viewport)
        for swallowed in [
            "let _ = self.begin_gizmo_transaction()",
            "let _ = self.record_gizmo_transaction_step()",
            "let _ = self.finish_gizmo_transaction()",
        ]:
            self.assertNotIn(swallowed, viewport)
        for duplicate_intent in [
            "EditorIntent::BeginGizmoDrag",
            "EditorIntent::DragGizmo",
            "EditorIntent::EndGizmoDrag",
        ]:
            self.assertNotIn(duplicate_intent, host)
        self.assertIn(".map_err(EditorBindingDispatchError::StateMutation)", binding)

        state_tests = source("zircon_editor/src/tests/editing/state/viewport.rs")
        self.assertIn("gizmo_transaction_failure_restores_transform", state_tests)

    def test_duplicate_gizmo_lifecycle_state_is_physically_removed(self) -> None:
        self.assertFalse(
            (
                ROOT
                / "zircon_editor/src/scene/viewport/interaction/gizmo_drag_state.rs"
            ).exists()
        )
        intent = source("zircon_editor/src/core/editing/intent.rs")
        controller = source(
            "zircon_editor/src/ui/host/editor_host_event_controller.rs"
        )
        runtime_access = "\n".join(
            source(path)
            for path in [
                "zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs",
                "zircon_editor/src/ui/host/editor_event_runtime_access/component_dispatch.rs",
                "zircon_editor/src/ui/host/editor_event_runtime_access/event_dispatch.rs",
                "zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs",
                "zircon_editor/src/ui/host/editor_event_runtime_access/input_dispatch.rs",
                "zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs",
                "zircon_editor/src/ui/host/editor_event_runtime_access/status.rs",
            ]
        )
        interaction = source("zircon_editor/src/scene/viewport/interaction/mod.rs")
        viewport = source("zircon_editor/src/scene/viewport/mod.rs")

        for removed in [
            "BeginGizmoDrag",
            "DragGizmo",
            "EndGizmoDrag",
            "GizmoDragState",
            "gizmo_drag",
        ]:
            self.assertNotIn(removed, intent)
            self.assertNotIn(removed, controller)
            self.assertNotIn(removed, runtime_access)
            self.assertNotIn(removed, interaction)
            self.assertNotIn(removed, viewport)

        history_tests = source("zircon_editor/src/tests/editing/history.rs")
        self.assertIn("state.begin_gizmo_transaction()", history_tests)
        self.assertIn("state.record_gizmo_transaction_step()", history_tests)
        self.assertIn("state.finish_gizmo_transaction()", history_tests)

    def test_gizmo_lifecycle_is_atomic_across_project_play_and_host_errors(self) -> None:
        project = source(
            "zircon_editor/src/ui/workbench/startup/editor_state_project.rs"
        )
        play = source(
            "zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs"
        )
        dispatch = source("zircon_editor/src/ui/host/editor_event_dispatch.rs")
        viewport_tests = source("zircon_editor/src/tests/editing/state/viewport.rs")
        play_tests = source("zircon_editor/src/tests/editing/state/play_mode.rs")

        self.assertGreaterEqual(project.count("with_exclusive_scene_transition("), 2)
        self.assertIn("pub(crate) fn with_exclusive_scene_transition", project)
        self.assertIn(".begin_exclusive_transition(operation)", project)
        self.assertGreaterEqual(project.count("clear_history_and_context"), 2)
        self.assertIn("self.cancel_gizmo_transaction()?", project)
        self.assertNotIn(".clear_history(HistoryContextId::Global)", project)
        self.assertGreaterEqual(play.count("with_exclusive_scene_transition("), 2)
        self.assertIn("failure_effects_for_event(&event)", dispatch)
        self.assertIn("EditorEvent::Viewport(_)", dispatch)
        self.assertIn("EditorEventEffect::RenderChanged", dispatch)
        for regression in [
            "replacing_world_cancels_the_old_gizmo_capture",
            "faulted_transaction_engine_blocks_world_replacement",
            "faulted_transaction_engine_blocks_project_clear",
        ]:
            self.assertIn(regression, viewport_tests)
        self.assertIn(
            "entering_play_during_gizmo_drag_restores_the_edit_transform",
            play_tests,
        )

    def test_gizmo_and_world_transitions_are_exclusive(self) -> None:
        transaction = source(
            "zircon_editor/src/core/editing/engine/transaction.rs"
        )
        exclusive_transition = source(
            "zircon_editor/src/core/editing/engine/transaction/exclusive_transition.rs"
        )
        intents = source(
            "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs"
        )
        viewport = source(
            "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs"
        )
        viewport_tests = source(
            "zircon_editor/src/tests/editing/state/viewport.rs"
        )
        locking_tests = source(
            "zircon_editor/src/tests/editing/transaction_engine/locking.rs"
        )

        self.assertIn("begin_exclusive_transition", transaction)
        self.assertIn("clear_history_and_context", exclusive_transition)
        self.assertIn("prepare_non_gizmo_scene_action", intents)
        self.assertIn("execute_gizmo_scene_command", viewport)
        for regression in [
            "ordinary_scene_edit_cancels_gizmo_before_command_capture",
            "deleting_during_drag_cancels_preview_before_command_capture",
            "missing_drag_target_release_cleans_gizmo_lifecycle",
        ]:
            self.assertIn(regression, viewport_tests)
        self.assertIn(
            "exclusive_transition_blocks_interleaved_engine_operations",
            locking_tests,
        )

    def test_editor_state_regressions_are_split_by_responsibility(self) -> None:
        root = source("zircon_editor/src/tests/editing/state.rs")
        self.assertIn("mod selection;", root)
        self.assertIn("mod play_mode;", root)
        self.assertIn("mod viewport;", root)
        for relative in [
            "zircon_editor/src/tests/editing/state.rs",
            "zircon_editor/src/tests/editing/state/selection.rs",
            "zircon_editor/src/tests/editing/state/play_mode.rs",
            "zircon_editor/src/tests/editing/state/viewport.rs",
        ]:
            line_count = len(source(relative).splitlines())
            self.assertLess(line_count, 900, f"{relative} has {line_count} lines")


if __name__ == "__main__":
    unittest.main()
