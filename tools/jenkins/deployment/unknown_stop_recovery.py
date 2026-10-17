"""One separately identified native attempt after an exact unknown public Stop."""
from __future__ import annotations

import argparse
from contextlib import closing
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
from types import SimpleNamespace

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..processes.boot import windows_boot_filetime
from ..state.locks import process_lock
from . import owned_upgrade as upgrade, recovery_keeper, lost_owner, lost_recovery_runtime
from .drain_observations import api_json, executor_observation, owner_observation, COMPUTER_QUERY
from .execution_activation import protected_holds

VARIANT = "external-native-stop-after-unknown"
KIND = "unknown-stop-native-recovery"
_ISSUED = object()
_ANCHORS = {"originalBlockedDescriptor": "formal-lifecycle/recovery-keeper.json",
            "unknownStopCommand": "recovery-lifecycle/command.json",
            "previousOperation": "deployment-operation.json",
            "previousHost": "deployment/host.json", "previousSelector": "deployment/driver.json"}


def validate_unknown_command(state, descriptor, origin, command):
    """The old delivery stays unresolved; this does not credit it with a Stop."""
    require = upgrade._require
    require(command.get("schemaVersion") == 1 and command.get("kind") == upgrade._COMMAND_KIND
            and command.get("status") == "unknown" and command.get("reasonCode") == "controller_unreachable"
            and command.get("keeper") == descriptor.get("keeper") and command.get("release") == descriptor.get("release")
            and command.get("previousRuntimeReceipt") == descriptor.get("runtimeReceipt")
            and command.get("recoveryDigest") == descriptor.get("recoveryDigest")
            and command.get("ownedUpgradeDigest") == descriptor.get("ownedUpgradeDigest")
            and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False
            and "result" not in command and origin.get("kind") == "recovery-keeper-runtime",
            "Only the exact unresolved pre-native controller-unreachable Stop is eligible")
    request, _ = upgrade._reference(state, command.get("request"))
    request_id = request.get("requestId")
    idle = {**descriptor, "admission": "idle"}
    require(request.get("schemaVersion") == 1 and request.get("kind") == upgrade._REQUEST_KIND
            and request.get("action") == "stop" and re.fullmatch("[0-9a-f]{32}", str(request_id))
            and request.get("keeper") == descriptor.get("keeper") and request.get("release") == descriptor.get("release")
            and request.get("runtimeReceipt") == descriptor.get("runtimeReceipt")
            and request.get("protectedHoldsDigest") == digest(descriptor.get("protectedHolds"))
            and request.get("descriptorSha256") == upgrade._sha(canonical_json(idle) + b"\n")
            and request.get("runtimeAcceptance") is False
            and upgrade._document(upgrade._raw(state / "recovery-lifecycle/requests" / (request_id + ".json"))) == command["request"]
            and not (state / "recovery-lifecycle/results" / (request_id + ".json")).exists(),
            "Unknown Stop request, preceding descriptor or missing result changed")
    return command["request"]


def _native_inventory(state, runtime):
    values = {}
    for path in sorted((state / "deployment/commands").glob("stop-*.json")):
        raw = upgrade._raw(path)
        upgrade._require(upgrade._document(raw).get("operationId") != runtime,
                         "This runtime already has a native Stop; no fresh attempt is permitted")
        values[str(path)] = upgrade._sha(raw)
    return values


def _unknown_native_rows(state, expected=None):
    path = state / "coordination.sqlite3"
    upgrade._plain(path)
    with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as connection:
        connection.execute("PRAGMA query_only=ON")
        connection.execute("BEGIN")
        rows = []
        for key, version, raw in connection.execute(
                "SELECT key,version,payload FROM records WHERE domain='native_job' ORDER BY key"):
            payload = upgrade._document(raw)
            if (key in {item["key"] for item in expected} if expected is not None
                    else payload.get("status") == "running" and not payload.get("completeProof")):
                data = raw.encode("utf-8") if isinstance(raw, str) else raw
                rows.append({"key": key, "version": version, "rawPayloadSha256": upgrade._sha(data)})
    upgrade._require(len(rows) == 3 and (expected is None or rows == expected), "The three protected historical unknown native records changed")
    return rows


def _quiet_activity(manager):
    jobs = api_json(manager, "api/json?tree=quietingDown,jobs[name,builds[building]]")

    computers = executor_observation(api_json(manager, COMPUTER_QUERY))
    queue = api_json(manager, "queue/api/json?tree=items[id]")
    upgrade._require(type(jobs.get("quietingDown")) is bool and isinstance(jobs.get("jobs"), list)
                     and all(isinstance(job, dict) and isinstance(job.get("builds"), list)
                             and all(isinstance(build, dict) and type(build.get("building")) is bool
                                     for build in job["builds"]) for job in jobs["jobs"])
                     and isinstance(queue.get("items"), list), "Current Jenkins activity is unobservable")
    building = [job.get("name") for job in jobs["jobs"] if any(build["building"] for build in job["builds"])]
    upgrade._require(not building and not computers["busyExecutors"] and not queue["items"],
                     "Current Jenkins jobs, executors or queue are active")
    return {"buildingJobs": [], "busyExecutors": 0, "queueItems": [],
            "quietingDownObserved": jobs["quietingDown"], "executorObservation": computers}


def _capture(manager, pins):
    repo, state, home = upgrade._paths(manager)
    raws = {key: upgrade._raw(state / name) for key, name in _ANCHORS.items()}
    descriptor, command = (upgrade._document(raws[name]) for name in
                           ("originalBlockedDescriptor", "unknownStopCommand"))
    runtime = descriptor.get("runtime", {})
    require = upgrade._require
    require(upgrade._sha(raws["originalBlockedDescriptor"]) == pins["descriptorSha256"]
            and upgrade._sha(raws["unknownStopCommand"]) == pins["commandSha256"]
            and descriptor.get("schemaVersion") == 1 and descriptor.get("kind") == upgrade._DESCRIPTOR_KIND
            and descriptor.get("admission") == "blocked" and runtime.get("state") == "running"
            and descriptor.get("automaticRestart") is False
            and all(descriptor.get(name) is False for name in upgrade._FLAGS[1:])
            and descriptor.get("keeper") == pins["keeper"]
            and runtime.get("operation", {}).get("operationId") == pins["runtimeOperationId"]
            and runtime.get("selected", {}).get("driverDigest") == pins["driverDigest"]
            and descriptor.get("bootFileTime") == str(windows_boot_filetime()),
            "Unknown Stop pins do not bind the exact blocked same-boot runtime")
    release, _ = upgrade._reference(state, descriptor["release"], release=True)
    context = (None if all(release.get(field) == fact for field, fact in
                           upgrade._configuration_references(manager).items())
               else upgrade._historical_configuration(manager, descriptor))
    historical_kwargs = {} if context is None else {"historical_configuration": context}
    release, target = upgrade._historical_release(manager, descriptor["release"], **historical_kwargs)
    upgrade._live_historical_operators(manager, release)
    native = manager
    if context is not None:
        from .manager import DeploymentManager
        from .paths import resolve_paths
        from .spec import DeploymentSpec
        spec = DeploymentSpec(context.validate(manager, descriptor["release"]), manager.spec.path)
        native = DeploymentManager(spec, resolve_paths(spec), manager.java, manager.war)
        require(native.paths == manager.paths, "Historical native layout changed")
    origin = recovery_keeper._runtime_receipt(native, descriptor, release, target, **historical_kwargs)
    pending = validate_unknown_command(state, descriptor, origin, command)
    require(pins["requestId"] != upgrade._reference(state, pending)[0]["requestId"],
            "The new native request must differ from the unknown public Stop")
    require(recovery_keeper._full_identity(pins["keeper"]["pid"]) == pins["keeper"], "Keeper identity changed")
    recovery_keeper._keeper_argv(repo, pins["keeper"], descriptor["release"], release["config"]["sha256"])
    require(runtime["selected"].get("recoveryRelease") == descriptor["release"], "Original runtime release changed")
    for key, expected in (("previousOperation", runtime["operation"]), ("previousHost", runtime["host"]),
                          ("previousSelector", runtime["selected"])):
        require(upgrade._document(raws[key]) == expected, "Current native anchors differ from the running issuance")
    observed = owner_observation(native)
    require({key: observed[key] for key in ("supervisor", "controller", "agent")} == runtime["owner"]
            and observed["driverDigest"] == pins["driverDigest"], "Live native owner changed")
    owner = {"operation": {key: runtime["operation"][key] for key in
             ("operationId", "generation", "hostPid", "homePath", "component", "controlPlaneOnly")}, **runtime["owner"]}
    execution = lost_owner._execution_gate(native, {"owner": owner, "selected": runtime["selected"],
                "anchor": SimpleNamespace(value={"protectedHolds": release["protectedHolds"]})})
    require(len(execution["protectedHolds"]) == 4, "All four historical holds must remain")
    rows = lost_recovery_runtime.execution_inventory(state)

    management = lost_recovery_runtime._management_quarantine(native, owner, runtime["selected"], rows)
    deliveries = upgrade._external_delivery_inventory(state, **historical_kwargs, unknown_request=pending)
    activity = _quiet_activity(native)
    native_inventory = _native_inventory(state, pins["runtimeOperationId"])
    unknown = _unknown_native_rows(state)
    fence = lost_owner._lifecycle_fence(native)
    require(all(fence.get(name) == release["expectedLifecycleFence"].get(name) for name in upgrade._LEGACY),
            "Historical lifecycle fence changed")
    require(all(upgrade._raw(state / _ANCHORS[key]) == raw for key, raw in raws.items())
            and descriptor["bootFileTime"] == str(windows_boot_filetime()), "Owner changed during preparation")
    value = {"schemaVersion": 1, "kind": "unknown-stop-native-preflight", "pins": pins,
             "originalRequest": pending, "bootFileTime": descriptor["bootFileTime"], "release": descriptor["release"],
             "anchors": {key: upgrade._ref(state / _ANCHORS[key], raw) for key, raw in raws.items()},
             "executionQuarantine": execution, "managementQuarantine": management, "activity": activity,
             "nativeStopInventory": native_inventory, "unknownNativeRows": unknown, "lifecycleFence": fence,
             "deliveryInventory": deliveries, "historicalConfiguration": None if context is None else context.document}
    return value, raws, native


@dataclass(frozen=True, init=False)
class UnknownStopNativeRecovery:
    snapshot: bytes
    _token: object

    def __init__(self, snapshot, token=None):
        upgrade._require(token is _ISSUED and type(snapshot) is bytes, "Only preparation issues a native recovery capability")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)

    @property
    def document(self):
        return upgrade._document(self.snapshot)

    def revalidate(self, manager):
        upgrade._require(type(self) is UnknownStopNativeRecovery and self._token is _ISSUED,
                         "An exact issued native recovery capability is required")
        current, raws, native = _capture(manager, self.document["pins"])
        upgrade._require(canonical_json(current) == self.snapshot, "Unknown Stop recovery owner or gate changed")
        return current, raws, native


def prepare_unknown_stop_native_recovery(manager, *, request_id, expected_driver_digest,
        expected_runtime_operation_id, expected_keeper, expected_command_sha256, expected_descriptor_sha256):
    upgrade._require(isinstance(request_id, str) and re.fullmatch("[0-9a-f]{32}", request_id),
                     "A fresh separate native request ID is required")
    pins = {"requestId": request_id, "driverDigest": require_digest(expected_driver_digest, "expectedDriverDigest"),
            "runtimeOperationId": expected_runtime_operation_id, "keeper": expected_keeper,
            "commandSha256": require_digest(expected_command_sha256, "expectedCommandSha256"),
            "descriptorSha256": require_digest(expected_descriptor_sha256, "expectedDescriptorSha256")}
    value, _, _ = _capture(manager, pins)
    path = manager.paths.state / "deployment/history" / ("unknown-stop-native-attempt-" + value["originalRequest"]["sha256"] + ".json")
    upgrade._require(not path.exists(), "The original unknown Stop already has a native attempt; reconcile it without retry")
    return UnknownStopNativeRecovery(canonical_json(value), _ISSUED)


def execute_unknown_stop_native_recovery(manager, capability):
    upgrade._require(type(capability) is UnknownStopNativeRecovery and capability._token is _ISSUED,
                     "An exact issued native recovery capability is required")
    state = manager.paths.state.absolute()
    with process_lock(state / "deployment-start.lock", reason="deployment_operation_busy"):
        value, raws, native = capability.revalidate(manager)
        path = state / "deployment/history" / ("unknown-stop-native-attempt-" + value["originalRequest"]["sha256"] + ".json")
        upgrade._plain(path)
        # Exclusive durable reservation precedes every drain/API/native mutation.
        raw = canonical_json({"schemaVersion": 1, "kind": "unknown-stop-native-attempt", "preflight": value,
                              "preflightDigest": upgrade._sha(capability.snapshot), "automaticRetry": False}) + b"\n"
        with path.open("xb") as stream:
            stream.write(raw); stream.flush(); os.fsync(stream.fileno())
        attempt = upgrade._ref(path, raw)
        try:
            archives = {key: recovery_keeper._archive_bytes(state, "unknown-stop-original-" + key, data)
                        for key, data in raws.items()}
            capability.revalidate(manager)

            stopped = native._stop_locked()  # Normal real HTTP drain, then original NativeJob stop protocol.
            descriptor = upgrade._document(raws["originalBlockedDescriptor"])
            command = upgrade._document(raws["unknownStopCommand"])
            upgrade._external_native_stop(native, descriptor, stopped.get("stopBinding"))
            validate_unknown_command(state, descriptor, upgrade._reference(state, descriptor["runtimeReceipt"])[0], command)
            upgrade._require(all(upgrade._raw(state / _ANCHORS[key]) == raws[key] for key in
                             ("originalBlockedDescriptor", "unknownStopCommand", "previousSelector"))
                             and protected_holds(state) == descriptor["protectedHolds"]
                             and _unknown_native_rows(state, value["unknownNativeRows"]) == value["unknownNativeRows"],
                             "Original unknown evidence or protected ownership changed during native Stop")
            for key, name in (("stoppedOperation", "deployment-operation.json"), ("stoppedHost", "deployment/host.json"),
                              ("previousSelector", "deployment/driver.json")):
                archives[key] = recovery_keeper._archive_bytes(state, "unknown-stop-terminal-" + key, upgrade._raw(state / name))
            body = {"schemaVersion": 1, "kind": KIND, "variant": VARIANT, "status": "stopped",
                    "requestId": value["pins"]["requestId"], "attempt": attempt, "originalRequest": value["originalRequest"],
                    "originalBlockedDescriptor": archives["originalBlockedDescriptor"],
                    "unknownStopCommand": archives["unknownStopCommand"], "externalNativeStop": stopped["stopBinding"],
                    "archives": archives, "keeper": descriptor["keeper"], "release": descriptor["release"],
                    "originalStopStatus": "unknown", "originalResultCreated": False, "automaticRetry": False,
                    **{name: False for name in upgrade._FLAGS}}
            reference = recovery_keeper._archive(state, "unknown-stop-native-recovery", body)
            validate_completed_recovery(native, descriptor, command, reference, stopped["stopBinding"])
            return {**body, "unknownStopRecovery": reference, "operationId": stopped["operationId"]}
        except BaseException as error:
            recovery_keeper._archive(state, "unknown-stop-native-failure", {"schemaVersion": 1, "kind": KIND,
                "status": "unknown", "attempt": attempt, "originalRequest": value["originalRequest"],
                "reasonCode": getattr(error, "code", type(error).__name__), "message": str(error)[:2000],
                "automaticRetry": False, **{name: False for name in upgrade._FLAGS}})
            raise


def validate_completed_recovery(manager, descriptor, command, reference, native_stop_ref):
    state = manager.paths.state.absolute()
    body, _ = upgrade._reference(state, reference)
    attempt, raw = upgrade._reference(state, body.get("attempt"))
    value = attempt.get("preflight", {})
    original = validate_unknown_command(state, descriptor, upgrade._reference(state, descriptor["runtimeReceipt"])[0], command)
    upgrade._require(body.get("kind") == KIND and body.get("variant") == VARIANT and body.get("status") == "stopped"
                     and body.get("externalNativeStop") == native_stop_ref and body.get("originalRequest") == original
                     and body.get("originalStopStatus") == "unknown" and body.get("originalResultCreated") is False
                     and body.get("automaticRetry") is False
                     and all(body.get(name) is False for name in upgrade._FLAGS)
                     and attempt.get("kind") == "unknown-stop-native-attempt" and attempt.get("automaticRetry") is False
                     and attempt.get("preflightDigest") == digest(value)
                     and body.get("requestId") == value.get("pins", {}).get("requestId")
                     and Path(body["attempt"]["path"]) == state / "deployment/history" / ("unknown-stop-native-attempt-" + original["sha256"] + ".json")
                     and value.get("schemaVersion") == 1 and value.get("kind") == "unknown-stop-native-preflight"
                     and value.get("pins", {}).get("commandSha256") == value.get("anchors", {}).get("unknownStopCommand", {}).get("sha256")
                     and value.get("pins", {}).get("descriptorSha256") == value.get("anchors", {}).get("originalBlockedDescriptor", {}).get("sha256")
                     and value.get("originalRequest") == original and value.get("bootFileTime") == descriptor.get("bootFileTime"),
                     "The separate native outcome lacks its exact one-attempt unknown ancestry")
    for key, document in (("originalBlockedDescriptor", descriptor), ("unknownStopCommand", command)):
        archived, archived_raw = upgrade._reference(state, body.get(key))
        upgrade._require(archived == document and upgrade._sha(archived_raw) == value["anchors"][key]["sha256"],
                         "Original unknown command or blocked descriptor archive changed")
    upgrade._require(_unknown_native_rows(state, value["unknownNativeRows"]) == value["unknownNativeRows"]

                     and all(row in protected_holds(state) for row in descriptor["protectedHolds"]),
                     "Protected historical native records or holds changed")
    upgrade._external_native_stop(manager, descriptor, native_stop_ref)
    return body


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    for name in ("request-id", "expected-driver-digest", "expected-runtime-operation-id", "expected-keeper-birth",
                 "expected-keeper-command-digest", "expected-command-sha256", "expected-descriptor-sha256"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--expected-keeper-pid", type=int, required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check-only", action="store_true")
    mode.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    from .manager import DeploymentManager
    from .paths import resolve_paths
    from .spec import load_spec
    spec = load_spec(args.repo_root.absolute() / ".jenkins/deployment-spec.json")
    manager = DeploymentManager(spec, resolve_paths(spec))
    keeper = {"pid": args.expected_keeper_pid, "creationTime": args.expected_keeper_birth,
              "executable": str(args.repo_root.absolute() / ".jenkins/runtime/python/python.exe"),
              "commandDigest": require_digest(args.expected_keeper_command_digest, "expectedKeeperCommandDigest")}
    capability = prepare_unknown_stop_native_recovery(manager, request_id=args.request_id,
        expected_driver_digest=args.expected_driver_digest, expected_runtime_operation_id=args.expected_runtime_operation_id,
        expected_keeper=keeper, expected_command_sha256=args.expected_command_sha256,
        expected_descriptor_sha256=args.expected_descriptor_sha256)
    result = ({"status": "ready", "checkOnly": True, "preflightDigest": upgrade._sha(capability.snapshot),
               "nativeStopIssued": False, "originalStopStatus": "unknown", "preflight": capability.document}
              if args.check_only else execute_unknown_stop_native_recovery(manager, capability))
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

