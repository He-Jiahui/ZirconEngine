import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorRetainedWorkbenchContributionProjectionContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_controller_projects_real_contributions_and_capabilities(self) -> None:
        projection = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/"
            "workbench_projection.rs"
        )

        self.assertIn("inner.contributions.snapshot()", projection)
        self.assertIn(".manager", projection)
        self.assertIn(".capability_snapshot()", projection)
        self.assertIn("collect::<CapabilitySet>()", projection)
        self.assertIn(
            "WorkbenchViewModel::build_with_contributions_and_context", projection
        )
        self.assertNotIn("ContributionSnapshot::default()", projection)
        self.assertNotIn("CapabilitySet::default()", projection)

    def test_full_retained_product_projections_use_the_controller_projection(self) -> None:
        product_paths = (
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/"
            "shell/builder.rs",
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
            "recompute_viewport.rs",
        )

        for relative in product_paths:
            source = self.read(relative)
            with self.subTest(path=relative):
                self.assertIn("build_workbench_view_model", source)
                self.assertNotIn("WorkbenchViewModel::build_with_context", source)

    def test_focus_dispatch_uses_the_authoritative_layout_query(self) -> None:
        source = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/layout/"
            "floating_window/dispatch.rs"
        )

        self.assertIn("runtime.floating_window_focus_target(window_id)", source)
        self.assertNotIn("build_workbench_view_model", source)
        self.assertNotIn("chrome_snapshot", source)


if __name__ == "__main__":
    unittest.main()
