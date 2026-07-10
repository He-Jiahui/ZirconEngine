import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path, strip_resolved_output_archives


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


class PluginDocsCurrentStatusConvergenceTests(unittest.TestCase):
    def test_current_plugin_authority_docs_reflect_validate_all_and_no_stale_rollout_pending(self):
        repo_root = Path(__file__).resolve().parents[2]

        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        structure_section = _section(
            structure_text,
            "### §6.6 双形态独立构建",
            "### 范式：插件 crate 骨架化",
        )

        plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_relationship_section = _section(
            plan_text,
            "## 8. 与既有计划的关系",
            "## 9. 审查和验收记录",
        )

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_status_block = _section(
            standalone_text,
            "> 状态：",
            "## 1. 设计原则",
        )
        standalone_current_contract_section = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        stale_phrases_by_section = {
            "engine-code-structure-convention §6.6": [
                "M5/T1 全量双形态 rollout 仍未关闭",
                "剩余 §6.6 工作是 feature-provider 独立包物化设计",
                "feature-provider 独立包物化和更广 editor/export/full regression 仍待后续长窗口",
            ],
            "Plugins 13 §8": [
                "三十一项",
                "M5/T1 full dual-form rollout 对其他非 dist 插件族仍未关闭",
                "M5/T1 full dual-form rollout 与 dist-capable 扩容仍未关闭",
            ],
            "plugin-standalone-build current carrier": [
                "未迁移、未声明 forms 的旧清单暂保留 crate-type 回退",
                "仅无 forms 的 legacy manifest 使用 crate-type 回退",
                "并保留无 forms legacy manifest 的 crate-type 回退",
            ],
        }
        sections = {
            "engine-code-structure-convention §6.6": structure_section,
            "Plugins 13 §8": plan_relationship_section,
            "plugin-standalone-build status": standalone_status_block,
            "plugin-standalone-build current carrier": standalone_current_contract_section,
        }

        failures: list[str] = []
        for section_name, stale_phrases in stale_phrases_by_section.items():
            section = sections[section_name]
            for phrase in stale_phrases:
                if phrase in strip_resolved_output_archives(section):
                    failures.append(f"{section_name}: {phrase}")

        required_phrases_by_section = {
            "engine-code-structure-convention §6.6": [
                "plugin validate --all",
                "target_count = 39",
                "failed_count = 0",
            ],
            "Plugins 13 §8": [
                "37 个 root dist-capable plugin + 2 个 Sound feature provider",
                "plugin validate --all",
                "runtime registration builder module-call guard hardening",
                "runtime_registration_builder_violation_count = 0",
            ],
            "plugin-standalone-build status": [
                "plugin validate --all",
                "target_count = 39",
                "failed_count = 0",
                "runtime registration builder module-call guard hardening",
                "runtime_registration_builder_violation_count = 0",
                "zircon_build distribution forms hard cutover",
            ],
            "plugin-standalone-build current carrier": [
                "distribution forms hard cutover",
                "不再从 Cargo `crate-type` 回退推测 carrier",
                "test_zircon_build_rejects_plugin_manifest_missing_distribution_forms",
            ],
        }
        for section_name, required_phrases in required_phrases_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin authority docs have stale or missing validation status:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
