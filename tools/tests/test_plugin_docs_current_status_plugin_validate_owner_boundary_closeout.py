import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusPluginValidateOwnerBoundaryCloseoutTests(
    unittest.TestCase
):
    def test_current_status_records_plugin_validate_owner_boundary_closeout(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_13_m5_t1_plugin_validate_owner_boundary_file_closeout"

        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
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
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "Plugins 09 status": _tail_section(
                plan_09_text, "## 状态与产出记录"
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
            "tools/tests/test_plugin_validate_owner_boundaries.py=68",
            "tools/tests/test_plugin_validate_options_dependency_owner_boundaries.py=236",
            "tools/tests/test_plugin_validate_distribution_test_owner_boundaries.py=87",
            "test_options_required_capability_gates_lives_in_options_owner",
            "test_distribution_contract_tests_live_in_distribution_contract_test_owner",
            "general PluginValidate owner boundary file should stay as a thin common-owner guard",
            "remaining large-file debt closed",
            "python -m unittest tools.tests.test_plugin_validate_owner_boundaries tools.tests.test_plugin_validate_options_dependency_owner_boundaries tools.tests.test_plugin_validate_distribution_test_owner_boundaries tools.tests.test_plugin_validate_test_owner_boundaries",
            "17/17",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the PluginValidate "
                "owner-boundary closeout:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
