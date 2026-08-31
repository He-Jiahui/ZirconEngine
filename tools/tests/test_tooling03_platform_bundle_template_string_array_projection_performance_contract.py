"""Performance contract for PlatformBundle template string arrays."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report_platform_bundle_template_schema import (
    platform_bundle_template_report_schema_diagnostics,
    template_report_string_array_projection,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER = REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_template_schema.py"


class PlatformBundleTemplateStringArrayProjectionPerformanceContractTests(unittest.TestCase):
    def test_projection_scans_each_array_once(self) -> None:
        source = OWNER.read_text(encoding="utf-8")
        helper = source[source.index("def template_report_string_array_projection("):source.index("def platform_bundle_template_report_schema_diagnostics(")]
        self.assertEqual(helper.count("for index, value in enumerate(values):"), 1)

    def test_projection_preserves_ordered_groups_for_mixed_values(self) -> None:
        result = template_report_string_array_projection(
            "template",
            {"diagnostics": ["", " message ", 7], "compatible_profiles": ["a", " b ", "a"]},
        )
        self.assertEqual(result[0], ["template.diagnostics[2] must be a string"])
        self.assertEqual(result[1], [])
        self.assertEqual(result[2], ["template.compatible_profiles[1] must be a non-empty trimmed string"])
        self.assertEqual(
            result[3], ["template.compatible_profiles duplicate entry a"]
        )
        self.assertTrue(result[4])

    def test_full_schema_accepts_large_clean_string_arrays(self) -> None:
        template = {"fatal": True, "diagnostics": ["message"], "compatible_profiles": [f"profile-{i}" for i in range(512)]}
        diagnostics = platform_bundle_template_report_schema_diagnostics(template)
        self.assertNotIn("must be a string array", "\n".join(diagnostics))
        self.assertNotIn("must contain duplicate entries", "\n".join(diagnostics))


if __name__ == "__main__":
    unittest.main()
