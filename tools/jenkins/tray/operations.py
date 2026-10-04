"""Serialize durable lifecycle operations; workers outlive a closed tray."""
from __future__ import annotations

import ctypes
import hashlib
import os
import secrets
import subprocess
import time
from contextlib import contextmanager
from ctypes import wintypes
from pathlib import Path
from typing import Mapping

from tools.jenkins.pilot.native.process_identity import process_creation_time, process_matches_creation_time
from .config import TrayConfig, TrayError, validate_runtime
from .persistence import read_json, write_json, safe_error

MUTATIONS = frozenset({"start", "stop", "reconcile"})
NONTERMINAL = frozenset({"reserved", "active", "pending"})


@contextmanager
def operation_lock(config: TrayConfig):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateMutexW.argtypes = [ctypes.c_void_p, wintypes.BOOL, wintypes.LPCWSTR]
    kernel.CreateMutexW.restype = wintypes.HANDLE
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.WaitForSingleObject.restype = wintypes.DWORD
    kernel.ReleaseMutex.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    name = "Global\\ZirconJenkinsTrayOperation-" + hashlib.sha256(
        os.path.normcase(str(config.pilot_root)).encode()).hexdigest()
    handle = kernel.CreateMutexW(None, False, name)
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    acquired = False
    try:
        result = kernel.WaitForSingleObject(handle, 0)
        if result not in (0, 0x80):
            raise TrayError("已有 Jenkins 启停操作，等待其完成后再操作")
        acquired = True
        yield
    finally:
        if acquired:
            kernel.ReleaseMutex(handle)
        kernel.CloseHandle(handle)


def read_operation(config: TrayConfig) -> dict | None:
    return read_json(config.state_dir / "operation.json")


def _save_operation(config: TrayConfig, value: dict) -> None:
    write_json(config.state_dir / "operations" / (value["operationId"] + ".json"), value)
    write_json(config.state_dir / "operation.json", value)


def _previous_pending(config: TrayConfig, previous: dict) -> dict:
    from . import service
    status = service.read_status(config)
    if previous["state"] in {"reserved", "active"}:
        pid, birth = previous.get("workerPid"), previous.get("workerCreationTime")
        if not isinstance(pid, int) or not isinstance(birth, str):
            raise TrayError("原操作缺少确切工作进程身份，必须保留并核对")
        if process_matches_creation_time(pid, birth):
            raise TrayError("原操作仍在运行，不能重复启动或停止")
    if hasattr(service, "reconcile"):
        result = service.reconcile(config, previous)
        if result.get("reconciled") is True:
            previous.update(state="failed" if result.get("failed") else "completed", result=result, finishedAt=time.time())
            _save_operation(config, previous)
            return result
    # Reaching a desired status does not prove cancelled resources/receipt release.
    if previous.get("action") == "start" and status.get("state") in {"ready", "busy"}:
        previous.update(state="completed", result={"reconciled": True, "status": status}, finishedAt=time.time())
        _save_operation(config, previous)
        return previous["result"]
    previous.update(state="pending", lastObservation=status)
    _save_operation(config, previous)
    raise TrayError("原操作尚未完成对账，已保留记录；不能创建重复操作")


def execute(config: TrayConfig, action: str, *, confirmation: Mapping | None = None) -> dict:
    validate_runtime(config)
    from . import service
    if action == "status":
        operation = read_operation(config)
        try:
            result = service.read_status(config)
        except Exception as error:
            result = {"state": "error", "message": safe_error(error), "canStart": False, "canStop": False}
        if operation and operation.get("state") in NONTERMINAL:
            result["operation"] = operation
            if operation.get("state") in {"reserved", "active"}:
                if process_matches_creation_time(operation["workerPid"], operation["workerCreationTime"]):
                    result["state"] = "starting" if operation["action"] == "start" else "stopping"
                    result["message"] = "正在启动 Jenkins" if operation["action"] == "start" else "正在停止 Jenkins"
                    result.update(canStart=False, canStop=False)
                else:
                    result.update(state="error", message="原启停操作需要对账", canStart=False, canStop=False)
            elif operation.get("state") == "pending":
                result.update(state="error", message="原启停操作尚未完成对账", canStart=False, canStop=False)
        return result
    if action == "stop-preview":
        return service.stop_preview(config)
    if action not in MUTATIONS:
        raise TrayError("未知托盘管理操作")
    with operation_lock(config):
        previous = read_operation(config)
        if previous and previous.get("state") in NONTERMINAL:
            return _previous_pending(config, previous)
        if action == "reconcile":
            return {"reconciled": True, "status": service.read_status(config)}
        operation = {"schemaVersion": 1, "operationId": secrets.token_hex(16), "action": action,
                     "state": "reserved", "workerPid": os.getpid(),
                     "workerCreationTime": process_creation_time(os.getpid()), "createdAt": time.time(),
                     "rootIdentity": dict(config.root_identity), "phase": "preflight"}
        if confirmation is not None:
            operation["confirmation"] = dict(confirmation)
        try:
            _save_operation(config, operation)
            if action == "start":
                from .worker_environment import load_msvc_environment
                os.environ.update(load_msvc_environment(config))
            operation.update(state="active", phase="effects", startedAt=time.time())
            _save_operation(config, operation)
            result = service.start(config) if action == "start" else service.stop(config, confirmation=confirmation)
            state = "completed" if result.get("pending") is not True else "pending"
            operation.update(state=state, result=result, finishedAt=time.time())
            _save_operation(config, operation)
            return {**result, "operationId": operation["operationId"]}
        except service.ConfirmationRequired as error:
            result = {"confirmationRequired": True, "preview": error.preview}
            operation.update(state="refused", result=result, finishedAt=time.time())
            _save_operation(config, operation)
            return result
        except Exception as error:
            # Effects may precede a lost response. Keep the exact operation for reconciliation.
            state = "failed" if operation["phase"] == "preflight" else "pending"
            import traceback
            frames = [{"file": f.filename, "line": f.lineno, "function": f.name}
                      for f in traceback.extract_tb(error.__traceback__)]
            operation.update(state=state, error=safe_error(error), errorType=type(error).__name__,
                             errorFrames=frames, finishedAt=time.time())
            _save_operation(config, operation)
            raise


def run_worker(config: TrayConfig, action: str, *, confirmation: Mapping | None = None) -> dict:
    validate_runtime(config)
    config.state_dir.mkdir(parents=True, exist_ok=True)
    config.logs_dir.mkdir(parents=True, exist_ok=True)
    nonce = secrets.token_hex(12)
    response = config.state_dir / "responses" / (nonce + ".json")
    arguments = [str(config.python_executable), "-B", str(config.launch_path), "--config", str(config.config_path),
                 "--response-file", str(response), action]
    if confirmation is not None:
        input_path = config.state_dir / "confirmations" / (nonce + ".json")
        write_json(input_path, dict(confirmation))
        arguments.extend(["--confirmation-file", str(input_path)])
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONUTF8="1",
                       PYTHONPATH=str(config.repo_root))
    from tools.jenkins.pilot.storage import require_managed_root
    scratch = require_managed_root(config.pilot_root / "tmp/tray", allow_root=False)
    scratch.mkdir(parents=True, exist_ok=True)
    environment.update(TEMP=str(scratch), TMP=str(scratch))
    logfile = config.logs_dir / ("worker-" + nonce + ".log")
    with logfile.open("xb", buffering=0) as log:
        process = subprocess.Popen(arguments, cwd=config.repo_root, env=environment, stdin=subprocess.DEVNULL,
                                   stdout=log, stderr=log, close_fds=True,
                                   creationflags=subprocess.CREATE_NO_WINDOW | subprocess.CREATE_NEW_PROCESS_GROUP)
    # The UI invokes this on a background thread. No pipe can be inherited by Jenkins.
    try:
        code = process.wait(timeout=900 if action == "start" else 300 if action in MUTATIONS else 45)
    except subprocess.TimeoutExpired:
        raise TrayError("管理进程仍未返回；已保留原操作，不会重复投递") from None
    result = read_json(response)
    if result is None:
        raise TrayError("管理进程没有返回有效记录，保留日志并核对原操作")
    if code or result.get("ok") is False:
        raise TrayError(str(result.get("error") or "管理操作失败"))
    return result["result"]
