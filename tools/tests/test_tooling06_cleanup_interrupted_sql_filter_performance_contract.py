from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DELETION = ROOT / "tools" / "session_coordinator" / "cleanup_deletion.py"


def test_interrupted_cleanup_filters_terminal_deletions_in_one_sql_scan() -> None:
    source = DELETION.read_text(encoding="utf-8")
    interrupted = source.split("def interrupted_target_deletions", 1)[1].split(
        "def complete_interrupted_target_deletion", 1
    )[0]

    assert "WITH deletion_events AS" in interrupted
    assert "terminal AS" in interrupted
    assert "json_extract" in interrupted
    assert "NOT EXISTS" not in interrupted
    assert "completed: set[str]" not in interrupted
