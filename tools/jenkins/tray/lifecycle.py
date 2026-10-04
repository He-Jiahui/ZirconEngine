"""Durable ownership recovery, quiet-down and terminal resource reconciliation."""
from __future__ import annotations

import hashlib
import json
import secrets
import time
import urllib.parse
from typing import Mapping

from tools.jenkins.pilot import bootstrap
from tools.jenkins.pilot.native.paths import WorkerStorage

from . import service
from .config import TrayError, validate_runtime
from .persistence import read_json, write_json


def ensure_owner(config):
    """Classify native keeper death before considering recovery; never catch startup errors."""
    owner = service._owner(config)
    attempt_path = config.state_dir / "recovery-attempt.json"
    attempt = read_json(attempt_path)
    if owner["live"]:
        if attempt and attempt.get("state") == "pending":
            from pathlib import Path
            from tools.jenkins.pilot.governance import require_live_storage_owner
            current = require_live_storage_owner(config.pilot_root, config.repo_root)
            pending_path = Path(attempt["stateFile"]).with_suffix(".pending")
            reserved = read_json(pending_path)
            if not reserved or current["authorityId"] != reserved["authorityId"]:
                raise TrayError("恢复返回了不同的所有权，必须保留原记录并核对")
            attempt.update(state="completed", authorityId=current["authorityId"], finishedAt=time.time())
            write_json(attempt_path, attempt)
        return owner
    if attempt and attempt.get("state") == "pending":
        raise TrayError("原保管进程恢复仍待核对；不会创建第二个恢复身份")
    with WorkerStorage(config.pilot_root, create=False) as backend:
        raw = backend.read_bytes("storage-owner.json", max_bytes=65536)
    expected = hashlib.sha256(raw).hexdigest()
    from tools.jenkins.pilot.governance_recovery import inspect_recovery, recover_storage
    inspect_recovery(config.repo_root, config.pilot_root, expected_authority_sha256=expected)
    state_file = config.repo_root / ".codex/state/jenkins-pilot" / ("tray-recovery-" + secrets.token_hex(12) + ".json")
    attempt = {"schemaVersion": 1, "state": "pending", "stateFile": str(state_file),
               "rootIdentity": dict(config.root_identity), "previousAuthoritySha256": expected,
               "createdAt": time.time()}
    write_json(attempt_path, attempt)
    recovered = recover_storage(config.repo_root, config.pilot_root,
                                expected_authority_sha256=expected, state_file=state_file)
    current = service._owner(config)
    if not current["live"] or recovered["owner"]["authorityId"] != current["authorityId"]:
        raise TrayError("保管进程恢复未完成确切身份核验")
    attempt.update(state="completed", authorityId=current["authorityId"], finishedAt=time.time())
    write_json(attempt_path, attempt)
    return current


def _cancel_owned_quiet(config, client, checkpoint):
    if checkpoint.get("quietOwned") is True:
        client.request("/cancelQuietDown", post=True, data=b"")
        if client.api("/api/json?tree=quietingDown").get("quietingDown") is not False:
            raise TrayError("本次静默模式未能恢复，原操作继续待核对")
        checkpoint["quietOwned"] = False
        checkpoint["quietRestoredAt"] = time.time()
        write_json(config.state_dir / "shutdown.json", checkpoint)


def start_runtime(config):
    validate_runtime(config)
    from tools.jenkins.pilot.deployment import verify_driver
    verify_driver(config.pilot_root)
    ensure_owner(config)
    checkpoint_path = config.state_dir / "start.json"
    from .operations import read_operation
    checkpoint = {"schemaVersion": 1, "state": "starting", "operationId": read_operation(config)["operationId"], "rootIdentity": dict(config.root_identity),
                  "createdAt": time.time()}
    write_json(checkpoint_path, checkpoint)
    controller = bootstrap.start_controller(config.pilot_root)
    checkpoint.update(controller=service._runtime(config)[0].get("controller"), phase="controller")
    write_json(checkpoint_path, checkpoint)
    agent = bootstrap.start_agent(config.pilot_root)
    checkpoint.update(agent=service._runtime(config)[0].get("agent"), phase="node")
    write_json(checkpoint_path, checkpoint)
    deadline = time.monotonic() + 120
    while True:
        status = service.read_status(config)
        if status["controller"]["ready"] and status["agent"]["online"]:
            break
        if time.monotonic() >= deadline:
            raise TrayError("Jenkins 节点在线握手超过 120 秒，原启动记录保留")
        if not status["agent"]["live"]:
            raise TrayError("Jenkins 节点在在线握手前已退出")
        time.sleep(0.5)
    shutdown = read_json(config.state_dir / "shutdown.json")
    if shutdown and shutdown.get("state") == "completed" and status.get("quietDown"):
        _cancel_owned_quiet(config, service._client(status["url"], config), shutdown)
        status = service.read_status(config)
    checkpoint.update(state="completed", finishedAt=time.time())
    write_json(checkpoint_path, checkpoint)
    return {"started": True, "url": controller["url"], "controller": controller, "agent": agent, "status": status}


def _check_confirmation(config, preview, confirmation):
    if not preview["confirmationRequired"]:
        return
    created = confirmation.get("createdAt") if isinstance(confirmation, Mapping) else None
    if (type(created) not in (float, int) or not 0 <= time.time() - created <= 120
            or confirmation.get("rootIdentity") != config.root_identity
            or any(confirmation.get(key) != preview.get(key) for key in ("activeBuilds", "controller", "agent"))):
        raise service.ConfirmationRequired(preview)


def _runtime_terminal(records):
    """Fresh Job0 and output-file release checks, including failed starts."""
    from .terminal import _job_zero
    from tools.jenkins.pilot.lifetime import _kernel, _wait_log_release
    result = {}
    for kind in ("agent", "controller"):
        record = records.get(kind)
        if record is None:
            continue
        if service._live(record):
            raise TrayError("原 Jenkins 运行进程仍存活")
        result[kind] = _job_zero(record)
        _wait_log_release(_kernel(), record, timeout_seconds=0)
        result[kind]["logReleased"] = True
    return result


def _finish_stop(config, checkpoint, terminal):
    # Job termination is intentionally abrupt; persist the frozen queue first.
    # This fixed administrative script contains no uploaded source or credentials.
    current = service._runtime(config)[0]
    if service._live(current.get("controller")):
        client = service._client(service._runtime(config)[2], config)
        script = "jenkins.model.Jenkins.get().getQueue().save(); println('tray queue persisted')"
        raw, _, code = client.request("/scriptText", post=True,
            content_type="application/x-www-form-urlencoded", data=urllib.parse.urlencode({"script": script}).encode())
        if code != 200 or raw.strip() != b"tray queue persisted":
            raise TrayError("Jenkins 排队任务尚未持久化，保留原停止记录")
        items = client.api("/queue/api/json?tree=items[id]").get("items")
        if not isinstance(items, list):
            raise TrayError("持久化后的队列身份无法确认")
        with WorkerStorage(config.pilot_root, create=False) as backend:
            raw = backend.read_bytes("jenkins_home/queue.xml", max_bytes=16 * 1024 * 1024) if backend.exists("jenkins_home/queue.xml") else None
        if items and raw is None:
            raise TrayError("排队任务缺少持久化文件，保留原停止记录")
        checkpoint["queueProof"] = {"saved": True, "queueIds": [v["id"] for v in items],
                                    "sha256": hashlib.sha256(raw).hexdigest() if raw is not None else None}
        write_json(config.state_dir / "shutdown.json", checkpoint)
    result = bootstrap.stop(config.pilot_root)
    native = _runtime_terminal(checkpoint["runtime"])
    checkpoint.update(state="completed", terminal=terminal, runtimeProofs=native, finishedAt=time.time())
    write_json(config.state_dir / "shutdown.json", checkpoint)
    return {"stopped": True, "cancelled": checkpoint["activeBuilds"], "queuedCount": checkpoint["queuedCount"],
            "bootstrap": result, "proof": {"quietDown": True, "terminal": terminal, "runtime": native,
                                           "rootIdentity": dict(config.root_identity)},
            "status": service.read_status(config)}


def stop_runtime(config, *, confirmation=None):
    validate_runtime(config)
    preview = service.stop_preview(config)
    _check_confirmation(config, preview, confirmation)
    if preview["status"]["state"] == "stopped":
        return {"stopped": True, "alreadyStopped": True, "status": preview["status"]}
    if not preview["status"].get("canStop"):
        raise TrayError("当前 Jenkins 身份或任务状态不明确，停止操作已拒绝")
    from .terminal import running_bindings, cancel_bindings
    bindings = running_bindings(config, preview["activeBuilds"])
    runtime, _, url = service._runtime(config)
    client = service._client(url, config)
    checkpoint = {"schemaVersion": 1, "state": "quieting", "rootIdentity": dict(config.root_identity),
                  "runtime": runtime, "bindings": bindings, "activeBuilds": preview["activeBuilds"],
                  "queuedCount": preview["queuedCount"], "quietOwned": not preview["status"]["quietDown"],
                  "createdAt": time.time(), "deadlineAt": time.time() + 120}
    from .operations import read_operation
    checkpoint["operationId"] = read_operation(config)["operationId"]
    write_json(config.state_dir / "shutdown.json", checkpoint)
    client.request("/quietDown", post=True, data=b"")
    if client.api("/api/json?tree=quietingDown").get("quietingDown") is not True:
        raise TrayError("Jenkins 未确认静默模式；原操作保留待核对")
    fresh = service.stop_preview(config)
    # Natural completion can remove an observed build; a new build needs fresh confirmation.
    unseen = [b for b in fresh["activeBuilds"] if b not in preview["activeBuilds"]]
    if unseen or fresh["controller"] != preview["controller"] or fresh["agent"] != preview["agent"]:
        _cancel_owned_quiet(config, client, checkpoint)
        checkpoint.update(state="refused", finishedAt=time.time())
        write_json(config.state_dir / "shutdown.json", checkpoint)
        raise service.ConfirmationRequired(service.stop_preview(config))
    checkpoint.update(state="cancelling")
    write_json(config.state_dir / "shutdown.json", checkpoint)
    terminal = cancel_bindings(config, client, bindings,
                               deadline=time.monotonic() + max(0, checkpoint["deadlineAt"] - time.time()))
    checkpoint.update(state="verified", terminal=terminal)
    write_json(config.state_dir / "shutdown.json", checkpoint)
    return _finish_stop(config, checkpoint, terminal)


def reconcile_runtime(config, previous):
    validate_runtime(config)
    status = service.read_status(config)
    if previous.get("rootIdentity") != config.root_identity:
        raise TrayError("原操作的物理根身份不一致")
    if previous.get("action") == "start":
        if status["controller"]["ready"] and status["agent"]["online"]:
            return {"reconciled": True, "status": status}
        runtime = service._runtime(config)[0]
        if not runtime.get("pendingLaunch") and not status["controller"]["live"] and not status["agent"]["live"]:
            recovery = read_json(config.state_dir / "recovery-attempt.json")
            if recovery and recovery.get("state") == "pending":
                return {"reconciled": False, "status": status}
            proof = _runtime_terminal(runtime)
            return {"reconciled": True, "failed": True, "runtimeProofs": proof, "status": status}
    if previous.get("action") == "stop":
        checkpoint = read_json(config.state_dir / "shutdown.json")
        if checkpoint and checkpoint.get("operationId") != previous.get("operationId"):
            return {"reconciled": True, "failed": True, "status": status}
        if not checkpoint or checkpoint.get("rootIdentity") != config.root_identity:
            # No shutdown record means no quiet/cancel effects were admitted.
            return {"reconciled": True, "failed": True, "status": status} if not checkpoint else {"reconciled": False, "status": status}
        if not status["controller"]["live"] and not status["agent"]["live"]:
            if checkpoint.get("state") not in {"verified", "completed"} or checkpoint.get("terminal", {}).get("verified") is not True:
                return {"reconciled": False, "status": status}
            from .terminal import local_terminal_proof
            for proof in checkpoint["terminal"].get("proofs", []):
                if proof.get("notLaunched"):
                    from tools.jenkins.pilot.execution_dispatch import _key
                    from tools.jenkins.pilot.contracts import RequestIdentity
                    with WorkerStorage(config.pilot_root, create=False) as backend:
                        if backend.exists("execution-intents/" + _key(RequestIdentity.from_dict(proof["binding"]["identity"])) + ".json"):
                            return {"reconciled": False, "status": status}
                elif local_terminal_proof(config, proof["binding"]) is None:
                    return {"reconciled": False, "status": status}
            native = _runtime_terminal(checkpoint["runtime"])
            checkpoint.update(state="completed", runtimeProofs=native, finishedAt=time.time())
            write_json(config.state_dir / "shutdown.json", checkpoint)
            return {"reconciled": True, "status": status, "runtimeProofs": native}
        from .terminal import observe_terminal
        from tools.jenkins.pilot.journal import SubmissionJournal
        client = service._client(status["url"], config)
        journal = SubmissionJournal(config.pilot_root / "submissions.sqlite3") if checkpoint["bindings"] else None
        proofs = []
        for binding in checkpoint["bindings"]:
            proof = observe_terminal(config, client, binding, journal)
            if proof is None:
                return {"reconciled": False, "status": status}
            proofs.append(proof)
        if checkpoint["state"] == "completed":
            _runtime_terminal(checkpoint["runtime"])
            return {"reconciled": True, "status": status}
        if checkpoint["state"] == "refused":
            return {"reconciled": True, "failed": True, "status": status}
        # Reconciliation only observes cancellation; it never reissues a stop request.
        if client.api("/api/json?tree=quietingDown").get("quietingDown") is not True:
            raise TrayError("原停止操作的静默模式尚未确认，保留原记录")
        active, _ = service._builds(client, validate_runtime(config))
        if active:
            return {"reconciled": False, "status": status}
        terminal = {"verified": True, "proofs": proofs, "reconciled": True}
        return {"reconciled": True, **_finish_stop(config, checkpoint, terminal)}
    return {"reconciled": False, "status": status}
