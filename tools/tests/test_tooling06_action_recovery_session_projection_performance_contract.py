from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ACTIONS = (
    ROOT
    / "tools"
    / "session_coordinator"
    / "control_plane"
    / "actions"
    / "service.py"
)


def test_action_recovery_projects_session_id_in_sql() -> None:
    source = ACTIONS.read_text(encoding="utf-8")
    recovery = source.split("    def recover_interrupted_actions", 1)[1].split(
        "    def complete_deferred", 1
    )[0]

    assert "json_extract(parameters_json, '$.sessionId') AS session_id" in recovery
    assert "json.loads" not in recovery
