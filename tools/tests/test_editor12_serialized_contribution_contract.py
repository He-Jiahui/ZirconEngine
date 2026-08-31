from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
INTERFACE = ROOT / "zircon_runtime_interface" / "src"


class SerializedContributionContractTests(unittest.TestCase):
    def test_interface_exports_a_tagged_editor_contribution_batch(self) -> None:
        source = (INTERFACE / "editor_contribution.rs").read_text(encoding="utf-8")
        lib = (INTERFACE / "lib.rs").read_text(encoding="utf-8")

        self.assertIn("pub struct SerializedContributionBatch", source)
        self.assertIn("pub enum SerializedEditorContribution", source)
        self.assertIn("View", source)
        self.assertIn("Drawer", source)
        self.assertIn("Menu", source)
        self.assertIn("Command", source)
        self.assertIn("AssetType", source)
        self.assertIn("LocalizationBundle", source)
        self.assertIn("SettingsPage", source)
        self.assertIn("deny_unknown_fields", source)
        self.assertEqual(source.count("schema: String"), 7)
        self.assertIn("pub fn expected_schema(&self) -> &'static str", source)
        for schema in (
            "zircon.editor.view/1",
            "zircon.editor.drawer/1",
            "zircon.editor.menu/1",
            "zircon.editor.command/1",
            "zircon.editor.asset-type/1",
            "zircon.editor.localization-bundle/1",
            "zircon.editor.settings-page/2",
        ):
            self.assertIn(schema, source)
        self.assertIn("settings_page_v1_literal_payload_is_rejected_by_the_hard_cut", source)
        self.assertIn("UnsupportedContributionSchema", source)
        self.assertIn("contribution.validate_schema()?", source)
        self.assertIn(
            "pub const SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1", source
        )
        self.assertIn("pub mod editor_contribution;", lib)
        self.assertIn("SerializedContributionBatch", lib)
        self.assertIn("SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1", lib)


if __name__ == "__main__":
    unittest.main()
