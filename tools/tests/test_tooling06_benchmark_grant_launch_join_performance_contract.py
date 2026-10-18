from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GRANTS = ROOT / "tools" / "session_coordinator" / "benchmark_validation_grants.py"


def test_benchmark_launch_loads_grant_identity_in_one_join() -> None:
    source = GRANTS.read_text(encoding="utf-8")
    launch = source.split("def require_benchmark_launch_grant", 1)[1].split(
        "@dataclass", 1
    )[0]

    assert "AS source_plan_path" in launch
    assert "AS target_plan_path" in launch
    assert "AS workflow_session_id" in launch
    assert "SELECT plan_path FROM sessions" not in launch
    assert "SELECT session_id FROM workflow_runs" not in launch
