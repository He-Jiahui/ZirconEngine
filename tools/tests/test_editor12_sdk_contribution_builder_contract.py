from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SDK = ROOT / "zircon_plugins" / "plugin_sdk"


class EditorContributionBuilderContractTests(unittest.TestCase):
    def test_editor_feature_exposes_a_dto_only_contribution_builder(self) -> None:
        cargo_toml = (SDK / "Cargo.toml").read_text(encoding="utf-8")
        lib = (SDK / "src" / "lib.rs").read_text(encoding="utf-8")
        source = (SDK / "src" / "editor_contribution.rs").read_text(encoding="utf-8")

        self.assertIn(
            'editor = ["runtime", "dep:zircon_editor", "editor_contribution"]',
            cargo_toml,
        )
        self.assertIn(
            'editor_contribution = ["dep:zircon_runtime_interface"]', cargo_toml
        )
        self.assertIn(
            'native = ["dep:serde", "dep:toml", "editor_contribution"]', cargo_toml
        )
        self.assertIn('#[cfg(feature = "editor_contribution")]', lib)
        self.assertIn("pub mod editor_contribution;", lib)
        self.assertIn("EditorContributionBuilder", lib)
        self.assertIn("pub struct EditorContributionBuilder", source)
        self.assertIn("pub fn new(package_id: impl Into<String>) -> Self", source)
        for method in (
            "pub fn view(",
            "pub fn drawer(",
            "pub fn menu(",
            "pub fn command(",
            "pub fn asset_type(",
            "pub fn settings_page(",
            "pub fn build(self)",
        ):
            self.assertIn(method, source)
        for schema_constant in (
            "VIEW_SCHEMA",
            "DRAWER_SCHEMA",
            "MENU_SCHEMA",
            "COMMAND_SCHEMA",
            "ASSET_TYPE_SCHEMA",
            "SETTINGS_PAGE_SCHEMA",
        ):
            self.assertIn(
                f"SerializedEditorContribution::{schema_constant}.to_string()", source
            )
        self.assertIn("SerializedContributionBatch::new(self.package_id, self.contributions)", source)
        self.assertNotIn("EditorExtensionRegistry", source)


if __name__ == "__main__":
    unittest.main()
