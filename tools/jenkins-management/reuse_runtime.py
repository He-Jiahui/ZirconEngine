"""Observe and reuse the selected healthy Jenkins owner without starting a holder."""
from __future__ import annotations
import argparse
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

def plain(path):
    path = Path(path).absolute()
    for part in (path, *path.parents):
        if part.exists() or part.is_symlink():
            if part.is_symlink() or getattr(part.lstat(), "st_file_attributes", 0) & 0x400:
                raise ValueError("Physical runtime paths required")
    return path

def observe(repo):
    repo = plain(repo)
    installed = json.loads(plain(repo / ".jenkins/state/management/installed.json").read_bytes())
    capsule = plain(installed["capsule"])
    if capsule.parent != repo / ".jenkins/runtime/management":
        raise ValueError("Management observer capsule escaped the runtime")
    raw = plain(capsule / "capsule.json").read_bytes()
    if hashlib.sha256(raw).hexdigest() != installed["capsuleSha256"] or capsule.name != installed["capsuleSha256"]:
        raise ValueError("Management observer identity changed")
    config = json.loads(raw)
    if set(config["files"]) not in ({"runner.py", "zircon-management.groovy"},
                                    {"runner.py", "source_preparation.py", "zircon-management.groovy"}):
        raise ValueError("Management observer file inventory changed")
    for name, checksum in config["files"].items():
        if hashlib.sha256(plain(capsule / name).read_bytes()).hexdigest() != checksum:
            raise ValueError("Management observer source changed")
    spec = importlib.util.spec_from_file_location("verified_live_jenkins_observer", capsule / "runner.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    selected_config, bound_repo, manager = module.boot(capsule)
    if bound_repo != repo or any(installed[k] != selected_config[k] for k in ("driverDigest", "runtimeOperationId")):
        raise ValueError("Observer differs from the selected runtime")
    from tools.jenkins.deployment.drain_observations import owner_observation
    from tools.jenkins.deployment.bootstrap import JenkinsBootstrap, _parse_node_config, _DESCRIPTION
    from tools.jenkins.tray.formal_native import identity as tray_identity
    owner = owner_observation(manager)
    health = manager.health()
    if health.get("ready") is not True:
        raise ValueError("The existing runtime health is not ready")
    selected = json.loads(plain(repo / ".jenkins/state/deployment/driver.json").read_bytes())
    snapshot = JenkinsBootstrap(manager.spec, manager.paths, manager)._snapshot(selected)
    base, count = manager.spec.agent["name"], manager.spec.agent["executors"]
    if snapshot["allNames"] != [base] or len(snapshot["nodes"]) != 1:
        raise ValueError("The sole fixed node inventory differs")
    node = snapshot["nodes"][0]
    live = _parse_node_config(base64.b64decode(node["configBase64"], validate=True))
    disk = _parse_node_config(base64.b64decode(node["diskConfigBase64"], validate=True))
    metadata = node["metadata"]
    if (live != disk or node["name"] != base or live["name"] != base
            or live["description"] != _DESCRIPTION or live["mode"] != "EXCLUSIVE"
            or live["label"] != base or live["remoteFS"] != str(manager.paths.agent.absolute())
            or live["numExecutors"] != str(count) or metadata["numExecutors"] != count
            or metadata["nodeClass"] != "hudson.slaves.DumbSlave"
            or metadata["offline"] is not False or metadata["channelConnected"] is not True):
        raise ValueError("The online fixed node is not the configured owned agent")
    code, body = manager._request("api/json?tree=numExecutors")
    if code != 200 or json.loads(body)["numExecutors"] != 0:
        raise ValueError("Controller executors must remain zero")
    release_ref = selected["recoveryRelease"]
    release_path = plain(release_ref["path"])
    if not release_path.is_relative_to(repo / ".jenkins/state/deployment"):
        raise ValueError("The selected runtime release escaped its namespace")
    release_raw = release_path.read_bytes()
    if hashlib.sha256(release_raw).hexdigest() != release_ref["sha256"]:
        raise ValueError("The selected runtime release changed")
    release = json.loads(release_raw)
    if release["targetDriver"]["driverDigest"] != selected_config["driverDigest"]:
        raise ValueError("The runtime release differs from its selected driver")
    command = json.loads(plain(repo / ".jenkins/state/recovery-lifecycle/command.json").read_bytes())
    if command.get("status") != "committed":
        raise ValueError("The existing keeper handoff is not committed")
    launcher = plain(repo / "tools/dev/jenkins_desktop_tray.py")
    pythonw = plain(repo / ".jenkins/runtime/python/pythonw.exe")
    for path, pins in ((launcher, release["operatorFiles"]), (pythonw, release["runtimeFiles"])):
        if hashlib.sha256(path.read_bytes()).hexdigest() != pins[str(path)]:
            raise ValueError("The selected desktop tray launcher changed")
    tray_ready = False
    try:
        ui = json.loads(plain(repo / ".jenkins/state/tray/ui.json").read_bytes())
        actual = tray_identity(ui["uiPid"]) or {}
        tray_ready = (str(actual.get("birth") or actual.get("creationTime")) == str(ui["uiBirth"])
            and os.path.normcase(actual["executable"]) == os.path.normcase(str(pythonw))
            and ui.get("ready") is True and ui.get("iconAdded") is True
            and ui.get("operation", {}).get("operationId") == owner["operationId"])
    except (OSError, ValueError, KeyError):
        pass
    owner_observation(manager, expected=owner)
    return {"state":"ready","serviceReady":True,"reusedExistingRuntime":True,
            "holderStarted":False,"bootstrapExecuted":False,
            "driverDigest":owner["driverDigest"],"runtimeOperationId":owner["operationId"],
            "controllerPid":owner["controller"]["pid"],"hostPid":owner["supervisor"]["pid"],
            "url":"http://" + manager.spec.controller["listenAddress"] + ":" + str(manager.spec.controller["httpPort"]) + "/",
            "fixedNode":base,"executors":count,"controllerExecutors":0,"trayReady":tray_ready,
            "trayLaunch":{"pythonw":str(pythonw),"launcher":str(launcher),
                          "release":str(release_path),"releaseSha256":release_ref["sha256"]}}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository-root", required=True)
    args = parser.parse_args()
    try:
        result = observe(args.repository_root)
    except Exception as error:
        print(json.dumps({"state":"error","reasonCode":getattr(error,"code","live_runtime_unproven"),
                          "message":str(error),"holderStarted":False},ensure_ascii=False))
        return 2
    print(json.dumps(result,ensure_ascii=False))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
