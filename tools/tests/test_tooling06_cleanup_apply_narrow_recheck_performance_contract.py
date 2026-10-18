from pathlib import Path


SOURCE = Path("tools/session_coordinator/cleanup.py")


def test_explicit_cleanup_apply_recheck_uses_narrow_job_rows() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert "SELECT job_id, session_id, status, cleanup_status, pid, target_key," in source
    assert "created_at, released_at, finished_at, last_heartbeat_at" in source
    assert "all_jobs = [self.cargo_jobs._from_row(row) for row in rows]" not in source
    assert 'row["status"] in ACTIVE_CARGO_STATUSES' in source
    assert 'parse_utc(row[column])' in source
