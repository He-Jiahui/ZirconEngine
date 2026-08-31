"""Immutable compile-gated Git snapshots for non-blocking early integration."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import IntegrityError
from typing import Callable, Iterator
from uuid import uuid4

from .database import Database
from .git_index_lock import (
    IndexLockRecovery,
    IndexLockRecoveryRefused,
    recover_stale_index_lock,
)
from .leases import LeaseService
from .models import CoordinatorError, utc_text
from .notifications import WeComNotificationService


@dataclass(frozen=True, slots=True)
class CandidatePath:
    path: str
    blob_oid: str


@dataclass(frozen=True, slots=True)
class IntegrationCandidate:
    candidate_id: str
    session_id: str
    plan_path: str
    request_id: str
    base_head: str
    compile_ticket_id: str
    status: str
    commit_sha: str | None
    lease_evidence: tuple[dict[str, str | None], ...]
    paths: tuple[CandidatePath, ...]


class IntegrationCandidateService:
    """Seal owned worktree files as Git blobs before the shared HEAD changes.

    Candidate creation has no index or commit side effect.  The later finalizer
    can construct a tree entirely from these blob OIDs, preserving edits made
    after a candidate was submitted.
    """

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        leases: LeaseService,
        *,
        index_lock_recoverer: Callable[[Path], IndexLockRecovery | None] | None = None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.leases = leases
        self.index_lock_recoverer = index_lock_recoverer or recover_stale_index_lock
        self.notifications: WeComNotificationService | None = None

    def set_notifications(self, notifications: WeComNotificationService) -> None:
        self.notifications = notifications

    def submit(
        self,
        *,
        session_id: str,
        request_id: str,
        paths: tuple[str, ...] | list[str],
        compile_ticket_id: str,
    ) -> IntegrationCandidate:
        session_id = self._text("session_id", session_id)
        request_id = self._text("request_id", request_id)
        compile_ticket_id = self._text("compile_ticket_id", compile_ticket_id)
        normalized_paths = self._paths(paths)
        with self.database.connect() as connection:
            existing = connection.execute(
                "SELECT candidate_id FROM integration_candidates WHERE request_id=?", (request_id,)
            ).fetchone()
            if existing is not None:
                return self._get_in_connection(connection, str(existing["candidate_id"]))

            owner = connection.execute(
                "SELECT plan_path FROM sessions WHERE session_id=?", (session_id,)
            ).fetchone()
            if owner is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            plan_path = str(owner["plan_path"] or "")
            if not plan_path:
                raise CoordinatorError(
                    "integration_candidate_plan_missing",
                    "Integration candidate owner must belong to a numbered Plan",
                )
            compile_ticket = connection.execute(
                """SELECT session_id, status, source_manifest_hash, source_manifest_json
                   FROM validation_tickets WHERE ticket_id=?""",
                (compile_ticket_id,),
            ).fetchone()
            if compile_ticket is None:
                raise CoordinatorError(
                    "integration_candidate_compile_ticket_missing",
                    "Integration candidate requires a durable compile ticket",
                )
            if str(compile_ticket["status"]) != "passed":
                raise CoordinatorError(
                    "integration_candidate_compile_pending",
                    "Integration candidate cannot be ready until its compile ticket passed",
                    details={"ticketId": compile_ticket_id, "status": str(compile_ticket["status"])},
                )
            if str(compile_ticket["session_id"]) != session_id:
                raise CoordinatorError(
                    "integration_candidate_compile_ticket_owner_mismatch",
                    "Integration candidate compile ticket must belong to the candidate Session",
                    details={"ticketId": compile_ticket_id},
                )
            compile_manifest_hash = str(compile_ticket["source_manifest_hash"])
            compile_manifest_json = str(compile_ticket["source_manifest_json"])

        self.leases.require_owned_live(
            session_id,
            list(normalized_paths),
            error_code="integration_candidate_lease_missing",
            message="Integration candidate requires live leases for every sealed path",
        )
        lease_evidence = self._lease_evidence(session_id, normalized_paths)
        snapshots = self._compile_bound_snapshots(
            normalized_paths,
            manifest_hash=compile_manifest_hash,
            manifest_json=compile_manifest_json,
        )
        candidate_id = uuid4().hex
        base_head = self._git("rev-parse", "HEAD")
        now = utc_text()
        with self.database.transaction() as connection:
            duplicate = connection.execute(
                "SELECT candidate_id FROM integration_candidates WHERE request_id=?", (request_id,)
            ).fetchone()
            if duplicate is not None:
                return self._get_in_connection(connection, str(duplicate["candidate_id"]))
            connection.execute(
                """
                INSERT INTO integration_candidates(
                    candidate_id, session_id, plan_path, request_id, base_head,
                    compile_ticket_id, status, lease_evidence_json, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, 'integration_ready', ?, ?, ?)
                """,
                (
                    candidate_id,
                    session_id,
                    plan_path,
                    request_id,
                    base_head,
                    compile_ticket_id,
                    json.dumps(lease_evidence, sort_keys=True),
                    now,
                    now,
                ),
            )
            connection.executemany(
                """
                INSERT INTO integration_candidate_paths(candidate_id, path, blob_oid)
                VALUES (?, ?, ?)
                """,
                [(candidate_id, item.path, item.blob_oid) for item in snapshots],
            )
            connection.execute(
                """
                INSERT INTO integration_candidate_events(candidate_id, event_type, payload_json, created_at)
                VALUES (?, 'integration.candidate_submitted', ?, ?)
                """,
                (
                    candidate_id,
                    json.dumps(
                        {
                            "baseHead": base_head,
                            "compileTicketId": compile_ticket_id,
                            "sourceManifestHash": compile_manifest_hash,
                            "leaseEvidence": lease_evidence,
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
            candidate = self._get_in_connection(connection, candidate_id)
        self._notify_submitted(candidate, submitted_at=now)
        return candidate

    def get(self, candidate_id: str) -> IntegrationCandidate:
        with self.database.connect() as connection:
            return self._get_in_connection(connection, self._text("candidate_id", candidate_id))

    def finalize(self, candidate_id: str, *, message: str) -> IntegrationCandidate:
        """Integrate an exact, compile-passed candidate without staging a worktree.

        The resulting commit is built from sealed blob IDs in a temporary index.
        Later edits in the shared worktree therefore remain untouched.  A head
        advance touching one of the candidate paths becomes a durable
        ``delayed_merge`` decision instead of a wait, retry loop, or overwrite.
        If the advance already contains every sealed blob, the candidate is
        accepted as converged instead of being misclassified as a conflict.
        """
        candidate_id = self._text("candidate_id", candidate_id)
        message = self._text("message", message)
        with self._git_mutex(f"candidate:{candidate_id}"):
            candidate = self.get(candidate_id)
            if candidate.status == "integrated_validation_pending":
                current_head = self._git("rev-parse", "HEAD")
                if self._head_contains_candidate(current_head, candidate.paths):
                    self._align_shared_index(candidate)
                return candidate
            if candidate.status in {"accepted", "delayed_merge"}:
                return candidate
            if candidate.status != "integration_ready":
                raise CoordinatorError(
                    "integration_candidate_status_invalid",
                    f"Cannot finalize candidate in {candidate.status} state",
                )
            if self._git("branch", "--show-current") != "main":
                raise CoordinatorError(
                    "integration_candidate_not_on_main",
                    "Early integration candidates may finalize only on main",
                )

            current_head = self._git("rev-parse", "HEAD")
            if candidate.commit_sha and self._prepared_commit_is_current(
                candidate, current_head
            ):
                self._align_shared_index(candidate)
                integrated = self._mark_integrated(
                    candidate, candidate.commit_sha, recovered=True
                )
                self._notify_integrated(candidate, candidate.commit_sha, message)
                return integrated
            if self._head_contains_candidate(current_head, candidate.paths):
                return self._mark_accepted_existing(candidate, current_head)
            if not self._is_ancestor(candidate.base_head, current_head):
                return self._delay_merge(candidate, current_head, "base_not_ancestor")
            conflicts = self._overlapping_paths(candidate, current_head)
            if conflicts:
                return self._delay_merge(candidate, current_head, "path_overlap", conflicts)

            commit_sha = candidate.commit_sha
            if not commit_sha or self._git("rev-parse", f"{commit_sha}^") != current_head:
                tree = self._tree_with_candidate_blobs(current_head, candidate.paths)
                commit_sha = self._git("commit-tree", tree, "-p", current_head, "-m", message)
                self._record_prepared_commit(candidate, commit_sha, current_head)
            try:
                self._git("update-ref", "HEAD", commit_sha, current_head)
            except subprocess.CalledProcessError:
                return self._delay_merge(candidate, self._git("rev-parse", "HEAD"), "head_cas_failed")
            self._align_shared_index(candidate)
            integrated = self._mark_integrated(candidate, commit_sha, recovered=False)
            self._notify_integrated(candidate, commit_sha, message)
            return integrated

    def _compile_bound_snapshots(
        self,
        paths: tuple[str, ...],
        *,
        manifest_hash: str,
        manifest_json: str,
    ) -> tuple[CandidatePath, ...]:
        try:
            manifest = json.loads(manifest_json)
            canonical = json.dumps(
                manifest,
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=True,
                allow_nan=False,
            )
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "integration_candidate_compile_manifest_invalid",
                "Compile ticket source manifest is not canonical JSON",
            ) from error
        if not isinstance(manifest, dict) or not all(
            isinstance(path, str) for path in manifest
        ):
            raise CoordinatorError(
                "integration_candidate_compile_manifest_invalid",
                "Compile ticket source manifest must be a path object",
            )
        canonical_hash = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
        if canonical_hash != manifest_hash:
            raise CoordinatorError(
                "integration_candidate_compile_manifest_invalid",
                "Compile ticket source manifest hash does not match its durable payload",
                details={"expectedManifestHash": manifest_hash, "actualManifestHash": canonical_hash},
            )

        manifest_paths = tuple(sorted(manifest, key=str.casefold))
        if manifest_paths != paths:
            raise CoordinatorError(
                "integration_candidate_compile_manifest_mismatch",
                "Compile ticket source manifest must exactly match candidate paths",
                details={"candidatePaths": list(paths), "manifestPaths": list(manifest_paths)},
            )

        validated: list[tuple[str, bytes]] = []
        for path in paths:
            expected_hash = manifest[path]
            if not isinstance(expected_hash, str) or len(expected_hash) != 64 or any(
                character not in "0123456789abcdef" for character in expected_hash
            ):
                raise CoordinatorError(
                    "integration_candidate_compile_manifest_tombstone",
                    "Integration candidates cannot seal a compile-ticket tombstone",
                    details={"path": path},
                )
            content = self._candidate_bytes(path)
            actual_hash = hashlib.sha256(content).hexdigest()
            if actual_hash != expected_hash:
                raise CoordinatorError(
                    "integration_candidate_compile_snapshot_stale",
                    "Candidate bytes differ from the passed compile ticket snapshot",
                    details={
                        "path": path,
                        "expectedHash": expected_hash,
                        "actualHash": actual_hash,
                    },
                )
            validated.append((path, content))
        return tuple(
            CandidatePath(path, self._write_blob_bytes(path, content))
            for path, content in validated
        )

    def _candidate_bytes(self, path: str) -> bytes:
        source = (self.repo_root / path).resolve()
        if (
            not source.is_file()
            or source.is_symlink()
            or not source.is_relative_to(self.repo_root)
            or source.stat().st_mode & 0o111
        ):
            raise CoordinatorError(
                "integration_candidate_path_invalid",
                "Candidate path must be a non-executable regular repository file",
            )
        return source.read_bytes()

    def _write_blob_bytes(self, path: str, content: bytes) -> str:
        result = subprocess.run(
            ["git", "hash-object", "-w", "--stdin", f"--path={path}"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            input=content,
        )
        return result.stdout.decode("ascii").strip()

    def _lease_evidence(
        self, session_id: str, paths: tuple[str, ...]
    ) -> list[dict[str, str | None]]:
        """Capture the precise live-lease rows that authorized the sealed blobs."""
        normalized = [self.leases.path_policy.normalize(path) for path in paths]
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT path_key, display_path, base_hash, acquired_at, expires_at
                FROM leases WHERE session_id=? ORDER BY path_key
                """,
                (session_id,),
            ).fetchall()
        rows_by_key = {str(row["path_key"]): row for row in rows}
        evidence: list[dict[str, str | None]] = []
        for path in normalized:
            matching = None
            path_parts = path.key.split("/")
            for part_count in range(1, len(path_parts) + 1):
                ancestor_key = "/".join(path_parts[:part_count])
                matching = rows_by_key.get(ancestor_key)
                if matching is not None:
                    break
            if matching is None:
                raise CoordinatorError(
                    "integration_candidate_lease_missing",
                    "Live lease disappeared before candidate evidence was sealed",
                    details={"paths": [path.display]},
                )
            evidence.append(
                {
                    "candidatePath": path.display,
                    "leasePath": str(matching["display_path"]),
                    "baseHash": str(matching["base_hash"]) if matching["base_hash"] else None,
                    "acquiredAt": str(matching["acquired_at"]),
                    "expiresAt": str(matching["expires_at"]),
                }
            )
        return evidence

    def _tree_with_candidate_blobs(
        self, current_head: str, paths: tuple[CandidatePath, ...]
    ) -> str:
        git_dir = Path(self._git("rev-parse", "--git-dir"))
        if not git_dir.is_absolute():
            git_dir = (self.repo_root / git_dir).resolve()
        descriptor, raw_index = tempfile.mkstemp(prefix="candidate-index-", dir=git_dir)
        os.close(descriptor)
        index_path = Path(raw_index)
        index_path.unlink(missing_ok=True)
        environment = {**os.environ, "GIT_INDEX_FILE": str(index_path)}
        try:
            self._git("read-tree", current_head, environment=environment)
            self._update_index_entries(paths, environment=environment)
            return self._git("write-tree", environment=environment)
        finally:
            index_path.unlink(missing_ok=True)
            index_path.with_name(index_path.name + ".lock").unlink(missing_ok=True)

    def _update_index_entries(
        self,
        paths: tuple[CandidatePath, ...],
        *,
        environment: dict[str, str] | None = None,
    ) -> None:
        payload = b"".join(
            f"100644 blob {item.blob_oid}\t".encode("ascii")
            + os.fsencode(item.path)
            + b"\0"
            for item in paths
        )
        subprocess.run(
            ["git", "update-index", "-z", "--index-info"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            input=payload,
            env=environment,
        )

    def _is_ancestor(self, base_head: str, current_head: str) -> bool:
        result = subprocess.run(
            ["git", "merge-base", "--is-ancestor", base_head, current_head],
            cwd=self.repo_root,
            capture_output=True,
            text=True,
        )
        if result.returncode in {0, 1}:
            return result.returncode == 0
        raise CoordinatorError(
            "integration_candidate_git_failed",
            "Git could not compare the candidate base to main",
            details={"stderr": result.stderr.strip()},
        )

    def _overlapping_paths(
        self, candidate: IntegrationCandidate, current_head: str
    ) -> tuple[str, ...]:
        if candidate.base_head == current_head:
            return ()
        changed = {
            line.strip()
            for line in self._git(
                "diff", "--name-only", "--no-renames", candidate.base_head, current_head, "--"
            ).splitlines()
            if line.strip()
        }
        return tuple(
            item.path for item in candidate.paths if item.path in changed
        )

    def _head_contains_candidate(
        self, current_head: str, paths: tuple[CandidatePath, ...]
    ) -> bool:
        for item in paths:
            result = subprocess.run(
                ["git", "rev-parse", f"{current_head}:{item.path}"],
                cwd=self.repo_root,
                capture_output=True,
                text=True,
            )
            if result.returncode != 0 or result.stdout.strip() != item.blob_oid:
                return False
        return True

    def _prepared_commit_is_current(
        self, candidate: IntegrationCandidate, current_head: str
    ) -> bool:
        assert candidate.commit_sha is not None
        return current_head == candidate.commit_sha or (
            self._is_ancestor(candidate.commit_sha, current_head)
            and self._head_contains_candidate(current_head, candidate.paths)
        )

    def _align_shared_index(self, candidate: IntegrationCandidate) -> None:
        """Advance only committed candidate entries in the shared index."""
        self._recover_index_lock(candidate)
        self._update_index_entries(candidate.paths)

    def _recover_index_lock(self, candidate: IntegrationCandidate) -> None:
        index_path = Path(self._git("rev-parse", "--git-path", "index"))
        if not index_path.is_absolute():
            index_path = self.repo_root / index_path
        lock_path = index_path.with_name(index_path.name + ".lock")
        try:
            recovery = self.index_lock_recoverer(lock_path)
        except IndexLockRecoveryRefused as error:
            details: dict[str, object] = {
                "candidate_id": candidate.candidate_id,
                "lock_path": self._display_git_path(lock_path),
                "reason": error.reason,
                **error.details,
            }
            if error.active_pids:
                details["active_git_pids"] = list(error.active_pids)
            raise CoordinatorError(
                "integration_candidate_index_lock_recovery_refused",
                "Git index lock cannot be recovered safely for integration candidate",
                details=details,
            ) from error
        if recovery is None:
            return
        payload: dict[str, object] = {
            "candidate_id": candidate.candidate_id,
            "lock_path": self._display_git_path(lock_path),
            **recovery.to_event_payload(),
        }
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO events(session_id, event_type, payload_json, created_at)
                VALUES (?, 'git.index_lock_recovered', ?, ?)
                """,
                (
                    candidate.session_id,
                    json.dumps(payload, sort_keys=True),
                    utc_text(),
                ),
            )

    def _display_git_path(self, path: Path) -> str:
        try:
            return path.resolve().relative_to(self.repo_root).as_posix()
        except ValueError:
            return str(path.resolve())

    def _record_prepared_commit(
        self, candidate: IntegrationCandidate, commit_sha: str, parent: str
    ) -> None:
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE integration_candidates SET commit_sha=?, updated_at=? WHERE candidate_id=?",
                (commit_sha, now, candidate.candidate_id),
            )
            connection.execute(
                """
                INSERT INTO integration_candidate_events(candidate_id, event_type, payload_json, created_at)
                VALUES (?, 'integration.finalize_prepared', ?, ?)
                """,
                (candidate.candidate_id, json.dumps({"commitSha": commit_sha, "parent": parent}, sort_keys=True), now),
            )

    def _mark_integrated(
        self, candidate: IntegrationCandidate, commit_sha: str, *, recovered: bool
    ) -> IntegrationCandidate:
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE integration_candidates
                SET status='integrated_validation_pending', commit_sha=?, updated_at=?
                WHERE candidate_id=?
                """,
                (commit_sha, now, candidate.candidate_id),
            )
            connection.execute(
                """
                INSERT INTO integration_candidate_events(candidate_id, event_type, payload_json, created_at)
                VALUES (?, 'integration.finalized', ?, ?)
                """,
                (candidate.candidate_id, json.dumps({"commitSha": commit_sha, "recovered": recovered}, sort_keys=True), now),
            )
            return self._get_in_connection(connection, candidate.candidate_id)

    def _mark_accepted_existing(
        self, candidate: IntegrationCandidate, current_head: str
    ) -> IntegrationCandidate:
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE integration_candidates
                SET status='accepted', commit_sha=?, updated_at=?
                WHERE candidate_id=?
                """,
                (current_head, now, candidate.candidate_id),
            )
            connection.execute(
                """
                INSERT INTO integration_candidate_events(candidate_id, event_type, payload_json, created_at)
                VALUES (?, 'integration.accepted_existing', ?, ?)
                """,
                (
                    candidate.candidate_id,
                    json.dumps({"commitSha": current_head}, sort_keys=True),
                    now,
                ),
            )
            return self._get_in_connection(connection, candidate.candidate_id)

    def _notify_integrated(
        self, candidate: IntegrationCandidate, commit_sha: str, message: str
    ) -> None:
        if self.notifications is None:
            return
        try:
            formatted = self.notifications.format_message(
                module=self._notification_module(candidate),
                summary=f"scoped candidate {candidate.candidate_id[:8]}: {message}",
                commit_time=self._git("show", "-s", "--format=%cI", commit_sha),
                shortstat=(
                    self._git("show", "--shortstat", "--format=", commit_sha).strip()
                    or "0 files changed"
                ),
                commit_content=(
                    f"{commit_sha} {self._git('show', '-s', '--format=%s', commit_sha)}"
                ),
            )
            self.notifications.notify_once(commit_sha=commit_sha, message=formatted)
        except Exception as error:
            self.notifications.record_post_commit_failure(
                commit_sha=commit_sha,
                error=error,
            )

    def _notify_submitted(
        self, candidate: IntegrationCandidate, *, submitted_at: str
    ) -> None:
        if self.notifications is None:
            return
        notification_key = f"candidate:{candidate.candidate_id}"
        try:
            formatted = self.notifications.format_candidate_submission(
                module=self._notification_module(candidate),
                candidate_id=candidate.candidate_id,
                base_head=candidate.base_head,
                submitted_at=submitted_at,
                sealed_path_count=len(candidate.paths),
            )
            self.notifications.notify_once(
                commit_sha=notification_key,
                message=formatted,
            )
        except Exception as error:
            self.notifications.record_preparation_failure(
                notification_key=notification_key,
                context="candidate-submission",
                error=error,
            )

    @staticmethod
    def _notification_module(candidate: IntegrationCandidate) -> str:
        parts = [
            part
            for part in candidate.plan_path.replace("\\", "/").rstrip("/").split("/")
            if part
        ]
        if len(parts) < 2:
            raise CoordinatorError(
                "notification_module_unavailable",
                "Integration notification requires a plan path inside a module folder",
            )
        return parts[-2]

    def _delay_merge(
        self,
        candidate: IntegrationCandidate,
        current_head: str,
        reason: str,
        conflicts: tuple[str, ...] = (),
    ) -> IntegrationCandidate:
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE integration_candidates SET status='delayed_merge', updated_at=? WHERE candidate_id=?",
                (now, candidate.candidate_id),
            )
            connection.execute(
                """
                INSERT INTO integration_candidate_events(candidate_id, event_type, payload_json, created_at)
                VALUES (?, 'integration.delayed_merge', ?, ?)
                """,
                (
                    candidate.candidate_id,
                    json.dumps(
                        {"currentHead": current_head, "reason": reason, "conflicts": list(conflicts)},
                        sort_keys=True,
                    ),
                    now,
                ),
            )
            return self._get_in_connection(connection, candidate.candidate_id)

    @contextmanager
    def _git_mutex(self, owner_id: str) -> Iterator[None]:
        try:
            with self.database.transaction() as connection:
                connection.execute(
                    "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, ?)",
                    (owner_id, utc_text()),
                )
        except IntegrityError as error:
            raise CoordinatorError(
                "git_mutex_occupied",
                "Another finalize operation owns the Git index mutex",
            ) from error
        try:
            yield
        finally:
            with self.database.transaction() as connection:
                connection.execute(
                    "DELETE FROM git_mutex WHERE lock_name='index' AND owner_id=?", (owner_id,)
                )

    def _paths(self, values: tuple[str, ...] | list[str]) -> tuple[str, ...]:
        if not isinstance(values, (tuple, list)) or not values:
            raise CoordinatorError("integration_candidate_paths_empty", "Candidate requires at least one path")
        normalized = {self._path(value) for value in values}
        return tuple(sorted(normalized, key=str.casefold))

    def _path(self, value: object) -> str:
        if not isinstance(value, str) or not value.strip():
            raise CoordinatorError("integration_candidate_path_invalid", "Candidate path must be non-empty")
        raw = value.strip().replace("\\", "/")
        candidate = (self.repo_root / raw).resolve()
        if not candidate.is_relative_to(self.repo_root) or any(part in {"", ".", ".."} for part in raw.split("/")):
            raise CoordinatorError("integration_candidate_path_invalid", "Candidate path must be safe and repo-relative")
        return candidate.relative_to(self.repo_root).as_posix()

    @staticmethod
    def _text(field: str, value: object) -> str:
        if not isinstance(value, str) or not value.strip():
            raise CoordinatorError("integration_candidate_input_invalid", f"{field} must be non-empty text")
        return value.strip()

    def _get_in_connection(self, connection, candidate_id: str) -> IntegrationCandidate:
        row = connection.execute(
            "SELECT * FROM integration_candidates WHERE candidate_id=?", (candidate_id,)
        ).fetchone()
        if row is None:
            raise CoordinatorError("integration_candidate_not_found", f"Unknown candidate {candidate_id}")
        path_rows = connection.execute(
            "SELECT path, blob_oid FROM integration_candidate_paths WHERE candidate_id=? ORDER BY path",
            (candidate_id,),
        ).fetchall()
        return IntegrationCandidate(
            candidate_id=str(row["candidate_id"]),
            session_id=str(row["session_id"]),
            plan_path=str(row["plan_path"]),
            request_id=str(row["request_id"]),
            base_head=str(row["base_head"]),
            compile_ticket_id=str(row["compile_ticket_id"]),
            status=str(row["status"]),
            commit_sha=str(row["commit_sha"]) if row["commit_sha"] is not None else None,
            lease_evidence=tuple(json.loads(str(row["lease_evidence_json"]))),
            paths=tuple(CandidatePath(str(item["path"]), str(item["blob_oid"])) for item in path_rows),
        )

    def _git(self, *arguments: str, environment: dict[str, str] | None = None) -> str:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
            env=environment,
        )
        return result.stdout.strip()
