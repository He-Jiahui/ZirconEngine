"""Prevent the retired plan-receipt test infrastructure from returning."""

from pathlib import Path
import unittest


TEST_ROOT = Path(__file__).resolve().parent
RETIRED_FILE_GLOBS = (
    "test_plugin_docs_current_status*.py",
    "plugin_docs_current_status*.py",
)
RETIRED_EXACT_FILES = (
    "plugin_status_document.py",
    "test_plugin_status_document.py",
)
RETIRED_SOURCE_FRAGMENTS = (
    "tools.tests.plugin_status_document",
    "StatusDocumentPath",
    "strip_resolved_output_archives",
    "resolved plan output archive",
)


class CurrentStatusValidationReceiptBoundaryTests(unittest.TestCase):
    def test_current_status_receipt_test_family_is_retired(self) -> None:
        violations: list[str] = []
        for pattern in RETIRED_FILE_GLOBS:
            violations.extend(
                path.relative_to(TEST_ROOT).as_posix()
                for path in sorted(TEST_ROOT.rglob(pattern))
            )
        for name in RETIRED_EXACT_FILES:
            violations.extend(
                path.relative_to(TEST_ROOT).as_posix()
                for path in sorted(TEST_ROOT.rglob(name))
            )

        for path in sorted(TEST_ROOT.rglob("*.py")):
            if path == Path(__file__).resolve():
                continue
            source = path.read_text(encoding="utf-8")
            for fragment in RETIRED_SOURCE_FRAGMENTS:
                if fragment in source:
                    violations.append(f"{path.name}: {fragment}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
