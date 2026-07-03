import unittest
from pathlib import Path


def _section(text: str, start_marker: str, end_marker: str | None = None) -> str:
    start = text.index(start_marker)
    if end_marker is None:
        return text[start:]
    return text[start : text.index(end_marker, start + len(start_marker))]


class PluginDocsCurrentStatusValidateCompileHostCommandValueTests(unittest.TestCase):
    def test_current_docs_reflect_validate_compile_host_command_value_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        slug = (
            "plugins_13_m5_t1_validate_compile_host_command_value_semantics_"
            "owner_split"
        )
        owner_file = "pipeline_report_validate_compile_host_command_value_semantics.py"
        phrase = "Validate CompileHost command value semantics owner"

        export_plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        export_plan_status = _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )

        standalone_plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_status = _section(
            standalone_plan_text,
            "## 9. 审查和验收记录",
        )

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_contract = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "09 export status": export_plan_status,
            "13 standalone status": standalone_status,
            "standalone current contract": standalone_contract,
            "export tooling docs": export_tool_text,
            "active session notes": session_text,
        }
        failures: list[str] = []
        for section_name, section in sections.items():
            for required in (slug, owner_file, phrase):
                if required not in section:
                    failures.append(f"{section_name}: missing {required}")

        if failures:
            self.fail(
                "Current plugin docs do not reflect Validate CompileHost "
                "command value semantics owner split:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
