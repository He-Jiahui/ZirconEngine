import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESOLUTION_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_semantics.py"
)
PATH_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_path_semantics.py"
)


class ZirconExportPlatformBundleTemplateResolutionPathSemanticsOwnerTests(
    unittest.TestCase
):
    def test_path_semantics_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            PATH_SEMANTICS.exists(),
            "PlatformBundle template resolution path semantics need a dedicated owner",
        )
        semantics_text = RESOLUTION_SEMANTICS.read_text(encoding="utf-8")
        path_semantics_text = PATH_SEMANTICS.read_text(encoding="utf-8")

        for function_name in (
            "template_resolution_path_containment_diagnostics",
            "template_resolution_template_dir_uniqueness_diagnostics",
            "template_resolution_entries_inside_root_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                semantics_text,
                f"{function_name} belongs in the path semantics owner",
            )
            self.assertIn(f"def {function_name}(", path_semantics_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_resolution_path_semantics import",
            semantics_text,
            "resolution semantics owner should consume path semantics directly",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_resolution_semantics import",
            path_semantics_text,
            "path semantics owner must not import resolution orchestration",
        )

    def test_resolution_semantics_owner_stays_under_large_file_threshold(self):
        line_count = len(RESOLUTION_SEMANTICS.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "Template resolution semantics owner should stay below 560 lines "
            "after path semantics split",
        )


if __name__ == "__main__":
    unittest.main()
