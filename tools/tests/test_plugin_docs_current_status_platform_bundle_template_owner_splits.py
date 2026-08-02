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


def _template_case(
    name: str,
    slug: str,
    owner_file: str,
    owner_phrase: str,
    detail_phrase: str,
) -> dict[str, object]:
    status_sections = ["09 export status", "13 standalone status"]
    required_by_section = {
        section: [slug, owner_file, owner_phrase] for section in status_sections
    }
    required_by_section["standalone current contract"] = [
        slug,
        owner_file,
        detail_phrase,
    ]
    required_by_section["export tooling docs"] = [
        owner_file,
        owner_phrase,
        detail_phrase,
    ]
    return {"name": name, "required_by_section": required_by_section}


PLATFORM_BUNDLE_TEMPLATE_DOC_STATUS_CASES = [
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_resolution_row_schema_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_resolution_row_schema_owner_split",
        "pipeline_report_platform_bundle_template_resolution_row_schema.py",
        "PlatformBundle template resolution row schema owner",
        "candidate and skipped-candidate row schema diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_manifest_identity_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_manifest_identity_owner_split",
        "pipeline_report_platform_bundle_template_manifest_identity.py",
        "PlatformBundle template manifest identity owner",
        "manifest/report identity diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_report_semantics_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_report_semantics_owner_split",
        "pipeline_report_platform_bundle_template_report_semantics.py",
        "PlatformBundle template report semantics owner",
        "template report semantics diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_resolution_path_semantics_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_resolution_path_semantics_owner_split",
        "pipeline_report_platform_bundle_template_resolution_path_semantics.py",
        "PlatformBundle template resolution path semantics owner",
        "template_dir/template_root path containment diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_copied_files_schema_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_copied_files_schema_owner_split",
        "pipeline_report_platform_bundle_template_copied_files_schema.py",
        "PlatformBundle template copied-files schema owner",
        "copied template_files[] schema diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_bundle_files_schema_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_bundle_files_schema_owner_split",
        "pipeline_report_platform_bundle_template_bundle_files_schema.py",
        "PlatformBundle template bundle/files schema owner",
        "embedded template bundle/files row schema diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_resolution_candidate_semantics_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_resolution_candidate_semantics_owner_split",
        "pipeline_report_platform_bundle_template_resolution_candidate_semantics.py",
        "PlatformBundle template resolution candidate semantics owner",
        "candidate profile/identity/bundle-format semantics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_schema_path_helper_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_schema_path_helper_owner_split",
        "pipeline_report_platform_bundle_template_path_schema_helpers.py",
        "PlatformBundle template path/hash schema helper owner",
        "path/hash schema diagnostics",
    ),
    _template_case(
        "test_current_export_plan_reflects_platform_bundle_template_manifest_files_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_manifest_files_owner_split",
        "pipeline_report_platform_bundle_template_manifest_files_schema.py",
        "PlatformBundle template manifest files schema owner",
        "[[files]] schema/presence/unique diagnostics",
    ),
    _template_case(
        "test_current_plugin_docs_reflect_platform_bundle_template_files_materialize_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_files_materialize_owner_split",
        "platform_bundle_template_files_materialize.py",
        "PlatformBundle template files materialize owner",
        "template file copy and native plugins overwrite filtering",
    ),
    _template_case(
        "test_current_plugin_docs_reflect_platform_bundle_template_resolution_failure_semantics_owner_split",
        "plugins_13_m5_t1_platform_bundle_template_resolution_failure_semantics_owner_split",
        "pipeline_report_platform_bundle_template_resolution_failure_semantics.py",
        "PlatformBundle template resolution failure semantics owner",
        "fatal no-match/root-failure/multiple-match diagnostics",
    ),
]


class PluginDocsCurrentStatusPlatformBundleTemplateOwnerSplitsTests(
    unittest.TestCase
):
    def test_current_platform_bundle_template_docs_reflect_owner_split_status_rows(
        self,
    ):
        sections = _current_doc_sections(Path(__file__).resolve().parents[2])
        failures: list[str] = []
        for case in PLATFORM_BUNDLE_TEMPLATE_DOC_STATUS_CASES:
            with self.subTest(case=case["name"]):
                case_failures: list[str] = []
                required_by_section = case["required_by_section"]
                assert isinstance(required_by_section, dict)
                for section_name, required_phrases in required_by_section.items():
                    section = sections[str(section_name)]
                    for phrase in required_phrases:
                        if phrase not in section:
                            case_failures.append(f"{section_name}: missing {phrase}")
                failures.extend(
                    f"{case['name']}: {failure}" for failure in case_failures
                )

        if failures:
            self.fail(
                "Current export/plugin docs do not reflect PlatformBundle "
                "template owner splits:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
