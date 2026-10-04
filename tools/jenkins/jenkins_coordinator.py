"""Explicit Jenkins coordinator enablement and exact-request management.

Operational enablement is separate from command, milestone and migration acceptance.
"""
from __future__ import annotations

import argparse
import json
import secrets
import sys
import time
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.jenkins.pilot.contracts import RequestIdentity
from tools.jenkins.pilot.deployment import client, verify_driver
from tools.jenkins.pilot.governance import require_live_storage_owner
from tools.jenkins.pilot.journal import SubmissionJournal
from tools.jenkins.tray.config import TrayError, absolute_plain_path, file_sha256, load_config, validate_runtime
from tools.jenkins.tray.persistence import read_json, safe_error, write_json
from tools.jenkins.tray.service import read_status


def _checked_evidence(path, acceptance):
    path = absolute_plain_path(path)
    value = read_json(path)
    if not value or value.get("accepted") is not True or value.get("acceptance") != acceptance:
        raise TrayError("协调器启用缺少匹配的已验收证据")
    return {"path": str(path), "sha256": file_sha256(path)}, value


def activate(config, *, tray_evidence, pilot_evidence):
    """Publish a new operational receipt without modifying historical acceptance."""
    validate_runtime(config)
    if config.python_executable.name.casefold() != "python.exe" or not config.python_executable.is_file():
        raise TrayError("协调器必须绑定已配置的 python.exe")
    status = read_status(config)
    if status["state"] != "ready" or not status["owner"]["live"] or status["quietDown"]:
        raise TrayError("正式启用前 Jenkins 必须通过原生身份、认证与节点就绪检查")
    owner = require_live_storage_owner(config.pilot_root, config.repo_root)
    driver = verify_driver(config.pilot_root)
    tray_reference, tray = _checked_evidence(tray_evidence, "jenkins-windows-tray")
    pilot_reference, pilot = _checked_evidence(pilot_evidence, "independent-jenkins-pilot")
    if (pilot.get("root") != str(config.pilot_root) or pilot.get("rootIdentity") != config.root_identity
            or pilot.get("driver", {}).get("driverInputHash") != driver
            or pilot.get("gates") != {f"I{i}": "passed" for i in range(1, 7)}):
        raise TrayError("独立 Jenkins 证据没有绑定当前根和驱动")
    if tray.get("driverInputHash") != driver:
        raise TrayError("托盘验收没有使用当前驱动")
    for name, expected in tray.get("sourceHashes", {}).items():
        path = absolute_plain_path(config.repo_root / name)
        if not path.is_relative_to(config.repo_root) or file_sha256(path) != expected:
            raise TrayError("托盘源码与实际验收版本不一致")
    if not tray.get("sourceHashes") or tray.get("allRequirementsPassed") is not True:
        raise TrayError("托盘验收门槛尚未全部通过")
    marker = config.repo_root / ".codex/coordinator-retirement.json"
    if file_sha256(marker) != pilot.get("retirementMarkerSha256"):
        raise TrayError("旧协调器退役标记已变化，启用操作拒绝")
    path = config.repo_root / ".codex/state/jenkins-coordinator/active.json"
    prior = read_json(path)
    if prior:
        if prior.get("configFile") != str(config.config_path) or prior.get("rootIdentity") != config.root_identity:
            raise TrayError("已有协调器绑定不同身份，必须先核对原记录")
        current = require_active(config.repo_root)
        return {"enabled": True, "alreadyEnabled": True, "activationId": current[1]["activationId"]}
    record = {"schemaVersion": 1, "enabled": True, "functionalTestsAllowed": True,
              "activationId": secrets.token_hex(16), "activatedAt": time.time(), "configFile": str(config.config_path),
              "repoRoot": str(config.repo_root), "root": str(config.pilot_root), "rootIdentity": dict(config.root_identity),
              "driverInputHash": driver, "ownerAtActivation": owner, "trayEvidence": tray_reference,
              "pilotEvidence": pilot_reference, "retirementMarkerSha256": file_sha256(marker),
              "scope": "independent-jenkins-command-coordination", "migrationAcceptance": False,
              "milestoneAcceptance": False, "wholeWorkspaceAcceptance": False}
    write_json(path.parent / "activations" / (record["activationId"] + ".json"), record)
    write_json(path, record)
    return {"enabled": True, "functionalTestsAllowed": True, "activationId": record["activationId"],
            "record": str(path), "url": status["url"]}


def require_active(repo_root):
    repo_root = absolute_plain_path(repo_root, directory=True)
    record = read_json(repo_root / ".codex/state/jenkins-coordinator/active.json")
    if not record or record.get("enabled") is not True or record.get("functionalTestsAllowed") is not True:
        raise TrayError("Jenkins 协调器尚未正式启用")
    config = load_config(Path(record["configFile"]))
    if (config.repo_root != repo_root or record.get("repoRoot") != str(repo_root)
            or record.get("root") != str(config.pilot_root) or record.get("rootIdentity") != config.root_identity):
        raise TrayError("Jenkins 协调器配置或物理根身份已变化")
    validate_runtime(config)
    require_live_storage_owner(config.pilot_root, config.repo_root)
    if verify_driver(config.pilot_root) != record["driverInputHash"]:
        raise TrayError("Jenkins 协调器驱动身份已变化")
    for key in ("trayEvidence", "pilotEvidence"):
        reference = record[key]
        if file_sha256(absolute_plain_path(reference["path"])) != reference["sha256"]:
            raise TrayError("Jenkins 协调器验收回执字节已变化")
    tray_path = absolute_plain_path(record["trayEvidence"]["path"])
    tray = read_json(tray_path)
    if not isinstance(tray, dict) or tray.get("accepted") is not True or tray.get("acceptance") != "jenkins-windows-tray":
        raise TrayError("托盘验收回执不再处于已验收状态")
    if tray.get("driverInputHash") != record["driverInputHash"] or tray.get("allRequirementsPassed") is not True:
        raise TrayError("托盘验收门槛或驱动身份已变化")
    for name, expected in tray.get("sourceHashes", {}).items():
        path = absolute_plain_path(repo_root / name)
        if not path.is_relative_to(repo_root) or file_sha256(path) != expected:
            raise TrayError("托盘源码与激活时验收版本不一致")
    if file_sha256(repo_root / ".codex/coordinator-retirement.json") != record["retirementMarkerSha256"]:
        raise TrayError("旧协调器退役状态已变化")
    return config, record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    sub = parser.add_subparsers(dest="action", required=True)
    enabling = sub.add_parser("activate")
    enabling.add_argument("--config", type=Path, required=True)
    enabling.add_argument("--tray-evidence", type=Path, required=True)
    enabling.add_argument("--pilot-evidence", type=Path, required=True)
    sub.add_parser("status")
    for action in ("submit", "reconcile", "cancel", "verify"):
        command = sub.add_parser(action)
        command.add_argument("--request-file", type=Path, required=True)
        command.add_argument("--job", default="zircon-pilot")
        if action == "submit":
            command.add_argument("--bundle", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.action == "activate":
            config = load_config(args.config)
            if Path(sys.executable) != config.python_executable:
                raise TrayError("协调器入口必须由已配置的 python.exe 执行")
            result = activate(config, tray_evidence=args.tray_evidence, pilot_evidence=args.pilot_evidence)
        else:
            config, record = require_active(args.repo_root)
            if Path(sys.executable) != config.python_executable:
                raise TrayError("协调器入口必须由已配置的 python.exe 执行")
            if args.action == "status":
                result = {"enabled": True, "functionalTestsAllowed": True, "activationId": record["activationId"],
                          "scope": record["scope"], "status": read_status(config)}
            else:
                identity = RequestIdentity.from_dict(read_json(absolute_plain_path(args.request_file)))
                journal = SubmissionJournal(config.pilot_root / "submissions.sqlite3")
                api = client(config.pilot_root)
                if args.action == "submit":
                    live = read_status(config)
                    if live["state"] not in {"ready", "busy"} or live.get("quietDown") or live.get("agent", {}).get("online") is not True:
                        raise TrayError("Jenkins 当前未就绪或已进入静默模式")
                    result = api.submit(args.job, identity, absolute_plain_path(args.bundle), journal)
                elif args.action == "reconcile":
                    result = api.reconcile(args.job, identity, journal)
                elif args.action == "cancel":
                    live = read_status(config)
                    if live.get("controller", {}).get("ready") is not True:
                        raise TrayError("Jenkins 控制器当前不可用于取消请求")
                    result = api.cancel(args.job, identity, journal)
                else:
                    from tools.jenkins.pilot.acceptance import verify_build
                    result = verify_build(root=config.pilot_root, repo_root=config.repo_root, identity=identity,
                                          job=args.job, journal=journal)
        print(json.dumps({"ok": True, "result": result}, ensure_ascii=True, sort_keys=True))
        return 0
    except Exception as error:
        print(json.dumps({"ok": False, "error": safe_error(error)}, ensure_ascii=True))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
