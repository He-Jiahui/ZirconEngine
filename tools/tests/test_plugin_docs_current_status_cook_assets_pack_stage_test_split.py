import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusCookAssetsPackStageTestSplitTests(unittest.TestCase):
    def test_current_status_records_cook_assets_pack_stage_test_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_09_e1_cook_assets_pack_stage_test_owner_split"

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
            "tools/zircon_export/tests/test_cook_assets_pack_stage.py=817",
            "tools/zircon_export/tests/test_cook_assets_project_fallback.py=827",
            "tools/zircon_export/tests/test_pack_stage_cli.py=241",
            "test_cook_assets_project_fallback_records_direct_res_asset_references",
            "test_pack_delta_args_are_forwarded_to_packer",
            "test_cook_assets_root_keeps_manifest_and_strategy_tests",
            "python -m unittest tools.zircon_export.tests.test_cook_assets_pack_stage tools.zircon_export.tests.test_cook_assets_project_fallback tools.zircon_export.tests.test_pack_stage_cli",
            "40/40",
            "python -m unittest tools.tests.test_zircon_export_cook_assets_pack_stage_test_owner_boundaries",
            "4/4",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the CookAssets/Pack "
                "stage test split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
