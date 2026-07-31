from __future__ import annotations

import hashlib
import json
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Callable, Mapping, Sequence

from .models import CoordinatorError


def _relative_path(value: object, *, code: str) -> str:
    if not isinstance(value, str):
        raise CoordinatorError(code, "Validation source path must be text")
    normalized = value.strip().replace("\\", "/").strip("/")
    path = PurePosixPath(normalized)
    if not normalized or path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
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
        commit = result.stdout.strip()
        payload = {
            "repoRoot": str(self.repo_root),
            "commit": commit,
            "mountPath": self.mount_path,
            "includeRoots": list(self.include_roots),
        }
        return ExternalGitSource.from_payload(payload)

    def to_payload(self) -> dict[str, object]:
        return {
            "repoRoot": str(self.repo_root),
            "commit": self.commit,
            "mountPath": self.mount_path,
            "includeRoots": list(self.include_roots),
            "sourceHash": self.source_hash,
        }


@dataclass(frozen=True, slots=True)
class CargoInputClosure:
    repository_paths: tuple[str, ...]
    external_sources: tuple[ExternalGitSource, ...]


class CargoInputClosurePlanner:
    """Derive local Cargo inputs from server-owned metadata and Git trees."""

    def __init__(
        self,
        repo_root: str | Path,
        *,
        metadata_runner: Callable[[tuple[str, ...]], Mapping[str, object]] | None = None,
    ) -> None:
        self.repo_root = Path(repo_root).resolve()
        self.metadata_runner = metadata_runner or self._cargo_metadata

    def plan(
        self,
        command: tuple[str, ...] | list[str],
        *,
        external_sources: tuple[ExternalGitSource, ...] | list[ExternalGitSource] = (),
        discover_external_sources: bool = False,
    ) -> CargoInputClosure:
        command_tuple = tuple(str(part) for part in command if str(part))
        package_name = self._package_name(command_tuple)
        metadata = self.metadata_runner(command_tuple)
        packages = {
            str(item["id"]): item
            for item in metadata.get("packages", [])
            if isinstance(item, Mapping) and "id" in item
        }
        selected = [
            package_id
            for package_id, package in packages.items()
            if package_name is None or package.get("name") == package_name
        ]
        if not selected:
            raise CoordinatorError(
                "validation_copy_cargo_target_missing",
                "Cargo metadata did not contain the requested package",
                details={"package": package_name},
            )
        dependency_ids = {
            str(node["id"]): tuple(
                str(dep["pkg"])
                for dep in node.get("deps", [])
                if isinstance(dep, Mapping) and "pkg" in dep
            )
            for node in (metadata.get("resolve") or {}).get("nodes", [])
            if isinstance(node, Mapping) and "id" in node
        }
        workspace_members = {
            str(package_id)
            for package_id in metadata.get("workspace_members", [])
            if str(package_id) in packages
        }
        queue = list(set(selected) | workspace_members)
        closure_ids: set[str] = set()
        while queue:
            package_id = queue.pop()
            if package_id in closure_ids:
                continue
            closure_ids.add(package_id)
            queue.extend(dependency_ids.get(package_id, ()))

        descriptors = tuple(source.pinned() for source in external_sources)
        repository_roots: set[str] = set()
        used_external: dict[str, ExternalGitSource] = {}
        discovered_roots: dict[Path, set[str]] = {}
        for package_id in closure_ids:
            package = packages.get(package_id)
            if package is None:
                continue
            manifest = Path(str(package["manifest_path"])).resolve()
            if manifest.is_relative_to(self.repo_root):
                relative_root = manifest.parent.relative_to(self.repo_root).as_posix()
                repository_roots.add(relative_root or ".")
                continue
            if package.get("source") is not None:
                # Registry and Git packages are fetched by Cargo; only source-null
                # local path packages require validation-copy external mounts.
                continue
            descriptor = next(
                (
                    source
                    for source in descriptors
                    if manifest.is_relative_to(source.repo_root)
                    and any(
                        manifest.relative_to(source.repo_root).as_posix() == root
                        or manifest.relative_to(source.repo_root).as_posix().startswith(root + "/")
                        for root in source.include_roots
                    )
                ),
                None,
            )
            if descriptor is None:
                if discover_external_sources:
                    external_root = self._external_git_root(manifest)
                    relative_root = manifest.parent.relative_to(external_root).as_posix()
                    if relative_root in {"", "."}:
                        raise CoordinatorError(
                            "validation_copy_external_source_layout_unsupported",
                            "Discovered external Cargo package must be below its sibling Git root",
                            details={"manifestPath": str(manifest)},
                        )
                    discovered_roots.setdefault(external_root, set()).add(relative_root)
                    continue
                raise CoordinatorError(
                    "validation_copy_external_source_missing",
                    "Cargo local path dependency has no pinned external source descriptor",
                    details={"manifestPath": str(manifest)},
                )
            used_external[descriptor.mount_path.casefold()] = descriptor

        root_manifest = self.repo_root / "Cargo.toml"
        manifest_queue = [root_manifest] if root_manifest.is_file() else []
        manifest_queue.extend(
            Path(str(packages[package_id]["manifest_path"])).resolve()
            for package_id in closure_ids
            if package_id in packages
            and Path(str(packages[package_id]["manifest_path"]))
            .resolve()
            .is_relative_to(self.repo_root)
        )
        scanned_manifests: set[Path] = set()
        while manifest_queue:
            manifest = manifest_queue.pop()
            if manifest in scanned_manifests:
                continue
            scanned_manifests.add(manifest)
            for dependency_manifest in self._manifest_path_dependencies(manifest):
                if dependency_manifest.is_relative_to(self.repo_root):
                    relative_root = dependency_manifest.parent.relative_to(
                        self.repo_root
                    ).as_posix()
                    repository_roots.add(relative_root or ".")
                    manifest_queue.append(dependency_manifest)
                    continue
                descriptor = next(
                    (
                        source
                        for source in descriptors
                        if dependency_manifest.is_relative_to(source.repo_root)
                        and any(
                            dependency_manifest.relative_to(
                                source.repo_root
                            ).as_posix()
                            == root
                            or dependency_manifest.relative_to(
                                source.repo_root
                            ).as_posix().startswith(root + "/")
                            for root in source.include_roots
                        )
                    ),
                    None,
                )
                if descriptor is not None:
                    used_external[descriptor.mount_path.casefold()] = descriptor
                    continue
                if not discover_external_sources:
                    raise CoordinatorError(
                        "validation_copy_external_source_missing",
                        "Cargo manifest path dependency has no pinned external source descriptor",
                        details={"manifestPath": str(dependency_manifest)},
                    )
                external_root = self._external_git_root(dependency_manifest)
                relative_root = dependency_manifest.parent.relative_to(
                    external_root
                ).as_posix()
                if relative_root in {"", "."}:
                    raise CoordinatorError(
                        "validation_copy_external_source_layout_unsupported",
                        "Discovered external Cargo package must be below its sibling Git root",
                        details={"manifestPath": str(dependency_manifest)},
                    )
                discovered_roots.setdefault(external_root, set()).add(relative_root)

        for external_root, include_roots in discovered_roots.items():
            descriptor = self._discovered_sibling_source(external_root, include_roots)
            used_external[descriptor.mount_path.casefold()] = descriptor

        roots = tuple(sorted(repository_roots, key=str.casefold))
        result = subprocess.run(
            ["git", "ls-files", "--", *roots],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            encoding="utf-8",
        )
        paths = {line for line in result.stdout.splitlines() if line}
        for root_file in ("Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml"):
            tracked = subprocess.run(
                ["git", "ls-files", "--error-unmatch", "--", root_file],
                cwd=self.repo_root,
                check=False,
                capture_output=True,
                encoding="utf-8",
            )
            if tracked.returncode == 0:
                paths.add(root_file)
        return CargoInputClosure(
            tuple(sorted(paths, key=str.casefold)),
            tuple(sorted(used_external.values(), key=lambda item: item.mount_path.casefold())),
        )

    def _manifest_path_dependencies(self, manifest: Path) -> tuple[Path, ...]:
        if not manifest.is_file():
            raise CoordinatorError(
                "validation_copy_cargo_manifest_path_missing",
                "Cargo path dependency manifest is unavailable",
                details={"manifestPath": str(manifest)},
            )
        try:
            document = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise CoordinatorError(
                "validation_copy_cargo_manifest_invalid",
                "Cargo manifest could not be inspected for local path dependencies",
                details={"manifestPath": str(manifest)},
            ) from error
        dependencies: set[Path] = set()

        def visit(node: Mapping[str, object]) -> None:
            for key, value in node.items():
                if not isinstance(value, Mapping):
                    continue
                if key in {"dependencies", "dev-dependencies", "build-dependencies"}:
                    for specification in value.values():
                        if isinstance(specification, Mapping) and specification.get("path"):
                            dependencies.add(
                                (manifest.parent / str(specification["path"])).resolve()
                                / "Cargo.toml"
                            )
                    continue
                visit(value)

        visit(document)
        return tuple(sorted(dependencies, key=lambda path: str(path).casefold()))

    def _external_git_root(self, manifest: Path) -> Path:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=manifest.parent,
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_external_source_missing",
                "Cargo local path dependency is not in a discoverable Git repository",
                details={"manifestPath": str(manifest)},
            )
        external_root = Path(result.stdout.strip()).resolve()
        if (
            not manifest.is_relative_to(external_root)
            or external_root.parent != self.repo_root.parent
            or external_root == self.repo_root
        ):
            raise CoordinatorError(
                "validation_copy_external_source_missing",
                "Automatic Cargo source discovery is restricted to sibling Git repositories",
                details={"manifestPath": str(manifest), "repoRoot": str(external_root)},
            )
        return external_root

    def _discovered_sibling_source(
        self, external_root: Path, package_roots: set[str]
    ) -> ExternalGitSource:
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=external_root,
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        if head.returncode != 0 or not head.stdout.strip():
            raise CoordinatorError(
                "validation_copy_external_source_missing",
                "Discovered sibling Git repository has no pinnable HEAD",
                details={"repoRoot": str(external_root)},
            )
        commit = head.stdout.strip()
        include_roots = set(package_roots)
        for root_file in (
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
        ):
            tracked = subprocess.run(
                ["git", "cat-file", "-e", f"{commit}:{root_file}"],
                cwd=external_root,
                check=False,
                capture_output=True,
                encoding="utf-8",
            )
            if tracked.returncode == 0:
                include_roots.add(root_file)
        return ExternalGitSource.from_payload(
            {
                "repoRoot": str(external_root),
                "commit": commit,
                "mountPath": external_root.name,
                "includeRoots": sorted(include_roots, key=str.casefold),
            }
        ).pinned()

    def _cargo_metadata(self, _command: tuple[str, ...]) -> Mapping[str, object]:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked"],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_cargo_metadata_failed",
                "Cargo metadata failed before validation-copy materialization",
                details={"stderr": result.stderr[-4096:]},
            )
        return json.loads(result.stdout)

    @staticmethod
    def _package_name(command: tuple[str, ...]) -> str | None:
        for index, part in enumerate(command):
            if part in {"-p", "--package"} and index + 1 < len(command):
                return command[index + 1]
            if part.startswith("--package="):
                return part.partition("=")[2]
        return None
