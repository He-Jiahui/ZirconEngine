from __future__ import annotations

import re
import tempfile
import unittest
from pathlib import Path

from tools.check_conventions import (
    ConventionCommand,
    audit_document_paths,
    convention_commands,
)


class CheckConventionsTests(unittest.TestCase):
    def test_ci_invokes_single_convention_entrypoint_without_duplicate_command_plan(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        workflow = (repo_root / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        rust_job = self._workflow_job(workflow, "rust")
        rust_steps = self._workflow_steps(rust_job)
        workflow_steps = self._workflow_steps(workflow)

        test_command = "python -m unittest tools.tests.test_check_conventions -v"
        runner_command = "python tools/check_conventions.py --json"
        workflow_runs = [
            self._normalized_yaml_commands(run)
            for step in workflow_steps
            if (run := self._workflow_step_run(step)) is not None
        ]
        rust_runs = [
            self._normalized_yaml_commands(run)
            for step in rust_steps
            if (run := self._workflow_step_run(step)) is not None
        ]
        normalized_workflow_runs = "\n".join(workflow_runs)

        self.assertEqual(normalized_workflow_runs.count(test_command), 1)
        self.assertEqual(normalized_workflow_runs.count(runner_command), 1)
        self.assertEqual("\n".join(rust_runs).count(test_command), 1)
        self.assertEqual("\n".join(rust_runs).count(runner_command), 1)

        blocking_steps = [
            next(
                step
                for step in rust_steps
                if (
                    (run := self._workflow_step_run(step)) is not None
                    and command in self._normalized_yaml_commands(run)
                )
            )
            for command in (test_command, runner_command)
        ]
        for step in blocking_steps:
            self.assertNotRegex(step, r"(?m)^\s*if\s*:")
            self.assertNotRegex(step, r"(?m)^\s*continue-on-error\s*:")
        test_step_index, runner_step_index = (
            rust_steps.index(step) for step in blocking_steps
        )
        build_step_index = next(
            index
            for index, step in enumerate(rust_steps)
            if (
                (run := self._workflow_step_run(step)) is not None
                and "cargo build --workspace" in self._normalized_yaml_commands(run)
            )
        )
        self.assertLess(test_step_index, runner_step_index)
        self.assertLess(runner_step_index, build_step_index)

        self.assertNotIn("tools/check_conventions.py --dry-run", normalized_workflow_runs)
        self.assertNotRegex(normalized_workflow_runs, r"\bcargo\s+fmt\b")
        self.assertNotRegex(normalized_workflow_runs, r"\bcargo\s+clippy\b")

    def test_workflow_run_extraction_ignores_step_names_and_comments(self) -> None:
        step = (
            "      - name: python tools/check_conventions.py --json\n"
            "        # python -m unittest tools.tests.test_check_conventions -v\n"
            "        run: |\n"
            "          python -c \"print('actual command')\"\n"
        )

        self.assertEqual(
            self._normalized_yaml_commands(self._workflow_step_run(step) or ""),
            "python -c \"print('actual command')\"",
        )

    @staticmethod
    def _workflow_job(workflow: str, job_name: str) -> str:
        lines = workflow.splitlines()
        start = next(
            index for index, line in enumerate(lines) if line == f"  {job_name}:"
        )
        end = next(
            (
                index
                for index in range(start + 1, len(lines))
                if re.fullmatch(r"  [A-Za-z0-9_-]+:\s*", lines[index])
            ),
            len(lines),
        )
        return "\n".join(lines[start:end])

    @staticmethod
    def _workflow_steps(job: str) -> list[str]:
        lines = job.splitlines()
        starts = [index for index, line in enumerate(lines) if line.startswith("      - ")]
        return [
            "\n".join(lines[start : starts[index + 1] if index + 1 < len(starts) else len(lines)])
            for index, start in enumerate(starts)
        ]

    @staticmethod
    def _workflow_step_run(step: str) -> str | None:
        lines = step.splitlines()
        for index, line in enumerate(lines):
            match = re.match(r"^(\s*)run\s*:\s*(.*)$", line)
            if match is None:
                continue
            indentation, value = match.groups()
            if value not in {"|", ">", "|-", ">-", "|+", ">+"}:
                return value
            block = []
            for child in lines[index + 1 :]:
                if child.strip() and len(child) - len(child.lstrip()) <= len(indentation):
                    break
                block.append(child.strip())
            return "\n".join(block)
        return None

    @staticmethod
    def _normalized_yaml_commands(source: str) -> str:
        return re.sub(r"\s+", " ", source).strip()

    def test_document_audit_reports_missing_related_code_and_implementation_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (repo_root / "src").mkdir()
            (repo_root / "src" / "present.rs").write_text("", encoding="utf-8")
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - src/present.rs\n"
                "  - src/missing.rs\n"
                "implementation_files:\n"
                "  - src/missing_impl.rs\n"
                "tests:\n"
                "  - cargo test -p example\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["document_count"], 1)
            self.assertEqual(report["checked_path_count"], 3)
            self.assertEqual(report["affected_document_count"], 1)
            self.assertEqual(report["reason_counts"], {"missing path": 2})
            self.assertEqual(report["path_root_counts"], {"src": 2})
            self.assertEqual(
                [(item["field"], item["path"]) for item in report["violations"]],
                [
                    ("implementation_files", "src/missing_impl.rs"),
                    ("related_code", "src/missing.rs"),
                ],
            )

    def test_document_audit_accepts_existing_files_and_directories(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (repo_root / "src" / "feature").mkdir(parents=True)
            (repo_root / "src" / "feature" / "mod.rs").write_text("", encoding="utf-8")
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - src/feature\n"
                "  - src/feature/mod.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["checked_path_count"], 2)
            self.assertEqual(report["violations"], [])

    def test_document_audit_rejects_absolute_and_parent_escape_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - ../outside.rs\n"
                "  - C:/outside.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(
                [item["reason"] for item in report["violations"]],
                ["repository escape", "absolute path"],
            )

    def test_convention_command_plan_is_stable_and_scoped(self) -> None:
        commands = convention_commands()

        self.assertEqual(
            commands,
            [
                ConventionCommand("fmt", ("cargo", "fmt", "--all", "--check")),
                ConventionCommand(
                    "clippy",
                    (
                        "cargo",
                        "clippy",
                        "-p",
                        "zircon_runtime_interface",
                        "-p",
                        "zircon_app",
                        "--all-targets",
                        "--no-deps",
                        "--locked",
                        "--",
                        "-D",
                        "warnings",
                    ),
                ),
            ],
        )


if __name__ == "__main__":
    unittest.main()
