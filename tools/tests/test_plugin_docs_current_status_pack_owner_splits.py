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


PACK_DOC_STATUS_CASES = [
    {
        "name": "test_current_export_plan_reflects_pack_stage_owner_split",
        "message": "Current export/plugin docs do not reflect Pack stage owner split",
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_stage_owner_split",
                "pack_stage.py",
                "Pack stage owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_stage_owner_split",
                "pack_stage.py",
                "Pack stage owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_stage_owner_split",
                "pack_stage.py",
                "Pack stage owner",
            ],
            "export tooling docs": [
                "pack_stage.py",
                "Pack stage owner",
                "pack command/report/path helpers",
            ],
        },
    },
    {
        "name": "test_current_export_plan_reflects_pack_stage_required_fields_owner_split",
        "message": (
            "Current export/plugin docs do not reflect Pack stage "
            "required-fields owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_stage_required_fields_owner_split",
                "pipeline_report_pack_stage_required_fields.py",
                "Pack stage required-fields owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_stage_required_fields_owner_split",
                "pipeline_report_pack_stage_required_fields.py",
                "Pack stage required-fields owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_stage_required_fields_owner_split",
                "pipeline_report_pack_stage_required_fields.py",
                "Pack stage required-fields owner",
            ],
            "export tooling docs": [
                "pipeline_report_pack_stage_required_fields.py",
                "Pack stage required-fields owner",
                "non-fatal and delta-publication required-field diagnostics",
            ],
        },
    },
    {
        "name": "test_current_export_plan_reflects_pack_file_evidence_owner_split",
        "message": (
            "Current export/plugin docs do not reflect Pack file evidence owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_file_evidence_owner_split",
                "pipeline_report_pack_file_evidence.py",
                "Pack file evidence owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_file_evidence_owner_split",
                "pipeline_report_pack_file_evidence.py",
                "Pack file evidence owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_file_evidence_owner_split",
                "pipeline_report_pack_file_evidence.py",
                "pack file and binary evidence diagnostics",
            ],
            "export tooling docs": [
                "pipeline_report_pack_file_evidence.py",
                "Pack file evidence owner",
                "pack file and binary evidence diagnostics",
            ],
        },
    },
    {
        "name": "test_current_export_plan_reflects_pack_manifest_schema_helper_owner_split",
        "message": (
            "Current export/plugin docs do not reflect pack manifest schema "
            "helper owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_manifest_schema_helper_owner_split",
                "pipeline_report_pack_manifest_schema_helpers.py",
                "Pack manifest schema helper owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_manifest_schema_helper_owner_split",
                "pipeline_report_pack_manifest_schema_helpers.py",
                "Pack manifest schema helper owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_manifest_schema_helper_owner_split",
                "pipeline_report_pack_manifest_schema_helpers.py",
                "Pack manifest reusable row/path/hash diagnostics",
            ],
            "export tooling docs": [
                "pipeline_report_pack_manifest_schema_helpers.py",
                "Pack manifest schema helper owner",
                "Pack manifest reusable row/path/hash diagnostics",
            ],
        },
    },
    {
        "name": "test_current_export_plan_reflects_pack_delta_semantics_owner_split",
        "message": (
            "Current export/plugin docs do not reflect pack delta semantic "
            "diagnostics owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_delta_semantics_owner_split",
                "pipeline_report_pack_delta_semantics.py",
                "Pack delta semantic diagnostics owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_delta_semantics_owner_split",
                "pipeline_report_pack_delta_semantics.py",
                "Pack delta semantic diagnostics owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_delta_semantics_owner_split",
                "pipeline_report_pack_delta_semantics.py",
                "Pack delta report count/target/asset-set diagnostics",
            ],
            "export tooling docs": [
                "pipeline_report_pack_delta_semantics.py",
                "pipeline_report_pack_delta_asset_set_semantics.py",
                "Pack delta semantic diagnostics owner",
                "Pack delta report publication/count/target diagnostics",
                "Pack delta asset-set semantics owner",
            ],
        },
    },
    {
        "name": "test_current_export_plan_reflects_pack_manifest_path_hash_schema_helper_owner_split",
        "message": (
            "Current export/plugin docs do not reflect Pack manifest path/hash "
            "schema helper owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_manifest_path_hash_schema_helper_owner_split",
                "pipeline_report_pack_manifest_path_hash_schema_helpers.py",
                "Pack manifest path/hash schema helper owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_manifest_path_hash_schema_helper_owner_split",
                "pipeline_report_pack_manifest_path_hash_schema_helpers.py",
                "Pack manifest path/hash schema helper owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_manifest_path_hash_schema_helper_owner_split",
                "pipeline_report_pack_manifest_path_hash_schema_helpers.py",
                "path/hash schema diagnostics",
            ],
            "export tooling docs": [
                "pipeline_report_pack_manifest_path_hash_schema_helpers.py",
                "Pack manifest path/hash schema helper owner",
                "path/hash schema diagnostics",
            ],
        },
    },
    {
        "name": "test_current_plugin_docs_reflect_pack_stage_path_owner_split",
        "message": (
            "Current plugin docs do not reflect Pack stage path/argument "
            "owner split"
        ),
        "required_by_section": {
            "09 export status": [
                "plugins_13_m5_t1_pack_stage_path_owner_split",
                "pack_stage_paths.py",
                "Pack stage path/argument owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_pack_stage_path_owner_split",
                "pack_stage_paths.py",
                "Pack stage path/argument owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_pack_stage_path_owner_split",
                "pack_stage_paths.py",
                "Pack path and argument preflight",
            ],
            "export tooling docs": [
                "pack_stage_paths.py",
                "Pack stage path/argument owner",
                "path and argument preflight",
            ],
        },
    },
]


class PluginDocsCurrentStatusPackOwnerSplitsTests(unittest.TestCase):
    def test_current_pack_docs_reflect_owner_split_status_rows(self):
        sections = _current_doc_sections(Path(__file__).resolve().parents[2])
        failures: list[str] = []
        for case in PACK_DOC_STATUS_CASES:
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
