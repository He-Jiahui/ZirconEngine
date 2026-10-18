from pathlib import Path


SOURCE = Path("tools/session_coordinator/cleanup.py")


def test_pressure_eviction_recheck_uses_narrow_safety_and_evidence_rows() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert "SELECT job_id, status, pid, target_key, process_tree_live_pids_json\n                       FROM cargo_jobs ORDER BY created_at, job_id" in source
    assert source.count(
        "SELECT job_id, session_id, status, cleanup_status, pid, target_key"
    ) >= 2
    assert '"SELECT * FROM cargo_jobs ORDER BY created_at, job_id"' not in source
