"""Bounded cleanup; unproved candidates remain needs-review."""
from __future__ import annotations

import json
from pathlib import Path
import shutil

from .paths import digest, file_fact, overlaps, physical


def _proof(authority, payload, target):
    registration = authority.get("repository", payload["repositoryId"])
    canonical = physical(registration["canonicalCheckout"])
    if overlaps(target, canonical):
        raise ValueError("canonical_source_checkout_protected")
    if payload["kind"] == "build-output":
        if target == canonical or target in canonical.parents:
            raise ValueError("canonical_source_checkout_protected")
        # A State-backed generated-output receipt is required, not an asserted flag or directory name.
        row = authority.state.get("generated_output", payload["outputRecordId"])
        if not row or digest(row["payload"]) != payload["outputRecordDigest"]:
            raise ValueError("generated_output_provenance_required")
        evidence = row["payload"]
        if evidence.get("repositoryId") != payload["repositoryId"] or evidence.get("regenerable") is not True:
            raise ValueError("output_repository_or_regenerability_unproved")
        if physical(evidence["root"]) != target or evidence.get("retain") is not False:
            raise ValueError("output_retained_or_scope_changed")
        protected = [".git", ".jenkins", "Cargo.toml", "CMakeLists.txt", "package.json", "pyproject.toml"]
        if any((target / name).exists() for name in protected):
            raise ValueError("source_or_control_candidate_protected")
    elif payload["kind"] in {"worktree", "source-copy"}:
        sealed = authority.get("sealed", payload["sealedOperationId"])
        if sealed["repositoryId"] != payload["repositoryId"] or physical(sealed["sourceCheckout"]) != target:
            raise ValueError("complete_source_preservation_required")
        from .inventory import repository_fact, text_git
        observation = repository_fact(target)
        if observation.get("errors") or observation.get("head") != sealed["baseHead"]:
            raise ValueError("source_checkout_changed")
        # Retain the base/detached commit as well as both index/worktree commits in the canonical object store.
        for commit in (sealed["baseHead"], sealed["indexCommit"], sealed["commit"]):
            if text_git(canonical, "cat-file", "-t", commit) != "commit":
                raise ValueError("source_objects_not_recoverable_in_canonical_repository")
            refs = text_git(canonical, "for-each-ref", "--contains=" + commit, "--format=%(refname)")
            if not refs:
                raise ValueError("detached_or_unique_source_commit_requires_retained_ref")
        for row in authority.state.list("governance_claim"):
            if row["payload"].get("sourceCheckout") == str(target) and row["payload"]["status"] == "claimed":
                raise ValueError("active_source_retention_reference")
        if payload.get("releasedRetentionRefs") != []:
            raise ValueError("retention_release_requires_authoritative_owner_transition")
        changed = {item["path"] for item in observation.get("changes", [])}
        if not changed <= set(sealed["paths"]):
            raise ValueError("unsealed_source_changes_protected")
    else:
        raise ValueError("unsupported_cleanup_kind")
    jobs = payload.get("writerJobIds", [])
    if not jobs:
        raise ValueError("exact_writer_termination_proof_required")
    proofs = []
    for key in jobs:
        row = authority.state.get("native_job", key)
        value = row["payload"] if row else {}
        if (value.get("status") != "terminal" or value.get("completeProof", {}).get("complete") is not True
                or value.get("repositoryId") != payload["repositoryId"]):
            raise ValueError("writer_job_original_attempt_not_terminal")
        # Match State's original output/write scope; a terminal unrelated job proves nothing.
        scopes = value.get("writeScopes") or value.get("outputRoots") or []
        if not any(overlaps(scope, target) for scope in scopes):
            raise ValueError("writer_proof_does_not_cover_cleanup_scope")
        proofs.append({"key": key, "digest": digest(value), "requestId": value.get("requestId"),
                       "attemptId": value.get("attemptId"), "driverDigest": value.get("driverDigest")})
    return registration, proofs


def plan(authority, payload, report):
    authority.require_closed(payload)
    target = physical(payload["target"])
    registration, proofs = _proof(authority, payload, target)
    selected = payload["files"]
    if not isinstance(selected, list) or not selected or len(selected) > 100000:
        raise ValueError("bounded_cleanup_files_required")
    facts, total, parent_identities, source_recovery = [], 0, {}, []
    protected = [physical(path) for path in payload.get("protectedPaths", [])]
    for relative in selected:
        from .paths import relative_source
        path = physical(target / relative_source(relative))
        from .inventory import authentication_path
        if authentication_path(path):
            raise ValueError("authentication_content_protected_without_reading")
        if any(overlaps(path, root) for root in protected):
            raise ValueError("protected_cleanup_path")
        fact = file_fact(path)
        if fact["identity"][-1] != 1:
            raise ValueError("hardlinked_cleanup_candidate_requires_review")
        if payload.get("expectedHashes", {}).get(relative) != fact["sha256"]:
            raise ValueError("cleanup_input_hash_changed")
        if payload["kind"] in {"worktree", "source-copy"}:
            # Ignored files do not appear in Git status. A clean status or a
            # retained base commit therefore cannot prove their recoverability.
            from .inventory import git
            sealed = authority.get("sealed", payload["sealedOperationId"])
            canonical = physical(registration["canonicalCheckout"])
            entry = git(canonical, "ls-tree", sealed["commit"], "--", relative).decode().strip()
            if not entry or fact["bytes"] > 100 * 1024 * 1024:
                raise ValueError("source_file_requires_exact_retained_Git_bytes")
            mode, kind, oid = entry.split("\t", 1)[0].split()
            if mode not in {"100644", "100755"} or kind != "blob":
                raise ValueError("source_file_requires_exact_retained_Git_bytes")
            import hashlib
            if hashlib.sha256(git(canonical, "cat-file", "blob", oid)).hexdigest() != fact["sha256"]:
                raise ValueError("source_file_requires_exact_retained_Git_bytes")
            source_recovery.append({"commit": sealed["commit"], "blob": oid, "path": relative, "sha256": fact["sha256"]})
        facts.append(fact)
        for parent in path.parents:
            info = physical(parent).lstat()
            parent_identities[str(parent)] = [info.st_dev, info.st_ino]
        total += fact["bytes"]
    if total > payload.get("maxLogicalBytes", 0):
        raise ValueError("cleanup_byte_budget_exceeded")
    value = {"status": "planned", "repositoryId": payload["repositoryId"], "roundId": payload["roundId"],
             "inventoryDigest": payload["inventoryDigest"], "target": str(target), "kind": payload["kind"],
             "files": facts, "parentIdentities": parent_identities, "writerProofs": proofs, "logicalBytes": total, "originalInput": payload,
             "sourceRecovery": source_recovery,
             "protectedPaths": [file_fact(path) for path in protected if path.is_file()], **authority.runtime}
    value["planDigest"] = digest(value)
    from .worker import _json
    _json(report / "cleanup-plan.json", value)
    authority.put("cleanup", payload["operationId"], value, expected_version=0)
    return {"status": "planned", "planId": payload["operationId"], "planDigest": value["planDigest"],
            "files": len(facts), "logicalBytes": total}


def apply(authority, payload, report):
    authority.require_closed(payload)
    value = authority.get("cleanup", payload["planId"])
    if value["planDigest"] != payload["planDigest"] or value["roundId"] != payload["roundId"]:
        raise ValueError("cleanup_original_plan_changed")
    target = physical(value["target"])
    _, proofs = _proof(authority, value["originalInput"], target)
    if proofs != value["writerProofs"]:
        raise ValueError("cleanup_original_writer_evidence_changed")
    from .guarded_delete import remove_verified
    # Preflight the entire bounded set; a changed member prevents starting this execution.
    for fact in value["files"] + value["protectedPaths"]:
        if file_fact(Path(fact["path"])) != fact:
            raise ValueError("cleanup_physical_identity_or_content_changed")
        remove_verified(fact, remove=False, expected_directory_identities=value["parentIdentities"])
    before = shutil.disk_usage(target.anchor).free
    deleted, blocked = [], []
    for fact in value["files"]:
        key = digest({"planId": payload["planId"], "path": fact["path"]})
        # Persist intent before deleting so an interruption cannot turn absence into success.
        authority.put("deletion", key, {"status": "deleting", "fact": fact, "planId": payload["planId"], **authority.runtime}, expected_version=0)
        try:
            receipt = remove_verified(fact, expected_directory_identities=value["parentIdentities"])
            authority.put("deletion", key, {**receipt, "planId": payload["planId"], **authority.runtime})
            deleted.append(receipt)
        except (OSError, ValueError) as error:
            blocked.append({"path": fact["path"], "reason": str(error)[:200]})
            break
    after = shutil.disk_usage(target.anchor).free
    result = {"status": "complete" if not blocked else "needs-review", "planId": payload["planId"],
        "deleted": deleted, "blocked": blocked, "logicalDeletedBytes": sum(item["logicalBytes"] for item in deleted),
        "deletedAllocationBytes": sum(item["allocationBytes"] for item in deleted),
        "observedFreeSpaceDelta": after - before, "actualFreedBytes": None,
        "spaceMeasurement": "Concurrent volume writes prevent attribution of the observed free-space delta.", **authority.runtime}
    from .worker import _json
    _json(report / "cleanup-receipt.json", result)
    authority.put("cleanup_execution", payload["operationId"], result, expected_version=0)
    return result


def reconcile(authority, payload, report):
    value = authority.get("cleanup", payload["planId"])
    rows = []
    for fact in value["files"]:
        key = digest({"planId": payload["planId"], "path": fact["path"]})
        receipt = authority.state.get("governance_deletion", key)
        exists = Path(fact["path"]).exists()
        prior = receipt["payload"] if receipt else {}
        status = "deleted" if prior.get("status") == "deleted" and not exists else "needs-review"
        if not receipt and exists and file_fact(Path(fact["path"])) == fact:
            status = "not-started"
        rows.append({"path": fact["path"], "status": status, "originalReceipt": prior})
    return {"status": "complete" if all(row["status"] == "deleted" for row in rows) else "needs-review",
            "originalPlanId": payload["planId"], "files": rows, "absenceAloneProvesDeletion": False}
