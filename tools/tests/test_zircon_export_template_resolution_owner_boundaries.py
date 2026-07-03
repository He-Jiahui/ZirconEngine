import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXPORT_TEMPLATE = REPO_ROOT / "tools/zircon_export/export_template.py"
EXPORT_TEMPLATE_RESOLUTION = (
    REPO_ROOT / "tools/zircon_export/export_template_resolution.py"
)
PLATFORM_BUNDLE = REPO_ROOT / "tools/zircon_export/platform_bundle.py"


class ZirconExportTemplateResolutionOwnerBoundaryTests(unittest.TestCase):
    def test_export_template_resolution_lives_in_resolution_owner(self):
        self.assertTrue(
            EXPORT_TEMPLATE_RESOLUTION.exists(),
            "ExportTemplate template-root resolution needs a dedicated owner",
        )
        template_text = EXPORT_TEMPLATE.read_text(encoding="utf-8")
        resolution_text = EXPORT_TEMPLATE_RESOLUTION.read_text(encoding="utf-8")
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")

        for function_name in (
            "resolve_export_template_from_root",
            "read_template_manifest_for_resolution",
            "template_manifest_matches_resolution",
            "template_resolution_candidate",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                template_text,
                f"{function_name} belongs in the export-template resolution owner",
            )
            self.assertIn(f"def {function_name}(", resolution_text)

        self.assertIn(
            "from .export_template_resolution import",
            platform_bundle_text,
            "PlatformBundle should consume template-root resolution directly",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            resolution_text,
            "template resolution owner must not import PlatformBundle orchestration",
        )

    def test_export_template_validation_owner_stays_under_split_threshold(self):
        line_count = len(EXPORT_TEMPLATE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            420,
            "export_template validation owner should stay below the split threshold",
        )


if __name__ == "__main__":
    unittest.main()
