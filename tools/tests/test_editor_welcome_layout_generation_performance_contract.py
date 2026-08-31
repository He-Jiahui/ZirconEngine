from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "zircon_editor/src/ui/retained_host/host_contract/data/welcome.rs"
APPLY = ROOT / "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"
PAINT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome"
)
PAINT_ROOT = PAINT.with_suffix(".rs")


class EditorWelcomeLayoutGenerationPerformanceContractTests(unittest.TestCase):
    def test_host_welcome_data_owns_a_typed_layout_projection(self) -> None:
        source = DATA.read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct WelcomePaneLayoutData", source)
        self.assertIn("pub layout: WelcomePaneLayoutData", source)
        self.assertIn("pub(crate) fn capture", source)
        for control_id in (
            "WelcomeOuterPanel",
            "WelcomeRecentPanel",
            "WelcomeMainPanel",
            "WelcomeHeroPanel",
            "WelcomeStatusPanel",
            "WelcomeNewProjectHeaderPanel",
            "WelcomeProjectNameField",
            "WelcomeLocationField",
            "WelcomePreviewPanel",
            "WelcomeValidationPanel",
            "WelcomeRecentHeaderPanel",
            "WelcomeRecentListPanel",
        ):
            self.assertEqual(source.count(f'"{control_id}"'), 1)

    def test_dispatch_patch_walk_also_compiles_layout_without_a_second_scan(self) -> None:
        source = APPLY.read_text(encoding="utf-8")
        start = source.index("fn welcome_nodes_with_native_dispatch(")
        end = source.index("\nfn project_welcome_pane(", start)
        function = source[start:end]
        compact = "".join(function.split())

        self.assertIn("for(row,node)innodes.iter().enumerate()", compact)
        self.assertIn("layout.capture(node)", function)
        self.assertIn("(nodes.with_row_patches(row_patches), layout)", function)
        self.assertEqual(function.count("nodes.iter()"), 1)

    def test_welcome_paint_does_not_scan_the_template_node_model(self) -> None:
        sources = [PAINT_ROOT.read_text(encoding="utf-8")]
        sources.extend(
            path.read_text(encoding="utf-8")
            for path in sorted(PAINT.rglob("*.rs"))
        )
        paint_source = "\n".join(sources)

        self.assertNotIn("welcome.nodes.row_count()", paint_source)
        self.assertNotIn("welcome.nodes.row_data(", paint_source)
        self.assertNotIn("welcome_node_frame", paint_source)


if __name__ == "__main__":
    unittest.main()
