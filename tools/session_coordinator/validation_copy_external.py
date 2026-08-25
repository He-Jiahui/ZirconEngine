from __future__ import annotations

import hashlib
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Mapping, Sequence

from .models import CoordinatorError


_CARGO_TOPOLOGY_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
)


def _relative_path(value: object, *, code: str) -> str:
    if not isinstance(value, str):
        raise CoordinatorError(code, "Validation source path must be text")
    normalized = value.strip().replace("\\", "/").strip("/")
    path = PurePosixPath(normalized)
    if not normalized or path.is_absolute() or any(
        part in {"", ".", ".."} for part in path.parts
    ):
        raise CoordinatorError(code, "Validation source path must be a safe relative path")
    return path.as_posix()


@dataclass(frozen=True, slots=True)
class ExternalGitSource:
    repo_root: Path
    commit: str
    mount_path: str
    include_roots: tuple[str, ...]
    source_hash: str

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
            or not set(payload) <= required | {"sourceHash"}
        ):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source requires repoRoot, commit, mountPath, and includeRoots",
            )
        repo_root = Path(str(payload["repoRoot"])).resolve()
        commit = str(payload["commit"]).strip()
        mount_path = _relative_path(
            payload["mountPath"], code="validation_copy_external_mount_escape"
        )
        raw_roots = payload["includeRoots"]
        if not isinstance(raw_roots, Sequence) or isinstance(raw_roots, (str, bytes)):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git includeRoots must be a non-empty list",
            )
        include_roots = tuple(
            sorted(
                {
                    _relative_path(
                        root, code="validation_copy_external_include_root_invalid"
                    )
                    for root in raw_roots
                },
                key=str.casefold,
            )
        )
        if not include_roots:
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source must declare include roots",
            )
        source_hash = hashlib.sha256(
            json.dumps(
                {
                    "repoRoot": str(repo_root),
                    "commit": commit,
                    "mountPath": mount_path,
                    "includeRoots": include_roots,
                },
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()
        return cls(repo_root, commit, mount_path, include_roots, source_hash)

    def pinned(self) -> "ExternalGitSource":
        if not self.repo_root.is_dir():
            raise CoordinatorError(
                "validation_copy_external_repository_missing",
                "External Git repository root does not exist",
                details={"repoRoot": str(self.repo_root)},
            )
        result = subprocess.run(
            ["git", "rev-parse", "--verify", f"{self.commit}^{{commit}}"],
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
        return ExternalGitSource.from_payload(
            {
                "repoRoot": str(self.repo_root),
                "commit": result.stdout.strip(),
                "mountPath": self.mount_path,
                "includeRoots": list(self.include_roots),
            }
        )

    def to_payload(self) -> dict[str, object]:
        return {
            "repoRoot": str(self.repo_root),
            "commit": self.commit,
            "mountPath": self.mount_path,
            "includeRoots": list(self.include_roots),
            "sourceHash": self.source_hash,
        }


def external_tree_paths(source: ExternalGitSource) -> frozenset[str]:
    result = subprocess.run(
        ["git", "ls-tree", "-r", "--name-only", source.commit],
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
