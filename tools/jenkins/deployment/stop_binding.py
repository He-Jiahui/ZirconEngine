"""Bind legacy lifecycle metadata to actual stop proofs and preserve transport bytes."""
from __future__ import annotations

import base64
import hashlib
import json
import time
from pathlib import Path

from ..contracts import JenkinsError, digest
from .driver import _plain
from .live_observations import observe_live_identity
from .recovery import has_terminal_deployment_proofs, has_terminal_lifecycle_proofs
from .startup_observations import observe_departed_identity


def _require(condition, message):
    if not condition:
        raise JenkinsError("stop_binding_unproven", message)


def capture_stop_owner(operation: dict, host: dict, home_path: str) -> dict:
    _require(operation.get("state") == "running" and host.get("status") == "running",
             "Only the running lifecycle owner can bind a stop")
    _require(operation.get("homePath") == home_path
             and operation.get("operationId")
             and operation.get("generation") == operation.get("operationId")
             and host.get("operationId") == operation.get("operationId")
             and host.get("generation") == operation.get("generation")
             and type(operation.get("hostPid")) is int
             and operation.get("hostPid") == operation.get("pid") == host.get("hostPid"),
             "Stop ownership does not match Home and generation")
    controller = host.get("controllerIdentity") or host.get("identity")
    _require(isinstance(controller, dict) and controller == host.get("identity", controller),
             "Controller identity is missing or conflicts")
    _require(type(host.get("agentLaunchAttempted")) is bool
             and type(host.get("controlPlaneOnly")) is bool
             and host.get("controlPlaneOnly") == operation.get("controlPlaneOnly"),
             "Stop component launch metadata is incomplete")
    agent = host.get("agentIdentity")
    _require((host["agentLaunchAttempted"] and isinstance(agent, dict))
             or (host["agentLaunchAttempted"] is False and agent is None),
             "Agent ownership is incomplete")
    observations = [observe_live_identity(value) for value in
                    (operation, controller, *([agent] if agent else []))]
    return {"operation": json.loads(json.dumps(operation)),
            "host": json.loads(json.dumps(host)), "liveIdentities": observations,
            "homePath": home_path}


def _archive_bytes(path: Path, raw: bytes) -> None:
    _plain(path)
    if path.exists():
        _require(path.read_bytes() == raw, "Immutable stop evidence conflicts")
    else:
        with path.open("xb") as stream:
            stream.write(raw)


def bind_owned_stop(state: Path, captured: dict, command_path: Path, result_path: Path) -> dict:
    operation, previous = captured["operation"], captured["host"]
    for path in (state, command_path, result_path):
        _plain(path)
    raw = result_path.read_bytes()
    history = state / "deployment/history"
    _plain(history)
    history.mkdir(parents=True, exist_ok=True)
    raw_archive = history / ("stop-result-" + hashlib.sha256(raw).hexdigest() + ".json")
    _archive_bytes(raw_archive, raw)
    result = json.loads(raw)
    command_raw = command_path.read_bytes()
    command = json.loads(command_raw)
    controller = previous.get("controllerIdentity") or previous["identity"]
    _require(command.get("action") == "stop"
             and command.get("operationId") == operation["operationId"]
             and command.get("generation") == operation["generation"]
             and command.get("hostPid") == operation["hostPid"]
             and command.get("controllerIdentity") == controller,
             "Stop command does not bind to the captured owner")
    _require(result.get("status") == "stopped"
             and result.get("operationId") == operation["operationId"]
             and result.get("generation") == operation["generation"],
             "Stop response belongs to another lifecycle generation")
    _require("identity" not in result or result["identity"] == controller,
             "Stop controller identity conflicts with the captured owner")
    host_path = state / "deployment/host.json"
    _plain(host_path)
    terminal_raw = host_path.read_bytes()
    _require(json.loads(terminal_raw) == result,
             "Terminal host differs from the actual stop response")
    metadata = {"hostPid": operation["hostPid"], "controllerIdentity": controller,
                "agentIdentity": previous.get("agentIdentity"),
                "agentLaunchAttempted": previous["agentLaunchAttempted"],
                "controlPlaneOnly": previous["controlPlaneOnly"]}
    for field, expected in metadata.items():
        _require(field not in result or result[field] == expected,
                 "Stop metadata conflicts with the captured owner: " + field)
    projected = {**result, **metadata}
    _require(has_terminal_deployment_proofs(operation, projected, captured["homePath"]),
             "Actual native termination proof is incomplete or mismatched")
    deadline = time.monotonic() + 15
    while True:
        try:
            observation = observe_departed_identity(operation)
            break
        except JenkinsError as error:
            if error.code != "deployment_owner_live" or time.monotonic() >= deadline:
                raise
            time.sleep(.2)
    _require(has_terminal_lifecycle_proofs(operation, projected, captured["homePath"], observation),
             "Exact supervisor departure is unproven")
    record = {"schemaVersion": 1, "kind": "owned-stop-metadata-binding",
              "capturedOwner": captured, "command": command,
              "commandSha256": hashlib.sha256(command_raw).hexdigest(),
              "rawStopResult": {"path": str(result_path), "archivePath": str(raw_archive),
                                "sha256": hashlib.sha256(raw).hexdigest(),
                                "base64": base64.b64encode(raw).decode("ascii")},
              "terminalHostRawSha256": hashlib.sha256(terminal_raw).hexdigest(),
              "terminalHost": projected, "hostDeparture": observation,
              "nativeProofSynthesized": False, "historicalHoldsReleased": False,
              "runtimeAcceptance": False}
    receipt_path = history / ("owned-stop-binding-" + digest(record) + ".json")
    encoded = (json.dumps(record, sort_keys=True, indent=2) + "\n").encode("utf-8")
    _archive_bytes(receipt_path, encoded)
    _require(host_path.read_bytes() == terminal_raw, "Host changed during stop binding")
    _require(json.loads((state / "deployment-operation.json").read_bytes()) == operation,
             "Lifecycle journal changed during stop binding")
    pending = host_path.with_suffix(".stop-binding.pending")
    _plain(pending)
    pending.write_text(json.dumps(projected, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    pending.replace(host_path)
    return {**operation, "state": "stopped", "serviceReady": False, "health": False,
            "nativeTerminationProof": result["nativeTerminationProof"],
            "agentTerminationProof": result.get("agentTerminationProof"),
            "stopBinding": {"path": str(receipt_path), "sha256": hashlib.sha256(encoded).hexdigest()},
            "stopResult": {"path": str(raw_archive), "sha256": hashlib.sha256(raw).hexdigest()}}
