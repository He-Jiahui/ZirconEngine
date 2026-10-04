from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.ai_effort import AiEffortService


class AiEffortNarrowStreamPerformanceContractTests(unittest.TestCase):
    def test_report_streams_only_aggregate_input_columns(self) -> None:
        source = inspect.getsource(AiEffortService.report)

        self.assertIn(
            "SELECT outcome, cost_class, COUNT(*) AS milestone_count,", source
        )
        self.assertIn("SUM(active_ai_hours) AS active_ai_hours", source)
        self.assertIn("SELECT active_ai_hours, blocked_by_json", source)
        self.assertIn("ORDER BY recorded_at, ledger_id", source)
        self.assertNotIn("SELECT * FROM ai_effort_milestones", source)
        self.assertNotIn("milestone_rows =", source)
        self.assertIn('milestone_count += int(row["milestone_count"])', source)


if __name__ == "__main__":
    unittest.main()
