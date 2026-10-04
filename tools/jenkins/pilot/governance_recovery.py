"""Explicit independent authority recovery, preserving every historic attempt."""
from __future__ import annotations

import argparse
from contextlib import ExitStack, contextmanager
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import time

from .contracts import PilotError, canonical_json, sha256_identity
from .governance import OWNER_FILE, OWNER_SCHEMA, _state_path, _write_state
from .native import process_identity as native_process
from .native.paths import WorkerStorage
from .native.windows_storage import identity_from_stream
from .storage import ManagedStorage, physical_identity, require_managed_root


def _hash(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _require_dead(pid: object, birth: object) -> dict:
    """Classify exact death with native handles; uncertainty always raises."""
    if type(pid) is not int or pid <= 0 or not isinstance(birth, str) or not birth.isdecimal():
        raise PilotError("recovery requires positive PID and exact creation FILETIME")
    api = native_process._kernel()
    handle = native_process._open(api, pid)
    if handle is None:
        return {"pid": pid, "creationTime": birth, "proof": "native-pid-absent"}
    try:
        actual = native_process._birth(api, handle)
        if actual != birth:
            return {"pid": pid, "creationTime": birth, "proof": "native-pid-reused", "observedCreationTime": actual}
        if native_process._alive(api, handle):
            raise PilotError("recovery refuses an exact live owner or validation process")
        return {"pid": pid, "creationTime": birth, "proof": "native-handle-signaled"}
    finally:
        api.CloseHandle(handle)


def _state_bytes(path: Path) -> tuple[bytes, dict]:
    """Read repo metadata and its full native file identity without following links."""
    for ancestor in (path, *path.parents):
        info = ancestor.lstat()
        if getattr(info, "st_file_attributes", 0) & 0x400 or ancestor.is_symlink():
            raise PilotError("recovery state metadata crosses a link or reparse point")
    with path.open("rb") as stream:
        before = identity_from_stream(stream)
        if before.link_count != 1:
            raise PilotError("recovery metadata has a hardlink alias")
        raw = stream.read(1024 * 1024 + 1)
        after = identity_from_stream(stream)
        if len(raw) > 1024 * 1024 or not before.same_file_state(after):
            raise PilotError("recovery metadata changed during inspection")
    return raw, {"device": before.device, "fileId": before.file_id.hex(), "creationTime": before.creation_time}


def _known_processes(value: object, records: list[dict]) -> None:
    if isinstance(value, dict):
        for pid_name, birth_name in (("pid", "creationTime"), ("owner_pid", "owner_process_creation_time"),
                                     ("launcherPid", "launcherCreationTime"), ("parentPid", "parentCreationTime"),
                                     ("childPid", "childCreationTime"), ("keeperPid", "keeperCreationTime")):
            if pid_name in value:
                records.append(_require_dead(value[pid_name], value.get(birth_name)))
        for item in value.values():
            _known_processes(item, records)
    elif isinstance(value, list):
        for item in value:
            _known_processes(item, records)


def _terminal_receipt(receipt: dict) -> None:
    tree, native = receipt.get("processTree", {}), receipt.get("nativeJobEvidence", {})
    records = native.get("records", {})
    if (tree.get("terminal") is not True or tree.get("pipeEOF") is not True
            or tree.get("readersFinished") is not True or native.get("errors") != []
            or receipt.get("terminalErrors") != [] or not records
            or tree.get("scope") != "windows_job"
            or any(type(records.get(phase, {}).get("activeProcesses")) is not int
                   or records[phase]["activeProcesses"] != 0 for phase in ("afterClose", "afterWait"))
            or set(tree.get("pipes", {})) != {"stdout", "stderr"}
            or any(pipe.get("eof") is not True or pipe.get("closed") is not True or "error" in pipe
                                            for pipe in tree["pipes"].values())):
        raise PilotError("recovery execution lacks exact native Job-zero and pipe-EOF terminal proof")


def preservation_chain(backend: WorkerStorage, owner: dict) -> list[dict]:
    """Verify every immutable authority link back to the original allocation."""
    chain, seen = [], set()
    current = owner
    while current.get("recoveryProof"):
        authority = current.get("authorityId")
        if authority in seen:
            raise PilotError("recovered authority preservation chain contains a replay")
        seen.add(authority)
        name = f"authority-history/{authority}/recovery-proof.json"
        if current["recoveryProof"] != name:
            raise PilotError("recovery proof path differs from its exact authority epoch")
        sha256_identity(current.get("recoveryProofSha256"), "recovery proof sha256")
        raw = backend.read_bytes(name, expected_sha256=current.get("recoveryProofSha256"), max_bytes=32 * 1024 * 1024)
        proof = json.loads(raw)
        checkpoint = proof.get("checkpoint")
        if (proof.get("schemaVersion") != 1 or proof.get("authorityId") != authority
                or proof.get("root") != owner["root"] or proof.get("stateFile") != current.get("stateFile")
                or proof.get("recoveredKeeperPid") != current.get("owner_pid")
                or proof.get("recoveredKeeperCreationTime") != current.get("owner_process_creation_time")
                or not isinstance(checkpoint, dict) or _hash(canonical_json(checkpoint)) != proof.get("checkpointSha256")
                or checkpoint.get("rootIdentity") != owner["rootIdentity"]):
            raise PilotError("recovered authority preservation chain changed")
        previous = checkpoint.get("previousOwner")
        if (not isinstance(previous, dict) or previous.get("schemaVersion") != OWNER_SCHEMA
                or previous.get("status") != "active" or previous.get("root") != owner["root"]
                or previous.get("rootIdentity") != owner["rootIdentity"]
                or previous.get("authorityId") != current.get("previousAuthorityId")
                or previous.get("authorityId") in seen
                or previous.get("stateFile") != checkpoint.get("previousStatePath")):
            raise PilotError("recovered authority previous physical owner changed or replayed")
        folder = str(Path(name).parent).replace("\\", "/")
        previous_raw = backend.read_bytes(folder + "/previous-owner.json", max_bytes=65536)
        archived_state = backend.read_bytes(folder + "/previous-state.json", max_bytes=1024 * 1024)
        state_raw, state_identity = _state_bytes(Path(checkpoint["previousStatePath"]))
        state = json.loads(state_raw)
        if (_hash(previous_raw) != checkpoint.get("previousAuthoritySha256") or json.loads(previous_raw) != previous
                or _hash(archived_state) != checkpoint.get("previousStateSha256") or archived_state != state_raw
                or _hash(state_raw) != checkpoint.get("previousStateSha256")
                or state_identity != checkpoint.get("previousStateIdentity")
                or state.get("owner") != previous or state.get("root") != owner["root"]):
            raise PilotError("recovered authority preservation chain changed")
        chain.append(checkpoint)
        current = previous
    if any(field in current for field in ("previousAuthorityId", "recoveryProof", "recoveryProofSha256")):
        raise PilotError("recovered authority preservation chain is incomplete")
    return chain


def inspect_recovery(repo_root: Path, root: Path, *, expected_authority_sha256: str) -> dict:
    """Read-only recovery checkpoint; never clear state, locks or receipts."""
    sha256_identity(expected_authority_sha256, "expected authority sha256")
    root = require_managed_root(root, allow_root=False)
    with WorkerStorage(root, create=False) as backend:
        owner_raw = backend.read_bytes(OWNER_FILE, max_bytes=65536)
        owner = json.loads(owner_raw)
        if (_hash(owner_raw) != expected_authority_sha256 or owner.get("schemaVersion") != OWNER_SCHEMA
                or owner.get("status") != "active" or owner.get("root") != str(root)
                or owner.get("rootIdentity") != physical_identity(root)):
            raise PilotError("recovery authority or physical root differs from the explicit checkpoint")
        death = _require_dead(owner.get("owner_pid"), owner.get("owner_process_creation_time"))
        state_path = _state_path(repo_root, Path(owner["stateFile"]))
        state_raw, state_identity = _state_bytes(state_path)
        state = json.loads(state_raw)
        if state.get("owner") != owner or state.get("root") != str(root):
            raise PilotError("recovery owner and original state metadata disagree")
        chain = preservation_chain(backend, owner)
        records, inventory, completed_intents = [death], {}, {}
        for checkpoint in chain:
            _known_processes(checkpoint, records)
        selected = {"process-state.json", "pilot-manifest.json", "asset-source-manifest.json"}
        for directory in ("execution-intents", "execution-inputs", "runs", "metadata", "tmp"):
            path = root / directory
            if path.exists():
                selected.update(name for name in backend.list_files(directory)
                                if name.endswith(("process.json", "probe.json", "metadata-state.json", "receipt.json"))
                                or directory == "execution-intents" and name.endswith(".json"))
        for name in sorted(selected):
            if not backend.exists(name):
                if name == "process-state.json":
                    continue
                raise PilotError("recovery lacks required resource or intent evidence")
            raw = backend.read_bytes(name, max_bytes=32 * 1024 * 1024)
            value = json.loads(raw)
            inventory[name] = _hash(raw)
            if name.endswith(("process-state.json", "process.json", "probe.json", "metadata-state.json")) or name.startswith("execution-intents/"):
                _known_processes(value, records)
            if name.startswith("execution-intents/"):
                from .execution_dispatch import _validate_intent, _key, _read_receipt
                if value.get("authorityId") != owner["authorityId"]:
                    from .execution_dispatch import _recovered_completed_intent
                    _recovered_completed_intent(ManagedStorage(root), owner, name, raw, value)
                    record = chain[0]["completedIntents"][name]
                    original_owner = next(item["previousOwner"] for item in chain
                                          if item["previousOwner"]["authorityId"] == value["authorityId"])
                    identity = _validate_intent(value, original_owner)
                else:
                    record = None
                    identity = _validate_intent(value, owner)
                if Path(name).stem != _key(identity) or value["status"] != "complete":
                    raise PilotError("recovery will not resubmit pending, reserved or started execution")
                binding = value["binding"]
                receipt = _read_receipt(ManagedStorage(root), identity, binding["job"], binding["buildNumber"], binding)
                if receipt is None:
                    raise PilotError("recovery lacks exact durable terminal execution evidence")
                _terminal_receipt(receipt)
                _known_processes(receipt.get("processTree"), records)
                receipt_name = f"runs/{identity.session_id}/{identity.request_id}/{identity.attempt_id}-g{identity.generation}/receipt.json"
                receipt_raw = backend.read_bytes(receipt_name, max_bytes=32 * 1024 * 1024)
                completed_intents[name] = {"intentSha256": _hash(raw), "previousAuthorityId": value["authorityId"],
                                          "binding": binding, "receiptPath": receipt_name,
                                          "receiptSha256": _hash(receipt_raw), "launch": value["launch"]}
                if record is not None and completed_intents[name] != record:
                    raise PilotError("historical completed execution preservation changed")
                from .snapshot import load_snapshot
                bundle = root / "execution-inputs" / Path(name).stem / "source.zip"
                if load_snapshot(bundle, expected_input_hash=identity.input_hash).bundle_hash != binding["bundleHash"]:
                    raise PilotError("recovery sealed execution source changed")
        generations = {}
        for state_file in sorted((root / "cargo-generations").glob("*/g1/state.json")):
            name = state_file.relative_to(root).as_posix()
            raw = backend.read_bytes(name, max_bytes=1024 * 1024)
            value = json.loads(raw)
            generation = state_file.parent
            if value.get("generationIdentity") != physical_identity(generation) or value.get("rootIdentity") != owner["rootIdentity"]:
                raise PilotError("recovery refuses changed private Cargo generation identity")
            for folder, expected in value.get("physicalIdentities", {}).items():
                if physical_identity(generation / folder) != expected:
                    raise PilotError("recovery refuses changed private Cargo directory identity")
            generations[name] = {"sha256": _hash(raw), "state": value.get("state"),
                                 "generationIdentity": value["generationIdentity"], "physicalIdentities": value["physicalIdentities"]}
        from .assets import verify_prepared_assets
        asset_digest = verify_prepared_assets(root)
        manifest = json.loads(backend.read_bytes("pilot-manifest.json", max_bytes=1024 * 1024))
        from .snapshot import load_snapshot, verify_materialized
        sealed = load_snapshot(Path(manifest["driverBundle"]), expected_input_hash=manifest["driverInputHash"])
        if sealed.bundle_hash != manifest["driverBundleHash"] or verify_materialized(sealed, Path(manifest["driverRoot"])) != manifest["driverInputHash"]:
            raise PilotError("recovery sealed driver resource evidence changed")
        if (backend.read_bytes(OWNER_FILE, max_bytes=65536) != owner_raw
                or _state_bytes(state_path) != (state_raw, state_identity)
                or preservation_chain(backend, owner) != chain):
            raise PilotError("recovery authority or original state changed during verification")
        return {"schemaVersion": 1, "root": str(root), "rootIdentity": owner["rootIdentity"],
                "previousOwner": owner, "previousAuthoritySha256": _hash(owner_raw),
                "previousStatePath": str(state_path), "previousStateSha256": _hash(state_raw),
                "previousStateIdentity": state_identity, "nativeDeathProofs": records,
                "resourceInventory": inventory, "assetProofSha256": asset_digest,
                "generations": generations, "completedIntents": completed_intents}


def recover_storage(repo_root: Path, root: Path, *, expected_authority_sha256: str,
                    state_file: Path) -> dict:
    """Reserve one explicit recovery epoch and reconcile that exact launch only."""
    repo_root = repo_root.resolve()
    state_file = _state_path(repo_root, state_file)
    if state_file.exists() or state_file.with_suffix(".pending").exists():
        raise PilotError("recovery state identity already exists; preserve and reconcile it")
    checkpoint = inspect_recovery(repo_root, root, expected_authority_sha256=expected_authority_sha256)
    if str(state_file) == checkpoint["previousStatePath"]:
        raise PilotError("recovery cannot overwrite the original state metadata")
    authority_id = secrets.token_hex(16)
    pending = {"schemaVersion": 1, "authorityId": authority_id, "root": str(root),
               "stateFile": str(state_file), "checkpoint": checkpoint, "checkpointSha256": _hash(canonical_json(checkpoint))}
    state_file.parent.mkdir(parents=True, exist_ok=True)
    pending_path = state_file.with_suffix(".pending")
    with pending_path.open("xb") as stream:
        stream.write(canonical_json(pending)); stream.flush(); os.fsync(stream.fileno())
    log_path = state_file.with_suffix(".launcher.log")
    with log_path.open("xb", buffering=0) as log:
        process = subprocess.Popen([sys.executable, "-B", "-m", "tools.jenkins.pilot.governance_recovery",
                                    "--repo-root", str(repo_root), "--state-file", str(state_file)],
                                   cwd=repo_root, env=dict(os.environ, PYTHONPATH=str(repo_root), PYTHONDONTWRITEBYTECODE="1"),
                                   stdin=subprocess.DEVNULL, stdout=log, stderr=log, creationflags=subprocess.CREATE_NO_WINDOW)
        birth = native_process.popen_process_creation_time(process)
    deadline = time.monotonic() + 180
    while time.monotonic() < deadline:
        if state_file.exists():
            from .governance import require_live_storage_owner
            try:
                owner = require_live_storage_owner(root, repo_root)
            except PilotError:
                # The unique new state is durable before the authority CAS.
                # Reconcile only that exact expected pre-transition interval.
                with WorkerStorage(root, create=False) as backend:
                    if _hash(backend.read_bytes(OWNER_FILE, max_bytes=65536)) != expected_authority_sha256:
                        raise
                time.sleep(0.1)
                continue
            if owner["authorityId"] != authority_id or owner["owner_pid"] != process.pid or owner["owner_process_creation_time"] != birth:
                raise PilotError("recovery returned a different native owner epoch")
            return json.loads(state_file.read_bytes())
        if not native_process.process_matches_creation_time(process.pid, birth):
            raise PilotError("recovery keeper exited; preserve pending and launcher evidence")
        time.sleep(0.1)
    raise PilotError("recovery remains pending; preserve its exact reserved epoch")


@contextmanager
def _generation_locks(backend: WorkerStorage, checkpoint: dict):
    """Observe existing writer locks without deleting, resetting or adopting caches."""
    import msvcrt
    with ExitStack() as pins:
        for name in checkpoint["generations"]:
            lock_name = str(Path(name).parent / "writer.lock").replace("\\", "/")
            if not backend.exists(lock_name):
                raise PilotError("recovery cannot prove an existing Cargo writer lock identity")
            stream = pins.enter_context(backend.open_lock_file(lock_name))
            stream.seek(0)
            try:
                msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError as error:
                raise PilotError("recovery refuses an occupied Cargo generation writer lock") from error
            def unlock(file=stream):
                file.seek(0)
                msvcrt.locking(file.fileno(), msvcrt.LK_UNLCK, 1)
            pins.callback(unlock)
        yield


def keep_recovered_storage(repo_root: Path, state_file: Path) -> None:
    from .governance import _serve_owner
    from .lifecycle import runtime_lock
    from .execution_dispatch import _runtime_lock
    repo_root = repo_root.resolve()
    state_file = _state_path(repo_root, state_file)
    pending = json.loads(state_file.with_suffix(".pending").read_bytes())
    checkpoint = pending["checkpoint"]
    root = Path(pending["root"])
    if state_file.exists() or _hash(canonical_json(checkpoint)) != pending["checkpointSha256"]:
        raise PilotError("recovery reservation changed or already completed")
    with WorkerStorage(root, create=False) as pin:
        if not pin.exists("execution-runtime.lock"):
            raise PilotError("recovery handoff lock identity is missing")
        with runtime_lock(root, timeout_seconds=0), _runtime_lock(ManagedStorage(root)), _generation_locks(pin, checkpoint):
            actual = inspect_recovery(repo_root, root, expected_authority_sha256=checkpoint["previousAuthoritySha256"])
            static_actual, static_expected = dict(actual), dict(checkpoint)
            fresh_deaths = static_actual.pop("nativeDeathProofs")
            old_deaths = static_expected.pop("nativeDeathProofs")
            # Death is re-proven with fresh native handles. An absent PID may
            # become a reused unrelated process without changing old identity.
            if (static_actual != static_expected
                    or [(item["pid"], item["creationTime"]) for item in fresh_deaths]
                    != [(item["pid"], item["creationTime"]) for item in old_deaths]):
                raise PilotError("recovery checkpoint changed before authority transition")
            relative = f"authority-history/{pending['authorityId']}"
            if (root / relative).exists():
                raise PilotError("recovery proof identity already exists")
            pin.create_directory(relative)
            owner_raw = pin.read_bytes(OWNER_FILE, max_bytes=65536)
            owner_identity = pin.file_identity(OWNER_FILE)
            old_state, _ = _state_bytes(Path(checkpoint["previousStatePath"]))
            pin.write_bytes(relative + "/previous-owner.json", owner_raw)
            pin.write_bytes(relative + "/previous-state.json", old_state)
            proof = {**pending, "recoveredKeeperPid": os.getpid(),
                     "recoveredKeeperCreationTime": native_process.process_creation_time(os.getpid()),
                     "transitionNativeDeathProofs": fresh_deaths}
            proof_bytes = canonical_json(proof)
            pin.write_bytes(relative + "/recovery-proof.json", proof_bytes)
            owner = {"schemaVersion": OWNER_SCHEMA, "authorityId": pending["authorityId"], "root": str(root),
                     "rootIdentity": checkpoint["rootIdentity"], "stateFile": str(state_file),
                     "owner_pid": os.getpid(), "owner_process_creation_time": proof["recoveredKeeperCreationTime"],
                     "status": "active", "previousAuthorityId": checkpoint["previousOwner"]["authorityId"],
                     "recoveryProof": relative + "/recovery-proof.json", "recoveryProofSha256": _hash(proof_bytes)}
            # New state is unique and written first; previous metadata is never mutated.
            with state_file.open("xb") as stream:
                stream.write(canonical_json({"schemaVersion": OWNER_SCHEMA, "root": str(root), "owner": owner})); stream.flush(); os.fsync(stream.fileno())
            pin.write_bytes(OWNER_FILE, canonical_json(owner), expected_identity=owner_identity)
        _serve_owner(repo_root, state_file, root, owner, pin)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--state-file", type=Path, required=True)
    args = parser.parse_args()
    try:
        keep_recovered_storage(args.repo_root, args.state_file)
    except BaseException as error:
        _write_state(args.state_file.with_suffix(".fatal.json"), {"schemaVersion": 1, "ownerPid": os.getpid(),
                     "errorType": type(error).__name__, "error": str(error), "recordedAtUnix": time.time()})
        raise


if __name__ == "__main__":
    main()
