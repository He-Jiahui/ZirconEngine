import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _current_doc_sections(repo_root: Path) -> dict[str, str]:
    export_plan_text = (
        repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
    ).read_text(encoding="utf-8")
    standalone_plan_text = (
        repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
    ).read_text(encoding="utf-8")
    standalone_doc_text = (
        repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
    ).read_text(encoding="utf-8")

    return {
        "09 export status": _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        ),
        "13 standalone status": standalone_plan_text[
            standalone_plan_text.index("## 9. 审查和验收记录") :
        ],
        "standalone current contract": _section(
            standalone_doc_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        ),
        "export tooling docs": (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8"),
    }


PLUGIN_BUILD_DOC_STATUS_CASES = [
    {
        "name": "test_current_export_plan_reflects_plugin_build_command_owner_split",
        "message": (
            "Current export/plugin docs do not reflect Plugin build Cargo "
            "command owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_plugin_build_command_owner_split",
                "plugin_build_command.py",
                "Plugin build Cargo command owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_plugin_build_command_owner_split",
                "plugin_build_command.py",
                "Plugin build Cargo command owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_plugin_build_command_owner_split",
                "plugin_build_command.py",
                "Cargo command construction/execution semantics",
            ],
            "export tooling docs": [
                "plugin_build_command.py",
                "Plugin build Cargo command owner",
                "Cargo command construction/execution semantics",
            ],
        },
    },
    {
        "name": "test_current_export_plan_reflects_plugin_build_preflight_owner_split",
        "message": (
            "Current export/plugin docs do not reflect Plugin build preflight "
            "owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_plugin_build_preflight_owner_split",
                "plugin_build_preflight.py",
                "Plugin build preflight owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_plugin_build_preflight_owner_split",
                "plugin_build_preflight.py",
                "Plugin build preflight owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_plugin_build_preflight_owner_split",
                "plugin_build_preflight.py",
                "distribution scalar validation and signing option normalization",
            ],
            "export tooling docs": [
                "plugin_build_preflight.py",
                "Plugin build preflight owner",
                "distribution scalar validation and signing option normalization",
            ],
        },
    },
]


class PluginDocsCurrentStatusPluginBuildOwnerSplitsTests(unittest.TestCase):
    def test_current_plugin_docs_reflect_build_package_owner_layering(self):
        repo_root = Path(__file__).resolve().parents[2]

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        export_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")

        sections = {
            "standalone current contract": _section(
                standalone_text,
                "## 6. 注册跨 ABI 编组",
                "## 9. 当前落地状态",
            ),
            "export plugin build docs": _section(
                export_text,
                "Plugin build signature/hash sidecar assembly is owned",
                "The independent plugin structure audit applies",
            ),
        }

        stale_phrases = [
            "`plugin_build.py` 只消费 `plugin_build_signing_audit(...)`",
            "`plugin_build.py` 只消费 `materialize_plugin_asset_pack(...)`",
            "`plugin_build.py` only consumes the signature owner",
            "`plugin_build.py` only consumes `materialize_plugin_asset_pack(...)`",
        ]
        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in stale_phrases:
                if phrase in section:
                    failures.append(f"{section_name}: stale {phrase}")

        required_by_section = {
            "standalone current contract": [
                "`plugin_build_package.py`",
                "`plugin_build.py` 只消费 `materialize_plugin_build_package(...)`",
            ],
            "export plugin build docs": [
                "`plugin_build_package.py`",
                "`plugin_build.py` build orchestration owner",
            ],
        }
        self._collect_missing_required_phrases(
            sections,
            required_by_section,
            failures,
        )
        if failures:
            self.fail(
                "Current plugin docs do not reflect build package owner layering:\n"
                + "\n".join(failures)
            )

    def test_current_plugin_build_docs_reflect_owner_split_status_rows(self):
        sections = _current_doc_sections(Path(__file__).resolve().parents[2])
        failures: list[str] = []
        for case in PLUGIN_BUILD_DOC_STATUS_CASES:
            with self.subTest(case=case["name"]):
                case_failures: list[str] = []
                self._collect_missing_required_phrases(
                    sections,
                    case["required_by_section"],
                    case_failures,
                )
                if case_failures:
                    failures.append(
                        f"{case['message']}:\n" + "\n".join(case_failures)
                    )
        if failures:
            self.fail("\n".join(failures))

    def _collect_missing_required_phrases(
        self,
        sections: dict[str, str],
        required_by_section: dict[str, list[str]],
        failures: list[str],
    ) -> None:
        for section_name, required_phrases in required_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")


if __name__ == "__main__":
    unittest.main()
