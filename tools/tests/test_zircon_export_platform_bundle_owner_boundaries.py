import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE = REPO_ROOT / "tools/zircon_export/platform_bundle.py"
PLATFORM_BUNDLE_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_materialize.py"
)
PLATFORM_BUNDLE_NATIVE_PLUGINS_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_native_plugins_payload.py"
)


class ZirconExportPlatformBundleOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_materialization_lives_in_materialize_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_MATERIALIZE.exists(),
            "PlatformBundle materialization and copy helpers need a dedicated owner",
        )
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        materialize_text = PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8")

        for function_name in (
            "materialize_platform_bundle",
            "copy_platform_bundle_file",
            "remove_platform_bundle_dir",
            "template_bundle_root",
            "template_bundle_output_path",
            "template_bundle_manifest_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                platform_bundle_text,
                f"{function_name} belongs in the materialization owner",
            )
            self.assertIn(f"def {function_name}(", materialize_text)

        self.assertIn(
            "from .platform_bundle_materialize import",
            platform_bundle_text,
            "PlatformBundle orchestration should consume the materialization owner",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            materialize_text,
            "materialization owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_native_plugins_payload_lives_in_payload_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_NATIVE_PLUGINS_PAYLOAD.exists(),
            "PlatformBundle native plugins payload rewriting needs a dedicated owner",
        )
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        materialize_text = PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8")
        payload_text = PLATFORM_BUNDLE_NATIVE_PLUGINS_PAYLOAD.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "native_plugins_payload_for_bundle",
            "native_plugins_package_for_bundle",
            "native_plugins_relative_payload_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                platform_bundle_text,
                f"{function_name} belongs in the native plugins payload owner",
            )
            self.assertNotIn(
                f"def {function_name}(",
                materialize_text,
                f"{function_name} should not stay in the generic materialization owner",
            )
            self.assertIn(f"def {function_name}(", payload_text)

        self.assertIn(
            "from .platform_bundle_native_plugins_payload import",
            platform_bundle_text,
            "PlatformBundle orchestration should consume native payload rewriting directly",
        )
        self.assertNotIn(
            "NATIVE_DYNAMIC_LOADER_MANIFEST",
            materialize_text,
            "loader manifest naming belongs with native plugins payload rewriting",
        )
        self.assertIn("NATIVE_DYNAMIC_LOADER_MANIFEST", payload_text)
        self.assertNotIn(
            "from .platform_bundle_materialize import",
            payload_text,
            "native payload owner must not import materialization orchestration",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            payload_text,
            "native payload owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_orchestration_stays_under_large_file_threshold(self):
        line_count = len(PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            700,
            "PlatformBundle orchestration should stay below 700 lines after materialization split",
        )

    def test_platform_bundle_materialization_stays_under_large_file_threshold(self):
        self.assertTrue(
            PLATFORM_BUNDLE_NATIVE_PLUGINS_PAYLOAD.exists(),
            "PlatformBundle native plugins payload owner should exist before line-budget checks",
        )
        materialize_line_count = len(
            PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8").splitlines()
        )
        payload_line_count = len(
            PLATFORM_BUNDLE_NATIVE_PLUGINS_PAYLOAD.read_text(
                encoding="utf-8"
            ).splitlines()
        )
        self.assertLess(
            materialize_line_count,
            440,
            "PlatformBundle materialization owner should stay below 440 lines",
        )
        self.assertLess(
            payload_line_count,
            140,
            "PlatformBundle native plugins payload owner should stay below 140 lines",
        )


if __name__ == "__main__":
    unittest.main()
