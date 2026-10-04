from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from tools.jenkins.contracts import JenkinsError


@dataclass(frozen=True)
class DeploymentSpec:
    raw: dict[str, Any]
    path: Path

    @property
    def repository_root(self) -> Path:
        raw = Path(str(self.raw["repositoryRoot"]))
        if not raw.is_absolute():
            raise JenkinsError("invalid_repository_root", "repositoryRoot must be absolute")
        # Check the lexical path before resolve(): resolving first would allow a
        # junction/symlink to masquerade as the approved repository root.
        current = raw
        while True:
            try:
                st = current.lstat()
                attrs = int(getattr(st, "st_file_attributes", 0))
                if current.is_symlink() or attrs & 0x400:
                    raise JenkinsError("repository_root_alias", "repositoryRoot contains a reparse point")
            except FileNotFoundError:
                pass
            if current == current.parent:
                break
            current = current.parent
        resolved = raw.resolve(strict=False)
        if resolved != raw:
            raise JenkinsError("repository_root_alias", "repositoryRoot resolves through an alias")
        return raw

    @property
    def controller(self) -> dict[str, Any]: return self.raw["controller"]

    @property
    def agent(self) -> dict[str, Any]: return self.raw["agent"]

    @property
    def storage(self) -> dict[str, Any]: return self.raw["storage"]

    @property
    def build_root(self) -> Path:
        return Path(self.storage.get("buildRoot", self.repository_root / '.jenkins/builds'))


def load_spec(path: str | Path) -> DeploymentSpec:
    path = Path(path).resolve()
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise JenkinsError("invalid_deployment_spec", "cannot read deployment specification", details={"path": str(path)}) from exc
    _validate(raw, path)
    return DeploymentSpec(raw, path)


def _validate(raw: dict[str, Any], path: Path) -> None:
    required = ("schemaVersion", "repositoryRoot", "controller", "agent", "storage")
    if not isinstance(raw, dict) or any(k not in raw for k in required):
        raise JenkinsError("invalid_deployment_spec", "required deployment fields are missing", details={"path": str(path)})
    controller = raw["controller"]
    if controller.get("version") != "2.580.1" or controller.get("java", {}).get("version") != "21.0.12.1":
        raise JenkinsError("unsupported_runtime", "controller or Java version is outside the locked deployment", details={"path": str(path)})
    if controller.get("executors") != 0 or controller.get("listenAddress") not in {"127.0.0.1", "localhost"}:
        raise JenkinsError("invalid_controller_policy", "controller must be loopback and have zero executors")
    home = str(controller.get("jenkinsHome", ""))
    if not home.replace("/", "\\").lower().endswith(r"\.jenkins\jenkins_home"):
        raise JenkinsError("invalid_home", "Jenkins Home must be repository-local .jenkins\\jenkins_home")
    roots = raw["storage"].get("allowedPhysicalRoots", [])
    canonical_roots = {r"d:\cargo-targets", r"e:\cargo-targets", r"f:\cargo-targets"}
    repository_build_root = str(DeploymentSpec(raw, path).repository_root / '.jenkins/builds').replace('/', '\\').lower()
    canonical_roots.add(repository_build_root)
    if not roots or any(str(root).replace("/", "\\").lower() not in canonical_roots for root in roots):
        raise JenkinsError("invalid_build_roots", "allowed physical roots must match the exact repository build root or a historical store")
