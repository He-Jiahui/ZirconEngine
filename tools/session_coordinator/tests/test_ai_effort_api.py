from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator import cli
from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.server import CoordinatorApplication, RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


class AiEffortApiTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        root = Path(self.directory.name)
        repo = init_repo(root / "repo")
        self.application = CoordinatorApplication(
            CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
        )
        self.application.supervision.mark_healthy()

    def tearDown(self) -> None:
        self.directory.cleanup()

    def test_report_exposes_user_baseline_without_inferred_milestones(self) -> None:
        report = self.application.command("ai_effort.report", {})["report"]

        self.assertEqual("user-2026-07-15", report["baselineId"])
        self.assertEqual(6005.0, report["historical"]["allEffectiveHours"])
        self.assertFalse(report["historical"]["extrapolationAllowed"])
        self.assertEqual(0, report["ledger"]["milestoneCount"])

    def test_record_is_explicit_idempotent_and_rejects_rewritten_hours(self) -> None:
        payload = {
            "ledger_id": "runtime07-m2-accepted-20260718",
            "plan_id": "docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md",
            "active_ai_hours": 3.5,
            "outcome": "accepted",
            "cost_class": "delivery_design",
            "blocked_by": ["runtime12-input-visibility"],
            "source_session_id": "runtime07-event-bus-delivery-policy-r2-20260718",
        }

        first = self.application.command("ai_effort.record", payload)["record"]
        second = self.application.command("ai_effort.record", payload)["record"]

        self.assertEqual(first, second)
        self.assertEqual(3.5, first["activeAiHours"])
        self.assertEqual(["runtime12-input-visibility"], first["blockedBy"])
        report = self.application.command("ai_effort.report", {})["report"]
        self.assertEqual(3.5, report["ledger"]["acceptedEffectiveHours"])
        with self.assertRaises(CoordinatorError) as rejected:
            self.application.command(
                "ai_effort.record", {**payload, "active_ai_hours": 4.0}
            )
        self.assertEqual("ai_effort_ledger_id_conflict", rejected.exception.code)

    def test_record_allows_an_explicitly_unattributed_quality_cost(self) -> None:
        record = self.application.command(
            "ai_effort.record",
            {
                "ledger_id": "runtime07-m2-red-20260718",
                "plan_id": "docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md",
                "active_ai_hours": 1.25,
                "outcome": "failed",
                "cost_class": "repair_validation",
                "blocked_by": ["text01-font-split"],
            },
        )["record"]

        self.assertIsNone(record["sourceSessionId"])
        report = self.application.command("ai_effort.report", {})["report"]
        self.assertEqual(1.25, report["ledger"]["failedQualityCostHours"])
        self.assertEqual(
            [{"blockedBy": "text01-font-split", "hours": 1.25, "milestones": 1}],
            report["ledger"]["blockedBy"],
        )

    def test_cli_record_forwards_only_explicit_fields(self) -> None:
        arguments = cli._parser().parse_args(
            [
                "ai-effort",
                "record",
                "--ledger-id",
                "runtime07-m2-accepted-20260718",
                "--plan-id",
                "docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md",
                "--active-ai-hours",
                "3.5",
                "--outcome",
                "accepted",
                "--cost-class",
                "delivery_design",
                "--blocked-by",
                "runtime12-input-visibility",
                "--source-session-id",
                "runtime07-event-bus-delivery-policy-r2-20260718",
            ]
        )
        client = mock.Mock()
        client.command.return_value = {"record": {"ledgerId": "runtime07-m2-accepted-20260718"}}

        with mock.patch.object(cli.CoordinatorClient, "from_runtime", return_value=client):
            result = cli._run(arguments)

        self.assertEqual({"record": {"ledgerId": "runtime07-m2-accepted-20260718"}}, result)
        client.command.assert_called_once_with(
            "ai_effort.record",
            {
                "ledger_id": "runtime07-m2-accepted-20260718",
                "plan_id": "docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md",
                "active_ai_hours": 3.5,
                "outcome": "accepted",
                "cost_class": "delivery_design",
                "blocked_by": ["runtime12-input-visibility"],
                "source_session_id": "runtime07-event-bus-delivery-policy-r2-20260718",
            },
        )

    def test_http_command_chain_reports_and_records_without_session_time_inference(self) -> None:
        root = Path(self.directory.name)
        repo = init_repo(root / "http-repo")
        config = CoordinatorConfig.for_repo(repo, state_root=root / "http-state", port=0)

        with RunningCoordinator.start(config):
            client = CoordinatorClient.from_runtime(config)
            report = client.command("ai_effort.report", {})["report"]
            record = client.command(
                "ai_effort.record",
                {
                    "ledger_id": "http-accepted-20260718",
                    "plan_id": "docs/plans/zircon_tooling/session_coordinator/01-plan.md",
                    "active_ai_hours": 2.0,
                    "outcome": "accepted",
                    "cost_class": "delivery_design",
                    "blocked_by": [],
                },
            )["record"]

        self.assertEqual(47.0, report["currentPlanBudget"]["scopeCompletePercent"])
        self.assertEqual(2.0, record["activeAiHours"])
        self.assertIsNone(record["sourceSessionId"])
