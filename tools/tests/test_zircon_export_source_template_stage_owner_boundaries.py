import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_TEMPLATE_STAGE = REPO_ROOT / "tools/zircon_export/source_template.py"
SOURCE_TEMPLATE_GENERATED_PROJECT = (
    REPO_ROOT / "tools/zircon_export/source_template_generated_project.py"
)


class ZirconExportSourceTemplateStageOwnerBoundaryTests(unittest.TestCase):
    def test_generated_project_materialization_lives_in_generated_project_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_GENERATED_PROJECT.exists(),
            "SourceTemplate generated project materialization needs a dedicated owner",
        )
        stage_text = SOURCE_TEMPLATE_STAGE.read_text(encoding="utf-8")
        generated_project_text = SOURCE_TEMPLATE_GENERATED_PROJECT.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "generated_file_summaries",
            "source_template_generated_files_plan_diagnostics",
            "materialize_generated_files",
            "generated_file_path_duplicate_diagnostics",
            "source_template_generated_file_report",
            "reset_generated_project_dir",
            "rewrite_generated_manifest_paths",
            "resolve_project_child",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the SourceTemplate generated project owner",
            )
            self.assertIn(f"def {function_name}(", generated_project_text)

        self.assertIn(
            "from .source_template_generated_project import",
            stage_text,
            "SourceTemplate stage runner should consume the generated project owner",
        )
        self.assertNotIn(
            "from .source_template import",
            generated_project_text,
            "SourceTemplate generated project owner must not import stage orchestration",
        )

    def test_source_template_stage_runner_stays_orchestration_sized(self):
        line_count = len(SOURCE_TEMPLATE_STAGE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            450,
            "SourceTemplate stage runner should stay below 450 lines after split",
        )

    def test_source_template_generated_project_owner_stays_leaf_sized(self):
        self.assertTrue(
            SOURCE_TEMPLATE_GENERATED_PROJECT.exists(),
            "SourceTemplate generated project owner should exist before size check",
        )
        line_count = len(
            SOURCE_TEMPLATE_GENERATED_PROJECT.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            320,
            "SourceTemplate generated project owner should stay below 320 lines",
        )


if __name__ == "__main__":
    unittest.main()
