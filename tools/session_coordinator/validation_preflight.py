from __future__ import annotations

import fnmatch
import hashlib
import tempfile
import tomllib
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

from .cargo_command_policy import (
    cargo_excluded_package_specs,
    cargo_manifest_path_argument,
    cargo_selects_workspace,
)
from .cargo_target_selection import _option_values
from .models import CoordinatorError
from .pinned_cargo_planner import (
    PinnedCargoPlannerView,
    _cargo_dependency_manifest,
)
from .validation_copy_external import EXTERNAL_REPOSITORY_ROOT, ExternalGitSource


_ArchiveLoader = Callable[[str], bytes]
_RuntimeIdentity = Callable[[tuple[str, ...], Path], str]
_TARGET_OPTIONS = {
    "bin": frozenset({"--bin"}),
    "example": frozenset({"--example"}),
    "test": frozenset({"--test"}),
    "bench": frozenset({"--bench"}),
}


@dataclass(frozen=True, slots=True)
class _PackageManifest:
    path: Path
    name: str
    version: str | None
    document: Mapping[str, object]


@dataclass(frozen=True, slots=True)
class _CargoSelection:
    requested: Path
    anchor: Path
    anchor_document: Mapping[str, object]
    manifests: tuple[_PackageManifest, ...]
    workspace: Mapping[str, object] | None


def enrich_admission_error(error: CoordinatorError) -> CoordinatorError:
    """Attach an actionable admission blocker without changing error identity."""
    details = dict(error.details)
    details["phase"] = "admission"
    blockers = details.get("blockers")
    if isinstance(blockers, (list, tuple)) and blockers:
        normalized_blockers: list[dict[str, object]] = []
        for raw_blocker in blockers:
            blocker = dict(raw_blocker) if isinstance(raw_blocker, Mapping) else {}
            blocker.setdefault("code", error.code)
            blocker.setdefault("message", error.message)
            blocker.setdefault(
                "repairCondition", _repair_condition(str(blocker["code"]))
            )
            normalized_blockers.append(blocker)
        details["blockers"] = normalized_blockers
    else:
        blocker = {
            "code": error.code,
            "message": error.message,
            "repairCondition": _repair_condition(error.code),
        }
        blocker.update(
            {
                key: value
                for key, value in error.details.items()
                if key not in {"blockers", "phase"}
            }
        )
        details["blockers"] = [blocker]
    return CoordinatorError(error.code, error.message, details=details)


def preflight_pinned_cargo(
    repo_root: str | Path,
    *,
    baseline_commit: str,
    overlay_files: Mapping[str, bytes | None],
    command: tuple[str, ...],
    external_sources: Sequence[ExternalGitSource | Mapping[str, object]] = (),
    external_archive_loader: _ArchiveLoader | None = None,
    runtime_identity: _RuntimeIdentity | None = None,
    planner_parent: str | Path | None = None,
) -> str | None:
    """Reject provably invalid Cargo tickets from their immutable source snapshot."""
    try:
        sources = _sealed_external_sources(external_sources)
        loader = _verified_archive_loader(sources, external_archive_loader)
        if planner_parent is None:
            with tempfile.TemporaryDirectory(prefix="cargo-admission-") as temporary:
                return _preflight_in_parent(
                    repo_root,
                    Path(temporary),
                    baseline_commit,
                    overlay_files,
                    command,
                    sources,
                    loader,
                    runtime_identity,
                )
        return _preflight_in_parent(
            repo_root,
            Path(planner_parent),
            baseline_commit,
            overlay_files,
            command,
            sources,
            loader,
            runtime_identity,
        )
    except CoordinatorError as error:
        raise enrich_admission_error(error) from error


def _preflight_in_parent(
    repo_root: str | Path,
    planner_parent: Path,
    baseline_commit: str,
    overlay_files: Mapping[str, bytes | None],
    command: tuple[str, ...],
    external_sources: tuple[ExternalGitSource, ...],
    external_archive_loader: _ArchiveLoader | None,
    runtime_identity: _RuntimeIdentity | None,
) -> str | None:
    with PinnedCargoPlannerView(
        repo_root,
        planner_parent,
        baseline_commit=baseline_commit,
        overlay_files=overlay_files,
        external_sources=external_sources,
        external_archive_loader=external_archive_loader,
    ) as view:
        source_root = view.require_active_repo_root()
        selection = _load_cargo_selection(view, command)
        _require_lockfile(selection.anchor)
        selected = _selected_packages(selection, command)
        _require_targets(selected, command)
        _require_path_dependency_manifests(view, selection, selected)
        return runtime_identity(command, source_root) if runtime_identity else None


def _sealed_external_sources(
    sources: Sequence[ExternalGitSource | Mapping[str, object]],
) -> tuple[ExternalGitSource, ...]:
    sealed: list[ExternalGitSource] = []
    for source in sources:
        if isinstance(source, ExternalGitSource):
            canonical = ExternalGitSource.from_payload(source.to_payload())
            if canonical.source_hash != source.source_hash:
                raise CoordinatorError(
                    "validation_copy_external_source_invalid",
                    "External Git source identity does not match its descriptor",
                    details={"repoRoot": str(source.repo_root)},
                )
        elif isinstance(source, Mapping):
            canonical = ExternalGitSource.from_payload(source)
        else:
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source must be a pinned descriptor",
            )
        if canonical.archive_hash is None:
            raise CoordinatorError(
                "pinned_cargo_external_source_unpinned",
                "Cargo admission requires a coordinator-sealed external source archive",
                details={"repoRoot": str(canonical.repo_root)},
            )
        if canonical.include_roots != (EXTERNAL_REPOSITORY_ROOT,):
            raise CoordinatorError(
                "pinned_cargo_external_source_unpinned",
                "Cargo admission requires a complete coordinator-sealed external archive",
                details={
                    "repoRoot": str(canonical.repo_root),
                    "includeRoots": list(canonical.include_roots),
                },
            )
        sealed.append(canonical)
    return tuple(sealed)


def _verified_archive_loader(
    sources: tuple[ExternalGitSource, ...], loader: _ArchiveLoader | None
) -> _ArchiveLoader | None:
    if not sources:
        return None
    if loader is None:
        raise CoordinatorError(
            "pinned_cargo_external_archive_unavailable",
            "Cargo admission requires the sealed external archive store",
        )
    expected_sizes = {
        source.archive_hash: source.archive_byte_count for source in sources
    }
    cache: dict[str, bytes] = {}

    def verified(object_hash: str) -> bytes:
        cached = cache.get(object_hash)
        if cached is not None:
            return cached
        content = loader(object_hash)
        if not isinstance(content, bytes):
            raise CoordinatorError(
                "pinned_cargo_external_archive_corrupt",
                "Sealed external archive store returned non-byte content",
                details={"archiveHash": object_hash},
            )
        if (
            hashlib.sha256(content).hexdigest() != object_hash
            or len(content) != expected_sizes.get(object_hash)
        ):
            raise CoordinatorError(
                "pinned_cargo_external_archive_corrupt",
                "Sealed external archive does not match its immutable descriptor",
                details={"archiveHash": object_hash},
            )
        cache[object_hash] = content
        return content

    return verified


def _require_lockfile(anchor_manifest: Path) -> None:
    lockfile = anchor_manifest.parent / "Cargo.lock"
    if not lockfile.is_file():
        raise CoordinatorError(
            "pinned_cargo_lockfile_missing",
            "Pinned Cargo workspace does not contain its Cargo.lock",
            details={"path": str(lockfile)},
        )
    try:
        tomllib.loads(lockfile.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise CoordinatorError(
            "pinned_cargo_lockfile_invalid",
            "Pinned Cargo workspace lockfile is not parseable TOML",
            details={"path": str(lockfile)},
        ) from error


def _load_cargo_selection(
    view: PinnedCargoPlannerView, command: tuple[str, ...]
) -> _CargoSelection:
    source_root = view.require_active_repo_root()
    planner_root = view.root
    assert planner_root is not None
    explicit = cargo_manifest_path_argument(command)
    requested = (source_root / (explicit or "Cargo.toml")).resolve()
    if not requested.is_relative_to(source_root) or not requested.is_file():
        raise CoordinatorError(
            "pinned_cargo_manifest_missing",
            "Requested Cargo manifest is absent from the pinned inputs",
            details={"manifestPath": explicit or "Cargo.toml"},
        )
    requested_document = _read_manifest(requested)
    anchor, anchor_document = _workspace_anchor(
        requested, requested_document, source_root, planner_root
    )
    workspace = anchor_document.get("workspace")
    workspace_version = _workspace_package_version(anchor_document)
    candidate_paths = {requested, anchor}
    if isinstance(workspace, Mapping):
        candidate_paths.update(_workspace_member_manifests(anchor, workspace))
    packages = [
        package
        for path in sorted(candidate_paths, key=lambda item: str(item).casefold())
        if (
            package := _package_manifest(
                path, _read_manifest(path), workspace_version=workspace_version
            )
        )
        is not None
    ]
    pending = list(packages)
    seen_paths = {package.path for package in packages}
    while pending:
        package = pending.pop()
        for dependency in _cargo_dependency_manifests(
            package.path,
            package.document,
            anchor,
            anchor_document,
            inherit_workspace=True,
        ):
            if (
                dependency in seen_paths
                or not dependency.is_relative_to(anchor.parent)
                or not dependency.is_file()
            ):
                continue
            seen_paths.add(dependency)
            discovered = _package_manifest(
                dependency,
                _read_manifest(dependency),
                workspace_version=workspace_version,
            )
            if discovered is not None and not (
                isinstance(workspace, Mapping)
                and _workspace_excludes(anchor, workspace, dependency)
            ):
                packages.append(discovered)
                pending.append(discovered)
    return _CargoSelection(
        requested,
        anchor,
        anchor_document,
        tuple(packages),
        workspace if isinstance(workspace, Mapping) else None,
    )


def _read_manifest(manifest: Path) -> Mapping[str, object]:
    try:
        return tomllib.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise CoordinatorError(
            "pinned_cargo_manifest_invalid",
            "Pinned Cargo manifest could not be parsed",
            details={"path": str(manifest)},
        ) from error


def _package_manifest(
    path: Path,
    document: Mapping[str, object],
    *,
    workspace_version: str | None,
) -> _PackageManifest | None:
    package = document.get("package")
    if not isinstance(package, Mapping):
        return None
    name = package.get("name")
    if not isinstance(name, str) or not name.strip():
        return None
    version = package.get("version")
    if (
        isinstance(version, Mapping)
        and version.get("workspace") is True
        and workspace_version is not None
    ):
        version = workspace_version
    return _PackageManifest(
        path,
        name.strip(),
        version.strip() if isinstance(version, str) and version.strip() else None,
        document,
    )


def _workspace_package_version(document: Mapping[str, object]) -> str | None:
    workspace = document.get("workspace")
    if not isinstance(workspace, Mapping):
        return None
    package = workspace.get("package")
    if not isinstance(package, Mapping):
        return None
    version = package.get("version")
    return version.strip() if isinstance(version, str) and version.strip() else None


def _workspace_anchor(
    requested: Path,
    document: Mapping[str, object],
    source_root: Path,
    planner_root: Path,
) -> tuple[Path, Mapping[str, object]]:
    if isinstance(document.get("workspace"), Mapping):
        return requested, document
    package = document.get("package")
    declared = package.get("workspace") if isinstance(package, Mapping) else None
    if isinstance(declared, str) and declared.strip():
        anchor = (requested.parent / declared / "Cargo.toml").resolve()
        if not anchor.is_relative_to(planner_root) or not anchor.is_file():
            raise CoordinatorError(
                "pinned_cargo_manifest_missing",
                "Declared Cargo workspace manifest is absent from the pinned inputs",
                details={"manifestPath": str(anchor)},
            )
        anchor_document = _read_manifest(anchor)
        if isinstance(anchor_document.get("workspace"), Mapping):
            return anchor, anchor_document
        return requested, document
    candidate = requested.parent.parent
    while candidate.is_relative_to(source_root):
        anchor = candidate / "Cargo.toml"
        if anchor.is_file():
            anchor_document = _read_manifest(anchor)
            workspace = anchor_document.get("workspace")
            if isinstance(workspace, Mapping):
                if _workspace_excludes(anchor, workspace, requested):
                    return requested, document
                if _workspace_contains_manifest(anchor, workspace, requested):
                    return anchor, anchor_document
                raise CoordinatorError(
                    "pinned_cargo_workspace_member_missing",
                    "Requested Cargo package is under a workspace but is not one of its members",
                    details={
                        "manifestPath": str(requested),
                        "workspaceManifest": str(anchor),
                    },
                )
        if candidate == source_root:
            break
        candidate = candidate.parent
    return requested, document


def _workspace_excludes(
    anchor: Path, workspace: Mapping[str, object], manifest: Path
) -> bool:
    raw_excludes = workspace.get("exclude")
    if not isinstance(raw_excludes, list):
        return False
    relative = manifest.parent.relative_to(anchor.parent).as_posix()
    return any(
        fnmatch.fnmatchcase(relative, pattern.replace("\\", "/").strip("/"))
        for pattern in raw_excludes
        if isinstance(pattern, str) and pattern.strip()
    )


def _workspace_contains_manifest(
    anchor: Path, workspace: Mapping[str, object], requested: Path
) -> bool:
    candidates = _workspace_member_manifests(anchor, workspace)
    if requested in candidates:
        return True
    if requested == anchor and "package" in _read_manifest(anchor):
        return True
    pending = list(candidates | {anchor})
    visited: set[Path] = set()
    while pending:
        manifest = pending.pop()
        if manifest in visited:
            continue
        visited.add(manifest)
        document = _read_manifest(manifest)
        for dependency in _cargo_dependency_manifests(
            manifest,
            document,
            anchor,
            _read_manifest(anchor),
            inherit_workspace=True,
        ):
            if dependency == requested:
                return True
            if dependency.is_relative_to(anchor.parent) and dependency.is_file():
                pending.append(dependency)
    return False


def _workspace_member_manifests(
    anchor: Path, workspace: Mapping[str, object]
) -> set[Path]:
    members = workspace.get("members")
    if not isinstance(members, list):
        return set()
    excludes = tuple(
        value.replace("\\", "/").strip("/")
        for value in workspace.get("exclude", [])
        if isinstance(value, str) and value.strip()
    ) if isinstance(workspace.get("exclude", []), list) else ()
    root = anchor.parent
    result: set[Path] = set()
    for raw_member in members:
        if not isinstance(raw_member, str) or not raw_member.strip():
            continue
        pattern = raw_member.replace("\\", "/").strip("/")
        for manifest in root.glob(f"{pattern}/Cargo.toml"):
            relative = manifest.parent.relative_to(root).as_posix()
            if any(fnmatch.fnmatchcase(relative, exclude) for exclude in excludes):
                continue
            result.add(manifest.resolve())
    return result


def _manifest_matches_workspace_patterns(
    anchor: Path, manifest: Path, patterns: tuple[str, ...]
) -> bool:
    try:
        relative = manifest.parent.relative_to(anchor.parent).as_posix()
    except ValueError:
        return False
    return any(fnmatch.fnmatchcase(relative, pattern) for pattern in patterns)


def _selected_packages(
    selection: _CargoSelection, command: tuple[str, ...]
) -> tuple[_PackageManifest, ...]:
    requested = _requested_packages(command)
    if requested:
        selected = tuple(
            manifest
            for manifest in selection.manifests
            if any(
                manifest.name == name
                and (version is None or manifest.version == version)
                for name, version in requested
            )
        )
        missing = sorted(
            (
                f"{name}@{version}" if version is not None else name
                for name, version in requested
                if not any(
                    manifest.name == name
                    and (version is None or manifest.version == version)
                    for manifest in selected
                )
            ),
            key=str.casefold,
        )
        if missing:
            raise CoordinatorError(
                "pinned_cargo_package_missing",
                "Requested Cargo package is absent from the pinned inputs",
                details={"packages": missing},
            )
        return selected
    if cargo_selects_workspace(command):
        excluded = set(cargo_excluded_package_specs(command))
        return tuple(
            manifest
            for manifest in selection.manifests
            if manifest.name not in excluded
        )
    requested_package = next(
        (
            manifest
            for manifest in selection.manifests
            if manifest.path == selection.requested
        ),
        None,
    )
    if selection.requested != selection.anchor and requested_package is not None:
        return (requested_package,)
    if selection.workspace is None:
        return (requested_package,) if requested_package is not None else ()
    defaults = selection.workspace.get("default-members")
    if isinstance(defaults, list):
        patterns = tuple(
            value.replace("\\", "/").strip("/")
            for value in defaults
            if isinstance(value, str) and value.strip()
        )
        return tuple(
            manifest
            for manifest in selection.manifests
            if _manifest_matches_workspace_patterns(
                selection.anchor, manifest.path, patterns
            )
        )
    if requested_package is not None:
        return (requested_package,)
    return selection.manifests


def _requested_packages(
    command: tuple[str, ...],
) -> tuple[tuple[str, str | None], ...]:
    result: list[tuple[str, str | None]] = []
    for specification in _option_values(command, frozenset({"-p", "--package"})):
        name, separator, version = specification.partition("@")
        item = (name, version if separator else None)
        if item not in result:
            result.append(item)
    return tuple(result)


def _require_targets(
    manifests: tuple[_PackageManifest, ...], command: tuple[str, ...]
) -> None:
    requested = {
        kind: set(_option_values(command, options))
        for kind, options in _TARGET_OPTIONS.items()
    }
    if "--lib" in command and not any(_manifest_has_library(item) for item in manifests):
        _raise_target_missing("lib", ["lib"], manifests)
    for kind, names in requested.items():
        if not names:
            continue
        available: set[str] = set()
        for manifest in manifests:
            available.update(_manifest_target_names(manifest, kind))
        missing = sorted(names - available, key=str.casefold)
        if missing:
            _raise_target_missing(kind, missing, manifests)


def _raise_target_missing(
    kind: str, names: Sequence[str], manifests: tuple[_PackageManifest, ...]
) -> None:
    raise CoordinatorError(
        "pinned_cargo_target_missing",
        "Requested Cargo target is absent from the pinned inputs",
        details={
            "targetKind": kind,
            "targets": list(names),
            "packages": sorted({item.name for item in manifests}, key=str.casefold),
        },
    )


def _manifest_has_library(manifest: _PackageManifest) -> bool:
    package = manifest.document["package"]
    assert isinstance(package, Mapping)
    library = manifest.document.get("lib")
    if isinstance(library, Mapping):
        raw_path = library.get("path", "src/lib.rs")
        return isinstance(raw_path, str) and (manifest.path.parent / raw_path).is_file()
    return package.get("autolib") is not False and (manifest.path.parent / "src/lib.rs").is_file()


def _manifest_target_names(manifest: _PackageManifest, kind: str) -> set[str]:
    result: set[str] = set()
    package = manifest.document["package"]
    assert isinstance(package, Mapping)
    entries = manifest.document.get(kind)
    if isinstance(entries, list):
        for entry in entries:
            if not isinstance(entry, Mapping):
                continue
            name = entry.get("name")
            raw_path = entry.get("path")
            if not isinstance(name, str) or not name.strip():
                if isinstance(raw_path, str):
                    name = Path(raw_path).stem
                else:
                    continue
            default_path = _default_named_target_path(manifest.path.parent, kind, name)
            target_path = manifest.path.parent / raw_path if isinstance(raw_path, str) else default_path
            if target_path.is_file():
                result.add(name.strip())
    auto_flag = {
        "bin": "autobins",
        "example": "autoexamples",
        "test": "autotests",
        "bench": "autobenches",
    }[kind]
    if package.get(auto_flag) is False:
        return result
    if kind == "bin" and (manifest.path.parent / "src/main.rs").is_file():
        result.add(manifest.name)
    directory = manifest.path.parent / ("src/bin" if kind == "bin" else f"{kind}s")
    if directory.is_dir():
        result.update(path.stem for path in directory.glob("*.rs") if path.is_file())
        result.update(path.parent.name for path in directory.glob("*/main.rs") if path.is_file())
    return result


def _default_named_target_path(package_root: Path, kind: str, name: str) -> Path:
    directory = "src/bin" if kind == "bin" else f"{kind}s"
    return package_root / directory / f"{name}.rs"


def _require_path_dependency_manifests(
    view: PinnedCargoPlannerView,
    selection: _CargoSelection,
    selected: tuple[_PackageManifest, ...],
) -> None:
    source_root = view.require_active_repo_root()
    view_root = view.root
    assert view_root is not None
    documents = {item.path: item.document for item in selected}
    documents[selection.anchor] = selection.anchor_document
    pending = list(documents)
    visited: set[Path] = set()
    while pending:
        manifest = pending.pop()
        if manifest in visited:
            continue
        visited.add(manifest)
        document = documents.get(manifest) or _read_manifest(manifest)
        workspace_manifest, workspace_document = _manifest_workspace_context(
            manifest, document, view_root
        )
        for dependency in _cargo_dependency_manifests(
            manifest,
            document,
            workspace_manifest,
            workspace_document,
            inherit_workspace=True,
        ):
            if dependency.is_file():
                documents.setdefault(dependency, _read_manifest(dependency))
                pending.append(dependency)
                continue
            if not dependency.is_relative_to(view_root):
                raise CoordinatorError(
                    "pinned_cargo_external_layout_unsupported",
                    "Pinned Cargo path dependency escaped the immutable planner view",
                    details={"path": str(dependency)},
                )
            code = (
                "pinned_cargo_manifest_missing"
                if dependency.is_relative_to(source_root)
                else "pinned_cargo_external_source_unpinned"
            )
            raise CoordinatorError(
                code,
                "Cargo path dependency manifest is absent from the pinned inputs",
                details={"path": str(dependency)},
            )


def _manifest_workspace_context(
    manifest: Path,
    document: Mapping[str, object],
    planner_root: Path,
) -> tuple[Path, Mapping[str, object]]:
    if isinstance(document.get("workspace"), Mapping):
        return manifest, document
    package = document.get("package")
    declared = package.get("workspace") if isinstance(package, Mapping) else None
    if isinstance(declared, str) and declared.strip():
        candidate = (manifest.parent / declared / "Cargo.toml").resolve()
        if candidate.is_relative_to(planner_root) and candidate.is_file():
            candidate_document = _read_manifest(candidate)
            if isinstance(candidate_document.get("workspace"), Mapping):
                return candidate, candidate_document
        return manifest, document
    try:
        mount_name = manifest.relative_to(planner_root).parts[0]
    except (ValueError, IndexError):
        return manifest, document
    mount_root = planner_root / mount_name
    candidate = manifest.parent.parent
    while candidate.is_relative_to(mount_root):
        workspace_manifest = candidate / "Cargo.toml"
        if workspace_manifest.is_file():
            workspace_document = _read_manifest(workspace_manifest)
            workspace = workspace_document.get("workspace")
            if isinstance(workspace, Mapping) and not _workspace_excludes(
                workspace_manifest, workspace, manifest
            ):
                return workspace_manifest, workspace_document
        if candidate == mount_root:
            break
        candidate = candidate.parent
    return manifest, document


def _cargo_dependency_manifests(
    manifest: Path,
    document: Mapping[str, object],
    workspace_manifest: Path,
    workspace_document: Mapping[str, object],
    *,
    inherit_workspace: bool,
) -> tuple[Path, ...]:
    dependencies: set[Path] = set()
    inherited = (
        _workspace_dependency_specs(workspace_document) if inherit_workspace else {}
    )
    for name, specification in _cargo_dependency_specs(document):
        dependency_path = specification.get("path")
        base = manifest
        if not isinstance(dependency_path, str) or not dependency_path:
            if specification.get("workspace") is not True:
                continue
            workspace_specification = inherited.get(name)
            if not isinstance(workspace_specification, Mapping):
                continue
            dependency_path = workspace_specification.get("path")
            base = workspace_manifest
        if isinstance(dependency_path, str) and dependency_path:
            dependencies.add(_cargo_dependency_manifest(base, dependency_path))
    if manifest == workspace_manifest:
        for dependency_path in _cargo_patch_paths(document):
            dependencies.add(_cargo_dependency_manifest(manifest, dependency_path))
    return tuple(sorted(dependencies, key=lambda path: str(path).casefold()))


def _cargo_dependency_specs(
    document: Mapping[str, object],
) -> tuple[tuple[str, Mapping[str, object]], ...]:
    result: list[tuple[str, Mapping[str, object]]] = []

    def collect(node: Mapping[str, object]) -> None:
        for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
            table = node.get(table_name)
            if not isinstance(table, Mapping):
                continue
            result.extend(
                (str(name), specification)
                for name, specification in table.items()
                if isinstance(specification, Mapping)
            )

    collect(document)
    targets = document.get("target")
    if isinstance(targets, Mapping):
        for target in targets.values():
            if isinstance(target, Mapping):
                collect(target)
    return tuple(result)


def _workspace_dependency_specs(
    document: Mapping[str, object],
) -> Mapping[str, object]:
    workspace = document.get("workspace")
    if not isinstance(workspace, Mapping):
        return {}
    dependencies = workspace.get("dependencies")
    return dependencies if isinstance(dependencies, Mapping) else {}


def _cargo_patch_paths(document: Mapping[str, object]) -> tuple[str, ...]:
    paths: set[str] = set()

    def collect(table: object) -> None:
        if not isinstance(table, Mapping):
            return
        for specification in table.values():
            if not isinstance(specification, Mapping):
                continue
            value = specification.get("path")
            if isinstance(value, str) and value:
                paths.add(value)

    patches = document.get("patch")
    if isinstance(patches, Mapping):
        for registry in patches.values():
            collect(registry)
    collect(document.get("replace"))
    return tuple(sorted(paths, key=str.casefold))


def _repair_condition(code: str) -> str:
    folded = code.casefold()
    if code == "validation_ticket_external_worktree_dirty":
        return "The external repository owner must finish and commit its pending changes before resubmission can seal that exact clean revision."
    if "compile_time_resource" in folded:
        if "outside_repository" in folded:
            return "Keep every compile-time include inside the pinned repository or declare it through a sealed external source, then resubmit."
        if "unresolved" in folded:
            return "Replace the dynamic compile-time include with a repository-local static path, then resubmit."
        return "Restore the referenced compile-time resource and include its exact hash in the sealed source manifest, then resubmit."
    if "compile_time_source" in folded:
        if "too_large" in folded or "limit" in folded:
            return "Reduce the compile-time source closure below the coordinator limits or narrow the selected target, then resubmit."
        if "invalid" in folded:
            return "Replace the invalid UTF-8 compile-time source with a valid immutable source file, then resubmit."
        return "Restore the referenced compile-time source at the pinned path and resubmit."
    if "dependency_root" in folded:
        return "Declare a non-empty dependency root that stays inside the pinned repository and resubmit."
    if "workspace_member" in folded:
        return "Add the requested package to the pinned workspace members or select a package that is already a member, then resubmit."
    if "archive" in folded:
        return "Restore the sealed archive at its recorded hash and resubmit with the complete external source archive."
    if "not_owned" in folded:
        return "Obtain current Session ownership and attribute the submitted source paths before resubmitting."
    if "snapshot_stale" in folded:
        return "Recompute the source manifest from the intended current bytes and deletion tombstones, then resubmit."
    if "lock" in folded:
        return "Seal a valid Cargo.lock in the immutable ticket inputs and resubmit."
    if "manifest" in folded:
        return "Seal every requested Cargo.toml and correct invalid manifest TOML before resubmitting."
    if "package" in folded:
        return "Select a package present in the pinned workspace snapshot and resubmit."
    if "target" in folded:
        return "Select a target declared by the pinned package snapshot and resubmit."
    if "external" in folded or code in {"object_unavailable", "object_corrupt"}:
        return "Seal the complete external source archive with its fixed hash and resubmit."
    return "Correct the immutable validation inputs described by this blocker and resubmit."
