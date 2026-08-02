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
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, ContextManager, Mapping

from .baselines import hash_file
from .database import Database
from .models import CoordinatorError, utc_text
from .processes import process_is_alive
from .validation_copies import CargoInputClosurePlanner, ExternalGitSource
from .workspace_copy_terminal import (
    ValidationCopyTerminalLifecycle,
    ValidationRunEvidence,
)

_ARCHIVE_PATH_ARGUMENT_LIMIT = 512


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
            "materializationPhase": self.materialization_phase,
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
        self._materialization_lock = threading.Lock()
        self._active_materialization_jobs: set[str] = set()
        self._materialization_worker_id = uuid.uuid4().hex
        self._cleanup_lock = threading.Lock()
        self._mutation_gate = mutation_gate
        self._cargo_materialization_preflight = cargo_materialization_preflight
        self._completion_hook: Callable[[str], None] | None = None
        self._terminal = ValidationCopyTerminalLifecycle(database, mutation_gate)

    def set_completion_hook(self, hook: Callable[[str], None]) -> None:
        self._completion_hook = hook

    def set_cargo_materialization_preflight(
        self, preflight: Callable[[], None] | None
    ) -> None:
        """Configure a worker-only admission check for durable Cargo copies."""
        self._cargo_materialization_preflight = preflight

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
            self._running_processes.pop(job_id, None)
            self._active_run_jobs.discard(job_id)

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
        head_commit = self._git_text("rev-parse", "HEAD")
        with self.database.transaction() as connection:
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
    ) -> WorkspaceCopyRecord:
        record = self.plan(
            session_id,
            include_paths=include_paths,
            external_sources=external_sources,
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
        return self._record_from_row(row)

    def _materialize_async_worker(self, record: WorkspaceCopyRecord) -> None:
        try:
            self._materialize_record(record)
        except BaseException:
            # The durable status records the failure.  Detached HTTP callers must
            # not turn a filesystem failure into an unhandled worker exception.
            return

    def _materialize_record(
        self, record: WorkspaceCopyRecord, *, worker_id: str | None = None
    ) -> WorkspaceCopyRecord:
        stage = "prepare"
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            attribution = self._session_attributions(record.session_id)
            stage = "baseline_archive"
            self._extract_baseline_manifest(record, attribution)
            overlays = tuple(
                path for path in record.manifest if path.casefold() in attribution
            )
            stage = "owned_overlay"
            self._overlay_attributed_sources(record.source_root, overlays, attribution)
            stage = "external_archive"
            self._extract_external_sources(record)
            stage = "manifest_hash"
            input_manifest_hash = self._input_manifest_hash(record)
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
            )
        )
        self._begin_materialization(record.job_id)
        return self._materialize_validation_record(
            record, normalized_roots, normalized_overlays, attribution
        )

    def materialize_validation_async(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str],
    ) -> WorkspaceCopyRecord:
        """Durably reserve a dependency-scoped validation copy before archive I/O."""
        record, normalized_roots, normalized_overlays, attribution = (
            self._plan_validation_materialization(
                session_id,
                dependency_roots=dependency_roots,
                overlay_paths=overlay_paths,
            )
        )
        self._begin_materialization(record.job_id)
        worker = threading.Thread(
            target=self._materialize_validation_async_worker,
            args=(record, normalized_roots, normalized_overlays, attribution),
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

    def _plan_validation_materialization(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...] | list[str],
        overlay_paths: tuple[str, ...] | list[str],
    ) -> tuple[
        WorkspaceCopyRecord,
        tuple[str, ...],
        tuple[str, ...],
        dict[str, str | None],
    ]:
        normalized_roots = tuple(
            sorted({self._normalize(path) for path in dependency_roots}, key=str.casefold)
        )
        if not normalized_roots:
            raise CoordinatorError(
                "validation_copy_dependency_roots_empty",
                "Validation template must declare source dependencies",
            )
        dependency_paths = tuple(
            path
            for path in self._git_text("ls-files", "--", *normalized_roots).splitlines()
            if path
        )
        if not dependency_paths:
            raise CoordinatorError(
                "validation_copy_dependencies_missing",
                "Validation template dependencies are absent from the pinned baseline",
            )
        normalized_overlays = tuple(
            sorted({self._normalize(path) for path in overlay_paths}, key=str.casefold)
        )
        attribution = self._session_attributions(session_id)
        unowned = sorted(
            (path for path in normalized_overlays if path.casefold() not in attribution),
            key=str.casefold,
        )
        if unowned:
            raise CoordinatorError(
                "validation_copy_overlay_not_owned",
                "Validation overlay paths must be current Session-owned sources",
                details={"paths": unowned},
            )
        record = self.plan(
            session_id,
            include_paths=tuple(sorted(set(dependency_paths) | set(normalized_overlays))),
        )
        return record, normalized_roots, normalized_overlays, attribution

    def _materialize_validation_async_worker(
        self,
        record: WorkspaceCopyRecord,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        attribution: dict[str, str | None],
    ) -> None:
        try:
            self._materialize_validation_record(
                record, dependency_roots, overlay_paths, attribution
            )
        except BaseException:
            return

    def _materialize_validation_record(
        self,
        record: WorkspaceCopyRecord,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        attribution: dict[str, str | None],
    ) -> WorkspaceCopyRecord:
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            self._extract_baseline_dependencies(record, dependency_roots)
            self._overlay_attributed_sources(
                record.source_root, overlay_paths, attribution
            )
            input_manifest_hash = self._input_manifest_hash(record)
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
    ) -> WorkspaceCopyRecord:
        descriptors = tuple(
            ExternalGitSource.from_payload(payload) for payload in external_sources
        )
        closure = CargoInputClosurePlanner(
            self.repo_root, metadata_runner=metadata_runner
        ).plan(
            command,
            external_sources=descriptors,
            discover_external_sources=discover_external_sources,
        )
        paths = tuple(
            sorted(
                set(closure.repository_paths)
                | {self._normalize(path) for path in overlay_paths},
                key=str.casefold,
            )
        )
        return self.materialize(
            session_id,
            include_paths=paths,
            external_sources=tuple(
                source.to_payload()
                for source in closure.external_sources
            ),
        )

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
        # Persist raw request strings.  Path normalization resolves against the
        # workspace, so it belongs to the claimed worker along with ownership
        # validation rather than the bounded acknowledgement path.
        overlays = tuple(sorted({str(path) for path in overlay_paths}, key=str.casefold))
        try:
            payload = json.dumps(
                {
                    "command": command_tuple,
                    "overlayPaths": overlays,
                    "externalSources": list(external_sources),
                    "discoverExternalSources": bool(discover_external_sources),
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
                error_stage='request_decode', error_path=NULL
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
            if self._cargo_materialization_preflight is not None:
                self._cargo_materialization_preflight()
            stage = "root_preparation"
            row = self._prepare_cargo_materialization_root(job_id)
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
            closure = CargoInputClosurePlanner(
                self.repo_root, metadata_runner=metadata_runner
            ).plan(
                command,
                external_sources=descriptors,
                discover_external_sources=bool(request.get("discoverExternalSources")),
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
            overlays = tuple(self._normalize(path) for path in raw_overlays)
            stage = "overlay_ownership"
            self._require_cargo_overlay_attribution(str(row["session_id"]), overlays)
            paths = tuple(
                sorted(set(closure.repository_paths) | set(overlays), key=str.casefold)
            )
            stage = "materialization_prepare"
            record = self._persist_cargo_closure(
                row,
                paths,
                tuple(source.to_payload() for source in closure.external_sources),
            )
            self._require_untracked_overlay_attribution(record)
            self._materialize_record(record, worker_id=self._materialization_worker_id)
        except BaseException as error:
            self._fail_materialization(
                job_id,
                error=error,
                stage=stage,
                worker_id=self._materialization_worker_id,
            )
            raise

    def _prepare_cargo_materialization_root(self, job_id: str):
        """Select a verified target and pin HEAD after a worker owns the request."""
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
        head_commit = self._git_text("rev-parse", "HEAD")
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
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
        return row

    def _require_cargo_overlay_attribution(
        self, session_id: str, overlays: tuple[str, ...]
    ) -> None:
        attribution = self._session_attributions(session_id)
        unowned = sorted(
            (path for path in overlays if path.casefold() not in attribution), key=str.casefold
        )
        if unowned:
            raise CoordinatorError(
                "validation_copy_overlay_not_owned",
                "Cargo validation overlay paths require current Session attribution",
                details={"paths": unowned},
            )

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
            environment = os.environ.copy()
            environment["CARGO_TARGET_DIR"] = str(target_root)
            process = subprocess.Popen(
                command_tuple,
                cwd=source_root,
                env=environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            process_started = True
            with self._running_lock:
                self._running_processes[job_id] = process
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    "UPDATE validation_copies SET run_pid = ? WHERE job_id = ? AND status = 'running'",
                    (process.pid, job_id),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_terminal_state_changed",
                        "Validation copy changed state while registering its process",
                    )
            exit_code, stdout, stderr = self._terminal.collect(process)
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
    ) -> dict[str, object]:
        """Launch a managed validation and return after the process is registered."""
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
        run_id = run_id or uuid.uuid4().hex
        started_at = utc_text()
        process: subprocess.Popen[str] | None = None
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
            environment = os.environ.copy()
            environment["CARGO_TARGET_DIR"] = str(target_root)
            process = subprocess.Popen(
                command_tuple,
                cwd=source_root,
                env=environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            with self._running_lock:
                self._running_processes[job_id] = process
            with self.database.transaction() as connection:
                cursor = connection.execute(
                    "UPDATE validation_copies SET run_pid = ? WHERE job_id = ? AND status = 'running'",
                    (process.pid, job_id),
                )
                if cursor.rowcount != 1:
                    raise CoordinatorError(
                        "validation_copy_terminal_state_changed",
                        "Validation copy changed state while registering its process",
                    )
            threading.Thread(
                target=self._finish_started_run,
                args=(session_id, job_id, run_id, command_tuple, started_at, process),
                name=f"zircon-validation-{job_id[:12]}",
                daemon=True,
            ).start()
        except BaseException:
            if process is not None and process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            try:
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
            finally:
                self._release_local_run(job_id)
            raise
        return {
            "jobId": job_id,
            "runId": run_id,
            "pid": process.pid,
            "status": "running",
        }

    def _finish_started_run(
        self,
        session_id: str,
        job_id: str,
        run_id: str,
        command: tuple[str, ...],
        started_at: str,
        process: subprocess.Popen[str],
    ) -> None:
        job_root = Path(
            self._validation_copy_row(job_id)["job_root"]
        ).resolve()
        evidence_persisted = False
        completion_succeeded = False
        try:
            exit_code, stdout, stderr = self._terminal.collect(process)
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
            if process.poll() is None:
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
                process_started=True,
                evidence_persisted=evidence_persisted,
            )
        finally:
            if completion_succeeded:
                self._cleanup_terminal_copy(session_id, job_root)
            self._release_local_run(job_id)

    def cancel(self, session_id: str, job_id: str) -> dict[str, object]:
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

    @staticmethod
    def _require_cleanup_unreferenced(connection, row) -> None:
        referenced = connection.execute(
            """SELECT 'job' AS kind, job_id AS reference_id
               FROM cargo_jobs WHERE source_copy_job_id=?
               UNION ALL
               SELECT 'reservation' AS kind, reservation_id AS reference_id
               FROM cargo_lane_reservations WHERE source_copy_job_id=?
               LIMIT 1""",
            (row["job_id"], row["job_id"]),
        ).fetchone()
        if referenced is not None:
            raise CoordinatorError(
                "validation_copy_referenced",
                "Validation copy is still referenced by durable Cargo evidence",
                details={
                    "jobId": row["job_id"],
                    "referenceKind": referenced["kind"],
                    "referenceId": referenced["reference_id"],
                },
            )

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

    def _recover_interrupted_cargo_materializations(self) -> None:
        """Restart only durable Cargo requests that no local worker currently owns.

        The copy filesystem is not transactional, so a successor removes an
        interrupted partial job root before re-claiming the same durable job id.  The
        database compare-and-set in ``_claim_cargo_materialization`` ensures that
        exactly one worker owns any attempt; a second recovery pass observes that
        claim and does nothing.
        """
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT job_id, job_root, materialization_phase
                FROM validation_copies
                WHERE status='planned' AND materialization_kind='cargo'
                  AND materialization_phase IN ('accepted', 'closure_planning', 'materializing')
                """
            ).fetchall()
        for row in rows:
            job_id = str(row["job_id"])
            if self._materialization_is_local(job_id):
                continue
            candidate = Path(str(row["job_root"])).resolve()
            try:
                self._validate_cleanup_root(candidate)
                with self._cleanup_lock:
                    if candidate.exists():
                        shutil.rmtree(candidate)
                gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
                with gate, self.database.transaction() as connection:
                    cursor = connection.execute(
                        """
                        UPDATE validation_copies
                        SET manifest_json='[]', external_sources_json='[]',
                            materialization_phase='accepted', materialization_started_at=NULL,
                            materialization_worker_id=NULL, input_manifest_hash=NULL,
                            error_code=NULL, error_stage=NULL, error_path=NULL
                        WHERE job_id=? AND status='planned' AND materialization_kind='cargo'
                          AND materialization_phase IN ('accepted', 'closure_planning', 'materializing')
                        """,
                        (job_id,),
                    )
                if cursor.rowcount == 1:
                    self._spawn_cargo_materialization_worker(job_id)
            except Exception:
                # Keep the durable row visible for the next recovery pass.  A failed
                # cleanup cannot safely be treated as a fresh worker claim.
                continue

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

    def _require_untracked_overlay_attribution(self, record: WorkspaceCopyRecord) -> None:
        attribution = self._session_attributions(record.session_id)
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
            or materialization_phase in {"accepted", "closure_planning", "materializing"}
        ):
            status = "materializing"
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
                    error_path=NULL,
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
        details = error.details if isinstance(error, CoordinatorError) else {}
        error_path = details.get("path")
        if error_path is None:
            paths = details.get("paths")
            if isinstance(paths, list) and paths:
                error_path = paths[0]
        error_path_text = str(error_path)[:1024] if error_path is not None else None
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE validation_copies
                SET status = 'failed', materialization_started_at = NULL,
                    error_code=?, error_stage=?, error_path=?,
                    materialization_phase=CASE
                        WHEN materialization_kind='cargo' THEN 'failed'
                        ELSE materialization_phase
                    END
                WHERE job_id = ? AND status = 'planned'
                  AND (materialization_kind IS NULL OR materialization_worker_id=?)
                """,
                (error_code, stage, error_path_text, job_id, worker_id),
            )

    def _extract_baseline_manifest(
        self, record: WorkspaceCopyRecord, attribution: dict[str, str | None]
    ) -> None:
        """Extract the pinned baseline in one archive stream, not one Git process per file."""
        baseline_paths = {
            path for path in record.manifest if path.casefold() not in attribution
        }
        if not baseline_paths:
            return
        # Keep small targeted copies cheap without crossing Windows command-line
        # limits for the all-tracked-file manifest.
        archive_paths = (
            ("--", *sorted(baseline_paths, key=str.casefold))
            if len(baseline_paths) <= _ARCHIVE_PATH_ARGUMENT_LIMIT
            else ()
        )
        process = subprocess.Popen(
            [
                "git",
                "archive",
                "--format=tar",
                self._head_commit(record.job_id),
                *archive_paths,
            ],
            cwd=self.repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        extracted: set[str] = set()
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
                    destination = (record.source_root / path).resolve()
                    if not destination.is_relative_to(record.source_root):
                        raise CoordinatorError(
                            "validation_copy_dependency_archive_escape",
                            "Pinned baseline archive escaped the validation source root",
                        )
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if member.issym():
                        # Match `git show <tree>:<path>` without materializing a
                        # filesystem link that could escape the validation root.
                        destination.write_text(member.linkname, encoding="utf-8")
                    else:
                        source = archive.extractfile(member)
                        if source is None:
                            raise CoordinatorError(
                                "validation_copy_dependency_archive_invalid",
                                "Pinned baseline archive contains an unreadable file",
                            )
                        with source:
                            destination.write_bytes(source.read())
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

    def _extract_baseline_dependencies(
        self, record: WorkspaceCopyRecord, dependency_roots: tuple[str, ...]
    ) -> None:
        result = subprocess.run(
            [
                "git",
                "archive",
                "--format=tar",
                self._head_commit(record.job_id),
                "--",
                *dependency_roots,
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_dependency_archive_failed",
                "Could not materialize validation template dependencies from the pinned baseline",
            )
        with tarfile.open(fileobj=io.BytesIO(result.stdout), mode="r:") as archive:
            for member in archive.getmembers():
                destination = (record.source_root / member.name).resolve()
                if not destination.is_relative_to(record.source_root):
                    raise CoordinatorError(
                        "validation_copy_dependency_archive_escape",
                        "Validation template dependency archive escaped its source root",
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
                destination.write_bytes(source.read())

    def _extract_external_sources(self, record: WorkspaceCopyRecord) -> None:
        for payload in record.external_sources:
            source = ExternalGitSource.from_payload(payload).pinned()
            mount = (record.job_root / source.mount_path).resolve()
            if not mount.is_relative_to(record.job_root) or mount == record.job_root:
                raise CoordinatorError(
                    "validation_copy_external_mount_escape",
                    "External Git archive escaped the validation job root",
                    details={"path": source.mount_path},
                )
            result = subprocess.run(
                [
                    "git",
                    "archive",
                    "--format=tar",
                    source.commit,
                    "--",
                    *source.include_roots,
                ],
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
                for member in archive.getmembers():
                    destination = (mount / member.name).resolve()
                    if not destination.is_relative_to(mount):
                        raise CoordinatorError(
                            "validation_copy_external_archive_escape",
                            "External Git archive member escaped its managed mount",
                            details={"path": member.name},
                        )
                    if not (member.isfile() or member.issym()):
                        continue
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if member.issym():
                        destination.write_text(member.linkname, encoding="utf-8")
                        continue
                    stream = archive.extractfile(member)
                    if stream is None:
                        raise CoordinatorError(
                            "validation_copy_external_archive_invalid",
                            "External Git archive contains an unreadable file",
                            details={"path": member.name},
                        )
                    with stream:
                        destination.write_bytes(stream.read())

    @staticmethod
    def _input_manifest_hash(record: WorkspaceCopyRecord) -> str:
        entries: list[dict[str, str]] = []
        target_root = record.target_root.resolve()
        for path in record.job_root.rglob("*"):
            resolved = path.resolve()
            if not path.is_file() or resolved.is_relative_to(target_root):
                continue
            entries.append(
                {
                    "path": path.relative_to(record.job_root).as_posix(),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                }
            )
        return hashlib.sha256(
            json.dumps(
                sorted(entries, key=lambda item: item["path"].casefold()),
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()

    def _overlay_attributed_sources(
        self,
        source_root: Path,
        overlay_paths: tuple[str, ...],
        attribution: dict[str, str | None],
    ) -> None:
        for path in overlay_paths:
            source = self.repo_root / path
            destination = source_root / path
            expected_hash = attribution[path.casefold()]
            if expected_hash is None:
                if source.exists():
                    raise CoordinatorError(
                        "validation_copy_owned_source_reappeared",
                        f"Owned deletion changed after attribution: {path}",
                    )
                if destination.exists():
                    destination.unlink()
                continue
            if not source.is_file():
                raise CoordinatorError(
                    "validation_copy_owned_source_missing",
                    f"Owned validation source is missing: {path}",
                )
            if hash_file(source) != expected_hash:
                raise CoordinatorError(
                    "validation_copy_attribution_stale",
                    f"Owned source changed after attribution: {path}",
                )
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(source.read_bytes())

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
            ["git", "show", f"{row['head_commit']}:{path}"],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
        )
        return result.stdout if result.returncode == 0 else None

    def _baseline_tracked_paths(self, job_id: str) -> set[str]:
        result = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", "-z", self._head_commit(job_id)],
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

    def _current_tracked_paths(self) -> set[str]:
        result = subprocess.run(
            ["git", "ls-files", "-z"],
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
            ["git", *arguments],
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
