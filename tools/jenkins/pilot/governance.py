"""Private physical storage ownership, independent of the retired coordinator."""
from __future__ import annotations

import argparse
import json
import os
import secrets
import subprocess
import sys
import time
from pathlib import Path

from .contracts import PilotError, canonical_json, identifier
from .native.process_identity import popen_process_creation_time, process_creation_time, process_matches_creation_time
from .native.paths import WorkerStorage
from .storage import ManagedStorage, physical_identity, require_managed_root

OWNER_SCHEMA = 2
OWNER_FILE = "storage-owner.json"
DEFAULT_STORAGE_PARENT = Path(r"D:\cargo-targets\zircon-jenkins")


def _state_path(repo_root: Path, state_file: Path) -> Path:
    permitted = (repo_root / ".codex/state/jenkins-pilot").resolve()
    value = state_file.resolve()
    if not value.is_relative_to(permitted) or value == permitted:
        raise PilotError("storage owner metadata must stay in the private Jenkins state directory")
    return value


def _write_state(path: Path, value: dict[str, object]) -> None:
    temporary = path.with_name(f".{path.name}.{secrets.token_hex(8)}.tmp")
    try:
        with temporary.open("xb") as stream:
            stream.write(canonical_json(value))
            stream.flush()
            os.fsync(stream.fileno())
        deadline = time.monotonic() + 10
        while True:
            try:
                os.replace(temporary, path)
                return
            except PermissionError as error:
                if getattr(error, "winerror", None) != 32 or time.monotonic() >= deadline:
                    raise
                time.sleep(0.1)
    finally:
        temporary.unlink(missing_ok=True)


def require_live_storage_owner(root: Path, repo_root: Path) -> dict[str, object]:
    """Match physical root, durable authority and a fresh exact native owner."""
    root = require_managed_root(root, allow_root=False)
    with WorkerStorage(root, create=False) as backend:
        raw = backend.read_bytes(OWNER_FILE, max_bytes=65536)
        owner = json.loads(raw)
        if not isinstance(owner, dict) or owner.get("schemaVersion") != OWNER_SCHEMA or owner.get("status") != "active":
            raise PilotError("storage has no current independent owner authority")
        identifier(owner.get("authorityId"), "authorityId")
        if owner.get("root") != str(root) or owner.get("rootIdentity") != physical_identity(root):
            raise PilotError("independent storage root identity changed")
        state_file = _state_path(repo_root, Path(owner["stateFile"]))
        state = json.loads(state_file.read_bytes())
        if state.get("owner") != owner or state.get("root") != str(root):
            raise PilotError("storage owner metadata does not match its authority")
        pid, birth = owner.get("owner_pid"), owner.get("owner_process_creation_time")
        if type(pid) is not int or pid <= 0 or not isinstance(birth, str) or not birth.isdecimal():
            raise PilotError("storage owner has no exact native identity")
        if not process_matches_creation_time(pid, birth):
            raise PilotError("independent storage owner is no longer live; preserve the existing root")
        if owner.get("recoveryProof"):
            from .governance_recovery import preservation_chain
            preservation_chain(backend, owner)
        if backend.read_bytes(OWNER_FILE, max_bytes=65536) != raw:
            raise PilotError("independent storage authority changed during verification")
    return owner


def allocate_storage(repo_root: Path, state_file: Path, prefix: str,
                     *, storage_parent: Path = DEFAULT_STORAGE_PARENT) -> dict[str, object]:
    """Allocate once; reconcile the same pending identity on delayed startup."""
    repo_root = repo_root.resolve()
    state_file = _state_path(repo_root, state_file)
    identifier(prefix, "storage prefix")
    if not prefix.startswith("jenkins-pilot-"):
        raise PilotError("independent storage prefix must start with jenkins-pilot-")
    if state_file.exists():
        state = json.loads(state_file.read_bytes())
        if state.get("status") == "failed":
            raise PilotError("independent owner startup failed: " + str(state.get("error")))
        current_owner = require_live_storage_owner(Path(state["root"]), repo_root)
        if current_owner != state.get("owner") or current_owner.get("stateFile") != str(state_file):
            raise PilotError("storage metadata belongs to a superseded authority; preserve its original state")
        return state
    state_file.parent.mkdir(parents=True, exist_ok=True)
    parent = require_managed_root(storage_parent, allow_root=False)
    pending_file = state_file.with_suffix(".pending")
    reservation = {"schemaVersion": OWNER_SCHEMA, "prefix": prefix, "status": "reserved",
                   "authorityId": secrets.token_hex(16)}
    reservation["root"] = str(parent / (prefix + "-" + reservation["authorityId"]))
    try:
        with pending_file.open("xb") as stream:
            stream.write(canonical_json(reservation))
            stream.flush()
            os.fsync(stream.fileno())
        dispatch = True
    except FileExistsError:
        dispatch = False
    if dispatch:
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONUTF8="1", PYTHONPATH=str(repo_root))
        log = state_file.with_suffix(".launcher.log").open("xb", buffering=0)
        process = subprocess.Popen(
            [sys.executable, "-B", "-m", "tools.jenkins.pilot.governance", "--repo-root", str(repo_root),
             "--state-file", str(state_file), "--prefix", prefix, "--root", reservation["root"],
             "--authority-id", reservation["authorityId"]], cwd=repo_root, env=env,
            stdin=subprocess.DEVNULL, stdout=log, stderr=log,
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
        log.close()
        try:
            reservation.update(pid=process.pid, creationTime=popen_process_creation_time(process))
            _write_state(pending_file, reservation)
        except BaseException:
            process.kill()
            process.wait(timeout=15)
            raise
    else:
        reservation = json.loads(pending_file.read_bytes())
        if reservation.get("prefix") != prefix or not reservation.get("pid") or not reservation.get("creationTime"):
            raise PilotError("allocation is uncertain; reconcile the existing owner reservation")
    deadline = time.monotonic() + 180
    while time.monotonic() < deadline:
        if state_file.exists():
            state = json.loads(state_file.read_bytes())
            if state.get("status") == "failed":
                raise PilotError("independent owner startup failed: " + str(state.get("error")))
            owner = require_live_storage_owner(Path(state["root"]), repo_root)
            if owner["owner_pid"] != reservation["pid"] or owner["owner_process_creation_time"] != reservation["creationTime"] or owner["authorityId"] != reservation["authorityId"]:
                raise PilotError("allocation returned a different native owner identity")
            return state
        if not process_matches_creation_time(reservation["pid"], reservation["creationTime"]):
            raise PilotError("independent owner exited before allocation; preserve its pending record")
        time.sleep(0.1)
    raise PilotError("independent allocation remains pending; reconcile the same state file")


def keep_storage(repo_root: Path, state_file: Path, prefix: str, root: Path, authority_id: str) -> None:
    """Keep this private root and fixed native Jobs owned; never reclaim data."""
    repo_root = repo_root.resolve()
    state_file = _state_path(repo_root, state_file)
    identifier(prefix, "storage prefix")
    identifier(authority_id, "authorityId")
    root = require_managed_root(root, allow_root=False)
    reservation = json.loads(state_file.with_suffix(".pending").read_bytes())
    if reservation.get("root") != str(root) or reservation.get("authorityId") != authority_id or reservation.get("prefix") != prefix:
        raise PilotError("keeper arguments do not match the immutable allocation reservation")
    if state_file.exists() or root.exists():
        raise PilotError("independent storage already exists; adoption is forbidden")
    with WorkerStorage(root.parent) as parent:
        parent.create_directory(root.name)
    storage = ManagedStorage(root)
    storage.ensure_layout()
    # Retain physical root and ancestor handles for the complete owner lifetime.
    # The keeper never releases this identity to permit path replacement.
    ownership_pin = WorkerStorage(root, create=False)
    owner = {"schemaVersion": OWNER_SCHEMA, "authorityId": authority_id, "root": str(root),
             "rootIdentity": physical_identity(root), "stateFile": str(state_file),
             "owner_pid": os.getpid(), "owner_process_creation_time": process_creation_time(os.getpid()),
             "status": "active"}
    storage.atomic_write(OWNER_FILE, canonical_json(owner))
    _write_state(state_file, {"schemaVersion": OWNER_SCHEMA, "root": str(root), "owner": owner})
    _serve_owner(repo_root, state_file, root, owner, ownership_pin)


def _serve_owner(repo_root: Path, state_file: Path, root: Path, owner: dict,
                 ownership_pin: WorkerStorage, *, service_intents: bool = True) -> None:
    """Retain authority and native Jobs; durable fatal evidence lives outside root."""
    storage = ManagedStorage(root)
    from .bootstrap import stop as stop_jenkins
    from .execution_dispatch import service_execution_intents
    stop_file = state_file.with_suffix(".stop")
    while True:
        ownership_pin.root_identity
        require_live_storage_owner(root, repo_root)
        try:
            if service_intents:
                service_execution_intents(root, repo_root, owner)
            if stop_file.exists():
                result = stop_jenkins(root)
                _write_state(state_file, {"schemaVersion": OWNER_SCHEMA, "root": str(root), "owner": owner,
                                         "stopped": True, "stopResult": result})
                stop_file.unlink()
        except Exception as error:
            storage.atomic_write("owner-attention.json", canonical_json({
                "schemaVersion": OWNER_SCHEMA, "ownerPid": os.getpid(), "errorType": type(error).__name__,
                "error": str(error), "recordedAtUnix": time.time()}))
        time.sleep(1)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--state-file", type=Path, required=True)
    parser.add_argument("--prefix", required=True)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--authority-id", required=True)
    args = parser.parse_args()
    try:
        keep_storage(args.repo_root, args.state_file, args.prefix, args.root, args.authority_id)
    except BaseException as error:
        state_file = _state_path(args.repo_root.resolve(), args.state_file)
        _write_state(state_file.with_suffix(".fatal.json"), {"schemaVersion": OWNER_SCHEMA,
                     "errorType": type(error).__name__, "error": str(error), "ownerPid": os.getpid(),
                     "recordedAtUnix": time.time()})
        if not state_file.exists():
            _write_state(state_file, {"schemaVersion": OWNER_SCHEMA, "status": "failed",
                                     "error": str(error), "ownerPid": os.getpid()})
        raise
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
