import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_materialize.py"
)
PLATFORM_BUNDLE_NATIVE_PLUGINS_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_native_plugins_materialize.py"
)


class ZirconExportPlatformBundleNativePluginsMaterializeOwnerBoundaryTests(
    unittest.TestCase
):
    def test_platform_bundle_native_plugins_directory_copy_lives_in_native_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_NATIVE_PLUGINS_MATERIALIZE.exists(),
            "PlatformBundle native plugins directory materialization needs a dedicated owner",
        )
        materialize_text = PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8")
        native_owner_text = PLATFORM_BUNDLE_NATIVE_PLUGINS_MATERIALIZE.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "materialize_platform_bundle_native_plugins",
            "copy_platform_bundle_native_plugins_dir",
            "copy_platform_bundle_native_plugins_file",
            "remove_platform_bundle_native_plugins_destination",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                materialize_text,
                f"{function_name} belongs in the native plugins materialize owner",
            )
            self.assertIn(f"def {function_name}(", native_owner_text)

        for retired_materialize_marker in ("def copy_dir_contents(",):
            self.assertNotIn(
                retired_materialize_marker,
                materialize_text,
                "native plugins directory overwrite/copy logic should not stay in the generic materialization owner",
            )

        for moved_native_owner_marker in (
            'plugins_destination = resolve_bundle_child(bundle_root, "plugins", diagnostics)',
            "native plugins destination ",
        ):
            self.assertNotIn(
                moved_native_owner_marker,
                materialize_text,
                "native plugins directory overwrite/copy logic should not stay in the generic materialization owner",
            )
            self.assertIn(moved_native_owner_marker, native_owner_text)

        self.assertIn(
            "from .platform_bundle_native_plugins_materialize import",
            materialize_text,
            "PlatformBundle materialization should consume the native plugins materialize owner",
        )
        self.assertIn(
            "from .platform_bundle_template_files_materialize import template_files_outside_directory",
            native_owner_text,
            "native plugins materialize owner should reuse the template-file filtering owner",
        )
        self.assertNotIn(
            "from .platform_bundle_materialize import",
            native_owner_text,
            "native plugins materialize owner must not import materialization orchestration",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            native_owner_text,
            "native plugins materialize owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_native_plugins_materialize_owner_line_budgets(self):
        self.assertTrue(
            PLATFORM_BUNDLE_NATIVE_PLUGINS_MATERIALIZE.exists(),
            "PlatformBundle native plugins materialize owner should exist before line-budget checks",
        )
        materialize_line_count = len(
            PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8").splitlines()
        )
        native_owner_line_count = len(
            PLATFORM_BUNDLE_NATIVE_PLUGINS_MATERIALIZE.read_text(
                encoding="utf-8"
            ).splitlines()
        )
        self.assertLess(
            materialize_line_count,
            300,
            "PlatformBundle materialization owner should stay below 300 lines after native plugins split",
        )
        self.assertLess(
            native_owner_line_count,
            150,
            "PlatformBundle native plugins materialize owner should stay below 150 lines",
        )


if __name__ == "__main__":
    unittest.main()
