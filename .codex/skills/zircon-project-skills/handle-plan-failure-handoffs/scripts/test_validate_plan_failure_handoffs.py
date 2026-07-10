from __future__ import annotations

import os
import re
import tempfile
import unittest
from pathlib import Path

from validate_plan_failure_handoffs import parse_handoff_records, validate_repository


REQUIRED_BODY = """# Cross-plan handoff

## 来源执行者

- 来源计划：origin
- 来源执行切片：M1 testing
- 修复责任计划：fixing
- 交接原因：lowest shared owner

## 失败现象与复现证据

Expected pass; observed failure from an exact command.

## 最低共享层根因

Shared catalog constructs the wrong provider identity.

## 架构修复验收

- Lower-layer regression passes.
- Original reproduction passes.
- Upward gate passes.

## 禁止临时方案

- No fallback, alias, shim, or test bypass.

## 修复结果与回传

待修复
"""

FIXED_RESULT = """- 根因：provider identity was constructed from the wrong owner.
- 架构修复：one provider-key constructor now owns definitions and selections.
- 验证：lower regression, original reproduction, and upward gate passed.
- 回传：origin M1 testing gate can resume.
"""


class HandoffFixture:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.origin_plan = root / "docs/plans/zircon_editor/editor/01-editor-kernel.md"
        self.fixing_plan = root / "docs/plans/zircon_runtime/frameworks/02-module-kernel.md"
        self.origin_child = self.origin_plan.parent / "01"
        self.fixing_child = self.fixing_plan.parent / "02"

    @staticmethod
    def _relative(source: Path, target: Path) -> str:
        return Path(os.path.relpath(target, source.parent)).as_posix()

    @staticmethod
    def _write(path: Path, content: str) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def seed(
        self,
        *,
        kind: str = "failure",
        artifact_parent: str | None = None,
        filename: str | None = None,
        omit_key: str | None = None,
        absolute_fixer_link: bool = False,
        same_plan: bool = False,
        unresolved_fixed_body: bool = False,
        empty_fixed_result: bool = False,
        duplicate_with_alternate_path_spelling: bool = False,
        empty_source_executor_section: bool = False,
    ) -> Path:
        if same_plan:
            self.fixing_plan = self.origin_plan
            self.fixing_child = self.origin_child
        parent_name = artifact_parent or ("fixing" if kind == "failure" else "origin")
        parent = self.fixing_child if parent_name == "fixing" else self.origin_child
        artifact_date = "2026-07-11" if kind == "failure" else "2026-07-12"
        default_name = f"{kind}-{artifact_date}-provider-lookup.md"
        artifact = parent / (filename or default_name)

        metadata = {
            "handoff_kind": kind,
            "status": "open" if kind == "failure" else "fixed",
            "created_at": "2026-07-11",
            "summary_slug": "provider-lookup",
            "origin_plan": self.origin_plan.relative_to(self.root).as_posix(),
            "fixing_plan": self.fixing_plan.relative_to(self.root).as_posix(),
            "origin_child_dir": self.origin_child.relative_to(self.root).as_posix(),
            "fixing_child_dir": self.fixing_child.relative_to(self.root).as_posix(),
        }
        if kind == "fixed":
            metadata["resolved_at"] = "2026-07-12"
        if omit_key:
            metadata.pop(omit_key)

        frontmatter = "\n".join(f"{key}: {value}" for key, value in metadata.items())
        body = REQUIRED_BODY
        if kind == "fixed" and not unresolved_fixed_body:
            body = REQUIRED_BODY.replace("待修复", FIXED_RESULT.rstrip())
        if kind == "fixed" and empty_fixed_result:
            body = REQUIRED_BODY.replace(
                "待修复",
                "- 根因：\n- 架构修复：\n- 验证：\n- 回传：",
            )
        body = body.replace(
            "- 来源计划：origin",
            f"- 来源计划：`{metadata.get('origin_plan', '')}`",
        ).replace(
            "- 修复责任计划：fixing",
            f"- 修复责任计划：`{metadata.get('fixing_plan', '')}`",
        )
        if empty_source_executor_section:
            body = re.sub(
                r"## 来源执行者\n.*?\n## 失败现象与复现证据",
                "## 来源执行者\n\n## 失败现象与复现证据",
                body,
                flags=re.DOTALL,
            )
        self._write(artifact, f"---\n{frontmatter}\n---\n\n{body}")

        status = "open 待修复" if kind == "failure" else "fixed 已修复"
        origin_link = self._relative(self.origin_plan, artifact)
        fixing_link = self._relative(self.fixing_plan, artifact)
        if absolute_fixer_link:
            fixing_link = artifact.resolve().as_posix()
        self._write(self.origin_plan, f"# Origin plan\n\n- {status}：[handoff]({origin_link})\n")
        self._write(self.fixing_plan, f"# Fixing plan\n\n- {status}：[handoff]({fixing_link})\n")
        if duplicate_with_alternate_path_spelling:
            duplicate = self.origin_child / "fixed-2026-07-12-provider-lookup.md"
            duplicate_frontmatter = frontmatter.replace(
                f"origin_plan: {metadata['origin_plan']}",
                f"origin_plan: ./{metadata['origin_plan']}",
            ).replace(
                f"fixing_plan: {metadata['fixing_plan']}",
                f"fixing_plan: {metadata['fixing_plan'].replace('/', chr(92))}",
            ).replace("handoff_kind: failure", "handoff_kind: fixed").replace(
                "status: open", "status: fixed"
            )
            duplicate_frontmatter += "\nresolved_at: 2026-07-12"
            fixed_body = REQUIRED_BODY.replace("待修复", FIXED_RESULT.rstrip())
            self._write(duplicate, f"---\n{duplicate_frontmatter}\n---\n\n{fixed_body}")
            origin_duplicate_link = self._relative(self.origin_plan, duplicate)
            fixing_duplicate_link = self._relative(self.fixing_plan, duplicate)
            self._write(
                self.origin_plan,
                self.origin_plan.read_text(encoding="utf-8")
                + f"- fixed 已修复：[returned]({origin_duplicate_link})\n",
            )
            self._write(
                self.fixing_plan,
                self.fixing_plan.read_text(encoding="utf-8")
                + f"- fixed 已修复：[returned]({fixing_duplicate_link})\n",
            )
        return artifact


class ValidatePlanFailureHandoffsTests(unittest.TestCase):
    def validate_fixture(self, configure) -> list[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            configure(HandoffFixture(root))
            return validate_repository(root)

    def test_valid_open_failure(self) -> None:
        errors = self.validate_fixture(lambda fixture: fixture.seed())
        self.assertEqual([], errors)

    def test_exports_structured_records_for_coordinator_import(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            artifact = HandoffFixture(root).seed()

            records, errors = parse_handoff_records(root)

            self.assertEqual([], errors)
            self.assertEqual(1, len(records))
            record = records[0]
            self.assertEqual(artifact.resolve(), record.artifact_path)
            self.assertEqual("failure", record.kind)
            self.assertEqual("provider-lookup", record.summary_slug)
            self.assertEqual("open", record.status)
            self.assertTrue(record.lifecycle_key.endswith("|provider-lookup"))

    def test_valid_returned_fix(self) -> None:
        errors = self.validate_fixture(lambda fixture: fixture.seed(kind="fixed"))
        self.assertEqual([], errors)

    def test_rejects_date_first_failure_name(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(filename="2026-07-11-editor-m1-failure-handoff.md")
        )
        self.assertTrue(any("noncanonical handoff filename" in error for error in errors), errors)

    def test_rejects_malformed_prefix_first_failure_name(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(filename="failure-2026-07-11-Provider-Lookup.md")
        )
        self.assertTrue(any("noncanonical handoff filename" in error for error in errors), errors)

    def test_rejects_missing_origin_provenance(self) -> None:
        errors = self.validate_fixture(lambda fixture: fixture.seed(omit_key="origin_plan"))
        self.assertTrue(any("missing frontmatter key 'origin_plan'" in error for error in errors), errors)

    def test_rejects_missing_fixer_provenance(self) -> None:
        errors = self.validate_fixture(lambda fixture: fixture.seed(omit_key="fixing_plan"))
        self.assertTrue(any("missing frontmatter key 'fixing_plan'" in error for error in errors), errors)

    def test_rejects_empty_source_executor_section(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(empty_source_executor_section=True)
        )
        self.assertTrue(any("source executor field" in error for error in errors), errors)

    def test_rejects_same_plan_handoff(self) -> None:
        errors = self.validate_fixture(lambda fixture: fixture.seed(same_plan=True))
        self.assertTrue(any("must belong to different numbered child plans" in error for error in errors), errors)

    def test_rejects_failure_in_origin_directory(self) -> None:
        errors = self.validate_fixture(lambda fixture: fixture.seed(artifact_parent="origin"))
        self.assertTrue(any("must be stored in fixing_child_dir" in error for error in errors), errors)

    def test_rejects_absolute_fixer_link(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(kind="fixed", absolute_fixer_link=True)
        )
        self.assertTrue(any("relative Markdown link" in error for error in errors), errors)

    def test_rejects_fixed_artifact_left_with_fixer(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(kind="fixed", artifact_parent="fixing")
        )
        self.assertTrue(any("must be returned to origin_child_dir" in error for error in errors), errors)

    def test_rejects_fixed_artifact_with_open_result_content(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(kind="fixed", unresolved_fixed_body=True)
        )
        self.assertTrue(any("still contains open-state marker" in error for error in errors), errors)

    def test_rejects_fixed_artifact_with_empty_result_fields(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(kind="fixed", empty_fixed_result=True)
        )
        self.assertTrue(any("requires a non-empty value" in error for error in errors), errors)

    def test_rejects_duplicate_lifecycle_with_alternate_path_spelling(self) -> None:
        errors = self.validate_fixture(
            lambda fixture: fixture.seed(duplicate_with_alternate_path_spelling=True)
        )
        self.assertTrue(any("duplicate canonical handoff lifecycle" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main(verbosity=2)
