import unittest
from pathlib import Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusPlatformBundleNativePayloadLoaderManifestTestSplitTests(
    unittest.TestCase
):
    def test_current_status_records_platform_bundle_native_payload_loader_manifest_test_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_09_e1_platform_bundle_native_payload_loader_manifest_test_owner_split"
        )

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
            "tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest.py=710",
            "tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest_abi_schema.py=268",
            "tools/zircon_export/tests/platform_bundle_native_payload_loader_manifest_test_support.py=29",
            "test_abi_schema_tests_have_dedicated_owner",
            "test_native_payload_refresh_fixture_lives_in_shared_support",
            "test_native_payload_loader_manifest_test_owners_stay_small",
            "python -m unittest tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest_abi_schema",
            "23/23",
            "python -m unittest tools.tests.test_zircon_export_platform_bundle_native_payload_loader_manifest_test_owner_boundaries",
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
                "Current plugin status docs do not record the PlatformBundle "
                "native payload loader manifest test split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
