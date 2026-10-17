"""Resolve immutable Git preimages for coordinator memory-only patch rendering.

This module neither reads dirty file bytes as a baseline nor writes Git, source,
State or an object store. A coordinator-issued baseline reference needs its own
authoritative resolver; an unimplemented reference never falls back to Git.
"""
from __future__ import annotations

import hashlib
import os
import re
import stat
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from types import MappingProxyType
from typing import Mapping

from ..contracts import JenkinsError
from . import canonical_path, repository_id
from .unified_patch import _normalized_path

_OID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
_SHA256 = re.compile(r"[0-9a-f]{64}\Z")
_MAX_PATHS = 512
_MAX_OBJECT_BYTES = 16 * 1024 * 1024
_MAX_BASE_BYTES = 64 * 1024 * 1024


@dataclass(frozen=True)
class GitPathProvenance:
    path: str
    git_path: str
    mode: str | None
    object_id: str | None
    sha256: str | None


@dataclass(frozen=True)
class GitBaselineProvenance:
    repository_id: str
    repository_root: str
    base_head: str
    tree_object_id: str
    paths: tuple[GitPathProvenance, ...]
    kind: str = "git-commit"


@dataclass(frozen=True)
class TrustedBaseline:
    provenance: GitBaselineProvenance
    before_bytes: Mapping[str, bytes | None] = field(repr=False)

    def __post_init__(self):
        object.__setattr__(self, "before_bytes", MappingProxyType(dict(self.before_bytes)))


def _git(root: Path, *arguments: str, allow_missing: bool = False) -> bytes:
    # Do not let inherited Git overrides replace the repository or its objects.
    environment = {key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")}
    environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                       GIT_OPTIONAL_LOCKS="0", GIT_NO_LAZY_FETCH="1", GIT_TERMINAL_PROMPT="0")
    try:
        result = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "--no-pager", "--no-replace-objects", "-C", str(root),
                                 *arguments], env=environment, capture_output=True, timeout=15)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise JenkinsError("baseline_git_unavailable", "Cannot read the immutable Git baseline") from error
    if result.returncode == 1 and allow_missing:
        return b""
    if result.returncode != 0:
        # stderr may contain unrelated configuration; keep it out of receipts.
        raise JenkinsError("baseline_git_unavailable", "Git could not verify the requested baseline")
    return result.stdout


def _plain_root(repository_root: str | Path, expected_id: str) -> Path:
    root = Path(repository_root).absolute()
    for path in (root, *root.parents):
        try:
            info = path.lstat()
        except OSError as error:
            raise JenkinsError("baseline_repository_invalid", "Baseline checkout is unavailable") from error
        if path.is_symlink() or getattr(info, "st_file_attributes", 0) & 0x400:
            raise JenkinsError("baseline_repository_alias", "Baseline checkout must have a physical canonical path")
    if not root.is_dir() or root.resolve() != root or repository_id(root) != expected_id:
        raise JenkinsError("baseline_repository_mismatch", "Baseline repository identity differs from the physical checkout")
    canonical_path(root, ".git")
    if not (root / ".git").exists():
        raise JenkinsError("baseline_required", "The checkout has no authoritative Git baseline")
    return root


def _binding(root: Path, expected_id: str, base_head: str) -> None:
    _plain_root(root, expected_id)
    try:
        actual_root = Path(_git(root, "rev-parse", "--show-toplevel").decode("utf-8").strip()).absolute()
        actual_head = _git(root, "rev-parse", "--verify", "HEAD^{commit}").decode("ascii").strip()
    except (UnicodeError, ValueError) as error:
        raise JenkinsError("baseline_git_unavailable", "Git returned an invalid repository binding") from error
    if actual_root != root:
        raise JenkinsError("baseline_repository_mismatch", "Git is bound to a different checkout")
    if actual_head != base_head:
        raise JenkinsError("baseline_base_changed", "The baseline commit differs from the checkout's current HEAD")


def _object(root: Path, kind: str, object_id: str, cache: dict) -> bytes:
    key = (kind, object_id)
    if key in cache:
        return cache[key]
    if not _OID.fullmatch(object_id):
        raise JenkinsError("baseline_object_invalid", "Git object identity is invalid")
    size = _git(root, "cat-file", "-s", object_id).strip()
    if not size.isdigit() or int(size) > _MAX_OBJECT_BYTES:
        raise JenkinsError("baseline_too_large", "Git baseline object exceeds the bounded intake")
    data = _git(root, "cat-file", kind, object_id)
    algorithm = hashlib.sha1 if len(object_id) == 40 else hashlib.sha256
    actual = algorithm(kind.encode("ascii") + b" " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()
    if len(data) != int(size) or actual != object_id:
        raise JenkinsError("baseline_object_corrupt", "Git object bytes differ from their immutable identity")
    cache[key] = data
    return data


def _tree(root: Path, object_id: str, cache: dict) -> dict:
    data = _object(root, "tree", object_id, cache)
    width = len(object_id) // 2
    result, cursor = {}, 0
    while cursor < len(data):
        end = data.find(b"\0", cursor)
        if end < 0 or end + 1 + width > len(data):
            raise JenkinsError("baseline_tree_invalid", "Git tree record is incomplete")
        header = data[cursor:end]
        try:
            raw_mode, raw_name = header.split(b" ", 1)
            mode, name = raw_mode.decode("ascii"), raw_name.decode("utf-8")
        except (ValueError, UnicodeError) as error:
            raise JenkinsError("baseline_tree_invalid", "Git tree path or mode is malformed") from error
        if not name or "/" in name or "\\" in name or name in {".", ".."}:
            raise JenkinsError("baseline_tree_invalid", "Git tree contains an invalid path component")
        key = name.casefold()
        if key in result:
            raise JenkinsError("baseline_path_collision", "Git tree contains case-insensitive aliases")
        result[key] = (name, mode, data[end + 1:end + 1 + width].hex())
        cursor = end + 1 + width
    return result


def _lookup(root: Path, tree_id: str, path: str, cache: dict):
    parts, actual = path.split("/"), []
    for index, part in enumerate(parts):
        entry = _tree(root, tree_id, cache).get(part)
        if entry is None:
            return "/".join([*actual, *parts[index:]]), None, None
        name, mode, oid = entry
        actual.append(name)
        if index == len(parts) - 1:
            if mode not in {"100644", "100755"}:
                raise JenkinsError("baseline_mode_invalid", "Patch preimages must be ordinary Git files")
            return "/".join(actual), mode, oid
        if mode not in {"40000", "040000"}:
            raise JenkinsError("baseline_mode_invalid", "Patch path crosses a non-directory Git entry")
        tree_id = oid
    raise JenkinsError("baseline_tree_invalid", "Git baseline path is empty")


def _scope(root: Path, before_hashes: Mapping) -> dict:
    if not isinstance(before_hashes, Mapping) or not before_hashes or len(before_hashes) > _MAX_PATHS:
        raise JenkinsError("baseline_scope_invalid", "Provide a bounded explicit before-hash scope")
    result = {}
    for path, checksum in before_hashes.items():
        if not isinstance(path, str):
            raise JenkinsError("baseline_scope_invalid", "Baseline paths must be strings")
        name = _normalized_path(path)
        if name.split("/", 1)[0] == ".git" or name in result:
            raise JenkinsError("baseline_scope_invalid", "Baseline paths contain controls or aliases")
        canonical_path(root, path)
        current = root
        for part in path.replace("\\", "/").split("/"):
            current /= part
            try:
                info = current.lstat()
            except FileNotFoundError:
                continue
            if current.is_symlink() or getattr(info, "st_file_attributes", 0) & 0x400:
                raise JenkinsError("baseline_path_alias", "Baseline paths cannot cross physical aliases")
        target = root.joinpath(*name.split("/"))
        if target.exists() and not stat.S_ISREG(target.lstat().st_mode):
            raise JenkinsError("baseline_mode_invalid", "Current patch path must be an ordinary file or absent")
        if checksum is not None and (not isinstance(checksum, str) or not _SHA256.fullmatch(checksum)):
            raise JenkinsError("baseline_hash_invalid", "Expected before hashes must be SHA-256 or null")
        result[name] = checksum
    return dict(sorted(result.items()))


def resolve_git_baseline(repository_root: str | Path, *, repository_id_value: str,
                         base_head: str, before_hashes: Mapping[str, str | None]) -> TrustedBaseline:
    """Read exactly this checkout's current immutable Git commit and blobs.

    Dirty files never supply a preimage. An expected hash that includes another
    uncommitted change needs a coordinator-issued baseline instead of Git.
    """
    if not isinstance(base_head, str) or not _OID.fullmatch(base_head):
        raise JenkinsError("baseline_base_invalid", "Git baseline requires a full immutable commit ID")
    root = _plain_root(repository_root, repository_id_value)
    scope = _scope(root, before_hashes)
    _binding(root, repository_id_value, base_head)
    # Older installed Git lacks the no-lazy-fetch switch. Reject partial clones
    # explicitly as well as disabling lazy fetch in the child environment.
    partial = _git(root, "config", "--local", "--includes", "--get-regexp",
                   r"^(extensions\.partialclone|remote\..*\.promisor)$", allow_missing=True)
    if partial:
        raise JenkinsError("baseline_required", "Partial clones need a coordinator-issued local baseline")
    cache: dict = {}
    commit = _object(root, "commit", base_head, cache)
    first = commit.split(b"\n", 1)[0]
    try:
        tree_id = first.removeprefix(b"tree ").decode("ascii")
    except UnicodeError as error:
        raise JenkinsError("baseline_object_invalid", "Git commit has an invalid root tree") from error
    if not first.startswith(b"tree ") or len(tree_id) != len(base_head) or not _OID.fullmatch(tree_id):
        raise JenkinsError("baseline_object_invalid", "Git commit has no valid root tree")
    contents, paths, total = {}, [], 0
    for path, expected in scope.items():
        git_path, mode, oid = _lookup(root, tree_id, path, cache)
        data = None if oid is None else _object(root, "blob", oid, cache)
        actual = None if data is None else hashlib.sha256(data).hexdigest()
        if actual != expected:
            raise JenkinsError("before_hash_mismatch", "Expected preimage is not the immutable Git baseline",
                               details={"path": path, "expected": expected, "observed": actual})
        total += len(data) if data is not None else 0
        if total > _MAX_BASE_BYTES:
            raise JenkinsError("baseline_too_large", "Git baseline exceeds the bounded intake")
        contents[path] = data
        paths.append(GitPathProvenance(path, git_path, mode, oid, actual))
    _scope(root, before_hashes)
    _binding(root, repository_id_value, base_head)
    return TrustedBaseline(GitBaselineProvenance(repository_id_value, str(root), base_head,
                                                tree_id, tuple(paths)), contents)


def resolve_baseline(repository_root: str | Path, *, repository_id_value: str,
                     before_hashes: Mapping[str, str | None], base_head: str | None = None,
                     baseline_ref: str | None = None) -> TrustedBaseline:
    if baseline_ref is not None or base_head is None:
        raise JenkinsError("baseline_required", "A coordinator-issued baseline reference has no authoritative resolver yet")
    return resolve_git_baseline(repository_root, repository_id_value=repository_id_value,
                                base_head=base_head, before_hashes=before_hashes)


__all__ = ["GitPathProvenance", "GitBaselineProvenance", "TrustedBaseline",
           "resolve_git_baseline", "resolve_baseline"]
