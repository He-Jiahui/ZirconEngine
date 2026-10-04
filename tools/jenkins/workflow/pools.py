"""Durable preparation input pools.

The preparation pool is the only writer of a sealed source directory.  A
writer hold and its resource reservation are checked immediately before any
filesystem operation; every file is content addressed and every update is
journalled so an interrupted update can be recovered without guessing which
foreign files belong to Jenkins.
"""
from __future__ import annotations

import hashlib
import os
import re
import tempfile
import time
from pathlib import Path, PurePosixPath
from typing import Any, Mapping

from ..contracts import JenkinsError, canonical_json, file_digest, require_digest
from ..source import compute_source_digest
from ..resources import ApprovedBuildRoot, canonical_build_root, physical_path_under

_HEX = re.compile(r"[0-9a-f]{64}\Z")
_PRESERVED_DIRS = {"target", ".target", "cargo-target", "cargo-targets"}


def _error(code: str, message: str, **details: Any) -> JenkinsError:
    return JenkinsError(code, message, details=details or None)


def _root(value: ApprovedBuildRoot | str | os.PathLike[str]) -> ApprovedBuildRoot:
    # Re-run physical admission even for an object handed to us by a caller;
    # a junction can be introduced after the object was constructed.
    return canonical_build_root(value.path if isinstance(value, ApprovedBuildRoot) else value)


def _reparse(path: Path) -> bool:
    try:
        st = path.lstat()
    except FileNotFoundError:
        return False
    if path.is_symlink() or getattr(st, "st_reparse_tag", 0):
        return True
    # Windows exposes junctions through the reparse attribute.  Keep this
    # branch independent of the host Python's stat implementation.
    if os.name == "nt":
        try:
            import ctypes
            attrs = ctypes.windll.kernel32.GetFileAttributesW(str(path))
            return attrs != 0xFFFFFFFF and bool(attrs & 0x0400)
        except Exception:
            return True
    return False


def _check_tree(path: Path) -> None:
    """Reject reparse points in an existing preparation tree."""
    if path.exists() and _reparse(path):
        raise _error("preparation_reparse", "preparation path contains a reparse point", path=str(path))
    if not path.exists():
        return
    for item in path.rglob("*"):
        if _reparse(item):
            raise _error("preparation_reparse", "preparation path contains a reparse point", path=str(item))


def _relative(value: Any) -> str:
    if not isinstance(value, str) or not value or "\\" in value:
        raise _error("manifest_path_invalid", "manifest paths must be canonical relative POSIX paths")
    p = PurePosixPath(value)
    if p.is_absolute() or any(part in ("", ".", "..") for part in p.parts):
        raise _error("manifest_path_invalid", "manifest path escapes the preparation directory", path=value)
    return p.as_posix()


def _manifest(payload: Mapping[str, Any], object_root: Path, approved_root: ApprovedBuildRoot | None = None) -> tuple[dict[str, Any], dict[str, str]]:
    entries = payload.get("entries")
    if not isinstance(entries, list):
        raise _error("sealed_manifest_missing", "sealed input manifest entries are required")
    seen: set[str] = set()
    normalized: list[dict[str, Any]] = []
    owned: dict[str, str] = {}
    for raw in entries:
        if not isinstance(raw, Mapping):
            raise _error("manifest_entry_invalid", "sealed input entry is not an object")
        path = _relative(raw.get("path"))
        key = path.casefold()
        if key in seen:
            raise _error("manifest_path_collision", "manifest contains colliding paths", path=path)
        seen.add(key)
        status = raw.get("status", "present")
        item = dict(raw); item["path"] = path; item["status"] = status
        if status == "deleted":
            normalized.append(item)
            continue
        if status != "present":
            raise _error("manifest_entry_invalid", "unknown sealed input entry status", path=path)
        digest = raw.get("objectDigest") or raw.get("sha256")
        if not isinstance(digest, str) or not _HEX.fullmatch(digest):
            raise _error("manifest_object_invalid", "sealed input object digest is invalid", path=path)
        source = object_root / "inputs" / "objects" / digest
        if approved_root is not None:
            physical_path_under(approved_root, source, allow_missing=False)
        if not source.is_file() or _reparse(source) or file_digest(source) != digest:
            raise _error("input_object_missing", "sealed input object is missing or corrupt", path=path)
        item["objectDigest"] = digest
        normalized.append(item); owned[path] = digest
    result = dict(payload); result["entries"] = sorted(normalized, key=lambda e: e["path"])
    return result, owned


def _hash(path: Path) -> str | None:
    if not path.exists():
        return None
    if not path.is_file() or _reparse(path):
        raise _error("preparation_scope", "managed preparation path is not a regular file", path=str(path))
    return file_digest(path)


def _inventory(destination: Path, previous: Mapping[str, str] | None) -> dict[str, str]:
    actual: dict[str, str] = {}
    if not destination.exists():
        return actual
    _check_tree(destination)
    expected_paths = set(previous or {})
    expected_dirs: set[str] = set()
    for rel in expected_paths:
        parts = rel.split("/")
        for index in range(1, len(parts)):
            expected_dirs.add("/".join(parts[:index]))
    for item in destination.rglob("*"):
        if item.is_dir():
            rel_dir = item.relative_to(destination).as_posix()
            if rel_dir.split("/", 1)[0].casefold() in _PRESERVED_DIRS:
                continue
            # Empty or unregistered directories are evidence of an outside
            # writer just like an unregistered file.  Permit only parents of
            # files already owned by the preparation manifest.
            if previous is None or rel_dir not in expected_dirs:
                raise _error("preparation_foreign_file", "preparation contains an unregistered directory", path=rel_dir)
            continue
        if not item.is_file():
            continue
        rel = item.relative_to(destination).as_posix()
        top = rel.split("/", 1)[0].casefold()
        if top in _PRESERVED_DIRS:
            continue
        actual[rel] = file_digest(item)
    if previous is None:
        if actual:
            raise _error("preparation_foreign_file", "empty preparation pool contains unregistered files")
        return actual
    # Every non-cache file must still be one previously owned by this pool.
    for path, value in actual.items():
        if path not in previous:
            raise _error("preparation_foreign_file", "preparation contains an unregistered file", path=path)
        if value != previous[path]:
            raise _error("preparation_hash_mismatch", "owned preparation file changed outside the pool", path=path)
    for path, value in previous.items():
        observed = actual.get(path)
        if observed != value:
            raise _error("preparation_hash_mismatch", "owned preparation file is missing or changed", path=path)
    return actual


def _atomic_copy(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=f".{target.name}.", suffix=".pending", dir=str(target.parent))
    try:
        with os.fdopen(fd, "wb") as out, source.open("rb") as src:
            while True:
                block = src.read(1024 * 1024)
                if not block: break
                out.write(block)
            out.flush(); os.fsync(out.fileno())
        os.replace(tmp, target)
    finally:
        try: os.unlink(tmp)
        except FileNotFoundError: pass


def _hold(state, execution_id: str, preparation_key: str) -> dict[str, Any]:
    execution = state.get("execution", execution_id)
    if not execution:
        raise _error("execution_not_found", "execution is required for preparation materialization")
    p = execution["payload"]
    reservation_key = p.get("reservationKey")
    if not reservation_key:
        raise _error("pool_writer_unreserved", "execution has no active resource reservation")
    reservation = state.get("resource_reservation", reservation_key)
    hold = state.get("pool_writer_hold", preparation_key)
    if (not reservation or reservation["payload"].get("status") != "active"
            or reservation["payload"].get("owner") != execution_id):
        raise _error("pool_writer_unreserved", "preparation requires an active reservation owned by execution")
    hp = hold["payload"] if hold else {}
    if (not hold or hp.get("status") != "active" or hp.get("owner") != execution_id
            or hp.get("reservationKey") != reservation_key):
        raise _error("pool_writer_unreserved", "preparation writer hold is not owned by execution")
    return p


def _recover(state, preparation_key: str, destination: Path, object_root: Path | None = None) -> None:
    row = state.get("preparation_update", preparation_key)
    if not row:
        return
    if row["payload"].get("status") == "complete":
        intent = row["payload"]
        if not state.get("preparation_source", preparation_key) and intent.get("manifest") and intent.get("ownedFiles") is not None:
            _inventory(destination, intent.get("ownedFiles", {}))
            state.put("preparation_source", preparation_key, {"executionId": intent.get("executionId"),
                       "preparationKey": preparation_key, "sourceDigest": intent.get("sourceDigest"),
                       "manifest": intent["manifest"], "objectRoot": intent.get("objectRoot"),
                       "ownedFiles": intent["ownedFiles"], "physicalPath": str(destination), "updatedAt": time.time()})
        return
    if row["payload"].get("status") != "updating":
        return
    intent = row["payload"]
    old_owned = intent.get("oldOwnedFiles", {})
    files_by_path = {item.get("path"): item for item in intent.get("files", [])}
    # A crash may leave a new file in place before its ``applied`` bit reaches
    # SQLite.  Accept only old/new hashes for journaled paths; every other
    # file remains unowned evidence and blocks recovery.
    if destination.exists():
        _check_tree(destination)
        for candidate in destination.rglob("*"):
            if not candidate.is_file():
                continue
            rel = candidate.relative_to(destination).as_posix()
            if rel.split("/", 1)[0].casefold() in _PRESERVED_DIRS:
                continue
            item = files_by_path.get(rel)
            if item is None or _hash(candidate) not in (item.get("oldHash"), item.get("newHash")):
                raise _error("preparation_recovery_ambiguous", "interrupted update contains foreign or unknown data", path=rel)
    for item in intent.get("files", []):
        target = destination / item["path"]
        observed = _hash(target)
        old, new = item.get("oldHash"), item.get("newHash")
        if observed not in (old, new):
            raise _error("preparation_recovery_ambiguous", "interrupted update has an unrecognised file hash", path=item["path"])
        if observed == new:
            item["applied"] = True
        elif object_root is None:
            raise _error("preparation_recovery_incomplete", "interrupted update needs its sealed object root")
        elif new is None:
            # Only a file recorded in oldOwnedFiles may be deleted.
            if item["path"] not in old_owned:
                raise _error("preparation_recovery_scope", "journalled deletion is not owned", path=item["path"])
            if target.exists():
                target.unlink()
            item["applied"] = True
        else:
            _atomic_copy(object_root / "inputs" / "objects" / new, target)
            if _hash(target) != new:
                raise _error("preparation_recovery_hash", "recovered object hash does not match", path=item["path"])
            item["applied"] = True
        current = state.get("preparation_update", preparation_key)
        state.put("preparation_update", preparation_key, {**intent, "files": intent["files"]}, expected_version=current["version"])
    current = state.get("preparation_update", preparation_key)
    state.put("preparation_update", preparation_key, {**intent, "status": "complete", "files": intent["files"], "recoveredAt": time.time()}, expected_version=current["version"])
    if intent.get("manifest") and intent.get("ownedFiles") is not None:
        state.put("preparation_source", preparation_key, {"executionId": intent.get("executionId"),
                   "preparationKey": preparation_key, "sourceDigest": intent.get("sourceDigest"),
                   "manifest": intent["manifest"], "objectRoot": intent.get("objectRoot"),
                   "ownedFiles": intent["ownedFiles"], "physicalPath": str(destination), "updatedAt": time.time()})


def materialize_preparation(state, approved_build_root: ApprovedBuildRoot | str | os.PathLike[str], execution_id: str,
                            *, preparation_key: str | None = None, preparation_path: str | os.PathLike[str] | None = None) -> dict[str, Any]:
    """Materialize a sealed source under an already-held preparation writer.

    This function deliberately accepts no force/clean switch.  Unknown files,
    aliases, changed owned files and corrupt CAS objects stop the operation.
    """
    root = _root(approved_build_root)
    execution = state.get("execution", execution_id)
    if not execution:
        raise _error("execution_not_found", "execution is required")
    ep = execution["payload"]
    key = str(preparation_key or ep.get("preparationKey") or "")
    if not key:
        raise _error("preparation_key_missing", "preparation key is required")
    p = _hold(state, execution_id, key)
    raw_dest = preparation_path or ep.get("preparationRoot")
    if not raw_dest:
        raise _error("preparation_path_missing", "execution has no preparation path")
    destination = physical_path_under(root, raw_dest, allow_missing=True)
    _check_tree(destination)
    sealed_ref = ep.get("sealedInputRef") or ep.get("sourceDigest")
    sealed = state.get("sealed_input", str(sealed_ref)) if sealed_ref else None
    if not sealed:
        raise _error("sealed_input_missing", "sealed input is unavailable")
    sp = sealed["payload"]
    manifest = sp.get("sourceManifest") or sp.get("manifest")
    object_raw = sp.get("objectRoot")
    if not isinstance(manifest, Mapping) or not isinstance(object_raw, str):
        raise _error("sealed_manifest_missing", "sealed input manifest and object root are required")
    object_root = physical_path_under(root, object_raw, allow_missing=False)
    # The object store itself must be the exact namespace selected by the
    # approved root; a nested or aliased object store could otherwise inject
    # bytes under a valid-looking manifest.
    expected_namespace = physical_path_under(root, root.namespace(), allow_missing=False)
    if os.path.normcase(str(object_root)) != os.path.normcase(str(expected_namespace)):
        raise _error("object_root_rejected", "sealed object root is not the approved build namespace")
    normalized, desired = _manifest(manifest, object_root, root)
    source_digest = str(sp.get("sourceDigest") or sealed_ref)
    require_digest(source_digest, "sourceDigest")
    if compute_source_digest(normalized) != source_digest:
        raise _error("sealed_manifest_digest_mismatch", "sealed manifest does not match source identity")
    _recover(state, key, destination, object_root)
    previous_row = state.get("preparation_source", key)
    previous = previous_row["payload"] if previous_row else None
    previous_owned = previous.get("ownedFiles", {}) if previous else None
    actual = _inventory(destination, previous_owned)
    old_source = previous.get("sourceDigest") if previous else None
    if old_source == source_digest:
        if actual != desired:
            raise _error("preparation_hash_mismatch", "same sealed source does not match materialized files")
        return {"physicalPath": str(destination), "reused": True, "incremental": False, "sourceDigest": source_digest}
    files: list[dict[str, Any]] = []
    old = previous_owned or {}
    for path in sorted(set(old) | set(desired)):
        files.append({"path": path, "oldHash": old.get(path), "newHash": desired.get(path), "applied": False})
    intent = {"executionId": execution_id, "preparationKey": key, "sourceDigest": source_digest,
              "oldSourceDigest": old_source, "status": "updating", "files": files,
              "oldOwnedFiles": old, "ownedFiles": desired, "manifest": normalized,
              "objectRoot": str(object_root),
              "manifestDigest": hashlib.sha256(canonical_json(normalized)).hexdigest(), "createdAt": time.time()}
    state.put("preparation_update", key, intent, expected_version=None)
    for item in files:
        target = destination / item["path"]
        if item["newHash"] is None:
            if target.exists(): target.unlink()
        else:
            _atomic_copy(object_root / "inputs" / "objects" / item["newHash"], target)
        item["applied"] = True
        row = state.get("preparation_update", key)
        state.put("preparation_update", key, {**row["payload"], "files": files}, expected_version=row["version"])
    state.put("preparation_update", key, {**intent, "status": "complete", "files": files, "completedAt": time.time()}, expected_version=state.get("preparation_update", key)["version"])
    state.put("preparation_source", key, {"executionId": execution_id, "preparationKey": key,
               "sourceDigest": source_digest, "manifest": normalized, "objectRoot": str(object_root),
               "ownedFiles": desired, "physicalPath": str(destination), "updatedAt": time.time()})
    return {"physicalPath": str(destination), "reused": False, "incremental": bool(previous), "sourceDigest": source_digest}


__all__ = ["materialize_preparation"]
