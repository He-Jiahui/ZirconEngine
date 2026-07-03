import unittest
from pathlib import Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusNativeBuildExecutionTests(unittest.TestCase):
    def test_current_status_records_native_build_execution_owner_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_13_m5_t1_native_build_execution_owner_split"

        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_13_status = _tail_section(
            plan_13_text,
            "## 9. 审查和验收记录",
        )
        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_09_status = _section(
            plan_09_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_status = _tail_section(
            standalone_text,
            "## 9. 当前落地状态",
        )
        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        export_tool_status = export_tool_text[
            export_tool_text.index("`test_zircon_export_native_build_workspace_owner_boundaries.py`") :
        ]
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 13 status": plan_13_status,
            "Plugins 09 status": plan_09_status,
            "standalone current status": standalone_status,
            "export tool docs": export_tool_status,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "native_build_execution.py",
            "NativeBuild execution owner",
            "execute_native_dynamic_build_plan",
            "native_build.py",
            "build-plan assembly",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        stale_phrases = [
            "native_build.py` keeps NativeDynamic build-plan assembly, Cargo execution, artifact copy, sidecar copy, and execution report assembly",
            "native_build.py` keeps NativeDynamic build-plan assembly, Cargo execution, artifact copy, sidecar copy",
            "while `native_build.py` keeps NativeDynamic build-plan assembly, Cargo execution",
        ]
        for section_name, section in sections.items():
            for phrase in stale_phrases:
                if phrase in section:
                    failures.append(f"{section_name}: stale {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record NativeBuild execution owner split:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
