from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import tomllib
from pathlib import Path, PurePosixPath
from typing import Callable, Mapping

from .models import CoordinatorError
from .portable_paths import normalize_portable_relative_path, portable_path_key
from .cargo_storage import prepare_isolated_cargo_home
from .trusted_tools import (
    bind_trusted_cargo,
    bind_trusted_rust_environment,
    trusted_git_command,
)
from .cargo_command_policy import (
    cargo_config_file_arguments,
    cargo_package_specs,
    rewrite_cargo_source_path_arguments,
    scrub_inherited_cargo_environment,
    validate_no_ambient_cargo_configs,
)
from .validation_copies import CargoInputClosurePlanner
from .validation_copy_external import (
    EXTERNAL_REPOSITORY_ROOT,
    ExternalGitSource,
    extract_external_archive,
)


_CARGO_CONTENT_NAMES = frozenset(
    {"Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml"}
)
_CARGO_CONFIG_PATHS = frozenset({".cargo/config", ".cargo/config.toml"})
_EXPLICIT_TARGET_TABLES = ("bin", "example", "test", "bench")
_MetadataExecutor = Callable[[Path, tuple[str, ...]], Mapping[str, object]]
_ArchiveLoader = Callable[[str], bytes]


class PinnedCargoPlannerView:
    """Create a small Cargo topology view from immutable Git objects."""

    def __init__(
        self,
        repo_root: str | Path,
        planner_parent: str | Path,
        *,
        baseline_commit: str,
        overlay_files: Mapping[str, bytes | None] | None = None,
        external_sources: tuple[ExternalGitSource, ...] | list[ExternalGitSource] = (),
        discover_external_sources: bool = False,
        external_archive_loader: _ArchiveLoader | None = None,
    ) -> None:
        self.logical_repo_root = Path(repo_root).resolve()
        self.planner_parent = Path(planner_parent).resolve()
        self.baseline_commit = _full_commit(
            self.logical_repo_root,
            baseline_commit,
            error_code="pinned_cargo_baseline_missing",
        )
        self.overlay_files = _normalized_overlay(overlay_files or {})
        self.external_sources = tuple(source.pinned() for source in external_sources)
        self.discover_external_sources = bool(discover_external_sources)
        self.external_archive_loader = external_archive_loader
        self._temporary: tempfile.TemporaryDirectory[str] | None = None
        self.root: Path | None = None
        self.repo_root: Path | None = None
        self._external_views: dict[Path, Path] = {}
        self._external_sources_by_root: dict[Path, ExternalGitSource] = {}
        self._external_sources_by_mount: dict[str, ExternalGitSource] = {}
        self._external_sources_by_view_name: dict[str, ExternalGitSource] = {}

    def __enter__(self) -> "PinnedCargoPlannerView":
        if self._temporary is not None:
            raise RuntimeError("Pinned Cargo planner view is already active")
        if not self.planner_parent.is_dir():
            raise CoordinatorError(
                "pinned_cargo_planner_parent_missing",
                "Pinned Cargo planner parent must already exist",
                details={"path": str(self.planner_parent)},
            )
        self._temporary = tempfile.TemporaryDirectory(
            prefix="cargo-planner-", dir=self.planner_parent
        )
        self.root = Path(self._temporary.name).resolve()
        # Match the final validation-copy layout exactly: the main repository is
        # mounted below ``source`` and sibling Git repositories sit beside it.
        self.repo_root = self.root / "source"
        try:
            _materialize_repository_topology(
                self.logical_repo_root,
                self.repo_root,
                self.baseline_commit,
                overlay_files=self.overlay_files,
            )
            for source in self.external_sources:
                self._materialize_external_source(source)
            if self.discover_external_sources:
                self._discover_external_views()
        except BaseException:
            self._temporary.cleanup()
            self._temporary = None
            self.root = None
            self.repo_root = None
            self._external_views.clear()
            self._external_sources_by_root.clear()
            self._external_sources_by_mount.clear()
            self._external_sources_by_view_name.clear()
            raise
        return self

    def __exit__(self, _error_type, _error, _traceback) -> None:
        if self._temporary is not None:
            self._temporary.cleanup()
        self._temporary = None
        self.root = None
        self.repo_root = None
        self._external_views.clear()
        self._external_sources_by_root.clear()
        self._external_sources_by_mount.clear()
        self._external_sources_by_view_name.clear()

    def _materialize_external_source(self, source: ExternalGitSource) -> None:
        if self.root is None:
            raise RuntimeError("Pinned Cargo planner view is not active")
        if source.repo_root.parent != self.logical_repo_root.parent:
            raise CoordinatorError(
                "pinned_cargo_external_layout_unsupported",
                "Pinned Cargo metadata supports sibling external Git repositories",
                details={"repoRoot": str(source.repo_root)},
            )
        existing = self._external_sources_by_root.get(source.repo_root)
        if existing is not None:
            if existing.commit != source.commit or existing.mount_path != source.mount_path:
                raise CoordinatorError(
                    "pinned_cargo_external_mount_conflict",
                    "Pinned Cargo metadata mapped one sibling to conflicting identities",
                    details={
                        "repoRoot": str(source.repo_root),
                        "existingCommit": existing.commit,
                        "conflictingCommit": source.commit,
                    },
                )
            return
        mount_key = source.mount_path.casefold()
        existing_mount = self._external_sources_by_mount.get(mount_key)
        if existing_mount is not None and existing_mount.repo_root != source.repo_root:
            raise CoordinatorError(
                "pinned_cargo_external_name_conflict",
                "Pinned Cargo metadata repositories require unique sibling mounts",
                details={"mountPath": source.mount_path},
            )
        view_name = source.repo_root.name
        view_name_key = view_name.casefold()
        existing_view_name = self._external_sources_by_view_name.get(view_name_key)
        if existing_view_name is not None and existing_view_name.repo_root != source.repo_root:
            raise CoordinatorError(
                "pinned_cargo_external_name_conflict",
                "Pinned Cargo metadata repositories require unique sibling names",
                details={"repoRoot": str(source.repo_root)},
            )
        if view_name_key == self.logical_repo_root.name.casefold():
            raise CoordinatorError(
                "pinned_cargo_external_name_conflict",
                "Pinned Cargo sibling cannot shadow the main repository",
                details={"repoRoot": str(source.repo_root)},
            )
        external_view = (self.root / view_name).resolve()
        if external_view == self.root or not external_view.is_relative_to(self.root):
            raise CoordinatorError(
                "pinned_cargo_external_layout_unsupported",
                "Pinned Cargo external mount escaped the planner view",
                details={"mountPath": source.mount_path},
            )
        if self.repo_root is not None and (
            external_view == self.repo_root or self.repo_root.is_relative_to(external_view)
        ):
            raise CoordinatorError(
                "pinned_cargo_external_name_conflict",
                "Pinned Cargo external mount overlaps the main repository view",
                details={"mountPath": source.mount_path},
            )
        if source.archive_hash is not None:
            if self.external_archive_loader is None:
                raise CoordinatorError(
                    "pinned_cargo_external_archive_unavailable",
                    "Pinned Cargo planner requires the sealed external archive store",
                    details={"mountPath": source.mount_path},
                )
            archive = self.external_archive_loader(source.archive_hash)
            if len(archive) != source.archive_byte_count:
                raise CoordinatorError(
                    "pinned_cargo_external_archive_corrupt",
                    "Sealed external archive byte count does not match its descriptor",
                    details={"mountPath": source.mount_path},
                )
            extract_external_archive(
                archive,
                external_view,
                error_code="pinned_cargo_external_archive_invalid",
            )
        else:
            _materialize_repository_topology(
                source.repo_root,
                external_view,
                source.commit,
                overlay_files={},
            )
        self._external_views[source.repo_root] = external_view
        self._external_sources_by_root[source.repo_root] = source
        self._external_sources_by_mount[mount_key] = source
        self._external_sources_by_view_name[view_name_key] = source

    def _discover_external_views(self) -> None:
        """Pin sibling path dependencies before Cargo metadata is executed."""
        if self.root is None or self.repo_root is None:
            raise RuntimeError("Pinned Cargo planner view is not active")
        pending = [self.repo_root, *self._external_views.values()]
        scanned: set[Path] = set()
        while pending:
            view_root = pending.pop()
            if view_root in scanned:
                continue
            scanned.add(view_root)
            for manifest in _view_manifests(view_root):
                for raw_path in _manifest_path_values(manifest):
                    dependency = _cargo_dependency_manifest(manifest, raw_path)
                    known_views = (
                        self.repo_root,
                        *self._external_views.values(),
                    )
                    if any(dependency.is_relative_to(known) for known in known_views):
                        continue
                    logical_root, view_name = self._logical_external_root(dependency)
                    source = self._external_sources_by_root.get(logical_root)
                    if source is None:
                        commit = _full_commit(
                            logical_root,
                            "HEAD",
                            error_code="pinned_cargo_external_commit_missing",
                        )
                        relative_manifest = dependency.relative_to(
                            self.root / view_name
                        )
                        include_root = relative_manifest.parent.as_posix()
                        if include_root in {"", "."}:
                            include_root = EXTERNAL_REPOSITORY_ROOT
                        source = ExternalGitSource.from_payload(
                            {
                                "repoRoot": str(logical_root),
                                "commit": commit,
                                "mountPath": logical_root.name,
                                "includeRoots": [include_root],
                            }
                        )
                        self._materialize_external_source(source)
                        self.external_sources = tuple(
                            sorted(
                                (*self.external_sources, source),
                                key=lambda item: item.mount_path.casefold(),
                            )
                        )
                        pending.append(self._external_views[logical_root])
                    elif self._external_views.get(logical_root) is not None:
                        pending.append(self._external_views[logical_root])

    def _logical_external_root(self, dependency: Path) -> tuple[Path, str]:
        if self.root is None:
            raise RuntimeError("Pinned Cargo planner view is not active")
        for logical_root, external_view in self._external_views.items():
            if dependency.is_relative_to(external_view):
                return logical_root, self._external_sources_by_root[
                    logical_root
                ].mount_path
        try:
            relative = dependency.relative_to(self.root)
        except ValueError as error:
            raise CoordinatorError(
                "pinned_cargo_external_layout_unsupported",
                "Pinned Cargo path dependency escaped the planner view",
                details={"path": str(dependency)},
            ) from error
        if not relative.parts:
            raise CoordinatorError(
                "pinned_cargo_external_layout_unsupported",
                "Pinned Cargo path dependency escaped the planner view",
                details={"path": str(dependency)},
            )
        view_name = PurePosixPath(relative.parts[0]).as_posix()
        explicit = self._external_sources_by_view_name.get(view_name.casefold())
        if explicit is not None:
            return explicit.repo_root, view_name
        logical_candidate = self.logical_repo_root.parent.joinpath(*relative.parts)
        logical_sibling = self.logical_repo_root.parent / view_name
        if not logical_sibling.is_dir():
            raise CoordinatorError(
                "pinned_cargo_external_source_missing",
                "Pinned Cargo path dependency has no discoverable sibling repository",
                details={"path": str(logical_candidate)},
            )
        result = subprocess.run(
            trusted_git_command(
                self.logical_repo_root, "rev-parse", "--show-toplevel"
            ),
            cwd=logical_sibling,
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0 or not result.stdout.strip():
            raise CoordinatorError(
                "pinned_cargo_external_source_missing",
                "Pinned Cargo path dependency is not in a discoverable Git repository",
                details={"path": str(logical_candidate)},
            )
        logical_root = Path(result.stdout.strip()).resolve()
        if (
            logical_root == self.logical_repo_root
            or logical_root.parent != self.logical_repo_root.parent
            or not logical_candidate.is_relative_to(logical_root)
        ):
            raise CoordinatorError(
                "pinned_cargo_external_layout_unsupported",
                "Automatic Cargo source discovery is restricted to sibling Git repositories",
                details={"path": str(logical_candidate), "repoRoot": str(logical_root)},
            )
        return logical_root, view_name

    def require_active_repo_root(self) -> Path:
        if self.repo_root is None or self.root is None:
            raise RuntimeError("Pinned Cargo planner view is not active")
        return self.repo_root

    def view_path_for_logical(self, path: Path) -> Path:
        normalized = path.resolve()
        if normalized.is_relative_to(self.logical_repo_root):
            return self.require_active_repo_root() / normalized.relative_to(
                self.logical_repo_root
            )
        for logical_root, view_root in self._external_views.items():
            if normalized.is_relative_to(logical_root):
                return view_root / normalized.relative_to(logical_root)
        raise CoordinatorError(
            "pinned_cargo_logical_path_unavailable",
            "Cargo topology path is outside the pinned planner repositories",
            details={"path": str(path)},
        )

    def ensure_main_view_file(self, relative_path: str) -> Path:
        relative = _safe_relative(relative_path)
        destination = self.require_active_repo_root().joinpath(
            *PurePosixPath(relative).parts
        )
        if destination.is_file():
            return destination
        if relative in self.overlay_files:
            raise CoordinatorError(
                "pinned_cargo_config_missing",
                "Explicit Cargo config was deleted by the sealed overlay",
                details={"path": relative},
            )
        entries = _git_tree_entries(self.logical_repo_root, self.baseline_commit)
        entry = entries.get(relative)
        if entry is None or entry[0] not in {"100644", "100755"}:
            raise CoordinatorError(
                "pinned_cargo_config_missing",
                "Explicit Cargo config is unavailable in the pinned baseline",
                details={"path": relative},
            )
        _write_git_blobs(
            self.logical_repo_root,
            self.require_active_repo_root(),
            entries,
            {relative},
        )
        return destination

    def logical_path_for_view(self, path: Path, *, preserve_main: bool) -> Path:
        normalized = path.resolve()
        main_view = self.require_active_repo_root()
        if normalized.is_relative_to(main_view):
            if preserve_main:
                return normalized
            return self.logical_repo_root / normalized.relative_to(main_view)
        for logical_root, view_root in self._external_views.items():
            if normalized.is_relative_to(view_root):
                return logical_root / normalized.relative_to(view_root)
        return normalized


class PinnedCargoInputClosurePlanner(CargoInputClosurePlanner):
    """Run Cargo topology discovery against a pinned planner view."""

    def __init__(
        self,
        view: PinnedCargoPlannerView,
        *,
        metadata_executor: _MetadataExecutor | None = None,
    ) -> None:
        self.view = view
        self.logical_repo_root = view.logical_repo_root
        self._metadata_executor = metadata_executor
        self._logical_planner = CargoInputClosurePlanner(
            self.logical_repo_root,
            metadata_runner=lambda _command: {},
        )
        super().__init__(view.require_active_repo_root(), metadata_runner=self._metadata)

    def _metadata(self, command: tuple[str, ...]) -> Mapping[str, object]:
        source_root = self.view.require_active_repo_root()
        pinned_command = self._command_for_view(command)
        metadata = (
            self._metadata_executor(source_root, pinned_command)
            if self._metadata_executor is not None
            else _run_cargo_metadata(
                source_root,
                pinned_command,
                trust_root=self.logical_repo_root,
            )
        )
        if not isinstance(metadata, Mapping):
            raise CoordinatorError(
                "pinned_cargo_metadata_invalid",
                "Pinned Cargo metadata executor returned a non-object payload",
            )
        # Main-repository paths remain inside the view while sibling repository
        # paths map back to their logical Git roots.  The inherited planner then
        # treats local workspace packages as repository inputs and external path
        # dependencies as explicitly pinned descriptors.
        return _rewrite_external_metadata_paths(dict(metadata), self.view)

    def _command_for_view(self, command: tuple[str, ...]) -> tuple[str, ...]:
        return rewrite_cargo_source_path_arguments(
            command,
            lambda option, value: self._path_argument_for_view(option, value),
        )

    def _path_argument_for_view(self, option: str, value: str) -> str:
        if option == "--config" and "=" in value and not Path(value).is_absolute():
            # Cargo's inline key=value form is not a filesystem path.  Storage
            # and compiler overrides are rejected at ticket submission.
            return value
        candidate = Path(value)
        normalized = value.replace("\\", "/")
        if not candidate.is_absolute():
            relative = normalize_portable_relative_path(
                normalized,
                code="cargo_source_path_argument_invalid",
                message="Cargo source path must remain inside the pinned planner view",
            )
            if option == "--config":
                self.view.ensure_main_view_file(relative)
            return relative
        mapped = self.view.view_path_for_logical(candidate)
        if option == "--config":
            try:
                relative = candidate.resolve().relative_to(
                    self.logical_repo_root
                ).as_posix()
            except ValueError as error:
                raise CoordinatorError(
                    "cargo_source_path_argument_invalid",
                    "Explicit Cargo config must be inside the pinned main repository",
                    details={"path": value},
                ) from error
            self.view.ensure_main_view_file(relative)
        return str(mapped)

    def plan(self, *args, **kwargs):
        return self.plan_pinned(*args, **kwargs)

    def plan_pinned(self, *args, **kwargs):
        # Call the base method explicitly.  Besides keeping the inherited
        # closure algorithm authoritative, this preserves the existing
        # ``CargoInputClosurePlanner.plan`` injection point used by coordinator
        # tests and downstream embedders.
        closure = CargoInputClosurePlanner.plan(self, *args, **kwargs)
        view_root = self.view.require_active_repo_root()
        repository_paths: set[str] = set()
        for relative in closure.repository_paths:
            candidate = view_root / PurePosixPath(relative)
            try:
                logical = self.view.logical_path_for_view(
                    candidate, preserve_main=False
                )
            except CoordinatorError:
                logical = candidate
            if logical.is_relative_to(self.logical_repo_root):
                repository_paths.add(
                    logical.relative_to(self.logical_repo_root).as_posix()
                )
            else:
                # External paths are represented by their mount-relative
                # names and are already consumed by the copy materializer.
                repository_paths.add(relative)
        overlay_paths = kwargs.get("overlay_paths", ())
        repository_paths.update(
            _safe_relative(path) for path in overlay_paths if isinstance(path, str)
        )
        raw_command = args[0] if args else kwargs.get("command", ())
        for raw_config in cargo_config_file_arguments(tuple(raw_command)):
            candidate = Path(raw_config)
            if candidate.is_absolute():
                try:
                    relative = candidate.resolve().relative_to(
                        self.logical_repo_root
                    )
                except ValueError as error:
                    raise CoordinatorError(
                        "cargo_source_path_argument_invalid",
                        "Explicit Cargo config must be inside the pinned repository",
                        details={"path": raw_config},
                    ) from error
                repository_paths.add(relative.as_posix())
            else:
                repository_paths.add(_safe_relative(raw_config))
        return type(closure)(
            tuple(sorted(repository_paths, key=str.casefold)),
            closure.external_sources,
        )

    def _manifest_path_dependencies(self, manifest: Path) -> tuple[Path, ...]:
        normalized = manifest.resolve()
        view_root = self.view.require_active_repo_root()
        manifest_view = (
            normalized
            if normalized.is_relative_to(view_root)
            else self.view.view_path_for_logical(normalized)
        )
        dependencies = super()._manifest_path_dependencies(manifest_view)
        return tuple(
            self.view.logical_path_for_view(path, preserve_main=True)
            for path in dependencies
        )

    def _tracked_git_paths(self, pathspecs: set[str], **kwargs) -> set[str]:
        return self._logical_planner._tracked_git_paths(pathspecs, **kwargs)

    def _baseline_compile_time_resources_by_source(
        self,
        baseline_commit: str,
        sources: set[str],
        package_roots: set[str],
        selected_package_roots: set[str],
    ) -> dict[str, tuple[str, ...]]:
        return self._logical_planner._baseline_compile_time_resources_by_source(
            baseline_commit,
            sources,
            package_roots,
            selected_package_roots,
        )

    def _external_git_root(self, manifest: Path) -> Path:
        logical_manifest = self.view.logical_path_for_view(
            manifest, preserve_main=False
        )
        return self._logical_planner._external_git_root(logical_manifest)

    def _discovered_sibling_source(
        self, external_root: Path, package_roots: set[str]
    ) -> ExternalGitSource:
        source = self.view._external_sources_by_root.get(external_root.resolve())
        if source is None:
            raise CoordinatorError(
                "pinned_cargo_external_source_unpinned",
                "Cargo metadata found a sibling source outside the pinned planner view",
                details={"repoRoot": str(external_root)},
            )
        return source


def _view_manifests(view_root: Path) -> tuple[Path, ...]:
    return tuple(
        sorted(
            (path.resolve() for path in view_root.rglob("Cargo.toml") if path.is_file()),
            key=lambda path: str(path).casefold(),
        )
    )


def _manifest_path_values(manifest: Path) -> tuple[str, ...]:
    try:
        document = tomllib.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise CoordinatorError(
            "pinned_cargo_manifest_invalid",
            "Pinned Cargo manifest could not be parsed for path dependencies",
            details={"path": str(manifest)},
        ) from error
    paths: set[str] = set()

    def collect(specifications: Mapping[str, object]) -> None:
        for specification in specifications.values():
            if not isinstance(specification, Mapping):
                continue
            dependency_path = specification.get("path")
            if isinstance(dependency_path, str) and dependency_path:
                paths.add(dependency_path)

    def visit(node: Mapping[str, object]) -> None:
        for key, value in node.items():
            if not isinstance(value, Mapping):
                continue
            if key in {"dependencies", "dev-dependencies", "build-dependencies"}:
                collect(value)
                continue
            if key == "patch":
                for registry in value.values():
                    if isinstance(registry, Mapping):
                        collect(registry)
                continue
            if key == "replace":
                collect(value)
                continue
            visit(value)

    visit(document)
    return tuple(sorted(paths, key=str.casefold))


def _cargo_dependency_manifest(manifest: Path, dependency_path: str) -> Path:
    candidate = (manifest.parent / dependency_path).resolve()
    if candidate.name.casefold() != "cargo.toml":
        candidate /= "Cargo.toml"
    return candidate


def _normalized_overlay(
    overlay_files: Mapping[str, bytes | None],
) -> dict[str, bytes | None]:
    normalized: dict[str, bytes | None] = {}
    for raw_path, content in overlay_files.items():
        relative = _safe_relative(raw_path)
        if content is not None and not isinstance(content, bytes):
            raise CoordinatorError(
                "pinned_cargo_overlay_invalid",
                "Pinned Cargo overlay values must be bytes or tombstones",
                details={"path": relative},
            )
        normalized[relative] = content
    return normalized


def _safe_relative(value: str) -> str:
    return normalize_portable_relative_path(
        value,
        code="pinned_cargo_path_invalid",
        message="Pinned Cargo path must be a safe portable repository-relative path",
    )


def _full_commit(repo_root: Path, commit: str, *, error_code: str) -> str:
    if not isinstance(commit, str) or not commit.strip():
        raise CoordinatorError(error_code, "Pinned Cargo commit is missing")
    result = subprocess.run(
        trusted_git_command(repo_root, "rev-parse", "--verify", f"{commit.strip()}^{{commit}}"),
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise CoordinatorError(
            error_code,
            "Pinned Cargo commit is unavailable",
            details={"repoRoot": str(repo_root), "commit": commit},
        )
    return result.stdout.strip()


def _materialize_repository_topology(
    logical_root: Path,
    view_root: Path,
    commit: str,
    *,
    overlay_files: Mapping[str, bytes | None],
) -> None:
    entries = _git_tree_entries(logical_root, commit)
    symlinks = sorted(
        (path for path, (mode, _object_hash) in entries.items() if mode == "120000"),
        key=str.casefold,
    )
    if symlinks:
        raise CoordinatorError(
            "pinned_cargo_symlink_unsupported",
            "Pinned Cargo repositories cannot contain filesystem links",
            details={"paths": symlinks[:16], "count": len(symlinks)},
        )
    regular_paths = {
        path for path, (mode, _object_hash) in entries.items() if mode in {"100644", "100755"}
    }
    content_paths = {
        path
        for path in regular_paths
        if PurePosixPath(path).name in _CARGO_CONTENT_NAMES
        or path in _CARGO_CONFIG_PATHS
    }
    _write_git_blobs(logical_root, view_root, entries, content_paths)
    _apply_overlay(view_root, overlay_files)

    available_paths = (regular_paths | set(overlay_files)) - {
        path for path, content in overlay_files.items() if content is None
    }
    target_paths, referenced_paths = _cargo_manifest_topology_paths(
        view_root, available_paths
    )
    baseline_references = referenced_paths & regular_paths
    _write_git_blobs(logical_root, view_root, entries, baseline_references)
    for relative in sorted(target_paths - set(overlay_files), key=str.casefold):
        destination = view_root.joinpath(*PurePosixPath(relative).parts)
        if not destination.exists():
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(b"")


def _git_tree_entries(repo_root: Path, commit: str) -> dict[str, tuple[str, str]]:
    result = subprocess.run(
        trusted_git_command(repo_root, "ls-tree", "-rz", commit),
        cwd=repo_root,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise CoordinatorError(
            "pinned_cargo_tree_unavailable",
            "Pinned Cargo commit tree could not be enumerated",
            details={"repoRoot": str(repo_root), "commit": commit},
        )
    entries: dict[str, tuple[str, str]] = {}
    portable_entries: dict[str, str] = {}
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        metadata, separator, raw_path = raw.partition(b"\t")
        if not separator:
            continue
        parts = metadata.split()
        if len(parts) != 3 or parts[1] != b"blob":
            continue
        path = raw_path.decode("utf-8", errors="surrogateescape").replace("\\", "/")
        relative = _safe_relative(path)
        key = portable_path_key(relative)
        previous = portable_entries.get(key)
        if previous is not None:
            raise CoordinatorError(
                "pinned_cargo_tree_path_collision",
                "Pinned Cargo tree contains paths that collide on the managed filesystem",
                details={"firstPath": previous, "secondPath": relative},
            )
        portable_entries[key] = relative
        entries[relative] = (
            parts[0].decode("ascii"),
            parts[2].decode("ascii"),
        )
    return entries


def _write_git_blobs(
    repo_root: Path,
    view_root: Path,
    entries: Mapping[str, tuple[str, str]],
    paths: set[str],
) -> None:
    selected = tuple(sorted(paths, key=str.casefold))
    if not selected:
        return
    object_hashes = [entries[path][1] for path in selected]
    try:
        result = subprocess.run(
            trusted_git_command(repo_root, "cat-file", "--batch"),
            cwd=repo_root,
            input=("".join(f"{object_hash}\n" for object_hash in object_hashes)).encode(
                "ascii"
            ),
            check=True,
            capture_output=True,
        )
        cursor = 0
        for relative, expected_hash in zip(selected, object_hashes, strict=True):
            header_end = result.stdout.index(b"\n", cursor)
            header = result.stdout[cursor:header_end]
            cursor = header_end + 1
            object_hash, object_type, size_text = header.rsplit(b" ", 2)
            if object_hash.decode("ascii") != expected_hash or object_type != b"blob":
                raise ValueError("unexpected Git object header")
            size = int(size_text)
            end = cursor + size
            if result.stdout[end : end + 1] != b"\n":
                raise ValueError("truncated Git object payload")
            destination = view_root.joinpath(*PurePosixPath(relative).parts)
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(result.stdout[cursor:end])
            cursor = end + 1
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        raise CoordinatorError(
            "pinned_cargo_blob_unavailable",
            "Pinned Cargo topology files could not be read from Git",
            details={"repoRoot": str(repo_root), "pathCount": len(selected)},
        ) from error


def _apply_overlay(
    view_root: Path, overlay_files: Mapping[str, bytes | None]
) -> None:
    for relative, content in overlay_files.items():
        destination = view_root.joinpath(*PurePosixPath(relative).parts)
        if content is None:
            if destination.is_dir():
                raise CoordinatorError(
                    "pinned_cargo_overlay_invalid",
                    "Pinned Cargo overlay tombstone cannot replace a directory",
                    details={"path": relative},
                )
            destination.unlink(missing_ok=True)
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(content)


def _cargo_manifest_topology_paths(
    view_root: Path, available_paths: set[str]
) -> tuple[set[str], set[str]]:
    targets: set[str] = set()
    references: set[str] = set()
    manifests = sorted(
        (
            path
            for path in available_paths
            if PurePosixPath(path).name == "Cargo.toml"
            and (view_root.joinpath(*PurePosixPath(path).parts)).is_file()
        ),
        key=str.casefold,
    )
    for manifest_relative in manifests:
        manifest_path = view_root.joinpath(*PurePosixPath(manifest_relative).parts)
        try:
            document = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
            raise CoordinatorError(
                "pinned_cargo_manifest_invalid",
                "Pinned Cargo manifest could not be parsed",
                details={"path": manifest_relative},
            ) from error
        package = document.get("package")
        if not isinstance(package, Mapping):
            continue
        package_root = PurePosixPath(manifest_relative).parent

        def add_if_available(relative: PurePosixPath) -> None:
            normalized = relative.as_posix()
            if normalized in available_paths:
                targets.add(normalized)

        lib = document.get("lib")
        if isinstance(lib, Mapping) and isinstance(lib.get("path"), str):
            add_if_available(package_root / _safe_manifest_path(lib["path"]))
        elif package.get("autolib") is not False:
            add_if_available(package_root / "src/lib.rs")

        for table_name in _EXPLICIT_TARGET_TABLES:
            entries = document.get(table_name)
            if not isinstance(entries, list):
                continue
            for entry in entries:
                if isinstance(entry, Mapping) and isinstance(entry.get("path"), str):
                    add_if_available(package_root / _safe_manifest_path(entry["path"]))

        if package.get("autobins") is not False:
            add_if_available(package_root / "src/main.rs")
            targets.update(
                _automatic_targets(available_paths, package_root, "src/bin")
            )
        for flag, directory in (
            ("autoexamples", "examples"),
            ("autotests", "tests"),
            ("autobenches", "benches"),
        ):
            if package.get(flag) is not False:
                targets.update(
                    _automatic_targets(available_paths, package_root, directory)
                )

        build = package.get("build")
        if isinstance(build, str):
            add_if_available(package_root / _safe_manifest_path(build))
        elif build is not False:
            add_if_available(package_root / "build.rs")
        for field in ("readme", "license-file"):
            value = package.get(field)
            if isinstance(value, str):
                candidate = (package_root / _safe_manifest_path(value)).as_posix()
                if candidate in available_paths:
                    references.add(candidate)
    return targets, references


def _safe_manifest_path(value: str) -> PurePosixPath:
    return PurePosixPath(
        normalize_portable_relative_path(
            value,
            code="pinned_cargo_manifest_path_invalid",
            message="Pinned Cargo manifest target path must stay inside its package",
        )
    )


def _automatic_targets(
    available_paths: set[str], package_root: PurePosixPath, directory: str
) -> set[str]:
    prefix = (package_root / directory).as_posix().rstrip("/") + "/"
    targets: set[str] = set()
    for path in available_paths:
        if not path.startswith(prefix) or not path.endswith(".rs"):
            continue
        remainder = path[len(prefix) :]
        if "/" not in remainder or remainder.endswith("/main.rs"):
            targets.add(path)
    return targets


def _run_cargo_metadata(
    source_root: Path,
    command: tuple[str, ...],
    *,
    trust_root: Path | None = None,
) -> Mapping[str, object]:
    cargo_index = next(
        (
            index
            for index, part in enumerate(command)
            if Path(part).name.casefold() in {"cargo", "cargo.exe"}
        ),
        None,
    )
    prefix = list(command[: cargo_index + 1]) if cargo_index is not None else ["cargo"]
    argument_index = (cargo_index + 1) if cargo_index is not None else 1
    if (
        cargo_index is not None
        and argument_index < len(command)
        and command[argument_index].startswith("+")
    ):
        prefix.append(command[argument_index])
        argument_index += 1
    global_arguments: list[str] = []
    metadata_arguments: list[str] = []
    passthrough_with_value = {"--filter-platform", "--manifest-path"}
    metadata_flags = {
        "--all-features",
        "--no-default-features",
    }
    global_with_value = {"--config"}
    global_flags = {
        "--locked",
        "--offline",
        "--frozen",
    }
    selected_packages = cargo_package_specs(command)
    index = argument_index
    while index < len(command):
        part = command[index]
        if part == "--":
            break
        if part in global_flags:
            global_arguments.append(part)
            index += 1
            continue
        if part in global_with_value and index + 1 < len(command):
            global_arguments.extend((part, command[index + 1]))
            index += 2
            continue
        if any(part.startswith(f"{flag}=") for flag in global_with_value):
            global_arguments.append(part)
            index += 1
            continue
        if part in metadata_flags:
            metadata_arguments.append(part)
            index += 1
            continue
        if part in {"--features", "-F"} and index + 1 < len(command):
            metadata_arguments.extend(
                (
                    part,
                    _metadata_feature_argument(
                        command[index + 1], selected_packages
                    ),
                )
            )
            index += 2
            continue
        if part.startswith(("--features=", "-F=")):
            option, _, raw_features = part.partition("=")
            metadata_arguments.append(
                option
                + "="
                + _metadata_feature_argument(raw_features, selected_packages)
            )
            index += 1
            continue
        if part in passthrough_with_value and index + 1 < len(command):
            metadata_arguments.extend((part, command[index + 1]))
            index += 2
            continue
        if any(part.startswith(f"{flag}=") for flag in passthrough_with_value):
            metadata_arguments.append(part)
        index += 1
    metadata_command = [
        *prefix,
        *global_arguments,
        "metadata",
        "--format-version",
        "1",
        *metadata_arguments,
    ]
    tool_trust_root = (trust_root or source_root).resolve()
    metadata_command = list(
        bind_trusted_cargo(
            tuple(metadata_command),
            tool_trust_root,
            working_directory=source_root,
        )
    )
    if "--locked" not in global_arguments and "--frozen" not in global_arguments:
        raise CoordinatorError(
            "pinned_cargo_lock_required",
            "Pinned Cargo metadata requires --locked or --frozen resolution",
        )

    validate_no_ambient_cargo_configs(source_root)
    environment = scrub_inherited_cargo_environment(os.environ)
    environment["CARGO_INCREMENTAL"] = "0"
    environment["CARGO_TARGET_DIR"] = str(source_root.parent / "metadata-target")
    try:
        environment["CARGO_HOME"] = str(
            prepare_isolated_cargo_home(
                source_root,
                source_root.parent / "metadata-cargo-home",
            )
        )
    except OSError as error:
        raise CoordinatorError(
            "pinned_cargo_home_isolation_failed",
            "Pinned Cargo metadata could not create an isolated configuration home",
            details={"errorType": type(error).__name__},
        ) from error
    environment = bind_trusted_rust_environment(
        environment,
        tuple(metadata_command),
        tool_trust_root,
        working_directory=source_root,
    )
    try:
        result = subprocess.run(
            metadata_command,
            cwd=source_root,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as error:
        raise CoordinatorError(
            "pinned_cargo_metadata_failed",
            "Pinned Cargo metadata could not start",
            details={"errorType": type(error).__name__},
        ) from error
    if result.returncode != 0:
        raise CoordinatorError(
            "pinned_cargo_metadata_failed",
            "Pinned Cargo metadata failed before validation-copy materialization",
            details={
                "exitCode": int(result.returncode),
                "stderr": result.stderr[-4096:],
            },
        )
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise CoordinatorError(
            "pinned_cargo_metadata_invalid",
            "Pinned Cargo metadata returned invalid JSON",
        ) from error
    if not isinstance(payload, Mapping):
        raise CoordinatorError(
            "pinned_cargo_metadata_invalid",
            "Pinned Cargo metadata returned a non-object payload",
        )
    return payload


def _metadata_feature_argument(
    raw_features: str, selected_packages: tuple[str, ...]
) -> str:
    features = tuple(
        feature
        for feature in re.split(r"[\s,]+", raw_features.strip())
        if feature
    )
    if not selected_packages:
        return ",".join(features)
    qualified: list[str] = []
    for feature in features:
        if "/" in feature:
            qualified.append(feature)
            continue
        qualified.extend(f"{package}/{feature}" for package in selected_packages)
    return ",".join(qualified)


def _rewrite_external_metadata_paths(
    metadata: dict[str, object], view: PinnedCargoPlannerView
) -> dict[str, object]:
    packages = metadata.get("packages")
    if not isinstance(packages, list):
        return metadata
    rewritten_packages: list[object] = []
    for package in packages:
        if not isinstance(package, Mapping):
            rewritten_packages.append(package)
            continue
        rewritten = dict(package)
        manifest_path = rewritten.get("manifest_path")
        if isinstance(manifest_path, str):
            rewritten["manifest_path"] = str(
                view.logical_path_for_view(Path(manifest_path), preserve_main=True)
            )
        targets = rewritten.get("targets")
        if isinstance(targets, list):
            rewritten_targets: list[object] = []
            for target in targets:
                if not isinstance(target, Mapping):
                    rewritten_targets.append(target)
                    continue
                rewritten_target = dict(target)
                source_path = rewritten_target.get("src_path")
                if isinstance(source_path, str):
                    rewritten_target["src_path"] = str(
                        view.logical_path_for_view(
                            Path(source_path), preserve_main=True
                        )
                    )
                rewritten_targets.append(rewritten_target)
            rewritten["targets"] = rewritten_targets
        rewritten_packages.append(rewritten)
    metadata["packages"] = rewritten_packages
    return metadata
