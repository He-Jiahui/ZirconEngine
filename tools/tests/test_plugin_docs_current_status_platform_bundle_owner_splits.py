import unittest
from pathlib import Path

from tools.tests.plugin_docs_current_status_platform_bundle_support import (
    assert_required_phrases,
    load_platform_bundle_sections,
)


class PluginDocsCurrentStatusPlatformBundleMaterializationOwnerTests(
    unittest.TestCase
):
    def setUp(self) -> None:
        self.sections = load_platform_bundle_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_platform_bundle_materialize_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_platform_bundle_materialize_owner_split",
                    "platform_bundle_materialize.py",
                    "PlatformBundle materialization owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_platform_bundle_materialize_owner_split",
                    "platform_bundle_materialize.py",
                    "PlatformBundle materialization owner",
                ],
                "export tooling docs": [
                    "platform_bundle_materialize.py",
                    "PlatformBundle materialization owner",
                    "bundle copy/cleanup helpers",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_platform_bundle_materialize_owner_split",
                    "platform_bundle_materialize.py",
                    "PlatformBundle materialization owner",
                ],
            },
            "Current export/plugin docs do not reflect PlatformBundle materialize owner split",
        )

    def test_current_export_plan_reflects_platform_bundle_native_plugins_payload_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_platform_bundle_native_plugins_payload_owner_split",
                    "platform_bundle_native_plugins_payload.py",
                    "PlatformBundle native plugins payload owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_platform_bundle_native_plugins_payload_owner_split",
                    "platform_bundle_native_plugins_payload.py",
                    "PlatformBundle native plugins payload owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_platform_bundle_native_plugins_payload_owner_split",
                    "platform_bundle_native_plugins_payload.py",
                    "native_plugins_payload bundle-path rewriting",
                ],
                "export tooling docs": [
                    "platform_bundle_native_plugins_payload.py",
                    "PlatformBundle native plugins payload owner",
                    "native_plugins_payload bundle-path rewriting",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_platform_bundle_native_plugins_payload_owner_split",
                    "platform_bundle_native_plugins_payload.py",
                    "PlatformBundle native plugins payload owner",
                ],
            },
            "Current export/plugin docs do not reflect PlatformBundle native plugins payload owner split",
        )

    def test_current_plugin_docs_reflect_platform_bundle_native_plugins_materialize_owner_split(
        self,
    ):
        slug = "plugins_13_m5_t1_platform_bundle_native_plugins_materialize_owner_split"
        owner_file = "platform_bundle_native_plugins_materialize.py"
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    slug,
                    owner_file,
                    "PlatformBundle native plugins materialize owner",
                ],
                "13 standalone status": [
                    slug,
                    owner_file,
                    "PlatformBundle native plugins materialize owner",
                ],
                "standalone current contract": [
                    slug,
                    owner_file,
                    "native plugins directory overwrite and recursive copy",
                ],
                "export tooling docs": [
                    owner_file,
                    "PlatformBundle native plugins materialize owner",
                    "native plugins directory overwrite and recursive copy",
                ],
                "active session notes": [
                    slug,
                    owner_file,
                    "PlatformBundle native plugins materialize owner",
                ],
            },
            "Current plugin docs do not reflect PlatformBundle native plugins materialize owner split",
        )


if __name__ == "__main__":
    unittest.main()
