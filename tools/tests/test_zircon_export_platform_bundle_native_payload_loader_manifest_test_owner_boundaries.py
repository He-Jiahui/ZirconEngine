"""Boundary tests for PlatformBundle native payload loader manifest ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ROOT_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest.py"
)
ABI_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest_abi_schema.py"
)
SUPPORT_FILE = (
    REPO_ROOT
    / "tools/zircon_export/tests/platform_bundle_native_payload_loader_manifest_test_support.py"
)

ABI_SCHEMA_TESTS = (
    "test_report_rejects_native_plugins_payload_loader_manifest_bad_abi_table",
    "test_report_rejects_native_plugins_payload_loader_manifest_unknown_abi_field",
    "test_report_rejects_native_plugins_payload_loader_manifest_abi_field_types",
    "test_report_rejects_native_plugins_payload_loader_manifest_abi_blank_strings",
    "test_report_rejects_native_plugins_payload_loader_manifest_abi_missing_required_field",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PlatformBundleNativePayloadLoaderManifestTestOwnerBoundaryTests(
    unittest.TestCase
):
    def test_abi_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            ABI_SCHEMA_TEST.exists(),
            "Native payload loader manifest ABI schema owner is missing",
        )

        root_text = ROOT_TEST.read_text(encoding="utf-8")
        abi_text = ABI_SCHEMA_TEST.read_text(encoding="utf-8")
        for test_name in ABI_SCHEMA_TESTS:
            with self.subTest(test=test_name):
                self.assertNotIn(f"def {test_name}", root_text)
                self.assertIn(f"def {test_name}", abi_text)

    def test_native_payload_refresh_fixture_lives_in_shared_support(self):
        root_text = ROOT_TEST.read_text(encoding="utf-8")
        abi_text = ABI_SCHEMA_TEST.read_text(encoding="utf-8")
        support_text = SUPPORT_FILE.read_text(encoding="utf-8")

        self.assertIn("def _refresh_platform_native_plugins_payload", support_text)
        self.assertNotIn("def _refresh_platform_native_plugins_payload", root_text)
        self.assertNotIn("def _refresh_platform_native_plugins_payload", abi_text)

    def test_native_payload_loader_manifest_test_owners_stay_small(self):
        budgets = (
            (ROOT_TEST, 820, "root loader manifest owner"),
            (ABI_SCHEMA_TEST, 360, "ABI schema owner"),
            (SUPPORT_FILE, 80, "loader manifest test support"),
        )
        for path, budget, description in budgets:
            with self.subTest(owner=description):
                self.assertTrue(path.exists(), f"{description} is missing")
                self.assertLess(
                    _line_count(path),
                    budget,
                    f"{description} should stay focused",
                )


if __name__ == "__main__":
    unittest.main()
