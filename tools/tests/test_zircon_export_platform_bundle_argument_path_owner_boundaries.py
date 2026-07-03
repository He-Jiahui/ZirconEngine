import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE = REPO_ROOT / "tools/zircon_export/platform_bundle.py"
PLATFORM_BUNDLE_ARGUMENTS = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_arguments.py"
)


class ZirconExportPlatformBundleArgumentPathOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_argument_path_helpers_live_in_argument_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_ARGUMENTS.exists(),
            "PlatformBundle argument/path resolution needs a dedicated owner",
        )
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        argument_text = PLATFORM_BUNDLE_ARGUMENTS.read_text(encoding="utf-8")

        for function_name in (
            "host_source_origin_from_args",
            "pack_source_origin",
            "delta_pack_source_origin",
            "platform_bundle_argument_diagnostics",
            "resolve_optional_platform_bundle_path_argument",
            "resolve_platform_bundle_path",
            "resolve_repo_root",
            "default_repo_root",
            "resolve_user_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                platform_bundle_text,
                f"{function_name} belongs in the PlatformBundle argument/path owner",
            )
            self.assertIn(f"def {function_name}(", argument_text)

        self.assertIn(
            "from .platform_bundle_arguments import",
            platform_bundle_text,
            "PlatformBundle orchestration should consume the argument/path owner",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            argument_text,
            "argument/path owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_orchestration_stays_under_argument_split_threshold(self):
        line_count = len(PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            540,
            "PlatformBundle orchestration should stay below 540 lines after argument/path split",
        )


if __name__ == "__main__":
    unittest.main()
