import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusPluginValidateAssetImporterOwnerBoundaryFileSplitTests(
    unittest.TestCase
):
    def test_current_status_records_plugin_validate_asset_importer_owner_boundary_file_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_13_m5_t1_plugin_validate_asset_importer_owner_boundary_file_split"
        )

        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_09_status = _section(
            plan_09_text, "## 状态与产出记录", "## 5. 里程碑与任务分解"
        )
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_13_status = _tail_section(plan_13_text, "## 9. 审查和验收记录")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_status = _tail_section(standalone_text, "## 9. 当前落地状态")
        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 09 status": plan_09_status,
            "Plugins 13 status": plan_13_status,
            "standalone current status": standalone_status,
            "export tool docs": export_tool_text,
            "structure convention": structure_text,
            "review findings": review_text,
        }
        required_phrases = [
            status_id,
            "tools/tests/test_plugin_validate_owner_boundaries.py=2587",
            "tools/tests/test_plugin_validate_asset_importer_owner_boundaries.py=635",
            "test_asset_importer_boundaries_leave_general_owner_file",
            "test_asset_importer_required_capability_gates_lives_in_gate_owner",
            "test_asset_importer_resource_kinds_lives_in_resource_kind_owner",
            "python -m unittest tools.tests.test_plugin_validate_asset_importer_owner_boundaries",
            "10/10",
            "py_compile",
            "remaining large-file debt",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record PluginValidate "
                "asset-importer owner boundary file split:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
