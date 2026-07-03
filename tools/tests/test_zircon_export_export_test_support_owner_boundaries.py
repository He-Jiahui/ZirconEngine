"""Boundary tests for zircon_export shared test support ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXPORT_TEST_SUPPORT = REPO_ROOT / "tools/zircon_export/tests/export_test_support.py"
NATIVE_DYNAMIC_SUPPORT = (
    REPO_ROOT / "tools/zircon_export/tests/native_dynamic_export_test_support.py"
)
PLATFORM_BUNDLE_SUPPORT = (
    REPO_ROOT / "tools/zircon_export/tests/platform_bundle_export_test_support.py"
)

NATIVE_DYNAMIC_HELPERS = (
    "_native_dynamic_package_export",
    "_write_native_dynamic_report",
    "_write_native_dynamic_stage_plugins",
    "_write_native_dynamic_package_fixture",
)
PLATFORM_BUNDLE_HELPERS = (
    "_write_platform_bundle_report_with_native_plugins_payload",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


def _python_test_files() -> list[Path]:
    return sorted((REPO_ROOT / "tools/zircon_export/tests").glob("*.py"))


def _root_support_imports(text: str) -> set[str]:
    imported: set[str] = set()
    lines = text.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        if line == "from tools.zircon_export.tests.export_test_support import (":
            index += 1
            while index < len(lines) and lines[index] != ")":
                item = lines[index].strip().removesuffix(",")
                if item:
                    imported.add(item)
                index += 1
        elif line.startswith(
            "from tools.zircon_export.tests.export_test_support import "
        ):
            imported.update(
                item.strip()
                for item in line.rsplit(" import ", maxsplit=1)[1].split(",")
                if item.strip()
            )
        index += 1
    return imported


class ZirconExportTestSupportOwnerBoundaryTests(unittest.TestCase):
    def test_native_dynamic_helpers_have_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_SUPPORT.exists(),
            "NativeDynamic export test support owner is missing",
        )

        root_text = EXPORT_TEST_SUPPORT.read_text(encoding="utf-8")
        native_text = NATIVE_DYNAMIC_SUPPORT.read_text(encoding="utf-8")
        for helper_name in NATIVE_DYNAMIC_HELPERS:
            with self.subTest(helper=helper_name):
                self.assertNotIn(f"def {helper_name}(", root_text)
                self.assertIn(f"def {helper_name}(", native_text)

    def test_platform_bundle_helpers_have_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_SUPPORT.exists(),
            "PlatformBundle export test support owner is missing",
        )

        root_text = EXPORT_TEST_SUPPORT.read_text(encoding="utf-8")
        platform_text = PLATFORM_BUNDLE_SUPPORT.read_text(encoding="utf-8")
        for helper_name in PLATFORM_BUNDLE_HELPERS:
            with self.subTest(helper=helper_name):
                self.assertNotIn(f"def {helper_name}(", root_text)
                self.assertIn(f"def {helper_name}(", platform_text)

    def test_moved_helpers_are_not_imported_from_root_support(self):
        moved_helpers = NATIVE_DYNAMIC_HELPERS + PLATFORM_BUNDLE_HELPERS
        failures: list[str] = []

        for path in _python_test_files():
            if path == EXPORT_TEST_SUPPORT:
                continue
            imported = _root_support_imports(path.read_text(encoding="utf-8"))
            for helper_name in moved_helpers:
                if helper_name in imported:
                    failures.append(f"{path.name}: imports {helper_name} from root")

        if failures:
            self.fail(
                "Moved export support helpers must import from focused owners:\n"
                + "\n".join(failures)
            )

    def test_export_test_support_owners_stay_small(self):
        self.assertLess(
            _line_count(EXPORT_TEST_SUPPORT),
            1000,
            "export_test_support.py should stay below the large-file budget",
        )
        for path, budget, description in (
            (NATIVE_DYNAMIC_SUPPORT, 380, "NativeDynamic support"),
            (PLATFORM_BUNDLE_SUPPORT, 260, "PlatformBundle support"),
        ):
            with self.subTest(owner=description):
                self.assertTrue(path.exists(), f"{description} owner is missing")
                self.assertLess(
                    _line_count(path),
                    budget,
                    f"{description} owner should stay focused",
                )


if __name__ == "__main__":
    unittest.main()
