import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusCompileHostOutputGateTestSplitTests(unittest.TestCase):
    def test_current_status_records_compile_host_output_gate_test_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_09_e1_compile_host_output_gate_command_test_owner_split"

        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 09 status": _tail_section(
                plan_09_text, "## 状态与产出记录"
            ),
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "tools/zircon_export/tests/test_compile_host_output_gate.py=754",
            "tools/zircon_export/tests/test_compile_host_plan_command_semantics.py=859",
            "test_compile_host_rejects_plan_command_metadata_mismatch",
            "test_compile_host_rejects_plan_with_duplicate_target_dir_option",
            "test_compile_host_output_gate_keeps_plan_and_host_output_tests",
            "python -m unittest tools.zircon_export.tests.test_compile_host_output_gate tools.zircon_export.tests.test_compile_host_plan_command_semantics",
            "24/24",
            "python -m unittest tools.tests.test_zircon_export_compile_host_output_gate_test_owner_boundaries",
            "3/3",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the CompileHost "
                "output-gate test split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
