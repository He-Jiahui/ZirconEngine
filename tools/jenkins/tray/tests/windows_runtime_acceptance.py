"""Opt-in real busy-stop and retained-queue acceptance on an owned fixture."""
from __future__ import annotations
import argparse
import json
import subprocess
import time
from pathlib import Path

from tools.jenkins.pilot.contracts import RequestIdentity
from tools.jenkins.pilot.deployment import client
from tools.jenkins.pilot.journal import SubmissionJournal
from tools.jenkins.pilot.runner import run_directory
from tools.jenkins.pilot.snapshot import capture
from tools.jenkins.pilot.storage import ManagedStorage
from tools.jenkins.pilot.native.process_identity import process_matches_creation_time
from tools.jenkins.tray.config import load_config
from tools.jenkins.tray.persistence import read_json, write_json
from tools.jenkins.tray.operations import read_operation
from tools.jenkins.tray import native_windows as nw
from tools.jenkins.tray.tests.windows_ui_acceptance import post, wait_until, dialog


def launch_tray(config):
    with (config.logs_dir / "acceptance-ui-launch.log").open("ab", buffering=0) as log:
        process = subprocess.Popen([str(config.pythonw_executable), "-B", str(config.launch_path), "--config",
            str(config.config_path), "tray"], cwd=config.repo_root, stdin=subprocess.DEVNULL, stdout=log,
            stderr=log, creationflags=subprocess.CREATE_NO_WINDOW)
    telemetry = lambda: read_json(config.state_dir / "ui-runtime.json") or {}
    ui = wait_until(telemetry, lambda v: not v.get("closed") and v.get("status") == "ready", 120)
    if not process_matches_creation_time(ui["pid"], ui["birth"]):
        raise AssertionError("real tray identity is not live")
    return ui


def run(config_path, case="002"):
    config = load_config(config_path)
    ui = launch_tray(config)
    api = client(config.pilot_root)
    journal = SubmissionJournal(config.pilot_root / "submissions.sqlite3")
    plan_path = config.pilot_root / "evidence" / ("tray-runtime-plan-" + case + ".json")
    plan = read_json(plan_path)
    if plan is None:
        name = "tools/jenkins_pilot/static_validation.py"
        sealed = capture(config.repo_root, ManagedStorage(config.pilot_root), paths=[name], untracked_allowlist=[name])
        running = RequestIdentity("tray-acceptance-001", "busy-stop-" + case, "attempt-001", 1, sealed.input_hash, "fault-probe")
        queued = RequestIdentity("tray-acceptance-001", "queue-survives-" + case, "attempt-001", 1, sealed.input_hash, "python-static")
        plan = {"bundle": str(sealed.bundle), "running": running.to_dict(), "queued": queued.to_dict()}
        write_json(plan_path, plan)
    running, queued = RequestIdentity.from_dict(plan["running"]), RequestIdentity.from_dict(plan["queued"])
    bundle = Path(plan["bundle"])
    api.submit("zircon-pilot", running, bundle, journal)
    api.submit("zircon-pilot", queued, bundle, journal)
    probe_path = run_directory(config.pilot_root, running) / "probe.json"
    def probe_observation():
        probe = read_json(probe_path)
        if not probe:
            return None
        if not all(process_matches_creation_time(probe[k + "Pid"], probe[k + "CreationTime"]) for k in ("parent", "child")):
            raise AssertionError("fault probe finished before busy stop could be tested")
        return probe
    probe = wait_until(probe_observation, bool, 600)
    remote_running = api.reconcile("zircon-pilot", running, journal)
    remote_queue = api.reconcile("zircon-pilot", queued, journal)
    if remote_running["state"] != "running" or remote_queue["state"] != "queued":
        raise AssertionError("acceptance lacks one running and one queued exact request")
    queue_id = remote_queue["queue_id"]
    telemetry = lambda: read_json(config.state_dir / "ui-runtime.json") or {}
    wait_until(telemetry, lambda v: v.get("status") == "busy", 30)
    original = read_operation(config)
    post(ui["hwnd"], nw.WM_COMMAND, 1004)
    cancelled_text = dialog(ui["pid"], "确认停止 Jenkins", 7, required_text=running.request_id, timeout=30)
    if read_operation(config)["operationId"] != original["operationId"]:
        raise AssertionError("cancelled confirmation created a mutation")
    post(ui["hwnd"], nw.WM_COMMAND, 1004)
    confirmed_text = dialog(ui["pid"], "确认停止 Jenkins", 6, required_text=running.request_id, timeout=30)
    def stop_observation():
        value = read_operation(config) or {}
        if value.get("action") == "stop" and value.get("state") == "pending":
            raise AssertionError("busy stop pending: " + str(value.get("error")))
        return value
    stopped = wait_until(stop_observation, lambda v: v.get("action") == "stop" and v.get("state") == "completed", 160)
    proof = stopped["result"]["proof"]
    if proof["terminal"].get("verified") is not True or not proof["terminal"]["proofs"]:
        raise AssertionError("busy stop has no exact local terminal proof")
    receipt_path = next((Path(item["receiptPath"]) for item in proof["terminal"]["proofs"]
                         if item["binding"]["identity"] == running.to_dict()), None)
    if receipt_path is None or read_json(receipt_path).get("outcome") != "cancelled":
        raise AssertionError("native fault execution completed naturally instead of being cancelled")
    if any(process_matches_creation_time(probe[k + "Pid"], probe[k + "CreationTime"]) for k in ("parent", "child")):
        raise AssertionError("cancelled descendants remain alive")
    for key in ("controller", "agent"):
        if proof["runtime"][key].get("activeProcesses") != 0 or proof["runtime"][key].get("logReleased") is not True:
            raise AssertionError("Jenkins runtime terminal proof missing")
    checkpoint = read_json(config.state_dir / "shutdown.json")
    if queue_id not in checkpoint["queueProof"]["queueIds"]:
        raise AssertionError("exact queued item was not preserved before stop")
    wait_until(telemetry, lambda v: v.get("status") == "stopped", 30)
    post(ui["hwnd"], nw.WM_COMMAND, 1003)
    started = wait_until(lambda: read_operation(config) or {}, lambda v: v.get("action") == "start" and v.get("state") == "completed" and v.get("operationId") != original["operationId"], 700)
    wait_until(telemetry, lambda v: v.get("status") in {"ready", "busy"}, 30)
    resumed = wait_until(lambda: api.reconcile("zircon-pilot", queued, journal), lambda v: v["state"] == "completed", 600)
    if resumed["queue_id"] != queue_id or resumed["result"] != "SUCCESS":
        raise AssertionError("retained queue identity did not complete after restart")
    from tools.jenkins.pilot.acceptance import verify_build
    queued_acceptance = verify_build(root=config.pilot_root, repo_root=config.repo_root, identity=queued,
                                    job="zircon-pilot", journal=journal)
    result = {"schemaVersion": 1, "cancelNo": True, "cancelYes": True, "confirmationText": confirmed_text,
              "cancelledConfirmationText": cancelled_text, "probe": probe, "stoppedOperation": stopped,
              "restartedOperation": started, "queueId": queue_id, "queuedResult": resumed,
              "queuedAcceptance": queued_acceptance, "completedAt": time.time(), "rootIdentity": dict(config.root_identity)}
    write_json(config.pilot_root / "evidence/tray-busy-stop-queue.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--case", default="002", choices=("002", "003", "004"))
    args = parser.parse_args()
    result = run(args.config, args.case)
    print(json.dumps({"passed": True, "queueId": result["queueId"]}, ensure_ascii=True))
