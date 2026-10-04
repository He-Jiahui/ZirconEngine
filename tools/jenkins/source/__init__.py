"""Source ownership, canonical paths, and sealed input manifests.

This module deliberately keeps filesystem reads outside the state database
transaction.  A caller first obtains a path claim, then verifies the claimed
before hashes while building a sealed manifest.  The manifest is immutable and
its digest is the source identity used by later stages.
"""

from __future__ import annotations

import hashlib
import stat
import json
import os
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Mapping
from types import MappingProxyType

from ..contracts import JenkinsError, digest, file_digest
from ..state import State


def canonical_path(repository_root: str | Path, value: str | Path) -> str:
    """Return a repository-relative, case-insensitive canonical path.

    Paths are stored in the same slash-separated form on every host.  Absolute
    paths, traversal, empty components, and paths resolving outside the
    repository are rejected.  Existing reparse points are rejected before
    resolving so a claim cannot silently follow a junction or symlink.
    """
    root = Path(repository_root).absolute()
    raw = str(value).replace("\\", "/")
    if Path(raw).is_absolute() or (len(raw) >= 2 and raw[1] == ":"):
        raise JenkinsError("invalid_source_path", "source path must be repository-relative",
                           details={"path": raw})
    parts = raw.split("/")
    if not raw or any(part in {"", ".", ".."} for part in parts):
        raise JenkinsError("invalid_source_path", "source path contains an invalid component",
                           details={"path": raw})
    current = root
    for part in parts:
        current = current / part
        try:
            if current.is_symlink() or getattr(current.stat(), "st_reparse_tag", 0):
                raise JenkinsError("reparse_path", "source paths cannot cross reparse points",
                                   details={"path": str(current)})
        except FileNotFoundError:
            pass
    resolved_root = root.resolve()
    resolved = (root / Path(*parts)).resolve(strict=False)
    try:
        resolved.relative_to(resolved_root)
    except ValueError as exc:
        raise JenkinsError("invalid_source_path", "source path resolves outside repository",
                           details={"path": raw}) from exc
    return "/".join(parts).casefold()


def canonical_paths(repository_root: str | Path, paths: Iterable[str | Path]) -> list[str]:
    values = sorted({canonical_path(repository_root, path) for path in paths})
    if not values:
        raise JenkinsError("empty_source_scope", "at least one source path is required")
    return values


def _path_for(root: Path, relative: str) -> Path:
    return root.joinpath(*relative.split("/"))


def _before_hash(root: Path, relative: str) -> str | None:
    path = _path_for(root, relative)
    if not path.exists():
        return None
    if path.is_file():
        return file_digest(path)
    if path.is_dir():
        tree = []
        for child in sorted(path.rglob("*"), key=lambda item: item.as_posix().casefold()):
            if child.is_symlink() or getattr(child.stat(), "st_reparse_tag", 0):
                raise JenkinsError("reparse_path", "source paths cannot cross reparse points",
                                   details={"path": str(child)})
            if child.is_file():
                tree.append((child.relative_to(path).as_posix().casefold(), file_digest(child)))
        return digest(tree)
    raise JenkinsError("source_not_file", "source scope contains an unsupported path",
                       details={"path": relative})


def _overlaps(left: str, right: str) -> bool:
    return left == right or left.startswith(right + "/") or right.startswith(left + "/")


@dataclass(frozen=True)
class PathClaim:
    path: str
    owner: str
    epoch: int
    before_hash: str | None
    version: int
    repository_id: str = ""

    @classmethod
    def from_record(cls, record: dict[str, Any]) -> "PathClaim":
        payload = record["payload"]
        return cls(payload.get("path", record["key"]), payload["owner"], payload["epoch"],
                   payload.get("beforeHash"), record["version"], payload.get("repositoryId", ""))


def repository_id(repository_root: str | Path) -> str:
    root = Path(repository_root).absolute().resolve()
    return hashlib.sha256(str(root).casefold().encode("utf-8")).hexdigest()[:32]


def _claim_key(repository: str, path: str) -> str:
    return f"{repository}:{path}"


def claim_paths(state: State, repository_root: str | Path, owner: str,
                paths: Iterable[str | Path], *, before_hashes: Mapping[str, str | None] | None = None) -> list[PathClaim]:
    """Atomically claim paths and their ancestors for one owner.

    Claims are idempotent for the same owner and before hash.  Any overlapping
    claim by another owner is rejected before writing any record.
    """
    if not owner or not isinstance(owner, str):
        raise JenkinsError("invalid_owner", "path claim owner is required")
    selected = canonical_paths(repository_root, paths)
    repo_id = repository_id(repository_root)
    supplied = {canonical_path(repository_root, p): h for p, h in (before_hashes or {}).items()}
    actual = {path: _before_hash(Path(repository_root).absolute(), path) for path in selected}
    for path, expected in supplied.items():
        if path in actual and expected != actual[path]:
            raise JenkinsError("before_hash_mismatch", "source changed before it was claimed",
                               details={"path": path, "expected": expected, "observed": actual[path]})
    with state.transaction() as connection:
        records = [PathClaim.from_record(record) for record in state.list("path_claims", connection=connection)
                   if record["payload"].get("repositoryId", repo_id) == repo_id]
        for path in selected:
            for existing in records:
                if existing.owner != owner and _overlaps(existing.path, path):
                    raise JenkinsError("path_claim_conflict", "source path is owned by another task",
                                       retryable=True, details={"path": path, "owner": existing.owner})
        result: list[PathClaim] = []
        for path in selected:
            key = _claim_key(repo_id, path)
            old = state.get("path_claims", key, connection=connection)
            if old is not None:
                prior = PathClaim.from_record(old)
                if prior.owner != owner or prior.before_hash != actual[path]:
                    raise JenkinsError("path_claim_conflict", "path claim changed before retry", retryable=True,
                                       details={"path": path})
                result.append(prior)
                continue
            payload = {"path": path, "repositoryId": repo_id,
                       "repositoryRoot": str(Path(repository_root).absolute()), "owner": owner, "epoch": 1,
                       "beforeHash": actual[path], "afterHash": None, "status": "claimed"}
            record = state.put("path_claims", key, payload, connection=connection)
            result.append(PathClaim.from_record(record))
        state.event("source_paths_claimed", {"owner": owner, "paths": selected}, connection=connection)
    return result


def release_paths(state: State, owner: str, claims: Iterable[PathClaim]) -> None:
    """Release only the exact claim versions owned by ``owner``."""
    claims = list(claims)
    with state.transaction() as connection:
        for claim in claims:
            if not claim.repository_id:
                raise JenkinsError("claim_repository_missing", "path claim is missing repository scope")
            current = state.get("path_claims", _claim_key(claim.repository_id, claim.path), connection=connection)
            if current is None:
                continue
            observed = PathClaim.from_record(current)
            if observed.owner != owner or observed.epoch != claim.epoch:
                raise JenkinsError("claim_conflict", "path claim changed before release", retryable=True,
                                   details={"path": claim.path})
            state.delete("path_claims", _claim_key(claim.repository_id, claim.path), expected_version=claim.version, connection=connection)
        state.event("source_paths_released", {"owner": owner, "paths": [c.path for c in claims]}, connection=connection)


def verify_before_hashes(repository_root: str | Path, claims: Iterable[PathClaim]) -> None:
    root = Path(repository_root).absolute()
    for claim in claims:
        observed = _before_hash(root, claim.path)
        if observed != claim.before_hash:
            raise JenkinsError("before_hash_mismatch", "source changed after path claim",
                               details={"path": claim.path, "expected": claim.before_hash, "observed": observed})


@dataclass(frozen=True)
class SealedManifest:
    """Immutable source manifest with a canonical content identity."""

    payload: dict[str, Any]
    source_digest: str

    def __post_init__(self) -> None:
        object.__setattr__(self, "payload", _freeze_mapping(self.payload))

    @property
    def entries(self) -> tuple[dict[str, Any], ...]:
        return tuple(self.payload["entries"])


# Public name used by the workflow contract.
SourceManifest = SealedManifest


def compute_source_digest(manifest: Mapping[str, Any]) -> str:
    """Compute the identity of a canonical manifest payload."""
    if not isinstance(manifest, Mapping) or "entries" not in manifest:
        raise JenkinsError("invalid_source_manifest", "source manifest entries are required")
    return digest(_thaw(manifest))


def _thaw(value: Any) -> Any:
    if isinstance(value, Mapping):
        return {key: _thaw(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_thaw(item) for item in value]
    return value


def _freeze_mapping(value: Any) -> Any:
    if isinstance(value, dict):
        return MappingProxyType({key: _freeze_mapping(value[key]) for key in sorted(value)})
    if isinstance(value, list):
        return tuple(_freeze_mapping(item) for item in value)
    return value


def seal_manifest(repository_root: str | Path, claims: Iterable[PathClaim], *, base_head: str | None = None,
                  external_inputs: Iterable[Mapping[str, Any]] = ()) -> SealedManifest:
    """Capture a stable manifest after claims and recheck it before returning."""
    root = Path(repository_root).absolute()
    claims = sorted(list(claims), key=lambda claim: claim.path)
    if not claims:
        raise JenkinsError("empty_source_scope", "cannot seal an empty source scope")
    verify_before_hashes(root, claims)
    entries: list[dict[str, Any]] = []
    for claim in claims:
        path = _path_for(root, claim.path)
        observed = _before_hash(root, claim.path)
        if observed is None:
            entries.append({"path": claim.path, "status": "deleted", "sha256": None, "size": 0,
                            "beforeHash": claim.before_hash})
            continue
        if path.is_dir():
            after = _before_hash(root, claim.path)
            entries.append({"path": claim.path, "status": "present", "sha256": after,
                            "size": 0, "mode": stat.S_IMODE(path.stat().st_mode),
                            "beforeHash": claim.before_hash, "kind": "directory"})
            continue
        data = path.read_bytes()
        after = hashlib.sha256(data).hexdigest()
        if after != observed:
            raise JenkinsError("source_changed", "source changed while manifest was sealed",
                               retryable=True, details={"path": claim.path})
        entries.append({"path": claim.path, "status": "present", "sha256": after,
                        "size": len(data), "mode": stat.S_IMODE(path.stat().st_mode),
                        "beforeHash": claim.before_hash})
    final = {"schemaVersion": 1, "baseHead": base_head, "entries": entries,
             "external": list(external_inputs)}
    source_digest = digest(final)
    verify_before_hashes(root, claims)
    # A second digest pass catches a file that changed after its first read.
    for entry in entries:
        if entry["status"] == "present" and _before_hash(root, entry["path"]) != entry["sha256"]:
            raise JenkinsError("source_changed", "source changed before manifest finalization", retryable=True,
                               details={"path": entry["path"]})
    return SealedManifest(final, source_digest)


__all__ = ["PathClaim", "SealedManifest", "SourceManifest", "canonical_path", "canonical_paths",
           "claim_paths", "release_paths", "verify_before_hashes", "seal_manifest", "compute_source_digest",
           "collect_cargo_evidence", "persist_cargo_evidence", "register_sealed_consumer",
           "get_sealed_consumer", "require_sealed_consumer", "get_sealed_manifest",
           "get_patch_operation", "repository_id"]


def _atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".pending", dir=str(path.parent))
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def _auth_for_source(state: State, payload: Mapping[str, Any], paths: list[str], action: str) -> None:
    repository_id = payload.get("repositoryId")
    session_id = payload.get("sessionId")
    if repository_id is None or session_id is None:
        raise JenkinsError("authorization_missing", "repositoryId and sessionId are required")
    state.check_authorization(repository_id, session_id, action, paths)


def _approved_object_root(build_root: str | Path) -> Path:
    from ..resources import canonical_build_root, physical_path_under
    root = canonical_build_root(build_root)
    return physical_path_under(root, root.namespace(), allow_missing=True)


def apply_owned_patch(state: State, repository_root: str | Path, owner: str,
                      patch: Mapping[str, Any], before_hashes: Mapping[str, str | None], *,
                      operation_id: str, authorization: Mapping[str, Any] | None = None,
                      build_root: str | Path | None = None) -> dict[str, Any]:
    """Apply only owned file edits after a before-hash and authorization gate.

    ``patch`` maps repository-relative paths to UTF-8 text/bytes, or ``None``
    for deletion.  The operation journal is immutable and records pre/post
    hashes.  A foreign change observed after the write aborts the operation and
    never compensates files outside the supplied patch.
    """
    root = Path(repository_root).absolute()
    paths = canonical_paths(root, patch)
    if set(paths) != {canonical_path(root, key) for key in before_hashes}:
        raise JenkinsError("patch_scope_mismatch", "patch and before-hash scopes differ")
    if authorization is not None:
        _auth_for_source(state, authorization, paths, "implementation")
    normalized = {canonical_path(root, key): value for key, value in before_hashes.items()}
    patch = {canonical_path(root, key): value for key, value in patch.items()}
    payload = {"owner": owner, "paths": paths, "beforeHashes": normalized,
               "patchDigest": digest(patch)}
    old = state.get_operation(operation_id)
    if old is not None:
        if old["kind"] != "source_patch" or any(old["payload"].get(k) != v for k, v in payload.items()):
            raise JenkinsError("patch_identity_mismatch", "patch operation cannot replace its original input")
        if old["payload"].get("repositoryRoot", str(root)).casefold() != str(root).casefold():
            raise JenkinsError("patch_repository_mismatch", "patch operation belongs to another checkout")
        if old["status"] in {"complete", "aborted"}:
            return old
    object_root = _approved_object_root(build_root)
    pre = {path: _before_hash(root, path) for path in paths}
    expected_after = {path: None if value is None else hashlib.sha256(
        value if isinstance(value, bytes) else str(value).encode("utf-8")).hexdigest()
        for path, value in patch.items()}
    if old is None and pre != normalized:
        raise JenkinsError("before_hash_mismatch", "source changed before patch application", retryable=True)
    if old is not None and any(pre[p] not in {normalized[p], expected_after[p]} for p in paths):
        raise JenkinsError("foreign_edit_detected", "source changed during an interrupted patch")
    if old is None:
        before_objects = {}
        for path, checksum in normalized.items():
            before_objects[path] = checksum
            if checksum is not None:
                data = _path_for(root, path).read_bytes()
                if hashlib.sha256(data).hexdigest() != checksum:
                    raise JenkinsError("before_hash_mismatch", "source changed while preserving original bytes")
                destination = object_root / "inputs" / "objects" / checksum
                if not destination.exists():
                    _atomic_write(destination, data)
                elif file_digest(destination) != checksum:
                    raise JenkinsError("object_digest_mismatch", "original source object is corrupt")
        payload.update(repositoryRoot=str(root), objectRoot=str(object_root),
                       authorization=dict(authorization or {}), beforeObjects=before_objects,
                       expectedAfterHashes=expected_after)
        operation = state.put_operation("source_patch", operation_id, payload)
    else:
        operation = old
        if old["payload"].get("objectRoot") != str(object_root):
            raise JenkinsError("patch_build_root_mismatch", "interrupted patch must use its original build root")
    if old is None:
        claims = claim_paths(state, root, owner, paths, before_hashes=pre)
    else:
        repo_id = repository_id(root)
        records = {record["payload"].get("path", record["key"]): record
                   for record in state.list("path_claims")
                   if record["payload"].get("repositoryId", repo_id) == repo_id}
        claims = []
        for relative in paths:
            record = records.get(relative)
            if record is None:
                raise JenkinsError("claim_missing", "interrupted patch lost its source claim", retryable=True)
            claim = PathClaim.from_record(record)
            if claim.owner != owner:
                raise JenkinsError("claim_conflict", "interrupted patch is no longer owned", retryable=True)
            claims.append(claim)
    try:
        for path, value in patch.items():
            relative = canonical_path(root, path)
            destination = _path_for(root, relative)
            current_hash = _before_hash(root, relative)
            if current_hash == expected_after[relative]:
                continue
            if current_hash != pre[relative]:
                raise JenkinsError("foreign_edit_detected", "source changed before patch write")
            if value is None:
                if destination.exists():
                    if destination.is_dir():
                        raise JenkinsError("patch_directory", "patch deletion cannot remove a directory")
                    destination.unlink()
            else:
                data = value if isinstance(value, bytes) else str(value).encode("utf-8")
                _atomic_write(destination, data)
        after = {path: _before_hash(root, path) for path in paths}
        if after != expected_after:
            raise JenkinsError("foreign_edit_detected", "source differs from the applied patch")
        with state.transaction() as connection:
            for claim in claims:
                current = state.get("path_claims", _claim_key(claim.repository_id, claim.path), connection=connection)
                if current is None or current["payload"].get("owner") != owner:
                    raise JenkinsError("claim_conflict", "path claim disappeared during patch", retryable=True)
                state.put("path_claims", _claim_key(claim.repository_id, claim.path),
                          {**current["payload"], "beforeHash": after[claim.path],
                            "originalBeforeHash": normalized[claim.path],
                           "afterHash": after[claim.path], "status": "applied"},
                          expected_version=current["version"], connection=connection)
        state.transition_operation(operation_id, "prepared", "complete",
                                   {"beforeHashes": normalized, "afterHashes": after})
        return state.get_operation(operation_id) or operation
    except BaseException as error:
        current = {path: _before_hash(root, path) for path in paths}
        state.transition_operation(operation_id, "prepared", "aborted",
                                   {"beforeHashes": pre, "observedHashes": current,
                                    "reason": type(error).__name__})
        raise


def _git_base_tree(root: Path, base_head: str | None) -> dict[str, tuple[str, str]]:
    if not base_head:
        return {}
    try:
        completed = subprocess.run(["git", "-C", str(root), "ls-tree", "-r", "--full-tree", base_head],
                                   capture_output=True, text=True, check=True)
    except (OSError, subprocess.CalledProcessError) as exc:
        raise JenkinsError("git_base_unavailable", "cannot read the declared base tree") from exc
    result: dict[str, tuple[str, str]] = {}
    for line in completed.stdout.splitlines():
        meta, separator, path = line.partition("\t")
        if separator:
            fields = meta.split()
            if len(fields) >= 3 and fields[1] == "blob":
                key = path.casefold()
                if key in result and result[key][0] != path:
                    raise JenkinsError("git_path_collision", "base tree contains case-colliding paths",
                                       details={"paths": [result[key][0], path]})
                result[key] = (path, fields[2])
    return result


def _git_blob(root: Path, base_head: str, path: str) -> bytes:
    try:
        completed = subprocess.run(["git", "-C", str(root), "show", f"{base_head}:{path}"],
                                   capture_output=True, check=True)
    except (OSError, subprocess.CalledProcessError) as exc:
        raise JenkinsError("git_base_unavailable", "cannot read a blob from the declared base tree",
                           details={"path": path}) from exc
    return completed.stdout


def _check_object_store_domain(object_root: str | Path) -> None:
    path = Path(object_root).absolute()
    components = [part.casefold() for part in path.parts]
    if '.jenkins' not in components:
        return
    from ..resources.paths import canonical_build_root, physical_path_under
    if components.count('.jenkins') != 1:
        raise JenkinsError('object_root_rejected', 'Input objects cannot use a nested Jenkins runtime directory')
    index = components.index('.jenkins')
    if index + 1 >= len(components) or components[index + 1] != 'builds':
        raise JenkinsError('object_root_rejected', 'Input objects must be in the managed build directory, outside Jenkins Home')
    physical_path_under(canonical_build_root(None), path)


def seal_candidate(repository_root: str | Path, claims: Iterable[PathClaim], *,
                   object_root: str | Path, base_head: str | None = None,
                   declared_dependencies: Iterable[str | Path] = (),
                   external_inputs: Iterable[Mapping[str, Any]] = ()) -> SealedManifest:
    """Seal candidate bytes into a content-addressed input object store."""
    root = Path(repository_root).absolute()
    object_root = Path(object_root).absolute()
    _check_object_store_domain(object_root)
    claims = list(claims)
    dependency_paths = canonical_paths(root, declared_dependencies) if declared_dependencies else []
    all_paths = {claim.path: claim for claim in claims}
    for path in dependency_paths:
        if path not in all_paths:
            all_paths[path] = PathClaim(path, "dependency", 0, _before_hash(root, path), 0)
    claims = list(all_paths.values())
    base_tree = _git_base_tree(root, base_head)
    sealed = seal_manifest(root, claims, base_head=base_head, external_inputs=external_inputs)
    entries: list[dict[str, Any]] = []
    objects = object_root / "inputs" / "objects"
    for entry in sealed.entries:
        item = dict(entry)
        if item["status"] == "present":
            source = _path_for(root, item["path"])
            # ``seal_manifest`` captured and rechecked the source snapshot.  Do
            # not let the second read below silently publish bytes from a
            # concurrent edit (including an ABA edit): the object digest must
            # agree with that sealed entry.  Directories have no file bytes;
            # their serialized manifest record is the canonical object.
            if source.is_file():
                data = source.read_bytes()
                checksum = hashlib.sha256(data).hexdigest()
                if checksum != item["sha256"]:
                    raise JenkinsError("source_changed", "source changed while candidate was sealed",
                                       retryable=True, details={"path": item["path"]})
            else:
                data = json.dumps(item, sort_keys=True).encode()
                checksum = hashlib.sha256(data).hexdigest()
                if _before_hash(root, item["path"]) != item["sha256"]:
                    raise JenkinsError("source_changed", "source changed while candidate was sealed",
                                       retryable=True, details={"path": item["path"]})
            destination = objects / checksum
            if not destination.exists():
                _atomic_write(destination, data)
            elif file_digest(destination) != checksum:
                raise JenkinsError("object_digest_mismatch", "content-addressed input object is corrupt")
            # Verify the original claim after the object write as well.  This
            # closes the window where the source changes after the second read
            # and prevents a manifest/object pair with different identities.
            if _before_hash(root, item["path"]) != item["sha256"]:
                raise JenkinsError("source_changed", "source changed before candidate finalization",
                                   retryable=True, details={"path": item["path"]})
            item["objectDigest"] = checksum
        base = base_tree.get(item["path"].casefold())
        if base:
            item["path"] = base[0]
            item["baseBlob"] = base[1]
        else:
            item["baseBlob"] = None
        entries.append(item)
    # The Git base tree is an authority reference only. Physical objects remain
    # bounded to the declared candidate/dependency closure; unrelated base
    # blobs are never copied into the sealed input store.
    entries.sort(key=lambda item: item["path"])
    payload = {"schemaVersion": 1, "baseHead": base_head, "entries": entries,
               "external": list(external_inputs),
               "declaredDependencies": dependency_paths}
    source_digest = compute_source_digest(payload)
    manifest_path = objects.parent / f"{source_digest}.json"
    _atomic_write(manifest_path, json.dumps(payload, ensure_ascii=False, sort_keys=True,
                                             separators=(",", ":")).encode("utf-8"))
    return SealedManifest(payload, source_digest)


def materialize_manifest(manifest: SealedManifest | Mapping[str, Any], object_root: str | Path,
                         destination: str | Path) -> None:
    """Materialize exactly the sealed entries into a fresh input directory."""
    payload = manifest.payload if isinstance(manifest, SealedManifest) else manifest
    _check_object_store_domain(object_root)
    destination = Path(destination).absolute()
    destination.mkdir(parents=True, exist_ok=True)
    objects = Path(object_root).absolute() / "inputs" / "objects"
    # Reuse is allowed only after every expected object and materialized file
    # is verified. A stale or foreign file is never silently accepted.
    if any(destination.iterdir()):
        expected = {entry["path"]: entry for entry in payload.get("entries", [])
                    if entry.get("status") != "deleted"}
        for entry in expected.values():
            target = _path_for(destination, entry["path"])
            source = objects / entry.get("objectDigest", "")
            if not source.is_file() or file_digest(source) != entry.get("objectDigest"):
                raise JenkinsError("input_object_missing", "sealed input object is missing or corrupt",
                                   details={"path": entry["path"]})
            if not target.is_file() or file_digest(target) != entry["objectDigest"]:
                raise JenkinsError("materialize_digest_mismatch", "existing materialization differs from sealed input",
                                   details={"path": entry["path"]})
        for candidate in destination.rglob("*"):
            if candidate.is_symlink() or getattr(candidate.stat(), "st_reparse_tag", 0):
                raise JenkinsError("materialize_scope_mismatch", "existing materialization contains a reparse point")
        actual = {p.relative_to(destination).as_posix().casefold() for p in destination.rglob("*") if p.is_file()}
        if actual != {path.casefold() for path in expected}:
            raise JenkinsError("materialize_scope_mismatch", "existing materialization contains unsealed files")
        return
    for entry in payload.get("entries", []):
        if entry.get("status") == "deleted":
            continue
        target = _path_for(destination, entry["path"])
        source = objects / entry["objectDigest"]
        if not source.is_file() or file_digest(source) != entry["objectDigest"]:
            raise JenkinsError("input_object_missing", "sealed input object is missing or corrupt",
                               details={"path": entry["path"]})
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        if file_digest(target) != entry["objectDigest"]:
            raise JenkinsError("materialize_digest_mismatch", "materialized input differs from object")


def collect_cargo_evidence(manifest: SealedManifest | Mapping[str, Any], object_root: str | Path,
                           build_root: str | Path, *, cargo: str = "cargo", rustc: str = "rustc",
                           environment: Mapping[str, str] | None = None,
                           environment_policy_digest: str | None = None) -> dict[str, Any]:
    """Collect Cargo/toolchain evidence only from a sealed materialization.

    No caller supplied metadata is accepted.  Commands are read-only metadata
    queries and use an external Cargo target directory under the approved
    build root; Jenkins Home is never used for compiler output.
    """
    payload = manifest.payload if isinstance(manifest, SealedManifest) else manifest
    from ..resources import canonical_build_root, physical_path_under
    physical_root = canonical_build_root(build_root)
    approved = _approved_object_root(build_root)
    objects = physical_path_under(physical_root, object_root, allow_missing=False)
    if objects != approved:
        raise JenkinsError("object_root_mismatch", "Cargo evidence must use this buildRoot's input store")
    run_root = physical_path_under(physical_root, approved / "inputs" / "runs" / compute_source_digest(payload))
    # Recheck every object and existing byte on reuse, rather than trusting an
    # already populated directory left by an interrupted or foreign command.
    materialize_manifest(payload, objects, run_root)
    env = dict(os.environ)
    if environment is not None:
        env.update({str(key): str(value) for key, value in environment.items()})
    env["CARGO_TARGET_DIR"] = str(approved / "target")
    env["CARGO_HOME"] = str(approved / "cargo-home")
    temporary = physical_path_under(physical_root, approved / "tmp" / "metadata")
    temporary.mkdir(parents=True, exist_ok=True)
    env.update(TEMP=str(temporary), TMP=str(temporary), CARGO_NET_OFFLINE="true")
    try:
        metadata = subprocess.run([cargo, "metadata", "--no-deps", "--format-version", "1", "--locked", "--offline"],
                                  cwd=str(run_root), env=env, capture_output=True, text=True, check=True)
        cargo_version = subprocess.run([cargo, "--version"], cwd=str(run_root), env=env,
                                       capture_output=True, text=True, check=True)
        rustc_version = subprocess.run([rustc, "--version", "--verbose"], cwd=str(run_root), env=env,
                                       capture_output=True, text=True, check=True)
    except (OSError, subprocess.CalledProcessError) as exc:
        raise JenkinsError("cargo_evidence_failed", "sealed Cargo evidence collection failed", retryable=True) from exc
    try:
        metadata_value = json.loads(metadata.stdout)
    except json.JSONDecodeError as exc:
        raise JenkinsError("cargo_metadata_invalid", "cargo metadata did not return JSON") from exc
    lock_path = run_root / "Cargo.lock"
    lock_digest = file_digest(lock_path) if lock_path.is_file() else None
    input_digests: dict[str, str] = {}
    for candidate in sorted(run_root.rglob("*"), key=lambda p: p.as_posix().casefold()):
        if not candidate.is_file() or candidate.is_symlink() or getattr(candidate.stat(), "st_reparse_tag", 0):
            continue
        rel = candidate.relative_to(run_root).as_posix()
        name = candidate.name.casefold()
        if name in {"cargo.toml", "cargo.lock"} or name.startswith("config"):
            input_digests[rel] = file_digest(candidate)
    if not input_digests:
        raise JenkinsError("cargo_inputs_missing", "sealed materialization contains no Cargo manifest")
    toolchain = {"cargo": cargo_version.stdout.strip(), "rustc": rustc_version.stdout.strip()}
    return {"cargoMetadata": metadata_value, "metadata": metadata_value,
            "metadataDigest": digest(metadata_value), "toolchain": toolchain,
            "toolchainDigest": digest(toolchain), "lockDigest": lock_digest,
            "compilerIdentity": rustc_version.stdout.strip(),
            "commandDigests": {"metadata": digest([cargo, "metadata", "--no-deps", "--format-version", "1", "--locked", "--offline"]),
                               "cargoVersion": digest([cargo, "--version"]),
                               "rustcVersion": digest([rustc, "--version", "--verbose"])},
             "sourceDigest": compute_source_digest(payload), "inputDigests": input_digests,
             # The sealed run is the authoritative workspace used for Cargo
             # metadata. Keep both names for older evidence readers, but bind
             # them to the same physically checked path.
             "workspaceRoot": str(run_root),
             "manifestInputsDigest": digest(input_digests), "materializedRoot": str(run_root),
             "buildRoot": str(physical_root.path), "environmentPolicyDigest": environment_policy_digest}


def _consumer_key(repository_id_value: str, session_id: str, owner: str, source_digest: str) -> str:
    return digest({"repositoryId": repository_id_value, "sessionId": session_id,
                   "owner": owner, "sourceDigest": source_digest})


def register_sealed_consumer(state: State, *, repository_id_value: str, session_id: str,
                             owner: str, source_digest: str, manifest_digest: str) -> dict[str, Any]:
    """Register an immutable per-consumer reference to a shared sealed input."""
    values = (repository_id_value, session_id, owner, source_digest, manifest_digest)
    if not all(isinstance(value, str) and value for value in values):
        raise JenkinsError("consumer_identity_missing", "sealed consumer identity is required")
    key = _consumer_key(repository_id_value, session_id, owner, source_digest)
    payload = {"repositoryId": repository_id_value, "sessionId": session_id, "owner": owner,
               "sourceDigest": source_digest, "manifestDigest": manifest_digest, "status": "active"}
    existing = state.get("sealed_consumer", key)
    if existing is not None:
        if existing["payload"] != payload:
            raise JenkinsError("consumer_identity_conflict", "sealed consumer reference is immutable")
        return existing
    return state.put("sealed_consumer", key, payload)


def get_sealed_consumer(state: State, *, repository_id_value: str, session_id: str,
                        owner: str, source_digest: str) -> dict[str, Any] | None:
    return state.get("sealed_consumer", _consumer_key(repository_id_value, session_id, owner, source_digest))


def require_sealed_consumer(state: State, *, repository_id_value: str, session_id: str,
                            owner: str, source_digest: str,
                            manifest_digest: str | None = None) -> dict[str, Any]:
    """Return the consumer binding or reject a cross-session/cross-source use."""
    record = get_sealed_consumer(state, repository_id_value=repository_id_value,
                                 session_id=session_id, owner=owner, source_digest=source_digest)
    if record is None:
        raise JenkinsError("sealed_consumer_missing", "sealed input is not registered for this consumer")
    payload = record["payload"]
    if manifest_digest is not None and payload.get("manifestDigest") != manifest_digest:
        raise JenkinsError("sealed_consumer_mismatch", "consumer manifest binding differs")
    return record


def get_sealed_manifest(state: State, *, repository_id_value: str, session_id: str,
                        owner: str, source_digest: str) -> SealedManifest:
    """Load the immutable manifest recorded by seal, after consumer binding."""
    require_sealed_consumer(state, repository_id_value=repository_id_value,
                            session_id=session_id, owner=owner,
                            source_digest=source_digest, manifest_digest=source_digest)
    record = state.get("sealed_input", source_digest)
    if record is None:
        raise JenkinsError("sealed_input_missing", "sealed source manifest is unavailable")
    payload = record["payload"]
    manifest = payload.get("sourceManifest") or payload.get("manifest")
    if not isinstance(manifest, Mapping) or compute_source_digest(manifest) != source_digest:
        raise JenkinsError("sealed_input_corrupt", "sealed source manifest digest does not match identity")
    return SealedManifest(dict(manifest), source_digest)


def persist_cargo_evidence(state: State, evidence: Mapping[str, Any], *, repository_id_value: str,
                           session_id: str, owner: str,
                           manifest: SealedManifest | Mapping[str, Any]) -> dict[str, Any]:
    """Persist collector output bound to an immutable sealed manifest."""
    payload = manifest.payload if isinstance(manifest, SealedManifest) else manifest
    source_digest = compute_source_digest(payload)
    if evidence.get("sourceDigest") != source_digest:
        raise JenkinsError("cargo_evidence_source_mismatch", "Cargo evidence is not bound to sealed source")
    require_sealed_consumer(state, repository_id_value=repository_id_value, session_id=session_id,
                            owner=owner, source_digest=source_digest, manifest_digest=source_digest)
    canonical = _thaw(dict(evidence))
    canonical.update({"repositoryId": repository_id_value,
                      "sourceDigest": source_digest, "manifestDigest": source_digest})
    existing = state.get("sealed_cargo_evidence", source_digest)
    if existing is not None:
        if existing["payload"] != canonical:
            raise JenkinsError("cargo_evidence_conflict", "sealed Cargo evidence is immutable")
        return existing
    return state.put("sealed_cargo_evidence", source_digest, canonical)


def get_patch_operation(state: State, operation_id: str) -> dict[str, Any]:
    operation = state.get_operation(operation_id)
    if operation is None or operation["kind"] != "source_patch":
        raise JenkinsError("operation_not_found", "source patch operation does not exist")
    if operation["status"] not in {"complete", "aborted"}:
        raise JenkinsError("operation_pending", "source patch operation has no terminal evidence", retryable=True)
    return operation


def handle(action: str, payload: dict, state: State, repo_root: Path, *, domain: str = "candidate") -> dict:
    """CLI handler for candidate claim/apply/seal/materialize operations."""
    if action == "claim":
        paths = payload.get("ownedPaths", [])
        _auth_for_source(state, payload, canonical_paths(repo_root, paths), "implementation")
        claims = claim_paths(state, repo_root, payload["owner"], paths,
                             before_hashes=payload.get("beforeHashes"))
        return {"status": "claimed", "claims": [claim.__dict__ for claim in claims]}
    if action == "apply":
        return apply_owned_patch(state, repo_root, payload["owner"], payload["patch"],
                                 payload["beforeHashes"], operation_id=payload["operationId"],
                                 authorization=payload,
                                 build_root=payload.get("buildRoot"))
    if action == "seal":
        claims = [PathClaim(**value) for value in payload["claims"]]
        paths = [claim.path for claim in claims]
        _auth_for_source(state, payload, paths, "validation")
        coverage = payload.get("coverage")
        if not isinstance(coverage, dict):
            raise JenkinsError("coverage_missing", "sealed source requires an explicit coverage object")
        build_root = payload.get("buildRoot")
        approved_root = _approved_object_root(build_root)
        from ..resources import canonical_build_root, physical_path_under
        supplied_root = physical_path_under(canonical_build_root(build_root), payload.get("objectRoot", ""))
        if supplied_root != approved_root:
            raise JenkinsError("object_root_mismatch", "objectRoot must be derived from the approved buildRoot")
        operation_refs = payload.get("patchOperationRefs", [])
        if not operation_refs or not all(isinstance(value, str) for value in operation_refs):
            raise JenkinsError("patch_journal_missing", "sealed source requires patch operation references")
        for operation_id in operation_refs:
            operation = get_patch_operation(state, operation_id)
            if operation["status"] != "complete":
                raise JenkinsError("patch_not_complete", "sealed source references an incomplete patch")
            if operation["payload"].get("owner") != payload.get("owner"):
                raise JenkinsError("patch_owner_mismatch", "sealed source patch owner differs")
        if coverage.get("sourceDigest") not in {None, payload.get("sourceDigest")}:
            raise JenkinsError("coverage_source_mismatch", "coverage is not bound to this source")
        manifest = seal_candidate(repo_root, claims, object_root=approved_root,
                                  base_head=payload.get("baseHead"),
                                  declared_dependencies=payload.get("declaredDependencies", []),
                                  external_inputs=payload.get("externalInputs", []))
        coverage = {**coverage, "sourceDigest": manifest.source_digest}
        coverage = {**coverage, "coverageDigest": digest(coverage)}
        plain_manifest = _thaw(manifest.payload)
        sealed_payload = {"status": "sealed", "repositoryRoot": str(repo_root),
                          "objectRoot": str(approved_root), "buildRoot": str(build_root),
                          "sourceManifest": plain_manifest, "manifest": plain_manifest,
                          "coverage": coverage, "sourceDigest": manifest.source_digest,
                          "coverageDigest": coverage["coverageDigest"],
                          "owner": payload.get("owner"), "claims": [claim.__dict__ for claim in claims],
                          "ownedPaths": paths, "patchOperationRefs": operation_refs,
                          "repositoryId": payload.get("repositoryId"),
                          "sessionId": payload.get("sessionId")}
        existing = state.get("sealed_input", manifest.source_digest)
        if existing is not None:
            prior = existing["payload"]
            for field in ("sourceManifest", "objectRoot", "repositoryId"):
                if prior.get(field) != sealed_payload.get(field):
                    raise JenkinsError("sealed_input_conflict", "sealed input identity is immutable")
        else:
            state.put("sealed_input", manifest.source_digest, sealed_payload)
        register_sealed_consumer(
            state,
            repository_id_value=str(payload.get("repositoryId")),
            session_id=str(payload.get("sessionId")),
            owner=str(payload.get("owner")),
            source_digest=manifest.source_digest,
            manifest_digest=manifest.source_digest,
        )
        return {"status": "sealed", "sourceDigest": manifest.source_digest, "manifest": plain_manifest}
    if action == "materialize":
        source_digest = str(payload.get("sourceDigest", ""))
        identity = (payload.get("repositoryId"), payload.get("sessionId"), payload.get("owner"))
        if source_digest and all(isinstance(value, str) and value for value in identity):
            manifest = get_sealed_manifest(state, repository_id_value=identity[0],
                                           session_id=identity[1], owner=identity[2],
                                           source_digest=source_digest).payload
        else:
            raise JenkinsError("sealed_consumer_missing", "materialization requires a registered sealed consumer")
        paths = [entry["path"] for entry in manifest.get("entries", [])]
        _auth_for_source(state, payload, paths, "validation")
        materialize_manifest(manifest, payload["objectRoot"], payload["destination"])
        return {"status": "materialized", "sourceDigest": compute_source_digest(manifest),
                "repositoryId": identity[0], "sessionId": identity[1], "owner": identity[2]}
    raise JenkinsError("operation_unknown", f"Unknown candidate action: {action}")
