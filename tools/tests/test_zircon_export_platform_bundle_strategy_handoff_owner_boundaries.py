import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE = REPO_ROOT / "tools/zircon_export/platform_bundle.py"
PLATFORM_BUNDLE_STRATEGY_HANDOFF = (
    REPO_ROOT / "tools/zircon_export/platform_bundle_strategy_handoff.py"
)


class ZirconExportPlatformBundleStrategyHandoffOwnerBoundaryTests(unittest.TestCase):
    def test_strategy_handoff_diagnostics_live_in_strategy_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_STRATEGY_HANDOFF.exists(),
            "PlatformBundle Validate strategy handoff needs a dedicated owner",
        )
        platform_bundle_text = PLATFORM_BUNDLE.read_text(encoding="utf-8")
        strategy_handoff_text = PLATFORM_BUNDLE_STRATEGY_HANDOFF.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "platform_bundle_strategy_handoff_diagnostics",
            "validate_report_uses_strategy",
            "validate_report_allows_native_plugins",
            "validate_report_strategy_diagnostics",
            "load_trusted_validate_strategy_report",
            "load_trusted_validate_strategy_report_with_diagnostic",
            "validate_report_strategy_handoff_diagnostic",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                platform_bundle_text,
                f"{function_name} belongs in the strategy handoff owner",
            )
            self.assertIn(f"def {function_name}(", strategy_handoff_text)

        self.assertIn(
            "from .platform_bundle_strategy_handoff import",
            platform_bundle_text,
            "PlatformBundle orchestration should consume the strategy handoff owner",
        )
        self.assertNotIn(
            "from .platform_bundle import",
            strategy_handoff_text,
            "strategy handoff owner must not import PlatformBundle orchestration",
        )

    def test_platform_bundle_orchestration_stays_under_split_threshold(self):
        line_count = len(PLATFORM_BUNDLE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            430,
            "PlatformBundle orchestration should stay below the split threshold",
        )


if __name__ == "__main__":
    unittest.main()
