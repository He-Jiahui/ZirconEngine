from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
AI_EFFORT = ROOT / "tools" / "session_coordinator" / "ai_effort.py"


def test_ai_effort_report_aggregates_hours_in_sql() -> None:
    source = AI_EFFORT.read_text(encoding="utf-8")
    report = source.split("    def report", 1)[1].split("    @staticmethod", 1)[0]

    assert "SUM(active_ai_hours)" in report
    assert "COUNT(*) AS milestone_count" in report
    blocker_projection = report.split("WHERE blocked_by_json != '[]'", 1)[1]
    assert "by_outcome[row[\"outcome\"]] += hours" not in blocker_projection
    assert "by_cost_class[row[\"cost_class\"]] += hours" not in blocker_projection
