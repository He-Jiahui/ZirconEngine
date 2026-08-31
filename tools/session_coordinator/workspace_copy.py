from __future__ import annotations

import hashlib
import io
import json
import os
import shutil
import subprocess
import tarfile
import threading
import uuid
from contextlib import nullcontext
from dataclasses import dataclass, field, replace
from pathlib import Path, PurePosixPath
from typing import Callable, ContextManager, Mapping

from .benchmark_validation_grants import (
    benchmark_child_environment,
    benchmark_run_environment,
    require_benchmark_launch_grant,
)
from .cargo_jobs import overlapping_cleanup_reservation, target_identity
from .cargo_command_policy import (
    inline_cargo_config_key,
    is_direct_cargo_command,
    rewrite_cargo_source_path_arguments,
    validate_cargo_storage_arguments,
    validate_inline_cargo_config,
)
from .database import Database
from .models import CoordinatorError, utc_text
from .portable_paths import normalize_portable_relative_path, portable_path_key
from .processes import (
    confirm_kill_on_close_job_terminated,
    popen_process_creation_time,
    process_creation_time,
    process_is_alive,
    terminate_process_tree,
)
from .snapshots import ObjectStore
from .windows_job_process import (
    close_process_job,
    create_atomic_kill_on_close_process,
    resume_popen_process,
    terminate_and_close_process_job,
)
from .trusted_tools import trusted_git_command
from .validation_copies import CargoInputClosurePlanner, ExternalGitSource
from .validation_copy_external import (
    external_archive_pathspecs,
    extract_external_archive,
)
from .validation_copy_cargo import ValidationCopyCargoExecution
from .pinned_cargo_planner import (
    PinnedCargoInputClosurePlanner,
    PinnedCargoPlannerView,
)
from .workspace_copy_terminal import (
    ValidationCopyTerminalLifecycle,
    ValidationRunEvidence,
)

_ARCHIVE_COMMAND_CHAR_LIMIT = 24_000


def _archive_member_destination(
    root: Path,
    member_name: str,
    *,
    error_code: str,
    seen_paths: dict[str, str] | None = None,
) -> Path:
    normalized = normalize_portable_relative_path(
        member_name,
        code=error_code,
        message="Archive member is not a safe portable extraction path",
    )
    if seen_paths is not None:
        key = portable_path_key(normalized)
        previous = seen_paths.get(key)
        if previous is not None:
            raise CoordinatorError(
                error_code,
                "Archive contains paths that collide on the managed filesystem",
                details={"firstPath": previous, "secondPath": normalized},
            )
        seen_paths[key] = normalized
    return root.joinpath(*PurePosixPath(normalized).parts)


def _is_managed_validation_root(root: Path) -> bool:
    return root.name.casefold() == "cargo-targets" and root.parent == Path(root.anchor)


@dataclass(frozen=True, slots=True)
class WorkspaceCopyRecord:
    job_id: str
    session_id: str
    job_root: Path
    source_root: Path
    target_root: Path
    manifest: tuple[str, ...]
    status: str
    external_sources: tuple[dict[str, object], ...] = ()
    input_manifest_hash: str | None = None
    error_code: str | None = None
    error_stage: str | None = None
    error_path: str | None = None
    materialization_phase: str | None = None
    terminal_evidence: ValidationRunEvidence | None = None
    error_details: dict[str, object] = field(default_factory=dict)
    materialization_kind: str | None = None

    def to_dict(self) -> dict[str, object]:
        return {
            "job_id": self.job_id,
            "session_id": self.session_id,
            "job_root": str(self.job_root),
            "source_root": str(self.source_root),
            "target_root": str(self.target_root),
            "manifest": list(self.manifest),
            "status": self.status,
            "externalSources": list(self.external_sources),
            "inputManifestHash": self.input_manifest_hash,
            "errorCode": self.error_code,
            "errorStage": self.error_stage,
            "errorPath": self.error_path,
            "errorDetails": dict(self.error_details),
            "materializationPhase": self.materialization_phase,
            "materializationKind": self.materialization_kind,
            "terminalEvidence": (
                self.terminal_evidence.to_dict()
                if self.terminal_evidence is not None
                else None
            ),
        }

    def acceptance_dict(self) -> dict[str, object]:
        """Return bounded durable-job metadata for asynchronous command acknowledgements."""
        return {
            "job_id": self.job_id,
            "session_id": self.session_id,
            "job_root": str(self.job_root),
            "source_root": str(self.source_root),
            "target_root": str(self.target_root),
            "status": self.status,
            "materializationPhase": self.materialization_phase,
        }


class WorkspaceCopyService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        target_roots: tuple[str | Path, ...],
        mutation_gate: Callable[[], ContextManager[None]] | None = None,
        cargo_materialization_preflight: Callable[[], None] | None = None,
        object_store: ObjectStore | None = None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        roots = tuple(Path(root).resolve() for root in target_roots)
        if not roots:
            raise CoordinatorError(
                "target_root_unavailable", "No managed target root is available for validation copy"
        )
        for root in roots:
            if not _is_managed_validation_root(root):
                raise CoordinatorError(
                    "invalid_target_root", f"Invalid validation-copy target root: {root}"
                )
        self.target_roots = roots
        self._running_lock = threading.Lock()
        self._active_run_jobs: set[str] = set()
        self._running_processes: dict[str, subprocess.Popen[str]] = {}
        self._running_process_jobs: dict[str, int] = {}
        self._materialization_lock = threading.Lock()
        self._active_materialization_jobs: set[str] = set()
        self._materialization_worker_id = (
            f"v2:{os.getpid()}:{process_creation_time(os.getpid())}:{uuid.uuid4().hex}"
        )
        self._cleanup_lock = threading.Lock()
        self._mutation_gate = mutation_gate
        self._cargo_materialization_preflight = cargo_materialization_preflight
        self._object_store = object_store
        self._cargo_execution: ValidationCopyCargoExecution | None = None
        self._completion_hook: Callable[[str], None] | None = None
        self._terminal = ValidationCopyTerminalLifecycle(database, mutation_gate)

    def set_completion_hook(self, hook: Callable[[str], None]) -> None:
        self._completion_hook = hook

    def set_cargo_materialization_preflight(
        self, preflight: Callable[[], None] | None
    ) -> None:
        """Configure a worker-only admission check for durable Cargo copies."""
        self._cargo_materialization_preflight = preflight

    def _run_cargo_materialization_preflight(self) -> None:
        preflight = self._cargo_materialization_preflight
        if preflight is None:
            return
        try:
            preflight()
            return
        except CoordinatorError as error:
            if error.code != "unmanaged_artifacts_detected":
                raise
        preflight()

    def set_cargo_execution(
        self, execution: ValidationCopyCargoExecution | None
    ) -> None:
        """Bind Cargo-materialized copies to the durable Cargo lane."""
        self._cargo_execution = execution

    def _reserve_local_run(self, job_id: str) -> None:
        with self._running_lock:
            if job_id in self._active_run_jobs:
                raise CoordinatorError(
                    "validation_copy_not_materialized",
                    "Validation copy is already running or unavailable",
                )
            self._active_run_jobs.add(job_id)

    def _release_local_run(self, job_id: str) -> None:
        with self._running_lock:
            process = self._running_processes.pop(job_id, None)
            job_handle = self._running_process_jobs.pop(job_id, None)
            self._active_run_jobs.discard(job_id)
        close_process_job(job_handle)
        close_process = getattr(process, "close", None)
        if close_process is not None:
            close_process()

    def _terminate_running_process_job(self, job_id: str) -> None:
        with self._running_lock:
            job_handle = self._running_process_jobs.pop(job_id, None)
        terminate_and_close_process_job(job_handle)

    def _reserve_local_materialization(self, job_id: str) -> bool:
        with self._materialization_lock:
            if job_id in self._active_materialization_jobs:
                return False
            self._active_materialization_jobs.add(job_id)
            return True

    def _release_local_materialization(self, job_id: str) -> None:
        with self._materialization_lock:
            self._active_materialization_jobs.discard(job_id)

    def _materialization_is_local(self, job_id: str) -> bool:
        with self._materialization_lock:
            return job_id in self._active_materialization_jobs

    def scoped_manifest_hash(
        self, job_id: str, paths: tuple[str, ...] | list[str]
    ) -> str:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT source_root, status FROM validation_copies WHERE job_id=?",
                (job_id,),
            ).fetchone()
        if row is None or row["status"] not in {"materialized", "running"}:
            raise CoordinatorError(
                "validation_copy_not_materialized",
                "Validation source manifest is not available",
            )
        source_root = Path(row["source_root"]).resolve()
        manifest: list[dict[str, object]] = []
        for raw in sorted(set(paths), key=str.casefold):
            normalized = self._normalize(raw)
            target = (source_root / normalized).resolve()
            if not target.is_relative_to(source_root):
                raise CoordinatorError(
                    "validation_copy_path_not_managed", "Validation manifest escaped its source root"
                )
            if target.is_file():
                digest = hashlib.sha256(target.read_bytes()).hexdigest()
                manifest.append({"path": normalized, "kind": "file", "blob": digest})
            else:
                manifest.append({"path": normalized, "kind": "deletion", "blob": None})
        return hashlib.sha256(
            json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest()

    def validation_manifest(self, session_id: str) -> tuple[str, ...]:
        """Derive a validation copy from tracked files plus current Session ownership."""
        self._require_session(session_id)
        tracked = set(self._git_text("ls-files").splitlines())
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT display_path, content_hash FROM attributions WHERE session_id = ?",
                (session_id,),
            ).fetchall()
        for row in rows:
            if row["content_hash"] is None:
                tracked.discard(row["display_path"])
            else:
                tracked.add(row["display_path"])
        manifest = tuple(sorted((path for path in tracked if path), key=str.casefold))
        if not manifest:
            raise CoordinatorError(
                "validation_copy_manifest_empty", "Validation copy has no server-derived files"
            )
        return manifest

    def plan(
        self,
        session_id: str,
        *,
        include_paths: tuple[str, ...] | list[str],
        external_sources: tuple[dict[str, object], ...]
        | list[dict[str, object]] = (),
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        self._require_session(session_id)
        manifest = tuple(
            sorted({self._normalize(path) for path in include_paths}, key=str.casefold)
        )
        if not manifest:
            raise CoordinatorError(
                "validation_copy_manifest_empty", "Validation copy requires source paths"
            )
        root = max(
            self.target_roots,
            key=lambda value: shutil.disk_usage(value.anchor or value.parent).free,
        )
        verify_root = self._managed_verify_root(root)
        job_id = uuid.uuid4().hex
        job_root = (verify_root / job_id).resolve()
        if job_root.parent != verify_root:
            raise CoordinatorError(
                "validation_copy_verify_escape", "Validation job escaped the managed verify root"
            )
        source_root = job_root / "source"
        target_root = job_root / "target"
        pinned_sources = tuple(
            ExternalGitSource.from_payload(payload).pinned()
            for payload in external_sources
        )
        mount_keys: set[str] = set()
        for external in pinned_sources:
            mount = (job_root / external.mount_path).resolve()
            mount_key = str(mount).casefold()
            if (
                not mount.is_relative_to(job_root)
                or mount == job_root
                or mount == source_root
                or mount == target_root
                or source_root.is_relative_to(mount)
                or target_root.is_relative_to(mount)
                or mount_key in mount_keys
            ):
                raise CoordinatorError(
                    "validation_copy_external_mount_escape",
                    "External Git mount must be a unique child of the validation job root",
                    details={"mountPath": external.mount_path},
                )
            mount_keys.add(mount_key)
        external_payloads = tuple(source.to_payload() for source in pinned_sources)
        head_commit = self._requested_baseline_commit(baseline_commit)
        with self.database.transaction() as connection:
            self._require_cleanup_available(connection, job_root)
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root, head_commit, manifest_json,
                    status, created_at, external_sources_json
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 'planned', ?, ?)
                """,
                (
                    job_id,
                    session_id,
                    str(job_root),
                    str(source_root),
                    str(target_root),
                    head_commit,
                    json.dumps(manifest),
                    utc_text(),
                    json.dumps(external_payloads, sort_keys=True),
                ),
            )
        return WorkspaceCopyRecord(
            job_id,
            session_id,
            job_root,
            source_root,
            target_root,
            manifest,
            "planned",
            external_payloads,
        )

    def materialize(
        self,
        session_id: str,
        *,
        include_paths: tuple[str, ...] | list[str],
        external_sources: tuple[dict[str, object], ...]
        | list[dict[str, object]] = (),
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        record = self.plan(
            session_id,
            include_paths=include_paths,
            external_sources=external_sources,
            baseline_commit=baseline_commit,
        )
        self._begin_materialization(record.job_id)
        return self._materialize_record(record)

    def materialize_async(
        self,
        session_id: str,
        *,
        include_paths: tuple[str, ...] | list[str],
        external_sources: tuple[dict[str, object], ...]
        | list[dict[str, object]] = (),
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        """Reserve a copy job immediately and materialize it off the request thread.

        A full workspace manifest can contain tens of thousands of tracked files.
        The coordinator must acknowledge that durable job before doing file I/O so
        Session heartbeats and Cargo lifecycle transitions keep progressing.
        """
        record = self.plan(
            session_id,
            include_paths=include_paths,
            external_sources=external_sources,
            baseline_commit=baseline_commit,
        )
        self._require_untracked_overlay_attribution(record)
        self._begin_materialization(record.job_id)
        worker = threading.Thread(
            target=self._materialize_async_worker,
            args=(record,),
            name=f"zircon-materialize-{record.job_id[:12]}",
            daemon=True,
        )
        worker.start()
        return WorkspaceCopyRecord(
            record.job_id,
            record.session_id,
            record.job_root,
            record.source_root,
            record.target_root,
            record.manifest,
            "materializing",
            record.external_sources,
        )

    def status(self, session_id: str, job_id: str) -> WorkspaceCopyRecord:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("validation_copy_not_found", f"Unknown validation-copy job: {job_id}")
        if row["session_id"] != session_id:
            raise CoordinatorError(
                "validation_copy_foreign_session", "Validation copy belongs to another Session"
            )
        return replace(
            self._record_from_row(row),
            terminal_evidence=self._terminal.latest_for_job(
                session_id=session_id, job_id=job_id
            ),
        )

    def _materialize_async_worker(self, record: WorkspaceCopyRecord) -> None:
        try:
            self._materialize_record(record)
        except BaseException:
            # The durable status records the failure.  Detached HTTP callers must
            # not turn a filesystem failure into an unhandled worker exception.
            return

    def _materialize_record(
        self,
        record: WorkspaceCopyRecord,
        *,
        worker_id: str | None = None,
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
    ) -> WorkspaceCopyRecord:
        stage = "prepare"
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            attribution = (
                self._session_attributions(record.session_id)
                if sealed_overlay_manifest is None
                else {
                    path.casefold(): object_hash
                    for path, object_hash in sealed_overlay_manifest.items()
                }
            )
            stage = "baseline_archive"
            input_entries = self._extract_baseline_manifest(record, attribution)
            overlays = tuple(
                path for path in record.manifest if path.casefold() in attribution
            )
            stage = (
                "sealed_overlay"
                if sealed_overlay_manifest is not None
                else "owned_overlay"
            )
            input_entries.update(
                self._overlay_sealed_sources(record, sealed_overlay_manifest)
                if sealed_overlay_manifest is not None
                else self._overlay_attributed_sources(record, overlays, attribution)
            )
            stage = "external_archive"
            input_entries.update(self._extract_external_sources(record))
            stage = "manifest_hash"
            input_manifest_hash = self._input_manifest_hash_from_entries(input_entries)
            self._complete_materialization(
                record.job_id, input_manifest_hash, worker_id=worker_id
            )
        except BaseException as error:
            self._fail_materialization(
                record.job_id, error=error, stage=stage, worker_id=worker_id
            )
            if record.job_root.exists():
                shutil.rmtree(record.job_root)
            raise
        return WorkspaceCopyRecord(
            record.job_id,
            record.session_id,
            record.job_root,
            record.source_root,
            record.target_root,
            record.manifest,
            "materialized",
            record.external_sources,
            input_manifest_hash,
            materialization_phase=(
                "materialized" if record.materialization_phase is not None else None
            ),
        )

    def materialize_validation(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str],
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        """Materialize declared template dependencies and Session-owned overlays.

        A milestone manifest names only files eligible for the eventual commit.  A
        validation template needs its own small, read-only baseline dependency
        closure.  Keeping those collections separate preserves exact commit
        attribution without copying the whole repository.
        """
        record, normalized_roots, normalized_overlays, attribution = (
            self._plan_validation_materialization(
                session_id,
                dependency_roots=dependency_roots,
                overlay_paths=overlay_paths,
                sealed_overlay_manifest=sealed_overlay_manifest,
                baseline_commit=baseline_commit,
            )
        )
        self._begin_materialization(record.job_id)
        return self._materialize_validation_record(
            record,
            normalized_roots,
            normalized_overlays,
            attribution,
            sealed_overlay_manifest,
        )

    def materialize_validation_async(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str],
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        """Durably reserve a dependency-scoped validation copy before archive I/O."""
        record, normalized_roots, normalized_overlays, attribution = (
            self._plan_validation_materialization(
                session_id,
                dependency_roots=dependency_roots,
                overlay_paths=overlay_paths,
                sealed_overlay_manifest=sealed_overlay_manifest,
                baseline_commit=baseline_commit,
            )
        )
        self._begin_materialization(record.job_id)
        worker = threading.Thread(
            target=self._materialize_validation_async_worker,
            args=(
                record,
                normalized_roots,
                normalized_overlays,
                attribution,
                sealed_overlay_manifest,
            ),
            name=f"zircon-validation-materialize-{record.job_id[:12]}",
            daemon=True,
        )
        worker.start()
        return WorkspaceCopyRecord(
            record.job_id,
            record.session_id,
            record.job_root,
            record.source_root,
            record.target_root,
            record.manifest,
            "materializing",
            record.external_sources,
        )

    def require_overlay_ownership(
        self,
        session_id: str,
        overlay_paths: tuple[str, ...] | list[str],
    ) -> tuple[str, ...]:
        """Normalize overlay paths and require current Session attribution."""
        normalized = tuple(
            sorted({self._normalize(path) for path in overlay_paths}, key=str.casefold)
        )
        attribution = self._session_attributions(session_id)
        unowned = sorted(
            (path for path in normalized if path.casefold() not in attribution),
            key=str.casefold,
        )
        if unowned:
            raise CoordinatorError(
                "validation_copy_overlay_not_owned",
                "Validation overlay paths require current Session attribution",
                details={"paths": unowned},
            )
        return normalized

    def _plan_validation_materialization(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str],
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ) -> tuple[
        WorkspaceCopyRecord,
        tuple[str, ...],
        tuple[str, ...],
        dict[str, str | None] | None,
    ]:
        normalized_roots = tuple(
            sorted({self._normalize(path) for path in dependency_roots}, key=str.casefold)
        )
        if not normalized_roots:
            raise CoordinatorError(
                "validation_copy_dependency_roots_empty",
                "Validation template must declare source dependencies",
            )
        pinned_commit = self._requested_baseline_commit(baseline_commit)
        dependency_paths = self._baseline_paths(pinned_commit, normalized_roots)
        if not dependency_paths:
            raise CoordinatorError(
                "validation_copy_dependencies_missing",
                "Validation template dependencies are absent from the pinned baseline",
            )
        sealed = (
            self._sealed_overlay_manifest(overlay_paths, sealed_overlay_manifest)
            if sealed_overlay_manifest is not None
            else None
        )
        normalized_overlays = (
            tuple(sealed)
            if sealed is not None
            else self.require_overlay_ownership(session_id, overlay_paths)
        )
        attribution = (
            None if sealed is not None else self._session_attributions(session_id)
        )
        record = self.plan(
            session_id,
            include_paths=tuple(sorted(set(dependency_paths) | set(normalized_overlays))),
            baseline_commit=pinned_commit,
        )
        return record, normalized_roots, normalized_overlays, attribution

    def _materialize_validation_async_worker(
        self,
        record: WorkspaceCopyRecord,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        attribution: dict[str, str | None] | None,
        sealed_overlay_manifest: Mapping[str, str | None] | None,
    ) -> None:
        try:
            self._materialize_validation_record(
                record,
                dependency_roots,
                overlay_paths,
                attribution,
                sealed_overlay_manifest,
            )
        except BaseException:
            return

    def _materialize_validation_record(
        self,
        record: WorkspaceCopyRecord,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        attribution: dict[str, str | None] | None,
        sealed_overlay_manifest: Mapping[str, str | None] | None,
    ) -> WorkspaceCopyRecord:
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            input_entries = self._extract_baseline_dependencies(record, dependency_roots)
            input_entries.update(
                self._overlay_sealed_sources(record, sealed_overlay_manifest)
                if sealed_overlay_manifest is not None
                else self._overlay_attributed_sources(
                    record,
                    overlay_paths,
                    attribution if attribution is not None else {},
                )
            )
            input_manifest_hash = self._input_manifest_hash_from_entries(input_entries)
            self._complete_materialization(record.job_id, input_manifest_hash)
        except BaseException as error:
            self._fail_materialization(
                record.job_id, error=error, stage="template_dependencies"
            )
            if record.job_root.exists():
                shutil.rmtree(record.job_root)
            raise
        return WorkspaceCopyRecord(
            record.job_id,
            record.session_id,
            record.job_root,
            record.source_root,
            record.target_root,
            record.manifest,
            "materialized",
            record.external_sources,
            input_manifest_hash,
        )

    def materialize_cargo(
        self,
        session_id: str,
        *,
        command: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str] = (),
        external_sources: tuple[dict[str, object], ...]
        | list[dict[str, object]] = (),
        metadata_runner=None,
        discover_external_sources: bool = False,
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError(
                "validation_copy_cargo_command_empty",
                "Cargo validation copy requires a command",
            )
        pinned_commit = self._requested_baseline_commit(baseline_commit)
        descriptors = tuple(
            ExternalGitSource.from_payload(payload) for payload in external_sources
        )
        normalized_overlays = self.require_overlay_ownership(
            session_id, overlay_paths
        )
        request_json = json.dumps(
            {
                "command": command_tuple,
                "overlayPaths": normalized_overlays,
                "overlayManifest": None,
                "externalSources": [
                    source.to_payload() for source in descriptors
                ],
                "discoverExternalSources": bool(discover_external_sources),
                "baselineCommit": pinned_commit,
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        record = self._plan_cargo_materialization(session_id, request_json)
        if not self._reserve_local_materialization(record.job_id):
            raise CoordinatorError(
                "validation_copy_materialization_busy",
                "Cargo validation copy is already materializing",
            )
        try:
            request = self._claim_cargo_materialization(record.job_id)
            if request is None:
                raise CoordinatorError(
                    "validation_copy_materialization_state_lost",
                    "Cargo validation copy could not enter closure planning",
                )
            self._materialize_cargo_request(
                record.job_id,
                request,
                metadata_runner=metadata_runner,
            )
        finally:
            self._release_local_materialization(record.job_id)
        return self.status(session_id, record.job_id)

    def _plan_cargo_closure_pinned(
        self,
        *,
        command: tuple[str, ...],
        descriptors: tuple[ExternalGitSource, ...],
        discover_external_sources: bool,
        overlays: tuple[str, ...],
        baseline_commit: str,
        metadata_runner=None,
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        planner_parent: Path | None = None,
    ):
        """Plan Cargo inputs from an immutable topology view.

        Cargo metadata is sensitive to manifests and build-script topology.  The
        live checkout can advance while a ticket waits in the FIFO, so metadata
        must run against the same commit and sealed overlay bytes that will later
        be archived into the validation copy.
        """
        if not baseline_commit:
            raise CoordinatorError(
                "validation_copy_baseline_missing",
                "Pinned Cargo closure planning requires a baseline commit",
            )
        if metadata_runner is not None:
            # Test/embedding metadata is already an immutable caller-provided
            # object.  Keep the historical planner injection semantics while
            # the production path below performs real metadata in the view.
            return CargoInputClosurePlanner(
                self.repo_root, metadata_runner=metadata_runner
            ).plan(
                command,
                external_sources=descriptors,
                discover_external_sources=discover_external_sources,
                external_archive_loader=(
                    self._object_store.get if self._object_store is not None else None
                ),
                overlay_paths=overlays,
                baseline_commit=baseline_commit,
            )
        overlay_files = self._cargo_planner_overlay_files(
            overlays,
            sealed_overlay_manifest=sealed_overlay_manifest,
        )
        # A planner view is intentionally ephemeral and contains manifests/config
        # plus target topology, not the full source tree. Production workers pass
        # their already-registered job root so artifact governance can see the
        # temporary directory for the entire metadata operation.
        owns_planner_parent = planner_parent is None
        if planner_parent is None:
            root = max(
                self.target_roots,
                key=lambda value: shutil.disk_usage(value.anchor or value.parent).free,
            )
            verify_root = self._managed_verify_root(root)
            verify_root.mkdir(parents=True, exist_ok=True)
            planner_parent = verify_root / f".cargo-planner-{uuid.uuid4().hex}"
            planner_parent.mkdir(parents=True, exist_ok=False)
        else:
            planner_parent = Path(planner_parent).resolve()
            self._validate_job_root(planner_parent)
            planner_parent.mkdir(parents=True, exist_ok=True)
        try:
            with PinnedCargoPlannerView(
                self.repo_root,
                planner_parent,
                baseline_commit=baseline_commit,
                overlay_files=overlay_files,
                external_sources=descriptors,
                discover_external_sources=discover_external_sources,
            ) as view:
                return PinnedCargoInputClosurePlanner(
                    view,
                ).plan_pinned(
                    command,
                    external_sources=descriptors,
                    discover_external_sources=discover_external_sources,
                    overlay_paths=overlays,
                    baseline_commit=baseline_commit,
                )
        finally:
            if owns_planner_parent:
                shutil.rmtree(planner_parent, ignore_errors=False)

    def _cargo_planner_overlay_files(
        self,
        overlays: tuple[str, ...],
        *,
        sealed_overlay_manifest: Mapping[str, str | None] | None,
    ) -> dict[str, bytes | None]:
        if sealed_overlay_manifest is not None:
            if self._object_store is None:
                raise CoordinatorError(
                    "validation_copy_source_store_unavailable",
                    "Sealed Cargo planner overlays require the coordinator object store",
                )
            sealed = self._sealed_overlay_manifest(
                overlays, sealed_overlay_manifest
            )
            return {
                path: (
                    None
                    if object_hash is None
                    else self._object_store.get(object_hash)
                )
                for path, object_hash in sealed.items()
            }

        result: dict[str, bytes | None] = {}
        for path in overlays:
            source = self.repo_root / path
            # Ownership was checked before this helper.  The materializer later
            # rechecks the attributed hash; reading current bytes here preserves
            # the existing typed stale-attribution failure while ensuring Cargo
            # never consults any other live-worktree path.
            result[path] = source.read_bytes() if source.is_file() else None
        return result

    def materialize_cargo_async(
        self,
        session_id: str,
        *,
        command: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str] = (),
        external_sources: tuple[dict[str, object], ...]
        | list[dict[str, object]] = (),
        metadata_runner=None,
        discover_external_sources: bool = False,
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ) -> WorkspaceCopyRecord:
        """Durably accept a Cargo copy before its closure is planned or copied.

        Cargo metadata, external-repository pinning, Git archive extraction, and full
        input hashing can each be much slower than an API request budget.  Persisting
        the request first makes the acceptance durable and lets a worker perform every
        filesystem- or Cargo-adjacent operation after the HTTP response has returned.
        ``metadata_runner`` is test-only injection and intentionally never persisted.
        """
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError(
                "validation_copy_cargo_command_empty",
                "Cargo validation copy requires a command",
            )
        pinned_commit = (
            self._requested_baseline_commit(baseline_commit)
            if baseline_commit is not None
            else None
        )
        # Persist raw request strings.  Path normalization resolves against the
        # workspace, so it belongs to the claimed worker along with ownership
        # validation rather than the bounded acknowledgement path.
        overlays = tuple(sorted({str(path) for path in overlay_paths}, key=str.casefold))
        try:
            payload = json.dumps(
                {
                    "command": command_tuple,
                    "overlayPaths": overlays,
                    "overlayManifest": (
                        dict(sealed_overlay_manifest)
                        if sealed_overlay_manifest is not None
                        else None
                    ),
                    "externalSources": list(external_sources),
                    "discoverExternalSources": bool(discover_external_sources),
                    "baselineCommit": pinned_commit,
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "validation_copy_cargo_request_invalid",
                "Cargo validation-copy request must contain JSON-compatible descriptors",
            ) from error

        record = self._plan_cargo_materialization(session_id, payload)
        self._spawn_cargo_materialization_worker(record.job_id, metadata_runner=metadata_runner)
        return record

    def _plan_cargo_materialization(
        self, session_id: str, request_json: str
    ) -> WorkspaceCopyRecord:
        self._require_session(session_id)
        job_id = uuid.uuid4().hex
        # This is only a durable placeholder.  The background worker selects and
        # validates a target root, then pins Git HEAD before touching the copy.  In
        # particular, request acknowledgement must not wait on disk probes, path
        # resolution, or Git when a host is overloaded.
        job_root = self.target_roots[0] / "verify" / job_id
        source_root = job_root / "source"
        target_root = job_root / "target"
        with self.database.transaction() as connection:
            self._require_cleanup_available(connection, job_root)
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root, head_commit,
                    manifest_json, status, created_at, external_sources_json,
                    materialization_kind, materialization_request_json,
                    materialization_phase, materialization_attempt
                ) VALUES (?, ?, ?, ?, ?, ?, '[]', 'planned', ?, '[]', 'cargo', ?, 'accepted', 0)
                """,
                (
                    job_id,
                    session_id,
                    str(job_root),
                    str(source_root),
                    str(target_root),
                    "pending",
                    utc_text(),
                    request_json,
                ),
            )
        return WorkspaceCopyRecord(
            job_id,
            session_id,
            job_root,
            source_root,
            target_root,
            (),
            "materializing",
            materialization_phase="accepted",
            materialization_kind="cargo",
        )

    def _require_cleanup_available(self, connection, job_root: Path) -> None:
        reservation = overlapping_cleanup_reservation(
            connection, target_identity(job_root)
        )
        if reservation is None:
            return
        raise CoordinatorError(
            "validation_copy_cleanup_reserved",
            "Validation copy overlaps a target with deletion already in progress",
            details={"reservedTarget": str(reservation["target_dir"])},
        )

    def _spawn_cargo_materialization_worker(self, job_id: str, *, metadata_runner=None) -> None:
        worker = threading.Thread(
            target=self._materialize_cargo_async_worker,
            args=(job_id, metadata_runner),
            name=f"zircon-cargo-materialize-{job_id[:12]}",
            daemon=True,
        )
        worker.start()

    def _materialize_cargo_async_worker(self, job_id: str, metadata_runner) -> None:
        if not self._reserve_local_materialization(job_id):
            return
        try:
            request = self._claim_cargo_materialization(job_id)
            if request is None:
                return
            self._materialize_cargo_request(job_id, request, metadata_runner=metadata_runner)
        except BaseException:
            # The worker persists a typed terminal failure.  Detached callers only
            # observe durable status and must never receive an unhandled exception.
            return
        finally:
            self._release_local_materialization(job_id)

    def _claim_cargo_materialization(self, job_id: str) -> dict[str, object] | None:
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            row = connection.execute(
                "SELECT * FROM validation_copies WHERE job_id=?", (job_id,)
            ).fetchone()
            if (
                row is None
                or row["status"] != "planned"
                or row["materialization_kind"] != "cargo"
                or row["materialization_phase"] != "accepted"
            ):
                return None
            try:
                payload = json.loads(str(row["materialization_request_json"] or "{}"))
            except json.JSONDecodeError:
                self._terminalize_invalid_cargo_request_in_connection(connection, job_id)
                return None
            if not isinstance(payload, dict):
                self._terminalize_invalid_cargo_request_in_connection(connection, job_id)
                return None
            cursor = connection.execute(
                """
                UPDATE validation_copies
                SET materialization_phase='closure_planning',
                    materialization_started_at=?, materialization_worker_id=?,
                    materialization_attempt=materialization_attempt+1
                WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
                  AND materialization_phase='accepted'
                """,
                (utc_text(), self._materialization_worker_id, job_id),
            )
            if cursor.rowcount != 1:
                return None
            return payload

    @staticmethod
    def _terminalize_invalid_cargo_request_in_connection(connection, job_id: str) -> None:
        """Make a corrupt accepted request terminal before a worker can release it."""
        connection.execute(
            """
            UPDATE validation_copies
            SET status='failed', materialization_started_at=NULL,
                materialization_phase='failed',
                error_code='validation_copy_cargo_request_invalid',
                error_stage='request_decode', error_path=NULL, error_details_json='{}'
            WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
              AND materialization_phase='accepted'
            """,
            (job_id,),
        )

    def _materialize_cargo_request(
        self, job_id: str, request: dict[str, object], *, metadata_runner
    ) -> None:
        stage = "artifact_governance"
        try:
            self._run_cargo_materialization_preflight()
            stage = "root_preparation"
            raw_baseline_commit = request.get("baselineCommit")
            if raw_baseline_commit is not None and not isinstance(
                raw_baseline_commit, str
            ):
                raise CoordinatorError(
                    "validation_copy_cargo_request_invalid",
                    "Cargo baseline commit must be a full Git object ID",
                )
            row = self._prepare_cargo_materialization_root(
                job_id, baseline_commit=raw_baseline_commit
            )
            stage = "closure_planning"
            command = tuple(str(part) for part in request.get("command") or () if str(part))
            raw_external = request.get("externalSources")
            if raw_external is None:
                raw_external = []
            if not isinstance(raw_external, list):
                raise CoordinatorError(
                    "validation_copy_cargo_request_invalid",
                    "Cargo external sources must be a list",
                )
            descriptors = tuple(
                ExternalGitSource.from_payload(payload)
                for payload in raw_external
                if isinstance(payload, Mapping)
            )
            if len(descriptors) != len(raw_external):
                raise CoordinatorError(
                    "validation_copy_external_source_invalid",
                    "Cargo external source descriptor must be an object",
                )
            raw_overlays = request.get("overlayPaths")
            if raw_overlays is None:
                raw_overlays = []
            if not isinstance(raw_overlays, list):
                raise CoordinatorError(
                    "validation_copy_cargo_request_invalid",
                    "Cargo overlay paths must be a list",
                )
            if any(not isinstance(path, str) for path in raw_overlays):
                raise CoordinatorError(
                    "validation_copy_cargo_request_invalid",
                    "Cargo overlay paths must contain strings",
                )
            raw_overlay_manifest = request.get("overlayManifest")
            if raw_overlay_manifest is None:
                sealed_overlay_manifest = None
                stage = "overlay_ownership"
                overlays = self.require_overlay_ownership(
                    str(row["session_id"]), raw_overlays
                )
            else:
                if not isinstance(raw_overlay_manifest, Mapping):
                    raise CoordinatorError(
                        "validation_copy_cargo_request_invalid",
                        "Cargo sealed overlay manifest must be an object",
                    )
                stage = "sealed_overlay"
                sealed_overlay_manifest = self._sealed_overlay_manifest(
                    raw_overlays, raw_overlay_manifest
                )
                overlays = tuple(sealed_overlay_manifest)
            stage = "closure_planning"
            closure = self._plan_cargo_closure_pinned(
                command=command,
                descriptors=descriptors,
                discover_external_sources=bool(
                    request.get("discoverExternalSources")
                ),
                overlays=overlays,
                baseline_commit=str(row["head_commit"]),
                metadata_runner=metadata_runner,
                sealed_overlay_manifest=sealed_overlay_manifest,
                planner_parent=Path(str(row["job_root"])),
            )
            paths = tuple(
                sorted(set(closure.repository_paths) | set(overlays), key=str.casefold)
            )
            stage = "materialization_prepare"
            record = self._persist_cargo_closure(
                row,
                paths,
                tuple(source.to_payload() for source in closure.external_sources),
            )
            self._require_untracked_overlay_attribution(
                record,
                sealed_overlay_paths=(
                    tuple(sealed_overlay_manifest)
                    if sealed_overlay_manifest is not None
                    else ()
                ),
            )
            self._materialize_record(
                record,
                worker_id=self._materialization_worker_id,
                sealed_overlay_manifest=sealed_overlay_manifest,
            )
        except BaseException as error:
            self._fail_materialization(
                job_id,
                error=error,
                stage=stage,
                worker_id=self._materialization_worker_id,
            )
            raise

    def _prepare_cargo_materialization_root(
        self, job_id: str, *, baseline_commit: str | None = None
    ):
        """Select a verified target and persist the request's pinned baseline."""
        root = max(
            self.target_roots,
            key=lambda value: shutil.disk_usage(value.anchor or value.parent).free,
        )
        verify_root = self._managed_verify_root(root)
        job_root = (verify_root / job_id).resolve()
        if job_root.parent != verify_root:
            raise CoordinatorError(
                "validation_copy_verify_escape", "Validation job escaped the managed verify root"
            )
        source_root = job_root / "source"
        target_root = job_root / "target"
        head_commit = self._requested_baseline_commit(baseline_commit)
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            self._require_cleanup_available(connection, job_root)
            cursor = connection.execute(
                """
                UPDATE validation_copies
                SET job_root=?, source_root=?, target_root=?, head_commit=?
                WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
                  AND materialization_phase='closure_planning'
                  AND materialization_worker_id=?
                """,
                (
                    str(job_root),
                    str(source_root),
                    str(target_root),
                    head_commit,
                    job_id,
                    self._materialization_worker_id,
                ),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_materialization_state_lost",
                    "Cargo validation copy changed state while its root was prepared",
                )
            row = connection.execute(
                "SELECT * FROM validation_copies WHERE job_id=?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("validation_copy_not_found", "Unknown validation-copy job")
        try:
            # Register the job row before creating its directory. Artifact
            # governance can therefore treat the planner view as managed even
            # when an audit races this worker.
            job_root.mkdir(parents=True, exist_ok=False)
        except FileExistsError as error:
            raise CoordinatorError(
                "validation_copy_verify_root_exists",
                "Cargo validation job root already exists before materialization",
                details={"jobRoot": str(job_root)},
            ) from error
        except OSError as error:
            raise CoordinatorError(
                "validation_copy_verify_root_create_failed",
                "Cargo validation job root could not be created",
                details={"jobRoot": str(job_root), "errorType": type(error).__name__},
            ) from error
        return row

    def _persist_cargo_closure(
        self,
        row,
        paths: tuple[str, ...],
        external_sources: tuple[dict[str, object], ...],
    ) -> WorkspaceCopyRecord:
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            cursor = connection.execute(
                """
                UPDATE validation_copies
                SET manifest_json=?, external_sources_json=?, materialization_phase='materializing'
                WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
                  AND materialization_phase='closure_planning'
                  AND materialization_worker_id=?
                """,
                (
                    json.dumps(paths),
                    json.dumps(external_sources, sort_keys=True),
                    str(row["job_id"]),
                    self._materialization_worker_id,
                ),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_materialization_state_lost",
                    "Cargo validation copy changed state while its closure was planned",
                )
        return WorkspaceCopyRecord(
            str(row["job_id"]),
            str(row["session_id"]),
            Path(str(row["job_root"])),
            Path(str(row["source_root"])),
            Path(str(row["target_root"])),
            paths,
            "materializing",
            external_sources,
            materialization_phase="materializing",
            materialization_kind="cargo",
        )

    def run(
        self, session_id: str, job_id: str, *, command: tuple[str, ...] | list[str]
    ) -> ValidationRunEvidence:
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError(
                "validation_copy_command_empty", "Validation command cannot be empty"
            )
        self._reserve_local_run(job_id)
        try:
            with self.database.transaction() as connection:
                row = connection.execute(
                    "SELECT * FROM validation_copies WHERE job_id = ?", (job_id,)
                ).fetchone()
                if row is None:
                    raise CoordinatorError(
                        "validation_copy_not_found", f"Unknown validation-copy job: {job_id}"
                    )
                if row["session_id"] != session_id:
                    raise CoordinatorError(
                        "validation_copy_foreign_session",
                        "Validation copy belongs to another Session",
                    )
                cursor = connection.execute(
                    """
                    UPDATE validation_copies SET status = 'running', run_pid = NULL
                    WHERE job_id = ? AND status = 'materialized'
                    """,
                    (job_id,),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_not_materialized",
                        "Validation copy is already running or unavailable",
                    )
        except BaseException:
            self._release_local_run(job_id)
            raise
        run_id = uuid.uuid4().hex
        started_at = utc_text()
        process: subprocess.Popen[str] | None = None
        job_root = Path(row["job_root"]).resolve()
        process_started = False
        evidence_persisted = False
        completion_succeeded = False
        evidence: ValidationRunEvidence | None = None
        try:
            source_root = Path(row["source_root"]).resolve()
            target_root = Path(row["target_root"]).resolve()
            self._validate_job_root(job_root)
            if source_root.parent != job_root or target_root.parent != job_root:
                raise CoordinatorError(
                    "validation_copy_path_not_managed",
                    "Validation-copy run roots escaped the job root",
                )
            environment = benchmark_child_environment(target_root)
            process = subprocess.Popen(
                command_tuple,
                cwd=source_root,
                env=environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            process_started = True
            with self._running_lock:
                self._running_processes[job_id] = process
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    """UPDATE validation_copies SET run_pid = ?
                       WHERE job_id = ? AND status = 'running'""",
                    (process.pid, job_id),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_terminal_state_changed",
                        "Validation copy changed state while registering its process",
                    )
            exit_code, stdout, stderr = self._terminal.collect(
                process,
            )
            evidence = self._terminal.persist(
                run_id=run_id,
                job_id=job_id,
                session_id=session_id,
                command=command_tuple,
                exit_code=exit_code,
                stdout=stdout,
                stderr=stderr,
                started_at=started_at,
            )
            evidence_persisted = True
            self._terminal.notify_completion(self._completion_hook, run_id)
            self._terminal.finalize_success(job_id)
            completion_succeeded = True
        except BaseException as error:
            if process is not None and process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            if evidence_persisted:
                self._terminal.preserve_completion_failure(
                    error=error,
                    run_id=run_id,
                    job_id=job_id,
                    session_id=session_id,
                )
            self._terminal.restore_after_failure(
                job_id,
                process_started=process_started,
                evidence_persisted=evidence_persisted,
            )
            raise
        finally:
            if completion_succeeded:
                self._cleanup_terminal_copy(session_id, job_root)
            self._release_local_run(job_id)
        if evidence is None:
            raise CoordinatorError(
                "validation_copy_terminal_evidence_missing",
                "Validation process completed without durable terminal evidence",
            )
        return evidence

    def start(
        self,
        session_id: str,
        job_id: str,
        *,
        command: tuple[str, ...] | list[str],
        run_id: str | None = None,
        benchmark_grant_id: str | None = None,
        environment: Mapping[str, str] | None = None,
    ) -> dict[str, object]:
        """Launch a managed validation and return after the process is registered."""
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError(
                "validation_copy_command_empty", "Validation command cannot be empty"
            )
        run_id = run_id or uuid.uuid4().hex
        with self.database.connect() as connection:
            route_row = connection.execute(
                "SELECT * FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
        if (
            route_row is not None
            and route_row["materialization_kind"] == "cargo"
            and benchmark_grant_id is None
            and self._is_linked_validation_ticket_run(
                session_id=session_id,
                job_id=job_id,
                run_id=run_id,
                command=command_tuple,
            )
        ):
            return self._advance_cargo_execution(
                session_id=session_id,
                row=route_row,
                command=command_tuple,
                run_id=run_id,
                environment=environment,
            )
        self._reserve_local_run(job_id)
        try:
            with self.database.transaction() as connection:
                row = connection.execute(
                    "SELECT * FROM validation_copies WHERE job_id = ?", (job_id,)
                ).fetchone()
                if row is None:
                    raise CoordinatorError(
                        "validation_copy_not_found", f"Unknown validation-copy job: {job_id}"
                    )
                environment_values = benchmark_run_environment(
                    environment, input_manifest_hash=row["input_manifest_hash"]
                )
                if benchmark_grant_id is None:
                    if row["session_id"] != session_id:
                        raise CoordinatorError(
                            "validation_copy_foreign_session",
                            "Validation copy belongs to another Session",
                        )
                    if environment is not None:
                        raise CoordinatorError(
                            "validation_copy_benchmark_grant_required",
                            "Benchmark child identity requires a Coordinator-issued grant",
                        )
                else:
                    require_benchmark_launch_grant(
                        connection,
                        grant_id=benchmark_grant_id,
                        session_id=session_id,
                        job_id=job_id,
                        copy_row=row,
                        command=command_tuple,
                        environment=environment_values,
                        validation_run_id=run_id,
                        required_copy_status="materialized",
                    )
                expected_input_manifest_hash = (
                    str(row["input_manifest_hash"])
                    if benchmark_grant_id is not None
                    else None
                )
                cursor = connection.execute(
                    """UPDATE validation_copies SET status = 'running', run_pid = NULL
                       WHERE job_id = ? AND status = 'materialized'""",
                    (job_id,),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_not_materialized",
                        "Validation copy is already running or unavailable",
                    )
        except BaseException:
            self._release_local_run(job_id)
            raise
        started_at = utc_text()
        process: subprocess.Popen[str] | None = None
        root_process_creation_time: str | None = None
        process_job: int | None = None
        process_job_registered = False
        collector_decision = threading.Event()
        collector_authorized = threading.Event()
        collector_thread: threading.Thread | None = None
        collector_started = False
        try:
            source_root = Path(row["source_root"]).resolve()
            target_root = Path(row["target_root"]).resolve()
            job_root = Path(row["job_root"]).resolve()
            self._validate_job_root(job_root)
            if source_root.parent != job_root or target_root.parent != job_root:
                raise CoordinatorError(
                    "validation_copy_path_not_managed",
                    "Validation-copy run roots escaped the job root",
                )
            child_environment = benchmark_child_environment(
                target_root, benchmark_environment=environment_values
            )
            if expected_input_manifest_hash is not None:
                current_input_manifest_hash = self._input_manifest_hash_for_roots(
                    job_root, target_root
                )
                if current_input_manifest_hash != expected_input_manifest_hash:
                    raise CoordinatorError(
                        "validation_copy_benchmark_manifest_stale",
                        "Materialized benchmark copy changed after its immutable identity was recorded",
                    )
            if benchmark_grant_id is not None:
                process, process_job = create_atomic_kill_on_close_process(
                    command_tuple,
                    cwd=source_root,
                    env=child_environment,
                )
                root_process_creation_time = popen_process_creation_time(process)
                if not root_process_creation_time or root_process_creation_time == "unknown":
                    raise CoordinatorError(
                        "validation_copy_benchmark_process_identity_unavailable",
                        "Benchmark root process creation time could not be recorded",
                    )
            else:
                process = subprocess.Popen(
                    command_tuple,
                    cwd=source_root,
                    env=child_environment,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                )
            with self._running_lock:
                self._running_processes[job_id] = process
                if process_job is not None:
                    self._running_process_jobs[job_id] = process_job
                    process_job_registered = True

            def finish_after_launch_decision() -> None:
                collector_decision.wait()
                if collector_authorized.is_set():
                    self._finish_started_run(
                        session_id,
                        job_id,
                        run_id,
                        command_tuple,
                        started_at,
                        process,
                        benchmark_grant_id is not None,
                    )

            collector_thread = threading.Thread(
                target=finish_after_launch_decision,
                name=f"zircon-validation-{job_id[:12]}",
                daemon=True,
            )
            collector_thread.start()
            collector_started = True
            with self.database.transaction() as connection:
                registered_copy = connection.execute(
                    "SELECT * FROM validation_copies WHERE job_id=?", (job_id,)
                ).fetchone()
                if benchmark_grant_id is not None:
                    require_benchmark_launch_grant(
                        connection,
                        grant_id=benchmark_grant_id,
                        session_id=session_id,
                        job_id=job_id,
                        copy_row=registered_copy,
                        command=command_tuple,
                        environment=environment_values,
                        validation_run_id=run_id,
                        required_copy_status="running",
                    )
                cursor = connection.execute(
                    "UPDATE validation_copies SET run_pid = ? WHERE job_id = ? AND status = 'running'",
                    (process.pid, job_id),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_terminal_state_changed",
                        "Validation copy changed state while registering its process",
                    )
                if benchmark_grant_id is not None:
                    binding_cursor = connection.execute(
                        """UPDATE workflow_validation_bindings
                           SET root_pid=?, root_process_creation_time=?
                           WHERE validation_run_id=? AND benchmark_grant_id=?
                             AND job_id=? AND session_id=? AND root_pid IS NULL
                             AND root_process_creation_time IS NULL""",
                        (
                            process.pid,
                            root_process_creation_time,
                            run_id,
                            benchmark_grant_id,
                            job_id,
                            session_id,
                        ),
                    )
                    if binding_cursor.rowcount != 1:
                        raise CoordinatorError(
                            "validation_copy_benchmark_binding_changed",
                            "Benchmark workflow binding changed while registering its process",
                        )
                    grant_cursor = connection.execute(
                        """UPDATE benchmark_validation_grants
                           SET status='consumed', consumed_at=?, validation_run_id=?,
                               root_pid=?, root_process_creation_time=?, job_isolated=?
                           WHERE grant_id=? AND status='launching'
                             AND job_id=? AND target_session_id=?""",
                        (
                            utc_text(),
                            run_id,
                            process.pid,
                            root_process_creation_time,
                            1 if process_job is not None else 0,
                            benchmark_grant_id,
                            job_id,
                            session_id,
                        ),
                    )
                    if grant_cursor.rowcount != 1:
                        raise CoordinatorError(
                            "validation_copy_benchmark_grant_changed",
                            "Benchmark grant changed while registering its process",
                        )
            if benchmark_grant_id is not None:
                resume_popen_process(process)
            collector_authorized.set()
            collector_decision.set()
        except BaseException:
            try:
                try:
                    if benchmark_grant_id is not None and process_job_registered:
                        self._terminate_running_process_job(job_id)
                    elif process is not None and process.poll() is None:
                        process.kill()
                        process.wait(timeout=5)
                finally:
                    collector_decision.set()
                    if collector_started and collector_thread is not None:
                        collector_thread.join()
                    with self.database.transaction() as connection:
                        cursor = connection.execute(
                            "UPDATE validation_copies SET status = 'materialized', run_pid = NULL "
                            "WHERE job_id = ? AND status = 'running'",
                            (job_id,),
                        )
                        if cursor.rowcount != 1:
                            raise CoordinatorError(
                                "validation_copy_terminal_state_changed",
                                "Validation copy changed state while rolling back its launch",
                            )
                        if benchmark_grant_id is not None:
                            connection.execute(
                                """UPDATE benchmark_validation_grants
                                   SET status='launching', consumed_at=NULL,
                                       validation_run_id=NULL, root_pid=NULL,
                                       root_process_creation_time=NULL, job_isolated=0
                                   WHERE grant_id=? AND status='consumed'
                                     AND validation_run_id=?""",
                                (benchmark_grant_id, run_id),
                            )
                            connection.execute(
                                """UPDATE workflow_validation_bindings
                                   SET root_pid=NULL, root_process_creation_time=NULL
                                   WHERE validation_run_id=?
                                     AND benchmark_grant_id=?""",
                                (run_id, benchmark_grant_id),
                            )
            finally:
                self._release_local_run(job_id)
                if process_job is not None and not process_job_registered:
                    close_process_job(process_job)
            raise
        result: dict[str, object] = {
            "jobId": job_id,
            "runId": run_id,
            "pid": process.pid,
            "status": "running",
        }
        if root_process_creation_time is not None:
            result["processCreationTime"] = root_process_creation_time
        return result

    def _is_linked_validation_ticket_run(
        self,
        *,
        session_id: str,
        job_id: str,
        run_id: str,
        command: tuple[str, ...],
    ) -> bool:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT ticket.command_json, event.payload_json
                   FROM validation_tickets ticket
                   JOIN validation_ticket_events event
                     ON event.ticket_id=ticket.ticket_id
                   WHERE ticket.ticket_id=? AND ticket.session_id=?
                     AND event.event_type='validation.ticket_copy_linked'
                   ORDER BY event.event_id DESC LIMIT 1""",
                (run_id, session_id),
            ).fetchone()
        if row is None:
            return False
        try:
            stored_command = json.loads(str(row["command_json"]))
            link = json.loads(str(row["payload_json"]))
        except (TypeError, ValueError, json.JSONDecodeError):
            return False
        return stored_command == list(command) and link.get("jobId") == job_id

    def _advance_cargo_execution(
        self,
        *,
        session_id: str,
        row,
        command: tuple[str, ...],
        run_id: str,
        environment: Mapping[str, str] | None,
    ) -> dict[str, object]:
        if row["session_id"] != session_id:
            raise CoordinatorError(
                "validation_copy_foreign_session",
                "Validation copy belongs to another Session",
            )
        if row["status"] != "materialized":
            raise CoordinatorError(
                "validation_copy_not_materialized",
                "Validation copy is already running or unavailable",
            )
        if environment is not None:
            raise CoordinatorError(
                "validation_copy_benchmark_grant_required",
                "Benchmark child identity requires a Coordinator-issued grant",
            )
        if self._cargo_execution is None:
            raise CoordinatorError(
                "validation_copy_cargo_execution_unavailable",
                "Cargo validation copy has no managed Cargo executor",
            )
        existing = self._terminal.latest_for_job(
            session_id=session_id, job_id=str(row["job_id"])
        )
        if existing is not None and existing.run_id == run_id:
            return {
                "jobId": str(row["job_id"]),
                "runId": run_id,
                "status": "completed",
                "exitCode": existing.exit_code,
                "stdoutTail": existing.stdout,
                "stderrTail": existing.stderr,
            }
        execution_command = self._cargo_command_for_materialized_copy(row, command)
        progress = self._cargo_execution.advance(
            session_id=session_id,
            copy_job_id=str(row["job_id"]),
            source_root=Path(str(row["source_root"])).resolve(),
            input_manifest_hash=row["input_manifest_hash"],
            command=execution_command,
            validation_run_id=run_id,
        )
        result = {"jobId": str(row["job_id"]), "runId": run_id, **progress}
        if progress.get("status") != "completed":
            return result
        evidence = self._terminal.persist(
            run_id=run_id,
            job_id=str(row["job_id"]),
            session_id=session_id,
            command=command,
            exit_code=int(progress.get("exitCode", -1)),
            stdout=str(progress.get("stdoutTail") or ""),
            stderr=str(progress.get("stderrTail") or ""),
            started_at=str(progress.get("startedAt") or utc_text()),
        )
        self._terminal.notify_completion(self._completion_hook, run_id)
        return {
            **result,
            "exitCode": evidence.exit_code,
            "stdoutTail": evidence.stdout,
            "stderrTail": evidence.stderr,
        }

    def _cargo_command_for_materialized_copy(
        self, row, command: tuple[str, ...]
    ) -> tuple[str, ...]:
        if not is_direct_cargo_command(command):
            opaque = " ".join(command).casefold()
            unsafe = (
                "--manifest-path",
                "--config",
                "--target-dir",
                "--build-dir",
            )
            if any(argument in opaque for argument in unsafe):
                raise CoordinatorError(
                    "validation_copy_cargo_command_opaque",
                    "Legacy shell-wrapped Cargo commands cannot carry source or storage path overrides",
                )
            return command

        validate_cargo_storage_arguments(command)
        job_root = Path(str(row["job_root"])).resolve()
        source_root = Path(str(row["source_root"])).resolve()
        try:
            raw_sources = json.loads(str(row["external_sources_json"] or "[]"))
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "Validation copy external source descriptors are not valid JSON",
            ) from error
        if not isinstance(raw_sources, list):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "Validation copy external source descriptors must be an array",
            )
        external_sources = tuple(
            ExternalGitSource.from_payload(payload)
            for payload in raw_sources
            if isinstance(payload, Mapping)
        )
        if len(external_sources) != len(raw_sources):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "Validation copy external source descriptors must be objects",
            )
        mappings = [(self.repo_root, source_root)]
        mappings.extend(
            (source.repo_root, (job_root / source.mount_path).resolve())
            for source in external_sources
        )
        physical_roots = tuple(physical for _logical, physical in mappings)

        def rewrite(option: str, value: str) -> str:
            if option == "--config" and inline_cargo_config_key(value) is not None:
                validate_inline_cargo_config(value)
                return value
            candidate = Path(value)
            if candidate.is_absolute():
                normalized = candidate.resolve(strict=False)
                for physical in physical_roots:
                    if normalized == physical or normalized.is_relative_to(physical):
                        return str(normalized)
                for logical, physical in mappings:
                    if normalized == logical or normalized.is_relative_to(logical):
                        return str(physical / normalized.relative_to(logical))
                raise CoordinatorError(
                    "validation_copy_cargo_source_path_unpinned",
                    "Cargo source path is outside every pinned validation repository",
                    details={"option": option, "path": value},
                )
            relative = PurePosixPath(value.replace("\\", "/"))
            if (
                not relative.parts
                or any(part in {"", ".", ".."} or ":" in part for part in relative.parts)
            ):
                raise CoordinatorError(
                    "validation_copy_cargo_source_path_unpinned",
                    "Cargo source path must be a normalized path inside the validation copy",
                    details={"option": option, "path": value},
                )
            resolved = source_root.joinpath(*relative.parts).resolve(strict=False)
            if not resolved.is_relative_to(source_root):
                raise CoordinatorError(
                    "validation_copy_cargo_source_path_unpinned",
                    "Cargo source path escaped the validation copy",
                    details={"option": option, "path": value},
                )
            return relative.as_posix()

        return rewrite_cargo_source_path_arguments(command, rewrite)

    def terminate_interrupted_benchmark(
        self,
        *,
        grant_id: str,
        job_id: str,
        root_pid: int,
        process_creation_time: str,
        job_isolated: bool,
    ) -> None:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT copy.status, copy.run_pid, grant.job_isolated
                   FROM benchmark_validation_grants grant
                   JOIN validation_copies copy ON copy.job_id=grant.job_id
                   WHERE grant.grant_id=? AND grant.job_id=?
                     AND grant.status='consumed' AND grant.root_pid=?
                     AND grant.root_process_creation_time=?""",
                (grant_id, job_id, root_pid, process_creation_time),
            ).fetchone()
        if row is None or bool(row["job_isolated"]) != job_isolated:
            raise CoordinatorError(
                "benchmark_validation_recovery_state_changed",
                "Interrupted benchmark no longer matches its durable launch state",
            )
        status = str(row["status"])
        run_pid = int(row["run_pid"] or 0)
        recoverable_state = (
            (status == "running" and run_pid == root_pid)
            or (status == "materialized" and run_pid == 0)
            or (job_isolated and status == "failed" and run_pid == 0)
        )
        if not recoverable_state:
            raise CoordinatorError(
                "benchmark_validation_recovery_state_changed",
                "Interrupted benchmark copy no longer matches its durable process state",
            )
        try:
            if job_isolated:
                confirm_kill_on_close_job_terminated(root_pid, process_creation_time)
            else:
                terminate_process_tree(root_pid, process_creation_time)
        except (OSError, subprocess.SubprocessError, TimeoutError, ValueError) as error:
            raise CoordinatorError(
                "benchmark_validation_recovery_termination_failed",
                "Interrupted benchmark process identity could not be terminated safely",
            ) from error
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """UPDATE validation_copies
                   SET status='materialized', run_pid=NULL
                   WHERE job_id=? AND status='running' AND run_pid=?""",
                (job_id, root_pid),
            )
            if cursor.rowcount == 0:
                current = connection.execute(
                    "SELECT status, run_pid FROM validation_copies WHERE job_id=?",
                    (job_id,),
                ).fetchone()
                current_is_preserved_terminal = (
                    current is not None
                    and current["run_pid"] is None
                    and (
                        current["status"] == "materialized"
                        or (job_isolated and current["status"] == "failed")
                    )
                )
                if not current_is_preserved_terminal:
                    raise CoordinatorError(
                        "benchmark_validation_recovery_state_changed",
                        "Validation copy changed state during benchmark recovery",
                    )

    def _finish_started_run(
        self,
        session_id: str,
        job_id: str,
        run_id: str,
        command: tuple[str, ...],
        started_at: str,
        process: subprocess.Popen[str],
        preserve_copy: bool = False,
    ) -> None:
        job_root: Path | None = None
        evidence_persisted = False
        completion_succeeded = False
        try:
            job_root = Path(
                self._validation_copy_row(job_id)["job_root"]
            ).resolve()
            exit_code, stdout, stderr = self._terminal.collect(
                process,
                after_root_exit=(
                    lambda: self._terminate_running_process_job(job_id)
                    if preserve_copy
                    else None
                ),
            )
            self._terminal.persist(
                run_id=run_id,
                job_id=job_id,
                session_id=session_id,
                command=command,
                exit_code=exit_code,
                stdout=stdout,
                stderr=stderr,
                started_at=started_at,
            )
            evidence_persisted = True
            self._terminal.notify_completion(self._completion_hook, run_id)
            self._terminal.finalize_success(job_id)
            completion_succeeded = True
        except BaseException as error:
            try:
                if preserve_copy:
                    self._terminate_running_process_job(job_id)
                elif process.poll() is None:
                    process.kill()
                    process.wait(timeout=5)
            except BaseException:
                # Terminal recovery below must remain authoritative even when
                # process cleanup reports a secondary failure.
                pass
            if evidence_persisted:
                self._terminal.preserve_completion_failure(
                    error=error,
                    run_id=run_id,
                    job_id=job_id,
                    session_id=session_id,
                )
            self._terminal.restore_after_failure(
                job_id,
                process_started=True,
                evidence_persisted=evidence_persisted,
            )
        finally:
            if completion_succeeded and not preserve_copy and job_root is not None:
                self._cleanup_terminal_copy(session_id, job_root)
            self._release_local_run(job_id)

    def cancel(self, session_id: str, job_id: str) -> dict[str, object]:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
            active_benchmark = (
                connection.execute(
                    """SELECT grant.target_session_id
                       FROM benchmark_validation_grants grant
                       WHERE grant.job_id=? AND grant.status='consumed'
                         AND grant.root_pid=?
                         AND NOT EXISTS (
                             SELECT 1 FROM validation_copy_runs run
                             WHERE run.run_id=grant.validation_run_id
                         )""",
                    (job_id, row["run_pid"]),
                ).fetchone()
                if row is not None and row["status"] == "running"
                else None
            )
        if row is None:
            raise CoordinatorError("validation_copy_not_found", f"Unknown validation-copy job: {job_id}")
        authorized_session_id = (
            str(active_benchmark["target_session_id"])
            if active_benchmark is not None
            else str(row["session_id"])
        )
        if authorized_session_id != session_id:
            raise CoordinatorError(
                "validation_copy_foreign_session", "Validation copy belongs to another Session"
            )
        if row["status"] == "running":
            with self._running_lock:
                process = self._running_processes.get(job_id)
            if process is None or process.poll() is not None:
                raise CoordinatorError(
                    "validation_copy_cancel_race", "Validation process already changed state"
                )
            process.terminate()
            return {"jobId": job_id, "status": "cancelling"}
        if row["status"] == "planned" and row["materialization_started_at"] is not None:
            raise CoordinatorError(
                "validation_copy_materialization_busy",
                "Validation copy is still materializing",
            )
        if row["status"] in {"planned", "materialized", "failed"}:
            removed = self.cleanup(session_id, row["job_root"])
            return {"jobId": job_id, "status": "removed", "jobRoot": str(removed)}
        raise CoordinatorError(
            "validation_copy_cancel_invalid", f"Validation job cannot be cancelled from {row['status']}"
        )

    def cleanup(self, session_id: str, job_root: str | Path) -> Path:
        candidate = Path(job_root).resolve()
        self._validate_cleanup_root(candidate)
        with self._cleanup_lock:
            with self.database.transaction() as connection:
                row = connection.execute(
                    "SELECT job_id, session_id, status FROM validation_copies WHERE job_root = ?",
                    (str(candidate),),
                ).fetchone()
                if row is None:
                    raise CoordinatorError(
                        "validation_copy_not_found", f"Unknown validation-copy job: {candidate}"
                    )
                if row["session_id"] != session_id:
                    raise CoordinatorError(
                        "validation_copy_foreign_session",
                        "Validation copy belongs to another Session",
                    )
                self._require_cleanup_unreferenced(connection, row)
                cursor = connection.execute(
                    """
                    UPDATE validation_copies SET status = 'cleanup_pending'
                    WHERE job_root = ? AND status IN ('planned', 'materialized', 'failed')
                      AND (status <> 'planned' OR materialization_started_at IS NULL)
                    """,
                    (str(candidate),),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_cleanup_busy",
                        "Validation copy is running or already removed",
                    )
            return self._remove_pending_cleanup(candidate)

    def cleanup_terminal_ticket_copy(self, ticket_id: str, job_id: str) -> bool:
        """Remove a terminal ticket's linked source copy when all Cargo use is proven done."""
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT copy.session_id, copy.job_root
                   FROM validation_copies copy
                   JOIN validation_tickets ticket ON ticket.ticket_id=?
                   WHERE copy.job_id=?
                     AND copy.status IN ('planned', 'materialized', 'failed')
                     AND ticket.status IN ('passed', 'failed', 'snapshot_stale')
                     AND ticket.session_id=copy.session_id
                     AND EXISTS(
                         SELECT 1 FROM validation_ticket_events event
                         WHERE event.ticket_id=ticket.ticket_id
                           AND event.event_type='validation.ticket_copy_linked'
                           AND json_extract(event.payload_json, '$.jobId')=copy.job_id
                     )""",
                (ticket_id, job_id),
            ).fetchone()
        if row is None:
            return False
        try:
            self.cleanup(str(row["session_id"]), str(row["job_root"]))
        except Exception:
            return False
        return True

    @staticmethod
    def _require_cleanup_unreferenced(connection, row) -> None:
        def reject(kind: str, reference_id: str) -> None:
            raise CoordinatorError(
                "validation_copy_referenced",
                "Validation copy is still referenced by durable Cargo evidence",
                details={
                    "jobId": row["job_id"],
                    "referenceKind": kind,
                    "referenceId": reference_id,
                },
            )

        active_ticket = connection.execute(
            """SELECT ticket.ticket_id
               FROM validation_tickets ticket
               WHERE ticket.status NOT IN ('passed', 'failed', 'snapshot_stale')
                 AND (
                     EXISTS(
                         SELECT 1 FROM validation_copy_runs run
                         WHERE run.job_id=? AND run.run_id=ticket.ticket_id
                     )
                     OR EXISTS(
                         SELECT 1 FROM validation_ticket_events event
                         WHERE event.ticket_id=ticket.ticket_id
                           AND event.event_type='validation.ticket_copy_linked'
                           AND json_extract(event.payload_json, '$.jobId')=?
                     )
                 )
               LIMIT 1""",
            (row["job_id"], row["job_id"]),
        ).fetchone()
        if active_ticket is not None:
            reject("ticket", str(active_ticket["ticket_id"]))

        cargo_runs = connection.execute(
            """SELECT run.run_id
               FROM cargo_job_runs run
               JOIN cargo_jobs job ON job.job_id=run.job_id
               WHERE job.source_copy_job_id=?
                 AND run.status NOT IN ('completed', 'launch_failed')
               LIMIT 1""",
            (row["job_id"],),
        ).fetchone()
        if cargo_runs is not None:
            reject("cargo_run", str(cargo_runs["run_id"]))

        reservation = connection.execute(
            """SELECT reservation_id
               FROM cargo_lane_reservations
               WHERE source_copy_job_id=? AND status NOT IN ('released', 'expired')
               LIMIT 1""",
            (row["job_id"],),
        ).fetchone()
        if reservation is not None:
            reject("reservation", str(reservation["reservation_id"]))

        cargo_jobs = connection.execute(
            """SELECT job_id, status, process_tree_live_pids_json
               FROM cargo_jobs WHERE source_copy_job_id=?""",
            (row["job_id"],),
        ).fetchall()
        for cargo_job in cargo_jobs:
            try:
                live_pids = json.loads(
                    str(cargo_job["process_tree_live_pids_json"])
                )
            except (TypeError, ValueError, json.JSONDecodeError):
                live_pids = None
            if cargo_job["status"] != "released" or live_pids != []:
                reject("job", str(cargo_job["job_id"]))

    def recover_interrupted_jobs(
        self, *, process_alive=process_is_alive, startup: bool = True
    ) -> tuple[int, int]:
        recovered_running = 0
        recovered_cleanup = 0
        with self._running_lock:
            locally_active = frozenset(self._active_run_jobs)
            with self.database.transaction() as connection:
                rows = connection.execute(
                    """SELECT copy.job_id, copy.run_pid,
                              EXISTS(
                                  SELECT 1 FROM validation_copy_runs run
                                  WHERE run.job_id = copy.job_id
                              ) AS has_terminal_evidence
                       FROM validation_copies copy
                       WHERE copy.status = 'running'"""
                ).fetchall()
                for row in rows:
                    if row["job_id"] in locally_active:
                        continue
                    pid = int(row["run_pid"] or 0)
                    if pid <= 0 or not process_alive(pid):
                        status = (
                            "failed"
                            if row["has_terminal_evidence"]
                            else "materialized"
                        )
                        cursor = connection.execute(
                            """
                            UPDATE validation_copies
                            SET status = ?, run_pid = NULL
                            WHERE job_id = ? AND status = 'running'
                            """,
                            (status, row["job_id"]),
                        )
                        recovered_running += cursor.rowcount
        with self.database.connect() as connection:
            cleanup_rows = connection.execute(
                """
                SELECT session_id, job_root FROM validation_copies
                WHERE status = 'cleanup_pending'
                """
            ).fetchall()
        for row in cleanup_rows:
            try:
                candidate = Path(row["job_root"]).resolve()
                self._validate_cleanup_root(candidate)
                with self._cleanup_lock:
                    self._remove_pending_cleanup(candidate)
            except Exception:
                continue
            recovered_cleanup += 1
        recovered_cleanup += self._recover_missing_copy_roots()
        recovered_cleanup += self._recover_terminal_ticket_copies()
        if startup:
            self._recover_interrupted_cargo_materializations()
            with self.database.connect() as connection:
                planned_rows = connection.execute(
                    """SELECT job_root FROM validation_copies
                       WHERE status = 'planned' AND materialization_kind IS NULL"""
                ).fetchall()
            for row in planned_rows:
                try:
                    candidate = Path(row["job_root"]).resolve()
                    self._validate_cleanup_root(candidate)
                    with self._cleanup_lock:
                        if candidate.exists():
                            shutil.rmtree(candidate)
                        with self.database.transaction() as connection:
                            connection.execute(
                                """UPDATE validation_copies
                                   SET status = 'removed', removed_at = ?
                                   WHERE job_root = ? AND status = 'planned'""",
                                (utc_text(), str(candidate)),
                            )
                except Exception:
                    continue
                recovered_cleanup += 1
        return recovered_running, recovered_cleanup

    def _recover_terminal_ticket_copies(self, *, batch_size: int = 4) -> int:
        if batch_size < 1:
            raise ValueError("batch_size must be positive")
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT copy.job_id, MIN(ticket.ticket_id) AS ticket_id,
                          copy.created_at
                   FROM validation_copies copy
                   JOIN validation_ticket_events event
                     ON event.event_type='validation.ticket_copy_linked'
                    AND json_extract(event.payload_json, '$.jobId')=copy.job_id
                   JOIN validation_tickets ticket ON ticket.ticket_id=event.ticket_id
                    WHERE copy.status IN ('planned', 'materialized', 'failed')
                      AND ticket.status IN ('passed', 'failed', 'snapshot_stale')
                      AND ticket.session_id=copy.session_id
                      AND NOT EXISTS(
                          SELECT 1 FROM validation_tickets active_ticket
                          WHERE active_ticket.status NOT IN (
                              'passed', 'failed', 'snapshot_stale'
                          )
                            AND (
                                EXISTS(
                                    SELECT 1 FROM validation_copy_runs run
                                    WHERE run.job_id=copy.job_id
                                      AND run.run_id=active_ticket.ticket_id
                                )
                                OR EXISTS(
                                    SELECT 1 FROM validation_ticket_events active_event
                                    WHERE active_event.ticket_id=active_ticket.ticket_id
                                      AND active_event.event_type='validation.ticket_copy_linked'
                                      AND json_extract(
                                          active_event.payload_json, '$.jobId'
                                      )=copy.job_id
                                )
                            )
                      )
                      AND NOT EXISTS(
                          SELECT 1 FROM cargo_job_runs run
                          JOIN cargo_jobs job ON job.job_id=run.job_id
                          WHERE job.source_copy_job_id=copy.job_id
                            AND run.status NOT IN ('completed', 'launch_failed')
                      )
                      AND NOT EXISTS(
                          SELECT 1 FROM cargo_lane_reservations reservation
                          WHERE reservation.source_copy_job_id=copy.job_id
                            AND reservation.status NOT IN ('released', 'expired')
                      )
                      AND NOT EXISTS(
                          SELECT 1 FROM cargo_jobs job
                          WHERE job.source_copy_job_id=copy.job_id
                            AND (
                                job.status <> 'released'
                                OR CASE
                                    WHEN json_valid(job.process_tree_live_pids_json)
                                    THEN (
                                        json_type(job.process_tree_live_pids_json) <> 'array'
                                        OR json_array_length(
                                            job.process_tree_live_pids_json
                                        ) <> 0
                                    )
                                    ELSE 1
                                END
                            )
                      )
                    GROUP BY copy.job_id, copy.created_at
                    ORDER BY copy.created_at, copy.job_id
                    LIMIT ?""",
                (batch_size,),
            ).fetchall()
        recovered = 0
        for row in rows:
            if self.cleanup_terminal_ticket_copy(
                str(row["ticket_id"]), str(row["job_id"])
            ):
                recovered += 1
        return recovered

    def _recover_missing_copy_roots(self) -> int:
        return self._terminal.recover_missing_roots(
            validate_cleanup_root=self._validate_cleanup_root,
            running_lock=self._running_lock,
            active_run_jobs=lambda: frozenset(self._active_run_jobs),
        )

    def _recover_interrupted_cargo_materializations(self) -> None:
        """Atomically take over dead owners before removing partial copy state."""
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT job_id, job_root, materialization_phase,
                       materialization_worker_id, materialization_attempt
                FROM validation_copies
                WHERE status='planned' AND materialization_kind='cargo'
                  AND materialization_phase IN (
                      'accepted', 'closure_planning', 'materializing', 'recovery_cleanup'
                  )
                """
            ).fetchall()
        for row in rows:
            job_id = str(row["job_id"])
            if self._materialization_is_local(job_id):
                continue
            previous_phase = str(row["materialization_phase"])
            previous_worker = row["materialization_worker_id"]
            attempt = int(row["materialization_attempt"] or 0)
            if previous_worker and self._materialization_owner_is_live(
                str(previous_worker)
            ):
                continue
            candidate = Path(str(row["job_root"])).resolve()
            claimed = False
            try:
                self._validate_cleanup_root(candidate)
                claim_gate = (
                    self._mutation_gate()
                    if self._mutation_gate is not None
                    else nullcontext()
                )
                with claim_gate, self.database.transaction() as connection:
                    cursor = connection.execute(
                        """
                        UPDATE validation_copies
                        SET materialization_phase='recovery_cleanup',
                            materialization_started_at=?, materialization_worker_id=?
                        WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
                          AND materialization_phase=?
                          AND materialization_worker_id IS ?
                          AND materialization_attempt=?
                        """,
                        (
                            utc_text(),
                            self._materialization_worker_id,
                            job_id,
                            previous_phase,
                            previous_worker,
                            attempt,
                        ),
                    )
                if cursor.rowcount != 1:
                    continue
                claimed = True
                with self._cleanup_lock:
                    if candidate.exists():
                        shutil.rmtree(candidate)
                reset_gate = (
                    self._mutation_gate()
                    if self._mutation_gate is not None
                    else nullcontext()
                )
                with reset_gate, self.database.transaction() as connection:
                    cursor = connection.execute(
                        """
                        UPDATE validation_copies
                        SET manifest_json='[]', external_sources_json='[]',
                            materialization_phase='accepted', materialization_started_at=NULL,
                            materialization_worker_id=NULL, input_manifest_hash=NULL,
                            error_code=NULL, error_stage=NULL, error_path=NULL,
                            error_details_json='{}'
                        WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
                          AND materialization_phase='recovery_cleanup'
                          AND materialization_worker_id=?
                          AND materialization_attempt=?
                        """,
                        (job_id, self._materialization_worker_id, attempt),
                    )
                if cursor.rowcount != 1:
                    continue
                self._spawn_cargo_materialization_worker(job_id)
            except Exception:
                if claimed:
                    try:
                        gate = (
                            self._mutation_gate()
                            if self._mutation_gate is not None
                            else nullcontext()
                        )
                        with gate, self.database.transaction() as connection:
                            connection.execute(
                                """
                                UPDATE validation_copies
                                SET materialization_phase=?, materialization_worker_id=?,
                                    materialization_started_at=NULL
                                WHERE job_id=? AND status='planned'
                                  AND materialization_phase='recovery_cleanup'
                                  AND materialization_worker_id=?
                                  AND materialization_attempt=?
                                """,
                                (
                                    previous_phase,
                                    previous_worker,
                                    job_id,
                                    self._materialization_worker_id,
                                    attempt,
                                ),
                            )
                    except Exception:
                        pass
                continue

    @staticmethod
    def _materialization_owner_is_live(worker_id: str) -> bool:
        parts = worker_id.split(":", 3)
        if len(parts) != 4 or parts[0] != "v2":
            return False
        try:
            pid = int(parts[1])
        except ValueError:
            return False
        if not process_is_alive(pid):
            return False
        try:
            return process_creation_time(pid) == parts[2]
        except OSError:
            # Failure to query a live process is not proof that its ownership
            # lease is abandoned.
            return True

    def _cleanup_terminal_copy(self, session_id: str, job_root: Path) -> None:
        """Preserve completed validation evidence while deferring failed deletion."""
        try:
            self.cleanup(session_id, job_root)
        except Exception:
            # ``cleanup_pending`` is intentionally durable and retried by the
            # coordinator maintenance loop; validation completion stays visible.
            return

    def _remove_pending_cleanup(self, candidate: Path) -> Path:
        if candidate.exists():
            shutil.rmtree(candidate)
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE validation_copies SET status = 'removed', removed_at = ?
                WHERE job_root = ? AND status = 'cleanup_pending'
                """,
                (utc_text(), str(candidate)),
            )
        return candidate

    def _validate_cleanup_root(self, candidate: Path) -> None:
        if any(
            candidate.parent == self._managed_verify_root(root) and candidate.name
            for root in self.target_roots
        ):
            return
        raise CoordinatorError(
            "validation_copy_path_not_managed",
            f"Validation-copy cleanup path is not a direct verify job: {candidate}",
        )

    def _validation_copy_row(self, job_id: str):
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT job_root FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("validation_copy_not_found", f"Unknown validation-copy job: {job_id}")
        return row

    def _session_attributions(self, session_id: str) -> dict[str, str | None]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT path_key, content_hash FROM attributions WHERE session_id = ?", (session_id,)
            ).fetchall()
        return {row["path_key"]: row["content_hash"] for row in rows}

    def _require_untracked_overlay_attribution(
        self,
        record: WorkspaceCopyRecord,
        *,
        sealed_overlay_paths: tuple[str, ...] = (),
    ) -> None:
        attribution = self._session_attributions(record.session_id)
        attribution.update({path.casefold(): None for path in sealed_overlay_paths})
        baseline_paths = self._baseline_tracked_paths(record.job_id)
        missing_from_baseline = [
            path
            for path in record.manifest
            if path.casefold() not in attribution and path not in baseline_paths
        ]
        if not missing_from_baseline:
            return
        current_paths = self._current_tracked_paths()
        baseline_drift = sorted(
            (path for path in missing_from_baseline if path in current_paths),
            key=str.casefold,
        )
        if baseline_drift:
            raise CoordinatorError(
                "validation_copy_baseline_drift",
                "Cargo closure contains tracked paths added after its pinned baseline; replay the request",
                details={"paths": baseline_drift},
            )
        unowned = sorted(missing_from_baseline, key=str.casefold)
        if unowned:
            raise CoordinatorError(
                "validation_copy_unowned_path",
                "Untracked validation inputs require current Session attribution before async materialization",
                details={"paths": unowned},
            )

    def _record_from_row(self, row) -> WorkspaceCopyRecord:
        status = str(row["status"])
        materialization_phase = row["materialization_phase"]
        if status == "planned" and (
            row["materialization_started_at"] is not None
            or materialization_phase
            in {"accepted", "closure_planning", "materializing", "recovery_cleanup"}
        ):
            status = "materializing"
        error_details = json.loads(str(row["error_details_json"] or "{}"))
        if not isinstance(error_details, dict):
            error_details = {}
        return WorkspaceCopyRecord(
            str(row["job_id"]),
            str(row["session_id"]),
            Path(str(row["job_root"])),
            Path(str(row["source_root"])),
            Path(str(row["target_root"])),
            tuple(json.loads(str(row["manifest_json"]))),
            status,
            tuple(json.loads(str(row["external_sources_json"] or "[]"))),
            row["input_manifest_hash"],
            row["error_code"],
            row["error_stage"],
            row["error_path"],
            materialization_phase,
            error_details=error_details,
            materialization_kind=row["materialization_kind"],
        )

    def _begin_materialization(self, job_id: str) -> None:
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            cursor = connection.execute(
                """
                UPDATE validation_copies
                SET materialization_started_at = ?
                WHERE job_id = ? AND status = 'planned'
                  AND materialization_started_at IS NULL
                """,
                (utc_text(), job_id),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_materialization_busy",
                    "Validation copy is already materializing or unavailable",
                )

    def _complete_materialization(
        self, job_id: str, input_manifest_hash: str, *, worker_id: str | None = None
    ) -> None:
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            cursor = connection.execute(
                """
                UPDATE validation_copies
                SET status = 'materialized', materialization_started_at = NULL,
                    input_manifest_hash=?, error_code=NULL, error_stage=NULL,
                    error_path=NULL, error_details_json='{}',
                    materialization_phase=CASE
                        WHEN materialization_kind='cargo' THEN 'materialized'
                        ELSE materialization_phase
                    END
                WHERE job_id = ? AND status = 'planned'
                  AND (materialization_kind IS NULL OR materialization_worker_id=?)
                """,
                (input_manifest_hash, job_id, worker_id),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_materialization_state_lost",
                    "Validation copy changed state while materializing",
                )

    def _fail_materialization(
        self,
        job_id: str,
        *,
        error: BaseException,
        stage: str,
        worker_id: str | None = None,
    ) -> None:
        error_code = (
            error.code if isinstance(error, CoordinatorError) else "validation_copy_materialization_failed"
        )
        if isinstance(error, CoordinatorError):
            details = error.details
        else:
            details = {"errorType": type(error).__name__}
            for name in ("errno", "winerror"):
                value = getattr(error, name, None)
                if isinstance(value, int):
                    details[name] = value
        error_path = details.get("path")
        if error_path is None:
            paths = details.get("paths")
            if isinstance(paths, (list, tuple)) and paths:
                error_path = paths[0]
        if error_path is None:
            error_path = details.get("resourcePath") or details.get("sourcePath")
        if error_path is None:
            error_path = details.get("manifestPath") or details.get("repoRoot")
        error_path_text = str(error_path)[:1024] if error_path is not None else None
        durable_details = self._materialization_error_details(details)
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE validation_copies
                SET status = 'failed', materialization_started_at = NULL,
                    error_code=?, error_stage=?, error_path=?, error_details_json=?,
                    materialization_phase=CASE
                        WHEN materialization_kind='cargo' THEN 'failed'
                        ELSE materialization_phase
                    END
                WHERE job_id = ? AND status = 'planned'
                  AND (materialization_kind IS NULL OR materialization_worker_id=?)
                """,
                (
                    error_code,
                    stage,
                    error_path_text,
                    json.dumps(durable_details, sort_keys=True),
                    job_id,
                    worker_id,
                ),
            )

    @staticmethod
    def _materialization_error_details(details: dict[str, object]) -> dict[str, object]:
        durable: dict[str, object] = {}
        for key in (
            "path",
            "sourcePath",
            "resourcePath",
            "manifestPath",
            "repoRoot",
            "operation",
            "errorType",
        ):
            value = details.get(key)
            if value is not None:
                durable[key] = str(value)
        for key in ("resourceRootCount", "errno", "winerror", "exitCode"):
            value = details.get(key)
            if isinstance(value, int) and not isinstance(value, bool):
                durable[key] = value
        paths = details.get("paths")
        if isinstance(paths, (list, tuple)):
            durable["paths"] = [str(path) for path in paths[:64]]
        return durable

    def _extract_baseline_manifest(
        self, record: WorkspaceCopyRecord, attribution: dict[str, str | None]
    ) -> dict[str, str]:
        """Extract the pinned baseline in one archive stream, not one Git process per file."""
        baseline_paths = {
            path for path in record.manifest if path.casefold() not in attribution
        }
        if not baseline_paths:
            return {}
        # Keep small targeted copies cheap without crossing Windows command-line
        # limits for the all-tracked-file manifest.
        head_commit = self._head_commit(record.job_id)
        targeted_command = trusted_git_command(
            self.repo_root,
            "archive",
            "--format=tar",
            head_commit,
            "--",
            *sorted(baseline_paths, key=str.casefold),
        )
        archive_command = (
            targeted_command
            if len(subprocess.list2cmdline(targeted_command))
            <= _ARCHIVE_COMMAND_CHAR_LIMIT
            else trusted_git_command(
                self.repo_root, "archive", "--format=tar", head_commit
            )
        )
        process = subprocess.Popen(
            archive_command,
            cwd=self.repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        extracted: set[str] = set()
        input_entries: dict[str, str] = {}
        stderr = b""
        try:
            if process.stdout is None:
                raise CoordinatorError(
                    "validation_copy_dependency_archive_failed",
                    "Pinned baseline archive did not provide a readable stream",
                )
            with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
                for member in archive:
                    path = member.name.replace("\\", "/")
                    if path not in baseline_paths or not (member.isfile() or member.issym()):
                        continue
                    destination = _archive_member_destination(
                        record.source_root,
                        path,
                        error_code="validation_copy_dependency_archive_escape",
                    )
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if member.issym():
                        # Match `git show <tree>:<path>` without materializing a
                        # filesystem link that could escape the validation root.
                        content = member.linkname.encode("utf-8")
                    else:
                        source = archive.extractfile(member)
                        if source is None:
                            raise CoordinatorError(
                                "validation_copy_dependency_archive_invalid",
                                "Pinned baseline archive contains an unreadable file",
                            )
                        with source:
                            content = source.read()
                    destination.write_bytes(content)
                    input_entries[
                        destination.relative_to(record.job_root).as_posix()
                    ] = hashlib.sha256(content).hexdigest()
                    extracted.add(path)
        except BaseException:
            if process.poll() is None:
                process.kill()
            try:
                process.communicate()
            except BaseException:
                pass
            raise
        else:
            _, stderr = process.communicate()
        if process.returncode != 0:
            raise CoordinatorError(
                "validation_copy_dependency_archive_failed",
                "Could not materialize the pinned validation baseline",
                details={"stderr": stderr.decode("utf-8", errors="replace")[-4096:]},
            )
        missing = sorted(baseline_paths - extracted, key=str.casefold)
        if missing:
            raise CoordinatorError(
                "validation_copy_unowned_path",
                f"Untracked validation path is not owned by Session {record.session_id}: {missing[0]}",
                details={"paths": missing},
            )
        return input_entries

    def _extract_baseline_dependencies(
        self, record: WorkspaceCopyRecord, dependency_roots: tuple[str, ...]
    ) -> dict[str, str]:
        result = subprocess.run(
            trusted_git_command(
                self.repo_root,
                "archive",
                "--format=tar",
                self._head_commit(record.job_id),
                "--",
                *dependency_roots,
            ),
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_dependency_archive_failed",
                "Could not materialize validation template dependencies from the pinned baseline",
            )
        input_entries: dict[str, str] = {}
        with tarfile.open(fileobj=io.BytesIO(result.stdout), mode="r:") as archive:
            seen_paths: dict[str, str] = {}
            for member in archive.getmembers():
                destination = _archive_member_destination(
                    record.source_root,
                    member.name,
                    error_code="validation_copy_dependency_archive_escape",
                    seen_paths=seen_paths,
                )
                if not member.isfile():
                    continue
                source = archive.extractfile(member)
                if source is None:
                    raise CoordinatorError(
                        "validation_copy_dependency_archive_invalid",
                        "Validation template dependency archive contains an unreadable file",
                    )
                destination.parent.mkdir(parents=True, exist_ok=True)
                content = source.read()
                destination.write_bytes(content)
                input_entries[
                    destination.relative_to(record.job_root).as_posix()
                ] = hashlib.sha256(content).hexdigest()
        return input_entries

    def _extract_external_sources(self, record: WorkspaceCopyRecord) -> dict[str, str]:
        input_entries: dict[str, str] = {}
        for payload in record.external_sources:
            source = ExternalGitSource.from_payload(payload).pinned()
            mount = (record.job_root / source.mount_path).resolve()
            if not mount.is_relative_to(record.job_root) or mount == record.job_root:
                raise CoordinatorError(
                    "validation_copy_external_mount_escape",
                    "External Git archive escaped the validation job root",
                    details={"path": source.mount_path},
                )
            if source.archive_hash is not None:
                if self._object_store is None:
                    raise CoordinatorError(
                        "validation_copy_source_store_unavailable",
                        "Sealed external archives require the coordinator object store",
                    )
                content = self._object_store.get(source.archive_hash)
                if len(content) != source.archive_byte_count:
                    raise CoordinatorError(
                        "validation_copy_external_archive_invalid",
                        "Sealed external archive byte count does not match its descriptor",
                        details={"path": source.mount_path},
                    )
                extracted = extract_external_archive(content, mount)
                input_entries.update(
                    {
                        (Path(source.mount_path) / relative).as_posix(): digest
                        for relative, digest in extracted.items()
                    }
                )
                continue
            archive_command = trusted_git_command(
                source.repo_root, "archive", "--format=tar", source.commit
            )
            pathspecs = external_archive_pathspecs(source)
            if pathspecs:
                archive_command.extend(("--", *pathspecs))
            result = subprocess.run(
                archive_command,
                cwd=source.repo_root,
                check=False,
                capture_output=True,
            )
            if result.returncode != 0:
                raise CoordinatorError(
                    "validation_copy_external_archive_failed",
                    "Could not materialize pinned external Git inputs",
                    details={
                        "path": source.mount_path,
                        "stderr": result.stderr.decode("utf-8", errors="replace")[-4096:],
                    },
                )
            with tarfile.open(fileobj=io.BytesIO(result.stdout), mode="r:") as archive:
                seen_paths: dict[str, str] = {}
                for member in archive.getmembers():
                    destination = _archive_member_destination(
                        mount,
                        member.name,
                        error_code="validation_copy_external_archive_escape",
                        seen_paths=seen_paths,
                    )
                    if not (member.isfile() or member.issym()):
                        continue
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if member.issym():
                        content = member.linkname.encode("utf-8")
                    else:
                        stream = archive.extractfile(member)
                        if stream is None:
                            raise CoordinatorError(
                                "validation_copy_external_archive_invalid",
                                "External Git archive contains an unreadable file",
                                details={"path": member.name},
                            )
                        with stream:
                            content = stream.read()
                    destination.write_bytes(content)
                    input_entries[
                        destination.relative_to(record.job_root).as_posix()
                    ] = hashlib.sha256(content).hexdigest()
        return input_entries

    @staticmethod
    def _input_manifest_hash(record: WorkspaceCopyRecord) -> str:
        return WorkspaceCopyService._input_manifest_hash_for_roots(
            record.job_root, record.target_root
        )

    @staticmethod
    def _input_manifest_hash_for_roots(job_root: Path, target_root: Path) -> str:
        entries: dict[str, str] = {}
        resolved_job_root = job_root.resolve()
        resolved_target_root = target_root.resolve()
        try:
            target_relative = resolved_target_root.relative_to(resolved_job_root)
        except ValueError as error:
            raise CoordinatorError(
                "validation_copy_verify_escape",
                "Validation target is outside its managed job root",
            ) from error
        for path in resolved_job_root.rglob("*"):
            relative = path.relative_to(resolved_job_root)
            if relative == target_relative or target_relative in relative.parents:
                continue
            if path.is_symlink():
                raise CoordinatorError(
                    "validation_copy_manifest_symlink_forbidden",
                    "Validation inputs cannot contain filesystem links",
                    details={"path": relative.as_posix()},
                )
            if not path.is_file():
                continue
            entries[relative.as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
        return WorkspaceCopyService._input_manifest_hash_from_entries(entries)

    @staticmethod
    def _input_manifest_hash_from_entries(entries: Mapping[str, str]) -> str:
        payload = [
            {"path": path, "sha256": entries[path]}
            for path in sorted(entries, key=str.casefold)
        ]
        return hashlib.sha256(
            json.dumps(
                payload,
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()

    def _overlay_attributed_sources(
        self,
        record: WorkspaceCopyRecord,
        overlay_paths: tuple[str, ...],
        attribution: dict[str, str | None],
    ) -> dict[str, str]:
        input_entries: dict[str, str] = {}
        for path in overlay_paths:
            source = self.repo_root / path
            destination = record.source_root / path
            expected_hash = attribution[path.casefold()]
            if expected_hash is None:
                if source.exists():
                    raise CoordinatorError(
                        "validation_copy_owned_source_reappeared",
                        f"Owned deletion changed after attribution: {path}",
                        details={"path": path},
                    )
                if destination.exists():
                    destination.unlink()
                continue
            if not source.is_file():
                raise CoordinatorError(
                    "validation_copy_owned_source_missing",
                    f"Owned validation source is missing: {path}",
                    details={"path": path},
                )
            content = source.read_bytes()
            actual_hash = hashlib.sha256(content).hexdigest()
            if actual_hash != expected_hash:
                raise CoordinatorError(
                    "validation_copy_attribution_stale",
                    f"Owned source changed after attribution: {path}",
                    details={"path": path},
                )
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)
            input_entries[
                destination.relative_to(record.job_root).as_posix()
            ] = actual_hash
        return input_entries

    def _sealed_overlay_manifest(
        self,
        overlay_paths: tuple[str, ...] | list[str],
        manifest: Mapping[str, object],
    ) -> dict[str, str | None]:
        declared: dict[str, str] = {}
        for raw_path in overlay_paths:
            if not isinstance(raw_path, str):
                raise CoordinatorError(
                    "validation_copy_sealed_manifest_invalid",
                    "Sealed overlay paths must contain strings",
                )
            path = self._normalize_sealed_path(raw_path)
            key = path.casefold()
            if key in declared and declared[key] != path:
                raise CoordinatorError(
                    "validation_copy_sealed_manifest_invalid",
                    "Sealed overlay paths must be unique",
                    details={"path": path},
                )
            declared[key] = path

        sealed: dict[str, tuple[str, str | None]] = {}
        for raw_path, raw_hash in manifest.items():
            if not isinstance(raw_path, str):
                raise CoordinatorError(
                    "validation_copy_sealed_manifest_invalid",
                    "Sealed overlay manifest paths must be strings",
                )
            path = self._normalize_sealed_path(raw_path)
            if raw_hash is None:
                object_hash = None
            elif (
                isinstance(raw_hash, str)
                and len(raw_hash) == 64
                and all(
                    character in "0123456789abcdef"
                    for character in raw_hash.casefold()
                )
            ):
                object_hash = raw_hash.casefold()
            else:
                raise CoordinatorError(
                    "validation_copy_sealed_manifest_invalid",
                    "Sealed overlay values must be SHA-256 or null tombstones",
                    details={"path": path},
                )
            key = path.casefold()
            if key in sealed:
                raise CoordinatorError(
                    "validation_copy_sealed_manifest_invalid",
                    "Sealed overlay manifest paths must be unique",
                    details={"path": path},
                )
            sealed[key] = (path, object_hash)

        if set(declared) != set(sealed):
            mismatched = sorted(
                {
                    *(declared[key] for key in set(declared) - set(sealed)),
                    *(sealed[key][0] for key in set(sealed) - set(declared)),
                },
                key=str.casefold,
            )
            raise CoordinatorError(
                "validation_copy_sealed_manifest_mismatch",
                "Sealed overlay paths must match the immutable source manifest",
                details={"paths": mismatched},
            )
        return {
            path: object_hash
            for path, object_hash in sorted(
                sealed.values(), key=lambda item: item[0].casefold()
            )
        }

    @staticmethod
    def _normalize_sealed_path(value: str) -> str:
        path = normalize_portable_relative_path(
            value,
            code="validation_copy_sealed_manifest_invalid",
            message="Sealed overlay path is unsafe on managed Windows copies",
        )
        folded = path.casefold()
        if folded == ".git" or folded.startswith(".git/"):
            raise CoordinatorError(
                "validation_copy_git_forbidden", ".git cannot enter validation copies"
            )
        if folded == "target" or folded.startswith("target/"):
            raise CoordinatorError(
                "validation_copy_target_forbidden",
                "Build output cannot enter validation copies",
            )
        if folded == ".codex/state" or folded.startswith(".codex/state/"):
            raise CoordinatorError(
                "validation_copy_state_forbidden",
                "Coordinator state cannot enter validation copies",
            )
        return path

    def _overlay_sealed_sources(
        self,
        record: WorkspaceCopyRecord,
        manifest: Mapping[str, str | None],
    ) -> dict[str, str]:
        if self._object_store is None:
            raise CoordinatorError(
                "validation_copy_source_store_unavailable",
                "Sealed validation sources require the coordinator object store",
            )
        sealed = self._sealed_overlay_manifest(tuple(manifest), manifest)
        input_entries: dict[str, str] = {}
        for path, object_hash in sealed.items():
            destination = record.source_root.joinpath(*PurePosixPath(path).parts)
            if object_hash is None:
                if os.path.lexists(destination):
                    if destination.is_dir() and not destination.is_symlink():
                        raise CoordinatorError(
                            "validation_copy_sealed_tombstone_invalid",
                            "Sealed source tombstone refers to a directory",
                            details={"path": path},
                        )
                    destination.unlink()
                continue
            content = self._object_store.get(object_hash)
            destination.parent.mkdir(parents=True, exist_ok=True)
            if destination.is_symlink():
                raise CoordinatorError(
                    "validation_copy_manifest_symlink_forbidden",
                    "Validation inputs cannot contain filesystem links",
                    details={"path": path},
                )
            destination.write_bytes(content)
            input_entries[
                destination.relative_to(record.job_root).as_posix()
            ] = object_hash
        return input_entries

    def _head_commit(self, job_id: str) -> str:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT head_commit FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("validation_copy_not_found", f"Unknown validation-copy job: {job_id}")
        return str(row["head_commit"])

    def _head_content(self, job_id: str, path: str) -> bytes | None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT head_commit FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("validation_copy_not_found", f"Unknown validation-copy job: {job_id}")
        result = subprocess.run(
                trusted_git_command(self.repo_root, "show", f"{row['head_commit']}:{path}"),
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        return result.stdout if result.returncode == 0 else None

    def _baseline_tracked_paths(self, job_id: str) -> set[str]:
        result = subprocess.run(
            trusted_git_command(self.repo_root, "ls-tree", "-r", "--name-only", "-z", self._head_commit(job_id)),
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_baseline_tree_failed",
                "Could not inspect the pinned validation baseline tree",
                details={"stderr": result.stderr.decode("utf-8", errors="replace")[-4096:]},
            )
        return {
            path.decode("utf-8", errors="surrogateescape")
            for path in result.stdout.split(b"\0")
            if path
        }

    def _baseline_paths(
        self, baseline_commit: str, roots: tuple[str, ...]
    ) -> tuple[str, ...]:
        command = [
            "git",
            "ls-tree",
            "-r",
            "--name-only",
            "-z",
            baseline_commit,
        ]
        if roots:
            command.extend(("--", *roots))
        result = subprocess.run(
            command,
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_baseline_tree_failed",
                "Could not inspect the requested validation baseline tree",
                details={
                    "stderr": result.stderr.decode("utf-8", errors="replace")[-4096:]
                },
            )
        return tuple(
            sorted(
                (
                    path.decode("utf-8", errors="surrogateescape")
                    for path in result.stdout.split(b"\0")
                    if path
                ),
                key=str.casefold,
            )
        )

    def _requested_baseline_commit(self, baseline_commit: str | None) -> str:
        if baseline_commit is None:
            return self._git_text("rev-parse", "HEAD")
        normalized = baseline_commit.strip().lower()
        if len(normalized) not in {40, 64} or any(
            character not in "0123456789abcdef" for character in normalized
        ):
            raise CoordinatorError(
                "validation_copy_baseline_invalid",
                "Validation baseline must be a full Git object ID",
            )
        return normalized

    def _current_tracked_paths(self) -> set[str]:
        result = subprocess.run(
            trusted_git_command(self.repo_root, "ls-files", "-z"),
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_current_tree_failed",
                "Could not inspect the current tracked validation inputs",
                details={"stderr": result.stderr.decode("utf-8", errors="replace")[-4096:]},
            )
        return {
            path.decode("utf-8", errors="surrogateescape")
            for path in result.stdout.split(b"\0")
            if path
        }

    def _managed_verify_root(self, root: Path) -> Path:
        if root.exists() and root.is_symlink():
            raise CoordinatorError(
                "validation_copy_verify_escape", "Managed target root cannot be a link"
            )
        verify_root = (root / "verify").resolve()
        if verify_root.parent != root or verify_root.name.casefold() != "verify":
            raise CoordinatorError(
                "validation_copy_verify_escape",
                f"Validation verify root resolves outside the managed target root: {verify_root}",
            )
        return verify_root

    def _validate_job_root(self, job_root: Path) -> None:
        candidate = job_root.resolve()
        if not any(candidate.parent == self._managed_verify_root(root) for root in self.target_roots):
            raise CoordinatorError(
                "validation_copy_verify_escape", "Validation job is outside managed verify roots"
            )

    def _git_text(self, *arguments: str) -> str:
        result = subprocess.run(
            trusted_git_command(self.repo_root, *arguments),
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def _normalize(self, value: str) -> str:
        candidate = (self.repo_root / value).resolve()
        try:
            relative = candidate.relative_to(self.repo_root).as_posix()
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error
        if relative == ".git" or relative.startswith(".git/"):
            raise CoordinatorError("validation_copy_git_forbidden", ".git cannot enter validation copies")
        if relative == "target" or relative.startswith("target/"):
            raise CoordinatorError(
                "validation_copy_target_forbidden", "Build output cannot enter validation copies"
            )
        if relative == ".codex/state" or relative.startswith(".codex/state/"):
            raise CoordinatorError(
                "validation_copy_state_forbidden", "Coordinator state cannot enter validation copies"
            )
        return relative

    def _require_session(self, session_id: str) -> None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT 1 FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
