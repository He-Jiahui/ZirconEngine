from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HOOK = ROOT / "tools" / "session_coordinator" / "codex_sync" / "hook.py"


def test_codex_hook_reuses_one_process_handle_for_identity_validation() -> None:
    source = HOOK.read_text(encoding="utf-8")
    signal_source = source.split("def signal_coordinator", 1)[1]

    assert "process_matches_creation_time" in signal_source
    assert "process_is_alive(pid)" not in signal_source
    assert "process_creation_time(pid)" not in signal_source
