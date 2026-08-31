from __future__ import annotations

import hashlib
import io
import json
import re
import subprocess
import tarfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Mapping, Sequence

from .models import CoordinatorError
from .portable_paths import normalize_portable_relative_path, portable_path_key
from .trusted_tools import trusted_git_command


_CARGO_TOPOLOGY_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
)
_FULL_GIT_COMMIT = re.compile(r"^[0-9a-f]{40}$", re.IGNORECASE)
EXTERNAL_REPOSITORY_ROOT = "@repo-root"


def _relative_path(value: object, *, code: str) -> str:
    return normalize_portable_relative_path(
        value,
        code=code,
        message="Validation source path must be a safe portable relative path",
    )


@dataclass(frozen=True, slots=True)
class ExternalGitSource:
    repo_root: Path
    commit: str
    mount_path: str
    include_roots: tuple[str, ...]
    source_hash: str
    archive_hash: str | None = None
    archive_byte_count: int | None = None

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> "ExternalGitSource":
        required = {
            "repoRoot",
            "commit",
            "mountPath",
            "includeRoots",
        }
        if (
            not isinstance(payload, Mapping)
            or not required <= set(payload)
            or not set(payload)
            <= required | {"sourceHash", "archiveHash", "archiveByteCount"}
        ):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source requires repoRoot, commit, mountPath, and includeRoots",
            )
        repo_root = Path(str(payload["repoRoot"])).resolve()
        commit = str(payload["commit"]).strip()
        if not _FULL_GIT_COMMIT.fullmatch(commit):
            raise CoordinatorError(
                "validation_copy_external_commit_invalid",
                "External Git source commit must be a full immutable object id",
                details={"commit": commit},
            )
        commit = commit.lower()
        mount_path = _relative_path(
            payload["mountPath"], code="validation_copy_external_mount_escape"
        )
        if (
            len(PurePosixPath(mount_path).parts) != 1
            or mount_path.casefold() != repo_root.name.casefold()
            or mount_path.casefold() in {"source", "target"}
        ):
            raise CoordinatorError(
                "validation_copy_external_mount_escape",
                "External Git mount must be its canonical non-reserved sibling repository name",
                details={"repoRoot": str(repo_root), "mountPath": mount_path},
            )
        mount_path = repo_root.name
        raw_roots = payload["includeRoots"]
        if not isinstance(raw_roots, Sequence) or isinstance(raw_roots, (str, bytes)):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git includeRoots must be a non-empty list",
            )
        include_roots = tuple(
            sorted(
                {
                    (
                        EXTERNAL_REPOSITORY_ROOT
                        if root == EXTERNAL_REPOSITORY_ROOT
                        else _relative_path(
                            root, code="validation_copy_external_include_root_invalid"
                        )
                    )
                    for root in raw_roots
                },
                key=str.casefold,
            )
        )
        if EXTERNAL_REPOSITORY_ROOT in include_roots:
            include_roots = (EXTERNAL_REPOSITORY_ROOT,)
        if not include_roots:
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source must declare include roots",
            )
        raw_archive_hash = payload.get("archiveHash")
        archive_hash = (
            str(raw_archive_hash).strip().lower()
            if raw_archive_hash is not None
            else None
        )
        raw_archive_bytes = payload.get("archiveByteCount")
        if (archive_hash is None) != (raw_archive_bytes is None):
            raise CoordinatorError(
                "validation_copy_external_archive_identity_invalid",
                "Sealed external sources require archiveHash and archiveByteCount together",
            )
        if archive_hash is not None and (
            len(archive_hash) != 64
            or any(character not in "0123456789abcdef" for character in archive_hash)
            or not isinstance(raw_archive_bytes, int)
            or isinstance(raw_archive_bytes, bool)
            or raw_archive_bytes < 0
        ):
            raise CoordinatorError(
                "validation_copy_external_archive_identity_invalid",
                "External source archive identity must be a SHA-256 and non-negative byte count",
            )
        archive_byte_count = (
            int(raw_archive_bytes) if raw_archive_bytes is not None else None
        )
        source_hash = hashlib.sha256(
            json.dumps(
                {
                    "repoRoot": str(repo_root),
                    "commit": commit,
                    "mountPath": mount_path,
                    "includeRoots": include_roots,
                    "archiveHash": archive_hash,
                    "archiveByteCount": archive_byte_count,
                },
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()
        return cls(
            repo_root,
            commit,
            mount_path,
            include_roots,
            source_hash,
            archive_hash,
            archive_byte_count,
        )

    def pinned(self) -> "ExternalGitSource":
        if self.archive_hash is not None:
            return self
        if not self.repo_root.is_dir():
            raise CoordinatorError(
                "validation_copy_external_repository_missing",
                "External Git repository root does not exist",
                details={"repoRoot": str(self.repo_root)},
            )
        result = subprocess.run(
            trusted_git_command(self.repo_root, "rev-parse", "--verify", f"{self.commit}^{{commit}}"),
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_external_commit_missing",
                "External Git source commit is unavailable",
                details={"repoRoot": str(self.repo_root), "commit": self.commit},
            )
        resolved = result.stdout.strip().lower()
        if resolved != self.commit:
            raise CoordinatorError(
                "validation_copy_external_commit_invalid",
                "External Git source did not resolve to its recorded immutable object id",
                details={"repoRoot": str(self.repo_root), "commit": self.commit},
            )
        return ExternalGitSource.from_payload(
            {
                "repoRoot": str(self.repo_root),
                "commit": resolved,
                "mountPath": self.mount_path,
                "includeRoots": list(self.include_roots),
            }
        )

    def to_payload(self) -> dict[str, object]:
        payload: dict[str, object] = {
            "repoRoot": str(self.repo_root),
            "commit": self.commit,
            "mountPath": self.mount_path,
            "includeRoots": list(self.include_roots),
            "sourceHash": self.source_hash,
        }
        if self.archive_hash is not None:
            payload["archiveHash"] = self.archive_hash
            payload["archiveByteCount"] = self.archive_byte_count
        return payload


def extract_external_archive(
    content: bytes,
    destination_root: Path,
    *,
    error_code: str = "validation_copy_external_archive_invalid",
) -> dict[str, str]:
    """Extract a sealed Git archive as regular, portable task-owned files."""
    destination_root.mkdir(parents=True, exist_ok=True)
    entries: dict[str, str] = {}
    portable_entries: dict[str, str] = {}
    try:
        archive = tarfile.open(fileobj=io.BytesIO(content), mode="r:")
    except (tarfile.TarError, OSError) as error:
        raise CoordinatorError(error_code, "External Git archive is unreadable") from error
    with archive:
        for member in archive.getmembers():
            if member.isdir():
                continue
            if not member.isfile():
                raise CoordinatorError(
                    error_code,
                    "External Git archive may contain only regular files",
                    details={"path": member.name},
                )
            relative = _relative_path(member.name, code=error_code)
            key = portable_path_key(relative)
            previous = portable_entries.get(key)
            if previous is not None:
                raise CoordinatorError(
                    error_code,
                    "External Git archive contains colliding portable paths",
                    details={"firstPath": previous, "secondPath": relative},
                )
            portable_entries[key] = relative
            destination = destination_root.joinpath(*PurePosixPath(relative).parts)
            if destination.exists() or destination.is_symlink():
                raise CoordinatorError(
                    error_code,
                    "External Git archive attempted to replace an existing path",
                    details={"path": relative},
                )
            stream = archive.extractfile(member)
            if stream is None:
                raise CoordinatorError(
                    error_code,
                    "External Git archive contains an unreadable file",
                    details={"path": relative},
                )
            with stream:
                payload = stream.read()
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(payload)
            entries[relative] = hashlib.sha256(payload).hexdigest()
    return entries


def external_tree_paths(source: ExternalGitSource) -> frozenset[str]:
    result = subprocess.run(
        trusted_git_command(source.repo_root, "ls-tree", "-r", "--name-only", source.commit),
        cwd=source.repo_root,
        check=False,
        capture_output=True,
        encoding="utf-8",
    )
    if result.returncode != 0:
        raise CoordinatorError(
            "validation_copy_external_commit_missing",
            "External Git source commit cannot provide topology inputs",
            details={"repoRoot": str(source.repo_root), "commit": source.commit},
        )
    return frozenset(line for line in result.stdout.splitlines() if line)


def external_source_includes_path(
    source: ExternalGitSource, relative_path: str
) -> bool:
    return EXTERNAL_REPOSITORY_ROOT in source.include_roots or any(
        relative_path == root or relative_path.startswith(root + "/")
        for root in source.include_roots
    )


def external_archive_pathspecs(source: ExternalGitSource) -> tuple[str, ...]:
    return (
        ()
        if EXTERNAL_REPOSITORY_ROOT in source.include_roots
        else source.include_roots
    )


def external_topology_paths(
    source: ExternalGitSource,
    manifest: Path,
    tracked_paths: frozenset[str],
) -> set[str]:
    relative_manifest = manifest.relative_to(source.repo_root)
    paths = {relative_manifest.as_posix()}
    parent = relative_manifest.parent
    while True:
        at_root = not parent.parts
        for name in _CARGO_TOPOLOGY_FILES:
            candidate = (
                PurePosixPath(name)
                if at_root
                else PurePosixPath(parent.as_posix()) / name
            )
            if candidate.as_posix() in tracked_paths:
                paths.add(candidate.as_posix())
        if at_root:
            return paths
        parent = parent.parent
