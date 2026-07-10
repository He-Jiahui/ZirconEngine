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
        "active session notes": (
            repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8"),
    }


def _payload_case(
    name: str,
    slug: str,
    owner_file: str,
    owner_phrase: str,
    detail_phrase: str,
) -> dict[str, object]:
    status_sections = ["09 export status", "13 standalone status", "active session notes"]
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


NATIVE_DYNAMIC_PAYLOAD_DOC_STATUS_CASES = [
    _payload_case(
        "test_current_plugin_docs_reflect_native_dynamic_payload_file_manifest_schema_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_file_manifest_schema_owner_split",
        "pipeline_report_native_dynamic_payload_file_manifest_schema.py",
        "NativeDynamic payload file_manifest schema owner",
        "file_manifest row schema diagnostics",
    ),
    _payload_case(
        "test_current_plugin_docs_reflect_native_dynamic_payload_materialized_packages_schema_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_materialized_packages_schema_owner_split",
        "pipeline_report_native_dynamic_payload_materialized_packages_schema.py",
        "NativeDynamic payload materialized_packages schema owner",
        "materialized_packages row schema diagnostics",
    ),
    _payload_case(
        "test_current_plugin_docs_reflect_native_dynamic_payload_package_path_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_package_path_owner_split",
        "pipeline_report_native_dynamic_payload_package_path.py",
        "NativeDynamic payload package path owner",
        "package path and package_report diagnostics",
    ),
    _payload_case(
        "test_current_plugin_docs_reflect_native_dynamic_payload_bundle_evidence_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_bundle_evidence_owner_split",
        "pipeline_report_native_dynamic_payload_bundle_evidence.py",
        "NativeDynamic payload bundle evidence owner",
        "current bundle file/hash/count and loadable-artifact diagnostics",
    ),
    _payload_case(
        "test_current_export_plan_reflects_native_dynamic_payload_file_manifest_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_file_manifest_owner_split",
        "native_dynamic_payload_file_manifest.py",
        "NativeDynamic payload file manifest owner",
        "NativeDynamic payload file manifest/path/hash diagnostics",
    ),
    _payload_case(
        "test_current_export_plan_reflects_native_dynamic_payload_loader_manifest_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_loader_manifest_owner_split",
        "pipeline_report_native_dynamic_payload_loader_manifest.py",
        "NativeDynamic payload loader manifest owner",
        "PlatformBundle NativeDynamic payload loader-manifest diagnostics",
    ),
    _payload_case(
        "test_current_export_plan_reflects_native_dynamic_payload_platform_bundle_handoff_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_platform_bundle_handoff_owner_split",
        "pipeline_report_native_dynamic_payload_platform_bundle.py",
        "NativeDynamic payload PlatformBundle handoff owner",
        "PlatformBundle NativeDynamic payload handoff diagnostics",
    ),
    _payload_case(
        "test_current_export_plan_reflects_native_dynamic_payload_platform_bundle_stage_report_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_platform_bundle_stage_report_owner_split",
        "pipeline_report_native_dynamic_payload_platform_bundle_stage.py",
        "NativeDynamic payload PlatformBundle stage-report handoff owner",
        "stage_report path/source handoff diagnostics",
    ),
    _payload_case(
        "test_current_export_plan_reflects_native_dynamic_payload_operation_audit_summary_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_operation_audit_summary_owner_split",
        "native_dynamic_payload_operation_audit.py",
        "NativeDynamic payload operation-audit summary owner",
        "operation-audit summary/consistency",
    ),
    _payload_case(
        "test_current_export_plan_reflects_native_dynamic_payload_directory_owner_split",
        "plugins_13_m5_t1_native_dynamic_payload_directory_owner_split",
        "native_dynamic_payload_directory.py",
        "NativeDynamic payload directory owner",
        "directory-backed payload summary diagnostics",
    ),
]


class PluginDocsCurrentStatusNativeDynamicPayloadOwnerSplitsTests(unittest.TestCase):
    def test_current_native_dynamic_payload_docs_reflect_owner_split_status_rows(
        self,
    ):
        sections = _current_doc_sections(Path(__file__).resolve().parents[2])
        failures: list[str] = []
        for case in NATIVE_DYNAMIC_PAYLOAD_DOC_STATUS_CASES:
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
                "Current export/plugin docs do not reflect NativeDynamic "
                "payload owner splits:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
