"""Retain the official strict desktop owner for a reviewed recovery release."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import signal
import subprocess
import sys
import time

REPO = Path(__file__).absolute().parents[2]
sys.dont_write_bytecode = True

PREFLIGHT_SHA = "6a85bde0adc22d61c15b68305e8e66360cf768722a18aa62c5dcb6a6cdb5266a"


def _bootstrap_plain(path):
    path = Path(path).absolute()
    if ".." in path.parts:
        raise ValueError("Recovery bootstrap paths must be canonical")
    for item in (path, *path.parents):
        if item.is_symlink() or (item.exists() and getattr(item.lstat(), "st_file_attributes", 0) & 0x400):
            raise ValueError("Recovery bootstrap paths must have physical identity")
    return path


def _bootstrap_raw(path):
    return _bootstrap_plain(path).read_bytes()


def _bootstrap_document(raw):
    def unique(pairs):
        value = {}
        for name, item in pairs:
            if name in value:
                raise ValueError("Recovery bootstrap document contains duplicate keys")
            value[name] = item
        return value
    value = json.loads(raw, object_pairs_hook=unique, parse_constant=lambda _: (_ for _ in ()).throw(
        ValueError("Recovery bootstrap document contains nonfinite values")))
    if not isinstance(value, dict):
        raise ValueError("Recovery bootstrap document must be an object")
    return value


def _bootstrap_snapshot(release):
    """Verify every immutable byte before importing the actor's tools package."""
    target = release.get("targetDriver")
    if not isinstance(target, dict) or not re.fullmatch("[0-9a-f]{64}", str(target.get("driverDigest", ""))):
        raise ValueError("A reviewed complete sealed target is required")
    root = REPO / ".jenkins/runtime/drivers" / target["driverDigest"]
    expected = {"root": str(root), "driverDigest": target["driverDigest"], "manifest": str(root / "driver-manifest.json"),
                "launcher": str(root / "launcher.py"), "steps": str(root / "pipeline/steps.groovy")}
    if target != expected or release.get("actorCodeRoot") != str(root):
        raise ValueError("The actor must load its exact reviewed sealed target")
    manifest_raw = _bootstrap_raw(target["manifest"])
    manifest = _bootstrap_document(manifest_raw)
    files = manifest.get("files")
    if type(manifest.get("schemaVersion")) is not int or manifest["schemaVersion"] != 1 or not isinstance(files, dict) or not files:
        raise ValueError("The actor manifest is unproven")
    identity = json.dumps({"schemaVersion": 1, "files": files}, ensure_ascii=False, sort_keys=True,
                          separators=(",", ":"), allow_nan=False).encode("utf-8")
    if hashlib.sha256(identity).hexdigest() != target["driverDigest"] or manifest.get("driverDigest") != target["driverDigest"]:
        raise ValueError("The actor manifest identity changed")
    for name, entry in files.items():
        relative = PurePosixPath(name)
        if (relative.as_posix() != name or relative.is_absolute() or any(part in {".", ".."} for part in relative.parts)
                or "\\" in name or ":" in name or not isinstance(entry, dict) or set(entry) != {"bytes", "sha256"}
                or type(entry["bytes"]) is not int or entry["bytes"] < 0
                or not re.fullmatch("[0-9a-f]{64}", str(entry["sha256"]))):
            raise ValueError("The actor manifest path or physical content proof is unproven")
        raw = _bootstrap_raw(root.joinpath(*relative.parts))
        if len(raw) != entry["bytes"] or hashlib.sha256(raw).hexdigest() != entry["sha256"]:
            raise ValueError("The sealed actor implementation changed")
    observed = set()
    for path in root.rglob("*"):
        _bootstrap_plain(path)
        if path.is_file():
            observed.add(path.relative_to(root).as_posix())
    if observed != set(files) | {"driver-manifest.json"} or _bootstrap_raw(target["manifest"]) != manifest_raw:
        raise ValueError("The sealed actor contains undeclared or changing files")
    required = {"tools/__init__.py", "tools/jenkins/contracts/__init__.py", "tools/jenkins/deployment/recovery_keeper.py",
                "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/owned_upgrade.py", "tools/jenkins/tray/__main__.py"}
    if release.get("lostRecoveryRuntime") is not None:
        required.add("tools/jenkins/deployment/lost_recovery_runtime.py")
    if not required <= set(files):
        raise ValueError("The sealed actor lifecycle and tray closure is incomplete")
    return root


def _load_runtime(release, release_ref):
    """One tools namespace keeps exact typed capabilities and public Stop valid."""
    global JenkinsError, _plain, prepare_lost_formal_owner_recovery, DeploymentManager, resolve_paths, load_spec, process_lock
    code_root = _bootstrap_snapshot(release) if release.get("actorCodeRoot") is not None else REPO
    if release.get("kind") == "formal-owned-recovery-upgrade-release" and code_root == REPO:
        raise ValueError("An owned successor needs the sealed actor bootstrap")
    if release.get("lostRecoveryRuntime") is not None and code_root == REPO:
        raise ValueError("A lost recovery runtime needs the sealed actor bootstrap")
    if code_root != REPO and any(name == "tools" or name.startswith("tools.") for name in sys.modules):
        raise ValueError("A sealed actor cannot inherit a different tools namespace")
    entry = str(REPO / "tools/dev/jenkins_recovery_keeper.py")
    if release.get("operatorFiles", {}).get(entry) != hashlib.sha256(_bootstrap_raw(entry)).hexdigest():
        raise ValueError("The stable recovery bootstrap changed")
    sys.path.insert(0, str(code_root))
    from tools.jenkins.contracts import JenkinsError
    from tools.jenkins.deployment.driver import _plain
    from tools.jenkins.deployment.lost_owner import prepare_lost_formal_owner_recovery
    from tools.jenkins.deployment.manager import DeploymentManager
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.state.locks import process_lock
    if code_root != REPO:
        for name, module in tuple(sys.modules.items()):
            if name == "tools" or name.startswith("tools."):
                location = getattr(module, "__file__", None)
                if location is None or not Path(location).absolute().is_relative_to(code_root):
                    raise ValueError("The actor loaded a mixed lifecycle implementation")
        os.environ.update(PYTHONPATH=str(code_root), ZIRCON_DRIVER_DIGEST=release["targetDriver"]["driverDigest"],
            ZIRCON_DRIVER_LAUNCHER=release["targetDriver"]["launcher"], JENKINS_PYTHON=str(REPO / ".jenkins/runtime/python/python.exe"),
            ZIRCON_REPO_ROOT=str(REPO),
            ZIRCON_REPOSITORY_ID=hashlib.sha256(json.dumps(str(REPO).casefold(), ensure_ascii=False,
                separators=(",", ":")).encode("utf-8")).hexdigest(), PYTHONDONTWRITEBYTECODE="1")
    return code_root


def parent_pid(pid):
    import ctypes
    from ctypes import wintypes
    class Basic(ctypes.Structure):
        _fields_ = [(name, ctypes.c_void_p) for name in ("reserved", "peb", "reserved2", "reserved3", "pid", "parent")]
    kernel, native = ctypes.WinDLL("kernel32", use_last_error=True), ctypes.WinDLL("ntdll")
    kernel.OpenProcess.argtypes, kernel.OpenProcess.restype = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD], wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    native.NtQueryInformationProcess.argtypes = [wintypes.HANDLE, wintypes.ULONG, ctypes.c_void_p, wintypes.ULONG, ctypes.c_void_p]
    handle = kernel.OpenProcess(0x1000, False, pid)
    if not handle:
        raise JenkinsError("recovery_host_unproven", "Cannot query the actual sandbox ancestry")
    try:
        value = Basic()
        if native.NtQueryInformationProcess(handle, 0, ctypes.byref(value), ctypes.sizeof(value), None) != 0:
            raise JenkinsError("recovery_host_unproven", "Actual sandbox ancestry is inconclusive")
        return int(value.parent or 0)
    finally:
        kernel.CloseHandle(handle)


def verify_sandbox_host(release, release_ref):
    from tools.jenkins.deployment.recovery_keeper import _full_identity
    from tools.jenkins.deployment.live_observations import _windows_command_line, _digest_args
    pinned = release.get("sandboxHost", {})
    host_path = Path(pinned.get("path", "")).absolute()
    installed = Path(os.environ["LOCALAPPDATA"]) / "OpenAI/Codex/bin"
    _plain(host_path)
    if (not host_path.is_relative_to(installed) or host_path.name != "codex.exe"
            or pinned.get("profile") != "zircon-strict" or pinned.get("includeManagedConfig") is not True
            or hashlib.sha256(host_path.read_bytes()).hexdigest() != pinned.get("sha256")):
        raise JenkinsError("recovery_host_changed", "The reviewed official strict host changed")
    record = Path(os.environ.get("ZIRCON_JENKINS_RECOVERY_HOST_RECORD", "")).absolute()
    if record.parent != REPO / ".jenkins/state/recovery-lifecycle/sandbox-hosts":
        raise JenkinsError("recovery_host_unproven", "The dispatcher must publish its exact sandbox host")
    _plain(record)
    deadline = time.monotonic() + 10
    while not record.exists() and time.monotonic() < deadline:
        time.sleep(.05)
    raw = record.read_bytes()
    value = json.loads(raw)
    host = value.get("host", {})
    expected = [str(host_path), "sandbox", "-P", "zircon-strict", "--include-managed-config", "-C", str(REPO), "--",
        str(REPO / ".jenkins/runtime/python/python.exe"), "-B", str(Path(__file__).absolute()),
        "--config-sha256", release["config"]["sha256"], "--release-file", release_ref["path"],
        "--release-sha256", release_ref["sha256"]]
    observed_args = list(_windows_command_line(host.get("pid", 0)))
    if (value.get("kind") != "official-strict-recovery-sandbox-host" or value.get("release") != release_ref
            or host.get("executable") != str(host_path) or _full_identity(host["pid"]) != host
            or observed_args not in (expected, [*expected, "--check-only"])
            or value.get("arguments") != observed_args or host.get("commandDigest") != _digest_args(tuple(observed_args))):
        raise JenkinsError("recovery_host_unproven", "The observed sandbox host differs from its reviewed launch")
    ancestor, ancestry = os.getpid(), []
    for _ in range(8):
        ancestor = parent_pid(ancestor)
        ancestry.append(ancestor)
        if ancestor == host["pid"]:
            break
        if ancestor <= 0:
            raise JenkinsError("recovery_host_unproven", "This keeper is outside the reviewed sandbox host")
    else:
        raise JenkinsError("recovery_host_unproven", "The sandbox host is outside the bounded actual ancestry")
    return {"record": {"path": str(record), "sha256": hashlib.sha256(raw).hexdigest()}, "host": host,
            "actualAncestry": ancestry, "officialHostSha256": pinned["sha256"]}


def reviewed_preflight():
    path = REPO / ".jenkins/state/migration-baseline/jenkins_desktop_holder.py"
    _plain(path)
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != PREFLIGHT_SHA:
        raise JenkinsError("recovery_preflight_changed", "The reviewed restriction probe changed")
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader("reviewed_recovery_preflight", loader=None))
    module.__file__ = str(path)
    sys.modules[module.__name__] = module
    exec(compile(raw, str(path), "exec"), module.__dict__)
    return module


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config-sha256", required=True)
    parser.add_argument("--release-file", type=Path, required=True)
    parser.add_argument("--release-sha256", required=True)
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--no-tray", action="store_true")
    args = parser.parse_args(argv)
    try:
        if Path(__file__).absolute() != REPO / "tools/dev/jenkins_recovery_keeper.py":
            raise ValueError("Use the reviewed canonical recovery bootstrap")
        release_path = args.release_file.absolute()
        if (str(release_path) != str(args.release_file) or ".." in release_path.parts
                or release_path.parent not in {REPO / ".jenkins/state/deployment", REPO / ".jenkins/state/migration-baseline",
                                               REPO / ".jenkins/state/recovery-lifecycle/history"}):
            raise ValueError("The reviewed release escaped its canonical evidence namespace")
        release_raw = _bootstrap_raw(release_path)
        if hashlib.sha256(release_raw).hexdigest() != args.release_sha256:
            raise ValueError("The reviewed release changed")
        release = _bootstrap_document(release_raw)
        release_ref = {"path": str(release_path), "sha256": args.release_sha256}
        _load_runtime(release, release_ref)
        if (release.get("kind") == "formal-owned-recovery-upgrade-release"
                or release.get("lostRecoveryRuntime") is not None):
            args.no_tray = True
    except Exception as error:
        print(json.dumps({"status": "recovery-blocked", "reasonCode": "recovery_bootstrap_unproven", "message": str(error),
                          "runtimeAcceptance": False, "cargoAcceptance": False}), flush=True)
        return 2
    temporary = REPO / ".jenkins/tmp"
    _plain(temporary)
    temporary.mkdir(parents=True, exist_ok=True)
    for name in ("TEMP", "TMP", "TMPDIR"):
        os.environ[name] = str(temporary)
    stopped = False
    keeper = None
    tray_stream = None
    baseline = reviewed_preflight()
    tag = str(time.time_ns())
    state_path = REPO / ".jenkins/state/recovery-lifecycle/holders" / (tag + ".json")
    state = {"schemaVersion": 1, "kind": "formal-recovery-desktop-holder", "automaticRestart": False,
             "runtimeAcceptance": False, "cargoAcceptance": False, "requestsResubmitted": False}

    def publish(status=None, evidence=None):
        if status is not None:
            state["status"] = status
        if evidence is not None:
            state.update(evidence)
        state["updatedAt"] = time.time()
        baseline.save(state_path, state)
        print(json.dumps({k: state[k] for k in ("status", "reasonCode", "updatedAt") if k in state}), flush=True)

    def request_stop(_signal, _frame):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGINT, request_stop)
    if hasattr(signal, "SIGBREAK"):
        signal.signal(signal.SIGBREAK, request_stop)
    try:
        if Path(__file__).absolute() != REPO / "tools/dev/jenkins_recovery_keeper.py":
            raise JenkinsError("recovery_entry_changed", "Use the reviewed canonical recovery entry")
        preflight = baseline.restriction_preflight(REPO, args.config_sha256)
        _plain(args.release_file)
        release_raw = args.release_file.read_bytes()
        if hashlib.sha256(release_raw).hexdigest() != args.release_sha256:
            raise JenkinsError("recovery_release_changed", "The reviewed release changed")
        release = json.loads(release_raw)
        if release["config"]["sha256"] != args.config_sha256:
            raise JenkinsError("recovery_config_changed", "The release restriction configuration differs")
        release_ref = {"path": str(args.release_file.absolute()), "sha256": args.release_sha256}
        host_evidence = verify_sandbox_host(release, release_ref)
        components = release.get("runtimeFiles", {})
        if not components:
            raise JenkinsError("recovery_runtime_unpinned", "The release must pin its runtime components")
        for component, checksum in components.items():
            path = Path(component).absolute()
            _plain(path)
            if not path.is_relative_to(REPO / ".jenkins/runtime") or hashlib.sha256(path.read_bytes()).hexdigest() != checksum:
                raise JenkinsError("recovery_runtime_changed", "The reviewed runtime component changed")
        spec = load_spec(REPO / ".jenkins/deployment-spec.json")
        manager = DeploymentManager(spec, resolve_paths(spec))
        from tools.jenkins.deployment.recovery_keeper import RecoveryKeeper, _release
        # Bootstrap verified the exact sealed actor before reading runtime evidence.
        _release(manager, release_ref, current_source=False)
        with process_lock(manager.paths.state / "deployment-start.lock", reason="deployment_start_busy"):
            if release.get("kind") == "formal-owned-recovery-upgrade-release":
                marker = release["ownedUpgrade"]
                if marker.get("variant") == "external-native-stop":
                    from tools.jenkins.deployment.owned_upgrade import prepare_external_native_stop_upgrade_start
                    plan = prepare_external_native_stop_upgrade_start(manager,
                        request_id=release["requestId"], successor_release_ref=release_ref,
                        original_blocked_descriptor_ref=marker["originalBlockedDescriptor"],
                        committed_command_ref=marker["committedCommand"],
                        external_native_stop_ref=marker["externalNativeStop"],
                        keeper_retirement_ref=marker["keeperRetirement"])
                elif marker.get("variant") is None:
                    from tools.jenkins.deployment.owned_upgrade import prepare_owned_upgrade_start
                    plan = prepare_owned_upgrade_start(manager, request_id=release["requestId"],
                        successor_release_ref=release_ref, stopped_descriptor_ref=marker["stoppedDescriptor"],
                        owned_stop_ref=marker["ownedStop"], keeper_retirement_ref=marker["keeperRetirement"])
                else:
                    raise JenkinsError("owned_recovery_upgrade_unproven", "Unknown owned transfer variant")
            elif release.get("lostRecoveryRuntime") is not None:
                from tools.jenkins.deployment.lost_recovery_runtime import prepare_lost_recovery_runtime
                from tools.jenkins.deployment.recovery_keeper import _reference
                marker = release["lostRecoveryRuntime"]
                plan = prepare_lost_recovery_runtime(
                    manager, request_id=release["requestId"], scope=release["scope"],
                    original_command_ref=marker["originalCommand"],
                    original_descriptor_ref=marker["originalDescriptor"],
                    reviewed_plan_ref=marker["plan"])
                if plan.document != _reference(manager.paths.state, marker["plan"]):
                    raise JenkinsError("recovery_owner_changed", "The reviewed departed recovery runtime changed")
            elif release.get("failedOwnStart") is not None:
                from tools.jenkins.deployment.lost_owner import prepare_failed_recovery_start
                plan = prepare_failed_recovery_start(manager, request_id=release["requestId"], scope=release["scope"],
                    original_command_ref=release["failedOwnStart"]["originalCommand"])
                from tools.jenkins.deployment.recovery_keeper import _failed_start_marker
                if plan.document != _failed_start_marker(manager.paths.state, release):
                    raise JenkinsError("recovery_owner_changed", "The reviewed failed-own-start interruption changed")
            else:
                plan = prepare_lost_formal_owner_recovery(manager, request_id=release["requestId"], scope=release["scope"])
        if plan.document["owner"] != release["expectedPreviousOwner"] or plan.document["lifecycleFence"] != release["expectedLifecycleFence"]:
            raise JenkinsError("recovery_owner_changed", "The exact reviewed interrupted runtime changed")
        publish("restricted-recovery-preflight-passed", {"restrictionPreflight": preflight,
                "officialSandboxHost": host_evidence,
                "reviewedRelease": {"path": str(args.release_file.absolute()), "sha256": args.release_sha256},
                "recoveryDigest": plan.recovery_digest if release["kind"] != "formal-owned-recovery-upgrade-release" else None,
                "ownedUpgradeDigest": plan.upgrade_digest if release["kind"] == "formal-owned-recovery-upgrade-release" else None,
                "actorCodeRoot": release.get("actorCodeRoot"), "serviceLifecycleExecuted": False})
        if args.check_only:
            return 0
        with process_lock(REPO / ".jenkins/state/recovery-lifecycle/owner.lock", reason="recovery_keeper_already_live"):
            keeper = RecoveryKeeper(manager, release_path=args.release_file, release_sha256=args.release_sha256,
                                    preflight=baseline.restriction_preflight, publish=publish,
                                    recovery_plan=plan if (release.get("failedOwnStart") is not None
                                                          or release.get("lostRecoveryRuntime") is not None) else None,
                                    upgrade_plan=plan if release["kind"] == "formal-owned-recovery-upgrade-release" else None)
            if release["kind"] == "formal-owned-recovery-upgrade-release":
                keeper.start_upgrade(plan)
            else:
                keeper.start(plan)
            if not args.no_tray:
                log = REPO / ".jenkins/logs" / ("recovery-tray-" + tag + ".log")
                log.parent.mkdir(parents=True, exist_ok=True)
                tray_stream = log.open("xb")
                tray = subprocess.Popen([sys.executable, "-B", "-m", "tools.jenkins.tray", "--formal", "--config", str(spec.path), "tray"],
                                        cwd=REPO, env=os.environ.copy(), stdin=subprocess.DEVNULL, stdout=tray_stream, stderr=tray_stream,
                                        creationflags=subprocess.CREATE_NO_WINDOW)
                from tools.jenkins.deployment.recovery_keeper import _full_identity
                tray_identity = _full_identity(tray.pid)
                membership = baseline.process_job_observation()
                if tray.pid not in membership["associatedJob"]["memberPids"]:
                    raise JenkinsError("recovery_tray_uncontained", "The tray escaped its retained outer Job")
                publish(None, {"trayIdentity": tray_identity, "trayJobObservation": membership, "trayExitStopsService": False})
            next_health = 0.0
            while True:
                if stopped:
                    stopped = False
                    keeper.stop()
                    publish("owned-recovery-runtime-stopped")
                    return 0
                keeper.pump()
                if time.monotonic() >= next_health:
                    publish(None, {"health": manager.health()})
                    next_health = time.monotonic() + 30
                time.sleep(.1)
    except Exception as error:
        publish("recovery-blocked", {"reasonCode": getattr(error, "code", type(error).__name__), "message": str(error)})
        # Once this actor owns a child, retaining the outer Job preserves the
        # original command and native evidence for explicit reconciliation.
        if keeper is not None:
            command_path = manager.paths.state / "recovery-lifecycle/command.json"
            command = json.loads(command_path.read_text(encoding="utf-8")) if command_path.exists() else {}
            owns_attempt = (command.get("keeper") == keeper.keeper and command.get("release") == keeper.release_ref
                            and command.get("status") in {"pending", "start-attempted", "stop-attempted", "unknown"})
            operation = manager._read() or {}
            if owns_attempt or operation.get("state") in {"starting", "running", "stopping"}:
                while True:
                    time.sleep(1)
        return 2
    finally:
        if tray_stream is not None:
            tray_stream.close()


if __name__ == "__main__":
    raise SystemExit(main())
