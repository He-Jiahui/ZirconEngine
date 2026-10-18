from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SNAPSHOT = ROOT / "tools" / "session_coordinator" / "control_plane" / "snapshot.py"


def test_run_health_aggregates_log_metadata_without_temporary_tuples() -> None:
    source = SNAPSHOT.read_text(encoding="utf-8")
    run_health = source.split("    def _run_health", 1)[1].split(
        "    def _experience", 1
    )[0]

    assert "log_stats = tuple" not in run_health
    assert "output_stats = tuple" not in run_health
    assert "last_output_mtime" in run_health
