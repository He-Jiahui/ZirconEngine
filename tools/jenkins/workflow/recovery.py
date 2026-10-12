"""Durable failure compensation over source and native process authorities."""
from __future__ import annotations

import hashlib
from pathlib import Path
import subprocess

from ..contracts import JenkinsError, digest, file_digest
from ..source import (PathClaim, canonical_path, repository_id, _before_hash,
                      _atomic_write, _path_for)
from ..resources import canonical_build_root, physical_path_under


def _termination(state, flow):
    eid = flow.get("executionId")
    execution = state.get("execution", eid) if eid else None
    jobs = [row for row in state.list("native_job")
            if row["payload"].get("executionId") == eid or row["payload"].get("owner") == eid]
    proofs = []
    from ..processes.guards import abort_evidence
    abort = abort_evidence(state, eid) if eid else None
    for row in jobs:
        payload = row["payload"]
        proof = payload.get("completeProof")
        if not abort and (payload.get("status") != "terminal" or not isinstance(proof, dict) or proof.get("complete") is not True):
            raise JenkinsError("native_termination_unproven", "compensation requires every native process to terminate", retryable=True)
        proofs.append({"nativeJobId": row["key"], "proofDigest": digest(proof) if proof else digest(abort)})
    if execution:
        payload = execution["payload"]
        active = [row for row in state.list("execution_consumer")
                  if row["payload"].get("executionId") == eid
                  and row["payload"].get("status") == "active"]
        if active:
            raise JenkinsError("shared_execution_active", "another active consumer still references this input", retryable=True)
        launch = state.get("execution_launch", eid)
        host = state.get("execution_host", eid)
        if not jobs and (host or (launch and not launch["payload"].get("cancelledBeforeClaim"))):
            raise JenkinsError("native_termination_unproven", "execution launch lacks a native termination proof", retryable=True)
        if payload.get("status") not in {"failed", "cancelled", "passed", "rejected"}:
            raise JenkinsError("execution_not_terminal", "execution must be terminal before source compensation", retryable=True)
    return {"executionId": eid, "nativeProofs": proofs, "abortProof": abort,
            "noNativeLaunch": not execution and not jobs}


def _before_bytes(operation, root, flow, relative, checksum):
    if checksum is None:
        return None
    payload = operation["payload"]
    objects = payload.get("beforeObjects")
    if isinstance(objects, dict) and relative in objects:
        if objects[relative] != checksum:
            raise JenkinsError("before_object_mismatch", "original bytes reference differs from its before hash")
        approved = canonical_build_root(flow["buildRoot"])
        object_root = physical_path_under(approved, payload.get("objectRoot", ""))
        path = physical_path_under(approved, object_root / "inputs" / "objects" / checksum)
        if not path.is_file() or file_digest(path) != checksum:
            raise JenkinsError("before_object_corrupt", "original source bytes are unavailable or corrupt")
        return path.read_bytes()
    # Legacy journals may recover only a byte-identical declared Git base.
    base = (flow.get("sourceManifest") or {}).get("baseHead")
    if not base:
        raise JenkinsError("before_object_missing", "legacy patch has no trustworthy original bytes")
    result = subprocess.run(["git", "-C", str(root), "show", f"{base}:{relative}"], capture_output=True)
    if result.returncode or hashlib.sha256(result.stdout).hexdigest() != checksum:
        raise JenkinsError("before_object_missing", "Git base differs from original uncommitted bytes")
    return result.stdout


def _compensate_patch(state, operation, flow, proof):
    patch = operation["payload"]
    root = Path(flow["repoRoot"]).absolute()
    owner = flow["identity"]["attemptId"]
    if patch.get("owner") != owner:
        raise JenkinsError("patch_owner_mismatch", "compensation may only undo this attempt's patch")
    if patch.get("repositoryRoot", str(root)).casefold() != str(root).casefold():
        raise JenkinsError("patch_repository_mismatch", "compensation journal belongs to another checkout")
    paths = [canonical_path(root, p) for p in patch["paths"]]
    state.check_authorization(flow["repositoryId"], flow["identity"]["sessionId"], "implementation", paths)
    before = patch["beforeHashes"]
    after = (operation.get("result") or {}).get("afterHashes") or patch.get("expectedAfterHashes")
    if not isinstance(after, dict) or set(after) != set(paths):
        raise JenkinsError("patch_after_missing", "patch lacks its complete expected after hashes")
    op_id = digest({"patch": operation["operationId"], "recoveryOwner": owner})
    journal = state.get_operation(op_id)
    if journal and journal["status"] == "complete":
        return journal
    if journal and journal["status"] == "blocked":
        raise JenkinsError("compensation_blocked", "earlier compensation stopped at a foreign edit", details=journal["result"])
    data = {p: _before_bytes(operation, root, flow, p, before[p]) for p in paths}
    repo_id = repository_id(root)
    claims = {row["payload"].get("path"): row for row in state.list("path_claims")
              if row["payload"].get("repositoryId") == repo_id}
    for p in paths:
        current = _before_hash(root, p)
        if current != after[p] and not (journal and current == before[p]):
            raise JenkinsError("foreign_edit_during_compensation", "later source edits prevent compensation", details={"path": p})
        for claimed_path, row in claims.items():
            if (claimed_path == p or claimed_path.startswith(p + "/") or p.startswith(claimed_path + "/")) and row["payload"].get("owner") != owner:
                raise JenkinsError("foreign_claim_during_compensation", "another source owner prevents compensation")
    if journal is None:
        journal = state.put_operation("source_compensation", op_id,
            {"patchOperationId": operation["operationId"], "owner": owner,
             "repositoryRoot": str(root), "paths": paths, "beforeHashes": before,
             "afterHashes": after, "terminationEvidence": proof})
    restored = list((journal.get("result") or {}).get("restoredPaths", []))
    try:
        for p in paths:
            current = _before_hash(root, p)
            if current == before[p]:
                if p not in restored:
                    restored.append(p)
                continue
            if current != after[p]:
                raise JenkinsError("foreign_edit_during_compensation", "source changed during compensation", details={"path": p})
            destination = _path_for(root, p)
            if data[p] is None:
                destination.unlink(missing_ok=True)
            else:
                _atomic_write(destination, data[p])
            if _before_hash(root, p) != before[p]:
                raise JenkinsError("foreign_edit_during_compensation", "restored source changed before receipt")
            restored.append(p)
            state.transition_operation(op_id, "prepared", "prepared", {"restoredPaths": restored})
        with state.transaction() as connection:
            for row in claims.values():
                if row["payload"].get("path") in paths and row["payload"].get("owner") == owner:
                    state.delete("path_claims", row["key"], expected_version=row["version"], connection=connection)
            state.transition_operation(op_id, "prepared", "complete", {"restoredPaths": restored, "terminationEvidence": proof}, connection=connection)
        return state.get_operation(op_id)
    except BaseException as error:
        state.transition_operation(op_id, "prepared", "blocked", {"restoredPaths": restored, "reason": getattr(error, "code", type(error).__name__)})
        raise


def _preserve_prepared_source(state, flow, proof, error):
    """Close a failed read-only preparation without undoing checkout edits."""
    kind = (flow.get('sourceManifest') or {}).get('preparationKind')
    reference = flow.get('sourcePreparationRef')
    operation = state.get_operation(reference) if isinstance(reference, str) else None
    if kind not in {'current-source', 'already-applied-patch'} or operation is None:
        raise JenkinsError('source_preparation_untrusted', 'Failure recovery requires its typed source preparation')
    intent, result = operation['payload'], operation.get('result') or {}
    bound_kind = intent.get('preparationKind', intent.get('kind'))
    bound_identity = intent.get('identity') or {}
    if (operation['kind'] != 'source_preparation' or operation['status'] != 'complete'
            or bound_kind != kind or not bound_identity
            or any(flow['identity'].get(key) != value for key, value in bound_identity.items())
            or intent.get('repositoryRoot', '').casefold() != str(flow['repoRoot']).casefold()
            or result.get('sourceDigest') != flow.get('sourceDigest')
            or result.get('canonicalSourceModified') is not False
            or sorted(result.get('ownedPaths', [])) != sorted(flow.get('ownedPaths', []))):
        raise JenkinsError('source_preparation_mismatch', 'Recovery differs from the completed read-only preparation')
    return {'status': 'complete', 'kind': 'coordinator-preparation-preserved',
            'preparationOperationId': reference, 'operations': [],
            'canonicalSourceModified': False, 'sourcePreserved': True,
            'terminationEvidence': proof, 'origin': error}


def recover_flow(state, repo_root, key, *, error=None):
    record = state.get("workflow", key)
    if record is None:
        raise JenkinsError("workflow_not_found", "recovery requires the registered workflow")
    flow = record["payload"]
    if flow.get("status") == "accepted" or flow.get("gitCommitSha"):
        return {"status": "complete", "workflowStatus": "accepted", "compensation": "accepted-source-preserved"}
    if flow.get("recovery", {}).get("status") == "complete":
        return {"status": "complete", "workflowStatus": flow["status"], "recovery": flow["recovery"]}
    eid = flow.get("executionId")
    execution = state.get("execution", eid) if eid else None
    if execution:
        from .execution import ExecutionEngine
        engine = ExecutionEngine(state, repo_root=repo_root, build_root=flow["buildRoot"])
        if execution["payload"].get("status") not in {"passed", "failed", "cancelled"}:
            from ..processes.boot import recover_previous_boot
            try:
                recover_previous_boot(state, eid)
            except JenkinsError as failure:
                if failure.code not in {"previous_boot_unproven", "boot_recovery_identity_missing", "boot_evidence_unavailable"}:
                    raise
        consumer = state.get("execution_consumer", f"{eid}:{key}")
        try:
            if consumer and consumer["payload"].get("status") == "active":
                engine.cancel(eid, key)
            engine.observe(eid)
        except JenkinsError as failure:
            if failure.code != "execution_identity_corrupt":
                raise
            # Native boot evidence cannot supply a missing execution key.
            # Quarantine the workflow without compensating source or holds.
            recovery = {"status": "blocked", "reasonCode": failure.code, "origin": error}
            current = state.get("workflow", key)
            state.put("workflow", key, {**current["payload"], "recovery": recovery},
                      expected_version=current["version"])
            return {"status": "blocked", "workflowStatus": current["payload"]["status"],
                    "recovery": recovery}
    references = flow.get("patchOperationRefs") or []
    if not references:
        sealed = state.get("sealed_input", flow.get("sourceDigest"))
        references = sealed["payload"].get("patchOperationRefs", []) if sealed else []
    owns_patch = any((state.get_operation(ref) or {}).get("payload", {}).get("owner") == flow["identity"]["attemptId"] for ref in references)
    if flow.get("consumerCancelled") and not owns_patch:
        recovery = {"status": "complete", "kind": "consumer-reference-released", "sourcePreserved": True,
                    "canonicalSourceModified": False, "nativeTerminationAccepted": False}
        current = state.get("workflow", key)
        state.put("workflow", key, {**current["payload"], "status": "cancelled", "recovery": recovery}, expected_version=current["version"])
        return {"status": "complete", "workflowStatus": "cancelled", "recovery": recovery}
    try:
        proof = _termination(state, flow)
        if not references and flow.get('sourcePreparationRef'):
            recovery = _preserve_prepared_source(state, flow, proof, error)
        elif not references:
            raise JenkinsError("patch_journal_missing", "failure recovery has no owned patch journal")
        else:
            for other in state.list("workflow"):
                if other["key"] != key and other["payload"].get("status") == "accepted" and set(references) & set(other["payload"].get("patchOperationRefs", [])):
                    raise JenkinsError("shared_source_accepted", "an accepted consumer still owns the shared source")
            base = (flow.get("sourceManifest") or {}).get("baseHead")
            if base:
                head = subprocess.run(["git", "-C", flow["repoRoot"], "rev-parse", "HEAD"], capture_output=True, text=True)
                if head.returncode or head.stdout.strip() != base:
                    raise JenkinsError("foreign_head_change", "Git HEAD changed before failure compensation")
            results = []
            for ref in reversed(references):
                operation = state.get_operation(ref)
                if operation is None or operation["kind"] != "source_patch":
                    raise JenkinsError("patch_journal_missing", "failure recovery patch journal is unavailable")
                results.append(_compensate_patch(state, operation, flow, proof)["operationId"])
            recovery = {"status": "complete", "operations": results, "terminationEvidence": proof, "origin": error}
    except JenkinsError as failure:
        recovery = {"status": "waiting" if failure.retryable else "blocked", "reasonCode": failure.code, "origin": error}
    current = state.get("workflow", key)
    stages = [dict(s) for s in current["payload"].get("stages", [])]
    for stage in stages:
        if stage.get("status") not in {"passed", "reused", "failed"}:
            stage["status"] = "blocked"
    status = "cancelled" if flow.get("status") == "cancelled" else "failed"
    state.put("workflow", key, {**current["payload"], "status": status, "stages": stages, "recovery": recovery}, expected_version=current["version"])
    identity = flow["identity"]
    request = state.get_request(flow["repositoryId"], identity["sessionId"], identity["requestId"])
    if request and request["status"] not in {"accepted", "failed", "cancelled"}:
        state.transition_request(flow["repositoryId"], identity["sessionId"], identity["requestId"], int(identity["generation"]), request["status"], status, {"recovery": recovery})
    return {"status": "complete" if recovery["status"] == "complete" else "reconciling" if recovery["status"] == "waiting" else "blocked", "workflowStatus": status, "recovery": recovery}
