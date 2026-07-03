import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_materialize.py"
)
PLATFORM_BUNDLE_TEMPLATE_FILES_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_template_files_materialize.py"
)


class ZirconExportPlatformBundleTemplateFilesMaterializeOwnerBoundaryTests(
    unittest.TestCase
):
    def test_platform_bundle_template_file_copy_lives_in_template_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_TEMPLATE_FILES_MATERIALIZE.exists(),
            "PlatformBundle template file materialization needs a dedicated owner",
        )
        materialize_text = PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8")
        template_owner_text = PLATFORM_BUNDLE_TEMPLATE_FILES_MATERIALIZE.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "materialize_platform_bundle_template_files",
            "template_files_outside_directory",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                materialize_text,
                f"{function_name} belongs in the template file materialize owner",
            )
            self.assertIn(f"def {function_name}(", template_owner_text)

        inline_loop_marker = 'for entry in template_report.get("files", [])'
        self.assertNotIn(
            inline_loop_marker,
            materialize_text,
            "PlatformBundle materialization should not inline template file copy loops",
        )
        self.assertIn(inline_loop_marker, template_owner_text)

        self.assertIn(
            "from .platform_bundle_template_files_materialize import",
            materialize_text,
            "PlatformBundle materialization should consume the template file owner",
        )
        self.assertNotIn(
            "from .platform_bundle_materialize import",
            template_owner_text,
            "template file owner must not import materialization orchestration",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            template_owner_text,
            "template file owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_template_file_owner_line_budgets(self):
        self.assertTrue(
            PLATFORM_BUNDLE_TEMPLATE_FILES_MATERIALIZE.exists(),
            "PlatformBundle template file materialize owner should exist before line-budget checks",
        )
        materialize_line_count = len(
            PLATFORM_BUNDLE_MATERIALIZE.read_text(encoding="utf-8").splitlines()
        )
        template_owner_line_count = len(
            PLATFORM_BUNDLE_TEMPLATE_FILES_MATERIALIZE.read_text(
                encoding="utf-8"
            ).splitlines()
        )
        self.assertLess(
            materialize_line_count,
            360,
            "PlatformBundle materialization owner should stay below 360 lines after template file split",
        )
        self.assertLess(
            template_owner_line_count,
            160,
            "PlatformBundle template file materialize owner should stay below 160 lines",
        )


if __name__ == "__main__":
    unittest.main()
