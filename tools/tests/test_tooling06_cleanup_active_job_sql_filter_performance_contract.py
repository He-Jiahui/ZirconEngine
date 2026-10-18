from pathlib import Path


SOURCE = Path("tools/session_coordinator/cleanup.py")


def test_immediate_cleanup_recheck_filters_active_jobs_in_sql() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert 'active_placeholders = ", ".join("?" for _ in ACTIVE_CARGO_STATUSES)' in source
    assert "WHERE status IN ({active_placeholders})" in source
    assert "rows = [row for row in all_rows if row[\"status\"] in ACTIVE_CARGO_STATUSES]" not in source
    assert "SELECT job_id, session_id, status, cleanup_status, pid, target_key" in source
