from pathlib import Path


SOURCE = Path("tools/session_coordinator/cargo_run_registration.py")


def test_registration_and_rollback_queries_project_only_consumed_columns() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert source.count(
        '"SELECT session_id, status, pid, command_json, root_process_kind "'
    ) == 1
    assert source.count('"SELECT session_id, status, pid, command_json "') == 1
    assert source.count('"SELECT * FROM cargo_jobs WHERE job_id=?"') == 1
