"""Prepare one current-boot recovery through the selected sealed Jenkins lifecycle.

This entry does not terminate processes, rewrite management records or start the
service. The public strict coordinator starter consumes its reviewed release.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
from pathlib import Path
import sys
import time

REPO = Path(__file__).absolute().parents[2]
STATE = REPO / ".jenkins/state"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def plain(path):
    path = Path(path).absolute()
    require(".." not in path.parts, "A boot recovery path is not canonical")
    for item in (path, *path.parents):
        if item.is_symlink() or (item.exists() and getattr(item.lstat(), "st_file_attributes", 0) & 0x400):
            raise RuntimeError("Boot recovery paths must have physical identity")
    return path


def read(path):
    path = plain(path)
    require(path.stat().st_size <= 8 * 1024 * 1024, "Boot recovery evidence exceeds its bound")
    return json.loads(path.read_bytes())


def reference(path):
    path = plain(path)
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def checked(reference_value):
    require(isinstance(reference_value, dict) and set(reference_value) == {"path", "sha256"},
            "A complete immutable reference is required")
    require(reference(reference_value["path"]) == reference_value, "A boot recovery input changed")
    return read(reference_value["path"])


def immutable(path, document):
    path = plain(path)
    raw = (json.dumps(document, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")
    if path.exists():
        require(path.read_bytes() == raw, "Existing boot recovery evidence differs; reconcile its original attempt")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as output:
            output.write(raw)
            output.flush()
            os.fsync(output.fileno())
    return reference(path)


def strict_configuration():
    try:
        with (REPO / ".codex/config.toml").open("r+b"):
            pass
    except PermissionError:
        return True
    raise RuntimeError("Actual zircon-strict protected-config update-open denial is required")


def startup_document(path, sha256):
    startup_ref = {"path": str(plain(path)), "sha256": sha256}
    document = checked(startup_ref)
    require(document.get("schemaVersion") == 1 and document.get("kind") == "formal-current-boot-startup"
            and document.get("repository") == str(REPO)
            and document.get("profile") == "zircon-strict" and document.get("includeManagedConfig") is True,
            "Use the pinned formal strict startup descriptor")
    for name, item in document["files"].items():
        if name == "codex" and document.get("hostResolution"):
            continue
        checked_file = reference(item["path"])
        require(checked_file == item, "A pinned startup component changed")
    require(document["files"]["preparer"]["path"] == str(Path(__file__).absolute()),
            "The startup descriptor names a different preparer")
    return document, startup_ref


def resolved_host(startup, args):
    policy = startup.get("hostResolution")
    if policy is None:
        return startup["files"]["codex"], None
    expected_root = plain(Path(os.environ["LOCALAPPDATA"]) / "OpenAI/Codex/bin")
    require(policy.get("installedRoot") == str(expected_root)
            and policy.get("signerSubject") == 'CN="OpenAI OpCo, LLC", O="OpenAI OpCo, LLC", L=San Francisco, S=California, C=US'
            and policy.get("requiredFiles") == ["codex.exe", "codex-command-runner.exe", "codex-windows-sandbox-setup.exe", "codex-code-mode-host.exe"],
            "The startup host resolution policy changed")
    executable = plain(args.codex_executable)
    require(executable.name == "codex.exe" and executable.parent.parent == expected_root
            and re.fullmatch("[0-9a-f]{16}", executable.parent.name),
            "Use a complete official installed Codex version directory")
    expected = dict(zip(policy["requiredFiles"], [args.codex_sha256, args.runner_sha256,
                    args.setup_sha256, args.code_mode_host_sha256]))
    bundle = {}
    for name, checksum in expected.items():
        require(re.fullmatch("[0-9a-f]{64}", checksum or ""), "A complete signed native host resolution is required")
        bundle[name] = reference(executable.parent / name)
        require(bundle[name]["sha256"] == checksum, "The native signature-verified official host bundle changed")
    return bundle["codex.exe"], bundle


def sealed_runtime(descriptor, selected):
    previous_release = checked(descriptor["release"])
    target = previous_release["targetDriver"]
    require(all(selected.get(key) == value for key, value in target.items()),
            "Selected runtime and departed recovery release disagree")
    bootstrap = REPO / "tools/dev/jenkins_recovery_keeper.py"
    require(reference(bootstrap)["sha256"] == previous_release["operatorFiles"][str(bootstrap)],
            "The reviewed public recovery bootstrap changed")
    spec = importlib.util.spec_from_file_location("_verified_boot_recovery_bootstrap", bootstrap)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    root = module._bootstrap_snapshot(previous_release)
    require(not any(name == "tools" or name.startswith("tools.") for name in sys.modules),
            "The boot preparer inherited another tools namespace")
    sys.path.insert(0, str(root))
    return previous_release, target



def boot_namespace(startup, startup_ref, boot):
    root = plain(Path(startup["reportRoot"]) / ("boot-" + boot))
    require(root.is_relative_to(REPO / ".jenkins/state/management/reports"),
            "Boot evidence must remain in the registered report namespace")
    return root / ("startup-" + startup_ref["sha256"][:16])



def recovery_namespace(startup, startup_ref, boot, descriptor_ref):
    require(re.fullmatch("[0-9a-f]{64}", descriptor_ref["sha256"]) is not None,
            "The predecessor descriptor needs its exact content identity")
    request_id = hashlib.sha256((boot + "\0" + descriptor_ref["sha256"]
                                + "\0" + startup_ref["sha256"]).encode()).hexdigest()[:32]
    return request_id, boot_namespace(startup, startup_ref, boot) / ("request-" + request_id)

def generation_proof_path(startup, descriptor):
    return plain(Path(startup["reportRoot"]) / ("generation-observation-"
                 + str(descriptor["bootFileTime"]) + "-" + descriptor["runtimeReceipt"]["sha256"] + ".json"))


def generation_proof(manager, startup, descriptor, selected, command, release, target, *, live):
    """A new attempt needs the exact predecessor's committed, once-live issuance."""
    from tools.jenkins.deployment.lost_recovery_runtime import _historical_runtime
    from tools.jenkins.deployment.owned_upgrade import _committed_command
    from tools.jenkins.deployment.drain_observations import owner_observation
    from tools.jenkins.deployment.live_observations import observe_live_identity
    from tools.jenkins.deployment.recovery_keeper import _archive_bytes

    runtime = descriptor["runtime"]
    require(runtime["state"] == "running" and runtime["selected"] == selected,
            "The exact generation selector changed")
    receipt = _historical_runtime(manager, descriptor, release, target)
    _committed_command(STATE, descriptor, receipt, command)
    owner = {"operationId": selected["runtimeOperationId"], "generation": selected["runtimeOperationId"],
             "homePath": runtime["operation"]["homePath"], **runtime["owner"],
             "driverDigest": target["driverDigest"]}
    originals = {"descriptor": STATE / "formal-lifecycle/recovery-keeper.json",
                 "command": STATE / "recovery-lifecycle/command.json",
                 "selector": STATE / "deployment/driver.json"}
    expected_documents = {"descriptor": descriptor, "command": command, "selector": selected}
    input_refs = {name: reference(path) for name, path in originals.items()}
    if live:
        owner_observation(manager, expected=owner)
        observe_live_identity(descriptor["keeper"])
        archives = {}
        for name, path in originals.items():
            raw = path.read_bytes()
            require(json.loads(raw) == expected_documents[name]
                    and hashlib.sha256(raw).hexdigest() == input_refs[name]["sha256"],
                    "Current generation changed during its fresh owner observation")
            archives[name] = _archive_bytes(STATE, "resident-generation-" + name, raw)
        body = {"schemaVersion": 1, "kind": "strict-committed-live-generation-observation",
                "bootFileTime": str(descriptor["bootFileTime"]),
                "runtimeOperationId": selected["runtimeOperationId"], "driverDigest": target["driverDigest"],
                "keeper": descriptor["keeper"], "release": descriptor["release"],
                "runtimeReceipt": descriptor["runtimeReceipt"], "result": command["result"],
                "owner": owner, "inputs": input_refs, "archives": archives,
                "strictActualConfigRplusBDenied": True, "nativeOwnerObserved": True}
        path = generation_proof_path(startup, descriptor)
        if path.exists():
            old = read(path)
            require({key: value for key, value in old.items() if key != "createdAt"} == body,
                    "Existing exact-generation owner proof differs")
        else:
            immutable(path, {"createdAt": time.time(), **body})
        for name, path in originals.items():
            require(reference(path) == input_refs[name], "Generation changed before publishing its owner proof")
        return reference(generation_proof_path(startup, descriptor))
    proof_ref = reference(generation_proof_path(startup, descriptor))
    proof = validate_generation_proof(manager, proof_ref, expected_release=descriptor["release"])
    require(proof["inputs"] == input_refs and proof["runtimeReceipt"] == descriptor["runtimeReceipt"]
            and proof["runtimeOperationId"] == selected["runtimeOperationId"] and proof["owner"] == owner,
            "A same-boot successor requires this exact predecessor's committed and fresh live owner proof")
    return proof_ref


def validate_generation_proof(manager, proof_ref, *, expected_release):
    from tools.jenkins.deployment.lost_recovery_runtime import _historical_runtime
    from tools.jenkins.deployment.owned_upgrade import _committed_command

    proof = checked(proof_ref)
    require(proof.get("schemaVersion") == 1
            and proof.get("kind") == "strict-committed-live-generation-observation"
            and proof.get("nativeOwnerObserved") is True
            and proof.get("strictActualConfigRplusBDenied") is True
            and proof.get("release") == expected_release
            and set(proof.get("archives", {})) == {"descriptor", "command", "selector"}
            and set(proof.get("inputs", {})) == {"descriptor", "command", "selector"},
            "A complete strict live-generation proof is required")
    snapshots = {name: checked(ref) for name, ref in proof["archives"].items()}
    require(all(proof["archives"][name]["sha256"] == proof["inputs"][name]["sha256"] for name in snapshots),
            "Generation archives do not retain the original ownership bytes")
    descriptor, command, selected = (snapshots[name] for name in ("descriptor", "command", "selector"))
    release = checked(expected_release)
    receipt = _historical_runtime(manager, descriptor, release, release["targetDriver"])
    _committed_command(STATE, descriptor, receipt, command)
    runtime = descriptor["runtime"]
    owner = {"operationId": selected["runtimeOperationId"], "generation": selected["runtimeOperationId"],
             "homePath": runtime["operation"]["homePath"], **runtime["owner"],
             "driverDigest": selected["driverDigest"]}
    require(proof["bootFileTime"] == str(descriptor["bootFileTime"])
            and proof["runtimeOperationId"] == selected["runtimeOperationId"]
            and proof["driverDigest"] == selected["driverDigest"]
            and proof["keeper"] == descriptor["keeper"] and proof["runtimeReceipt"] == descriptor["runtimeReceipt"]
            and proof["result"] == command["result"] and proof["owner"] == owner
            and runtime["selected"] == selected,
            "Generation proof changed its committed native owner")
    return proof


def prior_intents(manager, startup, startup_ref, boot, current_release, current_generation_proof, *, write):
    """Resolve proven success; an unknown public intent never acquires a new request."""
    root = boot_namespace(startup, startup_ref, boot).parent
    if not root.exists():
        return
    candidates = []
    for directory in root.iterdir():
        plain(directory)
        if not directory.is_dir() or re.fullmatch("startup-[0-9a-f]{16}", directory.name) is None:
            continue
        legacy = directory / "launch-intent.json"
        if legacy.exists():
            candidates.append(legacy)
        for request in directory.iterdir():
            plain(request)
            if request.is_dir() and re.fullmatch("request-[0-9a-f]{32}", request.name):
                intent = request / "launch-intent.json"
                if intent.exists():
                    candidates.append(intent)
    require(len(candidates) <= 4096, "Recovery intent inventory exceeds its explicit evidence bound")
    for path in candidates:
        intent_ref, intent = reference(path), read(path)
        require(intent.get("kind") == "current-boot-public-start-intent"
                and intent.get("bootFileTime") == boot,
                "A prior native public intent is malformed")
        completion_path = path.with_name("committed-generation.json")
        if completion_path.exists():
            completion = read(completion_path)
            require(completion.get("kind") == "resolved-native-public-start-intent"
                    and completion.get("intent") == intent_ref,
                    "The prior public intent completion changed")
            validate_generation_proof(manager, completion["generationProof"], expected_release=intent["release"])
            continue
        require(current_generation_proof is not None and intent.get("release") == current_release,
                "An unresolved public start intent requires reconciliation without replay")
        proof = validate_generation_proof(manager, current_generation_proof, expected_release=intent["release"])
        require(proof["bootFileTime"] == boot, "The pending intent has no exact current-boot committed owner")
        if write:
            immutable(completion_path, {"schemaVersion": 1, "kind": "resolved-native-public-start-intent",
                      "intent": intent_ref, "generationProof": current_generation_proof,
                      "publicStartReplayed": False})


def execute(args):
    strict_configuration()
    startup, startup_ref = startup_document(args.startup_file, args.startup_sha256)
    host, host_bundle = resolved_host(startup, args)
    selected_path = STATE / "deployment/driver.json"
    descriptor_path = STATE / "formal-lifecycle/recovery-keeper.json"
    command_path = STATE / "recovery-lifecycle/command.json"
    descriptor, selected = read(descriptor_path), read(selected_path)
    command = read(command_path)
    old_release, target = sealed_runtime(descriptor, selected)
    from tools.jenkins.deployment.manager import DeploymentManager
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.deployment.lost_recovery_runtime import prepare_lost_recovery_runtime
    from tools.jenkins.deployment.recovery_keeper import _release
    from tools.jenkins.deployment.drain_observations import owner_observation
    from tools.jenkins.processes.boot import windows_boot_filetime
    from tools.jenkins.state.locks import process_lock

    spec = load_spec(REPO / ".jenkins/deployment-spec.json")
    manager = DeploymentManager(spec, resolve_paths(spec))
    boot = str(windows_boot_filetime())
    current_generation_proof = None
    if boot == str(descriptor.get("bootFileTime")):
        try:
            current_generation_proof = generation_proof(
                manager, startup, descriptor, selected, command, old_release, target, live=True)
        except Exception:
            current_generation_proof = None
        if current_generation_proof is not None:
            result = {"status": "already-running-no-action", "bootFileTime": boot,
                      "runtimeOperationId": selected["runtimeOperationId"],
                      "driverDigest": target["driverDigest"], "keeper": descriptor["keeper"],
                      "generationProof": current_generation_proof, "runtimeReceipt": descriptor["runtimeReceipt"],
                      "descriptor": reference(descriptor_path), "startup": startup_ref,
                      "strictActualConfigRplusBDenied": True, "serviceStartIssued": False}
            if args.observer_pid is not None:
                from tools.jenkins.processes.identity import current_identity
                observer = current_identity(args.observer_pid)
                require(observer.creation_time == args.observer_birth
                        and Path(observer.executable).name.casefold() == "powershell.exe",
                        "The native resident observer identity changed")
                result["observerHost"] = observer.to_dict()
                proof_path = Path(startup["reportRoot"]) / ("resident-observation-" + boot + "-"
                    + str(observer.pid) + "-" + observer.creation_time + "-"
                    + selected["runtimeOperationId"] + ".json")
                proof = {"schemaVersion": 1, "kind": "strict-current-owner-resident-observation", **result}
                if proof_path.exists():
                    existing = read(proof_path)
                    require({key: value for key, value in existing.items() if key != "createdAt"} == proof,
                            "Existing resident observation differs from this exact runtime")
                    result["observerEvidence"] = reference(proof_path)
                else:
                    result["observerEvidence"] = immutable(proof_path, {"createdAt": time.time(), **proof})
            return result
    require(not args.observe_only, "The exact current committed owner is not live; observation issues no recovery")
    if boot == str(descriptor.get("bootFileTime")):
        current_generation_proof = generation_proof(
            manager, startup, descriptor, selected, command, old_release, target, live=False)
    request_id, output = recovery_namespace(startup, startup_ref, boot, reference(descriptor_path))
    prior_intents(manager, startup, startup_ref, boot, descriptor["release"],
                  current_generation_proof, write=not args.check_only)
    if args.check_only:
        with process_lock(STATE / "deployment-start.lock", reason="deployment_start_busy"):
            plan = prepare_lost_recovery_runtime(manager, request_id=request_id)
        return {"status": "eligible-for-current-boot-recovery", "bootFileTime": boot,
                "requestId": request_id, "driverDigest": target["driverDigest"],
                "recoveryDigest": plan.recovery_digest, "strictActualConfigRplusBDenied": True,
                "serviceStartIssued": False}
    output.mkdir(parents=True, exist_ok=True)
    with process_lock(output / "prepare.lock", reason="boot_recovery_preparation_busy"):
        attempt_path = output / "attempt.json"
        if attempt_path.exists():
            attempt = read(attempt_path)
            require(attempt["requestId"] == request_id and attempt["startup"] == startup_ref
                    and attempt["originalDescriptor"] == reference(descriptor_path)
                    and attempt["originalCommand"] == reference(command_path),
                    "A prior current-boot attempt changed; reconcile it without a new request")
        else:
            attempt = {"schemaVersion": 1, "kind": "current-boot-recovery-attempt",
                       "requestId": request_id, "bootFileTime": boot, "startup": startup_ref,
                       "createdAt": time.time(), "originalDescriptor": reference(descriptor_path),
                       "originalCommand": reference(command_path), "originalSelector": reference(selected_path),
                       "strictActualConfigRplusBDenied": True, "serviceStartIssued": False,
                       "automaticRetry": False, "queueReplayRequested": False,
                       "predecessorGenerationProof": current_generation_proof}
            immutable(attempt_path, attempt)
        plan_path = STATE / "migration-baseline" / ("boot-runtime-plan-" + boot + "-" + request_id + ".json")
        with process_lock(STATE / "deployment-start.lock", reason="deployment_start_busy"):
            plan = prepare_lost_recovery_runtime(
                manager, request_id=request_id,
                original_command_ref=attempt["originalCommand"],
                original_descriptor_ref=attempt["originalDescriptor"],
                reviewed_plan_ref=reference(plan_path) if plan_path.exists() else None)
            plan_ref = immutable(plan_path, plan.document)
            old_preparation = checked(old_release["preparation"])
            preparation = {**old_preparation, "preparedAt": attempt["createdAt"],
                           "state": "prepared-not-selected", "driver": target,
                           "previousSelectedDriver": selected,
                           "previousSelectorSha256": plan.document["lifecycleFence"]["deployment/driver.json"],
                           "protectedDeploymentOperationId": plan.document["operationId"],
                           "protectedHostDigest": plan.document["hostRecordDigest"],
                           "protectedHolds": plan.document["executionQuarantine"]["protectedHolds"],
                           "regression": old_release["regression"], "bootRecoveryPlan": plan_ref,
                           "newServiceStarted": False, "windowsRestartRequested": False}
            preparation_ref = immutable(STATE / "migration-baseline" /
                ("boot-runtime-preparation-" + boot + "-" + request_id + ".json"), preparation)
            release = {**old_release, "kind": "formal-lost-owner-recovery-release",
                       "preparedAt": attempt["createdAt"], "requestId": request_id,
                       "scope": "execution-runtime", "actorCodeRoot": target["root"],
                       "expectedPreviousOwner": plan.document["owner"],
                       "expectedLifecycleFence": plan.document["lifecycleFence"],
                       "protectedHolds": plan.document["executionQuarantine"]["protectedHolds"],
                       "preparation": preparation_ref,
                       "lostRecoveryRuntime": {"plan": plan_ref,
                                              "originalCommand": plan.document["originalCommand"],
                                              "originalDescriptor": plan.document["originalDescriptor"]},
                       "automaticRestart": False, "queueReplayRequested": False,
                       "runtimeAcceptance": False, "cargoAcceptance": False}
            release["sandboxHost"] = {**release["sandboxHost"], **host}
            if host_bundle is not None:
                release["sandboxBundle"] = host_bundle
            release.pop("ownedUpgrade", None)
            release.pop("failedOwnStart", None)
            release_ref = immutable(STATE / "deployment" /
                ("boot-runtime-release-" + boot + "-" + request_id + ".json"), release)
            _release(manager, release_ref, current_source=False)
            plan.revalidate(manager)
        result = {"schemaVersion": 1, "kind": "reviewed-current-boot-recovery",
                  "status": "prepared-not-started", "bootFileTime": boot,
                  "requestId": request_id, "startup": startup_ref,
                  "predecessorGenerationProof": current_generation_proof,
                  "driverDigest": target["driverDigest"], "release": release_ref,
                  "preparation": preparation_ref, "plan": plan_ref, "attempt": reference(attempt_path),
                  "launchIntent": str(output / "launch-intent.json"),
                  "strictActualConfigRplusBDenied": True, "serviceStartIssued": False,
                  "automaticRetry": False, "runtimeAcceptance": False, "cargoAcceptance": False}
        result_ref = immutable(output / "prepared.json", result)
        return {**result, "prepared": result_ref}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--startup-file", type=Path, required=True)
    parser.add_argument("--startup-sha256", required=True)
    parser.add_argument("--codex-executable", type=Path)
    parser.add_argument("--codex-sha256")
    parser.add_argument("--runner-sha256")
    parser.add_argument("--setup-sha256")
    parser.add_argument("--code-mode-host-sha256")
    parser.add_argument("--observer-pid", type=int)
    parser.add_argument("--observer-birth")
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--observe-only", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = execute(args)
    except Exception as error:
        print(json.dumps({"status": "boot-recovery-blocked",
                          "reasonCode": getattr(error, "code", type(error).__name__),
                          "message": str(error), "serviceStartIssued": False,
                          "runtimeAcceptance": False}), flush=True)
        return 2
    print(json.dumps(result, ensure_ascii=False), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

