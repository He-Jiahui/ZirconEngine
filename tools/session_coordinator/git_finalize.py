from __future__ import annotations

import codecs
import json
import os
import re
import subprocess
import tempfile
import uuid
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import IntegrityError
from typing import Callable, Iterator

from .baselines import BaselineService, hash_file
from .database import Database
from .failures import WORKFLOW_NODE_ID
from .models import CoordinatorError, SessionStatus, parse_utc, utc_now, utc_text
from .plans import PlanRepository
from .sessions import SessionService

if False:  # pragma: no cover - import only for static typing without a runtime cycle
    from .failures import FailureGraphService


SEMANTIC_MESSAGE = re.compile(r"^[a-z]+(?:\([^)]+\))?!?: .+")
MODULE_PREFIX = re.compile(r"^【[^】\r\n]*】")
_WECOM_ENDPOINT_MARKER = "qyapi" + ".weixin.qq.com/cgi-bin/" + "webhook/send?" + "key="
_SENSITIVE_NAME = (
    r'''(?:ZIRCON_COORDINATOR_MAINTENANCE_TOKEN|api[_-]?key|access[_-]?token|'''
    r'''client[_-]?secret|password)'''
)
_WECOM_WEBHOOK_NAME = r'''(?:WECOM|WECHAT)[_-]?WEBHOOK[_-]?(?:URL|KEY)'''
_SECRET_NAME = rf'''["']?(?:{_SENSITIVE_NAME}|{_WECOM_WEBHOOK_NAME})["']?'''
REDACTABLE_SECRET = re.compile(
    rf'''(?:{re.escape(_WECOM_ENDPOINT_MARKER)}|{_SECRET_NAME}\s*[:=])[^\r\n]*''',
    re.IGNORECASE,
)
_SECRET_NAME_BYTES = _SECRET_NAME.encode("ascii")
_STAGED_SECRET_MARKER_BYTES = re.compile(
    rb'''(?P<endpoint>'''
    + re.escape(_WECOM_ENDPOINT_MARKER.encode("ascii"))
    + rb''')|(?P<name>'''
    + _SECRET_NAME_BYTES
    + rb''')''',
    re.IGNORECASE,
)
_FORCE_ADD_PREFIXES = (".codex/skills/", ".codex/hooks/")
_FORCE_ADD_FILES = {".codex/hooks.json"}
_GIT_PATHSPEC_CHUNK_CHARS = 24_000
_GIT_FAILURE_STDERR_LIMIT = 2_048
_STAGED_BLOB_SCAN_CHUNK_BYTES = 64 * 1_024
_STAGED_SECRET_SCAN_OVERLAP_BYTES = 128
_HORIZONTAL_WHITESPACE_BYTES = frozenset(b" \t\v\f")
_TYPE_DECLARATION_BYTES = re.compile(
    rb'''(?:[&*]\s*)?(?:mut\s+)?[A-Za-z_][A-Za-z0-9_.:]*'''
    rb'''(?:\s*(?:\[[^\]\r\n]{0,384}\]|<[^>\r\n]{0,384}>))?'''
    rb'''(?:\s*\|\s*[A-Za-z_][A-Za-z0-9_.:]*)*\s*[?!]?'''
)
_TYPE_DECLARATION_SOURCE_SUFFIXES = {
    ".c",
    ".cc",
    ".cpp",
    ".cs",
    ".go",
    ".h",
    ".hpp",
    ".java",
    ".js",
    ".jsx",
    ".kt",
    ".py",
    ".pyi",
    ".rs",
    ".swift",
    ".ts",
    ".tsx",
}
_TYPE_DECLARATION_MAX_BYTES = 512
_UTF16_LE_ASCII_RUN = re.compile(rb'''(?:[\t\r\n\x20-\x7e]\x00){4}''')
_UTF16_BE_ASCII_RUN = re.compile(rb'''(?:\x00[\t\r\n\x20-\x7e]){4}''')
_UTF16_PROBE_OVERLAP_BYTES = 256


def _staged_utf16_probes(block: bytes) -> tuple[tuple[str, int], ...]:
    if block.startswith((codecs.BOM_UTF16_LE, codecs.BOM_UTF16_BE)):
        return (("utf-16", 0),)
    candidates: list[tuple[int, str]] = []
    little = _UTF16_LE_ASCII_RUN.search(block)
    if little is not None:
        candidates.append((little.start(), "utf-16-le"))
    big = _UTF16_BE_ASCII_RUN.search(block)
    if big is not None:
        candidates.append((big.start(), "utf-16-be"))
    return tuple(
        (encoding, start)
        for start, encoding in sorted(set(candidates), key=lambda item: item[0])
    )


class _StagedSecretStreamScanner:
    """Detect credential assignments without retaining an entire staged blob."""

    def __init__(self, *, allow_type_declaration: bool = False) -> None:
        self._carry = b""
        self._state: str | None = None
        self._quote_allowed = False
        self._allow_type_declaration = allow_type_declaration
        self._colon_value = bytearray()

    def feed(self, block: bytes) -> bool:
        if not block:
            return False
        data = self._carry + block
        self._carry = b""
        cursor = 0
        while cursor < len(data):
            if self._state is not None:
                detected, cursor = self._consume_assignment(data, cursor)
                if detected:
                    return True
                if self._state is not None:
                    return False
            match = _STAGED_SECRET_MARKER_BYTES.search(data, cursor)
            if match is None:
                self._carry = data[-_STAGED_SECRET_SCAN_OVERLAP_BYTES:]
                return False
            if match.group("endpoint") is not None:
                return True
            self._state = "after_name"
            self._quote_allowed = True
            cursor = match.end()
        return False

    def finish(self) -> bool:
        if self._state == "after_colon":
            detected = self._colon_is_secret()
            self._state = None
            self._colon_value.clear()
            return detected
        return False

    def _consume_assignment(self, data: bytes, cursor: int) -> tuple[bool, int]:
        while cursor < len(data):
            value = data[cursor]
            if self._state == "after_name":
                if self._quote_allowed and value in {ord('"'), ord("'")}:
                    self._quote_allowed = False
                    cursor += 1
                    continue
                self._quote_allowed = False
                if value in _HORIZONTAL_WHITESPACE_BYTES:
                    cursor += 1
                    continue
                if value == ord(":"):
                    self._state = "after_colon"
                    self._colon_value.clear()
                    cursor += 1
                    continue
                if value == ord("="):
                    self._state = "after_operator"
                    cursor += 1
                    continue
                self._state = None
                return False, cursor
            if self._state == "after_colon":
                if value == ord("="):
                    self._state = "after_operator"
                    self._colon_value.clear()
                    cursor += 1
                    continue
                if value in {ord("\r"), ord("\n")}:
                    detected = self._colon_is_secret()
                    self._state = None
                    self._colon_value.clear()
                    return detected, cursor + 1
                self._colon_value.append(value)
                if len(self._colon_value) > _TYPE_DECLARATION_MAX_BYTES:
                    return True, cursor
                cursor += 1
                continue
            if value in _HORIZONTAL_WHITESPACE_BYTES:
                cursor += 1
                continue
            if value in {ord("\r"), ord("\n")}:
                self._state = None
                return False, cursor + 1
            return True, cursor
        return False, cursor

    def _colon_is_secret(self) -> bool:
        value = bytes(self._colon_value).strip()
        if not value:
            return False
        return not (
            self._allow_type_declaration
            and _TYPE_DECLARATION_BYTES.fullmatch(value) is not None
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
        normalized = tuple(sorted({self._normalize(path) for path in paths}, key=str.casefold))
        if not normalized:
            raise CoordinatorError("finalize_paths_empty", "Finalize requires at least one path")
        session = self.sessions.get(session_id)
        formatted_message = self._format_message(message)
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
        if maintenance:
            # Maintenance commits use an isolated index image built from HEAD,
            # so unrelated staged or unattributed work cannot enter the tree.
            baseline = self.baselines.current()
        else:
            self._require_index_scope(normalized)
            # Baseline health summarizes the whole shared worktree.  Its
            # degraded state can be caused by unrelated preserved work, while
            # the scoped ownership and index guards below prove this commit.
            # Do not rehash or turn that observation into a global gate here.
            baseline = self.baselines.current()
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
                    formatted_message,
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
            formatted_message,
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
        commit_sha = ""
        index_snapshot_persisted = False
        index_restored = False
        retain_mutex = False
        try:
            with self.git_mutex(
                session_id, retain_on_error=lambda: retain_mutex
            ):
                index_path = self._index_path()
                index_existed = index_path.exists()
                index_content = index_path.read_bytes() if index_existed else b""
                try:
                    self._persist_finalize_start(
                        preview.request_id,
                        start_head=self._git("rev-parse", "HEAD"),
                        index_existed=index_existed,
                        index_content=index_content,
                    )
                    index_snapshot_persisted = True
                    session = self.sessions.get(session_id)
                    self._require_finalize_guards_under_mutex(session, preview)
                    self._require_attribution(
                        session_id, preview.paths, maintenance=maintenance
                    )
                    self._require_owned_scope(
                        session_id, preview.paths, maintenance=maintenance
                    )
                    if maintenance:
                        self._git("read-tree", self.baselines.current().head_commit)
                    else:
                        self._require_index_scope(preview.paths)
                    ordinary_paths, force_add_paths = self._partition_add_paths(
                        preview.paths,
                        error_code="finalize_ignored_path_forbidden",
                    )
                    self._git_add_partition(ordinary_paths, force_add_paths)
                    staged = self._staged_scope_paths()
                    if set(staged) != set(preview.paths):
                        raise CoordinatorError(
                            "finalize_staged_scope_mismatch",
                            "Staged paths do not exactly match the approved finalize set",
                            details={
                                "approved": list(preview.paths),
                                "staged": list(staged),
                            },
                        )
                    self._require_post_stage_attribution(
                        session_id, preview.paths, maintenance=maintenance
                    )
                    self._require_index_matches_worktree(preview.paths)
                    self._require_post_stage_attribution(
                        session_id, preview.paths, maintenance=maintenance
                    )
                    expected_blobs = self._staged_blobs(preview.paths)
                    self._require_no_staged_secrets()
                    for command in validation_commands:
                        result = subprocess.run(command, cwd=self.repo_root, check=False)
                        if result.returncode != 0:
                            raise CoordinatorError(
                                "finalize_validation_failed",
                                f"Validation command failed with exit code {result.returncode}",
                                details={
                                    "command": list(command),
                                    "exit_code": result.returncode,
                                },
                            )
                    self._require_index_scope(preview.paths)
                    self._require_staged_attribution(expected_blobs, maintenance=False)
                    self._require_no_staged_secrets()
                    session = self.sessions.get(session_id)
                    self._require_finalize_guards_under_mutex(session, preview)
                    commit_sha = self._create_scoped_commit(
                        preview.message,
                        expected_head=self.baselines.current().head_commit,
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
                except BaseException:
                    if not maintenance:
                        try:
                            self._restore_index(index_path, index_existed, index_content)
                            index_restored = True
                        except BaseException:
                            retain_mutex = True
                            raise
                    raise
                finally:
                    if maintenance:
                        try:
                            self._restore_index(index_path, index_existed, index_content)
                            index_restored = True
                            if committed:
                                self._reset_index_paths(commit_sha, preview.paths)
                        except BaseException:
                            retain_mutex = True
                            raise
                if committed:
                    with self.database.transaction() as connection:
                        connection.execute(
                            """
                            UPDATE finalize_requests
                            SET status = 'committed', commit_sha = ?, completed_at = ?,
                                index_snapshot = NULL
                            WHERE request_id = ?
                            """,
                            (commit_sha, utc_text(), preview.request_id),
                        )
        except BaseException as error:
            recovery_pending = index_snapshot_persisted and not index_restored
            if not committed and not recovery_pending:
                self._set_request_failed(preview.request_id, str(error))
            if not maintenance:
                self.sessions.set_status(
                    session_id,
                    SessionStatus.COMPLETED,
                    reason=(
                        "finalize baseline reconciliation pending"
                        if committed or recovery_pending
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

    def commit_milestone(
        self,
        session_id: str,
        *,
        paths: list[str] | tuple[str, ...],
        message: str,
        failure_workflow_node_keys: tuple[str, ...],
        validation_commands: tuple[tuple[str, ...], ...] = (),
        precommit_guard: Callable[[], None] | None = None,
        request_id: str | None = None,
    ) -> FinalizeResult:
        """Commit one accepted milestone while keeping its Session active.

        The coordinator Git mutex closes the checker-to-commit race. The index
        is revalidated under that mutex, and ``update-ref`` uses the baseline
        HEAD as a compare-and-swap guard.
        """
        normalized = tuple(sorted({self._normalize(path) for path in paths}, key=str.casefold))
        if not normalized:
            raise CoordinatorError("milestone_paths_empty", "Milestone commit requires paths")
        if (
            not isinstance(failure_workflow_node_keys, tuple)
            or not failure_workflow_node_keys
            or any(
                not isinstance(node_key, str)
                or WORKFLOW_NODE_ID.fullmatch(node_key) is None
                for node_key in failure_workflow_node_keys
            )
        ):
            raise CoordinatorError(
                "milestone_failure_scope_invalid",
                "Milestone commit requires explicit workflow node Failure scope",
            )
        untracked_paths: tuple[str, ...] = ()
        request_id = request_id or uuid.uuid4().hex
        session = self.sessions.get(session_id)
        formatted_message = self._format_message(message)
        if session.status not in {SessionStatus.ACTIVE, SessionStatus.WAITING_VALIDATION}:
            raise CoordinatorError(
                "milestone_session_not_active",
                f"Session {session_id} cannot commit a milestone while {session.status.value}",
            )
        committed = False
        commit_sha = ""
        retain_mutex = False
        with self.git_mutex(
            session_id, retain_on_error=lambda: retain_mutex
        ):
            session = self.sessions.get(session_id)
            if session.status not in {SessionStatus.ACTIVE, SessionStatus.WAITING_VALIDATION}:
                raise CoordinatorError(
                    "milestone_session_not_active",
                    f"Session {session_id} changed status before commit",
                )
            baseline = self.baselines.current()
            expected_head = baseline.head_commit
            if expected_head != self._git("rev-parse", "HEAD"):
                raise CoordinatorError(
                    "milestone_baseline_head_changed",
                    "HEAD changed after the coordinator baseline was captured",
                )
            head_tracked = self._head_tracked_paths(normalized)
            untracked_paths = tuple(path for path in normalized if path not in head_tracked)
            ordinary_paths, force_add_paths = self._partition_add_paths(
                normalized,
                error_code="milestone_ignored_path_forbidden",
            )
            self._require_attribution(session_id, normalized, maintenance=False)
            self._require_owned_scope(session_id, normalized, maintenance=False)
            self._require_plan_outputs(session, normalized, maintenance=False)
            self._require_milestone_failure_acceptance(
                session,
                failure_workflow_node_keys,
                normalized,
            )
            self._require_live_owned_leases(session_id, normalized)
            self._require_no_pending_patches(session_id)
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO finalize_requests(
                        request_id, session_id, message, paths_json, categories_json,
                        untracked_json, validation_json, maintenance, status, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 'finalizing', ?)
                    """,
                    (
                        request_id,
                        session_id,
                        formatted_message,
                        json.dumps(normalized),
                        json.dumps({key: list(value) for key, value in self._categorize(normalized).items()}),
                        json.dumps(untracked_paths),
                        json.dumps(validation_commands),
                        utc_text(),
                    ),
                )
            index_path = self._index_path()
            index_existed = index_path.exists()
            index_content = index_path.read_bytes() if index_existed else b""
            self._persist_finalize_start(
                request_id,
                start_head=expected_head,
                index_existed=index_existed,
                index_content=index_content,
            )
            finalize_error: BaseException | None = None
            try:
                # Build the commit tree from HEAD and this manifest only. The
                # shared index is restored afterwards so another Session's
                # staged work remains intact and cannot enter this commit.
                self._git("read-tree", expected_head)
                self._git_add_partition(ordinary_paths, force_add_paths)
                self._require_index_scope(normalized)
                self._require_post_stage_attribution(
                    session_id, normalized, maintenance=False
                )
                self._require_index_matches_worktree(normalized)
                self._require_post_stage_attribution(
                    session_id, normalized, maintenance=False
                )
                expected_blobs = self._staged_blobs(normalized)
                self._require_no_staged_secrets()
                for command in validation_commands:
                    result = subprocess.run(command, cwd=self.repo_root, check=False)
                    if result.returncode != 0:
                        raise CoordinatorError(
                            "milestone_validation_failed",
                            f"Milestone validation failed with exit code {result.returncode}",
                            details={"command": list(command), "exit_code": result.returncode},
                        )
                self._require_index_scope(normalized)
                self._require_staged_attribution(expected_blobs, maintenance=False)
                self._require_no_staged_secrets()
                self._require_live_owned_leases(session_id, normalized)
                self._require_milestone_failure_acceptance(
                    session,
                    failure_workflow_node_keys,
                    normalized,
                )
                if precommit_guard is not None:
                    precommit_guard()
                commit_sha = self._create_scoped_commit(
                    formatted_message, expected_head=expected_head
                )
                committed = True
                with self.database.transaction() as connection:
                    connection.execute(
                        """UPDATE finalize_requests SET ref_updated_sha = ?
                           WHERE request_id = ?""",
                        (commit_sha, request_id),
                    )
                self.baselines.accept_commit(
                    normalized,
                    commit_sha=commit_sha,
                    reason=f"milestone commit {commit_sha}",
                )
            except BaseException as error:
                finalize_error = error
                raise
            finally:
                try:
                    self._restore_index(index_path, index_existed, index_content)
                    if committed:
                        self._reset_index_paths(commit_sha, normalized)
                except BaseException:
                    retain_mutex = True
                    raise
                if finalize_error is not None and not committed:
                    self._set_request_failed(request_id, str(finalize_error))
            if committed:
                with self.database.transaction() as connection:
                    connection.execute(
                        """UPDATE finalize_requests
                           SET status = 'committed', commit_sha = ?, ref_updated_sha = ?, completed_at = ?,
                               index_snapshot = NULL
                           WHERE request_id = ?""",
                        (commit_sha, commit_sha, utc_text(), request_id),
                    )
        categories = self._categorize(normalized)
        return FinalizeResult(
            request_id,
            commit_sha,
            formatted_message,
            categories,
            untracked_paths,
        )

    @contextmanager
    def git_mutex(
        self,
        owner_id: str,
        *,
        retain_on_error: Callable[[], bool] | None = None,
    ) -> Iterator[None]:
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
        failed = False
        try:
            yield
        except BaseException:
            failed = True
            raise
        finally:
            retain = False
            if failed and retain_on_error is not None:
                try:
                    retain = retain_on_error()
                except BaseException:
                    retain = True
            if not retain:
                with self.database.transaction() as connection:
                    connection.execute(
                        "DELETE FROM git_mutex WHERE lock_name = 'index' AND owner_id = ?",
                        (owner_id,),
                    )

    def cleanup_shared_index(self, owner_id: str) -> dict[str, object]:
        """Reset only stale shared-index staging to HEAD under the coordinator mutex."""
        if not owner_id.strip():
            raise ValueError("Index cleanup owner cannot be empty")
        audit_id = uuid.uuid4().hex
        with self.git_mutex(owner_id):
            head = self._git("rev-parse", "HEAD")
            paths = self._staged_scope_paths()
            classification = self._classify_staged_paths(paths)
            if paths:
                # `reset --mixed <HEAD>` changes the shared index only.  The
                # working tree stays untouched, so Sessions retain every byte
                # of their pending work while stale staging is cleared.
                self._git("reset", "--mixed", "--quiet", head)
            remaining = self._staged_scope_paths()
            if remaining:
                raise CoordinatorError(
                    "index_cleanup_incomplete",
                    "Shared index still contains staged paths after cleanup",
                    details={"paths": list(remaining)},
                )
            payload = {
                "auditId": audit_id,
                "head": head,
                "stagedCount": len(paths),
                "remainingStagedCount": 0,
                "classification": classification,
            }
            with self.database.transaction() as connection:
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    ("git.index_cleanup", json.dumps(payload, sort_keys=True), utc_text()),
                )
        return {
            "audit_id": audit_id,
            "head": head,
            "paths": list(paths),
            "staged_count": len(paths),
            "remaining_staged_count": 0,
            "classification": classification,
        }

    def _classify_staged_paths(self, paths: tuple[str, ...]) -> dict[str, int]:
        if not paths:
            return {"unattributed": 0, "stale_owner": 0, "other_owner": 0}
        keys = {path.casefold() for path in paths}
        with self.database.connect() as connection:
            attributions = {
                row["display_path"].casefold(): row["session_id"]
                for row in connection.execute(
                    "SELECT display_path, session_id FROM attributions"
                )
            }
            statuses = {
                row["session_id"]: row["status"]
                for row in connection.execute("SELECT session_id, status FROM sessions")
            }
        result = {"unattributed": 0, "stale_owner": 0, "other_owner": 0}
        for key in keys:
            session_id = attributions.get(key)
            if session_id is None:
                result["unattributed"] += 1
            elif statuses.get(session_id) == SessionStatus.STALE.value:
                result["stale_owner"] += 1
            else:
                result["other_owner"] += 1
        return result

    def reconcile_request(self, request_id: str) -> FinalizeResult | None:
        """Finish every post-CAS obligation before workflow evidence may succeed."""
        retain_mutex = False
        with self.git_mutex(
            f"reconcile:{request_id}", retain_on_error=lambda: retain_mutex
        ):
            with self.database.connect() as connection:
                row = connection.execute(
                    "SELECT * FROM finalize_requests WHERE request_id=?",
                    (request_id,),
                ).fetchone()
            if row is None:
                return None
            commit_sha = row["commit_sha"] or row["ref_updated_sha"]
            if not commit_sha:
                return None
            if self._git("rev-parse", "HEAD") != commit_sha:
                raise CoordinatorError(
                    "finalize_reconcile_head_changed",
                    "Cannot reconcile a finalized commit after HEAD changed",
                    details={"requestId": request_id, "commitSha": commit_sha},
                )
            paths = tuple(json.loads(row["paths_json"]))
            if row["index_existed"] is not None and row["index_snapshot"] is not None:
                try:
                    self._restore_index(
                        self._index_path(),
                        bool(row["index_existed"]),
                        bytes(row["index_snapshot"]),
                    )
                    self._reset_index_paths(commit_sha, paths)
                except BaseException:
                    retain_mutex = True
                    raise
            baseline = self.baselines.current()
            if baseline.head_commit != commit_sha:
                self.baselines.accept_commit(
                    paths,
                    commit_sha=commit_sha,
                    reason=f"forward reconcile finalize {commit_sha}",
                )
            with self.database.transaction() as connection:
                connection.execute(
                    """UPDATE finalize_requests
                       SET status='committed', commit_sha=?, ref_updated_sha=?,
                           error_text=NULL, completed_at=COALESCE(completed_at, ?),
                           index_snapshot=NULL
                       WHERE request_id=?""",
                    (commit_sha, commit_sha, utc_text(), request_id),
                )
            categories = {
                key: tuple(value)
                for key, value in json.loads(row["categories_json"]).items()
            }
            return FinalizeResult(
                request_id,
                commit_sha,
                row["message"],
                categories,
                tuple(json.loads(row["untracked_json"])),
            )

    def recover_stale_mutex(self) -> int:
        """Recover an index transaction left by the previous single service process."""
        self._require_recovery_process_ownership()
        recovery_owner = f"recovery:{os.getpid()}:{uuid.uuid4().hex}"
        with self.database.transaction() as connection:
            rows = connection.execute("SELECT owner_id FROM git_mutex").fetchall()
            finalizing = connection.execute(
                """
                SELECT request_id, session_id, message, paths_json, start_head,
                       index_existed, index_snapshot, ref_updated_sha, status, commit_sha
                FROM finalize_requests
                WHERE status = 'finalizing'
                   OR (status = 'committed' AND commit_sha IS NULL AND ref_updated_sha IS NOT NULL)
                   OR (
                       status = 'committed'
                       AND session_id IN (
                           SELECT session_id FROM sessions WHERE status = 'finalizing'
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM finalize_requests pending
                           WHERE pending.session_id = finalize_requests.session_id
                             AND pending.status = 'finalizing'
                       )
                       AND request_id = (
                           SELECT committed.request_id
                           FROM finalize_requests committed
                           WHERE committed.session_id = finalize_requests.session_id
                             AND committed.status = 'committed'
                           ORDER BY committed.created_at DESC, committed.request_id DESC
                           LIMIT 1
                       )
                   )
                ORDER BY created_at
                """
            ).fetchall()
            recoveries = [dict(row) for row in finalizing]
            if rows:
                connection.execute(
                    """UPDATE git_mutex
                       SET owner_id = ?, acquired_at = ?
                       WHERE lock_name = 'index'""",
                    (recovery_owner, utc_text()),
                )
            else:
                connection.execute(
                    "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, ?)",
                    (recovery_owner, utc_text()),
                )
        completed = False
        try:
            current_head = self._git("rev-parse", "HEAD")
            for recovery in recoveries:
                request_id = str(recovery["request_id"])
                session_id = str(recovery["session_id"])
                if recovery.get("status") == "committed" and recovery.get("commit_sha"):
                    with self.database.transaction() as connection:
                        connection.execute(
                            """
                            UPDATE sessions
                            SET status = 'completed', status_reason = ?, updated_at = ?
                            WHERE session_id = ? AND status = 'finalizing'
                            """,
                            (
                                f"recovered finalized commit {recovery['commit_sha']}",
                                utc_text(),
                                session_id,
                            ),
                        )
                    continue
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
                    if index_existed is not None and index_snapshot is not None:
                        self._restore_index(
                            self._index_path(),
                            bool(index_existed),
                            bytes(index_snapshot),
                        )
                        self._reset_index_paths(current_head, paths)
                    self.baselines.accept_commit(
                        paths,
                        commit_sha=current_head,
                        reason=f"recovered finalize commit {current_head}",
                    )
                    reason = f"recovered finalized commit {current_head}"
                    with self.database.transaction() as connection:
                        connection.execute(
                            """
                            UPDATE finalize_requests
                            SET status = 'committed', commit_sha = ?, error_text = NULL,
                                completed_at = ?, index_snapshot = NULL
                            WHERE request_id = ?
                            """,
                            (current_head, utc_text(), request_id),
                        )
                        connection.execute(
                            """
                            UPDATE sessions
                            SET status = 'completed', status_reason = ?, updated_at = ?
                            WHERE session_id = ? AND status = 'finalizing'
                            """,
                            (reason, utc_text(), session_id),
                        )
                else:
                    if (
                        start_head != current_head
                        or index_existed is None
                        or index_snapshot is None
                    ):
                        raise CoordinatorError(
                            "finalize_recovery_head_ambiguous",
                            "Cannot discard a finalize index snapshot after HEAD changed",
                            details={
                                "request_id": request_id,
                                "start_head": start_head,
                                "current_head": current_head,
                            },
                        )
                    self._restore_index(
                        self._index_path(),
                        bool(index_existed),
                        bytes(index_snapshot),
                    )
                    reason = "finalize interrupted by service restart"
                    with self.database.transaction() as connection:
                        connection.execute(
                            """
                            UPDATE finalize_requests
                            SET status = 'failed', error_text = ?, completed_at = ?,
                                index_snapshot = NULL
                            WHERE request_id = ?
                            """,
                            ("service restarted during finalize", utc_text(), request_id),
                        )
                        connection.execute(
                            """
                            UPDATE sessions
                            SET status = 'completed', status_reason = ?, updated_at = ?
                            WHERE session_id = ? AND status = 'finalizing'
                            """,
                            (reason, utc_text(), session_id),
                        )
            completed = True
        finally:
            if completed:
                with self.database.transaction() as connection:
                    connection.execute(
                        "DELETE FROM git_mutex WHERE lock_name = 'index' AND owner_id = ?",
                        (recovery_owner,),
                    )
        return len(rows)

    def _require_recovery_process_ownership(self) -> None:
        lock_path = self.database.path.parent / "coordinator.lock"
        try:
            payload = json.loads(lock_path.read_text(encoding="utf-8"))
            owner_pid = int(payload["pid"])
        except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "finalize_recovery_process_unproven",
                "Cannot recover the Git mutex without the daemon process lock",
            ) from error
        if owner_pid != os.getpid():
            raise CoordinatorError(
                "finalize_recovery_process_unproven",
                "The current process does not own the daemon process lock",
                details={"owner_pid": owner_pid, "process_id": os.getpid()},
            )

    def _require_finalize_guards_under_mutex(
        self, session, preview: FinalizePreview
    ) -> None:
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
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT display_path, content_hash FROM attributions WHERE session_id = ?",
                (session_id,),
            ).fetchall()
        ignored_local_session_state = {
            path
            for path in self._ignored_paths(
                tuple(row["display_path"] for row in rows)
            )
            if self._is_local_session_state(path)
        }
        attributed_rows = tuple(
            row for row in rows if row["display_path"] not in ignored_local_session_state
        )
        # This gate runs under the shared Git mutex.  Never invoke
        # ``baselines.diff()`` here: it rebuilds a full workspace manifest and
        # makes an unrelated dirty tree stall every milestone commit.  The
        # contract is session-relative, so compare only the Session's
        # attributed paths with HEAD and confirm their attributed bytes.
        differing_from_head = self._worktree_paths_differing_from_head(
            tuple(row["display_path"] for row in attributed_rows)
        )
        owned_dirty: set[str] = set()
        for row in attributed_rows:
            path = row["display_path"]
            if path in differing_from_head and row["content_hash"] == hash_file(
                self.repo_root / path
            ):
                owned_dirty.add(path)
        omitted = sorted(owned_dirty - set(approved), key=str.casefold)
        if omitted:
            raise CoordinatorError(
                "finalize_owned_path_omitted",
                "Finalize manifest omits current Session-owned changes",
                details={"paths": omitted},
            )
        unchanged = sorted(set(approved) - owned_dirty, key=str.casefold)
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
            if not decision.allowed and not self._is_returned_fixed_handoff(
                session.plan_path, path
            ):
                raise CoordinatorError(
                    "finalize_invalid_plan_output",
                    decision.message,
                    details={"path": path, "plan_code": decision.code},
                )

    def _is_returned_fixed_handoff(self, session_plan_path: str, path: str) -> bool:
        normalized = path.replace("\\", "/")
        if not Path(normalized).name.startswith("fixed-"):
            return False
        if self.failures is not None:
            self.failures.import_repository()
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT kind, status, fixing_plan, origin_child_dir
                FROM failure_nodes
                WHERE artifact_path=? AND kind='fixed' AND status='fixed'
                """,
                (normalized,),
            ).fetchone()
        if row is None or row["fixing_plan"] != session_plan_path:
            return False
        origin_child = str(row["origin_child_dir"]).rstrip("/")
        return normalized.startswith(origin_child + "/")

    def _require_failure_acceptance(self, session, *, maintenance: bool) -> None:
        if maintenance or self.failures is None or not session.plan_path:
            return
        self._require_failure_graph_valid(session)
        self._raise_open_failures(
            self.failures.open_related_to_plan(session.plan_path)
        )

    def _require_milestone_failure_acceptance(
        self,
        session,
        failure_workflow_node_keys: tuple[str, ...],
        manifest_paths: tuple[str, ...],
    ) -> None:
        if self.failures is None or not session.plan_path:
            return
        self._require_failure_graph_valid(session)
        self._raise_open_failures(
            self.failures.open_for_manifest(
                session.plan_path,
                failure_workflow_node_keys,
                manifest_paths,
            )
        )

    def _require_failure_graph_valid(self, session) -> None:
        self.failures.import_repository()
        diagnostics = self.failures.validator_errors_for_plan(session.plan_path)
        if diagnostics:
            raise CoordinatorError(
                "finalize_failure_graph_invalid",
                "Failure handoff graph has canonical Markdown diagnostics",
                details={"diagnostics": diagnostics},
            )

    @staticmethod
    def _raise_open_failures(open_failures) -> None:
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

    def _require_live_owned_leases(self, session_id: str, paths: tuple[str, ...]) -> None:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT path_key, session_id, expires_at FROM leases"
            ).fetchall()
        leases = {row["path_key"]: row for row in rows}
        now = utc_now()
        missing = []
        for path in paths:
            row = leases.get(path.casefold())
            if (
                row is None
                or row["session_id"] != session_id
                or parse_utc(row["expires_at"]) <= now
            ):
                missing.append(path)
        if missing:
            raise CoordinatorError(
                "milestone_lease_missing",
                "Milestone paths require live leases owned by the committing Session",
                details={"paths": missing},
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
        staged = set(self._staged_scope_paths())
        foreign = sorted(staged - set(approved), key=str.casefold)
        if foreign:
            raise CoordinatorError(
                "finalize_foreign_index",
                "Git index contains paths outside the approved finalize set",
                details={"paths": foreign},
            )

    def _staged_scope_paths(self) -> tuple[str, ...]:
        # Immutable manifests describe delete/add paths independently. Git's
        # rename detection collapses that pair to the destination path and
        # makes an exact scoped finalize look incomplete even though both
        # index mutations are present.
        return tuple(
            self._git_lines("diff", "--cached", "--name-only", "--no-renames")
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
        staged = self._staged_content_blobs(self._staged_scope_paths())
        blob_paths: dict[str, list[str]] = {}
        for path, blob_id in staged.items():
            if blob_id is not None:
                blob_paths.setdefault(blob_id, []).append(path)
        blob_ids = tuple(sorted(blob_paths))
        if not blob_ids:
            return
        with tempfile.TemporaryFile() as stderr_stream:
            try:
                process = subprocess.Popen(
                    ["git", "cat-file", "--batch"],
                    cwd=self.repo_root,
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=stderr_stream,
                )
            except OSError as error:
                raise CoordinatorError(
                    "finalize_secret_scan_failed",
                    "Cannot start Git while scanning staged blobs for credentials",
                    details={"error": self._safe_git_stderr(str(error))},
                ) from error
            try:
                if process.stdin is None or process.stdout is None:
                    raise CoordinatorError(
                        "finalize_secret_scan_failed",
                        "Git staged-blob scanner did not expose its batch pipes",
                    )
                for blob_id in blob_ids:
                    allow_type_declaration = all(
                        Path(path).suffix.casefold() in _TYPE_DECLARATION_SOURCE_SUFFIXES
                        for path in blob_paths[blob_id]
                    )
                    process.stdin.write(blob_id.encode("ascii") + b"\n")
                    process.stdin.flush()
                    header = process.stdout.readline()
                    fields = header.split()
                    if (
                        len(fields) != 3
                        or fields[0] != blob_id.encode("ascii")
                        or fields[1] != b"blob"
                    ):
                        raise CoordinatorError(
                            "finalize_secret_scan_failed",
                            "Git returned an invalid staged-blob batch header",
                        )
                    try:
                        remaining = int(fields[2])
                    except ValueError as error:
                        raise CoordinatorError(
                            "finalize_secret_scan_failed",
                            "Git returned an invalid staged-blob size",
                        ) from error
                    raw_scanner = _StagedSecretStreamScanner(
                        allow_type_declaration=allow_type_declaration
                    )
                    unicode_streams: list[tuple[object, _StagedSecretStreamScanner]] = []
                    unicode_probe = b""
                    while remaining:
                        block = process.stdout.read(
                            min(remaining, _STAGED_BLOB_SCAN_CHUNK_BYTES)
                        )
                        if not block:
                            raise CoordinatorError(
                                "finalize_secret_scan_failed",
                                "Git ended a staged blob before its declared size",
                            )
                        remaining -= len(block)
                        detected = raw_scanner.feed(block)
                        if not unicode_streams:
                            probe_data = unicode_probe + block
                            probes = _staged_utf16_probes(probe_data)
                            if probes:
                                for encoding, start in probes:
                                    decoder = codecs.getincrementaldecoder(encoding)(
                                        errors="replace"
                                    )
                                    scanner = _StagedSecretStreamScanner(
                                        allow_type_declaration=allow_type_declaration
                                    )
                                    text = decoder.decode(
                                        probe_data[start:], final=remaining == 0
                                    )
                                    detected = detected or scanner.feed(
                                        text.encode("utf-8")
                                    )
                                    unicode_streams.append((decoder, scanner))
                            else:
                                unicode_probe = probe_data[-_UTF16_PROBE_OVERLAP_BYTES:]
                        else:
                            for decoder, scanner in unicode_streams:
                                text = decoder.decode(block, final=remaining == 0)
                                detected = detected or scanner.feed(text.encode("utf-8"))
                        if detected:
                            raise CoordinatorError(
                                "finalize_secret_detected",
                                "Staged content contains a maintenance capability or credential",
                            )
                    if raw_scanner.finish() or any(
                        scanner.finish() for _, scanner in unicode_streams
                    ):
                        raise CoordinatorError(
                            "finalize_secret_detected",
                            "Staged content contains a maintenance capability or credential",
                        )
                    if process.stdout.read(1) != b"\n":
                        raise CoordinatorError(
                            "finalize_secret_scan_failed",
                            "Git returned an invalid staged-blob batch terminator",
                        )
                process.stdin.close()
                process.stdin = None
                return_code = process.wait()
                stderr_stream.seek(0)
                stderr = stderr_stream.read(_GIT_FAILURE_STDERR_LIMIT * 2)
                if return_code != 0:
                    raise CoordinatorError(
                        "finalize_secret_scan_failed",
                        "Git failed while scanning staged blobs for credentials",
                        details={
                            "return_code": return_code,
                            "error": self._safe_git_stderr(stderr),
                        },
                    )
            except OSError as error:
                raise CoordinatorError(
                    "finalize_secret_scan_failed",
                    "Cannot read staged blobs while scanning for credentials",
                    details={"error": self._safe_git_stderr(str(error))},
                ) from error
            finally:
                try:
                    if process.stdin is not None:
                        process.stdin.close()
                except OSError:
                    pass
                try:
                    if process.poll() is None:
                        process.kill()
                    process.wait()
                except OSError:
                    pass
                try:
                    if process.stdout is not None:
                        process.stdout.close()
                except OSError:
                    pass

    def _require_staged_attribution(
        self, expected: dict[str, str | None], *, maintenance: bool
    ) -> None:
        if maintenance:
            return
        actual = self._staged_blobs(tuple(expected))
        mismatches: list[dict[str, str | None]] = []
        for path, expected_blob in expected.items():
            staged_blob = actual[path]
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

    def _require_post_stage_attribution(
        self,
        session_id: str,
        paths: tuple[str, ...],
        *,
        maintenance: bool,
    ) -> None:
        if maintenance:
            return
        try:
            self._require_attribution(session_id, paths, maintenance=False)
        except CoordinatorError as error:
            raise CoordinatorError(
                "finalize_staged_attribution_mismatch",
                "Worktree content changed between attribution and scoped staging",
                details={"cause": error.code, **error.details},
            ) from error

    def _require_index_matches_worktree(self, paths: tuple[str, ...]) -> None:
        for chunk in self._pathspec_chunks(paths):
            try:
                result = subprocess.run(
                    [
                        "git",
                        "--literal-pathspecs",
                        "diff",
                        "--quiet",
                        "--no-ext-diff",
                        "--",
                        *chunk,
                    ],
                    cwd=self.repo_root,
                    check=False,
                    capture_output=True,
                )
            except OSError as error:
                raise CoordinatorError(
                    "finalize_index_worktree_scan_failed",
                    "Cannot compare the scoped Git index with the worktree",
                    details={"error": str(error)},
                ) from error
            if result.returncode == 0:
                continue
            if result.returncode == 1:
                raise CoordinatorError(
                    "finalize_staged_attribution_mismatch",
                    "Scoped staged content does not match the attributed worktree",
                    details={"paths": list(chunk)},
                )
            raise CoordinatorError(
                "finalize_index_worktree_scan_failed",
                "Git failed while comparing the scoped index with the worktree",
                details={
                    "return_code": result.returncode,
                    "error": self._safe_git_stderr(os.fsdecode(result.stderr)),
                },
            )

    def _staged_blobs(self, paths: tuple[str, ...]) -> dict[str, str | None]:
        try:
            return self._staged_blobs_unchecked(paths)
        except (subprocess.CalledProcessError, OSError) as error:
            stderr = error.stderr if isinstance(error, subprocess.CalledProcessError) else None
            raise CoordinatorError(
                "finalize_index_blob_scan_failed",
                "Cannot read staged blob identities from the Git index",
                details={"error": self._safe_git_stderr(stderr or str(error))},
            ) from error

    def _staged_content_blobs(
        self, paths: tuple[str, ...]
    ) -> dict[str, str | None]:
        try:
            return self._staged_blobs_unchecked(paths, skip_gitlinks=True)
        except (subprocess.CalledProcessError, OSError) as error:
            stderr = error.stderr if isinstance(error, subprocess.CalledProcessError) else None
            raise CoordinatorError(
                "finalize_index_blob_scan_failed",
                "Cannot read staged content identities from the Git index",
                details={"error": self._safe_git_stderr(stderr or str(error))},
            ) from error

    def _staged_blobs_unchecked(
        self, paths: tuple[str, ...], *, skip_gitlinks: bool = False
    ) -> dict[str, str | None]:
        requested_by_key = {path.casefold(): path for path in paths}
        staged: dict[str, str | None] = {path: None for path in paths}
        for chunk in self._pathspec_chunks(paths):
            result = subprocess.run(
                [
                    "git",
                    "--literal-pathspecs",
                    "ls-files",
                    "--stage",
                    "-z",
                    "--",
                    *chunk,
                ],
                cwd=self.repo_root,
                check=True,
                capture_output=True,
            )
            for entry in result.stdout.split(b"\0"):
                if not entry:
                    continue
                metadata, separator, raw_path = entry.partition(b"\t")
                fields = metadata.split()
                if not separator or len(fields) != 3 or fields[2] != b"0":
                    raise CoordinatorError(
                        "finalize_index_blob_scan_failed",
                        "Git returned an invalid or unmerged staged index entry",
                    )
                display_path = os.fsdecode(raw_path)
                requested = requested_by_key.get(display_path.casefold())
                if requested is None:
                    raise CoordinatorError(
                        "finalize_index_blob_scan_failed",
                        f"Git returned an out-of-scope staged path: {display_path}",
                    )
                if skip_gitlinks and fields[0] == b"160000":
                    continue
                try:
                    staged[requested] = fields[1].decode("ascii")
                except UnicodeDecodeError as error:
                    raise CoordinatorError(
                        "finalize_index_blob_scan_failed",
                        "Git returned a non-ASCII staged blob identity",
                    ) from error
        return staged

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
        except CoordinatorError:
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

    def _ignored_paths(self, paths: tuple[str, ...]) -> set[str]:
        if not paths:
            return set()
        payload = b"\0".join(os.fsencode(path) for path in paths) + b"\0"
        try:
            result = subprocess.run(
                ["git", "check-ignore", "--no-index", "--stdin", "-z"],
                cwd=self.repo_root,
                check=False,
                input=payload,
                capture_output=True,
            )
        except OSError as error:
            raise CoordinatorError(
                "finalize_ignore_scan_failed",
                "Cannot classify ignored milestone paths",
                details={"error": str(error)},
            ) from error
        if result.returncode not in {0, 1}:
            raise CoordinatorError(
                "finalize_ignore_scan_failed",
                "Git failed while classifying ignored milestone paths",
                details={
                    "return_code": result.returncode,
                    "error": self._safe_git_stderr(os.fsdecode(result.stderr)),
                },
            )
        requested_by_key = {path.casefold(): path for path in paths}
        ignored: set[str] = set()
        for raw_path in result.stdout.split(b"\0"):
            if not raw_path:
                continue
            display_path = os.fsdecode(raw_path)
            requested = requested_by_key.get(display_path.casefold())
            if requested is None:
                raise CoordinatorError(
                    "finalize_ignore_scan_failed",
                    f"Git returned an out-of-scope ignored path: {display_path}",
                )
            ignored.add(requested)
        return ignored

    @staticmethod
    def _is_force_add_eligible(path: str) -> bool:
        normalized = path.casefold()
        return normalized in _FORCE_ADD_FILES or normalized.startswith(_FORCE_ADD_PREFIXES)

    def _partition_add_paths(
        self,
        paths: tuple[str, ...],
        *,
        error_code: str,
    ) -> tuple[tuple[str, ...], tuple[str, ...]]:
        ignored_set = self._ignored_paths(paths)
        force_add_paths = tuple(path for path in paths if path in ignored_set)
        forbidden_ignored = tuple(
            path for path in force_add_paths if not self._is_force_add_eligible(path)
        )
        if forbidden_ignored:
            raise CoordinatorError(
                error_code,
                "Only repository-owned Codex skills and hooks may be force-added",
                details={"paths": list(forbidden_ignored)},
            )
        ordinary_paths = tuple(path for path in paths if path not in ignored_set)
        return ordinary_paths, force_add_paths

    def _git_add_partition(
        self,
        ordinary_paths: tuple[str, ...],
        force_add_paths: tuple[str, ...],
    ) -> None:
        if ordinary_paths:
            self._git_add_paths(ordinary_paths)
        if force_add_paths:
            # These paths passed both the Session ownership gates and the
            # narrow repository-control allowlist before force-add.
            self._git_add_paths(force_add_paths, force=True)

    @staticmethod
    def _is_local_session_state(path: str) -> bool:
        return path.replace("\\", "/").casefold().startswith(".codex/sessions/")

    def _git_add_paths(self, paths: tuple[str, ...], *, force: bool = False) -> None:
        for chunk in self._pathspec_chunks(paths):
            arguments = ["add", "-A"]
            if force:
                arguments.append("-f")
            self._git(*arguments, "--", *chunk)

    def _reset_index_paths(self, commit_sha: str, paths: tuple[str, ...]) -> None:
        for chunk in self._pathspec_chunks(paths):
            self._git("reset", "--quiet", commit_sha, "--", *chunk)

    def _head_tracked_paths(self, paths: tuple[str, ...]) -> set[str]:
        tracked_keys: set[str] = set()
        for chunk in self._pathspec_chunks(paths):
            tracked_keys.update(
                path.casefold()
                for path in self._git_path_output(
                    "--literal-pathspecs",
                    "ls-tree",
                    "-r",
                    "-z",
                    "--name-only",
                    "HEAD",
                    "--",
                    *chunk,
                )
            )
        return {path for path in paths if path.casefold() in tracked_keys}

    def _worktree_paths_differing_from_head(
        self, paths: list[str] | tuple[str, ...]
    ) -> set[str]:
        normalized = tuple(dict.fromkeys(paths))
        tracked = self._head_tracked_paths(normalized)
        dirty_keys: set[str] = set()
        for chunk in self._pathspec_chunks(normalized):
            dirty_keys.update(
                path.casefold()
                for path in self._git_path_output(
                    "--literal-pathspecs",
                    "diff",
                    "--name-only",
                    "-z",
                    "--no-ext-diff",
                    "--no-renames",
                    "HEAD",
                    "--",
                    *chunk,
                )
            )
        dirty = {path for path in normalized if path.casefold() in dirty_keys}
        dirty.update(
            path
            for path in normalized
            if path not in tracked and (self.repo_root / path).exists()
        )
        return dirty

    def _git_path_output(self, *arguments: str) -> tuple[str, ...]:
        try:
            result = subprocess.run(
                ["git", *arguments],
                cwd=self.repo_root,
                check=True,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
        except subprocess.CalledProcessError as error:
            raise CoordinatorError(
                "finalize_head_content_failed",
                "Cannot compare the current workspace content with HEAD",
                details={"stderr": self._safe_git_stderr(error.stderr)},
            ) from error
        except OSError as error:
            raise CoordinatorError(
                "finalize_head_content_failed",
                "Cannot start Git while comparing workspace content with HEAD",
                details={"error": str(error)},
            ) from error
        return tuple(
            path.replace("\\", "/") for path in result.stdout.split("\0") if path
        )

    @staticmethod
    def _pathspec_chunks(paths: tuple[str, ...]) -> Iterator[tuple[str, ...]]:
        chunk: list[str] = []
        char_count = 0
        for path in paths:
            path_chars = len(path.encode("utf-16-le")) // 2 + 3
            if path_chars > _GIT_PATHSPEC_CHUNK_CHARS:
                raise CoordinatorError(
                    "finalize_pathspec_too_long",
                    "A finalize path exceeds the safe Windows Git command budget",
                    details={"path": path, "utf16Units": path_chars - 3},
                )
            if chunk and char_count + path_chars > _GIT_PATHSPEC_CHUNK_CHARS:
                yield tuple(chunk)
                chunk = []
                char_count = 0
            chunk.append(path)
            char_count += path_chars
        if chunk:
            yield tuple(chunk)

    def _normalize(self, value: str) -> str:
        candidate = (self.repo_root / value).resolve()
        try:
            return candidate.relative_to(self.repo_root).as_posix()
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error

    @staticmethod
    def _format_message(message: str) -> str:
        value = message.strip()
        if "[zircon-session:" in value.casefold():
            raise CoordinatorError(
                "finalize_message_forbidden", "Session-tagged Git messages are forbidden"
            )
        if MODULE_PREFIX.match(value):
            raise CoordinatorError(
                "finalize_message_prefix_forbidden",
                "Git commit subjects must not contain a plan-module prefix",
            )
        if not SEMANTIC_MESSAGE.fullmatch(value):
            raise CoordinatorError(
                "finalize_message_invalid",
                "Finalize message must be a Conventional Commit without a module prefix",
            )
        return value

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
        command = ["git", *arguments]
        try:
            result = subprocess.run(
                command,
                cwd=self.repo_root,
                check=True,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
        except subprocess.CalledProcessError as error:
            stderr = self._safe_git_stderr(error.stderr)
            command_label = " ".join(command[:2])
            path_chunk: list[str] = []
            if "--" in arguments:
                path_separator = arguments.index("--")
                path_chunk = list(arguments[path_separator + 1 :])
            message = f"{command_label} failed with exit code {error.returncode}"
            if stderr:
                message = f"{message}: {stderr}"
            raise CoordinatorError(
                "finalize_git_command_failed",
                message,
                details={
                    "command": command_label,
                    "exit_code": error.returncode,
                    "path_chunk": path_chunk,
                    "stderr": stderr,
                },
            ) from error
        except OSError as error:
            raise CoordinatorError(
                "finalize_git_command_failed",
                "Cannot start Git finalize command",
                details={"error": self._safe_git_stderr(str(error))},
            ) from error
        return result.stdout.strip()

    @staticmethod
    def _safe_git_stderr(value: str | bytes | None) -> str:
        if isinstance(value, bytes):
            text = value.decode("utf-8", errors="replace")
        else:
            text = value or ""
        sanitized = REDACTABLE_SECRET.sub("<redacted>", text.strip())
        return sanitized[:_GIT_FAILURE_STDERR_LIMIT]

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
                SET status = 'failed', error_text = ?, completed_at = ?,
                    index_snapshot = NULL
                WHERE request_id = ? AND status <> 'committed'
                """,
                (error, utc_text(), request_id),
            )
