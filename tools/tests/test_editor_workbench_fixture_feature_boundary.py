from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorWorkbenchFixtureFeatureBoundaryTests(unittest.TestCase):
    def test_fixture_module_is_excluded_from_default_product_builds(self) -> None:
        workbench = (
            ROOT / "zircon_editor/src/ui/workbench/mod.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            '#[cfg(any(test, feature = "integration-contracts"))]\n'
            "pub mod fixture;",
            workbench,
        )
        self.assertNotIn("\npub mod fixture;", workbench.replace(
            '#[cfg(any(test, feature = "integration-contracts"))]\n'
            "pub mod fixture;",
            "",
        ))

    def test_external_contract_target_enables_the_fixture_feature(self) -> None:
        manifest = (ROOT / "zircon_editor/Cargo.toml").read_text(encoding="utf-8")

        self.assertIn("integration-contracts = []", manifest)
        self.assertIn('required-features = ["integration-contracts"]', manifest)

    def test_floating_window_parity_schema_is_test_support_only(self) -> None:
        workbench = (
            ROOT / "zircon_editor/src/ui/workbench/mod.rs"
        ).read_text(encoding="utf-8")
        gate = '#[cfg(any(test, feature = "integration-contracts"))]'

        self.assertIn(f"{gate}\nmod floating_window;", workbench)
        self.assertIn(f"{gate}\npub use floating_window::{{", workbench)


if __name__ == "__main__":
    unittest.main()
