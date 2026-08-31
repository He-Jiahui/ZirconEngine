from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
PANE_PROJECTION = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs"
)


class EditorPaneVariantProjectionPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = PANE_PROJECTION.read_text(encoding="utf-8")

    def test_pane_projection_uses_one_kind_selected_native_body_builder(self) -> None:
        self.assertIn("native_body: build_native_body(", self.source)
        self.assertRegex(
            self.source,
            re.compile(
                r"fn build_native_body\([^)]*kind: ViewContentKind,.*?"
                r"let mut native_body = PaneNativeBodyData::default\(\);.*?"
                r"match kind \{.*?native_body\n\}",
                re.DOTALL,
            ),
        )

    def test_heavy_payload_builders_are_confined_to_matching_kind_arms(self) -> None:
        builder_start = self.source.index("fn build_native_body(")
        builder_end = self.source.index("\nfn ", builder_start + 1)
        builder = self.source[builder_start:builder_end]

        expected_arms = {
            "ViewContentKind::Hierarchy": "hierarchy_pane_data(chrome)",
            "ViewContentKind::Inspector": "inspector_pane_data(chrome, info)",
            "ViewContentKind::Console": "console_pane_data(chrome)",
            "ViewContentKind::PerformanceTimeline": (
                "performance_timeline_pane_data(pane_presentation)"
            ),
            "ViewContentKind::ModulePlugins": "module_plugins.clone()",
            "ViewContentKind::BuildExport": "build_export.clone()",
            "ViewContentKind::GeneratedBottom": (
                "generated_bottom_pane_data(pane_presentation)"
            ),
        }
        for kind, expression in expected_arms.items():
            with self.subTest(kind=kind):
                self.assertIn(kind, builder)
                self.assertEqual(builder.count(expression), 1)

    def test_animation_and_ui_asset_are_not_unconditionally_cloned(self) -> None:
        projection_start = self.source.index(
            "pub(super) fn pane_from_tab_with_template_v2_data("
        )
        projection_end = self.source.index("\npub(crate) fn find_tab_snapshot", projection_start)
        projection = self.source[projection_start:projection_end]

        self.assertNotIn("let ui_asset_pane = ui_asset_pane.cloned()", projection)
        self.assertNotIn("let animation_pane = animation_pane.cloned()", projection)
        self.assertIn("animation_pane,\n        runtime_diagnostics,", projection)


if __name__ == "__main__":
    unittest.main()
