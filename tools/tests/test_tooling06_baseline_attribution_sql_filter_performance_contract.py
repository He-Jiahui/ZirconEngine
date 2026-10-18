from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
BASELINES = ROOT / "tools" / "session_coordinator" / "baselines.py"


def test_baseline_attribution_filters_epoch_and_session_status_in_sql() -> None:
    source = BASELINES.read_text(encoding="utf-8")
    unattributed = source.split("    def _unattributed_changes", 1)[1].split(
        "    def _mark_degraded", 1
    )[0]

    assert "attributions.baseline_epoch = ?" in unattributed
    assert "sessions.status NOT IN" in unattributed
    assert 'if row["baseline_epoch"] is not None' not in unattributed
