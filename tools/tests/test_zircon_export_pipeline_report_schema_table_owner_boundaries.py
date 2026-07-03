from __future__ import annotations

import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_EXPORT_DIR = REPO_ROOT / "tools/zircon_export"
SCHEMA_TABLE = ZIRCON_EXPORT_DIR / "pipeline_report_schema_table.py"
SCHEMA_STRING_ARRAY = ZIRCON_EXPORT_DIR / "pipeline_report_schema_string_array.py"

STRING_ARRAY_HELPERS = (
    "non_empty_string_array_schema_diagnostics",
    "string_array_no_blank_entries_schema_diagnostics",
    "string_array_trimmed_non_empty_entries_schema_diagnostics",
    "string_array_unique_entries_schema_diagnostics",
    "string_array_duplicate_entry_index_schema_diagnostics",
)


def _imports_from(path: Path, module_name: str) -> set[str]:
    module = ast.parse(path.read_text(encoding="utf-8"))
    imported: set[str] = set()
    for node in ast.walk(module):
        if (
            isinstance(node, ast.ImportFrom)
            and node.level == 1
            and node.module == module_name
        ):
            imported.update(alias.name for alias in node.names)
    return imported


class PipelineReportSchemaTableOwnerBoundaryTests(unittest.TestCase):
    def test_string_array_helpers_live_in_string_array_owner(self):
        self.assertTrue(
            SCHEMA_STRING_ARRAY.exists(),
            "string-array schema diagnostics belong in a focused owner",
        )
        table_text = SCHEMA_TABLE.read_text(encoding="utf-8")
        string_array_text = SCHEMA_STRING_ARRAY.read_text(encoding="utf-8")

        for helper_name in STRING_ARRAY_HELPERS:
            self.assertNotIn(
                f"def {helper_name}(",
                table_text,
                f"{helper_name} must not remain in the table/sequence owner",
            )
            self.assertIn(
                f"def {helper_name}(",
                string_array_text,
                f"{helper_name} must be defined by the string-array owner",
            )

        self.assertNotIn(
            "from .pipeline_report_schema_table import",
            string_array_text,
            "string-array owner must not import the table/sequence owner",
        )
        self.assertLessEqual(
            len(table_text.splitlines()),
            390,
            "table/sequence owner should shrink after string-array helper split",
        )
        self.assertLessEqual(
            len(string_array_text.splitlines()),
            120,
            "string-array schema owner should stay a focused leaf module",
        )

    def test_schema_table_consumers_import_string_array_helpers_from_child_owner(self):
        failures: list[str] = []
        for path in ZIRCON_EXPORT_DIR.glob("*.py"):
            if path.name == SCHEMA_STRING_ARRAY.name:
                continue
            table_imports = _imports_from(path, "pipeline_report_schema_table")
            leaked = sorted(table_imports.intersection(STRING_ARRAY_HELPERS))
            if leaked:
                failures.append(f"{path.name} imports from table owner: {', '.join(leaked)}")

        self.assertEqual(
            [],
            failures,
            "string-array helpers must be imported from pipeline_report_schema_string_array",
        )


if __name__ == "__main__":
    unittest.main()
