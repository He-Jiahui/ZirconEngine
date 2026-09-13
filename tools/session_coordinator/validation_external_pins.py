"""Capture sibling Git identities for an immutable Cargo validation ticket."""

from __future__ import annotations

import re
import hashlib
import posixpath
import subprocess
import tarfile
import tempfile
import threading
import time
from dataclasses import dataclass
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
from .validation_copy_external import (
    EXTERNAL_GIT_PROBE_TIMEOUT_SECONDS,
    EXTERNAL_REPOSITORY_ROOT,
    ExternalGitSource,
)
from .trusted_tools import trusted_git_command


EXTERNAL_SOURCES_COVERAGE_KEY = "externalSources"
_FULL_GIT_COMMIT = re.compile(r"^[0-9a-f]{40}$", re.IGNORECASE)
_EXTERNAL_ARCHIVE_MAX_BYTES = 256 * 1024 * 1024
_EXTERNAL_ARCHIVE_TOTAL_MAX_BYTES = 512 * 1024 * 1024
_EXTERNAL_TREE_MAX_BYTES = 512 * 1024 * 1024
_EXTERNAL_TREE_MAX_ENTRIES = 100_000
_EXTERNAL_REPOSITORY_MAX_COUNT = 16
_EXTERNAL_GIT_TIMEOUT_SECONDS = EXTERNAL_GIT_PROBE_TIMEOUT_SECONDS
_EXTERNAL_TREE_RECORD_MAX_BYTES = 1024 * 1024
_EXTERNAL_MANIFEST_MAX_BYTES = 2 * 1024 * 1024
_EXTERNAL_MANIFEST_TOTAL_MAX_BYTES = 16 * 1024 * 1024
_EXTERNAL_MANIFEST_ARCHIVE_MAX_BYTES = 32 * 1024 * 1024
_EXTERNAL_MANIFEST_MAX_COUNT = 4_096
_EXTERNAL_MANIFEST_PATH_MAX_BYTES = 4 * 1024
_EXTERNAL_MANIFEST_PATH_TOTAL_MAX_BYTES = 24 * 1024


@dataclass
class _ExternalManifestBudget:
    max_file_bytes: int = _EXTERNAL_MANIFEST_MAX_BYTES
    max_total_bytes: int = _EXTERNAL_MANIFEST_TOTAL_MAX_BYTES
    max_count: int = _EXTERNAL_MANIFEST_MAX_COUNT
    total_bytes: int = 0
    count: int = 0

    def remaining_bytes(self, repo_root: Path, relative: str) -> int:
        remaining = self.max_total_bytes - self.total_bytes
        if remaining <= 0:
            raise CoordinatorError(
                "validation_ticket_external_manifest_total_too_large",
                "Cargo manifests exceed the external discovery byte budget",
                details={
                    "repoRoot": str(repo_root),
                    "path": relative,
                    "maxTotalByteCount": self.max_total_bytes,
                },
            )
        return min(self.max_file_bytes, remaining)

    def account(self, repo_root: Path, relative: str, byte_count: int) -> None:
        if self.count >= self.max_count:
            raise CoordinatorError(
                "validation_ticket_external_manifest_limit",
                "Cargo manifest discovery exceeds the coordinator file-count budget",
                details={
                    "repoRoot": str(repo_root),
                    "path": relative,
                    "maxManifestCount": self.max_count,
                },
            )
        if byte_count > self.max_file_bytes:
            raise CoordinatorError(
                "validation_ticket_external_manifest_too_large",
                "Cargo manifest exceeds the external discovery per-file budget",
                details={
                    "repoRoot": str(repo_root),
                    "path": relative,
                    "byteCount": byte_count,
                    "maxByteCount": self.max_file_bytes,
                },
            )
        if self.total_bytes + byte_count > self.max_total_bytes:
            raise CoordinatorError(
                "validation_ticket_external_manifest_total_too_large",
                "Cargo manifests exceed the external discovery byte budget",
                details={
                    "repoRoot": str(repo_root),
                    "path": relative,
                    "totalByteCount": self.total_bytes + byte_count,
                    "maxTotalByteCount": self.max_total_bytes,
                },
            )
        self.count += 1
        self.total_bytes += byte_count


def discover_and_seal_pinned_external_sources(
    repo_root: str | Path,
    *,
    baseline_commit: str,
    overlay_files: Mapping[str, bytes | None] | None = None,
    command: tuple[str, ...] | list[str] = (),
    max_archive_bytes: int = _EXTERNAL_ARCHIVE_MAX_BYTES,
    max_total_archive_bytes: int = _EXTERNAL_ARCHIVE_TOTAL_MAX_BYTES,
    max_tree_bytes: int = _EXTERNAL_TREE_MAX_BYTES,
    max_tree_entries: int = _EXTERNAL_TREE_MAX_ENTRIES,
    max_external_repositories: int = _EXTERNAL_REPOSITORY_MAX_COUNT,
    git_timeout_seconds: float = _EXTERNAL_GIT_TIMEOUT_SECONDS,
) -> tuple[tuple[dict[str, object], ...], tuple[tuple[str, str, bytes], ...]]:
    """Discover and seal sibling sources under one submission deadline."""
    deadline = _external_git_deadline(git_timeout_seconds)
    discovered = discover_pinned_external_sources(
        repo_root,
        baseline_commit=baseline_commit,
        overlay_files=overlay_files,
        command=command,
        max_external_repositories=max_external_repositories,
        git_timeout_seconds=git_timeout_seconds,
        _deadline=deadline,
    )
    return seal_pinned_external_sources(
        discovered,
        max_archive_bytes=max_archive_bytes,
        max_total_archive_bytes=max_total_archive_bytes,
        max_tree_bytes=max_tree_bytes,
        max_tree_entries=max_tree_entries,
        max_external_repositories=max_external_repositories,
        git_timeout_seconds=git_timeout_seconds,
        _deadline=deadline,
    )


def discover_pinned_external_sources(
    repo_root: str | Path,
    *,
    baseline_commit: str,
    overlay_files: Mapping[str, bytes | None] | None = None,
    command: tuple[str, ...] | list[str] = (),
    max_external_repositories: int = _EXTERNAL_REPOSITORY_MAX_COUNT,
    git_timeout_seconds: float = _EXTERNAL_GIT_TIMEOUT_SECONDS,
    _deadline: float | None = None,
) -> tuple[dict[str, object], ...]:
    """Discover path dependencies without executing Cargo.

    The scanner follows Cargo manifest ``path`` values in the immutable baseline
    (and any sealed manifest overlays).  Every sibling repository reached by that
    graph is represented by its current HEAD commit captured at ticket submission.
    The worker later materializes exactly these descriptors; it never falls back to
    a sibling HEAD observed after a ticket has waited in the FIFO.
    """
    _require_positive_integer_budget(
        "maxExternalRepositoryCount", max_external_repositories
    )
    deadline = _deadline or _external_git_deadline(git_timeout_seconds)
    manifest_budget = _ExternalManifestBudget()
    root = Path(repo_root).resolve()
    baseline = _resolve_commit(root, baseline_commit, deadline=deadline)
    overlays = {
        _safe_relative(path): content
        for path, content in (overlay_files or {}).items()
    }
    config_paths = {".cargo/config", ".cargo/config.toml"}
    config_paths.update(
        _safe_relative(path)
        for path in cargo_config_file_arguments(tuple(command))
    )
    baseline_paths = set(
        _git_tree_paths(
            root,
            baseline,
            config_paths=config_paths,
            deadline=deadline,
        )
    )
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
    baseline_configs = config_paths & baseline_paths
    archive_paths = baseline_manifests | baseline_configs
    baseline_contents = _git_cargo_manifests(
        root,
        baseline,
        archive_paths,
        deadline=deadline,
        manifest_budget=manifest_budget,
    )
    for relative in sorted(config_paths, key=str.casefold):
        content = overlays.get(relative, baseline_contents.get(relative))
        if content is not None:
            _validate_cargo_config(content, root / relative)
    selected_manifests, workspace_anchor = _selected_main_manifests(
        main_manifests,
        baseline_contents,
        overlays,
        tuple(command),
        metadata_topology=True,
    )
    workspace_scan_manifests = selected_manifests & {workspace_anchor}
    manifest_queue: list[tuple[Path, str, bytes | None, bool, bool]] = []
    for relative in sorted(selected_manifests, key=str.casefold):
        content = (
            overlays[relative]
            if relative in overlays
            else baseline_contents.get(relative)
        )
        if content is not None:
            manifest_queue.append((root / relative, baseline, content, False, False))

    # A root can be reached through several package manifests. Keep the narrowest
    # package root for each descriptor while preserving all needed package paths.
    discovered: dict[Path, tuple[str, set[str]]] = {}
    visited: set[tuple[Path, str, str]] = set()
    while manifest_queue:
        (
            manifest,
            commit,
            content,
            patches_only,
            include_workspace_dependencies,
        ) = manifest_queue.pop()
        relative_manifest = manifest.relative_to(root).as_posix() if manifest.is_relative_to(root) else None
        visit_key = (manifest.parent.resolve(), commit, relative_manifest or str(manifest))
        if visit_key in visited:
            continue
        visited.add(visit_key)
        for raw_path in _manifest_path_values(
            content,
            manifest,
            patches_only=patches_only,
            include_workspace_dependencies=include_workspace_dependencies,
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
                            (dependency, baseline, dependency_content, False, False)
                        )
                continue

            external_root = _external_git_root(
                root, dependency, deadline=deadline
            )
            if (
                external_root not in discovered
                and len(discovered) >= max_external_repositories
            ):
                raise CoordinatorError(
                    "validation_ticket_external_repository_limit",
                    "External Git snapshot exceeds the coordinator repository-count budget",
                    details={
                        "repositoryCount": len(discovered) + 1,
                        "maxRepositoryCount": max_external_repositories,
                    },
                )
            external_commit = _git_head(external_root, deadline=deadline)
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
            external_content = _git_show(
                external_root,
                external_commit,
                relative.as_posix(),
                deadline=deadline,
                manifest_budget=manifest_budget,
            )
            if external_content is not None:
                manifest_queue.append(
                    (dependency, external_commit, external_content, False, False)
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
    max_total_archive_bytes: int = _EXTERNAL_ARCHIVE_TOTAL_MAX_BYTES,
    max_tree_bytes: int = _EXTERNAL_TREE_MAX_BYTES,
    max_tree_entries: int = _EXTERNAL_TREE_MAX_ENTRIES,
    max_external_repositories: int = _EXTERNAL_REPOSITORY_MAX_COUNT,
    git_timeout_seconds: float = _EXTERNAL_GIT_TIMEOUT_SECONDS,
    _deadline: float | None = None,
) -> tuple[tuple[dict[str, object], ...], tuple[tuple[str, str, bytes], ...]]:
    """Capture complete sibling commits so queued tickets survive Git GC."""
    for name, value in (
        ("maxArchiveByteCount", max_archive_bytes),
        ("maxTotalArchiveByteCount", max_total_archive_bytes),
        ("maxTreeByteCount", max_tree_bytes),
        ("maxTreeEntryCount", max_tree_entries),
        ("maxExternalRepositoryCount", max_external_repositories),
    ):
        _require_positive_integer_budget(name, value)
    deadline = _deadline or _external_git_deadline(git_timeout_seconds)
    if len(descriptors) > max_external_repositories:
        raise CoordinatorError(
            "validation_ticket_external_repository_limit",
            "External Git snapshot exceeds the coordinator repository-count budget",
            details={
                "repositoryCount": len(descriptors),
                "maxRepositoryCount": max_external_repositories,
            },
        )
    sealed: list[dict[str, object]] = []
    captured: list[tuple[str, str, bytes]] = []
    total_archive_bytes = 0
    for payload in descriptors:
        source = ExternalGitSource.from_payload(payload).pinned(
            timeout_seconds=_remaining_external_git_seconds(
                deadline, phase="commit_probe", repo_root=Path(str(payload["repoRoot"]))
            )
        )
        if _external_worktree_is_dirty(
            source,
            timeout_seconds=_remaining_external_git_seconds(
                deadline, phase="worktree_status", repo_root=source.repo_root
            ),
        ):
            raise CoordinatorError(
                "validation_ticket_external_worktree_dirty",
                "External Git worktrees must be clean before immutable validation",
                details={"repoRoot": str(source.repo_root)},
            )
        _require_external_tree_budget(
            source,
            max_bytes=max_tree_bytes,
            max_entries=max_tree_entries,
            timeout_seconds=_remaining_external_git_seconds(
                deadline, phase="tree_measurement", repo_root=source.repo_root
            ),
        )
        remaining_archive_bytes = max_total_archive_bytes - total_archive_bytes
        if remaining_archive_bytes <= 0:
            raise CoordinatorError(
                "validation_ticket_external_archive_total_too_large",
                "External Git snapshots exceed the coordinator aggregate sealing budget",
                details={
                    "totalArchiveByteCount": total_archive_bytes,
                    "maxTotalArchiveByteCount": max_total_archive_bytes,
                },
            )
        effective_archive_limit = min(max_archive_bytes, remaining_archive_bytes)
        try:
            archive = _capture_external_archive(
                source,
                max_bytes=effective_archive_limit,
                timeout_seconds=_remaining_external_git_seconds(
                    deadline, phase="archive_capture", repo_root=source.repo_root
                ),
            )
        except CoordinatorError as error:
            if (
                error.code == "validation_ticket_external_archive_too_large"
                and effective_archive_limit < max_archive_bytes
            ):
                raise CoordinatorError(
                    "validation_ticket_external_archive_total_too_large",
                    "External Git snapshots exceed the coordinator aggregate sealing budget",
                    details={
                        "totalArchiveByteCount": total_archive_bytes,
                        "maxTotalArchiveByteCount": max_total_archive_bytes,
                        "repoRoot": str(source.repo_root),
                    },
                ) from error
            raise
        if len(archive) > remaining_archive_bytes:
            raise CoordinatorError(
                "validation_ticket_external_archive_total_too_large",
                "External Git snapshots exceed the coordinator aggregate sealing budget",
                details={
                    "totalArchiveByteCount": total_archive_bytes + len(archive),
                    "maxTotalArchiveByteCount": max_total_archive_bytes,
                    "repoRoot": str(source.repo_root),
                },
            )
        total_archive_bytes += len(archive)
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


def _require_positive_integer_budget(name: str, value: object) -> None:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise CoordinatorError(
            "validation_ticket_external_archive_budget_invalid",
            "External Git sealing budgets must be positive integers",
            details={"budget": name, "value": value},
        )


def _external_git_deadline(timeout_seconds: float) -> float:
    if (
        isinstance(timeout_seconds, bool)
        or not isinstance(timeout_seconds, (int, float))
        or timeout_seconds <= 0
    ):
        raise CoordinatorError(
            "validation_ticket_external_archive_budget_invalid",
            "External Git sealing timeout must be positive",
            details={"budget": "gitTimeoutSeconds", "value": timeout_seconds},
        )
    return time.monotonic() + float(timeout_seconds)


def _remaining_external_git_seconds(
    deadline: float,
    *,
    phase: str,
    repo_root: Path | None = None,
) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        details = {"phase": phase}
        if repo_root is not None:
            details["repoRoot"] = str(repo_root)
        raise CoordinatorError(
            "validation_ticket_external_deadline_exceeded",
            "External Git input sealing exceeded the submission deadline",
            details=details,
        )
    return remaining


def _external_worktree_is_dirty(
    source: ExternalGitSource,
    *,
    timeout_seconds: float,
) -> bool:
    with tempfile.TemporaryFile() as error_stream:
        try:
            process = subprocess.Popen(
                trusted_git_command(
                    source.repo_root,
                    "status",
                    "--porcelain=v1",
                    "-z",
                    "--untracked-files=all",
                ),
                cwd=source.repo_root,
                stdout=subprocess.PIPE,
                stderr=error_stream,
            )
        except OSError as error:
            raise CoordinatorError(
                "validation_ticket_external_status_failed",
                "External Git worktree status could not be verified before sealing",
                details={
                    "repoRoot": str(source.repo_root),
                    "errorType": type(error).__name__,
                },
            ) from error
        timed_out = threading.Event()

        def terminate_on_timeout() -> None:
            if process.poll() is None:
                timed_out.set()
                try:
                    process.kill()
                except OSError:
                    pass

        watchdog = threading.Timer(timeout_seconds, terminate_on_timeout)
        watchdog.daemon = True
        watchdog.start()
        dirty = False
        try:
            if process.stdout is None:
                raise OSError("Git status did not expose stdout")
            dirty = bool(process.stdout.read(1))
            if dirty and process.poll() is None:
                process.kill()
            process.wait()
        except BaseException:
            if process.poll() is None:
                process.kill()
                process.wait()
            raise
        finally:
            watchdog.cancel()
            if process.stdout is not None:
                process.stdout.close()
        if timed_out.is_set():
            raise CoordinatorError(
                "validation_ticket_external_status_timeout",
                "External Git worktree status exceeded the coordinator deadline",
                details={"repoRoot": str(source.repo_root)},
            )
        if dirty:
            return True
        if process.returncode != 0:
            error_stream.seek(0)
            raise CoordinatorError(
                "validation_ticket_external_status_failed",
                "External Git worktree status could not be verified before sealing",
                details={
                    "repoRoot": str(source.repo_root),
                    "stderr": error_stream.read().decode(
                        "utf-8", errors="replace"
                    )[-4096:],
                },
            )
        return False


def _require_external_tree_budget(
    source: ExternalGitSource,
    *,
    max_bytes: int,
    max_entries: int,
    timeout_seconds: float = _EXTERNAL_GIT_TIMEOUT_SECONDS,
) -> None:
    entry_count = 0
    byte_count = 0
    with tempfile.TemporaryFile() as error_stream:
        try:
            process = subprocess.Popen(
                trusted_git_command(
                    source.repo_root,
                    "ls-tree",
                    "-rlz",
                    source.commit,
                ),
                cwd=source.repo_root,
                stdout=subprocess.PIPE,
                stderr=error_stream,
            )
        except OSError as error:
            raise CoordinatorError(
                "validation_ticket_external_archive_failed",
                "External Git tree could not be measured before sealing",
                details={
                    "repoRoot": str(source.repo_root),
                    "errorType": type(error).__name__,
                },
            ) from error
        timed_out = threading.Event()

        def terminate_on_timeout() -> None:
            if process.poll() is None:
                timed_out.set()
                try:
                    process.kill()
                except OSError:
                    pass

        watchdog = threading.Timer(timeout_seconds, terminate_on_timeout)
        watchdog.daemon = True
        watchdog.start()

        def account(entry: bytes) -> None:
            nonlocal entry_count, byte_count
            if not entry:
                return
            header, separator, _path = entry.partition(b"\t")
            fields = header.split()
            if not separator or len(fields) < 4 or fields[1] != b"blob":
                return
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

        pending = bytearray()
        try:
            if process.stdout is None:
                raise OSError("Git tree measurement did not expose stdout")
            while True:
                chunk = process.stdout.read(64 * 1024)
                if not chunk:
                    break
                pending.extend(chunk)
                while True:
                    boundary = pending.find(b"\0")
                    if boundary < 0:
                        break
                    account(bytes(pending[:boundary]))
                    del pending[: boundary + 1]
                if len(pending) > _EXTERNAL_TREE_RECORD_MAX_BYTES:
                    raise CoordinatorError(
                        "validation_ticket_external_archive_failed",
                        "External Git tree returned an oversized record",
                        details={"repoRoot": str(source.repo_root)},
                    )
            if pending:
                raise CoordinatorError(
                    "validation_ticket_external_archive_failed",
                    "External Git tree returned a truncated record",
                    details={"repoRoot": str(source.repo_root)},
                )
            process.wait()
        except BaseException:
            if process.poll() is None:
                process.kill()
                process.wait()
            raise
        finally:
            watchdog.cancel()
            if process.stdout is not None:
                process.stdout.close()
        if timed_out.is_set():
            raise CoordinatorError(
                "validation_ticket_external_tree_timeout",
                "External Git tree measurement exceeded the coordinator deadline",
                details={"repoRoot": str(source.repo_root)},
            )
        if process.returncode != 0:
            error_stream.seek(0)
            raise CoordinatorError(
                "validation_ticket_external_archive_failed",
                "External Git tree could not be measured before sealing",
                details={
                    "repoRoot": str(source.repo_root),
                    "stderr": error_stream.read().decode(
                        "utf-8", errors="replace"
                    )[-4096:],
                },
            )


def _capture_external_archive(
    source: ExternalGitSource,
    *,
    max_bytes: int,
    timeout_seconds: float = _EXTERNAL_GIT_TIMEOUT_SECONDS,
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
        timed_out = threading.Event()

        def terminate_on_timeout() -> None:
            if process.poll() is None:
                timed_out.set()
                try:
                    process.kill()
                except OSError:
                    pass

        watchdog = threading.Timer(timeout_seconds, terminate_on_timeout)
        watchdog.daemon = True
        watchdog.start()
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
            process.wait()
        except BaseException:
            if process.poll() is None:
                process.kill()
                process.wait()
            raise
        finally:
            watchdog.cancel()
            if process.stdout is not None:
                process.stdout.close()
        if timed_out.is_set():
            raise CoordinatorError(
                "validation_ticket_external_archive_timeout",
                "External Git inputs could not be sealed within the coordinator deadline",
                details={"repoRoot": str(source.repo_root)},
            )
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


def _bounded_git_stdout(
    repo: Path,
    arguments: tuple[str, ...],
    *,
    max_bytes: int,
    timeout_seconds: float,
    too_large_code: str,
    too_large_message: str,
    phase: str,
) -> tuple[int, bytes, str]:
    with tempfile.TemporaryFile() as error_stream:
        try:
            process = subprocess.Popen(
                trusted_git_command(repo, *arguments),
                cwd=repo,
                stdout=subprocess.PIPE,
                stderr=error_stream,
            )
        except OSError as error:
            raise CoordinatorError(
                "validation_ticket_external_git_failed",
                "Git could not stream bounded external Cargo inputs",
                details={
                    "repoRoot": str(repo),
                    "phase": phase,
                    "errorType": type(error).__name__,
                },
            ) from error
        timed_out = threading.Event()

        def terminate_on_timeout() -> None:
            if process.poll() is None:
                timed_out.set()
                try:
                    process.kill()
                except OSError:
                    pass

        watchdog = threading.Timer(timeout_seconds, terminate_on_timeout)
        watchdog.daemon = True
        watchdog.start()
        output = BytesIO()
        try:
            if process.stdout is None:
                raise OSError("Git did not expose bounded stdout")
            while True:
                chunk = process.stdout.read(64 * 1024)
                if not chunk:
                    break
                if output.tell() + len(chunk) > max_bytes:
                    raise CoordinatorError(
                        too_large_code,
                        too_large_message,
                        details={
                            "repoRoot": str(repo),
                            "phase": phase,
                            "maxByteCount": max_bytes,
                        },
                    )
                output.write(chunk)
            process.wait()
        except BaseException:
            if process.poll() is None:
                process.kill()
                process.wait()
            raise
        finally:
            watchdog.cancel()
            if process.stdout is not None:
                process.stdout.close()
        if timed_out.is_set():
            raise CoordinatorError(
                "validation_ticket_external_deadline_exceeded",
                "External Git input discovery exceeded the submission deadline",
                details={"repoRoot": str(repo), "phase": phase},
            )
        error_stream.seek(0)
        stderr = error_stream.read().decode("utf-8", errors="replace")[-4096:]
        return int(process.returncode), output.getvalue(), stderr


def _run_git(
    repo: Path,
    *arguments: str,
    deadline: float | None = None,
    timeout_seconds: float = 10.0,
) -> str:
    timeout = (
        min(
            timeout_seconds,
            _remaining_external_git_seconds(
                deadline, phase="discovery", repo_root=repo
            ),
        )
        if deadline is not None
        else timeout_seconds
    )
    try:
        completed = subprocess.run(
            trusted_git_command(repo, *arguments),
            cwd=repo,
            check=False,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
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


def _resolve_commit(
    repo: Path, commit: str, *, deadline: float | None = None
) -> str:
    resolved = _run_git(
        repo,
        "rev-parse",
        "--verify",
        f"{commit}^{{commit}}",
        deadline=deadline,
    )
    if not _FULL_GIT_COMMIT.fullmatch(resolved):
        raise CoordinatorError(
            "validation_ticket_baseline_invalid",
            "Cargo validation baseline is not a full Git commit",
        )
    return resolved.lower()


def _git_head(repo: Path, *, deadline: float | None = None) -> str:
    return _resolve_commit(repo, "HEAD", deadline=deadline)


def _git_tree_paths(
    repo: Path,
    commit: str,
    *,
    config_paths: set[str] | frozenset[str] = frozenset(),
    deadline: float | None = None,
    max_record_bytes: int = _EXTERNAL_MANIFEST_PATH_MAX_BYTES,
    max_selected_path_bytes: int = _EXTERNAL_MANIFEST_PATH_TOTAL_MAX_BYTES,
    max_manifest_count: int = _EXTERNAL_MANIFEST_MAX_COUNT,
) -> tuple[str, ...]:
    timeout = (
        _remaining_external_git_seconds(
            deadline, phase="main_tree_scan", repo_root=repo
        )
        if deadline is not None
        else _EXTERNAL_GIT_TIMEOUT_SECONDS
    )
    with tempfile.TemporaryFile() as error_stream:
        try:
            process = subprocess.Popen(
                trusted_git_command(repo, "ls-tree", "-r", "-z", "--name-only", commit),
                cwd=repo,
                stdout=subprocess.PIPE,
                stderr=error_stream,
            )
        except OSError as error:
            raise CoordinatorError(
                "validation_ticket_external_git_failed",
                "Git could not enumerate the baseline Cargo tree",
                details={"repoRoot": str(repo), "errorType": type(error).__name__},
            ) from error
        timed_out = threading.Event()

        def terminate_on_timeout() -> None:
            if process.poll() is None:
                timed_out.set()
                try:
                    process.kill()
                except OSError:
                    pass

        watchdog = threading.Timer(timeout, terminate_on_timeout)
        watchdog.daemon = True
        watchdog.start()
        selected: set[str] = set()
        selected_path_bytes = 0
        pending = bytearray()
        try:
            if process.stdout is None:
                raise OSError("Git baseline tree did not expose stdout")
            while True:
                chunk = process.stdout.read(64 * 1024)
                if not chunk:
                    break
                pending.extend(chunk)
                while True:
                    boundary = pending.find(b"\0")
                    if boundary < 0:
                        break
                    record = bytes(pending[:boundary])
                    del pending[: boundary + 1]
                    if len(record) > max_record_bytes:
                        raise CoordinatorError(
                            "validation_ticket_external_manifest_path_too_large",
                            "A baseline Cargo path exceeds the discovery record budget",
                            details={"repoRoot": str(repo)},
                        )
                    if not record:
                        continue
                    try:
                        path = record.decode("utf-8")
                    except UnicodeDecodeError as error:
                        raise CoordinatorError(
                            "validation_ticket_external_git_failed",
                            "Git returned a non-UTF-8 baseline Cargo path",
                            details={"repoRoot": str(repo)},
                        ) from error
                    if path == "Cargo.toml" or path.endswith("/Cargo.toml"):
                        if path not in selected:
                            selected_path_bytes += len(record)
                            if selected_path_bytes > max_selected_path_bytes:
                                raise CoordinatorError(
                                    "validation_ticket_external_manifest_path_total_too_large",
                                    "Baseline Cargo paths exceed the discovery argv budget",
                                    details={
                                        "repoRoot": str(repo),
                                        "totalPathByteCount": selected_path_bytes,
                                        "maxTotalPathByteCount": max_selected_path_bytes,
                                    },
                                )
                            selected.add(path)
                        if len(selected) > max_manifest_count:
                            raise CoordinatorError(
                                "validation_ticket_external_manifest_limit",
                                "Baseline Cargo manifests exceed the discovery file-count budget",
                                details={
                                    "repoRoot": str(repo),
                                    "maxManifestCount": max_manifest_count,
                                },
                            )
                    elif path in config_paths:
                        if path not in selected:
                            selected_path_bytes += len(record)
                            if selected_path_bytes > max_selected_path_bytes:
                                raise CoordinatorError(
                                    "validation_ticket_external_manifest_path_total_too_large",
                                    "Baseline Cargo paths exceed the discovery argv budget",
                                    details={
                                        "repoRoot": str(repo),
                                        "totalPathByteCount": selected_path_bytes,
                                        "maxTotalPathByteCount": max_selected_path_bytes,
                                    },
                                )
                            selected.add(path)
                if len(pending) > max_record_bytes:
                    raise CoordinatorError(
                        "validation_ticket_external_manifest_path_too_large",
                        "A baseline Cargo path exceeds the discovery record budget",
                        details={"repoRoot": str(repo)},
                    )
            if pending:
                raise CoordinatorError(
                    "validation_ticket_external_git_failed",
                    "Git returned a truncated baseline Cargo path",
                    details={"repoRoot": str(repo)},
                )
            process.wait()
        except BaseException:
            if process.poll() is None:
                process.kill()
                process.wait()
            raise
        finally:
            watchdog.cancel()
            if process.stdout is not None:
                process.stdout.close()
        if timed_out.is_set():
            raise CoordinatorError(
                "validation_ticket_external_deadline_exceeded",
                "External Git input discovery exceeded the submission deadline",
                details={"repoRoot": str(repo), "phase": "main_tree_scan"},
            )
        if process.returncode != 0:
            error_stream.seek(0)
            raise CoordinatorError(
                "validation_ticket_external_git_failed",
                "Git could not enumerate the baseline Cargo tree",
                details={
                    "repoRoot": str(repo),
                    "stderr": error_stream.read().decode(
                        "utf-8", errors="replace"
                    )[-4096:],
                },
            )
        return tuple(sorted(selected, key=str.casefold))


def _git_cargo_manifests(
    repo: Path,
    commit: str,
    paths: set[str],
    *,
    deadline: float | None = None,
    manifest_budget: _ExternalManifestBudget | None = None,
) -> dict[str, bytes]:
    """Read all baseline manifests with one Git archive process."""
    if not paths:
        return {}
    timeout = (
        min(
            30.0,
            _remaining_external_git_seconds(
                deadline, phase="manifest_archive", repo_root=repo
            ),
        )
        if deadline is not None
        else 30.0
    )
    return_code, archive_bytes, stderr = _bounded_git_stdout(
        repo,
        ("archive", "--format=tar", commit, "--", *sorted(paths)),
        max_bytes=_EXTERNAL_MANIFEST_ARCHIVE_MAX_BYTES,
        timeout_seconds=timeout,
        too_large_code="validation_ticket_external_manifest_total_too_large",
        too_large_message="Cargo manifest archive exceeds the discovery byte budget",
        phase="manifest_archive",
    )
    if return_code != 0:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git could not archive Cargo manifests for external input pinning",
            details={"repoRoot": str(repo), "stderr": stderr},
        )
    budget = manifest_budget or _ExternalManifestBudget()
    result: dict[str, bytes] = {}
    try:
        with tarfile.open(fileobj=BytesIO(archive_bytes), mode="r:") as archive:
            for member in archive:
                if not member.isfile():
                    continue
                normalized_name = member.name.replace("\\", "/")
                budget.account(repo, normalized_name, int(member.size))
                extracted = archive.extractfile(member)
                if extracted is not None:
                    content = extracted.read(_EXTERNAL_MANIFEST_MAX_BYTES + 1)
                    if len(content) != member.size:
                        raise tarfile.ReadError(
                            f"truncated Cargo manifest {normalized_name}"
                        )
                    result[normalized_name] = content
    except tarfile.TarError as error:
        raise CoordinatorError(
            "validation_ticket_external_git_failed",
            "Git returned an unreadable Cargo manifest archive",
            details={"repoRoot": str(repo)},
        ) from error
    return result


def _git_show(
    repo: Path,
    commit: str,
    relative: str,
    *,
    deadline: float | None = None,
    manifest_budget: _ExternalManifestBudget | None = None,
) -> bytes | None:
    budget = manifest_budget or _ExternalManifestBudget()
    max_bytes = budget.remaining_bytes(repo, relative)
    timeout = (
        min(
            10.0,
            _remaining_external_git_seconds(
                deadline, phase="manifest_read", repo_root=repo
            ),
        )
        if deadline is not None
        else 10.0
    )
    too_large_code = (
        "validation_ticket_external_manifest_total_too_large"
        if max_bytes < budget.max_file_bytes
        else "validation_ticket_external_manifest_too_large"
    )
    return_code, content, _stderr = _bounded_git_stdout(
        repo,
        ("show", f"{commit}:{relative}"),
        max_bytes=max_bytes,
        timeout_seconds=timeout,
        too_large_code=too_large_code,
        too_large_message="Cargo manifest exceeds the external discovery byte budget",
        phase="manifest_read",
    )
    if return_code != 0:
        return None
    budget.account(repo, relative, len(content))
    return content


def _external_git_root(
    repo_root: Path,
    dependency: Path,
    *,
    deadline: float | None = None,
) -> Path:
    candidate = dependency.parent
    while not candidate.exists() and candidate != candidate.parent:
        candidate = candidate.parent
    output = _run_git(
        candidate, "rev-parse", "--show-toplevel", deadline=deadline
    )
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


def _workspace_path_has_prefix(
    package_root: str, excludes: tuple[str, ...]
) -> bool:
    normalized_root = posixpath.normpath(package_root).strip("/")
    for exclude in excludes:
        normalized_exclude = posixpath.normpath(exclude).strip("/")
        if normalized_exclude in {"", "."}:
            return True
        if normalized_root == normalized_exclude or normalized_root.startswith(
            normalized_exclude + "/"
        ):
            return True
    return False


def _selected_main_manifests(
    manifest_paths: set[str],
    baseline_contents: Mapping[str, bytes],
    overlays: Mapping[str, bytes | None],
    command: tuple[str, ...],
    *,
    metadata_topology: bool = False,
) -> tuple[set[str], str]:
    if not command:
        return set(manifest_paths), "Cargo.toml"
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
    if metadata_topology and explicit_manifest is not None:
        explicit_package_path = PurePosixPath(explicit_manifest).parent
        explicit_document = documents.get(explicit_manifest, {})
        explicit_package = explicit_document.get("package")
        declared_workspace = (
            explicit_package.get("workspace")
            if isinstance(explicit_package, Mapping)
            else None
        )
        has_declared_workspace = isinstance(declared_workspace, str) and bool(
            declared_workspace.strip()
        )
        declared_anchor: str | None = None
        if has_declared_workspace:
            candidate = posixpath.normpath(
                (
                    explicit_package_path
                    / str(declared_workspace).replace("\\", "/")
                    / "Cargo.toml"
                ).as_posix()
            )
            candidate_document = documents.get(candidate)
            if candidate_document is not None and isinstance(
                candidate_document.get("workspace"), Mapping
            ):
                declared_anchor = candidate
        workspace_candidates: list[str] = []
        for relative, document in documents.items():
            workspace_document = document.get("workspace")
            if not isinstance(workspace_document, Mapping):
                continue
            workspace_parent = PurePosixPath(relative).parent
            if not explicit_package_path.is_relative_to(workspace_parent):
                continue
            member_root = explicit_package_path.relative_to(
                workspace_parent
            ).as_posix()
            members = tuple(
                str(value).replace("\\", "/").strip("/")
                for value in workspace_document.get("members", [])
                if isinstance(value, str) and value.strip()
            )
            excludes = tuple(
                str(value).replace("\\", "/").strip("/")
                for value in workspace_document.get("exclude", [])
                if isinstance(value, str) and value.strip()
            )
            explicitly_included = _workspace_path_has_prefix(member_root, members)
            if not explicitly_included and _workspace_path_has_prefix(
                member_root, excludes
            ):
                continue
            workspace_candidates.append(relative)
        if declared_anchor is not None:
            anchor_manifest = declared_anchor
        elif not has_declared_workspace and workspace_candidates:
            anchor_manifest = max(
                workspace_candidates,
                key=lambda value: len(PurePosixPath(value).parent.parts),
            )
    anchor_document = documents.get(anchor_manifest, {})
    workspace = anchor_document.get("workspace")
    if not isinstance(workspace, Mapping):
        requested = set(cargo_package_specs(command))
        if requested:
            return (
                {
                    relative
                    for relative, package_name in package_names.items()
                    if package_name in requested
                },
                anchor_manifest,
            )
        return (
            {explicit_manifest or anchor_manifest} & manifest_paths,
            anchor_manifest,
        )
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
        if relative != anchor_manifest
        and matches(members, relative)
    }
    if "package" in anchor_document:
        workspace_manifests.add(anchor_manifest)
    if metadata_topology:
        selected = workspace_manifests | {anchor_manifest}
        if explicit_manifest is not None:
            selected.add(explicit_manifest)
        return selected, anchor_manifest
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
        selected = (
            matched | {anchor_manifest}
            if requested - matched_names
            else matched
        )
        return selected, anchor_manifest
    if cargo_selects_workspace(command):
        excluded_packages = set(cargo_excluded_package_specs(command))
        return (
            {
                relative
                for relative in workspace_manifests
                if package_names.get(relative) not in excluded_packages
            }
            | {anchor_manifest},
            anchor_manifest,
        )
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
        selected = (
            selected_defaults | {anchor_manifest}
            if unresolved_default
            else selected_defaults
        )
        return selected, anchor_manifest
    if "package" in anchor_document:
        return {anchor_manifest}, anchor_manifest
    return workspace_manifests, anchor_manifest


def _manifest_path_values(
    content: bytes,
    manifest: Path,
    *,
    patches_only: bool = False,
    include_workspace_dependencies: bool = False,
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
                if include_workspace_dependencies:
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
