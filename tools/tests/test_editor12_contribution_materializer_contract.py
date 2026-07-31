from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MATERIALIZER = ROOT / "zircon_editor" / "src" / "core" / "plugin" / "materializer.rs"


class ContributionMaterializerContractTests(unittest.TestCase):
    def test_materializer_uses_a_candidate_registry_and_all_known_data_variants(self) -> None:
        source = MATERIALIZER.read_text(encoding="utf-8")

        self.assertIn("materialize_serialized_contribution_batch", source)
        self.assertIn("let mut candidate = registry.clone();", source)
        self.assertIn("*registry = candidate;", source)
        self.assertIn("SerializedEditorContribution::View", source)
        self.assertIn("SerializedEditorContribution::Drawer", source)
        self.assertIn("SerializedEditorContribution::Menu", source)
        self.assertIn("SerializedEditorContribution::Command", source)
        self.assertIn("AssetType", source)
        self.assertIn("SettingsPage", source)
        self.assertGreaterEqual(source.count(".."), 6)

    def test_materializer_covers_all_six_kinds_and_candidate_rollback_in_rust(self) -> None:
        source = MATERIALIZER.read_text(encoding="utf-8")

        self.assertIn("materializes_every_supported_contribution_kind", source)
        self.assertIn("failed_batch_does_not_publish_partial_contributions", source)
        self.assertIn("registry.command_ids().count(), 1", source)
        self.assertIn("registry.command_ids().count(), 0", source)


if __name__ == "__main__":
    unittest.main()
