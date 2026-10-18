from pathlib import Path


SOURCE = Path("tools/session_coordinator/reserved_starts.py")


def test_reserved_start_acknowledgement_projects_only_binding_fields() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert "SELECT session_id, status, job_id, lane_scope, command_fingerprint" in source
    assert "SELECT session_id, status, pid, started_at FROM cargo_jobs" in source
    assert '"SELECT * FROM cargo_lane_reservations WHERE reservation_id=?"' not in source
    assert 'connection.execute("SELECT * FROM cargo_jobs WHERE job_id=?"' not in source
