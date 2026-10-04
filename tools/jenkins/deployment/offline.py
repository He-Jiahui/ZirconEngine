from __future__ import annotations
from pathlib import Path
from .plugins import PluginMaintenance, read_stopped_operation

def install_staged(home: Path, stage: Path, manifest: dict, *, stopped_proof: dict | None = None) -> dict:
    # Legacy callers may pass a receipt, but it is validated against its
    # persisted deployment journal.  A caller-created boolean/dict is never
    # accepted as native termination evidence.
    proof = stopped_proof if isinstance(stopped_proof, dict) else {}
    operation_id = proof.get("operationId")
    state_dir = Path(home).parent / "state"
    read_stopped_operation(state_dir, operation_id, expected_home=home)
    return PluginMaintenance(home, state_dir).install(stage, manifest, operation_id=operation_id)

def rollback(home: Path, backup: Path) -> dict:
    """Compatibility wrapper; all mutation goes through the locked executor."""
    home = Path(home)
    state_dir = home.parent / "state"
    backup = Path(backup).absolute()
    operation_id = backup.name
    expected_backup = (home / "plugin-backups" / operation_id).absolute()
    if backup != expected_backup:
        from .spec import JenkinsError
        raise JenkinsError("plugin_backup_home_mismatch", "plugin backup is outside the operation's Jenkins Home")
    # This read is deliberate: it rejects a missing, running, forged, or stale
    # operation before PluginMaintenance is allowed to touch the Home.
    read_stopped_operation(state_dir, operation_id, expected_home=home)
    return PluginMaintenance(home, state_dir).rollback(backup, operation_id=operation_id)

def stopped_proof_from_operation(state_dir: Path, operation_id: str) -> dict:
    """Read proof emitted by the native deployment manager; callers cannot mint it."""
    return read_stopped_operation(state_dir, operation_id)
