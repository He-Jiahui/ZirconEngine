"""Authoritative maintenance inventory and bounded artifact garbage collection."""
from __future__ import annotations
import time
from pathlib import Path
from typing import Any
from .artifacts import ArtifactStore
from .contracts import JenkinsError, digest, file_digest, response
from .resources.paths import canonical_build_root, build_namespace, physical_path_under
from .state import State

def _identity(payload: dict) -> tuple[str, str]:
    identity = payload.get("identity") or payload
    repo, session = identity.get("repositoryId"), identity.get("sessionId")
    if not isinstance(repo, str) or not isinstance(session, str):
        raise JenkinsError("maintenance_identity_missing", "maintenance requires repository and session identity")
    return repo, session

def _authorize(payload: dict, state: State) -> tuple[str, str]:
    repo, session = _identity(payload)
    state.check_authorization(repo, session, "gc", payload.get("ownedPaths", []))
    return repo, session

def _store(payload: dict) -> tuple[Path, ArtifactStore]:
    root = canonical_build_root(payload.get("buildRoot"))
    return root.path, ArtifactStore(build_namespace(root, "artifacts"))

def _active_refs(state: State) -> set[str]:
    return {str(r["payload"].get("digest")) for r in state.list("artifact_ref") if r["payload"].get("status") == "active" and isinstance(r["payload"].get("digest"), str)}

def _historical_digests(state: State) -> set[str]:
    protected: set[str] = set()
    # Inventory, object and GC records contain the objects being evaluated;
    # they are audit evidence rather than live retention references.  Durable
    # acceptance/execution records remain protected.
    domains = ("acceptance", "request_result", "execution", "workflow", "receipt", "native_job")
    def walk(value: Any) -> None:
        if isinstance(value, dict):
            for key, item in value.items():
                if isinstance(item, str) and (key.lower().endswith("digest") or key in {"digest", "artifact"}) and len(item) == 64: protected.add(item)
                walk(item)
        elif isinstance(value, (list, tuple)):
            for item in value: walk(item)
    for domain in domains:
        for row in state.list(domain): walk(row["payload"])
    return protected

def inventory(payload: dict, state: State, repo_root: Path) -> dict:
    root, store = _store(payload); files, total = [], 0
    if store.objects.exists():
        for path in sorted(store.objects.iterdir(), key=lambda p: p.name):
            if path.is_file():
                size = path.stat().st_size; total += size; files.append({"digest": path.name, "size": size})
    registered = {r["key"] for r in state.list("artifact_object")}; active = _active_refs(state)
    unknown = sorted(item["digest"] for item in files if item["digest"] not in registered)
    protected = sorted(active | _historical_digests(state))
    op = str(payload.get("operationId") or digest({"kind": "inventory", "root": str(root), "files": files})[:32])
    result = {"operationId": op, "buildRoot": str(root), "objectCount": len(files), "totalBytes": total, "activeDigests": sorted(active), "unknownDigests": unknown, "protectedDigests": protected}
    existing = state.get("maintenance_inventory", op)
    if existing:
        # Inventory operations are immutable snapshots.  Replays return the
        # original evidence rather than changing its timestamp or digest.
        return existing["payload"]
    result["observedAt"] = time.time()
    state.put("maintenance_inventory", op, {**result, "inventoryDigest": digest(result)})
    return result

def garbage_collect(payload: dict, state: State, repo_root: Path) -> dict:
    repo, session = _authorize(payload, state); root, store = _store(payload)
    op = str(payload.get("operationId") or digest({"kind": "gc", "root": str(root), "session": session})[:32])
    old = state.get("maintenance_gc", op)
    if old: return old["payload"]
    scope = payload.get("cleanupScope")
    if not isinstance(scope, dict): raise JenkinsError("cleanup_scope_required", "GC requires an explicit cleanup scope")
    digests = scope.get("digests")
    max_bytes, max_objects = scope.get("maxBytes"), scope.get("maxObjects")
    if not isinstance(digests, list) or any(not isinstance(d, str) or len(d) != 64 or any(c not in "0123456789abcdef" for c in d.lower()) for d in digests):
        raise JenkinsError("cleanup_scope_invalid", "cleanupScope.digests must contain SHA-256 digests")
    if not isinstance(max_bytes, int) or max_bytes <= 0 or not isinstance(max_objects, int) or max_objects <= 0:
        raise JenkinsError("cleanup_scope_invalid", "cleanupScope limits must be positive integers")
    requested = set(digests)
    active, historical = _active_refs(state), _historical_digests(state); candidates = []
    planned_bytes = 0
    with state.transaction() as conn:
        for row in state.list("artifact_object", connection=conn):
            d, status, path = row["key"], row["payload"].get("status"), store.objects / row["key"]
            if d not in requested or status == "deleted" or d in active or d in historical or status not in {"ready", "deleting"} or not path.is_file(): continue
            size = path.stat().st_size
            if len(candidates) >= max_objects or planned_bytes + size > max_bytes:
                continue
            if file_digest(path) != d:
                raise JenkinsError("artifact_digest_mismatch", "registered artifact bytes changed before GC")
            if status == "deleting" and row["payload"].get("gcOperationId") != op:
                raise JenkinsError("artifact_gc_pending", "artifact has an unresolved deletion intent", retryable=True)
            if status == "ready": state.put("artifact_object", d, {**row["payload"], "status": "deleting", "gcOperationId": op}, expected_version=row["version"], connection=conn)
            candidates.append((d, size))
            planned_bytes += size
    removed = []
    used_bytes = 0
    for d, size in candidates:
        if len(removed) >= max_objects or used_bytes + size > max_bytes: break
        if d in active or d in _active_refs(state): continue
        # Revalidate the physical namespace immediately before unlinking.
        approved = canonical_build_root(payload.get("buildRoot"))
        physical_path_under(approved, str(store.objects / d), allow_missing=False)
        if file_digest(store.objects / d) != d:
            raise JenkinsError("artifact_digest_mismatch", "registered artifact bytes changed during GC")
        try: (store.objects / d).unlink()
        except FileNotFoundError: continue
        except OSError as exc: raise JenkinsError("artifact_gc_failed", f"cannot remove registered artifact {d}") from exc
        removed.append({"digest": d, "bytes": size})
        used_bytes += size
    with state.transaction() as conn:
        for item in removed:
            row = state.get("artifact_object", item["digest"], connection=conn)
            if row and not any(r["payload"].get("digest") == item["digest"] and r["payload"].get("status") == "active" for r in state.list("artifact_ref", connection=conn)):
                state.put("artifact_object", item["digest"], {**row["payload"], "status": "deleted", "deletedAt": time.time()}, expected_version=row["version"], connection=conn)
    result = {"operationId": op, "repositoryId": repo, "sessionId": session, "buildRoot": str(root), "removed": removed, "reclaimedBytes": sum(item["bytes"] for item in removed), "protectedDigests": sorted(active | historical), "unknownProtected": True, "completedAt": time.time()}
    state.put("maintenance_gc", op, {**result, "resultDigest": digest(result)})
    return result

def handle(action: str, payload: dict, state: State, repo_root: Path, *, domain: str = "maintenance") -> dict:
    if action in {"inventory", "inventory-maintenance"}: return response("accepted", operation_id=payload.get("operationId"), result=inventory(payload, state, repo_root))
    if action in {"gc", "gc-maintenance", "cache-maintenance"}: return response("accepted", operation_id=payload.get("operationId"), result=garbage_collect(payload, state, repo_root))
    raise JenkinsError("unsupported_maintenance_action", action)
