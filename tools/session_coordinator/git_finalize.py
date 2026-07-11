from __future__ import annotations

import json
import os
import re
import subprocess
import uuid
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import IntegrityError
from typing import Iterator

from .baselines import BaselineHealth, BaselineService, hash_file
from .database import Database
from .models import CoordinatorError, SessionStatus, utc_text
from .plans import PlanRepository
from .sessions import SessionService

if False:  # pragma: no cover - import only for static typing without a runtime cycle
    from .failures import FailureGraphService


SEMANTIC_MESSAGE = re.compile(r"^[a-z]+(?:\([^)]+\))?!?: .+")
_WECOM_ENDPOINT_MARKER = "qyapi" + ".weixin.qq.com/cgi-bin/" + "webhook/send?" + "key="
FORBIDDEN_SECRET = re.compile(
    re.escape(_WECOM_ENDPOINT_MARKER) + r"|(?:WECOM|WECHAT).*WEBHOOK.*(?:URL|KEY)",
    re.IGNORECASE,
)


@dataclass(frozen=True, slots=True)
class FinalizePreview:
    request_id: str
    session_id: str
    message: str
    paths: tuple[str, ...]
    categories: dict[str, tuple[str, ...]]
    untracked_paths: tuple[str, ...]
    maintenance: bool

    def to_dict(self) -> dict[str, object]:
        return {
            "request_id": self.request_id,
            "session_id": self.session_id,
            "message": self.message,
            "paths": list(self.paths),
            "categories": {key: list(value) for key, value in self.categories.items()},
            "untracked_paths": list(self.untracked_paths),
            "maintenance": self.maintenance,
        }


@dataclass(frozen=True, slots=True)
class FinalizeResult:
    request_id: str
    commit_sha: str
    message: str
    categories: dict[str, tuple[str, ...]]
    untracked_paths: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "request_id": self.request_id,
            "commit_sha": self.commit_sha,
            "message": self.message,
            "categories": {key: list(value) for key, value in self.categories.items()},
            "untracked_paths": list(self.untracked_paths),
        }


class GitFinalizeService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        baselines: BaselineService,
        sessions: SessionService,
        plans: PlanRepository | None = None,
        failures: "FailureGraphService | None" = None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.baselines = baselines
        self.sessions = sessions
        self.plans = plans or PlanRepository(self.repo_root)
        self.failures = failures

    def preview(
        self,
        session_id: str,
        *,
        paths: list[str] | tuple[str, ...],
        message: str,
        validation_commands: tuple[tuple[str, ...], ...] = (),
        maintenance: bool = False,
    ) -> FinalizePreview:
        self._validate_message(message)
        normalized = tuple(sorted({self._normalize(path) for path in paths}, key=str.casefold))
        if not normalized:
            raise CoordinatorError("finalize_paths_empty", "Finalize requires at least one path")
        session = self.sessions.get(session_id)
        if not maintenance and session.status is not SessionStatus.COMPLETED:
            raise CoordinatorError(
                "finalize_session_not_completed",
                f"Session {session_id} must be completed before explicit finalize",
            )
        self._require_attribution(session_id, normalized, maintenance=maintenance)
        self._require_owned_scope(session_id, normalized, maintenance=maintenance)
        self._require_plan_outputs(session, normalized, maintenance=maintenance)
        self._require_failure_acceptance(session, maintenance=maintenance)
        self._require_git_mutex_available()
        self._require_index_scope(normalized)
        unattributed = self.baselines.scan()
        baseline = self.baselines.current()
        if baseline.health is BaselineHealth.DEGRADED:
            raise CoordinatorError(
                "finalize_baseline_degraded",
                "Workspace baseline is degraded",
                details={"paths": [change.path for change in unattributed]},
            )
        if baseline.head_commit != self._git("rev-parse", "HEAD"):
            raise CoordinatorError(
                "finalize_baseline_head_changed",
                "HEAD changed after the current workspace baseline was captured",
            )
        self._require_no_foreign_leases(session_id, normalized)
        self._require_no_pending_patches(session_id)
        categories = self._categorize(normalized)
        untracked = tuple(path for path in normalized if not self._is_tracked(path))
        request_id = uuid.uuid4().hex
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO finalize_requests(
                    request_id, session_id, message, paths_json, categories_json,
                    untracked_json, validation_json, maintenance, status, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'previewed', ?)
                """,
                (
                    request_id,
                    session_id,
                    message,
                    json.dumps(normalized),
                    json.dumps({key: list(value) for key, value in categories.items()}),
                    json.dumps(untracked),
                    json.dumps(validation_commands),
                    1 if maintenance else 0,
                    utc_text(),
                ),
            )
        return FinalizePreview(
            request_id,
            session_id,
            message,
            normalized,
            categories,
            untracked,
            maintenance,
        )

    def finalize(
        self,
        session_id: str,
        *,
        paths: list[str] | tuple[str, ...],
        message: str,
        validation_commands: tuple[tuple[str, ...], ...] = (),
        maintenance: bool = False,
    ) -> FinalizeResult:
        preview = self.preview(
            session_id,
            paths=paths,
            message=message,
            validation_commands=validation_commands,
            maintenance=maintenance,
        )
        if not maintenance:
            self.sessions.set_status(
                session_id, SessionStatus.FINALIZING, reason="explicit finalize --commit"
            )
        committed = False
        index_path: Path | None = None
        index_existed = False
        index_content = b""
        try:
            with self.git_mutex(session_id):
                index_path = self._index_path()
                index_existed = index_path.exists()
                index_content = index_path.read_bytes() if index_existed else b""
                self._persist_finalize_start(
                    preview.request_id,
                    start_head=self._git("rev-parse", "HEAD"),
                    index_existed=index_existed,
                    index_content=index_content,
                )
                session = self.sessions.get(session_id)
                self._require_finalize_guards_under_mutex(session, preview)
                self._require_attribution(session_id, preview.paths, maintenance=maintenance)
                self._require_owned_scope(session_id, preview.paths, maintenance=maintenance)
                self._require_index_scope(preview.paths)
                expected_blobs = self._expected_staged_blobs(
                    preview.paths, maintenance=maintenance
                )
                self._git("add", "-A", "--", *preview.paths)
                staged = tuple(self._git_lines("diff", "--cached", "--name-only"))
                if set(staged) != set(preview.paths):
                    raise CoordinatorError(
                        "finalize_staged_scope_mismatch",
                        "Staged paths do not exactly match the approved finalize set",
                        details={"approved": list(preview.paths), "staged": list(staged)},
                    )
                self._require_staged_attribution(
                    expected_blobs, maintenance=maintenance
                )
                self._require_no_staged_secrets()
                for command in validation_commands:
                    result = subprocess.run(command, cwd=self.repo_root, check=False)
                    if result.returncode != 0:
                        raise CoordinatorError(
                            "finalize_validation_failed",
                            f"Validation command failed with exit code {result.returncode}",
                            details={"command": list(command), "exit_code": result.returncode},
                        )
                self._require_index_scope(preview.paths)
                self._require_staged_attribution(
                    expected_blobs, maintenance=maintenance
                )
                self._require_no_staged_secrets()
                session = self.sessions.get(session_id)
                self._require_finalize_guards_under_mutex(session, preview)
                commit_sha = self._create_scoped_commit(
                    preview.message, expected_head=self.baselines.current().head_commit
                )
                committed = True
                with self.database.transaction() as connection:
                    connection.execute(
                        """
                        UPDATE finalize_requests
                        SET ref_updated_sha = ?, status = 'finalizing'
                        WHERE request_id = ?
                        """,
                        (commit_sha, preview.request_id),
                    )
                self.baselines.accept_commit(
                    preview.paths,
                    commit_sha=commit_sha,
                    reason=f"finalize commit {commit_sha}",
                )
                with self.database.transaction() as connection:
                    connection.execute(
                        """
                        UPDATE finalize_requests
                        SET status = 'committed', commit_sha = ?, completed_at = ?
                        WHERE request_id = ?
                        """,
                        (commit_sha, utc_text(), preview.request_id),
                    )
        except BaseException as error:
            if not committed and index_path is not None:
                self._restore_index(index_path, index_existed, index_content)
            if not committed:
                self._set_request_failed(preview.request_id, str(error))
            if not maintenance:
                self.sessions.set_status(
                    session_id,
                    SessionStatus.COMPLETED,
                    reason=(
                        "finalize baseline reconciliation pending"
                        if committed
                        else "finalize failed; worktree preserved"
                    ),
                )
            raise
        if not maintenance:
            self.sessions.set_status(
                session_id, SessionStatus.COMPLETED, reason=f"finalized {commit_sha}"
            )
        return FinalizeResult(
            preview.request_id,
            commit_sha,
            preview.message,
            preview.categories,
            preview.untracked_paths,
        )

    @contextmanager
    def git_mutex(self, owner_id: str) -> Iterator[None]:
        try:
            with self.database.transaction() as connection:
                connection.execute(
                    "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, ?)",
                    (owner_id, utc_text()),
                )
        except IntegrityError as error:
            raise CoordinatorError(
                "git_mutex_occupied", "Another finalize operation owns the Git index mutex"
            ) from error
        try:
            yield
        finally:
            with self.database.transaction() as connection:
                connection.execute(
                    "DELETE FROM git_mutex WHERE lock_name = 'index' AND owner_id = ?",
                    (owner_id,),
                )

    def recover_stale_mutex(self) -> int:
        """Recover an index transaction left by the previous single service process."""
        recoveries: list[dict[str, object]] = []
        with self.database.transaction() as connection:
            rows = connection.execute("SELECT owner_id FROM git_mutex").fetchall()
            finalizing = connection.execute(
                """
                SELECT request_id, session_id, message, paths_json, start_head,
                       index_existed, index_snapshot, ref_updated_sha
                FROM finalize_requests
                WHERE status = 'finalizing'
                   OR (status = 'committed' AND commit_sha IS NULL AND ref_updated_sha IS NOT NULL)
                ORDER BY created_at
                """
            ).fetchall()
            recoveries = [
                dict(row)
                for row in finalizing
            ]
            connection.execute("DELETE FROM git_mutex")
        current_head = self._git("rev-parse", "HEAD")
        for recovery in recoveries:
            request_id = str(recovery["request_id"])
            session_id = str(recovery["session_id"])
            start_head = recovery.get("start_head")
            index_existed = recovery.get("index_existed")
            index_snapshot = recovery.get("index_snapshot")
            paths = tuple(json.loads(str(recovery["paths_json"])))
            recovered_commit = self._match_recovered_commit(
                current_head,
                start_head=str(start_head) if start_head else None,
                message=str(recovery["message"]),
                paths=paths,
            )
            ref_updated_sha = recovery.get("ref_updated_sha")
            if ref_updated_sha == current_head:
                recovered_commit = True
            if recovered_commit:
                self.baselines.accept_commit(
                    paths,
                    commit_sha=current_head,
                    reason=f"recovered finalize commit {current_head}",
                )
                with self.database.transaction() as connection:
                    connection.execute(
                        """
                        UPDATE finalize_requests
                        SET status = 'committed', commit_sha = ?, error_text = NULL, completed_at = ?
                        WHERE request_id = ?
                        """,
                        (current_head, utc_text(), request_id),
                    )
                reason = f"recovered finalized commit {current_head}"
            else:
                if start_head == current_head and index_existed is not None and index_snapshot is not None:
                    self._restore_index(
                        self._index_path(), bool(index_existed), bytes(index_snapshot)
                    )
                with self.database.transaction() as connection:
                    connection.execute(
                        """
                        UPDATE finalize_requests
                        SET status = 'failed', error_text = ?, completed_at = ?
                        WHERE request_id = ?
                        """,
                        ("service restarted during finalize", utc_text(), request_id),
                    )
                reason = "finalize interrupted by service restart"
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE sessions
                    SET status = 'completed', status_reason = ?, updated_at = ?
                    WHERE session_id = ? AND status = 'finalizing'
                    """,
                    (reason, utc_text(), session_id),
                )
        return len(rows)

    def _require_finalize_guards_under_mutex(
        self, session, preview: FinalizePreview
    ) -> None:
        if self.baselines.current().health is BaselineHealth.DEGRADED:
            raise CoordinatorError("finalize_baseline_degraded", "Workspace baseline is degraded")
        if self.baselines.current().head_commit != self._git("rev-parse", "HEAD"):
            raise CoordinatorError(
                "finalize_baseline_head_changed",
                "HEAD changed after the finalize preview",
            )
        self._require_plan_outputs(session, preview.paths, maintenance=preview.maintenance)
        self._require_failure_acceptance(session, maintenance=preview.maintenance)
        self._require_no_foreign_leases(session.session_id, preview.paths)
        self._require_no_pending_patches(session.session_id)

    def _require_attribution(
        self, session_id: str, paths: tuple[str, ...], *, maintenance: bool
    ) -> None:
        if maintenance:
            return
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT path_key, session_id, content_hash FROM attributions"
            ).fetchall()
        attribution = {row["path_key"]: row for row in rows}
        for path in paths:
            row = attribution.get(path.casefold())
            if (
                row is None
                or row["session_id"] != session_id
                or row["content_hash"] != hash_file(self.repo_root / path)
            ):
                raise CoordinatorError(
                    "finalize_unattributed_path",
                    f"Path is not attributed to Session {session_id}: {path}",
                )

    def _require_owned_scope(
        self, session_id: str, approved: tuple[str, ...], *, maintenance: bool
    ) -> None:
        if maintenance:
            return
        changes = {change.path: change.current_hash for change in self.baselines.diff()}
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT display_path, content_hash FROM attributions WHERE session_id = ?",
                (session_id,),
            ).fetchall()
        owned_dirty = {
            row["display_path"]
            for row in rows
            if row["display_path"] in changes
            and row["content_hash"] == changes[row["display_path"]]
        }
        omitted = sorted(owned_dirty - set(approved), key=str.casefold)
        if omitted:
            raise CoordinatorError(
                "finalize_owned_path_omitted",
                "Finalize manifest omits current Session-owned changes",
                details={"paths": omitted},
            )
        unchanged = sorted(set(approved) - set(changes), key=str.casefold)
        if unchanged:
            raise CoordinatorError(
                "finalize_path_unchanged",
                "Finalize manifest contains paths without workspace changes",
                details={"paths": unchanged},
            )

    def _require_plan_outputs(self, session, paths: tuple[str, ...], *, maintenance: bool) -> None:
        plan_paths = [path for path in paths if path.casefold().startswith("docs/plans/")]
        if not plan_paths:
            return
        if not session.plan_path:
            raise CoordinatorError(
                "finalize_session_plan_missing",
                "A Session must register its numbered plan before finalizing plan output",
            )
        for path in plan_paths:
            decision = self.plans.authorize_write(
                session.plan_path, path, maintenance=maintenance
            )
            if not decision.allowed:
                raise CoordinatorError(
                    "finalize_invalid_plan_output",
                    decision.message,
                    details={"path": path, "plan_code": decision.code},
                )

    def _require_failure_acceptance(self, session, *, maintenance: bool) -> None:
        if maintenance or self.failures is None or not session.plan_path:
            return
        open_failures = self.failures.open_for_plan(session.plan_path)
        if open_failures:
            raise CoordinatorError(
                "finalize_open_failure",
                "Required Failure handoffs must be architecturally fixed and returned before finalize",
                details={"paths": [item.artifact_path for item in open_failures]},
            )

    def _require_no_foreign_leases(self, session_id: str, paths: tuple[str, ...]) -> None:
        keys = {path.casefold() for path in paths}
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT path_key, session_id, display_path FROM leases"
            ).fetchall()
        conflicts = [
            row["display_path"]
            for row in rows
            if row["path_key"] in keys and row["session_id"] != session_id
        ]
        if conflicts:
            raise CoordinatorError(
                "finalize_foreign_lease",
                "Finalize paths are leased by another Session",
                details={"paths": conflicts},
            )

    def _require_no_pending_patches(self, session_id: str) -> None:
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT patch_id, status FROM patches
                WHERE session_id = ? AND status IN ('queued', 'applying', 'needs_rebase')
                LIMIT 1
                """,
                (session_id,),
            ).fetchone()
        if row is not None:
            raise CoordinatorError(
                "finalize_pending_patch",
                f"Patch {row['patch_id']} is {row['status']}",
            )

    def _require_index_scope(self, approved: tuple[str, ...]) -> None:
        staged = set(self._git_lines("diff", "--cached", "--name-only"))
        foreign = sorted(staged - set(approved), key=str.casefold)
        if foreign:
            raise CoordinatorError(
                "finalize_foreign_index",
                "Git index contains paths outside the approved finalize set",
                details={"paths": foreign},
            )

    def _require_git_mutex_available(self) -> None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT owner_id FROM git_mutex WHERE lock_name = 'index'"
            ).fetchone()
        if row is not None:
            raise CoordinatorError(
                "git_mutex_occupied",
                "Another finalize operation owns the Git index mutex",
                details={"owner_id": row["owner_id"]},
            )

    def _require_no_staged_secrets(self) -> None:
        patch = self._git("diff", "--cached", "--no-ext-diff", "--unified=0", "--")
        added = "\n".join(
            line[1:] for line in patch.splitlines() if line.startswith("+") and not line.startswith("+++")
        )
        if FORBIDDEN_SECRET.search(added):
            raise CoordinatorError(
                "finalize_secret_detected",
                "Staged content contains a WeCom webhook URL or credential marker",
            )

    def _require_staged_attribution(
        self, expected: dict[str, str | None], *, maintenance: bool
    ) -> None:
        if maintenance:
            return
        mismatches: list[dict[str, str | None]] = []
        for path, expected_blob in expected.items():
            result = subprocess.run(
                ["git", "rev-parse", "--verify", f":{path}"],
                cwd=self.repo_root,
                check=False,
                capture_output=True,
                text=True,
            )
            staged_blob = result.stdout.strip() if result.returncode == 0 else None
            if staged_blob != expected_blob:
                mismatches.append(
                    {
                        "path": path,
                        "expected": expected_blob,
                        "staged": staged_blob,
                    }
                )
        if mismatches:
            raise CoordinatorError(
                "finalize_staged_attribution_mismatch",
                "Staged content no longer matches the approved Session attribution",
                details={"mismatches": mismatches},
            )

    def _expected_staged_blobs(
        self, paths: tuple[str, ...], *, maintenance: bool
    ) -> dict[str, str | None]:
        if maintenance:
            return {}
        expected: dict[str, str | None] = {}
        for path in paths:
            source = self.repo_root / path
            if not source.is_file():
                expected[path] = None
                continue
            result = subprocess.run(
                ["git", "hash-object", "--path", path, "--", path],
                cwd=self.repo_root,
                check=False,
                capture_output=True,
                text=True,
            )
            if result.returncode != 0:
                raise CoordinatorError(
                    "finalize_blob_hash_failed",
                    f"Cannot calculate the staged blob identity for {path}",
                )
            expected[path] = result.stdout.strip()
        return expected

    def _create_scoped_commit(self, message: str, *, expected_head: str) -> str:
        tree = self._git("write-tree")
        commit_sha = self._git("commit-tree", tree, "-p", expected_head, "-m", message)
        self._git("update-ref", "HEAD", commit_sha, expected_head)
        return commit_sha

    def _match_recovered_commit(
        self,
        current_head: str,
        *,
        start_head: str | None,
        message: str,
        paths: tuple[str, ...],
    ) -> bool:
        if not start_head or current_head == start_head:
            return False
        try:
            parent = self._git("rev-parse", f"{current_head}^")
            subject = self._git("log", "-1", "--format=%s", current_head)
            changed = set(
                self._git_lines("diff-tree", "--no-commit-id", "--name-only", "-r", current_head)
            )
        except subprocess.CalledProcessError:
            return False
        return parent == start_head and subject == message and changed == set(paths)

    @staticmethod
    def _categorize(paths: tuple[str, ...]) -> dict[str, tuple[str, ...]]:
        groups: dict[str, list[str]] = {"code": [], "docs": [], "tests": [], "scripts": []}
        for path in paths:
            parts = Path(path).parts
            lower = path.casefold()
            if "tests" in {part.casefold() for part in parts} or Path(path).name.casefold().startswith("test_"):
                category = "tests"
            elif lower.startswith("docs/") or Path(path).suffix.casefold() in {".md", ".rst"}:
                category = "docs"
            elif lower.startswith("tools/") or Path(path).suffix.casefold() in {".ps1", ".sh", ".bat"}:
                category = "scripts"
            else:
                category = "code"
            groups[category].append(path)
        return {key: tuple(value) for key, value in groups.items()}

    def _is_tracked(self, path: str) -> bool:
        return subprocess.run(
            ["git", "ls-files", "--error-unmatch", "--", path],
            cwd=self.repo_root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        ).returncode == 0

    def _normalize(self, value: str) -> str:
        candidate = (self.repo_root / value).resolve()
        try:
            return candidate.relative_to(self.repo_root).as_posix()
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error

    @staticmethod
    def _validate_message(message: str) -> None:
        if "[zircon-session:" in message.casefold():
            raise CoordinatorError(
                "finalize_message_forbidden", "Session-tagged Git messages are forbidden"
            )
        if not SEMANTIC_MESSAGE.fullmatch(message.strip()):
            raise CoordinatorError(
                "finalize_message_invalid", "Finalize message must be an ordinary semantic commit"
            )

    def _index_path(self) -> Path:
        value = self._git("rev-parse", "--git-path", "index")
        path = Path(value)
        return path if path.is_absolute() else self.repo_root / path

    @staticmethod
    def _restore_index(path: Path, existed: bool, content: bytes) -> None:
        if existed:
            temporary = path.with_suffix(path.suffix + ".zircon-restore")
            temporary.write_bytes(content)
            os.replace(temporary, path)
        else:
            path.unlink(missing_ok=True)

    def _git(self, *arguments: str) -> str:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def _git_lines(self, *arguments: str) -> list[str]:
        output = self._git(*arguments)
        return [line.replace("\\", "/") for line in output.splitlines() if line]

    def _set_request_status(self, request_id: str, status: str) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE finalize_requests SET status = ? WHERE request_id = ?",
                (status, request_id),
            )

    def _persist_finalize_start(
        self,
        request_id: str,
        *,
        start_head: str,
        index_existed: bool,
        index_content: bytes,
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE finalize_requests
                SET status = 'finalizing', start_head = ?, index_existed = ?, index_snapshot = ?
                WHERE request_id = ?
                """,
                (start_head, 1 if index_existed else 0, index_content, request_id),
            )

    def _set_request_failed(self, request_id: str, error: str) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE finalize_requests
                SET status = 'failed', error_text = ?, completed_at = ?
                WHERE request_id = ? AND status <> 'committed'
                """,
                (error, utc_text(), request_id),
            )
