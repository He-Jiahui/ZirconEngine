from pathlib import Path


SOURCE = Path("tools/session_coordinator/command_requests.py")


def test_terminal_compaction_queries_project_only_consumed_columns() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert source.count(
        "SELECT request_id, status, response_json, error_json\n"
        "                        FROM command_requests"
    ) == 2
    assert "SELECT * FROM command_requests\n                        WHERE retention_class='durable'" not in source
