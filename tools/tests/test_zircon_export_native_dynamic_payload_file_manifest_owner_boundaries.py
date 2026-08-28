import unittest
from pathlib import Path

from tools.zircon_export.native_dynamic_payload import (
    normalized_file_manifest as facade_normalized_file_manifest,
)
from tools.zircon_export.native_dynamic_payload_file_manifest import (
    normalized_file_manifest,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_PAYLOAD = REPO_ROOT / "tools/zircon_export/native_dynamic_payload.py"
NATIVE_DYNAMIC_PAYLOAD_FILE_MANIFEST = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_payload_file_manifest.py"
)


class ZirconExportNativeDynamicPayloadFileManifestOwnerBoundaryTests(unittest.TestCase):
    def test_file_manifest_normalizer_preserves_facade_compatibility(self):
        self.assertIs(facade_normalized_file_manifest, normalized_file_manifest)
        row = {"path": "plugins/example.dll", "bytes": 7, "sha256": "a" * 64}
        self.assertEqual(normalized_file_manifest([row]), [row])
        self.assertIsNone(normalized_file_manifest({"path": "not-an-array"}))
        self.assertIsNone(
            normalized_file_manifest(
                [{"path": "plugins/example.dll", "bytes": "7", "sha256": "a" * 64}]
            )
        )

    def test_file_manifest_path_and_hash_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_PAYLOAD_FILE_MANIFEST.exists(),
            "NativeDynamic payload file manifest/path/hash helpers need a dedicated owner",
        )
        payload_text = NATIVE_DYNAMIC_PAYLOAD.read_text(encoding="utf-8")
        file_manifest_text = NATIVE_DYNAMIC_PAYLOAD_FILE_MANIFEST.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "normalized_file_manifest",
            "native_dynamic_file_manifest",
            "native_dynamic_plugins_bundle_file_manifest",
            "native_dynamic_plugins_file_manifest",
            "native_dynamic_package_payload_file_manifest",
            "read_native_dynamic_payload_file",
            "native_dynamic_package_loadable_artifacts",
            "native_dynamic_payload_tree_entries",
            "resolve_native_dynamic_payload_path",
            "native_dynamic_content_hash",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                payload_text,
                f"{function_name} belongs in the file-manifest owner",
            )
            self.assertIn(f"def {function_name}(", file_manifest_text)

        self.assertIn(
            "from .native_dynamic_payload_file_manifest import",
            payload_text,
            "payload summary owner should consume the file-manifest owner",
        )
        self.assertNotIn(
            "from .native_dynamic_payload import",
            file_manifest_text,
            "file-manifest owner must not import the payload summary owner",
        )

    def test_file_manifest_consumers_import_owner_directly(self):
        for relative_path in (
            "tools/zircon_export/native_dynamic_materialize.py",
            "tools/zircon_export/native_dynamic_templates.py",
            "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py",
            "tools/zircon_export/pipeline_report_native_dynamic_payload_package_report.py",
            "tools/zircon_export/pipeline_report_native_dynamic_payload_stage_report.py",
            "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py",
            "tools/zircon_export/plugin_build_package.py",
        ):
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(
                "from .native_dynamic_payload_file_manifest import",
                text,
                f"{relative_path} should consume the file-manifest owner directly",
            )

    def test_payload_summary_owner_stays_under_large_file_threshold(self):
        line_count = len(NATIVE_DYNAMIC_PAYLOAD.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            620,
            "NativeDynamic payload summary owner should stay below 620 lines after file-manifest split",
        )


if __name__ == "__main__":
    unittest.main()
