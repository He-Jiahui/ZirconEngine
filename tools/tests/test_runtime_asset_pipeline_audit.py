import sys
import unittest
from pathlib import Path


class RuntimeAssetPipelineAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        audit_scripts = (
            self.repo_root
            / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
        )
        sys.path.insert(0, str(audit_scripts))

    def test_current_child_guard_owners_close_the_runtime_04_audit(self) -> None:
        from runtime_structure_audits.asset_pipeline_boundary import (
            asset_pipeline_boundary_audit,
        )

        audit = asset_pipeline_boundary_audit(self.repo_root)

        self.assertEqual(audit["expected_source_file_count"], 25)
        self.assertEqual(audit["expected_guard_file_count"], 22)
        self.assertEqual(audit["test_anchor_count"], 28)
        self.assertEqual(audit["behavior_test_anchor_count"], 24)
        self.assertEqual(audit["missing_guard_files"], [])
        self.assertEqual(audit["missing_test_anchors"], [])
        self.assertEqual(audit["missing_behavior_test_anchors"], [])
        self.assertEqual(audit["missing_cargo_gate_anchors"], [])
        self.assertTrue(audit["mirror_docs_guard_present"])
        self.assertEqual(audit["risks"], [])

    def test_data_import_errors_preserve_typed_sources(self) -> None:
        importer_error = (
            self.repo_root / "zircon_runtime/src/asset/importer/error.rs"
        ).read_text(encoding="utf-8")
        import_context = (
            self.repo_root / "zircon_runtime/src/asset/importer/contract.rs"
        ).read_text(encoding="utf-8")
        data_importer = (
            self.repo_root
            / "zircon_runtime/src/asset/importer/ingest/import_data_asset.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("SourceTextDecode {", importer_error)
        self.assertIn("source: std::string::FromUtf8Error", importer_error)
        self.assertIn("JsonDeserialize {", importer_error)
        self.assertIn("source: serde_json::Error", importer_error)
        self.assertIn("AssetImportError::SourceTextDecode", import_context)
        self.assertIn("AssetImportError::TomlDeserialize", data_importer)
        self.assertIn("AssetImportError::JsonDeserialize", data_importer)
        self.assertNotIn('Parse(format!("parse toml data:', data_importer)
        self.assertNotIn('Parse(format!("parse json data:', data_importer)


if __name__ == "__main__":
    unittest.main()
