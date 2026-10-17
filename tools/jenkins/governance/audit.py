"""State-backed inventory review and bounded admission/output audits."""
from __future__ import annotations

import json
import re
from pathlib import Path
import time

from .inventory import repository_fact, text_git
from .paths import build_namespace, digest, file_fact, overlaps, physical


def admission(authority, request):
    repository_id = request.get("payload", {}).get("repositoryId")
    if not repository_id:
        return {"status": "checked", "scope": "control-report-root", "repository": None}
    registration = authority.get("repository", repository_id)
    root = physical(registration["canonicalCheckout"])
    namespace = build_namespace(registration["buildNamespace"])
    common = physical(registration["commonDir"])
    if overlaps(namespace, root) or overlaps(namespace, common):
        raise ValueError("source_and_compiler_output_roots_overlap")
    return {"status": "checked", "repository": repository_id, "canonicalCheckout": str(root),
            "commonDir": str(common), "buildNamespace": str(namespace),
            "registrationDigest": digest(registration), "scope": "registered-physical-roots",
            "machineWideWriterConfinement": False}


def _inventory(authority, inventory_id=None):
    rows = [row for row in authority.state.list("governance_inventory")
            if row["payload"].get("inventoryPolicy", "whole-volume-v1") == "whole-volume-v1"]
    if inventory_id:
        rows = [row for row in rows if row["key"] == inventory_id]
    if not rows:
        return None
    reference = max(rows, key=lambda row: row["updatedAt"])["payload"]
    path = physical(reference["inventoryRef"]["path"])
    if file_fact(path) != reference["inventoryRef"]:
        raise ValueError("audit_original_inventory_changed")
    value = json.loads(path.read_text(encoding="utf-8"))
    if digest({k: v for k, v in value.items() if k != "inventoryDigest"}) != reference["inventoryDigest"]:
        raise ValueError("audit_inventory_digest_changed")
    return reference, value


def _claim_proof(authority, claim, registration, current):
    reasons = []
    if claim.get("driverDigest") != authority.runtime["driverDigest"]:
        reasons.append("previous-driver-requires-adoption")
    facts = current.get("sourceFacts", {})
    for path, expected in claim.get("sourceHashes", claim.get("hashes", {})).items():
        observed = facts.get(path)
        if path not in facts:
            try:
                source = physical(Path(claim["sourceCheckout"]) / path, exists=False)
                observed = file_fact(source) if source.exists() else None
            except (ValueError, OSError):
                reasons.append("source-unreadable:" + path)
                continue
        if (observed or {}).get("sha256") != expected:
            reasons.append("source-input-changed:" + path)
    seals = [row["payload"] for row in authority.state.list("governance_sealed")
             if row["payload"]["repositoryId"] == claim["repositoryId"]
             and row["payload"]["ownerSessionId"] == claim["ownerSessionId"]
             and set(claim["paths"]) <= set(row["payload"]["paths"])]
    if not seals:
        reasons.append("retained-managed-source-seal-missing")
    for sealed in seals:
        try:
            operation = authority.state.get_operation(sealed["operationId"])
            if not operation or operation["status"] != "complete":
                reasons.append("original-takeover-operation-not-complete")
            root = physical(registration["canonicalCheckout"])
            head = text_git(root, "rev-parse", "--verify", sealed["branch"])
            if head != sealed["commit"]:
                patches = [row["payload"] for row in authority.state.list("governance_patch")
                           if row["payload"].get("branch") == sealed["branch"]
                           and row["payload"].get("commit") == head]
                if not patches:
                    reasons.append("managed-reference-changed-without-receipt")
        except (ValueError, OSError):
            reasons.append("managed-reference-unreadable")
    return {"status": "verified-managed" if not reasons else "needs-review", "reasons": sorted(set(reasons)),
            "claimId": claim["claimId"], "ownerSessionId": claim["ownerSessionId"],
            "paths": claim["paths"], "driverDigest": claim.get("driverDigest")}



def _managed_run(authority, payload, report):
    from .inventory import MANAGED_LIMITS, MANAGED_POLICY, MANAGED_ROOTS
    from .worker import _json, _managed_binding, _managed_index, _terminal
    inventory_id, expected = payload.get("inventoryId"), payload.get("inventoryDigest")
    if not inventory_id or not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected):
        raise ValueError("managed_audit_requires_exact_inventory_id_and_digest")
    record = authority.state.get("governance_inventory", inventory_id)
    if record is None:
        raise ValueError("managed_audit_original_inventory_missing")
    reference = record["payload"]
    if (reference.get("inventoryPolicy") != MANAGED_POLICY or reference.get("inventoryId") != inventory_id
            or reference.get("inventoryDigest") != expected or reference.get("binding") != _managed_binding(authority)):
        raise ValueError("managed_audit_inventory_identity_changed")
    inventory_path = physical(reference["inventoryRef"]["path"])
    index_path = physical(reference["managedIndexRef"]["path"])
    native_path = physical(reference["nativeReceipt"]["path"])
    if (inventory_path.name != "inventory.json" or inventory_path.parent.name != inventory_id
            or index_path != inventory_path.parent / "managed-index.json"
            or native_path != inventory_path.parent / "native-scan/native-receipt.json"):
        raise ValueError("managed_audit_control_paths_changed")
    for path, fact in ((inventory_path, reference["inventoryRef"]), (index_path, reference["managedIndexRef"]),
                       (native_path, reference["nativeReceipt"])):
        if file_fact(path) != fact:
            raise ValueError("managed_audit_retained_bytes_changed")
    inventory, index, receipt = [json.loads(path.read_text(encoding="utf-8"))
                                 for path in (inventory_path, index_path, native_path)]
    if (digest({k: v for k, v in inventory.items() if k != "inventoryDigest"}) != expected
            or digest({k: v for k, v in index.items() if k != "indexDigest"}) != index.get("indexDigest")
            or inventory.get("stateIndex") != index or index.get("binding") != reference["binding"]
            or inventory.get("inventoryPolicy") != MANAGED_POLICY
            or inventory.get("roots") != [str(Path(root)) for root in MANAGED_ROOTS]
            or inventory.get("limits") != MANAGED_LIMITS or inventory.get("globalInventoryComplete") is not False):
        raise ValueError("managed_audit_inventory_digest_changed")
    native_key = digest(str(native_path.parent))
    native = authority.state.get("governance_native", native_key)
    if (reference.get("nativeRecordKey") != native_key or native is None
            or native["payload"].get("receipt") != reference["nativeReceipt"]
            or native["payload"].get("operationId") != inventory_id
            or native["payload"].get("driverDigest") != reference["binding"]["driverDigest"]
            or native["payload"].get("capsuleDigest") != reference["binding"]["capsuleDigest"]
            or not _terminal(native["payload"]) or not _terminal(receipt)
            or native["payload"].get("rootExitCode") != 0 or receipt.get("rootExitCode") != 0):
        raise ValueError("managed_audit_native_scan_not_terminal")
    fresh = _managed_index(authority)
    omissions = [*inventory["errors"], *fresh["omissions"]]
    for root in inventory["roots"]:
        physical_root = inventory["rootCoverage"][root].get("physicalRoot") or {}
        if (not physical_root.get("verified") or not physical_root.get("afterVerified")
                or not physical_root.get("volumeIdentity")):
            omissions.append({"path": root, "reason": "managed-root-identity-not-proved"})
    complete = inventory["status"] == fresh["status"] == "complete" and not omissions
    summary = {"repositories": 0, "legacyOwners": 0, "omissions": len(omissions),
               "protectedOwners": len(fresh["records"]), "protectedScopes": len(inventory["protected"]),
               "cleanupAuthorized": 0}
    value = {"status": "complete" if complete else "needs-review", "inventoryPolicy": MANAGED_POLICY,
        "inventoryId": inventory_id, "inventoryDigest": expected, "inventoryRef": reference["inventoryRef"],
        "coverage": {key: inventory[key] for key in ("roots", "rootCoverage", "limits", "status", "coverageScope")},
        "globalInventoryComplete": False, "missingInventory": False, "omissions": omissions,
        "legacyOwners": [], "protectedOwners": fresh["records"], "protected": inventory["protected"],
        "summary": summary, "observedAt": time.time(), "binding": fresh["binding"],
        "scopeAcceptance": "bounded-inventory-only",
        "retention": "All active, historical, unknown, source and referenced objects remain protected.",
        "cleanup": "Existing ownership/source/native/closed-claim proofs are still required; no new deletion authority."}
    _json(report / "inventory-review.json", value)
    result = {key: value[key] for key in ("status", "inventoryPolicy", "inventoryId", "inventoryDigest",
                                         "inventoryRef", "summary", "globalInventoryComplete", "scopeAcceptance")}
    result["review"] = file_fact(report / "inventory-review.json")
    authority.put("audit", payload["operationId"], result, expected_version=0)
    return result


def run(authority, payload, report):
    if payload.get("inventoryPolicy", "whole-volume-v1") == "managed-roots-v1":
        return _managed_run(authority, payload, report)
    if payload.get("inventoryPolicy", "whole-volume-v1") != "whole-volume-v1":
        raise ValueError("unsupported_inventory_policy")
    from .worker import _json
    selected = _inventory(authority, payload.get("inventoryId"))
    registrations = {row["payload"]["commonDir"].casefold(): row["payload"]
                     for row in authority.state.list("governance_repository")}
    value = {"status": "needs-review", "observedAt": time.time(), "projects": [], "outputCandidates": [],
             "sourceCandidates": [], "legacyOwners": [], "missingInventory": selected is None,
             "liveChats": "Historical chat metadata cannot establish current ownership.", **authority.runtime}
    if selected:
        reference, inventory = selected
        value.update(inventoryRef=reference["inventoryRef"], inventoryDigest=reference["inventoryDigest"],
                     coverage={k: inventory.get(k) for k in ("roots", "rootCoverage", "directories", "files", "logicalBytes", "status", "limitations")},
                     omissions=inventory.get("errors", []), links=inventory.get("links", []))
        for repo in inventory["repositories"]:
            registration = registrations.get(repo.get("commonDir", "").casefold())
            item = {"path": repo["path"], "commonDir": repo.get("commonDir"), "bare": repo.get("bare"),
                    "primaryBranch": repo.get("primaryBranch"), "primaryHead": repo.get("primaryHead"),
                    "rootCommits": repo.get("rootCommits", []), "branches": repo.get("branches", []),
                    "worktrees": repo.get("worktrees"), "unmanagedChanges": [], "claims": [],
                    "status": "unregistered" if not registration else "registered",
                    "repositoryErrors": repo.get("errors", [])}
            if registration:
                try:
                    current = repository_fact(physical(repo["path"]), registration, content_hashes=False)
                    claims = [row["payload"] for row in authority.state.list("governance_claim")
                              if row["payload"]["repositoryId"] == registration["repositoryId"]
                              and row["payload"]["status"] == "claimed"
                              and row["payload"]["sourceCheckout"] == repo["path"]]
                    item["claims"] = [_claim_proof(authority, claim, registration, current) for claim in claims]
                    managed = [path for proof in item["claims"] if proof["status"] == "verified-managed" for path in proof["paths"]]
                    item["unmanagedChanges"] = [change for change in current.get("changes", [])
                                                if not any(overlaps(change["path"], scope) for scope in managed)]
                    item["currentObservation"] = current
                except (ValueError, OSError) as error:
                    item["status"] = "needs-review"
                    item["repositoryErrors"].append(type(error).__name__)
            else:
                item["unmanagedChanges"] = repo.get("changes", [])
            if not item["primaryBranch"] or item["repositoryErrors"]:
                item["status"] = "needs-review"
            value["projects"].append(item)
        for candidate in inventory.get("outputCandidates", []):
            item = dict(candidate)
            try:
                build_namespace(item["path"])
                item["compilerRootStatus"] = "allowed-physical-root"
            except (ValueError, OSError):
                item["compilerRootStatus"] = "outside-root-or-unreadable"
            item["status"] = "needs-review"
            item["reason"] = "candidate-is-not-regenerability-or-current-pool-proof"
            value["outputCandidates"].append(item)
        value["sourceCandidates"] = inventory.get("sourceCandidates", [])
    for domain in ("management_worktree", "path_claims", "patch_owner"):
        for row in authority.state.list(domain):
            value["legacyOwners"].append({"domain": domain, "key": row["key"], "recordDigest": digest(row["payload"]),
                "status": "needs-review", "reason": "legacy-record-requires-source-reference-and-original-writer-reconciliation"})
    value["summary"] = {"repositories": len(value["projects"]),
        "unmanagedChanges": sum(len(item["unmanagedChanges"]) for item in value["projects"]),
        "unregisteredRepositories": sum(item["status"] == "unregistered" for item in value["projects"]),
        "outputCandidates": len(value["outputCandidates"]),
        "outsideCompilerRoots": sum(item["compilerRootStatus"] != "allowed-physical-root" for item in value["outputCandidates"]),
        "sourceCandidates": len(value["sourceCandidates"]), "legacyOwners": len(value["legacyOwners"]),
        "omissions": len(value.get("omissions", [])), "links": len(value.get("links", []))}
    _json(report / "inventory-review.json", value)
    text = ["# Pending centralized claims", "", "The claim round remains open until one explicit chat reply.", "",
            "| Checkout | Primary | Unmanaged paths | Registration |", "| --- | --- | ---: | --- |"]
    for item in value["projects"]:
        row = [item["path"], item["primaryBranch"] or "needs-review", str(len(item["unmanagedChanges"])), item["status"]]
        text.append("| " + " | ".join(cell.replace("|", "\\|").replace("\n", " ") for cell in row) + " |")
    text.extend(["", "Full file paths, hashes, branches, worktrees, candidates and omissions are in inventory-review.json.",
                 "No candidate is abandoned or deletable because an owner has not replied."])
    (report / "pending-claims.md").write_text("\n".join(text) + "\n", encoding="utf-8")
    result = {"status": "needs-review" if not selected else "awaiting-user-claims", "summary": value["summary"],
              "inventoryRef": value.get("inventoryRef"), "inventoryDigest": value.get("inventoryDigest"),
              "review": file_fact(report / "inventory-review.json"), "claimList": file_fact(report / "pending-claims.md"),
              "scope": "bounded-current-State-and-most-recent-whole-volume-inventory", "machineWideWriterConfinement": False}
    authority.put("audit", payload["operationId"], result, expected_version=0)
    return result
