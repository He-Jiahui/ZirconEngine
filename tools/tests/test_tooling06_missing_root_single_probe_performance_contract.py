from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TERMINAL = ROOT / "tools" / "session_coordinator" / "workspace_copy_terminal.py"


def test_missing_root_recovery_uses_one_probe_per_phase() -> None:
    source = TERMINAL.read_text(encoding="utf-8")
    recovery = source.split("    def recover_missing_roots", 1)[1].split(
        "    @staticmethod", 1
    )[0]

    assert recovery.count("_path_exists_or_symlink(") == 2
    assert ".exists() or" not in recovery
    assert ".is_symlink()" not in recovery
