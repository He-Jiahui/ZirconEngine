"""Credential-free observations and guarded independent Jenkins lifecycle."""
from __future__ import annotations

import json
import time
import urllib.parse
from pathlib import Path
from typing import Mapping

from tools.jenkins.pilot import bootstrap
from tools.jenkins.pilot.client import JenkinsClient
from tools.jenkins.pilot.jenkins_config import PilotPaths
from tools.jenkins.pilot.native.paths import WorkerStorage
from tools.jenkins.pilot.native.process_identity import process_matches_creation_time

from .config import TrayConfig, TrayError, absolute_plain_path, validate_runtime
from .persistence import read_json, safe_error


class ConfirmationRequired(TrayError):
    def __init__(self, preview: Mapping[str, object]):
        super().__init__("停止 Jenkins 前需要确认当前运行任务")
        self.preview = dict(preview)


def _json(path: Path, limit: int = 2 * 1024 * 1024) -> dict:
    path = absolute_plain_path(path)
    try:
        with WorkerStorage(path.parent, create=False) as backend:
            if not backend.exists(path.name):
                return {}
            value = json.loads(backend.read_bytes(path.name, max_bytes=limit))
    except FileNotFoundError:
        return {}
    except (OSError, ValueError, UnicodeError) as exc:
        raise TrayError("Jenkins 状态文件不可读或身份无效") from exc
    if not isinstance(value, dict):
        raise TrayError("Jenkins 状态文件格式无效")
    return value


def _credentials(config: TrayConfig) -> dict[str, str]:
    value = _json(config.pilot_root / "credentials.json", 65536)
    if value.get("username") != "admin" or not isinstance(value.get("password"), str):
        raise TrayError("Jenkins 凭据文件不完整")
    return {"username": "admin", "password": value["password"]}


def _runtime(config: TrayConfig) -> tuple[dict, dict, str]:
    paths = PilotPaths(config.pilot_root)
    state, manifest = _json(paths.process_state_path), _json(paths.manifest_path)
    url = str(state.get("url") or manifest.get("controllerUrl") or "")
    if url:
        parsed = urllib.parse.urlsplit(url)
        if (parsed.scheme != "http" or parsed.hostname != "127.0.0.1" or not parsed.port
                or parsed.path != "/" or parsed.username or parsed.password or parsed.query or parsed.fragment):
            raise TrayError("Jenkins 地址没有绑定本机独立实例")
    return state, manifest, url


def _live(record: object) -> bool:
    if record is None:
        return False
    if not isinstance(record, dict):
        raise TrayError("Jenkins 进程身份记录无效")
    pid, birth = record.get("pid"), record.get("creationTime")
    if type(pid) is not int or pid <= 0 or not isinstance(birth, str) or not birth:
        raise TrayError("Jenkins 进程身份记录不完整")
    try:
        return process_matches_creation_time(pid, birth)
    except ProcessLookupError:
        return False
    except (OSError, ValueError) as exc:
        raise TrayError("无法确认 Jenkins 进程身份") from exc


def _runtime_evidence(record: object) -> bool:
    if not _live(record):
        return False
    try:
        from tools.jenkins.pilot.lifetime import wait_runtime_started
        wait_runtime_started(record, preparation_seconds=0)
    except (OSError, ValueError, ProcessLookupError, TimeoutError, KeyError) as exc:
        raise TrayError("Jenkins Java 与原生 keeper 身份无法确认") from exc
    return True


def _owner(config: TrayConfig) -> dict:
    owner = _json(config.pilot_root / "storage-owner.json", 65536)
    live = _live({"pid": owner.get("owner_pid"), "creationTime": owner.get("owner_process_creation_time")})
    if live:
        from tools.jenkins.pilot.governance import require_live_storage_owner
        owner = require_live_storage_owner(config.pilot_root, config.repo_root)
    return {"live": live, "pid": owner["owner_pid"], "creationTime": owner["owner_process_creation_time"],
            "authorityId": owner.get("authorityId")}


def _client(url: str, config: TrayConfig) -> JenkinsClient:
    if not url:
        raise TrayError("Jenkins 控制器地址尚未建立")
    return JenkinsClient(url, **_credentials(config), timeout=8)


def _builds(client: JenkinsClient, manifest: Mapping[str, object]) -> tuple[list[dict], int]:
    """Inspect every exported build; never treat truncated/invalid data as idle."""
    job = str(manifest.get("jobName") or manifest.get("job") or "zircon-pilot")
    jobs = client.api("/api/json?tree=jobs[name]").get("jobs")
    queue = client.api("/queue/api/json?tree=items[id,cancelled,task[name]]").get("items")
    if not isinstance(jobs, list) or not isinstance(queue, list):
        raise TrayError("Jenkins 任务或队列快照不完整")
    active = []
    for entry in jobs:
        if not isinstance(entry, dict) or not isinstance(entry.get("name"), str):
            raise TrayError("Jenkins 任务身份无法确认")
        name = entry["name"]
        payload = client.api(f"/job/{urllib.parse.quote(name, safe='')}/api/json?tree=allBuilds[number,building,result,actions[parameters[name,value]]]")
        builds = payload.get("allBuilds")
        if not isinstance(builds, list):
            raise TrayError("Jenkins 完整构建清单不可用；停止操作已禁用")
        for item in builds:
            if not isinstance(item, dict) or type(item.get("building")) is not bool:
                raise TrayError("Jenkins 构建状态无法确认")
            if not item["building"]:
                continue
            if type(item.get("number")) is not int or item["number"] < 1:
                raise TrayError("活动构建缺少确切编号")
            params = {}
            for action in item.get("actions", []):
                if isinstance(action, dict):
                    for parameter in action.get("parameters", []):
                        if not isinstance(parameter, dict) or not isinstance(parameter.get("name"), str):
                            raise TrayError("Jenkins 构建参数无效")
                        key = parameter["name"]
                        if key in params:
                            raise TrayError("Jenkins 构建参数重复")
                        params[key] = parameter.get("value")
            active.append({"job": name, "number": item["number"], "sessionId": params.get("SESSION_ID"),
                           "requestId": params.get("REQUEST_ID"), "attemptId": params.get("ATTEMPT_ID"),
                           "generation": params.get("GENERATION"), "known": name == job})
    queued = 0
    for item in queue:
        if not isinstance(item, dict) or type(item.get("id")) is not int:
            raise TrayError("Jenkins 排队任务身份无法确认")
        if not item.get("cancelled"):
            queued += 1
    return sorted(active, key=lambda b: (b["job"], b["number"])), queued


def read_status(config: TrayConfig) -> dict[str, object]:
    manifest = validate_runtime(config)
    state, _, url = _runtime(config)
    owner = _owner(config)
    controller_live = _runtime_evidence(state.get("controller"))
    agent_live = _runtime_evidence(state.get("agent"))
    result = {"schemaVersion": 1, "state": "stopped", "message": "Jenkins 已停止", "url": url,
              "owner": owner, "controller": {"live": controller_live, "ready": False},
              "agent": {"live": agent_live, "online": False}, "activeBuilds": [], "queuedCount": 0,
              "canStart": not controller_live, "canStop": controller_live or agent_live, "quietDown": False}
    if state.get("pendingLaunch"):
        result.update(state="error", message="原启动交接尚未完成，需要核对原记录", canStart=False, canStop=False)
        return result
    if not controller_live:
        if agent_live:
            result.update(state="degraded", message="控制器已停止，节点仍存活", canStart=False)
        if not owner["live"]:
            result["message"] = "Jenkins 已停止；下次启动先核验并恢复保管进程"
        return result
    try:
        client = _client(url, config)
        info = client.api("/api/json?tree=quietingDown,mode")
        if type(info.get("quietingDown")) is not bool:
            raise TrayError("Jenkins 静默状态无法确认")
        result["controller"]["ready"] = True
        result["quietDown"] = info["quietingDown"]
        node_name = str(manifest.get("agentName") or "zircon-windows-agent")
        node = client.api("/computer/" + urllib.parse.quote(node_name, safe="") + "/api/json?tree=offline,idle")
        if type(node.get("offline")) is not bool:
            raise TrayError("Jenkins 节点状态无法确认")
        online = node["offline"] is False and agent_live
        result["agent"]["online"] = online
        active, queued = _builds(client, manifest)
        result.update(activeBuilds=active, queuedCount=queued, canStart=not agent_live, canStop=True)
        if active:
            result.update(state="busy", message="Jenkins 正在运行构建")
        elif online:
            result.update(state="ready", message="Jenkins 已就绪")
        else:
            result.update(state="degraded", message="Jenkins 执行节点离线")
        if result["quietDown"]:
            result.update(state="stopping", message="Jenkins 处于静默模式，排队任务保留", canStart=False)
    except Exception as exc:
        result.update(state="error", message="Jenkins 身份或状态无法确认", error=safe_error(exc),
                      canStart=False, canStop=False)
    shutdown = read_json(config.state_dir / "shutdown-bindings.json")
    if shutdown and shutdown.get("state") in {"cancelling", "pending"}:
        # A completed lifecycle checkpoint may legitimately outlive the
        # terminal worker's pending marker.  Release the marker only when its
        # immutable root and bindings agree with the completed shutdown proof;
        # otherwise keep the service blocked for reconciliation.
        lifecycle = read_json(config.state_dir / "shutdown.json")
        consistent = (
            isinstance(lifecycle, dict)
            and lifecycle.get("state") == "completed"
            and lifecycle.get("rootIdentity") == config.root_identity
            and lifecycle.get("terminal", {}).get("verified") is True
            and shutdown.get("rootIdentity") == config.root_identity
            and shutdown.get("bindings") == lifecycle.get("bindings")
            and isinstance(shutdown.get("operationId"), str)
            and bool(shutdown["operationId"])
            and shutdown["operationId"] == lifecycle.get("operationId")
        )
        if not consistent:
            result.update(state="error", message="原停止操作待核对，记录已保留", canStart=False, canStop=False)
    return result


def stop_preview(config: TrayConfig) -> dict[str, object]:
    status = read_status(config)
    state, _, _ = _runtime(config)
    def public_record(value):
        return {key: value[key] for key in ("pid", "creationTime", "keeperPid", "keeperCreationTime")
                if key in value} if isinstance(value, dict) else {}
    return {"schemaVersion": 1, "confirmationRequired": bool(status["activeBuilds"]), "createdAt": time.time(),
            "rootIdentity": dict(config.root_identity), "controller": public_record(state.get("controller")),
            "agent": public_record(state.get("agent")), "activeBuilds": status["activeBuilds"],
            "queuedCount": status["queuedCount"], "status": status}


def start(config: TrayConfig) -> dict:
    from .lifecycle import start_runtime
    return start_runtime(config)


def _start_with_recovery(config: TrayConfig) -> dict:
    from .lifecycle import ensure_owner
    ensure_owner(config)
    return bootstrap.start_controller(config.pilot_root)


def stop(config: TrayConfig, *, confirmation: Mapping | None = None) -> dict:
    from .lifecycle import stop_runtime
    return stop_runtime(config, confirmation=confirmation)


def reconcile(config: TrayConfig, previous: Mapping[str, object]) -> dict:
    from .lifecycle import reconcile_runtime
    return reconcile_runtime(config, previous)
