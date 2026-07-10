import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusConvergenceGuardOwnerSplitTests(unittest.TestCase):
    def test_current_status_records_convergence_guard_owner_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_09_e1_current_status_convergence_guard_owner_split"

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
            "tools/tests/test_plugin_docs_current_status_convergence.py=118",
            "tools/tests/test_plugin_docs_current_status_native_dynamic_report_owner_splits.py=890",
            "tools/tests/test_plugin_docs_current_status_native_dynamic_build_owner_splits.py=446",
            "tools/tests/test_plugin_docs_current_status_platform_bundle_owner_splits.py=688",
            "tools/tests/test_plugin_docs_current_status_source_template_compile_host_owner_splits.py=786",
            "tools/tests/test_plugin_docs_current_status_export_template_cook_assets_owner_splits.py=799",
            "test_root_guard_stays_as_authority_status_owner",
            "test_focused_owner_files_exist_and_stay_small",
            "python -m unittest tools.tests.test_plugin_docs_current_status_convergence_owner_boundaries",
            "2/2",
            "python -m unittest tools.tests.test_plugin_docs_current_status_convergence tools.tests.test_plugin_docs_current_status_native_dynamic_report_owner_splits tools.tests.test_plugin_docs_current_status_native_dynamic_build_owner_splits tools.tests.test_plugin_docs_current_status_platform_bundle_owner_splits tools.tests.test_plugin_docs_current_status_source_template_compile_host_owner_splits tools.tests.test_plugin_docs_current_status_export_template_cook_assets_owner_splits",
            "42/42",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the convergence "
                "guard owner split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
