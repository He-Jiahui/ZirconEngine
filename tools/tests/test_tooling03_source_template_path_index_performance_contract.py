from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export import pipeline_report_source_template_generated_files as generated_files


class CountingPath(str):
    comparisons = 0

    def __eq__(self, other: object) -> bool:
        type(self).comparisons += 1
        return super().__eq__(other)

    __hash__ = str.__hash__


class SourceTemplatePathIndexPerformanceContractTests(unittest.TestCase):
    def test_validate_path_duplicate_detection_has_linear_comparison_budget(self) -> None:
        path_count = 2_000
        paths = [CountingPath(f"src/generated_{index:05d}.rs") for index in range(path_count)]
        validate_report = {
            "plan_summary": {
                "generated_files": [
                    {
                        "path": path,
                        "purpose": "generated source",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    }
                    for path in paths
                ]
            }
        }

        CountingPath.comparisons = 0
        with tempfile.TemporaryDirectory() as temp_dir, mock.patch.object(
            generated_files,
            "source_template_generated_file_path",
            side_effect=lambda project_dir, path, diagnostics, **kwargs: project_dir / path,
        ):
            actual_paths, diagnostics = (
                generated_files.source_template_validate_generated_file_paths(
                    validate_report,
                    Path(temp_dir),
                )
            )

        self.assertEqual(actual_paths, paths)
        self.assertEqual(diagnostics, [])
        self.assertLess(CountingPath.comparisons, path_count * 4)

    def test_plan_report_difference_has_linear_comparison_budget(self) -> None:
        path_count = 2_000
        report_paths = [
            CountingPath(f"src/generated_{index:05d}.rs") for index in range(path_count)
        ]
        validate_report = {
            "plan_summary": {
                "generated_files": [
                    {
                        "path": CountingPath(path),
                        "purpose": "generated source",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    }
                    for path in report_paths
                ]
            }
        }

        CountingPath.comparisons = 0
        with tempfile.TemporaryDirectory() as temp_dir, mock.patch.object(
            generated_files,
            "source_template_generated_file_path",
            side_effect=lambda project_dir, path, diagnostics, **kwargs: project_dir / path,
        ):
            diagnostics = generated_files.source_template_generated_file_plan_diagnostics(
                report_paths,
                validate_report,
                Path(temp_dir),
            )

        self.assertEqual(diagnostics, [])
        self.assertLess(CountingPath.comparisons, path_count * 8)

    def test_duplicate_diagnostics_preserve_existing_order_and_cardinality(self) -> None:
        duplicate_path = "src/repeated.rs"
        validate_report = {
            "plan_summary": {
                "generated_files": [
                    {
                        "path": duplicate_path,
                        "purpose": "first",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    },
                    {
                        "path": duplicate_path,
                        "purpose": "second",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    },
                    {
                        "path": duplicate_path,
                        "purpose": "third",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    },
                ]
            }
        }

        with tempfile.TemporaryDirectory() as temp_dir, mock.patch.object(
            generated_files,
            "source_template_generated_file_path",
            side_effect=lambda project_dir, path, diagnostics, **kwargs: project_dir / path,
        ):
            paths, diagnostics = generated_files.source_template_validate_generated_file_paths(
                validate_report,
                Path(temp_dir),
            )

        self.assertEqual(paths, [duplicate_path])
        self.assertEqual(
            diagnostics,
            [
                f"SourceTemplate Validate generated file path {duplicate_path} is duplicated",
                f"SourceTemplate Validate generated file path {duplicate_path} is duplicated",
            ],
        )

    def test_missing_and_undeclared_diagnostics_keep_input_order(self) -> None:
        report_paths = ["src/report_b.rs", "src/report_a.rs"]
        validate_report = {
            "plan_summary": {
                "generated_files": [
                    {
                        "path": "src/plan_b.rs",
                        "purpose": "generated source",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    },
                    {
                        "path": "src/plan_a.rs",
                        "purpose": "generated source",
                        "byte_length": 1,
                        "content_digest": "0" * 64,
                    },
                ]
            }
        }

        with tempfile.TemporaryDirectory() as temp_dir, mock.patch.object(
            generated_files,
            "source_template_generated_file_path",
            side_effect=lambda project_dir, path, diagnostics, **kwargs: project_dir / path,
        ):
            diagnostics = generated_files.source_template_generated_file_plan_diagnostics(
                report_paths,
                validate_report,
                Path(temp_dir),
            )

        self.assertEqual(
            diagnostics,
            [
                "SourceTemplate report missing generated file from Validate plan: src/plan_b.rs",
                "SourceTemplate report missing generated file from Validate plan: src/plan_a.rs",
                "SourceTemplate report generated file src/report_b.rs is not declared by Validate plan",
                "SourceTemplate report generated file src/report_a.rs is not declared by Validate plan",
            ],
        )


if __name__ == "__main__":
    unittest.main()
