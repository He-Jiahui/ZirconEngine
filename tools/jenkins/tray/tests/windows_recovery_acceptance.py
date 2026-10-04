"""Opt-in keeper-death recovery on this task's stopped, idle fixture only."""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import time
from ctypes import wintypes
from pathlib import Path

from tools.jenkins.pilot.governance import require_live_storage_owner
from tools.jenkins.pilot.native import process_identity as identity
from tools.jenkins.pilot.storage import physical_identity
from tools.jenkins.tray import native_windows as nw, service
from tools.jenkins.tray.config import load_config
from tools.jenkins.tray.operations import read_operation
from tools.jenkins.tray.persistence import read_json, write_json
from tools.jenkins.tray.tests.windows_ui_acceptance import post, wait_until


def run(config_path):
    config = load_config(config_path)
    if (config.config_path.name != "acceptance.json"
            or not config.pilot_root.name.startswith("jenkins-pilot-tray-acceptance-001-")):
        raise AssertionError("keeper fault injection is restricted to this owned acceptance fixture")
    status = service.read_status(config)
    if status["state"] != "ready" or status["activeBuilds"] or status["queuedCount"]:
        raise AssertionError("fault injection requires an idle fixture with no queued requests")
    telemetry = lambda: read_json(config.state_dir / "ui-runtime.json") or {}
    ui = wait_until(telemetry, lambda v: v.get("status") == "ready" and not v.get("closed"), 60)
    if not identity.process_matches_creation_time(ui["pid"], ui["birth"]):
        raise AssertionError("fixture tray identity is not live")
    prior_operation = read_operation(config)["operationId"]
    post(ui["hwnd"], nw.WM_COMMAND, 1004)
    stopped = wait_until(lambda: read_operation(config), lambda v: v.get("action") == "stop"
        and v.get("state") == "completed" and v["operationId"] != prior_operation, 160)
    wait_until(telemetry, lambda v: v.get("status") == "stopped", 60)
    owner = require_live_storage_owner(config.pilot_root, config.repo_root)
    if service.read_status(config)["state"] != "stopped":
        raise AssertionError("fixture services are not stopped before keeper injection")
    historical = {}
    for path in config.pilot_root.joinpath("runs").rglob("receipt.json"):
        historical[str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for path in config.pilot_root.joinpath("execution-intents").glob("*.json"):
        if read_json(path).get("status") != "complete":
            raise AssertionError("fixture has a nonterminal execution intent")
        historical[str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    historical[owner["stateFile"]] = hashlib.sha256(Path(owner["stateFile"]).read_bytes()).hexdigest()
    before = {"schemaVersion": 1, "owner": owner, "stoppedOperation": stopped,
              "historicalHashes": historical, "rootIdentity": dict(config.root_identity), "observedAt": time.time()}
    write_json(config.pilot_root / "evidence/tray-keeper-recovery-before.json", before)
    api = identity._kernel()
    api.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
    api.TerminateProcess.restype = wintypes.BOOL
    handle = api.OpenProcess(0x1000 | 0x100000 | 1, False, owner["owner_pid"])
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if identity._birth(api, handle) != owner["owner_process_creation_time"] or not identity._alive(api, handle):
            raise AssertionError("keeper handle no longer identifies the exact live fixture owner")
        if not api.TerminateProcess(handle, 70):
            raise ctypes.WinError(ctypes.get_last_error())
        if api.WaitForSingleObject(handle, 15000) != 0:
            raise AssertionError("owned fixture keeper did not signal exit")
    finally:
        api.CloseHandle(handle)
    if identity.process_matches_creation_time(owner["owner_pid"], owner["owner_process_creation_time"]):
        raise AssertionError("old keeper is still alive")
    wait_until(telemetry, lambda v: v.get("status") == "stopped", 60)
    post(ui["hwnd"], nw.WM_COMMAND, 1003)
    started = wait_until(lambda: read_operation(config), lambda v: v.get("action") == "start"
        and v.get("state") == "completed" and v["operationId"] != stopped["operationId"], 900)
    wait_until(telemetry, lambda v: v.get("status") == "ready", 60)
    current = require_live_storage_owner(config.pilot_root, config.repo_root)
    if current["authorityId"] == owner["authorityId"] or current.get("previousAuthorityId") != owner["authorityId"]:
        raise AssertionError("recovery did not append the exact previous authority")
    if physical_identity(config.pilot_root) != config.root_identity:
        raise AssertionError("recovery replaced the physical fixture root")
    for name, digest in historical.items():
        if hashlib.sha256(Path(name).read_bytes()).hexdigest() != digest:
            raise AssertionError("recovery changed a historical execution or original state byte")
    result = {"schemaVersion": 1, "accepted": True, "nativeKeeperDeath": True,
              "oldOwner": owner, "newOwner": current, "samePhysicalRoot": True,
              "historicalBytesUnchanged": True, "historicalCount": len(historical),
              "startedOperation": started, "recoveryAttempt": read_json(config.state_dir / "recovery-attempt.json"),
              "completedAt": time.time()}
    write_json(config.pilot_root / "evidence/tray-keeper-recovery.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(run(args.config), ensure_ascii=True))
