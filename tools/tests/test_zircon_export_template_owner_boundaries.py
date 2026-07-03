import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXPORT_TEMPLATE = REPO_ROOT / "tools/zircon_export/export_template.py"
EXPORT_TEMPLATE_MANIFEST = (
    REPO_ROOT / "tools/zircon_export/export_template_manifest.py"
)


class ZirconExportTemplateOwnerBoundaryTests(unittest.TestCase):
    def test_export_template_manifest_helpers_live_in_manifest_owner(self):
        self.assertTrue(
            EXPORT_TEMPLATE_MANIFEST.exists(),
            "Export-template manifest/path/hash helpers need a dedicated owner",
        )
        template_text = EXPORT_TEMPLATE.read_text(encoding="utf-8")
        manifest_text = EXPORT_TEMPLATE_MANIFEST.read_text(encoding="utf-8")

        for function_name in (
            "template_bundle_config",
            "template_optional_path_field",
            "template_file_manifest",
            "template_bundle_file_path",
            "table_unknown_field_diagnostics",
            "resolve_template_child",
            "resolve_bundle_child",
            "normalize_relative_path",
            "is_safe_relative_path",
            "compute_template_content_hash",
            "is_sha256_hex",
            "workspace_engine_version",
            "validated_target_platform",
            "normalize_target_platform",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                template_text,
                f"{function_name} belongs in the export-template manifest owner",
            )
            self.assertIn(f"def {function_name}(", manifest_text)

        self.assertIn(
            "from .export_template_manifest import",
            template_text,
            "export_template orchestration should consume manifest/path/hash owner",
        )
        self.assertNotIn(
            "from .export_template import",
            manifest_text,
            "manifest/path/hash owner must not import template orchestration",
        )

    def test_export_template_orchestration_stays_under_large_file_threshold(self):
        line_count = len(EXPORT_TEMPLATE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            650,
            "export_template orchestration should stay below the split threshold",
        )


if __name__ == "__main__":
    unittest.main()
