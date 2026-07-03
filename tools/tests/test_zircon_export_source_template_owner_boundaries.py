import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_TEMPLATE_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report_source_template.py"
SOURCE_TEMPLATE_GENERATED_FILES = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_source_template_generated_files.py"
)
SOURCE_TEMPLATE_PATH_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_source_template_path_semantics.py"
)


class ZirconExportSourceTemplateOwnerBoundaryTests(unittest.TestCase):
    def test_source_template_generated_files_live_in_generated_files_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_GENERATED_FILES.exists(),
            "SourceTemplate generated file diagnostics need a dedicated owner",
        )
        report_text = SOURCE_TEMPLATE_REPORT.read_text(encoding="utf-8")
        generated_files_text = SOURCE_TEMPLATE_GENERATED_FILES.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "source_template_generated_file_diagnostics",
            "source_template_generated_file_plan_diagnostics",
            "source_template_validate_generated_file_paths",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the SourceTemplate generated files owner",
            )
            self.assertIn(f"def {function_name}(", generated_files_text)

        self.assertIn(
            "from .pipeline_report_source_template_generated_files import",
            report_text,
            "SourceTemplate report owner should consume the generated files owner",
        )
        self.assertNotIn(
            "from .pipeline_report_source_template import",
            generated_files_text,
            "SourceTemplate generated files owner must not import report orchestration",
        )

    def test_source_template_path_semantics_live_in_path_semantics_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_PATH_SEMANTICS.exists(),
            "SourceTemplate path/string semantics need a shared owner",
        )
        report_text = SOURCE_TEMPLATE_REPORT.read_text(encoding="utf-8")
        generated_files_text = SOURCE_TEMPLATE_GENERATED_FILES.read_text(
            encoding="utf-8"
        )
        path_semantics_text = SOURCE_TEMPLATE_PATH_SEMANTICS.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "resolve_source_template_path_or_diagnostic",
            "source_template_is_non_empty_trimmed_string",
            "source_template_generated_file_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in SourceTemplate path semantics",
            )
            self.assertIn(f"def {function_name}(", path_semantics_text)

        self.assertIn(
            "from .pipeline_report_source_template_path_semantics import",
            report_text,
        )
        self.assertIn(
            "from .pipeline_report_source_template_path_semantics import",
            generated_files_text,
        )
        self.assertNotIn(
            "from .pipeline_report_source_template import",
            path_semantics_text,
            "SourceTemplate path semantics must not import report orchestration",
        )

    def test_source_template_report_owner_stays_under_large_file_threshold(self):
        line_count = len(SOURCE_TEMPLATE_REPORT.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "SourceTemplate report owner should stay below 560 lines after split",
        )

    def test_source_template_generated_files_owner_stays_leaf_sized(self):
        self.assertTrue(
            SOURCE_TEMPLATE_GENERATED_FILES.exists(),
            "SourceTemplate generated files owner should exist before size check",
        )
        line_count = len(
            SOURCE_TEMPLATE_GENERATED_FILES.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            260,
            "SourceTemplate generated files owner should stay below 260 lines",
        )


if __name__ == "__main__":
    unittest.main()
