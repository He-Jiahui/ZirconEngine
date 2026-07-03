from __future__ import annotations

import unittest
from pathlib import Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


def load_source_template_compile_host_sections(repo_root: Path) -> dict[str, str]:
    export_plan_text = (
        repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
    ).read_text(encoding="utf-8")
    standalone_plan_text = (
        repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
    ).read_text(encoding="utf-8")
    standalone_text = (
        repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
    ).read_text(encoding="utf-8")
    export_tool_text = (
        repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
    ).read_text(encoding="utf-8")
    session_text = (
        repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
    ).read_text(encoding="utf-8")

    return {
        "09 export status": _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        ),
        "13 standalone status": _tail_section(
            standalone_plan_text,
            "## 9. 审查和验收记录",
        ),
        "standalone current contract": _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        ),
        "export tooling docs": export_tool_text,
        "active session notes": session_text,
    }


def assert_required_phrases(
    test_case: unittest.TestCase,
    sections: dict[str, str],
    required_by_section: dict[str, list[str]],
    message: str,
) -> None:
    failures: list[str] = []
    for section_name, required_phrases in required_by_section.items():
        section = sections[section_name]
        for phrase in required_phrases:
            if phrase not in section:
                failures.append(f"{section_name}: missing {phrase}")

    if failures:
        test_case.fail(message + ":\n" + "\n".join(failures))
