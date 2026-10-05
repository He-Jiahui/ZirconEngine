"""Cancellation proves exact local resources terminal before stopping Jenkins."""
from __future__ import annotations

import ctypes
import hashlib
import json
import msvcrt
import sqlite3
import time
import urllib.parse
from ctypes import wintypes
from typing import Mapping

from tools.jenkins.pilot.contracts import RequestIdentity, digest
from tools.jenkins.pilot.execution_dispatch import _key, _runtime_lock, _read_receipt, _validate_intent
from tools.jenkins.pilot.governance import require_live_storage_owner
from tools.jenkins.pilot.governance_recovery import _terminal_receipt, _known_processes
from tools.jenkins.pilot.journal import SubmissionJournal
from tools.jenkins.pilot.native.paths import WorkerStorage
from tools.jenkins.pilot.native.process_identity import process_matches_creation_time
from tools.jenkins.pilot.storage import ManagedStorage, require_managed_root, physical_identity

from .config import TrayConfig, TrayError, validate_runtime
from .persistence import read_json, write_json

_TERM_AFTER_SECONDS = 3
_KILL_AFTER_SECONDS = 8


def _journal_rows(config: TrayConfig) -> list[dict]:
    """A read transaction on the existing journal; no DDL or recovery writes."""
    with WorkerStorage(config.pilot_root, create=False) as backend:
        if not backend.exists("submissions.sqlite3"):
            return []
        if backend.exists("submissions.sqlite3-wal") or backend.exists("submissions.sqlite3-shm"):
            raise TrayError("请求日志含不支持的 WAL 状态，必须保留并核对")
        with backend._windows.open_relative("submissions.sqlite3", "rb"):
            database = config.pilot_root / "submissions.sqlite3"
            with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=3) as connection:
                connection.row_factory = sqlite3.Row
                connection.execute("BEGIN")
                return [dict(row) for row in connection.execute("SELECT * FROM submissions")]


def running_bindings(config: TrayConfig, active_builds: list[Mapping]) -> list[dict]:
    manifest = validate_runtime(config)
    rows = _journal_rows(config)
    result = []
    for build in active_builds:
        number, job = build.get("number"), build.get("job")
        if type(number) is not int or number < 1 or not isinstance(job, str):
            raise TrayError("活动构建缺少明确身份，停止操作已拒绝")
        candidates = [row for row in rows if row["job"] == job and row["build_number"] == number]
        # A queued journal can become running between the last reconciliation
        # and the tray snapshot. Match immutable request identity, then verify
        # actual Jenkins parameters before issuing its cancellation.
        if not candidates:
            candidates = [row for row in rows if row["job"] == job
                          and row["session_id"] == build.get("sessionId")
                          and row["request_id"] == build.get("requestId")
                          and row["build_number"] in (None, number)]
        if len(candidates) != 1:
            raise TrayError("活动构建未绑定本实例的确切请求，停止操作已拒绝")
        row = candidates[0]
        identity = RequestIdentity.from_dict(json.loads(row["identity_json"]))
        if (identity.session_id != build.get("sessionId") or identity.request_id != build.get("requestId")
                or identity.attempt_id != build.get("attemptId") or str(identity.generation) != str(build.get("generation"))):
            raise TrayError("活动构建与请求日志的身份不一致")
        result.append({"identity": identity.to_dict(), "job": job, "buildNumber": number,
                       "bundleHash": row["bundle_hash"], "driverInputHash": manifest["driverInputHash"]})
    return result


def _job_zero(launch: dict) -> dict:
    from tools.jenkins.pilot.lifetime import _kernel, _open_owner
    from tools.jenkins.pilot.native.windows_job_process import _JobObjectBasicAccountingInformation
    api = _kernel()
    owner = _open_owner(api, launch["keeperPid"], launch["keeperCreationTime"])
    handle = wintypes.HANDLE()
    try:
        if not api.DuplicateHandle(owner, launch["keeperJobHandle"], api.GetCurrentProcess(),
                                   ctypes.byref(handle), 4, False, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        api.QueryInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p,
                                                  wintypes.DWORD, ctypes.c_void_p]
        api.QueryInformationJobObject.restype = wintypes.BOOL
        value = _JobObjectBasicAccountingInformation()
        if not api.QueryInformationJobObject(handle, 1, ctypes.byref(value), ctypes.sizeof(value), None):
            raise ctypes.WinError(ctypes.get_last_error())
        if value.active_processes:
            raise TrayError("原执行器 Job 中仍存在进程")
        if process_matches_creation_time(launch["pid"], launch["creationTime"]):
            raise TrayError("原执行器进程仍在运行")
        return {"activeProcesses": 0, "hostPid": launch["pid"], "hostCreationTime": launch["creationTime"]}
    finally:
        if handle.value:
            api.CloseHandle(handle)
        api.CloseHandle(owner)


def _cargo_release(config: TrayConfig, identity: RequestIdentity, receipt: dict) -> dict:
    from tools.jenkins.pilot.runner import run_directory
    from tools.jenkins.pilot.managed_cargo import terminal_verified
    run = run_directory(config.pilot_root, identity)
    with WorkerStorage(run, create=False) as backend:
        request = json.loads(backend.read_bytes("managed-request.json", max_bytes=4 * 1024 * 1024))
        result = json.loads(backend.read_bytes("managed-result.json", max_bytes=1024 * 1024))
    binding = {"requestHash": digest(request), "observationHash": digest(result.get("observation")), "runRoot": str(run)}
    if (result.get("identity") != identity.to_dict() or result.get("requestHash") != digest(request)
            or result.get("lockReleased") is not True or result.get("terminalVerified") is not True
            or result.get("terminalBinding") != binding or not terminal_verified(result.get("observation", {}))):
        raise TrayError("Cargo 执行缺少原生终态及缓存释放记录")
    generation = request["generation"]
    directory = require_managed_root(generation["generationPath"], allow_root=False)
    if not directory.is_relative_to(config.pilot_root / "cargo-generations"):
        raise TrayError("Cargo generation 不属于当前 Jenkins 根")
    if physical_identity(directory) != generation["generationIdentity"]:
        raise TrayError("Cargo generation 物理身份已变化")
    release_name = f"releases/{digest(binding)}.json"
    if result.get("releaseRecord") != release_name:
        raise TrayError("缓存释放记录没有匹配当前请求")
    with WorkerStorage(directory, create=False) as backend:
        released = json.loads(backend.read_bytes(release_name, max_bytes=1024 * 1024))
        if (digest(released) != result.get("releaseRecordHash") or released.get("state") != "released"
                or released.get("identity") != identity.to_dict() or released.get("terminalBinding") != binding):
            raise TrayError("缓存释放证明与原请求不匹配")
        with backend._windows.open_relative("writer.lock", "r+b") as lock:
            lock.seek(0)
            try:
                msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError:
                raise TrayError("缓存 writer 锁仍被占用") from None
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)
    return {"releaseRecord": release_name, "releaseRecordHash": result["releaseRecordHash"], "writerReleased": True}


def local_terminal_proof(config: TrayConfig, binding: dict) -> dict | None:
    identity = RequestIdentity.from_dict(binding["identity"])
    storage = ManagedStorage(config.pilot_root)
    relative = "execution-intents/" + _key(identity) + ".json"
    with storage.backend() as backend:
        if not backend.exists(relative):
            return None
        intent = json.loads(backend.read_bytes(relative, max_bytes=1024 * 1024))
    owner = require_live_storage_owner(config.pilot_root, config.repo_root)
    _validate_intent(intent, owner)
    if intent["binding"] != binding:
        raise TrayError("执行交接记录与停止请求不一致")
    if intent["status"] != "complete":
        return None
    receipt = _read_receipt(storage, identity, binding["job"], binding["buildNumber"], binding)
    if receipt is None:
        return None
    _terminal_receipt(receipt)
    deaths = []
    _known_processes(receipt.get("processTree"), deaths)
    _known_processes(receipt.get("nativeJobEvidence"), deaths)
    from tools.jenkins.pilot.runner import run_directory
    run = run_directory(config.pilot_root, identity)
    path = run / "receipt.json"
    with WorkerStorage(run, create=False) as backend:
        raw = backend.read_bytes("receipt.json", max_bytes=16 * 1024 * 1024)
    if json.loads(raw) != receipt:
        raise TrayError("原生终态回执在核验期间已变化")
    proof = {"binding": binding, "receiptPath": str(path), "receiptSha256": hashlib.sha256(raw).hexdigest(),
             "nativeDeaths": deaths, "pipeEOF": True, "jobZero": True}
    if intent["launch"] is not None:
        proof["launcherJob"] = _job_zero(intent["launch"])
    if identity.template == "managed-cargo-check-v2":
        proof["cargoRelease"] = _cargo_release(config, identity, receipt)
    return proof


def observe_terminal(config: TrayConfig, client, binding: dict, journal) -> dict | None:
    """Observe original remote/local terminal evidence without another cancellation."""
    identity = RequestIdentity.from_dict(binding["identity"])
    remote = client.reconcile(binding["job"], identity, journal)
    if remote["state"] not in {"completed", "cancelled"}:
        return None
    proof = local_terminal_proof(config, binding)
    if proof is not None:
        return proof
    with _runtime_lock(ManagedStorage(config.pilot_root), skip_busy=True) as acquired:
        if acquired:
            with WorkerStorage(config.pilot_root, create=False) as backend:
                no_intent = not backend.exists("execution-intents/" + _key(identity) + ".json")
            if no_intent:
                fresh = client.reconcile(binding["job"], identity, journal)
                if fresh["state"] in {"completed", "cancelled"}:
                    return {"binding": binding, "notLaunched": True, "terminalBuild": fresh["result"]}
    return None


def cancel_bindings(config: TrayConfig, client, bindings: list[dict], deadline: float) -> dict:
    """Cancel only running builds; queued submissions are left untouched."""
    if not isinstance(deadline, (int, float)) or deadline <= time.monotonic():
        raise TrayError("停止对账期限已耗尽；原记录保留")
    operation = read_json(config.state_dir / "operation.json") or {}
    checkpoint = {"schemaVersion": 1, "rootIdentity": dict(config.root_identity), "bindings": bindings,
                  "operationId": operation.get("operationId"), "state": "cancelling", "createdAt": time.time()}
    write_json(config.state_dir / "shutdown-bindings.json", checkpoint)
    if not bindings:
        checkpoint.update(state="verified", proofs=[])
        write_json(config.state_dir / "shutdown-bindings.json", checkpoint)
        return {"verified": True, "proofs": []}
    journal = SubmissionJournal(config.pilot_root / "submissions.sqlite3")
    for binding in bindings:
        if time.monotonic() >= deadline:
            raise TrayError("停止对账期限已耗尽；原记录保留")
        client.timeout = min(8, max(0.1, deadline - time.monotonic()))
        identity = RequestIdentity.from_dict(binding["identity"])
        record = client.api(f"/job/{urllib.parse.quote(binding['job'], safe='')}/{binding['buildNumber']}/api/json?tree=number,building,result,actions[parameters[name,value]]")
        if record.get("number") != binding["buildNumber"] or not client._matches(record, identity, binding["bundleHash"]):
            raise TrayError("Jenkins 活动构建的完整参数与请求不匹配")
        client.cancel(binding["job"], identity, journal)
    proofs = {}
    stop_started = time.monotonic()
    terminated = set()
    killed = set()
    while time.monotonic() < deadline:
        for binding in bindings:
            key = _key(RequestIdentity.from_dict(binding["identity"]))
            if key in proofs:
                continue
            client.timeout = min(8, max(0.1, deadline - time.monotonic()))
            identity = RequestIdentity.from_dict(binding["identity"])
            remote = client.reconcile(binding["job"], identity, journal)
            if remote["state"] not in {"completed", "cancelled"}:
                if time.monotonic() - stop_started >= _TERM_AFTER_SECONDS and key not in terminated:
                    # Pipeline /stop is asynchronous and may wait on agent I/O.
                    # Escalate once to its documented /term after fresh exact binding.
                    target = f"/job/{urllib.parse.quote(binding['job'], safe='')}/{binding['buildNumber']}"
                    fresh = client.api(target + "/api/json?tree=number,building,result,actions[parameters[name,value]]")
                    if fresh.get("number") != binding["buildNumber"] or not client._matches(fresh, identity, binding["bundleHash"]):
                        raise TrayError("原取消构建身份已变化，停止升级已拒绝")
                    if fresh.get("building") is True:
                        client.request(target + "/term", post=True, data=b"")
                    terminated.add(key)
                    checkpoint["termBuilds"] = sorted(terminated)
                    write_json(config.state_dir / "shutdown-bindings.json", checkpoint)
                elif time.monotonic() - stop_started >= _KILL_AFTER_SECONDS and key in terminated and key not in killed:
                    target = f"/job/{urllib.parse.quote(binding['job'], safe='')}/{binding['buildNumber']}"
                    fresh = client.api(target + "/api/json?tree=number,building,result,actions[parameters[name,value]]")
                    if fresh.get("number") != binding["buildNumber"] or not client._matches(fresh, identity, binding["bundleHash"]):
                        raise TrayError("原取消构建身份已变化，最终终止已拒绝")
                    if fresh.get("building") is True:
                        # Last-resort Pipeline termination can skip archive/finally.
                        # The keeper-owned worker independently writes its receipt;
                        # native Job0/EOF and cache release remain mandatory below.
                        client.request(target + "/kill", post=True, data=b"")
                    killed.add(key)
                    checkpoint["killBuilds"] = sorted(killed)
                    write_json(config.state_dir / "shutdown-bindings.json", checkpoint)
                continue
            proof = local_terminal_proof(config, binding)
            if proof is None:
                # A pipeline may be stopped before its build handoff. Under
                # the handoff lock, a fresh terminal Jenkins check proves that
                # this exact request cannot newly admit a validation process.
                with _runtime_lock(ManagedStorage(config.pilot_root), skip_busy=True) as acquired:
                    if acquired:
                        with WorkerStorage(config.pilot_root, create=False) as backend:
                            no_intent = not backend.exists("execution-intents/" + key + ".json")
                        if no_intent:
                            fresh = client.reconcile(binding["job"], identity, journal)
                            if fresh["state"] in {"completed", "cancelled"}:
                                proof = {"binding": binding, "notLaunched": True, "terminalBuild": fresh["result"]}
            if proof is not None:
                proofs[key] = proof
        if len(proofs) == len(bindings):
            checkpoint.update(state="verified", proofs=list(proofs.values()), verifiedAt=time.time())
            write_json(config.state_dir / "shutdown-bindings.json", checkpoint)
            return {"verified": True, "proofs": list(proofs.values())}
        time.sleep(min(0.5, max(0, deadline - time.monotonic())))
    checkpoint.update(state="pending", proofs=list(proofs.values()), error="取消与原生资源对账超过 120 秒期限")
    write_json(config.state_dir / "shutdown-bindings.json", checkpoint)
    raise TrayError(checkpoint["error"])
