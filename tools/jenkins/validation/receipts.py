"""Managed producer evidence and authoritative acceptance receipts.

Jenkins' UI result is not an acceptance result.  Heavy recipes require
terminal process evidence and every recipe requires a content-addressed output
closure.  Arbitrary ``success.json`` files are deliberately rejected.
"""
from __future__ import annotations

import os
import time
import fnmatch
from pathlib import Path
from typing import Any, Iterable
from uuid import uuid4

from ..contracts import JenkinsError, digest, file_digest, require_digest

SCHEMA_VERSION = 1
PRODUCER_KIND = "jenkins.managed.producer-receipt"
ACCEPTANCE_KIND = "jenkins.authoritative.acceptance-receipt"
LIGHT_STAGES = frozenset({"patch", "comments", "syntax", "format", "acceptance"})
HEAVY_STAGES = frozenset({"build", "unit_test", "integration_test", "regression_test"})
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
        root = closure.get("root") if isinstance(closure, dict) else None
        expected_root = record["payload"].get("outputRoot") or record["payload"].get("buildRoot")
        if expected_root and Path(root).absolute() != Path(expected_root).absolute():
            raise _failure("output_root_mismatch", "Collector output closure root differs from registered root")
        required = record["payload"].get("requiredOutputs", [])
        validate_producer_receipt(receipt, output_root=root, required_outputs=required)
        with self.state.transaction() as connection:
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
        validate_producer_receipt(receipt)
        with self.state.transaction() as connection:
            self.state.put("validation_receipt", execution_id, receipt, connection=connection)
            self.state.put("validation_attempt", execution_id, {**record["payload"], "status": "passed", "receipt": receipt}, expected_version=record["version"], connection=connection)
        return receipt

    def get(self, execution_id: str) -> dict[str, Any] | None:
        record = self.state.get("validation_attempt", execution_id)
        if not record or record["payload"].get("status") != "passed":
            return None
        return record["payload"].get("receipt")

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
            return old["payload"]
        self.state.put("acceptance", acceptance_id, acceptance)
        return acceptance

    def accept_flow(self, flow: dict[str, Any]) -> dict[str, Any]:
        """Accept a workflow only when every required stage has a State receipt."""
        flow_id = flow.get("flowId")
        if not flow_id:
            raise _failure("workflow_identity_missing", "Acceptance flow requires flowId")
        authority = self.state.get("workflow", flow_id)
        if not authority:
            raise _failure("workflow_unknown", "Acceptance flow is not registered in State")
        contract = authority["payload"]
        required = list(contract.get("requiredStages", []))
        receipts = contract.get("stageReceipts", {})
        recipe_map = contract.get("stageRecipeDigests", {})
        if not isinstance(recipe_map, dict) or any(stage not in recipe_map for stage in required):
            raise _failure("workflow_recipe_contract_missing", "Workflow lacks immutable recipe digest for every stage")
        for key in ("requiredStages", "stageReceipts", "requiredOutputs", "stageRecipeDigests"):
            if key in flow and flow[key] != contract.get(key):
                raise _failure("workflow_contract_mismatch", "Caller cannot override workflow stage contract", field=key)
        if not required or not isinstance(receipts, dict):
            raise _failure("workflow_stages_missing", "Acceptance flow must enumerate required stages and receipts")
        collected = []
        for stage in required:
            execution_id = receipts.get(stage)
            if not execution_id:
                raise _failure("workflow_stage_missing", "Required workflow stage has no receipt", stage=stage)
            value = self.get(execution_id)
            if value is None:
                raise _failure("workflow_stage_unaccepted", "Required workflow stage is not terminal and trusted", stage=stage)
            attempt = self.state.get("validation_attempt", execution_id)
            if not attempt or attempt["payload"].get("stage") != stage:
                raise _failure("workflow_stage_unregistered", "Stage receipt lacks registered attempt", stage=stage)
            expected = {"executionId": execution_id, "stage": stage,
                        "sourceDigest": contract.get("sourceDigest"),
                        "coverageDigest": contract.get("coverageDigest"),
                        "driverDigest": contract.get("driverDigest"),
                        "recipeDigest": recipe_map[stage]}
            by_stage = contract.get("requiredOutputsByStage", {})
            validate_producer_receipt(value, expected=expected, output_root=value["outputClosure"]["root"],
                                      required_outputs=by_stage.get(stage, contract.get("requiredOutputs", [])))
            collected.append(value)
        identity = {"sourceDigest": contract.get("sourceDigest"), "coverageDigest": contract.get("coverageDigest"), "driverDigest": contract.get("driverDigest"), "generation": contract.get("generation"), "beforePatch": contract.get("beforePatch"), "dagDigest": contract.get("dagDigest")}
        acceptance_id = digest({"flow": flow_id, "identity": identity, "stages": required})
        old = self.state.get("acceptance", acceptance_id)
        if old: return old["payload"]
        acceptance = issue_acceptance_receipt(collected[-1], acceptance_id=acceptance_id)
        acceptance["stages"] = required
        acceptance["flowIdentity"] = identity
        acceptance["receiptDigest"] = digest({k: v for k, v in acceptance.items() if k != "receiptDigest"})
        self.state.put("acceptance", acceptance_id, acceptance)
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
    if producer_receipt is not None:
        producer = validate_producer_receipt(producer_receipt, output_root=output_root)
        if receipt.get("producerReceiptDigest") != digest(producer):
            raise _failure("acceptance_producer_mismatch", "Acceptance receipt is not bound to producer evidence")
    return dict(receipt)


__all__ = ["ACCEPTANCE_KIND", "PRODUCER_KIND", "closure_manifest", "validate_producer_receipt", "issue_acceptance_receipt", "validate_acceptance_receipt"]
