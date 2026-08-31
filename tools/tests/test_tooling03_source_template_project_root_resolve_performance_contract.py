from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.source_template_generated_project import (
    generated_file_path_safety_diagnostics,
    materialize_generated_files,
    source_template_generated_file_report,
)


class CountingProjectDirectory:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.resolve_calls = 0

    def mkdir(self, *args: object, **kwargs: object) -> None:
        self.path.mkdir(*args, **kwargs)

    def resolve(self) -> Path:
        self.resolve_calls += 1
        return self.path.resolve()

    def __str__(self) -> str:
        return str(self.path)


class Tooling03SourceTemplateProjectRootResolvePerformanceContractTests(
    unittest.TestCase
):
    FILES = [
        {
            "path": f"src/generated_{index}.rs",
            "purpose": "generated test source",
            "contents": f"pub const VALUE_{index}: usize = {index};\n",
        }
        for index in range(4)
    ]

    def test_each_generated_file_pass_resolves_the_project_root_once(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir) / "generated-project"

            materialize_root = CountingProjectDirectory(root)
            materialize_diagnostics: list[str] = []
            self.assertTrue(
                materialize_generated_files(
                    materialize_root,
                    self.FILES,
                    materialize_diagnostics,
                )
            )
            self.assertEqual(materialize_diagnostics, [])
            self.assertEqual(
                materialize_root.resolve_calls,
                1,
                "materialization must cache the generated project root",
            )

            safety_root = CountingProjectDirectory(root)
            safety_diagnostics = generated_file_path_safety_diagnostics(
                safety_root,
                {"plan_summary": {"generated_files": self.FILES}},
            )
            self.assertEqual(safety_diagnostics, [])
            self.assertEqual(
                safety_root.resolve_calls,
                1,
                "path safety validation must cache the generated project root",
            )

            report_root = CountingProjectDirectory(root)
            report_diagnostics: list[str] = []
            report = source_template_generated_file_report(
                report_root,
                self.FILES,
                report_diagnostics,
            )
            self.assertEqual(report_diagnostics, [])
            self.assertEqual(len(report), len(self.FILES))
            self.assertEqual(
                report_root.resolve_calls,
                1,
                "generated file reporting must cache the generated project root",
            )


if __name__ == "__main__":
    unittest.main()
