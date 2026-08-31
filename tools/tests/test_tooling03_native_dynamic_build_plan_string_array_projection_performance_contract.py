"""Performance contract for NativeDynamic build-plan string arrays."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report_native_dynamic_build_plan_schema import (
    native_dynamic_build_plan_string_array_projection,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER = REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema.py"


class NativeDynamicBuildPlanStringArrayProjectionPerformanceContractTests(unittest.TestCase):
    def test_projection_preserves_grouped_diagnostics(self) -> None:
        result = native_dynamic_build_plan_string_array_projection(
            "plan.build_features", "build_features", ["", " trim ", "same", "same"]
        )
        self.assertEqual(result[0], [])
        self.assertEqual(result[1], ["plan.build_features must not contain blank entries"])
        self.assertEqual(result[2], ["plan.build_features[1] must be a non-empty trimmed string"])
        self.assertEqual(result[3], ["plan.build_features must not contain duplicate entries"])
        self.assertTrue(result[4])

    def test_non_string_entry_suppresses_value_groups(self) -> None:
        result = native_dynamic_build_plan_string_array_projection(
            "plan.diagnostics", "diagnostics", ["message", 7]
        )
        self.assertEqual(result[:4], (["plan.diagnostics[1] must be a string"], [], [], []))
        self.assertTrue(result[4])

    def test_projection_has_one_array_loop(self) -> None:
        source = OWNER.read_text(encoding="utf-8")
        helper = source[source.index("def native_dynamic_build_plan_string_array_projection("):source.index("def native_dynamic_build_plan_schema_diagnostics(")]
        self.assertEqual(helper.count("for index, entry in enumerate(value):"), 1)


if __name__ == "__main__":
    unittest.main()
