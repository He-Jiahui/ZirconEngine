"""One fixed Jenkins build handoff to its independent storage keeper."""
from __future__ import annotations

from contextlib import contextmanager
import json
import ctypes
import hashlib
import errno
from ctypes import wintypes
import os
from pathlib import Path
import sys
import time

from .contracts import PilotError, RequestIdentity, canonical_json, digest, identifier, sha256_identity
from .native.process_identity import process_creation_time, process_matches_creation_time
from .snapshot import load_snapshot
from .storage import ManagedStorage, require_managed_root


@contextmanager
def _runtime_lock(storage, *, wait_seconds=0, skip_busy=False):
    import msvcrt
    with storage.backend() as backend, backend.open_lock_file("execution-runtime.lock") as stream:
        if stream.seek(0, 2) == 0:
            stream.write(b"0")
        stream.seek(0)
        deadline = time.monotonic() + max(0, wait_seconds)
        while True:
            try:
                stream.seek(0)
                msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
                break
            except OSError as error:
                if error.errno != errno.EACCES:
                    raise
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    if skip_busy:
                        yield False
                        return
                    raise PilotError("execution handoff is busy; preserve the original intent") from error
                time.sleep(min(0.05, remaining))
        try:
            yield True
        finally:
            stream.seek(0)
            msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)


def _owner(root, repo_root):
    from .governance import require_live_storage_owner
    return require_live_storage_owner(root, repo_root)


def _key(identity):
    return digest({"sessionId": identity.session_id, "requestId": identity.request_id,
                   "attemptId": identity.attempt_id, "generation": identity.generation})


def _receipt_path(storage, identity):
    return storage.path(f"runs/{identity.session_id}/{identity.request_id}/{identity.attempt_id}-g{identity.generation}/receipt.json")


def _read_receipt(storage, identity, job, build_number, binding=None):
    path = _receipt_path(storage, identity)
    if not path.exists():
        return None
    value = json.loads(path.read_bytes())
    if (value.get("schemaVersion") != 2 or value.get("identity") != identity.to_dict()
            or value.get("job") != job or value.get("buildNumber") != build_number):
        raise PilotError("durable execution receipt does not match this exact Jenkins build")
    if binding is not None and (value.get("bundleHash") != binding["bundleHash"]
            or value.get("toolchain", {}).get("driverInputHash") != binding["driverInputHash"]):
        raise PilotError("durable execution receipt does not match its sealed inputs and driver")
    return value


def _driver(root, repo_root):
    from .deployment import verify_driver
    from .jenkins_config import read_manifest
    manifest = read_manifest(root)
    driver = require_managed_root(Path(manifest["driverRoot"]), allow_root=False)
    if not driver.is_relative_to(root) or driver == root:
        raise PilotError("sealed driver must be private to this storage authority")
    if verify_driver(root) != manifest["driverInputHash"]:
        raise PilotError("sealed driver hash differs from controller manifest")
    proof = json.loads(ManagedStorage(root).path("driver-proof.json").read_bytes())
    if (proof.get("driverRoot") != str(driver) or proof.get("repoRoot") != str(repo_root)
            or proof.get("driverInputHash") != manifest["driverInputHash"]):
        raise PilotError("sealed driver authority does not match dispatch")
    return driver, manifest


def _admit_build(root, job, build_number, binding, manifest):
    from .client import JenkinsClient
    from .jenkins_config import read_credentials
    client = JenkinsClient(str(manifest["controllerUrl"]), **read_credentials(root), timeout=2)
    node_name = identifier(manifest["agentName"], "agentName")
    node = client.api(f"/computer/{node_name}/api/json?tree=offline")
    build = client.api(f"/job/{job}/{build_number}/api/json?tree=number,building,result,actions[parameters[name,value]]")
    identity = RequestIdentity.from_dict(binding["identity"])
    if (build.get("number") != build_number or node.get("offline") is not False or build.get("building") is not True
            or not client._matches(build, identity, binding["bundleHash"])):
        raise PilotError("Jenkins build must be building on its online admitted node")


def _validate_intent(value, owner):
    fields = {"schemaVersion", "authorityId", "rootIdentity", "binding", "fingerprint", "status", "launch"}
    if not isinstance(value, dict) or set(value) != fields or value["schemaVersion"] != 2:
        raise PilotError("execution intent schema is unsupported")
    if value["authorityId"] != owner["authorityId"] or value["rootIdentity"] != owner["rootIdentity"]:
        raise PilotError("execution intent belongs to a different physical owner")
    binding = value["binding"]
    if set(binding) != {"identity", "job", "buildNumber", "bundleHash", "driverInputHash"}:
        raise PilotError("execution binding contains unsupported fields")
    identity = RequestIdentity.from_dict(binding["identity"])
    identifier(binding["job"], "job")
    if type(binding["buildNumber"]) is not int or binding["buildNumber"] < 1:
        raise PilotError("execution binding needs an exact build number")
    if digest(binding) != value["fingerprint"] or value["status"] not in {"pending", "reserved", "started", "complete"}:
        raise PilotError("execution intent fingerprint or lifecycle is invalid")
    for name in ("bundleHash", "driverInputHash"):
        sha256_identity(binding[name], name)
    return identity


def _recovered_completed_intent(storage, owner, relative, raw, value):
    """Skip only immutable old terminal work named by this recovery epoch."""
    if value.get("authorityId") == owner["authorityId"]:
        return False
    from .governance_recovery import preservation_chain, _require_dead, _known_processes, _terminal_receipt
    with storage.backend() as backend:
        chain = preservation_chain(backend, owner)
    previous = next((item["previousOwner"] for item in chain
                     if item["previousOwner"]["authorityId"] == value.get("authorityId")), None)
    if previous is None:
        raise PilotError("foreign authority execution intent has no exact preservation ancestry")
    identity = _validate_intent(value, previous)
    if relative != f"execution-intents/{_key(identity)}.json" or value["status"] != "complete":
        raise PilotError("pending or uncertain historical execution cannot be skipped")
    record = chain[0].get("completedIntents", {}).get(relative)
    if (not isinstance(record, dict) or record.get("intentSha256") != hashlib.sha256(raw).hexdigest()
            or record.get("previousAuthorityId") != previous["authorityId"]
            or record.get("binding") != value["binding"] or record.get("launch") != value["launch"]):
        raise PilotError("historical intent differs from immutable recovery checkpoint")
    for checkpoint in chain:
        if checkpoint.get("completedIntents", {}).get(relative) != record:
            raise PilotError("historical intent differs across immutable recovery ancestry")
        if checkpoint["previousOwner"]["authorityId"] == previous["authorityId"]:
            break
    launch = value["launch"]
    if not isinstance(launch, dict):
        raise PilotError("historical execution host has not proved exact terminal identity")
    _require_dead(launch.get("pid"), launch.get("creationTime"))
    _known_processes(launch, [])
    receipt_name = _receipt_path(storage, identity).relative_to(storage.root).as_posix()
    if record.get("receiptPath") != receipt_name:
        raise PilotError("historical receipt path differs from its exact attempt")
    sha256_identity(record.get("receiptSha256"), "recovery receipt sha256")
    with storage.backend() as backend:
        receipt_raw = backend.read_bytes(receipt_name, expected_sha256=record.get("receiptSha256"), max_bytes=16 * 1024 * 1024)
    receipt = json.loads(receipt_raw)
    binding = value["binding"]
    if (receipt.get("schemaVersion") != 2 or receipt.get("identity") != identity.to_dict()
            or receipt.get("job") != binding["job"] or receipt.get("buildNumber") != binding["buildNumber"]
            or receipt.get("bundleHash") != binding["bundleHash"]
            or receipt.get("toolchain", {}).get("driverInputHash") != binding["driverInputHash"]):
        raise PilotError("historical receipt binding changed")
    _terminal_receipt(receipt)
    _known_processes(receipt.get("processTree"), [])
    return True


def _active_conflict(storage, owner, key):
    folder = storage.path("execution-intents")
    if not folder.exists():
        return
    for path in folder.glob("*.json"):
        relative = path.relative_to(storage.root).as_posix()
        with storage.backend() as backend:
            raw = backend.read_bytes(relative, max_bytes=1024 * 1024)
        value = json.loads(raw)
        if _recovered_completed_intent(storage, owner, relative, raw, value):
            continue
        identity = _validate_intent(value, owner)
        if path.stem != _key(identity):
            raise PilotError("execution intent filename does not match its identity")
        if path.stem != key:
            launch = value["launch"]
            if launch is not None and process_matches_creation_time(launch["pid"], launch["creationTime"]):
                raise PilotError("another native worker is still alive")
            receipt = _read_receipt(storage, identity, value["binding"]["job"], value["binding"]["buildNumber"], value["binding"])
            if receipt is None:
                raise PilotError("another native execution is active or uncertain; Jenkins must serialize admission")


def _require_submission_budget(deadline):
    if deadline is not None and time.monotonic() >= deadline:
        raise PilotError("execution submission exhausted the original observation deadline")


def _submit(storage, owner, identity, binding, bundle_bytes, *, deadline=None):
    _require_submission_budget(deadline)
    key = _key(identity)
    relative = f"execution-intents/{key}.json"
    path = storage.path(relative)
    if path.exists():
        original = json.loads(path.read_bytes())
        _validate_intent(original, owner)
        if original["binding"] != binding:
            raise PilotError("this immutable attempt is already bound to a different build or content")
        return original
    _active_conflict(storage, owner, key)
    _require_submission_budget(deadline)
    storage.atomic_write(f"execution-inputs/{key}/source.zip", bundle_bytes)
    staged = storage.path(f"execution-inputs/{key}/source.zip")
    sealed = load_snapshot(staged, expected_input_hash=identity.input_hash)
    if sealed.bundle_hash != binding["bundleHash"]:
        raise PilotError("private source archive changed while staging")
    _require_submission_budget(deadline)
    storage.atomic_write(f"execution-inputs/{key}/request.json", canonical_json(identity.to_dict()))
    value = {"schemaVersion": 2, "authorityId": owner["authorityId"], "rootIdentity": owner["rootIdentity"],
             "binding": binding, "fingerprint": digest(binding), "status": "pending", "launch": None}
    _require_submission_budget(deadline)
    storage.atomic_write(relative, canonical_json(value))
    return value


def enqueue_and_wait(*, root, repo_root, bundle, identity, job, build_number, maximum_seconds=1200):
    root = require_managed_root(Path(root), allow_root=False)
    repo_root = Path(repo_root).resolve()
    identifier(job, "job")
    if type(build_number) is not int or build_number < 1 or maximum_seconds <= 0:
        raise PilotError("dispatch needs an exact build number and positive wait budget")
    deadline = time.monotonic() + maximum_seconds
    owner = _owner(root, repo_root)
    driver, manifest = _driver(root, repo_root)
    sealed = load_snapshot(Path(bundle), expected_input_hash=identity.input_hash)
    binding = {"identity": identity.to_dict(), "job": job, "buildNumber": build_number,
               "bundleHash": sealed.bundle_hash, "driverInputHash": manifest["driverInputHash"]}
    storage = ManagedStorage(root)
    with _runtime_lock(storage, wait_seconds=max(0, deadline - time.monotonic())):
        # A contended handoff may outlive a keeper or admission state. Validate
        # both again while holding the exact original submission lock.
        owner = _owner(root, repo_root)
        if manifest["driverInputHash"] != _driver(root, repo_root)[1]["driverInputHash"]:
            raise PilotError("sealed driver changed while waiting for submission handoff")
        if time.monotonic() >= deadline:
            raise PilotError("execution handoff exhausted the original observation deadline")
        _admit_build(root, job, build_number, binding, manifest)
        _require_submission_budget(deadline)
        _submit(storage, owner, identity, binding, Path(bundle).read_bytes(), deadline=deadline)
    while time.monotonic() < deadline:
        owner = _owner(root, repo_root)
        receipt = _read_receipt(storage, identity, job, build_number, binding)
        if receipt is not None:
            return receipt
        value = json.loads(storage.path(f"execution-intents/{_key(identity)}.json").read_bytes())
        _validate_intent(value, owner)
        if value["binding"] != binding:
            raise PilotError("immutable execution intent changed")
        # Reservation is transient during atomic startup. The keeper never
        # retries it; the waiter preserves it through its bounded observation.
        if value["launch"] is not None and not process_matches_creation_time(value["launch"]["pid"], value["launch"]["creationTime"]):
            # Check receipt after observing exit to close the publication race.
            receipt = _read_receipt(storage, identity, job, build_number, binding)
            if receipt is not None:
                return receipt
            raise PilotError("independent worker exited without a matching durable receipt")
        time.sleep(0.2)
    raise PilotError("execution remains pending; preserve and reconcile its original intent")


def service_execution_intents(root, repo_root, owner):
    root = require_managed_root(Path(root), allow_root=False)
    repo_root = Path(repo_root).resolve()
    live = _owner(root, repo_root)
    if (live != owner or owner["owner_pid"] != os.getpid()
            or owner["owner_process_creation_time"] != process_creation_time(os.getpid())):
        raise PilotError("only the exact standalone keeper may launch build execution")
    storage = ManagedStorage(root)
    folder = storage.path("execution-intents")
    if not folder.exists():
        return
    candidates = []
    # Historical recovery proof verification can be expensive. It reads only
    # immutable terminal work and must not occupy the submission handoff lock.
    for path in folder.glob("*.json"):
        relative = path.relative_to(root).as_posix()
        with storage.backend() as backend:
            raw = backend.read_bytes(relative, max_bytes=1024 * 1024)
        value = json.loads(raw)
        if _recovered_completed_intent(storage, owner, relative, raw, value):
            continue
        identity = _validate_intent(value, owner)
        if path.stem != _key(identity):
            raise PilotError("execution intent filename identity mismatch")
        if value["status"] == "complete":
            binding = value["binding"]
            if _read_receipt(storage, identity, binding["job"], binding["buildNumber"], binding) is None:
                raise PilotError("completed execution intent has no matching durable receipt")
            continue
        candidates.append(path)
    if not candidates:
        return
    with _runtime_lock(storage, skip_busy=True) as acquired:
        if not acquired:
            return  # Keeper polling never competes by blocking a producer.
        for path in candidates:
            relative = path.relative_to(root).as_posix()
            with storage.backend() as backend:
                raw = backend.read_bytes(relative, max_bytes=1024 * 1024)
            value = json.loads(raw)
            if _recovered_completed_intent(storage, owner, relative, raw, value):
                continue
            identity = _validate_intent(value, owner)
            key = _key(identity)
            if path.stem != key:
                raise PilotError("execution intent filename identity mismatch")
            binding = value["binding"]
            if _read_receipt(storage, identity, binding["job"], binding["buildNumber"], binding) is not None:
                if value["launch"] is not None and process_matches_creation_time(value["launch"]["pid"], value["launch"]["creationTime"]):
                    continue
                if value["status"] != "complete":
                    value["status"] = "complete"
                    storage.atomic_write(relative, canonical_json(value))
                continue
            if value["status"] != "pending":
                continue  # Reserved/started work is never retried or adopted.
            _active_conflict(storage, owner, key)
            driver, manifest = _driver(root, repo_root)
            if binding["driverInputHash"] != manifest["driverInputHash"]:
                raise PilotError("execution was submitted against a different driver")
            bundle = storage.path(f"execution-inputs/{key}/source.zip")
            if load_snapshot(bundle, expected_input_hash=identity.input_hash).bundle_hash != binding["bundleHash"]:
                raise PilotError("reserved source archive hash changed")
            request = storage.path(f"execution-inputs/{key}/request.json")
            if json.loads(request.read_bytes()) != identity.to_dict():
                raise PilotError("reserved request identity changed")
            _admit_build(root, binding["job"], binding["buildNumber"], binding, manifest)
            value["status"] = "reserved"
            storage.atomic_write(relative, canonical_json(value))
            argv = [sys.executable, "-B", "-m", "tools.jenkins.pilot.runner", "--root", str(root),
                    "--repo-root", str(repo_root), "--bundle", str(bundle), "--request-file", str(request),
                    "--build-number", str(binding["buildNumber"]), "--job", binding["job"],
                    "--monitor-jenkins", "--execution-only"]
            scratch = storage.temporary_directory(prefix="execution-")
            trusted_names = {"PATH", "INCLUDE", "LIB", "LIBPATH", "VCTOOLSINSTALLDIR", "WINDOWSSDKDIR",
                             "WINDOWSSDKVERSION", "WINDIR", "SYSTEMROOT", "COMSPEC", "PATHEXT",
                             "USERPROFILE", "LOCALAPPDATA", "APPDATA", "RUSTUP_HOME"}
            # This function runs only in the exact keeper. Tool discovery and
            # the MSVC environment are bound when that owner is launched.
            env = {name: value for name, value in os.environ.items() if name.upper() in trusted_names}
            env.update(PYTHONPATH=str(driver), PYTHONDONTWRITEBYTECODE="1", PYTHONUTF8="1",
                       TEMP=str(scratch), TMP=str(scratch))
            from .lifetime import launch_owned
            def persist(record):
                value["launch"] = record
                value["status"] = "started"
                storage.atomic_write(relative, canonical_json(value))
            log = storage.path(f"execution-inputs/{key}/launcher.log")
            with storage.backend() as backend, backend.open_lock_file(log.relative_to(root).as_posix()) as stream:
                # A previously occupied log makes this reserved launch
                # uncertain; never overwrite its historical output.
                if stream.seek(0, 2) != 0:
                    raise PilotError("reserved launcher log already contains output")
                launch_owned(argv, cwd=driver, env=env, stream=stream, log_path=log, lease=owner, on_record=persist)


def require_execution_worker(*, root, repo_root, identity, job, build_number):
    """Allow only a current member of this build's exact keeper-owned Job."""
    root = require_managed_root(Path(root), allow_root=False)
    repo_root = Path(repo_root).resolve()
    owner = _owner(root, repo_root)
    storage = ManagedStorage(root)
    key = _key(identity)
    with storage.backend() as backend:
        value = json.loads(backend.read_bytes(f"execution-intents/{key}.json", max_bytes=1024 * 1024))
    _validate_intent(value, owner)
    binding = value["binding"]
    if (binding["identity"] != identity.to_dict() or binding["job"] != job
            or binding["buildNumber"] != build_number or value["status"] not in {"reserved", "started"}
            or not isinstance(value["launch"], dict)):
        raise PilotError("execution-only worker has no matching reserved Jenkins build")
    driver, manifest = _driver(root, repo_root)
    if binding["driverInputHash"] != manifest["driverInputHash"]:
        raise PilotError("execution worker driver binding changed")
    bundle = storage.path(f"execution-inputs/{key}/source.zip")
    if load_snapshot(bundle, expected_input_hash=identity.input_hash).bundle_hash != binding["bundleHash"]:
        raise PilotError("execution worker source binding changed")
    with storage.backend() as backend:
        request = json.loads(backend.read_bytes(f"execution-inputs/{key}/request.json", max_bytes=64 * 1024))
    if request != identity.to_dict():
        raise PilotError("execution worker request binding changed")
    launch = value["launch"]
    if (launch.get("keeperPid") != owner["owner_pid"]
            or launch.get("keeperCreationTime") != owner["owner_process_creation_time"]
            or not process_matches_creation_time(launch["pid"], launch["creationTime"])):
        raise PilotError("execution host identity is not live under this exact keeper")
    from .lifetime import _kernel, _open_owner
    api = _kernel()
    keeper = _open_owner(api, owner["owner_pid"], owner["owner_process_creation_time"])
    query = wintypes.HANDLE()
    host = None
    try:
        local = api.GetCurrentProcess()
        if not api.DuplicateHandle(keeper, launch["keeperJobHandle"], local, ctypes.byref(query), 4, False, 0):
            raise OSError(ctypes.get_last_error(), "cannot retain exact execution Job query authority")
        api.IsProcessInJob.argtypes = [wintypes.HANDLE, wintypes.HANDLE, ctypes.POINTER(wintypes.BOOL)]
        api.IsProcessInJob.restype = wintypes.BOOL
        from .native.process_identity import _birth
        host = api.OpenProcess(0x1000 | 0x100000, False, launch["pid"])
        if not host:
            raise OSError(ctypes.get_last_error(), "cannot retain exact execution host identity")
        if _birth(api, host) != launch["creationTime"] or api.WaitForSingleObject(host, 0) != 258:
            raise PilotError("exact execution host exited or its PID was reused")
        host_member = wintypes.BOOL()
        if not api.IsProcessInJob(host, query, ctypes.byref(host_member)):
            raise OSError(ctypes.get_last_error(), "cannot prove execution host Job binding")
        if not host_member.value:
            raise PilotError("exact execution host is outside its reserved Job")
        member = wintypes.BOOL()
        if not api.IsProcessInJob(local, query, ctypes.byref(member)):
            raise OSError(ctypes.get_last_error(), "cannot prove current execution worker membership")
        if not member.value:
            raise PilotError("current process is outside the exact Jenkins execution Job")
        if api.WaitForSingleObject(keeper, 0) != 258:
            raise PilotError("exact storage keeper exited during worker authorization")
    finally:
        if host:
            api.CloseHandle(host)
        if query.value:
            api.CloseHandle(query)
        api.CloseHandle(keeper)
    return value
