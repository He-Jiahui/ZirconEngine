from __future__ import annotations

import json
import os
import shutil
import subprocess
import uuid
from dataclasses import dataclass
from pathlib import Path

from .baselines import hash_file
from .database import Database
from .models import CoordinatorError, utc_text
from .processes import process_is_alive


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
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        roots = tuple(Path(root).resolve() for root in target_roots)
        if not roots:
            raise CoordinatorError(
                "target_root_unavailable", "No managed target root is available for validation copy"
            )
        for root in roots:
            if root.name.casefold() != "zircon-engine" or root.parent.name.casefold() != "targets":
                raise CoordinatorError(
                    "invalid_target_root", f"Invalid validation-copy target root: {root}"
                )
        self.target_roots = roots

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
        try:
            self._validate_job_root(record.job_root)
            record.source_root.mkdir(parents=True, exist_ok=False)
            record.target_root.mkdir(parents=True, exist_ok=False)
            attribution = self._session_attributions(session_id)
            for path in record.manifest:
                destination = record.source_root / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                source = self.repo_root / path
                attributed_hash = attribution.get(path.casefold())
                if attributed_hash is not None:
                    if not source.is_file():
                        raise CoordinatorError(
                            "validation_copy_owned_source_missing",
                            f"Owned validation source is missing: {path}",
                        )
                    if hash_file(source) != attributed_hash:
                        raise CoordinatorError(
                            "validation_copy_attribution_stale",
                            f"Owned source changed after attribution: {path}",
                        )
                    destination.write_bytes(source.read_bytes())
                    continue
                head_content = self._head_content(record.job_id, path)
                if head_content is None:
                    raise CoordinatorError(
                        "validation_copy_unowned_path",
                        f"Untracked validation path is not owned by Session {session_id}: {path}",
                    )
                destination.write_bytes(head_content)
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = 'materialized' WHERE job_id = ?",
                    (record.job_id,),
                )
        except BaseException:
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = 'failed' WHERE job_id = ?",
                    (record.job_id,),
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
            with self.database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET run_pid = ? WHERE job_id = ? AND status = 'running'",
                    (process.pid, job_id),
                )
            stdout_full, stderr_full = process.communicate()
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
        return ValidationRunEvidence(
            run_id, job_id, command_tuple, int(process.returncode), stdout, stderr
        )

    def cleanup(self, session_id: str, job_root: str | Path) -> Path:
        candidate = Path(job_root).resolve()
        valid = False
        for root in self.target_roots:
            verify_root = self._managed_verify_root(root)
            if candidate.parent == verify_root and candidate.name:
                valid = True
                break
        if not valid:
            raise CoordinatorError(
                "validation_copy_path_not_managed",
                f"Validation-copy cleanup path is not a direct verify job: {candidate}",
            )
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
                    "validation_copy_foreign_session", "Validation copy belongs to another Session"
                )
            cursor = connection.execute(
                """
                UPDATE validation_copies SET status = 'cleanup_pending'
                WHERE job_root = ? AND status IN ('planned', 'materialized', 'failed')
                """,
                (str(candidate),),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_cleanup_busy", "Validation copy is running or already removed"
                )
        try:
            if candidate.exists():
                shutil.rmtree(candidate)
        except BaseException:
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE validation_copies SET status = 'materialized'
                    WHERE job_root = ? AND status = 'cleanup_pending'
                    """,
                    (str(candidate),),
                )
            raise
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE validation_copies SET status = 'removed', removed_at = ?
                WHERE job_root = ?
                """,
                (utc_text(), str(candidate)),
            )
        return candidate

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
            if startup:
                cursor = connection.execute(
                    """
                    UPDATE validation_copies SET status = 'materialized'
                    WHERE status = 'cleanup_pending'
                    """
                )
                recovered_cleanup = cursor.rowcount
        return recovered_running, recovered_cleanup

    def _session_attributions(self, session_id: str) -> dict[str, str | None]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT path_key, content_hash FROM attributions WHERE session_id = ?", (session_id,)
            ).fetchall()
        return {row["path_key"]: row["content_hash"] for row in rows}

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
