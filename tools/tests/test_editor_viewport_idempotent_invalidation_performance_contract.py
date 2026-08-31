from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
FEEDBACK = ROOT / "zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs"
APPLY_COMMAND = (
    ROOT
    / "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_apply_command.rs"
)
ACCESSORS = (
    ROOT
    / "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs"
)
VIEWPORT_EVENT = ROOT / "zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs"
PRODUCT_TEST = (
    ROOT
    / "zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs"
)


class ViewportIdempotentInvalidationContract(unittest.TestCase):
    def test_feedback_carries_authoritative_settings_change(self) -> None:
        source = FEEDBACK.read_text(encoding="utf-8")
        self.assertIn("pub(crate) settings_changed: bool", source)

    def test_controller_compares_local_settings_before_writing(self) -> None:
        source = APPLY_COMMAND.read_text(encoding="utf-8")
        self.assertIn("replace_if_changed", source)
        self.assertIn("feedback.settings_changed", source)
        self.assertNotIn("self.state.settings.grid_mode = *mode", source)
        self.assertNotIn("self.state.settings.display_mode = *mode", source)

    def test_snap_authority_reports_whether_it_changed(self) -> None:
        source = ACCESSORS.read_text(encoding="utf-8")
        region = source.split("fn set_project_snap_step", 1)[1].split(
            "fn reset_project_settings", 1
        )[0]
        self.assertIn("Result<bool, SceneViewportControllerError>", region)
        self.assertIn("let receipt = self.settings_mutations.set(", region)
        self.assertIn("Ok(receipt.changed())", region)

    def test_event_invalidation_consumes_feedback_instead_of_assuming_setters_change(self) -> None:
        source = VIEWPORT_EVENT.read_text(encoding="utf-8")
        self.assertIn("feedback.settings_changed", source)
        structural = source.split("fn structural_viewport_event", 1)[1].split(
            "fn viewport_effects", 1
        )[0]
        for setter in (
            "SetTransformSpace",
            "SetProjectionMode",
            "SetDisplayMode",
            "SetGridMode",
            "SetTranslateSnap",
            "SetRotateSnapDegrees",
            "SetScaleSnap",
            "SetPreviewLighting",
            "SetPreviewSkybox",
            "SetGizmosEnabled",
        ):
            self.assertNotIn(setter, structural)

    def test_product_regression_covers_repeated_viewport_setting(self) -> None:
        source = PRODUCT_TEST.read_text(encoding="utf-8")
        self.assertIn("fn repeated_viewport_setting_is_an_invalidation_noop", source)
        self.assertIn("assert!(!repeated.presentation_dirty)", source)
        self.assertIn("assert!(!repeated.render_dirty)", source)


if __name__ == "__main__":
    unittest.main()
