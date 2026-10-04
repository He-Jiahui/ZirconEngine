"""Verify an independent Jenkins command without granting milestone acceptance."""
from __future__ import annotations

import math
import hashlib
from typing import Mapping, Sequence

from .contracts import PilotError, RequestIdentity, digest, identifier, sha256_identity
from .snapshot import portable_path


def verify_receipt(
    receipt: Mapping[str, object], *, identity: RequestIdentity, job: str,
    build_number: int, bundle_hash: str, command: Sequence[str], artifacts: Mapping[str, bytes],
    driver_input_hash: str,
) -> dict[str, object]:
    required = {
        "schemaVersion", "acceptance", "identity", "job", "buildNumber", "bundleHash",
        "command", "commandHash", "sourceBefore", "sourceAfter", "exitCode", "outcome",
        "toolchain", "processTree", "timings", "artifacts", "nativeJobEvidence", "terminalErrors",
    }
    if not isinstance(receipt, Mapping) or not required.issubset(receipt):
        raise PilotError("pilot receipt is missing required evidence")
    if type(receipt["schemaVersion"]) is not int or receipt["schemaVersion"] != 2 or receipt["acceptance"] != "jenkins-command-evidence-v2":
        raise PilotError("receipt is not current independent Jenkins command evidence")
    actual_identity = RequestIdentity.from_dict(receipt["identity"])
    if actual_identity != identity or receipt["job"] != identifier(job, "job"):
        raise PilotError("receipt belongs to another request or Jenkins job")
    if type(build_number) is not int or build_number < 1 or type(receipt["buildNumber"]) is not int or receipt["buildNumber"] != build_number:
        raise PilotError("receipt belongs to another Jenkins build")
    if receipt["bundleHash"] != sha256_identity(bundle_hash, "bundleHash"):
        raise PilotError("receipt belongs to another sealed archive")
    expected_command = list(command)
    if not expected_command or any(not isinstance(value, str) or not value for value in expected_command):
        raise PilotError("receipt verification requires the expected registered command")
    if receipt["command"] != expected_command or receipt["commandHash"] != digest(expected_command):
        raise PilotError("receipt command does not match the declared validation template")
    if receipt["sourceBefore"] != identity.input_hash or receipt["sourceAfter"] != identity.input_hash:
        raise PilotError("receipt has no unchanged, matching source proof")
    if receipt["outcome"] != "passed" or type(receipt["exitCode"]) is not int or receipt["exitCode"] != 0:
        raise PilotError("pilot validation did not pass")
    tree = receipt["processTree"]
    if not isinstance(tree, Mapping) or tree.get("terminal") is not True or tree.get("scope") != "windows_job":
        raise PilotError("receipt does not prove the owned process tree is terminal")
    if type(tree.get("pid")) is not int or tree["pid"] < 1 or not isinstance(tree.get("creationTime"), str) or not tree["creationTime"].isdecimal():
        raise PilotError("process proof has no stable owned process identity")
    pipes = tree.get("pipes")
    if tree.get("readersFinished") is not True or tree.get("pipeEOF") is not True or not isinstance(pipes, Mapping) or set(pipes) != {"stdout", "stderr"} or any(not isinstance(p, Mapping) or p.get("eof") is not True or "error" in p for p in pipes.values()):
        raise PilotError("receipt does not prove complete native output capture")
    native = receipt["nativeJobEvidence"]
    if not isinstance(native, Mapping) or native.get("access") != "JOB_OBJECT_QUERY" or native.get("errors") != [] or receipt["terminalErrors"] != []:
        raise PilotError("native terminal evidence has errors or is missing")
    final = native.get("records", {}).get("afterClose", {})
    if type(final.get("activeProcesses")) is not int or final["activeProcesses"] != 0:
        raise PilotError("native Job has no observed zero-process accounting")
    toolchain = receipt["toolchain"]
    if not isinstance(toolchain, Mapping) or not toolchain or any(not isinstance(value, str) or not value for value in toolchain.values()):
        raise PilotError("receipt has no actual toolchain evidence")
    if toolchain.get("driverInputHash") != sha256_identity(driver_input_hash, "driverInputHash"):
        raise PilotError("receipt belongs to a different trusted driver")
    timings = receipt["timings"]
    if not isinstance(timings, Mapping) or not {"prepareSeconds", "runSeconds"}.issubset(timings):
        raise PilotError("receipt is missing execution timings")
    for value in timings.values():
        if type(value) not in {int, float} or not math.isfinite(value) or value < 0:
            raise PilotError("receipt execution timings are invalid")
    declared_artifacts = receipt["artifacts"]
    if not isinstance(declared_artifacts, list) or not declared_artifacts:
        raise PilotError("receipt has no validation artifacts")
    names = set()
    for artifact in declared_artifacts:
        if not isinstance(artifact, Mapping) or not {"path", "sha256", "size"}.issubset(artifact):
            raise PilotError("receipt artifact fields are invalid")
        name = portable_path(artifact["path"])
        if name.casefold() in names:
            raise PilotError("receipt contains duplicate artifact paths")
        names.add(name.casefold())
        sha256_identity(artifact["sha256"], "artifact sha256")
        if type(artifact["size"]) is not int or artifact["size"] < 0:
            raise PilotError("receipt artifact size is invalid")
        actual = artifacts.get(name)
        if not isinstance(actual, bytes) or len(actual) != artifact["size"] or hashlib.sha256(actual).hexdigest() != artifact["sha256"]:
            raise PilotError("receipt artifact is missing or differs from the archived content")
    if identity.template == "managed-cargo-check-v2":
        proof = receipt.get("managedCargo")
        if not isinstance(proof, Mapping) or proof.get("accepted") is not True or not proof.get("sourceDigest") or not proof.get("compatibilityKey"):
            raise PilotError("Cargo receipt has no independently accepted execution evidence")
        result = proof.get("receipt", {})
        if result.get("identity") != identity.to_dict() or result.get("lockReleased") is not True or result.get("terminalVerified") is not True or result.get("inputsUnchanged") is not True:
            raise PilotError("Cargo terminal receipt lacks its exact resource release proof")
        observed = {key: receipt[key] for key in ("command", "commandHash", "exitCode", "processTree", "nativeJobEvidence", "terminalErrors")}
        if result.get("observation") != observed:
            raise PilotError("Cargo and runner native terminal observations differ")
        expected_artifacts = {"managed-request.json", "managed-result.json", "managed-evidence.json", "output.log"}
        if not expected_artifacts.issubset(names):
            raise PilotError("Cargo independent metrics artifacts are incomplete")
    return {"acceptance": "jenkins-validation", "accepted": True, "scope": "sealed-input",
            "milestoneAcceptance": False, "migrationAcceptance": False,
            "identity": identity.to_dict(), "buildNumber": build_number}
