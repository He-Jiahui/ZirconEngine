"""Merge disjoint authored inputs and require their actual joint acceptance."""
from __future__ import annotations

from pathlib import Path
from collections.abc import Mapping

from ..contracts import JenkinsError, canonical_json, digest, file_digest
from ..source import (compute_source_digest, register_sealed_consumer, canonical_path,
                      _atomic_write)
from ..resources import canonical_build_root, physical_path_under


def merge_joint_inputs(state, items):
    records = [state.get("sealed_input", i["sourceDigest"])["payload"] for i in items]
    roots = {str(p.get("repositoryRoot") or p.get("repoRoot")) for p in records}
    object_roots = {str(p.get("objectRoot")) for p in records}
    if len(roots) != 1 or len(object_roots) != 1 or "None" in roots | object_roots:
        raise JenkinsError("joint_storage_conflict", "joint sources must share one proven checkout and object root")
    ownership = {p: i["sessionId"] for i in items for p in i["paths"]}
    selected = {}
    owned_entries = {}
    required_tests = set()
    dependencies = set()
    external = {}
    for item, record in zip(items, records):
        manifest = record.get("sourceManifest") or record["manifest"]
        required_tests.update(item["coverage"].get("requiredTests", []))
        dependencies.update(manifest.get("declaredDependencies", []))
        for entry in manifest["entries"]:
            path = canonical_path(next(iter(roots)), entry["path"])
            if path in item["paths"]:
                owned_entries[path] = dict(entry)
            selected.setdefault(path, []).append(dict(entry))
        for value in manifest.get("external", []):
            external[digest(value)] = value
    entries = []
    for path, candidates in sorted(selected.items()):
        if path in ownership:
            if path not in owned_entries:
                raise JenkinsError("joint_owned_source_missing", "author's input lacks its owned source path")
            entry = owned_entries[path]
        else:
            if len({digest(c) for c in candidates}) != 1:
                raise JenkinsError("joint_dependency_conflict", "joint snapshots disagree on an unowned dependency", details={"path": path})
            entry = candidates[0]
        entries.append(entry)
    # Preserve source spelling (Cargo.toml) while using the same canonical
    # ordering as source sealing and preparation materialization.
    entries.sort(key=lambda entry: entry["path"])
    manifest = {"schemaVersion": 1, "baseHead": items[0]["baseHead"], "entries": entries,
                "external": [external[k] for k in sorted(external)], "declaredDependencies": sorted(dependencies)}
    source = compute_source_digest(manifest)
    coverage = {"template": "cross_module", "sourceDigest": source,
                "ownedPaths": sorted(ownership), "requiredTests": sorted(required_tests),
                "features": list(items[0]["coverage"].get("features", [])),
                "authorInputs": [{"sessionId": i["sessionId"], "sourceDigest": i["sourceDigest"],
                                  "coverageDigest": i["coverageDigest"], "paths": i["paths"]} for i in items]}
    coverage["coverageDigest"] = digest(coverage)
    payload = {"status": "merged", "sourceDigest": source, "sourceManifest": manifest,
               "coverage": coverage, "coverageDigest": coverage["coverageDigest"],
               "repositoryId": items[0]["repositoryId"], "repositoryRoot": next(iter(roots)),
               "objectRoot": next(iter(object_roots)), "buildRoot": items[0]["buildRoot"],
               "ownedPaths": sorted(ownership), "authors": coverage["authorInputs"],
               "patchOperationRefs": list(dict.fromkeys(ref for p in records for ref in p.get("patchOperationRefs", [])))}
    old = state.get("joint_input", source)
    if old and old["payload"] != payload:
        raise JenkinsError("joint_input_conflict", "joint input cannot replace its authored manifest")
    if not old:
        state.put("joint_input", source, payload, expected_version=0)
    return payload


def verify_joint_acceptance(state, joint, reference):
    if not reference:
        return False
    receipt_id = reference.get("receiptRef")
    receipt = state.get("acceptance", receipt_id) if isinstance(receipt_id, str) else None
    if not receipt:
        raise JenkinsError("joint_acceptance_untrusted", "joint acceptance must reference the registered collector receipt")
    payload = receipt["payload"]
    if payload.get("status") != "accepted" or payload.get("sourceDigest") != joint["sourceDigest"] or payload.get("coverageDigest") != joint["coverageDigest"]:
        raise JenkinsError("joint_acceptance_mismatch", "joint receipt must validate the complete merged source and coverage")
    workflow = state.get("workflow", payload.get("flowId", ""))
    if not workflow or workflow["payload"].get("status") != "accepted" or workflow["payload"].get("sourceDigest") != joint["sourceDigest"]:
        raise JenkinsError("joint_flow_unaccepted", "joint receipt requires its accepted workflow")
    return True


def seal_joint_input(state, group, *, session_id, owner):
    joint = state.get("joint_input", group.joint_manifest_digest)["payload"]
    repo = joint["repositoryId"]
    for author in joint["authors"]:
        state.check_authorization(repo, author["sessionId"], "validation", author["paths"])
    state.check_authorization(repo, session_id, "validation", joint["ownedPaths"])
    approved = canonical_build_root(joint["buildRoot"])
    object_root = physical_path_under(approved, joint["objectRoot"])
    for entry in joint["sourceManifest"]["entries"]:
        checksum = entry.get("objectDigest")
        if checksum:
            obj = physical_path_under(approved, object_root / "inputs" / "objects" / checksum)
            if not obj.is_file() or file_digest(obj) != checksum:
                raise JenkinsError("joint_object_corrupt", "joint source object is unavailable or corrupt")
    manifest_path = object_root / "inputs" / (joint["sourceDigest"] + ".json")
    if manifest_path.exists() and file_digest(manifest_path) != joint["sourceDigest"]:
        raise JenkinsError("joint_manifest_corrupt", "joint manifest bytes differ from the canonical source identity")
    if not manifest_path.exists():
        _atomic_write(manifest_path, canonical_json(joint["sourceManifest"]))
    payload = {**joint, "status": "sealed", "manifest": joint["sourceManifest"],
               "owner": owner, "sessionId": session_id, "claims": [], "jointGroupId": group.group_id}
    old = state.get("sealed_input", joint["sourceDigest"])
    if old and old["payload"] != payload:
        raise JenkinsError("joint_seal_conflict", "joint source sealing is immutable")
    if not old:
        state.put("sealed_input", joint["sourceDigest"], payload, expected_version=0)
    register_sealed_consumer(state, repository_id_value=repo, session_id=session_id,
                            owner=owner, source_digest=joint["sourceDigest"], manifest_digest=joint["sourceDigest"])
    return payload
