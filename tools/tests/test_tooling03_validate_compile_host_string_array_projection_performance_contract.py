"""Performance contract for CompileHost string-array validation."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report_validate_compile_host_schema import (
    validate_compile_host_string_array_projection,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER = REPO_ROOT / "tools/zircon_export/pipeline_report_validate_compile_host_schema.py"


class ValidateCompileHostStringArrayProjectionPerformanceContractTests(unittest.TestCase):
    def test_projection_preserves_groups(self) -> None:
        result = validate_compile_host_string_array_projection(
            "host.app_features", ["", " trim ", "x", "x"], check_trimmed=True, check_duplicate_indexes=True
        )
        self.assertEqual(result[0], [])
        self.assertEqual(result[1], ["host.app_features must not contain blank entries"])
        self.assertEqual(result[2], ["host.app_features[1] must be a non-empty trimmed string"])
        self.assertEqual(result[3], ["host.app_features[3] duplicates entry 2"])

    def test_projection_has_one_entry_loop(self) -> None:
        source = OWNER.read_text(encoding="utf-8")
        helper = source[source.index("def validate_compile_host_string_array_projection("):source.index("def validate_library_embed_compile_host_schema_diagnostics(")]
        self.assertEqual(helper.count("for index, entry in enumerate(value):"), 1)


if __name__ == "__main__":
    unittest.main()
