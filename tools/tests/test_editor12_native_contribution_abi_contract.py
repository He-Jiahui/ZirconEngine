from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LOADER = ROOT / "zircon_runtime" / "src" / "plugin" / "native_plugin_loader"


class NativeContributionAbiContractTests(unittest.TestCase):
    def test_editor_entry_parses_only_the_versioned_contribution_batch_payload(self) -> None:
        abi = (LOADER / "native_plugin_abi.rs").read_text(encoding="utf-8")
        schema = (LOADER / "behavior_validation" / "schema.rs").read_text(encoding="utf-8")
        report = (LOADER / "behavior_validation" / "report.rs").read_text(encoding="utf-8")

        self.assertIn("SerializedContributionBatch", abi)
        self.assertIn("SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1", abi)
        self.assertIn("pub editor_contribution_batch: Option<SerializedContributionBatch>", abi)
        self.assertIn("editor_contribution_batch_from_behavior", abi)
        self.assertIn("serde_json::from_str", abi)
        self.assertIn("batch.package_id() != plugin_id", abi)
        self.assertIn("PluginModuleKind::Editor", abi)
        self.assertIn("editor_contribution_batch_decodes_valid_editor_payload", abi)
        self.assertIn("editor_contribution_batch_rejects_package_mismatch", abi)
        self.assertIn("expected_registration_manifest_schema", schema)
        self.assertIn("SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1", schema)
        self.assertIn("expected_registration_manifest_schema(module_kind)", report)


if __name__ == "__main__":
    unittest.main()
