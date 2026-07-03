import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD = REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_payload.py"
PAYLOAD_PLATFORM_BUNDLE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py"
)
PAYLOAD_PLATFORM_BUNDLE_STAGE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle_stage.py"
)
PAYLOAD_PACKAGE_PATH = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_package_path.py"
)
PLATFORM_BUNDLE = REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle.py"
PATH_RESOLVE_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_path_resolve_errors.py"
)


class ZirconExportNativeDynamicPayloadPlatformBundleHandoffOwnerBoundaryTests(
    unittest.TestCase
):
    def test_platform_bundle_payload_handoff_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            PAYLOAD_PLATFORM_BUNDLE.exists(),
            "PlatformBundle NativeDynamic payload handoff diagnostics need a dedicated owner",
        )
        payload_text = PAYLOAD.read_text(encoding="utf-8")
        handoff_text = PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8")
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        path_resolve_test_text = PATH_RESOLVE_TEST.read_text(encoding="utf-8")

        function_name = "platform_bundle_native_plugins_payload_diagnostics"
        self.assertNotIn(
            f"def {function_name}(",
            payload_text,
            f"{function_name} belongs in the PlatformBundle payload handoff owner",
        )
        self.assertIn(f"def {function_name}(", handoff_text)
        self.assertNotIn(
            "def platform_bundle_native_plugins_package_path_diagnostics(",
            handoff_text,
            "package path diagnostics should live in their package path leaf owner",
        )
        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_package_path import",
            handoff_text,
            "payload handoff owner should consume package path diagnostics through the leaf owner",
        )

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_platform_bundle import",
            platform_bundle_text,
            "PlatformBundle final-report diagnostics should consume the payload handoff owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload import",
            platform_bundle_text,
            "PlatformBundle final-report diagnostics must not borrow payload handoff helpers from the payload projection owner",
        )
        self.assertIn(
            "from tools.zircon_export.pipeline_report_native_dynamic_payload_platform_bundle import",
            path_resolve_test_text,
            "path-resolution tests should bind directly to the payload handoff owner",
        )
        self.assertNotIn(
            "platform_bundle_native_plugins_payload_diagnostics,",
            payload_text,
            "payload projection owner must not re-export PlatformBundle payload diagnostics",
        )

    def test_payload_projection_and_handoff_owners_stay_under_budget(self):
        self.assertTrue(
            PAYLOAD_PLATFORM_BUNDLE.exists(),
            "PlatformBundle NativeDynamic payload handoff diagnostics need a dedicated owner",
        )
        payload_lines = len(PAYLOAD.read_text(encoding="utf-8").splitlines())
        handoff_lines = len(
            PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines()
        )

        self.assertLess(
            payload_lines,
            120,
            "NativeDynamic payload projection owner should stay tiny after PlatformBundle handoff split",
        )
        self.assertLess(
            handoff_lines,
            480,
            "PlatformBundle NativeDynamic payload handoff owner should stay below 480 lines",
        )

    def test_stage_report_handoff_lives_in_stage_owner(self):
        self.assertTrue(
            PAYLOAD_PLATFORM_BUNDLE_STAGE.exists(),
            "PlatformBundle NativeDynamic stage-report handoff diagnostics need a dedicated owner",
        )
        handoff_text = PAYLOAD_PLATFORM_BUNDLE.read_text(encoding="utf-8")
        stage_text = PAYLOAD_PLATFORM_BUNDLE_STAGE.read_text(encoding="utf-8")
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        path_resolve_test_text = PATH_RESOLVE_TEST.read_text(encoding="utf-8")

        for function_name in (
            "stage_payload_source_diagnostics",
            "current_output_native_dynamic_report_path",
            "native_dynamic_stage_report_path",
            "platform_bundle_native_plugins_stage_report_handoff",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                handoff_text,
                f"{function_name} belongs in the stage-report handoff owner",
            )
            self.assertIn(f"def {function_name}(", stage_text)

        self.assertNotIn(
            "def _stage_payload_source_diagnostics(",
            handoff_text,
            "private stage payload source diagnostics should not remain in the payload handoff owner",
        )
        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_platform_bundle_stage import",
            handoff_text,
            "payload handoff owner should consume stage-report handoff through the stage owner",
        )
        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_platform_bundle_stage import",
            platform_bundle_text,
            "PlatformBundle final-report owner should import NativeDynamic stage report path from the stage owner",
        )
        self.assertIn(
            "from tools.zircon_export.pipeline_report_native_dynamic_payload_platform_bundle_stage import",
            path_resolve_test_text,
            "path-resolution tests should bind stage path helpers directly to the stage owner",
        )
        for forbidden_import in (
            "from .pipeline_report_native_dynamic_payload_platform_bundle import",
            "from .pipeline_report_native_dynamic_payload import",
            "from .pipeline_report_platform_bundle import",
        ):
            self.assertNotIn(
                forbidden_import,
                stage_text,
                "stage-report handoff owner must stay independent from payload handoff, payload projection, and PlatformBundle owners",
            )
        self.assertLess(
            len(handoff_text.splitlines()),
            390,
            "PlatformBundle NativeDynamic payload handoff owner should shrink after stage-report split",
        )
        self.assertLess(
            len(stage_text.splitlines()),
            220,
            "stage-report handoff owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
