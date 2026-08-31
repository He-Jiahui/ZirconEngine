from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor03WorldSpaceInteractiveTransactionContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_editor03_owns_one_generation_checked_batch_session(self) -> None:
        module = self.read("zircon_editor/src/core/editing/interactive_transform/mod.rs")
        session = self.read("zircon_editor/src/core/editing/interactive_transform/session.rs")
        editing = self.read("zircon_editor/src/core/editing/mod.rs")

        self.assertIn("pub(crate) mod interactive_transform;", editing)
        self.assertIn("InteractiveTransformSession", module)
        self.assertIn("document: DocumentId", session)
        self.assertIn("scene.world_generation()", session)
        self.assertIn("PrimaryTargetMismatch", session)
        self.assertIn("scene.parent_of", session)
        self.assertIn("scene.world_matrix", session)
        self.assertIn("try_affine_inverse", session)
        self.assertIn("to_scale_rotation_translation", session)
        self.assertIn("recomposition_residual", session)
        self.assertNotIn("scene.nodes()", session)

    def test_viewport_requests_world_targets_and_workbench_has_no_private_gizmo_capture(self) -> None:
        feedback = self.read(
            "zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs"
        )
        controller = self.read(
            "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs"
        )
        state = self.read(
            "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs"
        )
        camera = self.read(
            "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_camera.rs"
        )
        selection = self.read(
            "zircon_editor/src/ui/workbench/state/editor_state_selection.rs"
        )
        intents = self.read(
            "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs"
        )
        camera_authority_tests = self.read(
            "zircon_editor/src/tests/editing/state/camera_authority.rs"
        )

        self.assertIn("ViewportTransformRequest", feedback)
        self.assertIn("target_world", feedback)
        self.assertIn(".world_transform(entity)", controller)
        self.assertIn("InteractiveTransformSession", state)
        self.assertIn("active_camera_transform_before", state)
        self.assertIn("resync_after_interactive_transform", state)
        self.assertIn("resync_active_camera_after_scene_mutation", camera)
        self.assertIn("scene.world_transform(active_camera)", camera)
        self.assertIn("scene.world_transform(primary_root)", camera)
        self.assertIn("orbit_controller", camera)
        self.assertIn("scene.world_transform(selected)?.translation", selection)
        self.assertIn("set_orbit_target(node.world_translation)", selection)
        self.assertIn("capture_active_scene_camera_authority", intents)
        self.assertIn("resync_active_scene_camera_after_mutation", intents)
        self.assertIn(
            "active_camera_transform_command_refreshes_the_editor_camera_snapshot",
            camera_authority_tests,
        )
        self.assertIn(
            "active_camera_parent_transform_undo_redo_keeps_the_editor_camera_authoritative",
            camera_authority_tests,
        )
        self.assertIn(
            "unrelated_transform_command_preserves_a_navigated_editor_camera",
            camera_authority_tests,
        )
        self.assertNotIn("GizmoTransactionCapture", state)
        production = state.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("scene.update_transform", production)

    def test_release_commits_one_versioned_batch_command(self) -> None:
        command = self.read("zircon_editor/src/core/editing/command.rs")
        batch = self.read(
            "zircon_editor/src/core/editing/command/batch_transform.rs"
        )
        codecs = self.read(
            "zircon_editor/src/core/editing/journal_codecs/scene.rs"
        )

        self.assertIn("BatchTransform", command)
        self.assertIn("applied_transform_batch", command)
        self.assertIn("BatchTransformCommand", batch)
        self.assertIn("expected_applied_world_generation", batch)
        self.assertIn("validate_applied_targets", batch)
        self.assertIn("zircon.editor.scene.batch_transform", batch)
        self.assertIn("BatchTransformCodec", codecs)


if __name__ == "__main__":
    unittest.main()
