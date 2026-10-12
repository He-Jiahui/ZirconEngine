"""Pinned repair of a broker exit write whose native proofs were already saved.

This entry does not adopt a process, infer death from a missing PID, release a
reservation, or publish an artifact. It preserves the original rows in an audit
record before changing only a proven completed launch from started to exited.
"""
from __future__ import annotations

from dataclasses import asdict, dataclass
import hashlib
import time
from typing import Mapping

from ..contracts import JenkinsError, digest, require_digest
from ..processes.guards import guard_phases_complete
from ..state import State
from .quarantine import matches_binding


@dataclass(frozen=True)
class RecordPin:
    version: int
    payload_digest: str

    def __post_init__(self) -> None:
        if type(self.version) is not int or self.version < 1:
            raise ValueError("record pin requires a positive version")
        require_digest(self.payload_digest, "payloadDigest")

    @classmethod
    def from_row(cls, row: dict) -> "RecordPin":
        return cls(row["version"], digest(row["payload"]))


@dataclass(frozen=True)
class CompletedLaunchBinding:
    execution_id: str
    operation_id: str
    generation: str
    driver_digest: str

    def __post_init__(self) -> None:
        if not all(isinstance(value, str) and value for value in
                   (self.execution_id, self.operation_id, self.generation)):
            raise ValueError("completed launch binding requires exact non-empty string identities")
        require_digest(self.driver_digest, "driverDigest")


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise JenkinsError("execution_launch_reconciliation_unproven", message)


def _pinned(state: State, domain: str, key: str, pin: RecordPin, connection) -> dict:
    row = state.get(domain, key, connection=connection)
    if row is None or row["version"] != pin.version or digest(row["payload"]) != pin.payload_digest:
        raise JenkinsError("state_conflict", "Pinned reconciliation input changed: " + domain, retryable=True)
    return row


def _complete_proof(row: dict, execution_id: str, *, role: str) -> dict:
    job = row["payload"]
    proof = job.get("completeProof")
    identity = proof.get("identity") if isinstance(proof, dict) else None
    _require(job.get("nativeJobId") == row["key"] and job.get("executionId") == execution_id
             and job.get("owner") == execution_id and job.get("role") == role
             and job.get("status") == "terminal", "Native Job ownership does not match the execution")
    _require(isinstance(proof, dict) and proof.get("complete") is True and job.get("terminalProof") == proof
             and isinstance(identity, dict), "Native Job lacks its original complete termination proof")
    _require(type(job.get("pid")) is int and job["pid"] > 0 and type(identity.get("pid")) is int
             and identity["pid"] == job["pid"] and isinstance(job.get("birthToken"), str)
             and job["birthToken"].isdecimal() and int(job["birthToken"]) > 0
             and identity.get("creationTime") == job["birthToken"]
             and isinstance(job.get("executable"), str) and bool(job["executable"])
             and identity.get("executable") == job["executable"], "Native termination proof has a different process identity")
    _require(type(proof.get("processExitCode")) is int and type(proof.get("activeProcesses")) is int
             and proof["activeProcesses"] == 0
             and all(proof.get(field) is True for field in ("childrenGone", "stdoutEof", "stderrEof")),
             "Native termination proof does not prove process exit, zero children, and both log EOFs")
    return proof


def _archive(state: State, row: dict, connection) -> dict:
    raw = connection.execute("SELECT payload FROM records WHERE domain=? AND key=?",
                             (row["domain"], row["key"])).fetchone()[0]
    return {"row": row, "version": row["version"], "payloadDigest": digest(row["payload"]),
            "rawPayload": raw, "rawSha256": hashlib.sha256(raw.encode("utf-8")).hexdigest()}


def reconcile_completed_launch(state: State, *, binding: CompletedLaunchBinding,
                               expected_launch: RecordPin, expected_execution: RecordPin,
                               expected_host: RecordPin,
                               expected_native_jobs: Mapping[str, RecordPin]) -> dict:
    """Repair one explicitly pinned launch using saved native evidence only.

    The caller supplies the reviewed runtime/driver/execution tuple and pins
    every launch, execution, host, guard and phase input. All reads, proof
    checks, audit persistence and the launch CAS share one transaction.
    """
    key = binding.execution_id
    with state.transaction() as connection:
        launch = _pinned(state, "execution_launch", key, expected_launch, connection)
        execution = _pinned(state, "execution", key, expected_execution, connection)
        host = _pinned(state, "execution_host", key, expected_host, connection)
        lp, ep, hp = launch["payload"], execution["payload"], host["payload"]
        _require(lp.get("status") == "started" and lp.get("executionId") == key and ep.get("executionId", key) == key
                 and matches_binding(lp, ep, binding.driver_digest, binding.operation_id)
                 and lp.get("generation") == binding.generation,
                 "Launch and execution do not match the pinned completed runtime")
        _require(hp.get("status") in {"terminal", "cancelled"} and hp.get("executionId") == key
                 and hp.get("operationId") == binding.operation_id and hp.get("generation") == binding.generation,
                 "Host is not terminal for this exact execution and generation")
        launcher = lp.get("launcherIdentity")
        native_identity = {"hostPid": hp.get("hostPid"), "hostBirthToken": hp.get("hostBirthToken")}
        _require(isinstance(launcher, dict) and type(native_identity["hostPid"]) is int
                 and native_identity["hostPid"] == launcher.get("pid")
                 and native_identity["hostBirthToken"] == launcher.get("creationTime")
                 and lp.get("nativeHostIdentity") == native_identity,
                 "Host and launch have different PID/birth identities")
        guard_id = lp.get("guardNativeJobId")
        _require(isinstance(guard_id, str) and bool(guard_id) and guard_id in expected_native_jobs,
                 "The exact enclosing guard must be pinned")
        guard = _pinned(state, "native_job", guard_id, expected_native_jobs[guard_id], connection)
        guard_proof = _complete_proof(guard, key, role="execution_host_guard")
        _require(guard_proof["identity"] == launcher, "Guard proof does not match the recorded launcher")
        _require(guard_phases_complete(state, guard, connection=connection), "A nested phase lacks its own complete proof")
        phase_records = hp.get("phaseRecords")
        _require(isinstance(phase_records, list) and bool(phase_records)
                 and all(isinstance(item, dict) and isinstance(item.get("nativeJobId"), str) for item in phase_records),
                 "Terminal host lacks exact phase records")
        phase_ids = [item["nativeJobId"] for item in phase_records]
        _require(len(set(phase_ids)) == len(phase_ids) and guard_id not in phase_ids,
                 "Host phase identities are repeated or identify the enclosing guard")
        expected_ids = {guard_id, *phase_ids}
        related_ids = {row["key"] for row in state.list("native_job", connection=connection)
                       if row["payload"].get("executionId") == key or row["payload"].get("owner") == key
                       or row["payload"].get("guardNativeJobId") == guard_id}
        _require(set(expected_native_jobs) == expected_ids and related_ids == expected_ids,
                 "Native pins and terminal host do not cover exactly all execution Jobs")
        phases = []
        for record in phase_records:
            phase = _pinned(state, "native_job", record["nativeJobId"], expected_native_jobs[record["nativeJobId"]], connection)
            proof = _complete_proof(phase, key, role="phase")
            _require(phase["payload"].get("guardNativeJobId") == guard_id and record.get("terminalProof") == proof,
                     "Host phase proof or enclosing guard differs from its native record")
            if hp["status"] == "terminal":
                _require(proof["processExitCode"] == 0, "A successful terminal host contains a failed phase")
            phases.append(phase)
        final_phase = phases[-1]
        _require(hp.get("nativeJobId") == final_phase["key"] and lp.get("nativeHostUUID") == final_phase["key"]
                 and hp.get("terminalProof") == final_phase["payload"]["completeProof"]
                 and hp.get("pid") == final_phase["payload"]["pid"]
                 and hp.get("processBirthToken") == final_phase["payload"]["birthToken"],
                 "Launch and host do not identify the same final native phase")
        _require(state.get("execution_abort", key, connection=connection) is None,
                 "An existing abort record requires its original failure reconciliation")
        audit_key = key + ":" + str(expected_launch.version)
        archives = [_archive(state, row, connection) for row in (launch, execution, host, guard, *phases)]
        audit = {"schemaVersion": 1, "cause": "completed_native_proofs_with_missing_launch_exit",
                 "binding": asdict(binding), "archives": archives, "archivesDigest": digest(archives),
                 "artifactPublicationAllowed": False, "observedAt": time.time()}
        state.put("execution_launch_reconciliation", audit_key, audit, expected_version=0, connection=connection)
        result = state.put("execution_launch", key, {**lp, "status": "exited",
                           "exitCode": guard_proof["processExitCode"], "exitedAt": time.time(),
                           "reconciliationAuditKey": audit_key}, expected_version=expected_launch.version, connection=connection)
        event_id = state.event("execution_launch_reconciled", {"auditKey": audit_key, "cause": audit["cause"],
            "binding": audit["binding"], "beforeVersion": launch["version"], "beforeDigest": expected_launch.payload_digest,
            "afterVersion": result["version"], "afterDigest": digest(result["payload"]),
            "archivesDigest": audit["archivesDigest"], "artifactPublicationAllowed": False}, connection=connection)
    return {"launch": result, "auditKey": audit_key, "eventId": event_id, "artifactPublicationAllowed": False}
