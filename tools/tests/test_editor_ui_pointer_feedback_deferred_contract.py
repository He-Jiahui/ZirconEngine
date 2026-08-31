from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER_FEEDBACK = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/"
    "pointer_feedback.rs"
)
TOOLTIP = ROOT / "zircon_editor/src/ui/retained_host/app/workbench_tooltip.rs"
RECOMPUTE = ROOT / "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs"
DISPATCH_EFFECTS = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs"
)


class EditorUiPointerFeedbackDeferredContractTests(unittest.TestCase):
    def test_callback_defers_surface_rebuild_and_marks_projection(self):
        feedback = POINTER_FEEDBACK.read_text(encoding="utf-8")
        tooltip = TOOLTIP.read_text(encoding="utf-8")
        self.assertIn("feedback_deferred_count", feedback)
        self.assertNotIn("refresh_after_state_change(self.runtime.as_ref())", feedback)
        self.assertIn("effects.request_workbench_projection();", tooltip)

    def test_frame_recompute_refreshes_before_projection_patch(self):
        recompute = RECOMPUTE.read_text(encoding="utf-8")
        decision = recompute.index("let shell_content_scope")
        refresh = recompute.index("refresh_prepared_state_change()")
        target = recompute.index("RecomputeInvalidationTarget::WorkbenchProjection")
        patch = recompute.index("apply_workbench_projection_presentation()")
        self.assertIn("has_pending_surface_state_change()", recompute[decision:target])
        self.assertLess(refresh, patch)

    def test_pending_surface_state_promotes_projection_even_before_host_patch(self):
        effects = DISPATCH_EFFECTS.read_text(encoding="utf-8")
        self.assertIn("has_pending_surface_state_change()", effects)
        self.assertIn("HostInvalidationMask::WORKBENCH_PROJECTION", effects)


if __name__ == "__main__":
    unittest.main()
