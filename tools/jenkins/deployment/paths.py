from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

from .spec import DeploymentSpec, JenkinsError
from ..resources.paths import canonical_build_root, build_namespace


@dataclass(frozen=True)
class DeploymentPaths:
    repo: Path
    root: Path
    home: Path
    runtime: Path
    logs: Path
    state: Path
    cache: Path
    tmp: Path
    agent: Path
    build_root: Path
    build_namespace: Path


def _physical(path: Path) -> Path:
    return Path(os.path.realpath(path))


def _assert_plain(path: Path, *, allow_missing: bool = True) -> None:
    current = path
    while current != current.parent:
        try:
            st = current.lstat()
            attrs = int(getattr(st, "st_file_attributes", 0))
        except FileNotFoundError:
            st = None; attrs = 0
        if st is not None and (attrs & 0x400 or current.is_symlink()):
            raise JenkinsError("reparse_path", "reparse points are not allowed in managed paths", details={"path": str(current)})
        current = current.parent
    if not allow_missing and not path.exists():
        raise JenkinsError("missing_path", "managed path does not exist", details={"path": str(path)})


def resolve_paths(spec: DeploymentSpec, build_root: str | Path | None = None) -> DeploymentPaths:
    repo = spec.repository_root
    root = repo / ".jenkins"
    home = root / "jenkins_home"
    selected_raw = str(build_root) if build_root is not None else str(spec.build_root)
    normalized = lambda value: str(value).replace("/", "\\").casefold()
    allowed_raw = {normalized(x) for x in spec.storage["allowedPhysicalRoots"]}
    if normalized(selected_raw) not in allowed_raw:
        raise JenkinsError("invalid_build_root", "buildRoot is outside the approved physical roots", details={"buildRoot": selected_raw})
    # Admit original strings before Path normalization so aliases cannot disappear.
    approved = canonical_build_root(selected_raw, repository_root=repo)
    selected = approved.path
    allowed = {str(_physical(Path(x))).casefold() for x in spec.storage["allowedPhysicalRoots"]}
    if str(selected).casefold() not in allowed:
        raise JenkinsError("physical_identity_mismatch", "buildRoot physical identity differs from its approved root")
    for p in (repo, root, home, root / "runtime", root / "logs", root / "state", root / "cache", root / "tmp", root / "agent", selected):
        _assert_plain(p)
    namespace = build_namespace(approved)
    return DeploymentPaths(repo, root, home, root / "runtime", root / "logs", root / "state", root / "cache", root / "tmp", root / "agent", selected, namespace)
