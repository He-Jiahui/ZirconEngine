from __future__ import annotations

import io
import json
import re
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from tools.check_conventions import (
    ConventionCommand,
    audit_document_paths,
    audit_rule_guard_coverage,
    convention_commands,
    main,
    run_conventions,
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

        test_command = (
            "python -m unittest tools.tests.test_check_conventions "
            "tools.tests.test_frameworks_06_ci_toolchain_contract -v"
        )
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
        self.assertEqual(
            self._copied_owned_cargo_commands(normalized_workflow_runs),
            [],
        )
        self.assertNotIn(
            "python -m unittest tools.tests.test_frameworks_05_layer_direction -v",
            normalized_workflow_runs,
        )

    def test_duplicate_cargo_guard_recognizes_optional_toolchain_prefix(self) -> None:
        unpinned_commands = "\n".join(
            (
                "cargo fmt --all --check",
                "cargo clippy -p zircon_app --all-targets --no-deps",
                "cargo test -p zircon_runtime --lib structure_convention --locked --jobs 1",
            )
        )
        pinned_commands = "\n".join(
            (
                "cargo +1.94.1 fmt --all --check",
                "cargo +1.94.1 clippy -p zircon_app --all-targets --no-deps --jobs 1",
                "cargo +1.94.1 test -p zircon_runtime --lib structure_convention --locked --jobs 1",
            )
        )

        expected = ["fmt", "clippy", "structure_convention"]
        self.assertEqual(self._copied_owned_cargo_commands(unpinned_commands), expected)
        self.assertEqual(self._copied_owned_cargo_commands(pinned_commands), expected)

    @staticmethod
    def _copied_owned_cargo_commands(source: str) -> list[str]:
        patterns = (
            ("fmt", r"\bcargo(?:\s+\+\S+)?\s+fmt\b"),
            ("clippy", r"\bcargo(?:\s+\+\S+)?\s+clippy\b"),
            (
                "structure_convention",
                r"\bcargo(?:\s+\+\S+)?\s+test\b[^\r\n]*\bstructure_convention\b",
            ),
        )
        return [name for name, pattern in patterns if re.search(pattern, source)]

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

    def test_rule_guard_audit_accepts_current_development_conventions(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]

        report = audit_rule_guard_coverage(repo_root)

        expected_rule_ids = [
            *(f"GEN-S{index}" for index in range(1, 7)),
            *(f"GEN-M{index}" for index in range(1, 5)),
            *(f"GEN-Q{index}" for index in range(1, 8)),
            *(f"GEN-T{index}" for index in range(1, 5)),
            *(f"GEN-D{index}" for index in range(1, 5)),
            *(f"RT-{index}" for index in range(1, 11)),
            *(f"ED-{index}" for index in range(1, 9)),
            *(f"PL-{index}" for index in range(1, 12)),
            *(f"IF-{index}" for index in range(1, 5)),
            *(f"WF-{index}" for index in range(1, 6)),
        ]
        should_rule_ids = {
            "GEN-S6",
            "GEN-M4",
            "GEN-Q5",
            "GEN-Q6",
            "GEN-T4",
            "GEN-D4",
            "RT-9",
            "RT-10",
            "ED-7",
            "ED-8",
            "PL-10",
            "PL-11",
            "IF-4",
            "WF-5",
        }

        self.assertEqual(report["rule_ids"], expected_rule_ids)
        self.assertEqual(
            report["must_rule_ids"],
            [rule_id for rule_id in expected_rule_ids if rule_id not in should_rule_ids],
        )
        self.assertEqual(report["rule_count"], 63)
        self.assertEqual(report["must_rule_count"], 49)
        self.assertEqual(report["violation_count"], 0)
        self.assertEqual(report["violations"], [])

    def test_rule_guard_audit_rejects_missing_guard_and_duplicate_rule_id(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            convention = (
                repo_root
                / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
            )
            convention.parent.mkdir(parents=True)
            convention.write_text(
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | first rule | |\n"
                "| GEN-S1 | MUST | duplicate rule | G1 |\n"
                "| GEN-S2 | SHOULD | advisory rule | |\n",
                encoding="utf-8",
            )

            report = audit_rule_guard_coverage(repo_root)

            self.assertEqual(report["rule_count"], 3)
            self.assertEqual(report["must_rule_count"], 2)
            self.assertEqual(report["violation_count"], 2)
            self.assertEqual(
                [(item["rule_id"], item["reason"]) for item in report["violations"]],
                [
                    ("GEN-S1", "missing guard"),
                    ("GEN-S1", "duplicate rule id"),
                ],
            )

    def test_rule_guard_audit_rejects_malformed_rows_and_unknown_guards(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            convention = (
                repo_root
                / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
            )
            convention.parent.mkdir(parents=True)
            convention.write_text(
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | valid rule | G1 |\n"
                "| GEN-S2 | MUST | missing guard column |\n"
                "| GEN-S3 | MUST | misspelled guard | G8 |\n"
                "table ended\n"
                "| GEN-S4 | MUST | outside recognized table | G1 |\n",
                encoding="utf-8",
            )

            report = audit_rule_guard_coverage(repo_root)

            self.assertEqual(report["rule_ids"], ["GEN-S1", "GEN-S3"])
            self.assertEqual(report["must_rule_ids"], ["GEN-S1", "GEN-S3"])
            self.assertEqual(
                [(item["line"], item["rule_id"], item["reason"]) for item in report["violations"]],
                [
                    (4, "<malformed>", "malformed rule row"),
                    (5, "GEN-S3", "unknown guard"),
                    (7, "<malformed>", "rule row outside recognized table"),
                ],
            )

    def test_rule_guard_audit_ignores_unrelated_uppercase_tables(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            convention = (
                repo_root
                / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
            )
            convention.parent.mkdir(parents=True)
            convention.write_text(
                "| Gate | State |\n"
                "|---|---|\n"
                "| CI | required |\n"
                "| G1 | active |\n"
                "\n"
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | valid rule | G1 |\n"
                "table ended\n"
                "| GEN-S2 | MUST | outside recognized table | G1 |\n",
                encoding="utf-8",
            )

            report = audit_rule_guard_coverage(repo_root)

            self.assertEqual(report["rule_ids"], ["GEN-S1"])
            self.assertEqual(
                [(item["line"], item["reason"]) for item in report["violations"]],
                [(10, "rule row outside recognized table")],
            )

    def test_rule_guard_audit_rejects_unknown_nonempty_should_guard(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            convention = (
                repo_root
                / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
            )
            convention.parent.mkdir(parents=True)
            convention.write_text(
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | SHOULD | empty advisory guard remains valid | |\n"
                "| GEN-S2 | SHOULD | misspelled advisory guard | G8 |\n",
                encoding="utf-8",
            )

            report = audit_rule_guard_coverage(repo_root)

            self.assertEqual(report["rule_ids"], ["GEN-S1", "GEN-S2"])
            self.assertEqual(report["must_rule_count"], 0)
            self.assertEqual(
                [(item["rule_id"], item["reason"]) for item in report["violations"]],
                [("GEN-S2", "unknown guard")],
            )

    def test_rule_guard_audit_requires_separator_immediately_after_header(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            convention = (
                repo_root
                / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
            )
            convention.parent.mkdir(parents=True)
            convention.write_text(
                "| ID | 级别 | 规则 | 守卫 |\n"
                "| GEN-S1 | MUST | row without separator | G1 |\n",
                encoding="utf-8",
            )

            report = audit_rule_guard_coverage(repo_root)

            self.assertIn(
                (2, "missing rule table separator"),
                [(item["line"], item["reason"]) for item in report["violations"]],
            )

    def test_rule_guard_audit_rejects_repeated_table_markers(self) -> None:
        malformed_sequences = {
            "repeated header": (
                "| ID | 级别 | 规则 | 守卫 |\n"
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | valid rule | G1 |\n"
            ),
            "repeated leading separator": (
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | valid rule | G1 |\n"
            ),
            "separator after data row": (
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | first rule | G1 |\n"
                "|---|---|---|---|\n"
                "| GEN-S2 | MUST | second rule | G1 |\n"
            ),
            "header after data row": (
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | first rule | G1 |\n"
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S2 | MUST | second rule | G1 |\n"
            ),
        }
        for label, contents in malformed_sequences.items():
            with self.subTest(label=label), tempfile.TemporaryDirectory() as temporary_directory:
                repo_root = Path(temporary_directory)
                convention = (
                    repo_root
                    / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
                )
                convention.parent.mkdir(parents=True)
                convention.write_text(contents, encoding="utf-8")

                report = audit_rule_guard_coverage(repo_root)

                self.assertGreater(report["violation_count"], 0)
                self.assertTrue(
                    any(
                        item["reason"]
                        in {
                            "missing rule table separator",
                            "unexpected rule table header",
                            "unexpected rule table separator",
                        }
                        for item in report["violations"]
                    )
                )

    def test_rule_guard_audit_rejects_empty_rule_text(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            convention = (
                repo_root
                / "docs/plans/zircon_runtime/frameworks/development-conventions.md"
            )
            convention.parent.mkdir(parents=True)
            convention.write_text(
                "| ID | 级别 | 规则 | 守卫 |\n"
                "|---|---|---|---|\n"
                "| GEN-S1 | MUST | | G1 |\n",
                encoding="utf-8",
            )

            report = audit_rule_guard_coverage(repo_root)

            self.assertEqual(
                [(item["rule_id"], item["reason"]) for item in report["violations"]],
                [("GEN-S1", "empty rule text")],
            )

    def test_powershell_wrapper_exposes_all_owned_gates(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        wrapper = (repo_root / "tools" / "check-conventions.ps1").read_text(
            encoding="utf-8-sig"
        )

        self.assertIn(
            "[ValidateSet('docs', 'guards', 'layering', 'structure', 'fmt', 'clippy')]",
            wrapper,
        )
        self.assertEqual(self._copied_owned_cargo_commands(wrapper), [])
        self.assertNotIn("test_frameworks_05_layer_direction", wrapper)
        self.assertRegex(
            wrapper,
            r"(?ms)& python @arguments\s+exit \$LASTEXITCODE\s*\Z",
        )

    def test_powershell_multi_gate_docs_use_one_array_argument(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        module_docs = (
            repo_root / "docs" / "cli-and-tooling" / "check-conventions.md"
        ).read_text(encoding="utf-8-sig")

        self.assertNotIn("`-Only <gate>` 可选择单门并可重复指定", module_docs)
        self.assertIn("`-Only` 接受一个 PowerShell 数组", module_docs)
        self.assertIn(
            "pwsh -NoProfile -Command \"& './tools/check-conventions.ps1' "
            "-Only structure,fmt -DryRun -Json\"",
            module_docs,
        )

    def test_json_mode_captures_child_output_and_propagates_failure(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]

        def completed_child(argv: tuple[str, ...], **kwargs: object) -> subprocess.CompletedProcess[str]:
            self.assertTrue(kwargs["capture_output"])
            self.assertTrue(kwargs["text"])
            return subprocess.CompletedProcess(
                argv,
                17,
                stdout="child stdout\n",
                stderr="child stderr\n",
            )

        with patch("tools.check_conventions.subprocess.run", side_effect=completed_child):
            report = run_conventions(
                repo_root,
                ["structure"],
                dry_run=False,
                capture_output=True,
            )

        self.assertFalse(report["passed"])
        self.assertEqual(report["commands"][0]["exit_code"], 17)
        self.assertEqual(report["commands"][0]["stdout"], "child stdout\n")
        self.assertEqual(report["commands"][0]["stderr"], "child stderr\n")

        output = io.StringIO()
        with (
            patch("tools.check_conventions.subprocess.run", side_effect=completed_child),
            patch.object(
                sys,
                "argv",
                ["check_conventions.py", "--only", "structure", "--json"],
            ),
            redirect_stdout(output),
        ):
            exit_code = main()

        self.assertEqual(exit_code, 1)
        payload = json.loads(output.getvalue())
        self.assertFalse(payload["passed"])
        self.assertEqual(payload["commands"][0]["exit_code"], 17)
        self.assertEqual(payload["commands"][0]["stdout"], "child stdout\n")

    def test_json_mode_captures_subprocess_launch_failure(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        launch_error = FileNotFoundError(2, "missing executable", "cargo")

        with patch("tools.check_conventions.subprocess.run", side_effect=launch_error):
            report = run_conventions(
                repo_root,
                ["structure"],
                dry_run=False,
                capture_output=True,
            )

        self.assertFalse(report["passed"])
        self.assertIsNone(report["commands"][0]["exit_code"])
        self.assertEqual(report["commands"][0]["launch_error"]["kind"], "FileNotFoundError")
        self.assertIn("missing executable", report["commands"][0]["launch_error"]["message"])

        output = io.StringIO()
        with (
            patch("tools.check_conventions.subprocess.run", side_effect=launch_error),
            patch.object(
                sys,
                "argv",
                ["check_conventions.py", "--only", "structure", "--json"],
            ),
            redirect_stdout(output),
        ):
            exit_code = main()

        self.assertEqual(exit_code, 1)
        payload = json.loads(output.getvalue())
        self.assertFalse(payload["passed"])
        self.assertEqual(
            payload["commands"][0]["launch_error"]["kind"], "FileNotFoundError"
        )

    def test_convention_command_plan_is_stable_and_scoped(self) -> None:
        commands = convention_commands()

        self.assertEqual(
            commands,
            [
                ConventionCommand(
                    "layering",
                    (
                        "python",
                        "-m",
                        "unittest",
                        "tools.tests.test_frameworks_05_layer_direction",
                        "-v",
                    ),
                ),
                ConventionCommand(
                    "structure",
                    (
                        "cargo",
                        "+1.94.1",
                        "test",
                        "-p",
                        "zircon_runtime",
                        "--lib",
                        "structure_convention",
                        "--locked",
                        "--jobs",
                        "1",
                    ),
                ),
                ConventionCommand(
                    "fmt", ("cargo", "+1.94.1", "fmt", "--all", "--check")
                ),
                ConventionCommand(
                    "clippy",
                    (
                        "cargo",
                        "+1.94.1",
                        "clippy",
                        "-p",
                        "zircon_runtime_interface",
                        "-p",
                        "zircon_app",
                        "--all-targets",
                        "--no-deps",
                        "--locked",
                        "--jobs",
                        "1",
                        "--",
                        "-D",
                        "warnings",
                    ),
                ),
            ],
        )


if __name__ == "__main__":
    unittest.main()
