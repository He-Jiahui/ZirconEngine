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
from typing import Callable, ContextManager

from .baselines import hash_file
from .database import Database
from .models import CoordinatorError, utc_text
from .processes import process_is_alive

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

    def to_dict(self) -> dict[str, object]:
        return {
            "job_id": self.job_id,
            "session_id": self.session_id,
            "job_root": str(self.job_root),
            "source_root": str(self.source_root),
            "target_root": str(self.target_root),
            "manifest": list(self.manifest),
            "status": self.status,
        }


@dataclass(frozen=True, slots=True)
class ValidationRunEvidence:
    run_id: str
    job_id: str
    command: tuple[str, ...]
    exit_code: int
    stdout: str
    stderr: str

    def to_dict(self) -> dict[str, object]:
        return {
            "run_id": self.run_id,
            "job_id": self.job_id,
            "command": list(self.command),
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
        }


class WorkspaceCopyService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        target_roots: tuple[str | Path, ...],
        mutation_gate: Callable[[], ContextManager[None]] | None = None,
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
        self._running_processes: dict[str, subprocess.Popen[str]] = {}
        self._cleanup_lock = threading.Lock()
        self._mutation_gate = mutation_gate
        self._completion_hook: Callable[[str], None] | None = None

    def set_completion_hook(self, hook: Callable[[str], None]) -> None:
        self._completion_hook = hook

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
        self, session_id: str, *, include_paths: tuple[str, ...] | list[str]
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
        head_commit = self._git_text("rev-parse", "HEAD")
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root, head_commit, manifest_json,
                    status, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 'planned', ?)
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
                ),
            )
        return WorkspaceCopyRecord(
            job_id, session_id, job_root, source_root, target_root, manifest, "planned"
        )

    def materialize(
        self, session_id: str, *, include_paths: tuple[str, ...] | list[str]
    ) -> WorkspaceCopyRecord:
        record = self.plan(session_id, include_paths=include_paths)
        self._begin_materialization(record.job_id)
        return self._materialize_record(record)

    def materialize_async(
        self, session_id: str, *, include_paths: tuple[str, ...] | list[str]
    ) -> WorkspaceCopyRecord:
        """Reserve a copy job immediately and materialize it off the request thread.

        A full workspace manifest can contain tens of thousands of tracked files.
        The coordinator must acknowledge that durable job before doing file I/O so
        Session heartbeats and Cargo lifecycle transitions keep progressing.
        """
        record = self.plan(session_id, include_paths=include_paths)
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

    def _materialize_record(self, record: WorkspaceCopyRecord) -> WorkspaceCopyRecord:
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            attribution = self._session_attributions(record.session_id)
            self._extract_baseline_manifest(record, attribution)
            overlays = tuple(
                path for path in record.manifest if path.casefold() in attribution
            )
            self._overlay_attributed_sources(record.source_root, overlays, attribution)
            self._complete_materialization(record.job_id)
        except BaseException:
            self._fail_materialization(record.job_id)
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
        self._begin_materialization(record.job_id)
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            self._extract_baseline_dependencies(record, normalized_roots)
            self._overlay_attributed_sources(
                record.source_root, normalized_overlays, attribution
            )
            self._complete_materialization(record.job_id)
        except BaseException:
            self._fail_materialization(record.job_id)
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
        )

    def run(
        self, session_id: str, job_id: str, *, command: tuple[str, ...] | list[str]
    ) -> ValidationRunEvidence:
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError(
                "validation_copy_command_empty", "Validation command cannot be empty"
            )
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT * FROM validation_copies WHERE job_id = ?", (job_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError("validation_copy_not_found", f"Unknown validation-copy job: {job_id}")
            if row["session_id"] != session_id:
                raise CoordinatorError(
                    "validation_copy_foreign_session", "Validation copy belongs to another Session"
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
        run_id = uuid.uuid4().hex
        started_at = utc_text()
        process: subprocess.Popen[str] | None = None
        job_root = Path(row["job_root"]).resolve()
        process_finished = False
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
            with self._running_lock:
                self._running_processes[job_id] = process
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET run_pid = ? WHERE job_id = ? AND status = 'running'",
                    (process.pid, job_id),
                )
            stdout_full, stderr_full = process.communicate()
            process_finished = True
            completed_at = utc_text()
            stdout = stdout_full[-65536:]
            stderr = stderr_full[-65536:]
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO validation_copy_runs(
                        run_id, job_id, session_id, command_json, exit_code,
                        stdout_text, stderr_text, started_at, completed_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        run_id,
                        job_id,
                        session_id,
                        json.dumps(command_tuple),
                        process.returncode,
                        stdout,
                        stderr,
                        started_at,
                        completed_at,
                    ),
                )
                connection.execute(
                    "UPDATE validation_copies SET status = 'materialized', run_pid = NULL WHERE job_id = ? AND status = 'running'",
                    (job_id,),
                )
            if self._completion_hook is not None:
                self._completion_hook(run_id)
        except BaseException:
            if process is not None and process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = 'materialized', run_pid = NULL WHERE job_id = ? AND status = 'running'",
                    (job_id,),
                )
            raise
        finally:
            if process_finished:
                self._cleanup_terminal_copy(session_id, job_root)
            with self._running_lock:
                self._running_processes.pop(job_id, None)
        return ValidationRunEvidence(
            run_id, job_id, command_tuple, int(process.returncode), stdout, stderr
        )

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
                connection.execute(
                    "UPDATE validation_copies SET run_pid = ? WHERE job_id = ? AND status = 'running'",
                    (process.pid, job_id),
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
            with self._running_lock:
                self._running_processes.pop(job_id, None)
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = 'materialized', run_pid = NULL WHERE job_id = ? AND status = 'running'",
                    (job_id,),
                )
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
        process_finished = False
        try:
            stdout_full, stderr_full = process.communicate()
            process_finished = True
            gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
            with gate, self.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO validation_copy_runs(
                           run_id, job_id, session_id, command_json, exit_code,
                           stdout_text, stderr_text, started_at, completed_at
                       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    (
                        run_id,
                        job_id,
                        session_id,
                        json.dumps(command),
                        int(process.returncode),
                        stdout_full[-65536:],
                        stderr_full[-65536:],
                        started_at,
                        utc_text(),
                    ),
                )
                connection.execute(
                    "UPDATE validation_copies SET status = 'materialized', run_pid = NULL WHERE job_id = ? AND status = 'running'",
                    (job_id,),
                )
            if self._completion_hook is not None:
                self._completion_hook(run_id)
        except BaseException:
            gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
            with gate, self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = 'failed', run_pid = NULL WHERE job_id = ? AND status = 'running'",
                    (job_id,),
                )
        finally:
            if process_finished:
                self._cleanup_terminal_copy(session_id, job_root)
            with self._running_lock:
                self._running_processes.pop(job_id, None)

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

    def recover_interrupted_jobs(
        self, *, process_alive=process_is_alive, startup: bool = True
    ) -> tuple[int, int]:
        recovered_running = 0
        recovered_cleanup = 0
        with self.database.transaction() as connection:
            rows = connection.execute(
                "SELECT job_id, run_pid FROM validation_copies WHERE status = 'running'"
            ).fetchall()
            for row in rows:
                pid = int(row["run_pid"] or 0)
                if pid <= 0 or not process_alive(pid):
                    connection.execute(
                        """
                        UPDATE validation_copies
                        SET status = 'materialized', run_pid = NULL
                        WHERE job_id = ? AND status = 'running'
                        """,
                        (row["job_id"],),
                    )
                    recovered_running += 1
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
            with self.database.connect() as connection:
                planned_rows = connection.execute(
                    "SELECT job_root FROM validation_copies WHERE status = 'planned'"
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

    def _record_from_row(self, row) -> WorkspaceCopyRecord:
        status = str(row["status"])
        if status == "planned" and row["materialization_started_at"] is not None:
            status = "materializing"
        return WorkspaceCopyRecord(
            str(row["job_id"]),
            str(row["session_id"]),
            Path(str(row["job_root"])),
            Path(str(row["source_root"])),
            Path(str(row["target_root"])),
            tuple(json.loads(str(row["manifest_json"]))),
            status,
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

    def _complete_materialization(self, job_id: str) -> None:
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            cursor = connection.execute(
                """
                UPDATE validation_copies
                SET status = 'materialized', materialization_started_at = NULL
                WHERE job_id = ? AND status = 'planned'
                """,
                (job_id,),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_materialization_state_lost",
                    "Validation copy changed state while materializing",
                )

    def _fail_materialization(self, job_id: str) -> None:
        gate = self._mutation_gate() if self._mutation_gate is not None else nullcontext()
        with gate, self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE validation_copies
                SET status = 'failed', materialization_started_at = NULL
                WHERE job_id = ? AND status = 'planned'
                """,
                (job_id,),
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
            raise
        finally:
            if process.stdout is not None:
                process.stdout.close()
            if process.stderr is not None:
                stderr = process.stderr.read()
                process.stderr.close()
            process.wait()
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
