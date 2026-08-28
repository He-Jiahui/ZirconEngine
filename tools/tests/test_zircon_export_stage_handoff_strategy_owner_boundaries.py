import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STAGE_HANDOFF = REPO_ROOT / "tools/zircon_export/stage_handoff.py"
STAGE_HANDOFF_STRATEGY = (
    REPO_ROOT / "tools/zircon_export/stage_handoff_strategy.py"
)
SOURCE_TEMPLATE_ORCHESTRATOR = REPO_ROOT / "tools/zircon_export/source_template.py"
STRATEGY_CONSUMERS = (
    REPO_ROOT / "tools/zircon_export/compile_host_plan.py",
    REPO_ROOT / "tools/zircon_export/cook_assets.py",
    REPO_ROOT / "tools/zircon_export/native_dynamic_plan.py",
    REPO_ROOT / "tools/zircon_export/pack_stage.py",
    REPO_ROOT / "tools/zircon_export/pipeline_report.py",
    REPO_ROOT / "tools/zircon_export/pipeline_stages.py",
    REPO_ROOT / "tools/zircon_export/platform_bundle_strategy_handoff.py",
    REPO_ROOT / "tools/zircon_export/source_template_plan_command.py",
)


class ZirconExportStageHandoffStrategyOwnerBoundaryTests(unittest.TestCase):
    def test_stage_handoff_strategy_lives_in_dedicated_owner(self):
        self.assertTrue(
            STAGE_HANDOFF_STRATEGY.exists(),
            "Validate strategy handoff rules need a dedicated owner",
        )
        stage_handoff_text = STAGE_HANDOFF.read_text(encoding="utf-8")
        strategy_text = STAGE_HANDOFF_STRATEGY.read_text(encoding="utf-8")

        for function_name in (
            "validate_report_requires_bundle_strategy_diagnostic",
            "validate_report_requires_bundle_strategy_diagnostics",
            "native_dynamic_payload_allowed",
            "export_strategy_diagnostics",
            "export_strategy_list_is_invalid",
            "export_strategy_list_is_empty",
            "export_strategies_from_validate_report",
            "unsupported_export_strategies_from_validate_report",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_handoff_text,
                f"{function_name} belongs in the stage handoff strategy owner",
            )
            self.assertIn(f"def {function_name}(", strategy_text)

        self.assertNotIn(
            "normalize_export_strategy",
            stage_handoff_text,
            "generic stage report handoff should not import strategy normalization",
        )
        self.assertIn("normalize_export_strategy", strategy_text)
        self.assertIn(
            "from .stage_handoff import",
            strategy_text,
            "strategy owner may consume generic stage report handoff primitives",
        )
        self.assertNotIn(
            "from .pipeline_report import",
            strategy_text,
            "strategy owner must not import final Report orchestration",
        )

    def test_strategy_consumers_import_strategy_owner_directly(self):
        for consumer in STRATEGY_CONSUMERS:
            text = consumer.read_text(encoding="utf-8")
            self.assertIn(
                "from .stage_handoff_strategy import",
                text,
                f"{consumer.name} should consume strategy helpers directly",
            )

    def test_source_template_orchestrator_delegates_strategy_planning(self):
        text = SOURCE_TEMPLATE_ORCHESTRATOR.read_text(encoding="utf-8")
        self.assertIn("from .source_template_plan_command import", text)
        self.assertNotIn("from .stage_handoff_strategy import", text)

    def test_stage_handoff_owners_stay_under_large_file_thresholds(self):
        stage_handoff_lines = len(STAGE_HANDOFF.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            stage_handoff_lines,
            400,
            "generic stage handoff owner should stay below 400 lines after strategy split",
        )
        self.assertTrue(
            STAGE_HANDOFF_STRATEGY.exists(),
            "Validate strategy handoff rules need a dedicated owner",
        )
        strategy_lines = len(
            STAGE_HANDOFF_STRATEGY.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            strategy_lines,
            220,
            "stage handoff strategy owner should stay below 220 lines",
        )


if __name__ == "__main__":
    unittest.main()
