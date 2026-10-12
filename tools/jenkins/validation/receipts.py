"""Managed producer evidence and authoritative acceptance receipts.

Jenkins' UI result is not an acceptance result.  Heavy recipes require
terminal process evidence and every recipe requires a content-addressed output
closure.  Arbitrary ``success.json`` files are deliberately rejected.
"""
from __future__ import annotations

import os
import time
import fnmatch
from copy import deepcopy
from contextlib import nullcontext
from pathlib import Path
from typing import Any, Iterable
from uuid import uuid4

from ..contracts import JenkinsError, digest, file_digest, require_digest

SCHEMA_VERSION = 1
PRODUCER_KIND = "jenkins.managed.producer-receipt"
ACCEPTANCE_KIND = "jenkins.authoritative.acceptance-receipt"
LIGHT_STAGES = frozenset({"source", "patch", "comments", "syntax", "format", "acceptance"})
HEAVY_STAGES = frozenset({"build", "unit_test", "integration_test", "regression_test",
                          "package_check", "editor_exe", "runtime_dll", "package"})
APPROVED_PRODUCERS = frozenset({"zircon-jenkins", "jenkins-managed-runtime"})


def _failure(code: str, message: str, **details: Any) -> JenkinsError:
    return JenkinsError(code, message, details=details)


def closure_manifest(root: str | os.PathLike[str], paths: Iterable[str | os.PathLike[str]] | None = None) -> dict[str, Any]:
    """Build deterministic file evidence rooted at ``root``."""
    base = Path(root).absolute()
    if not base.is_dir():
        raise _failure("output_root_missing", "Output closure root is not a directory", root=str(base))
    _physical_closure_path(base)
    candidates = list(paths) if paths is not None else [p for p in base.rglob("*") if p.is_file()]
    entries: list[dict[str, Any]] = []
    for raw in candidates:
        path = Path(raw)
        path = (base / path) if not path.is_absolute() else path.absolute()
        _physical_closure_path(path)
        if path.is_symlink():
            raise _failure("output_alias", "Output closure cannot contain a symlink", path=str(path))
        try:
            relative = path.relative_to(base)
        except ValueError as exc:
            raise _failure("output_path_escape", "Output path escaped closure root", path=str(path)) from exc
        if not path.is_file():
            raise _failure("output_not_regular", "Output closure contains a non-regular file", path=str(path))
        entries.append({"path": relative.as_posix(), "sha256": file_digest(path), "size": path.stat().st_size})
    entries.sort(key=lambda item: item["path"])
    return {"root": str(base), "files": entries, "digest": digest(entries)}


def _physical_closure_path(path: Path) -> None:
    if ".." in path.parts:
        raise _failure("output_path_escape", "Output paths cannot contain parent traversal")
    for component in (path, *path.parents):
        if component.exists() or component.is_symlink():
            if component.is_symlink() or getattr(component.lstat(), "st_file_attributes", 0) & 0x400:
                raise _failure("output_alias", "Output paths cannot follow aliases or reparse points")


def _required_digest(receipt: dict[str, Any], name: str) -> None:
    try:
        require_digest(receipt.get(name), name)
    except JenkinsError as exc:
        raise _failure("receipt_digest_invalid", str(exc), field=name) from exc


def _task_scratch_evidence(value: dict[str, Any]) -> dict[str, Any]:
    """Keep legacy evidence absent, but never accept a partial new binding."""
    fields = ("taskScratch", "effectiveEnvironmentDigest")
    present = [name in value for name in fields]
    if not any(present):
        return {}
    scratch = value.get("taskScratch")
    if (not all(present) or not isinstance(scratch, dict)
            or set(scratch) != {"scratchGeneration", "effectiveEnvironment"}):
        raise _failure("receipt_scratch_invalid", "Receipt requires its complete task scratch binding")
    environment = scratch.get("effectiveEnvironment")
    if (not isinstance(environment, dict) or set(environment) != {"TEMP", "TMP", "TMPDIR"}
            or any(not isinstance(path, str) or not path for path in environment.values())):
        raise _failure("receipt_scratch_invalid", "Task scratch requires all three effective TEMP overrides")
    _required_digest(scratch, "scratchGeneration")
    _required_digest(value, "effectiveEnvironmentDigest")
    return {name: deepcopy(value[name]) for name in fields}


def _same_task_scratch(value: dict[str, Any], expected: dict[str, Any]) -> dict[str, Any]:
    evidence = _task_scratch_evidence(value)
    if evidence != _task_scratch_evidence(expected):
        raise _failure("receipt_scratch_mismatch", "Task scratch differs from its original registered evidence")
    return evidence


def _verify_closure(receipt: dict[str, Any], output_root: str | os.PathLike[str] | None,
                    required_outputs: Iterable[str] | None = None) -> None:
    closure = receipt.get("outputClosure")
    if not isinstance(closure, dict) or not isinstance(closure.get("files"), list):
        raise _failure("output_closure_missing", "Managed producer receipt lacks output closure evidence")
    if closure.get("digest") != digest(closure["files"]):
        raise _failure("output_closure_digest_mismatch", "Output closure digest is not over its file list")
    if required_outputs is not None:
        paths = {entry.get("path") for entry in closure["files"]}
        missing = sorted(set(required_outputs) - paths)
        if missing:
            raise _failure("required_output_missing", "Required output is absent from closure", paths=missing)
    for pattern in receipt.get("requiredOutputGlobs", []):
        if (not isinstance(pattern, str) or not pattern or Path(pattern).is_absolute()
                or ".." in Path(pattern).parts):
            raise _failure("required_output_pattern_invalid", "Required output patterns must be relative")
        if not any(fnmatch.fnmatchcase(str(item.get("path", "")), pattern) for item in closure["files"]):
            raise _failure("required_output_missing", "No output matches the immutable required pattern", pattern=pattern)
    if output_root is not None:
        paths = [entry.get("path") for entry in closure["files"]]
        actual = closure_manifest(output_root, paths)
        if actual["digest"] != closure["digest"] or actual["files"] != closure["files"]:
            raise _failure("output_closure_changed", "Output closure no longer matches producer evidence")


def validate_producer_receipt(receipt: dict[str, Any], *, expected: dict[str, Any] | None = None,
                              output_root: str | os.PathLike[str] | None = None,
                              required_outputs: Iterable[str] | None = None) -> dict[str, Any]:
    """Validate managed producer evidence and return a copy of the receipt."""
    if not isinstance(receipt, dict) or receipt.get("schemaVersion") != SCHEMA_VERSION or receipt.get("kind") != PRODUCER_KIND:
        raise _failure("fabricated_receipt", "Receipt is not a managed Jenkins producer receipt")
    if receipt.get("status") != "passed":
        raise _failure("producer_not_passed", "Only a passed producer receipt can be accepted")
    stage = receipt.get("stage")
    if not isinstance(stage, str) or stage not in LIGHT_STAGES | HEAVY_STAGES:
        raise _failure("receipt_stage_invalid", "Receipt stage is not approved", stage=stage)
    mode = "heavy" if stage in HEAVY_STAGES or receipt.get("requiresCompilerProof") is True else "light"
    for field in ("executionId", "sourceDigest", "recipeDigest", "coverageDigest", "driverDigest"):
        if not isinstance(receipt.get(field), str) or not receipt[field]:
            raise _failure("receipt_field_invalid", f"Receipt field {field} is required", field=field)
    for field in ("sourceDigest", "recipeDigest", "coverageDigest", "driverDigest"):
        _required_digest(receipt, field)
    producer = receipt.get("producer")
    if (not isinstance(producer, dict) or producer.get("managed") is not True
            or producer.get("name") not in APPROVED_PRODUCERS):
        raise _failure("untrusted_producer", "Receipt producer is not an approved managed producer")
    _task_scratch_evidence(receipt)
    if expected and any(name in expected for name in ("taskScratch", "effectiveEnvironmentDigest")):
        _same_task_scratch(receipt, expected)
    if mode == "heavy":
        terminal = receipt.get("terminalProof")
        if (not isinstance(terminal, dict) or terminal.get("state") != "exited"
                or terminal.get("exitCode") != 0 or terminal.get("eofObserved") is not True
                or terminal.get("nativeJobActive") is not False):
            raise _failure("terminal_proof_missing", "Heavy validation requires successful terminal process proof")
        if (not isinstance(terminal.get("pid"), int) or terminal["pid"] <= 0
                or not terminal.get("birthToken") or not terminal.get("nativeJobId")):
            raise _failure("terminal_proof_invalid", "Terminal proof lacks process identity")
    _verify_closure(receipt, output_root, required_outputs)
    for field in ("executionId", "sourceDigest", "recipeDigest", "coverageDigest", "driverDigest", "stage"):
        if expected and field in expected and receipt.get(field) != expected[field]:
            raise _failure("receipt_identity_mismatch", f"Receipt {field} does not match expected identity", field=field)
    return dict(receipt)


class TrustedReceiptAuthority:
    """State-backed receipt authority used by Jenkins pipeline owners.

    Callers never upload a producer receipt.  They register an immutable
    execution manifest before launch, then the managed collector records the
    terminal process proof and output closure against that manifest.
    """
    def __init__(self, state: Any, *, repo_root: str | os.PathLike[str] | None = None) -> None:
        self.state = state
        self.repo_root = Path(repo_root).resolve() if repo_root else None

    def register_attempt(self, manifest: dict[str, Any]) -> dict[str, Any]:
        required = ("executionId", "stage", "sourceDigest", "recipeDigest", "coverageDigest", "driverDigest")
        if any(not manifest.get(name) for name in required):
            raise _failure("manifest_invalid", "Immutable execution manifest is incomplete")
        for name in required[2:]:
            _required_digest(manifest, name)
        _task_scratch_evidence(manifest)
        execution_id = manifest["executionId"]
        if manifest.get("stage") in HEAVY_STAGES and (not manifest.get("nativeJobId") or not manifest.get("birthToken")):
            raise _failure("manifest_runner_missing", "Manifest must bind the managed native runner")
        existing = self.state.get("validation_attempt", execution_id)
        payload = {**manifest, "status": "registered", "producerName": "zircon-jenkins"}
        if existing:
            old_payload = dict(existing["payload"])
            old_payload.pop("status", None)
            old_payload.pop("receipt", None)
            requested = dict(payload)
            requested.pop("status", None)
            if old_payload != requested:
                raise _failure("manifest_immutable", "Execution manifest cannot be replaced")
            return existing
        return self.state.put("validation_attempt", execution_id, payload)

    def record_terminal(self, execution_id: str, proof: dict[str, Any], closure: dict[str, Any]) -> dict[str, Any]:
        record = self.state.get("validation_attempt", execution_id)
        if not record:
            raise _failure("attempt_unknown", "Execution was not registered before launch")
        native_job_id = record["payload"].get("nativeJobId")
        job = self.state.get("native_job", native_job_id) if native_job_id else None
        if not job or job["payload"].get("status") != "terminal":
            raise _failure("native_job_not_terminal", "State has no trusted terminal native job record")
        if proof.get("nativeJobId") != native_job_id or proof.get("nativeJobId") != job["payload"].get("nativeJobId"):
            raise _failure("terminal_identity_mismatch", "Terminal proof does not match registered native job")
        if proof.get("birthToken") != job["payload"].get("birthToken") or proof.get("birthToken") != record["payload"].get("birthToken"):
            raise _failure("terminal_identity_mismatch", "Terminal proof does not match registered process birth")
        trusted = job["payload"].get("completeProof")
        if not isinstance(trusted, dict) or trusted.get("complete") is not True:
            raise _failure("terminal_proof_untrusted", "Native job lacks complete collector proof")
        for key in ("processExitCode", "stdoutEof", "stderrEof", "childrenGone"):
            if proof.get(key) != trusted.get(key):
                raise _failure("terminal_proof_mismatch", "Caller proof differs from trusted collector proof", field=key)
        proof = dict(trusted, nativeJobId=native_job_id, birthToken=job["payload"].get("birthToken"), pid=job["payload"].get("pid"), state="exited", exitCode=trusted.get("processExitCode"), eofObserved=trusted.get("stdoutEof") and trusted.get("stderrEof"), nativeJobActive=False)
        receipt = {
            "schemaVersion": SCHEMA_VERSION, "kind": PRODUCER_KIND, "status": "passed",
            "executionId": execution_id, "stage": record["payload"]["stage"],
            "sourceDigest": record["payload"]["sourceDigest"], "recipeDigest": record["payload"]["recipeDigest"],
            "coverageDigest": record["payload"]["coverageDigest"], "driverDigest": record["payload"]["driverDigest"],
            "producer": {"managed": True, "name": "zircon-jenkins"}, "terminalProof": proof,
            "outputClosure": closure,
            "requiredOutputGlobs": list(record["payload"].get("requiredOutputGlobs", [])),
        }
        receipt.update(_task_scratch_evidence(record["payload"]))
        root = closure.get("root") if isinstance(closure, dict) else None
        expected_root = record["payload"].get("outputRoot") or record["payload"].get("buildRoot")
        if expected_root and Path(root).absolute() != Path(expected_root).absolute():
            raise _failure("output_root_mismatch", "Collector output closure root differs from registered root")
        required = record["payload"].get("requiredOutputs", [])
        validate_producer_receipt(receipt, output_root=root, required_outputs=required)
        with self.state.transaction() as connection:
            self._verify_task_scratch(record["payload"], receipt, connection=connection, require_phase=False)
            self.state.put("validation_receipt", execution_id, receipt, connection=connection)
            self.state.put("validation_attempt", execution_id, {**record["payload"], "status": "passed", "receipt": receipt}, expected_version=record["version"], connection=connection)
        return receipt

    def record_light_terminal(self, execution_id: str, evidence: dict[str, Any], closure: dict[str, Any]) -> dict[str, Any]:
        """Record a trusted patch/syntax/format/classification result.

        Light stages do not require compiler or native-job evidence, but their
        operation journal must be registered by the light owner before this
        collector call.  The caller cannot manufacture a receipt by passing a
        JSON object directly.
        """
        record = self.state.get("validation_attempt", execution_id)
        if not record or record["payload"].get("status") != "registered":
            raise _failure("attempt_unknown", "Light execution was not registered")
        stage = record["payload"].get("stage")
        if stage not in LIGHT_STAGES:
            raise _failure("light_stage_invalid", "Execution is not a light validation stage")
        journal = self.state.get("light_attempt", execution_id) or self.state.get("operation_journal", execution_id)
        if not journal or journal["payload"].get("status") not in {"registered", "passed", "terminal"}:
            raise _failure("light_evidence_untrusted", "No trusted light operation journal exists")
        payload = journal["payload"]
        for key in ("sourceDigest", "driverDigest"):
            if payload.get(key) != record["payload"].get(key):
                raise _failure("light_identity_mismatch", "Light evidence identity differs from execution manifest", field=key)
        if evidence.get("status") != "passed" or evidence.get("exitCode", 0) != 0:
            raise _failure("light_not_passed", "Light operation did not pass")
        if evidence.get("tool") is None or evidence.get("commandDigest") is None:
            raise _failure("light_evidence_incomplete", "Light evidence requires fixed tool and command digest")
        receipt = {
            "schemaVersion": SCHEMA_VERSION, "kind": PRODUCER_KIND, "status": "passed",
            "executionId": execution_id, "stage": stage,
            "sourceDigest": record["payload"]["sourceDigest"], "recipeDigest": record["payload"]["recipeDigest"],
            "coverageDigest": record["payload"]["coverageDigest"], "driverDigest": record["payload"]["driverDigest"],
            "producer": {"managed": True, "name": "zircon-jenkins"}, "lightEvidence": dict(evidence),
            "outputClosure": closure,
        }
        if _task_scratch_evidence(record["payload"]):
            raise _failure("receipt_scratch_unproven", "Task scratch requires evidence from its native phase collector")
        validate_producer_receipt(receipt)
        with self.state.transaction() as connection:
            self.state.put("validation_receipt", execution_id, receipt, connection=connection)
            self.state.put("validation_attempt", execution_id, {**record["payload"], "status": "passed", "receipt": receipt}, expected_version=record["version"], connection=connection)
        return receipt

    def get(self, execution_id: str, *, connection=None) -> dict[str, Any] | None:
        record = self.state.get("validation_attempt", execution_id, connection=connection)
        if not record or record["payload"].get("status") != "passed":
            return None
        receipt = record["payload"].get("receipt")
        if isinstance(receipt, dict):
            self._verify_task_scratch(record["payload"], receipt, connection=connection)
        return receipt

    def _verify_task_scratch(self, manifest, receipt, *, connection=None, require_phase=True):
        evidence = _same_task_scratch(receipt, manifest)
        native = self.state.get("native_job", manifest.get("nativeJobId"), connection=connection)
        job = native["payload"] if native else {}
        execution_id = job.get("executionId")
        host_row = (self.state.get("execution_host", execution_id, connection=connection)
                    if isinstance(execution_id, str) and execution_id else None)
        host = host_row["payload"] if host_row else {}
        records = host.get("phaseRecords", [])
        matches = ([phase for phase in records if isinstance(phase, dict)
                    and phase.get("nativeJobId") == manifest.get("nativeJobId")]
                   if isinstance(records, list) else [])
        if not evidence:
            if ("taskScratch" in host or any(_task_scratch_evidence(phase) for phase in matches)):
                raise _failure("receipt_scratch_mismatch", "A native scratch binding cannot be omitted from its receipt")
            return
        scratch = evidence["taskScratch"]
        execution_row = self.state.get("execution", execution_id, connection=connection) if host_row else None
        execution = execution_row["payload"] if execution_row else {}
        scratch_row = self.state.get("execution_scratch", execution_id, connection=connection) if execution_row else None
        registered = scratch_row["payload"] if scratch_row else {}
        expected_execution = {"scratchGeneration": scratch["scratchGeneration"],
                              "effectiveEnvironmentDigest": digest(scratch["effectiveEnvironment"])}
        if (not native or native["key"] != manifest.get("nativeJobId")
                or job.get("nativeJobId") != native["key"] or job.get("role") != "phase"
                or job.get("owner") != execution_id or not isinstance(execution_id, str) or not execution_id
                or not host_row or host.get("executionId") != execution_id
                or host.get("taskScratch") != scratch or not execution_row
                or execution.get("taskScratch") != expected_execution
                or host.get("operationId") != execution.get("operationId")
                or str(host.get("generation")) != str(execution.get("generation"))
                or not scratch_row or registered.get("executionId") != execution_id
                or registered.get("status") != "owned"
                or registered.get("scratchGeneration") != scratch["scratchGeneration"]
                or registered.get("effectiveEnvironment") != scratch["effectiveEnvironment"]):
            raise _failure("receipt_scratch_unproven", "Receipt lacks its original execution and prelaunch scratch binding")
        proof = job.get("completeProof")
        identity = proof.get("identity") if isinstance(proof, dict) else None
        terminal = receipt.get("terminalProof") or {}
        if (job.get("status") != "terminal" or not isinstance(proof, dict)
                or proof.get("complete") is not True or job.get("terminalProof") != proof
                or type(proof.get("processExitCode")) is not int or proof["processExitCode"] != 0
                or proof.get("activeProcesses") not in (None, 0)
                or any(proof.get(name) is not True for name in ("stdoutEof", "stderrEof", "childrenGone"))
                or type(job.get("pid")) is not int or job["pid"] <= 0
                or not job.get("birthToken") or not job.get("executable") or not isinstance(identity, dict)
                or identity.get("pid") != job["pid"] or identity.get("creationTime") != job["birthToken"]
                or identity.get("executable") != job["executable"]
                or manifest.get("birthToken") != job["birthToken"]
                or receipt.get("executionId") != manifest.get("executionId")
                or terminal.get("nativeJobId") != native["key"] or terminal.get("birthToken") != job["birthToken"]
                or terminal.get("pid") != job["pid"]
                or any(terminal.get(name) != value for name, value in proof.items())):
            raise _failure("receipt_scratch_unproven", "Scratch evidence must belong to the exact terminal native phase")
        launch = self.state.get("execution_launch", execution_id, connection=connection)
        guard_id = launch["payload"].get("guardNativeJobId") if launch else None
        if guard_id or job.get("guardNativeJobId"):
            guard = self.state.get("native_job", guard_id, connection=connection)
            guard_payload = guard["payload"] if guard else {}
            launcher = launch["payload"].get("launcherIdentity", {}) if launch else {}
            if (not guard_id or job.get("guardNativeJobId") != guard_id or not guard
                    or guard["key"] != guard_id or guard_payload.get("nativeJobId") != guard_id
                    or guard_payload.get("role") != "execution_host_guard"
                    or guard_payload.get("owner") != execution_id or guard_payload.get("executionId") != execution_id
                    or not isinstance(launcher, dict) or type(launcher.get("pid")) is not int
                    or launcher["pid"] <= 0 or not launcher.get("creationTime")
                    or host.get("hostPid") != launcher["pid"]
                    or host.get("hostBirthToken") != launcher["creationTime"]
                    or guard_payload.get("pid") != launcher["pid"]
                    or guard_payload.get("birthToken") != launcher["creationTime"]):
                raise _failure("receipt_scratch_unproven", "Scratch phase differs from its registered execution guard")
            if require_phase:
                from ..processes.guards import guard_phases_complete
                guard_proof = guard_payload.get("completeProof")
                guard_identity = guard_proof.get("identity") if isinstance(guard_proof, dict) else None
                if (guard_payload.get("status") != "terminal" or not isinstance(guard_proof, dict)
                        or guard_proof.get("complete") is not True
                        or guard_payload.get("terminalProof") != guard_proof
                        or not isinstance(guard_identity, dict)
                        or guard_identity.get("pid") != guard_payload.get("pid")
                        or guard_identity.get("creationTime") != guard_payload.get("birthToken")
                        or guard_identity.get("executable") != guard_payload.get("executable")
                        or guard_proof.get("activeProcesses") != 0
                        or any(guard_proof.get(name) is not True for name in ("stdoutEof", "stderrEof", "childrenGone"))
                        or not guard_phases_complete(self.state, guard, connection=connection)):
                    raise JenkinsError("receipt_scratch_pending", "The scratch execution guard is not terminal", retryable=True)
        if not require_phase:
            return  # The host publishes phaseRecords after collecting this receipt.
        if not matches:
            raise JenkinsError("receipt_scratch_pending", "The scratch phase evidence is not yet published", retryable=True)
        if (len(matches) != 1 or not isinstance(manifest.get("phase"), str) or not manifest["phase"]
                or matches[0].get("phase") != manifest["phase"]
                or matches[0].get("stageReceipts", {}).get(manifest.get("stage")) != manifest.get("executionId")
                or matches[0].get("receipts", {}).get(manifest.get("stage")) != receipt
                or matches[0].get("terminalProof") != proof
                or _same_task_scratch(matches[0], manifest) != evidence):
            raise _failure("receipt_scratch_unproven", "Receipt differs from its persisted native phase evidence")
        published = execution.get("phaseRecords")
        if published is not None:
            copied = ([phase for phase in published if isinstance(phase, dict)
                       and phase.get("nativeJobId") == native["key"]] if isinstance(published, list) else [])
            if len(copied) != 1 or copied[0] != matches[0]:
                raise _failure("receipt_scratch_unproven", "Execution publication differs from its original scratch phase")

    def accept(self, execution_id: str, expected: dict[str, Any]) -> dict[str, Any]:
        receipt = self.get(execution_id)
        if receipt is None:
            raise _failure("acceptance_receipt_missing", "No trusted terminal receipt exists for execution")
        validate_producer_receipt(receipt, expected=expected, required_outputs=expected.get("requiredOutputs"))
        acceptance_id = digest({k: expected.get(k, receipt.get(k)) for k in ("sourceInputDigest", "coverageDigest", "generation", "executionId")})
        acceptance = issue_acceptance_receipt(receipt, acceptance_id=acceptance_id,
                                               output_root=receipt["outputClosure"].get("root"),)
        old = self.state.get("acceptance", acceptance_id)
        if old:
            return validate_acceptance_receipt(old["payload"], producer_receipt=receipt)
        self.state.put("acceptance", acceptance_id, acceptance)
        return acceptance

    def accept_flow(self, flow: dict[str, Any]) -> dict[str, Any]:
        """Atomically accept this consumer's trusted, published workflow evidence."""
        flow_id = flow.get("flowId")
        if not isinstance(flow_id, str) or not flow_id:
            raise _failure("workflow_identity_missing", "Acceptance flow requires flowId")
        authority = self.state.get("workflow", flow_id)
        if not authority:
            raise _failure("workflow_unknown", "Acceptance flow is not registered in State")
        contract = authority["payload"]
        required = self._flow_stages(contract)
        # Reading an existing acceptance preserves history after refs are
        # released or outputs are collected. It never issues a new receipt.
        if contract.get("status") == "accepted":
            with self.state.transaction() as connection:
                return self._accepted_flow(flow_id, flow, connection)
        heavy = bool(set(required) & HEAVY_STAGES)
        if heavy:
            from ..workflow.execution import execution_lock
            lock = execution_lock(self.state, contract.get("executionId"))
        else:
            lock = nullcontext()
        # The execution key lock precedes SQLite, matching finish and cancel.
        with lock:
            with self.state.transaction() as connection:
                fresh = self.state.get("workflow", flow_id, connection=connection)
                if not fresh:
                    raise _failure("workflow_unknown", "Acceptance flow is not registered in State")
                if (fresh["payload"].get("executionId") != contract.get("executionId")
                        or bool(set(self._flow_stages(fresh["payload"])) & HEAVY_STAGES) != heavy):
                    raise _failure("workflow_contract_mismatch", "Workflow execution changed before acceptance")
                return self._accept_flow(flow_id, flow, fresh, connection)

    def _accepted_flow(self, flow_id, flow, connection):
        record = self.state.get("workflow", flow_id, connection=connection)
        if not record or record["payload"].get("status") != "accepted":
            raise _failure("workflow_contract_mismatch", "Workflow acceptance changed before observation")
        contract = record["payload"]
        self._flow_contract(flow, contract)
        receipt = contract.get("acceptance")
        stored = (self.state.get("acceptance", receipt.get("acceptanceId"), connection=connection)
                  if isinstance(receipt, dict) else None)
        if not stored or stored["payload"] != receipt:
            raise _failure("workflow_acceptance_missing", "Accepted workflow lacks its authoritative stored receipt")
        validate_acceptance_receipt(receipt)
        if receipt.get("acceptanceId") != self._flow_acceptance_id(flow_id, contract):
            raise _failure("workflow_acceptance_mismatch", "Historical acceptance differs from its workflow")
        final_ref = contract.get("stageReceipts", {}).get(self._flow_stages(contract)[-1])
        attempt = self.state.get("validation_attempt", final_ref, connection=connection)
        manifest = attempt["payload"] if attempt else {}
        producer = manifest.get("receipt")
        if any(name in value for value in (receipt, manifest, producer if isinstance(producer, dict) else {})
               for name in ("taskScratch", "effectiveEnvironmentDigest")):
            producer = self.get(final_ref, connection=connection)
            if producer is None:
                raise _failure("workflow_stage_unaccepted", "Historical scratch acceptance lacks its original producer")
            # Retained State proof remains checkable after output collection.
            validate_acceptance_receipt(receipt, producer_receipt=producer)
        return receipt

    def _accept_request(self, flow_id, contract, acceptance, connection):
        """Project this accepted consumer into its exact registered request."""
        identity = contract.get("identity")
        if not isinstance(identity, dict):
            return  # Isolated receipt fixtures need not register a request.
        repository = contract.get("repositoryId") or identity.get("repositoryId")
        session, request_id = identity.get("sessionId"), identity.get("requestId")
        formal = bool(self.repo_root and (self.repo_root / ".jenkins/state/deployment/driver.json").is_file())
        if not repository or not session or not request_id:
            if not formal:
                return
            raise _failure("workflow_request_mismatch", "Workflow has no complete request identity")
        request = self.state.get_request(repository, session, request_id, connection=connection)
        if request is None:
            if formal:
                raise _failure("workflow_request_missing", "Formal acceptance requires its original registered request")
            return  # Receipt authority also supports isolated stage fixtures.
        generation = identity.get("generation")
        if (request["generation"] != generation or contract.get("generation") != generation
                or identity.get("repositoryId", repository) != repository):
            raise _failure("workflow_request_mismatch", "Accepted consumer differs from its request generation")
        payload = request.get("payload") or {}
        nested = payload.get("identity") or {}
        expected = {"attemptId": identity.get("attemptId"),
                    "sourceInputDigest": contract.get("sourceDigest"),
                    "coverageDigest": contract.get("coverageDigest"),
                    "stageImplementationDigest": contract.get("driverDigest")}
        aliases = {"sourceInputDigest": ("sourceInputDigest", "sealedInputRef", "sourceDigest"),
                   "stageImplementationDigest": ("stageImplementationDigest", "driverDigest")}
        for field, value in expected.items():
            candidates = [mapping[name] for mapping in (payload, nested)
                          for name in aliases.get(field, (field,))
                          if name in mapping and mapping[name] not in (None, "")]
            if value in (None, "") or not candidates or any(candidate != value for candidate in candidates):
                raise _failure("workflow_request_mismatch", "Accepted consumer differs from its immutable request", field=field)
        result = {"flowId": flow_id, "receiptRef": acceptance["acceptanceId"]}
        if request["status"] == "accepted":
            previous = self.state.get("request_result", digest([repository, session, request_id]), connection=connection)
            if not previous or previous["payload"].get("result") != result:
                raise _failure("workflow_request_mismatch", "Request was accepted by another consumer receipt")
            return
        if request["status"] in {"failed", "cancelled", "cancel_requested"}:
            raise _failure("workflow_request_terminal", "A terminal or cancelled request cannot accept a consumer")
        self.state.transition_request(repository, session, request_id, generation,
                                      request["status"], "accepted", result, connection=connection)

    def reconcile_request(self, flow):
        """Reconcile only an already accepted flow into its original request."""
        flow_id = flow.get("flowId")
        if not isinstance(flow_id, str) or not isinstance(flow.get("identity"), dict):
            raise _failure("workflow_identity_missing", "Request reconciliation requires the original full flow identity")
        with self.state.transaction() as connection:
            record = self.state.get("workflow", flow_id, connection=connection)
            if not record or record["payload"].get("status") != "accepted":
                raise _failure("workflow_unaccepted", "Request reconciliation cannot issue an acceptance")
            contract = record["payload"]
            if flow["identity"] != contract.get("identity"):
                raise _failure("workflow_request_mismatch", "Request reconciliation belongs to another consumer")
            identity = contract["identity"]
            self.state.check_authorization(contract.get("repositoryId") or identity.get("repositoryId"),
                                           identity.get("sessionId"), "validation",
                                           contract.get("ownedPaths", []), connection=connection)
            acceptance = self._accepted_flow(flow_id, flow, connection)
            self._accept_request(flow_id, contract, acceptance, connection)
            request = self.state.get_request(contract.get("repositoryId") or identity.get("repositoryId"),
                                             identity.get("sessionId"), identity.get("requestId"), connection=connection)
            if request is None:
                raise _failure("workflow_request_missing", "Accepted flow has no original request to reconcile")
            return {"status": "accepted", "requestId": request["requestId"],
                    "flowId": flow_id, "receiptRef": acceptance["acceptanceId"]}

    @staticmethod
    def _flow_stages(contract):
        stages = contract.get("requiredStages", [])
        if (not isinstance(stages, list) or not stages
                or any(not isinstance(stage, str) or stage not in LIGHT_STAGES | HEAVY_STAGES for stage in stages)
                or len(stages) != len(set(stages))):
            raise _failure("workflow_stages_missing", "Workflow requires distinct approved stages")
        return stages

    @staticmethod
    def _flow_identity(contract):
        return {name: contract.get(name) for name in ("sourceDigest", "coverageDigest", "driverDigest",
                                                     "generation", "beforePatch", "dagDigest")}

    def _flow_acceptance_id(self, flow_id, contract):
        return digest({"flow": flow_id, "identity": self._flow_identity(contract),
                       "stages": list(contract.get("requiredStages", []))})

    @staticmethod
    def _flow_contract(flow, contract):
        for key in ("requiredStages", "stageReceipts", "requiredOutputs", "requiredOutputsByStage",
                    "stageRecipeDigests", "executionId", "sourceDigest", "coverageDigest", "driverDigest",
                    "generation", "beforePatch", "dagDigest", "identity"):
            if key in flow and flow[key] != contract.get(key):
                raise _failure("workflow_contract_mismatch", "Caller cannot override workflow stage contract", field=key)

    def _consumer_outputs(self, flow_id, contract, connection):
        from ..resources import canonical_build_root, physical_path_under
        execution_id = contract.get("executionId")
        execution = self.state.get("execution", execution_id, connection=connection)
        if not execution:
            raise _failure("workflow_execution_missing", "Heavy workflow lacks its persisted execution")
        payload = execution["payload"]
        if payload.get("status") != "passed" or payload.get("cancelRequested"):
            raise JenkinsError("workflow_execution_pending", "Execution must finish publication before acceptance", retryable=True)
        consumer = self.state.get("execution_consumer", f"{execution_id}:{flow_id}", connection=connection)
        if (not consumer or consumer["payload"].get("status") != "active"
                or consumer["payload"].get("executionId") != execution_id
                or consumer["payload"].get("consumerId") != flow_id
                or flow_id not in payload.get("consumers", [])):
            raise _failure("workflow_consumer_inactive", "Only this execution's active consumer may accept")
        binding = self.state.get("execution_binding", digest({"consumer": flow_id,
                                "executionKey": payload.get("executionKeyDigest")}), connection=connection)
        if (not binding or binding["payload"].get("executionId") != execution_id
                or binding["payload"].get("consumerId") != flow_id):
            raise _failure("workflow_consumer_mismatch", "Consumer lacks its immutable execution binding")
        identity = {name: payload.get(name) for name in ("sourceDigest", "recipeDigest", "coverageDigest", "driverDigest")}
        for name in ("sourceDigest", "coverageDigest", "driverDigest"):
            _required_digest(payload, name)
            if identity[name] != contract.get(name):
                raise _failure("workflow_execution_mismatch", "Workflow differs from its persisted execution", field=name)
        if str(payload.get("generation")) != str(contract.get("generation")):
            raise _failure("workflow_execution_mismatch", "Workflow differs from its execution generation")
        for stage in set(contract.get("requiredStages", [])) & HEAVY_STAGES:
            if (contract.get("stageRecipeDigests", {}).get(stage) != identity["recipeDigest"]
                    or contract.get("stageReceipts", {}).get(stage) != payload.get("stageReceipts", {}).get(stage)):
                raise _failure("workflow_execution_mismatch", "Heavy stage must belong to this execution", stage=stage)
        bundle_row = self.state.get("artifact_bundle", execution_id, connection=connection)
        bundle = bundle_row["payload"] if bundle_row else {}
        files, bundle_identity = bundle.get("files"), bundle.get("identity")
        if (bundle.get("status") != "published" or bundle.get("executionId") != execution_id
                or bundle.get("executionKey") != payload.get("executionKeyDigest")
                or not isinstance(bundle_identity, dict)
                or any(bundle_identity.get(name) != value for name, value in identity.items())
                or not isinstance(files, list) or not files
                or bundle.get("bundleDigest") != digest({"executionKey": bundle.get("executionKey"),
                                                         "identity": bundle_identity, "files": files})
                or payload.get("artifacts") != bundle):
            raise _failure("workflow_artifact_bundle_missing", "Execution lacks its complete published output identity")
        if any(not isinstance(record.get("buildRoot"), str) or not record["buildRoot"]
               for record in (contract, payload, bundle)):
            raise _failure("workflow_artifact_root_mismatch", "Publication requires an explicit approved root identity")
        root = canonical_build_root(contract["buildRoot"])
        if (canonical_build_root(payload.get("buildRoot")).path != root.path
                or canonical_build_root(bundle.get("buildRoot")).path != root.path):
            raise _failure("workflow_artifact_root_mismatch", "Consumer and publication must use the same approved root")
        objects = physical_path_under(root, root.namespace() / "artifacts" / "artifacts" / "objects", allow_missing=False)
        refs = []
        for item in files:
            if not isinstance(item, dict):
                raise _failure("workflow_artifact_invalid", "Published bundle contains an invalid file")
            object_digest = require_digest(item.get("digest"), "artifactDigest")
            relative, size = item.get("path"), item.get("size")
            if (not isinstance(relative, str) or not relative or Path(relative).is_absolute()
                    or ".." in Path(relative).parts or type(size) is not int or size < 0
                    or item.get("executionId") != execution_id):
                raise _failure("workflow_artifact_invalid", "Published file identity is incomplete")
            ref_key = f"{flow_id}:{object_digest}"
            ref = self.state.get("artifact_ref", ref_key, connection=connection)
            ref_payload = ref["payload"] if ref else {}
            if (ref_payload.get("status") != "active" or ref_payload.get("owner") != flow_id
                    or ref_payload.get("kind") != "consumer" or ref_payload.get("executionId") != execution_id
                    or ref_payload.get("digest") != object_digest):
                raise _failure("workflow_artifact_ref_missing", "Consumer lacks its active published artifact reference")
            registered = self.state.get("artifact_object", object_digest, connection=connection)
            obj = registered["payload"] if registered else {}
            if (obj.get("status") not in {"ready", "published"} or obj.get("digest") != object_digest
                    or obj.get("size") != size):
                raise _failure("workflow_artifact_object_missing", "Published artifact object is unavailable")
            path = physical_path_under(root, objects / object_digest, allow_missing=False)
            if not path.is_file() or path.stat().st_size != size or file_digest(path) != object_digest:
                raise _failure("workflow_artifact_changed", "Published artifact bytes differ from consumer evidence")
            refs.append(ref_key)
        return {"executionId": execution_id, "executionKeyDigest": payload["executionKeyDigest"],
                "consumerId": flow_id, "bundleDigest": bundle["bundleDigest"], "artifactRefs": sorted(set(refs))}

    def _accept_flow(self, flow_id, flow, authority, connection):
        contract = authority["payload"]
        self._flow_contract(flow, contract)
        if contract.get("status") == "accepted":
            return self._accepted_flow(flow_id, flow, connection)
        if contract.get("status") in {"failed", "cancelled", "rejected"} or contract.get("consumerCancelled") or contract.get("cancelRequested"):
            raise _failure("workflow_terminal", "Cancelled or failed workflow cannot receive a new acceptance")
        required = self._flow_stages(contract)
        receipts = contract.get("stageReceipts", {})
        recipe_map = contract.get("stageRecipeDigests", {})
        if not isinstance(recipe_map, dict) or any(stage not in recipe_map for stage in required):
            raise _failure("workflow_recipe_contract_missing", "Workflow lacks immutable recipe digest for every stage")
        if not required or not isinstance(receipts, dict):
            raise _failure("workflow_stages_missing", "Acceptance flow must enumerate required stages and receipts")
        outputs = self._consumer_outputs(flow_id, contract, connection) if set(required) & HEAVY_STAGES else None
        collected = []
        for stage in required:
            execution_id = receipts.get(stage)
            if not execution_id:
                raise _failure("workflow_stage_missing", "Required workflow stage has no receipt", stage=stage)
            value = self.get(execution_id, connection=connection)
            if value is None:
                raise _failure("workflow_stage_unaccepted", "Required workflow stage is not terminal and trusted", stage=stage)
            attempt = self.state.get("validation_attempt", execution_id, connection=connection)
            if not attempt or attempt["payload"].get("stage") != stage:
                raise _failure("workflow_stage_unregistered", "Stage receipt lacks registered attempt", stage=stage)
            registered = self.state.get("validation_receipt", execution_id, connection=connection)
            if not registered or registered["payload"] != value:
                raise _failure("workflow_stage_unaccepted", "Stage receipt differs from its trusted collector", stage=stage)
            expected = {"executionId": execution_id, "stage": stage,
                        "sourceDigest": contract.get("sourceDigest"),
                        "coverageDigest": contract.get("coverageDigest"),
                        "driverDigest": contract.get("driverDigest"),
                        "recipeDigest": recipe_map[stage]}
            by_stage = contract.get("requiredOutputsByStage", {})
            validate_producer_receipt(value, expected=expected, output_root=value["outputClosure"]["root"],
                                      required_outputs=by_stage.get(stage, contract.get("requiredOutputs", [])))
            collected.append(value)
        if outputs is not None:
            bundle = self.state.get("artifact_bundle", contract["executionId"], connection=connection)["payload"]
            by_stage = {value["stage"]: value for value in collected}
            for item in bundle["files"]:
                producer = by_stage.get(item.get("stage"))
                entry = {"path": item["path"], "sha256": item["digest"], "size": item["size"]}
                if (not producer or item.get("stage") not in HEAVY_STAGES
                        or entry not in producer["outputClosure"]["files"]):
                    raise _failure("workflow_artifact_mismatch", "Published artifact must belong to a required stage receipt")
        identity = self._flow_identity(contract)
        acceptance_id = self._flow_acceptance_id(flow_id, contract)
        old = self.state.get("acceptance", acceptance_id, connection=connection)
        if old:
            acceptance = validate_acceptance_receipt(old["payload"])
            if (acceptance.get("flowIdentity") != identity or acceptance.get("stages") != required
                    or acceptance.get("producerReceiptDigest") != digest(collected[-1])
                    or outputs is not None and acceptance.get("executionEvidence") != outputs):
                raise _failure("workflow_acceptance_mismatch", "Stored acceptance differs from workflow evidence")
        else:
            acceptance = issue_acceptance_receipt(collected[-1], acceptance_id=acceptance_id)
            acceptance["stages"] = required
            acceptance["flowIdentity"] = identity
            if outputs is not None:
                acceptance["executionEvidence"] = outputs
            acceptance["receiptDigest"] = digest({k: v for k, v in acceptance.items() if k != "receiptDigest"})
            self.state.put("acceptance", acceptance_id, acceptance, expected_version=0, connection=connection)
        stages = [dict(item) for item in contract.get("stages", [])]
        by_stage = {value["stage"]: value for value in collected}
        for item in stages:
            if item.get("name") in by_stage:
                item.update(status="passed", receipt=by_stage[item["name"]])
            elif item.get("name") == "acceptance":
                item.update(status="passed", receipt=acceptance)
        self.state.put("workflow", flow_id, {**contract, "status": "accepted", "stages": stages,
                                           "acceptance": acceptance}, expected_version=authority["version"],
                       connection=connection)
        self._accept_request(flow_id, contract, acceptance, connection)
        return acceptance


def issue_acceptance_receipt(producer_receipt: dict[str, Any], *, acceptance_id: str | None = None,
                             authority: str = "jenkins-validation", output_root: str | os.PathLike[str] | None = None) -> dict[str, Any]:
    """Validate producer evidence and issue the authoritative acceptance receipt."""
    producer = validate_producer_receipt(producer_receipt, output_root=output_root)
    receipt = {
        "schemaVersion": SCHEMA_VERSION, "kind": ACCEPTANCE_KIND, "status": "accepted",
        "acceptanceId": acceptance_id or uuid4().hex, "authority": authority, "acceptedAt": time.time(),
        "executionId": producer["executionId"], "stage": producer["stage"],
        "sourceDigest": producer["sourceDigest"], "recipeDigest": producer["recipeDigest"],
        "coverageDigest": producer["coverageDigest"], "driverDigest": producer["driverDigest"],
        "producerReceiptDigest": digest(producer), "outputClosure": producer["outputClosure"],
    }
    receipt.update(_task_scratch_evidence(producer))
    receipt["receiptDigest"] = digest(receipt)
    return receipt


def validate_acceptance_receipt(receipt: dict[str, Any], *, producer_receipt: dict[str, Any] | None = None,
                                output_root: str | os.PathLike[str] | None = None) -> dict[str, Any]:
    if not isinstance(receipt, dict) or receipt.get("schemaVersion") != SCHEMA_VERSION or receipt.get("kind") != ACCEPTANCE_KIND:
        raise _failure("fabricated_acceptance", "Receipt is not an authoritative Jenkins acceptance receipt")
    if receipt.get("status") != "accepted" or receipt.get("authority") != "jenkins-validation":
        raise _failure("acceptance_not_authoritative", "Receipt was not issued by the validation authority")
    stored = receipt.get("receiptDigest")
    if not isinstance(stored, str):
        raise _failure("acceptance_digest_missing", "Acceptance receipt digest is missing")
    unsigned = dict(receipt); unsigned.pop("receiptDigest", None)
    if stored != digest(unsigned):
        raise _failure("acceptance_digest_mismatch", "Acceptance receipt was modified")
    _task_scratch_evidence(receipt)
    if producer_receipt is not None:
        producer = validate_producer_receipt(producer_receipt, output_root=output_root)
        _same_task_scratch(receipt, producer)
        if receipt.get("producerReceiptDigest") != digest(producer):
            raise _failure("acceptance_producer_mismatch", "Acceptance receipt is not bound to producer evidence")
    return dict(receipt)


__all__ = ["ACCEPTANCE_KIND", "PRODUCER_KIND", "closure_manifest", "validate_producer_receipt", "issue_acceptance_receipt", "validate_acceptance_receipt"]
