"""Capture sibling Git identities for an immutable Cargo validation ticket."""

from __future__ import annotations

import re
import hashlib
import subprocess
import tarfile
import tempfile
import tomllib
from fnmatch import fnmatchcase
from io import BytesIO
from pathlib import Path, PurePosixPath
from typing import Mapping

from .models import CoordinatorError
from .portable_paths import normalize_portable_relative_path
from .cargo_command_policy import (
    cargo_config_file_arguments,
    cargo_excluded_package_specs,
    cargo_manifest_path_argument,
    cargo_package_specs,
    cargo_selects_workspace,
)
from .validation_copy_external import EXTERNAL_REPOSITORY_ROOT, ExternalGitSource
from .trusted_tools import trusted_git_command


EXTERNAL_SOURCES_COVERAGE_KEY = "externalSources"
_FULL_GIT_COMMIT = re.compile(r"^[0-9a-f]{40}$", re.IGNORECASE)
_EXTERNAL_ARCHIVE_MAX_BYTES = 256 * 1024 * 1024
_EXTERNAL_TREE_MAX_BYTES = 512 * 1024 * 1024
_EXTERNAL_TREE_MAX_ENTRIES = 100_000


def discover_pinned_external_sources(
    repo_root: str | Path,
    *,
    baseline_commit: str,
    overlay_files: Mapping[str, bytes | None] | None = None,
    command: tuple[str, ...] | list[str] = (),
) -> tuple[dict[str, object], ...]:
    """Discover path dependencies without executing Cargo.

    The scanner follows Cargo manifest ``path`` values in the immutable baseline
    (and any sealed manifest overlays).  Every sibling repository reached by that
    graph is represented by its current HEAD commit captured at ticket submission.
    The worker later materializes exactly these descriptors; it never falls back to
    a sibling HEAD observed after a ticket has waited in the FIFO.
    """
    root = Path(repo_root).resolve()
    baseline = _resolve_commit(root, baseline_commit)
    overlays = {
        _safe_relative(path): content
        for path, content in (overlay_files or {}).items()
    }
    baseline_paths = set(_git_tree_paths(root, baseline))
    baseline_manifests = {
        path
        for path in baseline_paths
        if path == "Cargo.toml" or path.endswith("/Cargo.toml")
    }
    main_manifests = set(baseline_manifests)
    main_manifests.update(
        path
        for path, content in overlays.items()
        if content is not None
        and (path == "Cargo.toml" or path.endswith("/Cargo.toml"))
    )
    config_paths = {".cargo/config", ".cargo/config.toml"}
    config_paths.update(
        _safe_relative(path)
        for path in cargo_config_file_arguments(tuple(command))
    )
    baseline_configs = config_paths & baseline_paths
    archive_paths = baseline_manifests | baseline_configs
    baseline_contents = _git_cargo_manifests(root, baseline, archive_paths)
    for relative in sorted(config_paths, key=str.casefold):
        content = overlays.get(relative, baseline_contents.get(relative))
        if content is not None:
            _validate_cargo_config(content, root / relative)
    selected_manifests = _selected_main_manifests(
        main_manifests,
        baseline_contents,
        overlays,
        tuple(command),
    )
    explicit_manifest = cargo_manifest_path_argument(tuple(command))
    workspace_anchor = (
        _safe_relative(explicit_manifest)
        if explicit_manifest is not None
        else "Cargo.toml"
    )
    workspace_scan_manifests = selected_manifests & {workspace_anchor}
    manifest_queue: list[tuple[Path, str, bytes | None, bool]] = []
    for relative in sorted(selected_manifests, key=str.casefold):
        content = (
            overlays[relative]
            if relative in overlays
            else baseline_contents.get(relative)
        )
        if content is not None:
            manifest_queue.append((root / relative, baseline, content, False))
    if "Cargo.toml" in main_manifests and "Cargo.toml" not in selected_manifests:
        root_content = overlays.get("Cargo.toml", baseline_contents.get("Cargo.toml"))
        if root_content is not None:
            manifest_queue.append((root / "Cargo.toml", baseline, root_content, True))

    # A root can be reached through several package manifests. Keep the narrowest
    # package root for each descriptor while preserving all needed package paths.
    discovered: dict[Path, tuple[str, set[str]]] = {}
    visited: set[tuple[Path, str, str]] = set()
    while manifest_queue:
        manifest, commit, content, patches_only = manifest_queue.pop()
        relative_manifest = manifest.relative_to(root).as_posix() if manifest.is_relative_to(root) else None
        visit_key = (manifest.parent.resolve(), commit, relative_manifest or str(manifest))
        if visit_key in visited:
            continue
        visited.add(visit_key)
        for raw_path in _manifest_path_values(
            content,
            manifest,
            patches_only=patches_only,
            include_workspace_members=(
                (
                    manifest.is_relative_to(root)
                    and manifest.relative_to(root).as_posix()
                    in workspace_scan_manifests
                )
                or not manifest.is_relative_to(root)
            ),
        ):
            dependency = _cargo_dependency_manifest(manifest, raw_path)
            if dependency.is_relative_to(root):
                dependency_relative = dependency.relative_to(root).as_posix()
                if dependency_relative in main_manifests:
                    dependency_content = (
                        overlays[dependency_relative]
                        if dependency_relative in overlays
                        else baseline_contents.get(dependency_relative)
                    )
                    if dependency_content is not None:
                        manifest_queue.append(
                            (dependency, baseline, dependency_content, False)
                        )
                continue

            external_root = _external_git_root(root, dependency)
            external_commit = _git_head(external_root)
            relative = dependency.relative_to(external_root)
            include_root = relative.parent.as_posix()
            if include_root in {"", "."}:
                include_root = EXTERNAL_REPOSITORY_ROOT
            previous = discovered.get(external_root)
            if previous is None:
                previous = (external_commit, set())
                discovered[external_root] = previous
            elif previous[0] != external_commit:
                raise CoordinatorError(
                    "validation_copy_external_commit_changed",
                    "A sibling repository changed identity while external inputs were pinned",
                    details={
                        "repoRoot": str(external_root),
                        "firstCommit": previous[0],
                        "currentCommit": external_commit,
                    },
                )
            previous[1].add(include_root)
            external_content = _git_show(external_root, external_commit, relative.as_posix())
            if external_content is not None:
                manifest_queue.append(
                    (dependency, external_commit, external_content, False)
                )

    descriptors = [
        ExternalGitSource.from_payload(
            {
                "repoRoot": str(external_root),
                "commit": commit,
                "mountPath": external_root.name,
                "includeRoots": sorted(include_roots, key=str.casefold),
            }
        ).to_payload()
        for external_root, (commit, include_roots) in discovered.items()
    ]
    return tuple(sorted(descriptors, key=lambda item: str(item["mountPath"]).casefold()))


def seal_pinned_external_sources(
    descriptors: tuple[dict[str, object], ...] | list[dict[str, object]],
    *,
    max_archive_bytes: int = _EXTERNAL_ARCHIVE_MAX_BYTES,
    max_tree_bytes: int = _EXTERNAL_TREE_MAX_BYTES,
    max_tree_entries: int = _EXTERNAL_TREE_MAX_ENTRIES,
) -> tuple[tuple[dict[str, object], ...], tuple[tuple[str, str, bytes], ...]]:
    """Capture complete sibling commits so queued tickets survive Git GC."""
    sealed: list[dict[str, object]] = []
    captured: list[tuple[str, str, bytes]] = []
    for payload in descriptors:
        source = ExternalGitSource.from_payload(payload).pinned()
        status = subprocess.run(
            trusted_git_command(source.repo_root, "status", "--porcelain=v1", "-z", "--untracked-files=all"),
            cwd=source.repo_root,
            check=False,
            capture_output=True,
        )
        if status.returncode != 0:
            raise CoordinatorError(
                "validation_ticket_external_status_failed",
                "External Git worktree status could not be verified before sealing",
                details={"repoRoot": str(source.repo_root)},
            )
        if status.stdout:
            raise CoordinatorError(
                "validation_ticket_external_worktree_dirty",
                "External Git worktrees must be clean before immutable validation",
                details={"repoRoot": str(source.repo_root)},
            )
        _require_external_tree_budget(
            source,
            max_bytes=max_tree_bytes,
            max_entries=max_tree_entries,
        )
        archive = _capture_external_archive(
            source,
            max_bytes=max_archive_bytes,
        )
        archive_hash = hashlib.sha256(archive).hexdigest()
        sealed_source = ExternalGitSource.from_payload(
            {
                "repoRoot": str(source.repo_root),
                "commit": source.commit,
                "mountPath": source.mount_path,
                # A complete archive is intentional: Cargo targets, build scripts,
                # and include_* resources may legally live outside a package root.
                "includeRoots": [EXTERNAL_REPOSITORY_ROOT],
                "archiveHash": archive_hash,
                "archiveByteCount": len(archive),
            }
        )
        sealed.append(sealed_source.to_payload())
        captured.append(
            (f"external/{source.mount_path}.tar", archive_hash, archive)
        )
    return (
        tuple(sorted(sealed, key=lambda item: str(item["mountPath"]).casefold())),
        tuple(captured),
    )


def _require_external_tree_budget(
    source: ExternalGitSource,
    *,
    max_bytes: int,
    max_entries: int,
) -> None:
    result = subprocess.run(
        trusted_git_command(
            source.repo_root,
            "ls-tree",
            "-rlz",
            source.commit,
        ),
        cwd=source.repo_root,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise CoordinatorError(
            "validation_ticket_external_archive_failed",
            "External Git tree could not be measured before sealing",
            details={"repoRoot": str(source.repo_root)},
        )
    entry_count = 0
    byte_count = 0
    for entry in result.stdout.split(b"\0"):
        if not entry:
            continue
        header, separator, _path = entry.partition(b"\t")
        fields = header.split()
        if not separator or len(fields) < 4 or fields[1] != b"blob":
            continue
        try:
            size = int(fields[3])
        except ValueError as error:
            raise CoordinatorError(
                "validation_ticket_external_archive_failed",
                "External Git tree returned an invalid blob size",
                details={"repoRoot": str(source.repo_root)},
            ) from error
        entry_count += 1
        byte_count += size
        if entry_count > max_entries or byte_count > max_bytes:
            raise CoordinatorError(
                "validation_ticket_external_archive_too_large",
                "External Git snapshot exceeds the coordinator sealing budget",
                details={
                    "repoRoot": str(source.repo_root),
                    "entryCount": entry_count,
                    "byteCount": byte_count,
                    "maxEntryCount": max_entries,
                    "maxByteCount": max_bytes,
                },
            )


def _capture_external_archive(
    source: ExternalGitSource,
    *,
    max_bytes: int,
) -> bytes:
    with tempfile.TemporaryFile() as error_stream:
        process = subprocess.Popen(
            trusted_git_command(
                source.repo_root, "archive", "--format=tar", source.commit
            ),
            cwd=source.repo_root,
            stdout=subprocess.PIPE,
            stderr=error_stream,
        )
        archive = BytesIO()
        try:
            if process.stdout is None:
                raise OSError("Git archive did not expose a stdout stream")
            while True:
                chunk = process.stdout.read(1024 * 1024)
                if not chunk:
                    break
                if archive.tell() + len(chunk) > max_bytes:
                    process.kill()
                    process.wait()
                    raise CoordinatorError(
                        "validation_ticket_external_archive_too_large",
                        "External Git archive exceeds the coordinator sealing budget",
                        details={
                            "repoRoot": str(source.repo_root),
                            "maxArchiveByteCount": max_bytes,
                        },
                    )
                archive.write(chunk)
            process.wait(timeout=60)
        except BaseException:
            if process.poll() is None:
                process.kill()
                process.wait()
            raise
        if process.returncode != 0:
            error_stream.seek(0)
            raise CoordinatorError(
                "validation_ticket_external_archive_failed",
                "External Git inputs could not be sealed at ticket submission",
                details={
                    "repoRoot": str(source.repo_root),
                    "commit": source.commit,
                    "stderr": error_stream.read().decode(
                        "utf-8", errors="replace"
                    )[-4096:],
                },
            )
        return archive.getvalue()


def external_sources_from_coverage(
    coverage: Mapping[str, object],
) -> tuple[dict[str, object], ...]:
    """Read and validate the immutable descriptors stored on a ticket."""
    raw = coverage.get(EXTERNAL_SOURCES_COVERAGE_KEY)
    if raw is None:
        return ()
    if not isinstance(raw, (list, tuple)):
        raise CoordinatorError(
            "validation_ticket_external_sources_invalid",
            "coverage.externalSources must be an array of pinned descriptors",
        )
    result: list[dict[str, object]] = []
    for item in raw:
        if not isinstance(item, Mapping):
            raise CoordinatorError(
                "validation_ticket_external_sources_invalid",
                "coverage.externalSources entries must be objects",
            )
        # ExternalGitSource performs the complete path, commit, and include-root
        # validation and returns a canonical payload.
        result.append(ExternalGitSource.from_payload(item).to_payload())
    return tuple(result)


def merge_external_sources_into_coverage(
    coverage: Mapping[str, object],
    discovered: tuple[dict[str, object], ...],
) -> dict[str, object]:
    """Replace caller hints with the coordinator-sealed discovery result."""
    result = dict(coverage)
    existing = external_sources_from_coverage(result)
    sealed_by_root = {
        str(item["repoRoot"]).casefold(): item for item in discovered
    }
    for prior in existing:
        key = str(prior["repoRoot"]).casefold()
        sealed = sealed_by_root.get(key)
        if sealed is None:
            raise CoordinatorError(
                "validation_ticket_external_source_unexpected",
                "Caller-provided external source was not discovered from Cargo inputs",
                details={"repoRoot": prior["repoRoot"]},
            )
        if "archiveHash" in prior or "archiveByteCount" in prior:
            raise CoordinatorError(
                "validation_ticket_external_archive_coordinator_owned",
                "External source archives are sealed by the coordinator",
                details={"repoRoot": prior["repoRoot"]},
            )
        if (
            prior["commit"] != sealed["commit"]
            or prior["mountPath"] != sealed["mountPath"]
        ):
            raise CoordinatorError(
                "validation_ticket_external_source_conflict",
                "Caller-provided external source conflicts with the submission pin",
                details={"repoRoot": prior["repoRoot"]},
            )
    # Presence of this key is also the version marker that distinguishes a new
    # submit-time scan with no sibling dependencies from a legacy ticket.
    result[EXTERNAL_SOURCES_COVERAGE_KEY] = [
        sealed_by_root[key]
        for key in sorted(sealed_by_root, key=str.casefold)
    ]
    return result


def _safe_relative(value: object) -> str:
    return normalize_portable_relative_path(
        value,
        code="validation_ticket_external_sources_invalid",
        message="Sealed overlay path must be safe, portable, and relative",
    )


def _run_git(repo: Path, *arguments: str) -> str:
    try:
        completed = subprocess.run(
        trusted_git_command(repo, *arguments),
            cwd=repo,
            check=False,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git did not respond while external Cargo inputs were being pinned",
            details={"repoRoot": str(repo), "errorType": type(error).__name__},
        ) from error
    if completed.returncode != 0:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git could not provide immutable external Cargo inputs",
            details={"repoRoot": str(repo), "stderr": completed.stderr[-2048:]},
        )
    return completed.stdout.strip()


def _resolve_commit(repo: Path, commit: str) -> str:
    resolved = _run_git(repo, "rev-parse", "--verify", f"{commit}^{{commit}}")
    if not _FULL_GIT_COMMIT.fullmatch(resolved):
        raise CoordinatorError(
            "validation_ticket_baseline_invalid",
            "Cargo validation baseline is not a full Git commit",
        )
    return resolved.lower()


def _git_head(repo: Path) -> str:
    return _resolve_commit(repo, "HEAD")


def _git_tree_paths(repo: Path, commit: str) -> tuple[str, ...]:
    output = _run_git(repo, "ls-tree", "-r", "--name-only", commit)
    return tuple(path for path in output.splitlines() if path)


def _git_cargo_manifests(
    repo: Path,
    commit: str,
    paths: set[str],
) -> dict[str, bytes]:
    """Read all baseline manifests with one Git archive process."""
    if not paths:
        return {}
    try:
        completed = subprocess.run(
            trusted_git_command(
                repo, "archive", "--format=tar", commit, "--", *sorted(paths)
            ),
            cwd=repo,
            check=False,
            capture_output=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git did not provide Cargo manifests while external inputs were pinned",
            details={"repoRoot": str(repo), "errorType": type(error).__name__},
        ) from error
    if completed.returncode != 0:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git could not archive Cargo manifests for external input pinning",
            details={"repoRoot": str(repo), "stderr": completed.stderr[-2048:]},
        )
    result: dict[str, bytes] = {}
    try:
        with tarfile.open(fileobj=BytesIO(completed.stdout), mode="r:") as archive:
            for member in archive:
                if not member.isfile():
                    continue
                extracted = archive.extractfile(member)
                if extracted is not None:
                    result[member.name.replace("\\", "/")] = extracted.read()
    except tarfile.TarError as error:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git returned an unreadable Cargo manifest archive",
            details={"repoRoot": str(repo)},
        ) from error
    return result


def _git_show(repo: Path, commit: str, relative: str) -> bytes | None:
    try:
        completed = subprocess.run(
            trusted_git_command(repo, "show", f"{commit}:{relative}"),
            cwd=repo,
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git could not read a Cargo manifest while external inputs were pinned",
            details={"repoRoot": str(repo), "path": relative},
        ) from error
    if completed.returncode != 0:
        return None
    return bytes(completed.stdout)


def _external_git_root(repo_root: Path, dependency: Path) -> Path:
    candidate = dependency.parent
    while not candidate.exists() and candidate != candidate.parent:
        candidate = candidate.parent
    output = _run_git(candidate, "rev-parse", "--show-toplevel")
    external_root = Path(output).resolve()
    if (
        external_root == repo_root
        or external_root.parent != repo_root.parent
        or not dependency.is_relative_to(external_root)
    ):
        raise CoordinatorError(
            "validation_copy_external_source_missing",
            "Automatic Cargo source discovery is restricted to sibling Git repositories",
            details={"manifestPath": str(dependency), "repoRoot": str(external_root)},
        )
    return external_root


def _cargo_dependency_manifest(manifest: Path, raw_path: str) -> Path:
    candidate = (manifest.parent / raw_path).resolve()
    if candidate.name.casefold() != "cargo.toml":
        candidate /= "Cargo.toml"
    return candidate


def _selected_main_manifests(
    manifest_paths: set[str],
    baseline_contents: Mapping[str, bytes],
    overlays: Mapping[str, bytes | None],
    command: tuple[str, ...],
) -> set[str]:
    if not command:
        return set(manifest_paths)
    explicit = cargo_manifest_path_argument(command)
    explicit_manifest = _safe_relative(explicit) if explicit is not None else None

    documents: dict[str, Mapping[str, object]] = {}
    package_names: dict[str, str] = {}
    for relative in manifest_paths:
        content = overlays.get(relative, baseline_contents.get(relative))
        if content is None:
            continue
        try:
            document = tomllib.loads(content.decode("utf-8"))
        except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
            raise CoordinatorError(
                "validation_ticket_external_manifest_invalid",
                "Cargo manifest could not be parsed while external inputs were selected",
                details={"manifestPath": relative},
            ) from error
        documents[relative] = document
        package = document.get("package")
        if isinstance(package, Mapping) and isinstance(package.get("name"), str):
            package_names[relative] = str(package["name"])

    anchor_manifest = explicit_manifest or "Cargo.toml"
    anchor_document = documents.get(anchor_manifest, {})
    workspace = anchor_document.get("workspace")
    if not isinstance(workspace, Mapping):
        requested = set(cargo_package_specs(command))
        if requested:
            return {
                relative
                for relative, package_name in package_names.items()
                if package_name in requested
            }
        return {anchor_manifest} & manifest_paths
    anchor_parent = PurePosixPath(anchor_manifest).parent
    raw_members = workspace.get("members", [])
    members = tuple(
        str(value).replace("\\", "/").strip("/")
        for value in raw_members
        if isinstance(value, str) and value.strip()
    )
    raw_excludes = workspace.get("exclude", [])
    excludes = tuple(
        str(value).replace("\\", "/").strip("/")
        for value in raw_excludes
        if isinstance(value, str) and value.strip()
    )

    def matches(patterns: tuple[str, ...], manifest_path: str) -> bool:
        package_path = PurePosixPath(manifest_path).parent
        try:
            package_root = package_path.relative_to(anchor_parent).as_posix()
        except ValueError:
            return False
        return any(fnmatchcase(package_root, pattern) for pattern in patterns)

    workspace_manifests = {
        relative
        for relative in documents
        if relative != "Cargo.toml"
        and matches(members, relative)
        and not matches(excludes, relative)
    }
    if "package" in anchor_document:
        workspace_manifests.add(anchor_manifest)
    requested = set(cargo_package_specs(command))
    if requested:
        matched = {
            relative
            for relative in workspace_manifests
            if package_names.get(relative) in requested
        }
        # An unresolved package may be a sibling workspace member. Queue the
        # workspace manifest so its literal member paths are examined.
        matched_names = {package_names.get(relative) for relative in matched}
        return (
            matched | {anchor_manifest}
            if requested - matched_names
            else matched
        )
    if cargo_selects_workspace(command):
        excluded_packages = set(cargo_excluded_package_specs(command))
        return {
            relative
            for relative in workspace_manifests
            if package_names.get(relative) not in excluded_packages
        } | {anchor_manifest}
    raw_defaults = workspace.get("default-members")
    if isinstance(raw_defaults, list):
        defaults = tuple(
            str(value).replace("\\", "/").strip("/")
            for value in raw_defaults
            if isinstance(value, str) and value.strip()
        )
        selected_defaults = {
            relative
            for relative in workspace_manifests
            if matches(defaults, relative)
        }
        unresolved_default = any(
            not any(matches((pattern,), relative) for relative in workspace_manifests)
            for pattern in defaults
        )
        return (
            selected_defaults | {anchor_manifest}
            if unresolved_default
            else selected_defaults
        )
    if "package" in anchor_document:
        return {anchor_manifest}
    return workspace_manifests


def _manifest_path_values(
    content: bytes,
    manifest: Path,
    *,
    patches_only: bool = False,
    include_workspace_members: bool = False,
) -> tuple[str, ...]:
    try:
        document = tomllib.loads(content.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise CoordinatorError(
            "validation_ticket_external_manifest_invalid",
            "Cargo manifest could not be parsed while external inputs were pinned",
            details={"manifestPath": str(manifest)},
        ) from error
    paths: set[str] = set()
    package = document.get("package")
    if isinstance(package, Mapping):
        workspace_path = package.get("workspace")
        if isinstance(workspace_path, str) and workspace_path:
            paths.add(workspace_path)
    workspace = document.get("workspace")
    if include_workspace_members and isinstance(workspace, Mapping):
        members = workspace.get("members")
        if isinstance(members, list):
            for member in members:
                if not isinstance(member, str) or not member.strip():
                    continue
                if any(marker in member for marker in ("*", "?", "[")):
                    raise CoordinatorError(
                        "validation_ticket_external_workspace_glob_unsupported",
                        "Cross-repository workspace members must use literal paths",
                        details={"manifestPath": str(manifest), "member": member},
                    )
                paths.add(member)

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
            if patches_only and key == "workspace":
                inherited = value.get("dependencies")
                if isinstance(inherited, Mapping):
                    collect(inherited)
                continue
            if (
                not patches_only
                and key in {"dependencies", "dev-dependencies", "build-dependencies"}
            ):
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
            if not patches_only:
                visit(value)

    visit(document)
    return tuple(sorted(paths, key=str.casefold))


def _validate_cargo_config(content: bytes, config_path: Path) -> None:
    try:
        document = tomllib.loads(content.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise CoordinatorError(
            "validation_ticket_external_config_invalid",
            "Cargo config could not be parsed while validation inputs were pinned",
            details={"configPath": str(config_path)},
        ) from error
    # Configuration files are pinned and hashed, but most Cargo roots can
    # redirect compiler execution, output storage, dependency resolution, or
    # environment values.  Keep only transport and display policy here.
    allowed_roots = {"net", "http", "term", "future-incompat-report"}
    unsupported_roots = {
        key for key in document if str(key).casefold() not in allowed_roots
    }
    if unsupported_roots:
        raise CoordinatorError(
            "validation_ticket_external_config_unsupported",
            "Cargo configuration that changes build inputs, tools, or storage is unsupported",
            details={
                "configPath": str(config_path),
                "configRoots": sorted(str(key) for key in unsupported_roots),
            },
        )
