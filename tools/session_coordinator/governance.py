from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from types import MappingProxyType
from typing import Mapping
from uuid import uuid4

from .database import Database
from .models import CoordinatorError
from .models import utc_text


_STALE_CANDIDATE_STATUSES = (
    "registered",
    "active",
    "waiting_lease",
    "resolving_failure",
    "waiting_validation",
)
_EXECUTABLE_SESSION_STATUSES = _STALE_CANDIDATE_STATUSES + ("finalizing",)


@dataclass(frozen=True, slots=True)
class GovernanceCandidate:
    """One resource transition guarded by its observed preconditions."""

    kind: str
    identity: str
    action: str
    reason: str
    expected: Mapping[str, object]

    def __post_init__(self) -> None:
        for field_name, value in (
            ("kind", self.kind),
            ("identity", self.identity),
            ("action", self.action),
            ("reason", self.reason),
        ):
            if not value.strip():
                raise ValueError(f"Governance candidate {field_name} cannot be empty")
        normalized = {
            str(key): value
            for key, value in sorted(self.expected.items(), key=lambda item: str(item[0]).casefold())
        }
        try:
            json.dumps(normalized, sort_keys=True, separators=(",", ":"))
        except (TypeError, ValueError) as error:
            raise ValueError("Governance candidate conditions must be JSON serializable") from error
        object.__setattr__(self, "expected", MappingProxyType(normalized))

    def payload(self) -> dict[str, object]:
        return {
            "kind": self.kind,
            "identity": self.identity,
            "action": self.action,
            "reason": self.reason,
            "expected": dict(self.expected),
        }


@dataclass(frozen=True, slots=True)
class GovernancePreview:
    """Canonical, immutable preview that must match before a governed apply."""

    operation: str
    candidates: tuple[GovernanceCandidate, ...]
    fingerprint: str

    @classmethod
    def create(
        cls, operation: str, candidates: tuple[GovernanceCandidate, ...]
    ) -> "GovernancePreview":
        if not operation.strip():
            raise ValueError("Governance operation cannot be empty")
        ordered = tuple(
            sorted(
                candidates,
                key=lambda candidate: (
                    candidate.kind.casefold(),
                    candidate.identity.casefold(),
                    candidate.action.casefold(),
                ),
            )
        )
        identities = {
            (candidate.kind.casefold(), candidate.identity.casefold())
            for candidate in ordered
        }
        if len(identities) != len(ordered):
            raise ValueError("Governance preview cannot contain duplicate resource identities")
        payload = {
            "operation": operation,
            "candidates": [candidate.payload() for candidate in ordered],
        }
        serialized = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
        return cls(
            operation=operation,
            candidates=ordered,
            fingerprint=hashlib.sha256(serialized.encode("utf-8")).hexdigest(),
        )

    def require_fingerprint(self, fingerprint: str) -> None:
        if fingerprint != self.fingerprint:
            raise CoordinatorError(
                "governance_preview_stale",
                "Governance candidates changed after preview",
                details={"operation": self.operation, "expectedFingerprint": self.fingerprint},
            )


@dataclass(frozen=True, slots=True)
class GovernanceApplyResult:
    """The durable outcome of applying one immutable preview."""

    operation: str
    fingerprint: str
    applied: tuple[str, ...]
    skipped: tuple[str, ...]
    conflicts: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "operation": self.operation,
            "fingerprint": self.fingerprint,
            "applied": list(self.applied),
            "skipped": list(self.skipped),
            "conflicts": list(self.conflicts),
        }


class StateConvergenceService:
    """Build a read-only, resource-protected convergence preview.

    Applying the preview is intentionally separate from selection: the caller
    must later re-check every `expected` value inside one governed CAS
    transaction.  This keeps discovery from holding a SQLite write lock while
    Windows process-tree inspection or filesystem work is in progress.
    """

    def __init__(self, database: Database, repo_root: str | Path | None = None):
        self.database = database
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else None

    def preview(
        self,
        *,
        now: datetime | None = None,
        stale_after_seconds: int = 24 * 60 * 60,
        archive_after_seconds: int = 24 * 60 * 60,
    ) -> GovernancePreview:
        if stale_after_seconds <= 0 or archive_after_seconds <= 0:
            raise ValueError("Governance retention windows must be positive")
        current_time = now or datetime.now(UTC)
        now_text = utc_text(current_time)
        stale_cutoff = utc_text(current_time - timedelta(seconds=stale_after_seconds))
        archive_cutoff = utc_text(current_time - timedelta(seconds=archive_after_seconds))
        with self.database.connect() as connection:
            candidates = [
                *self._stale_session_candidates(connection, stale_cutoff, now_text),
                *self._archive_session_candidates(connection, archive_cutoff, now_text),
                *self._terminal_run_candidates(connection),
                *self._expired_reservation_candidates(connection, now_text),
                *self._session_note_candidates(connection),
            ]
        return GovernancePreview.create("converge", tuple(candidates))

    def apply(
        self,
        preview: GovernancePreview,
        *,
        fingerprint: str,
        actor: str,
        now: datetime | None = None,
    ) -> GovernanceApplyResult:
        """Apply exactly one preview without holding a database lock for file I/O.

        Every database row is rechecked in the transaction that changes it. A
        newer heartbeat, lease, patch, process or note edit makes only that
        candidate a conflict; it cannot cause a broad retry or a blind state
        transition.
        """

        preview.require_fingerprint(fingerprint)
        if preview.operation != "converge":
            raise CoordinatorError(
                "governance_operation_unsupported",
                f"Unsupported governance operation: {preview.operation}",
            )
        if not actor.strip():
            raise ValueError("Governance actor cannot be empty")

        current_time = utc_text(now or datetime.now(UTC))
        # The server records previews before apply, but direct service callers
        # must retain the same immutable audit boundary.  This insert is
        # idempotent, so it cannot replace a preview observed by another
        # caller or make a retry depend on event ordering.
        self.record_preview(preview, actor=actor, now=now)
        applied: list[str] = []
        skipped: list[str] = []
        conflicts: list[str] = []
        note_candidates: list[GovernanceCandidate] = []

        with self.database.transaction() as connection:
            for candidate in preview.candidates:
                label = self._candidate_label(candidate)
                if candidate.kind == "session_note":
                    note_candidates.append(candidate)
                    continue
                if candidate.kind == "session" and candidate.action == "mark_stale":
                    changed = self._apply_mark_stale(connection, candidate, current_time)
                elif candidate.kind == "session" and candidate.action == "archive":
                    changed = self._apply_archive_session(connection, candidate, current_time)
                elif candidate.kind == "cargo_run" and candidate.action == "complete":
                    changed = self._apply_terminal_run(connection, candidate, current_time)
                elif candidate.kind == "reservation" and candidate.action == "expire":
                    changed = self._apply_expired_reservation(connection, candidate, current_time)
                else:
                    skipped.append(f"{label}:unsupported")
                    continue
                if changed:
                    applied.append(label)
                else:
                    conflicts.append(label)

        for candidate in note_candidates:
            label = self._candidate_label(candidate)
            if candidate.action == "report_unowned":
                skipped.append(f"{label}:unowned")
            elif candidate.action == "archive":
                if self._archive_session_note(candidate, current_time):
                    applied.append(label)
                else:
                    conflicts.append(label)
            else:
                skipped.append(f"{label}:unsupported")

        result = GovernanceApplyResult(
            operation=preview.operation,
            fingerprint=preview.fingerprint,
            applied=tuple(applied),
            skipped=tuple(skipped),
            conflicts=tuple(conflicts),
        )
        self._record_apply(actor=actor, result=result, created_at=current_time)
        return result

    def record_preview(
        self, preview: GovernancePreview, *, actor: str, now: datetime | None = None
    ) -> GovernancePreview:
        """Durably retain a preview so apply never trusts caller-provided rows."""

        if not actor.strip():
            raise ValueError("Governance actor cannot be empty")
        created_at = utc_text(now or datetime.now(UTC))
        with self.database.transaction() as connection:
            if self._find_recorded_preview(connection, preview.fingerprint) is None:
                candidates = [candidate.payload() for candidate in preview.candidates]
                connection.execute(
                    """
                    INSERT INTO governance_previews(
                        fingerprint, operation, candidate_count, candidates_json, actor, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    (
                        preview.fingerprint,
                        preview.operation,
                        len(candidates),
                        json.dumps(candidates, sort_keys=True, separators=(",", ":")),
                        actor,
                        created_at,
                    ),
                )
                self._event(
                    connection,
                    None,
                    "governance.converge_preview",
                    {
                        "actor": actor,
                        "fingerprint": preview.fingerprint,
                        "operation": preview.operation,
                        "candidateCount": len(candidates),
                    },
                    created_at,
                )
        return preview

    def load_preview(self, fingerprint: str) -> GovernancePreview:
        with self.database.connect() as connection:
            recorded = self._find_recorded_preview(connection, fingerprint)
        if recorded is None:
            raise CoordinatorError(
                "governance_preview_not_found",
                "Governance preview was not recorded by this coordinator",
                details={"fingerprint": fingerprint},
            )
        candidates = tuple(
            GovernanceCandidate(
                kind=str(candidate["kind"]),
                identity=str(candidate["identity"]),
                action=str(candidate["action"]),
                reason=str(candidate["reason"]),
                expected=dict(candidate["expected"]),
            )
            for candidate in recorded["candidates"]
        )
        preview = GovernancePreview.create(str(recorded["operation"]), candidates)
        preview.require_fingerprint(fingerprint)
        return preview

    def _stale_session_candidates(self, connection, cutoff: str, now: str) -> list[GovernanceCandidate]:
        rows = connection.execute(
            """
            SELECT session_id, status, last_heartbeat_at
            FROM sessions
            WHERE status IN (?, ?, ?, ?, ?)
              AND last_heartbeat_at < ?
              AND NOT EXISTS (
                  SELECT 1 FROM leases
                  WHERE leases.session_id=sessions.session_id AND leases.expires_at > ?
              )
              AND NOT EXISTS (
                  SELECT 1 FROM patches
                  WHERE patches.session_id=sessions.session_id
                    AND patches.status IN ('queued', 'applying', 'needs_rebase')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_jobs
                  WHERE cargo_jobs.session_id=sessions.session_id
                    AND cargo_jobs.status IN ('leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_lane_reservations
                  WHERE cargo_lane_reservations.session_id=sessions.session_id
                    AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM validation_copies
                  WHERE validation_copies.session_id=sessions.session_id
                    AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
              )
            ORDER BY session_id
            """,
            (*_STALE_CANDIDATE_STATUSES, cutoff, now),
        ).fetchall()
        return [
            GovernanceCandidate(
                kind="session",
                identity=str(row["session_id"]),
                action="mark_stale",
                reason="heartbeat_expired",
                expected={
                    "status": str(row["status"]),
                    "heartbeat": str(row["last_heartbeat_at"]),
                },
            )
            for row in rows
        ]

    def _archive_session_candidates(
        self, connection, cutoff: str, now: str
    ) -> list[GovernanceCandidate]:
        rows = connection.execute(
            """
            SELECT session_id, updated_at
            FROM sessions
            WHERE status='stale' AND updated_at < ?
              AND NOT EXISTS (
                  SELECT 1 FROM leases
                  WHERE leases.session_id=sessions.session_id AND leases.expires_at > ?
              )
              AND NOT EXISTS (
                  SELECT 1 FROM patches
                  WHERE patches.session_id=sessions.session_id
                    AND patches.status IN ('queued', 'applying', 'needs_rebase')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_jobs
                  WHERE cargo_jobs.session_id=sessions.session_id
                    AND cargo_jobs.status IN ('leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_lane_reservations
                  WHERE cargo_lane_reservations.session_id=sessions.session_id
                    AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM validation_copies
                  WHERE validation_copies.session_id=sessions.session_id
                    AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
              )
            ORDER BY session_id
            """,
            (cutoff, now),
        ).fetchall()
        return [
            GovernanceCandidate(
                kind="session",
                identity=str(row["session_id"]),
                action="archive",
                reason="stale_retention_elapsed",
                expected={"status": "stale", "updatedAt": str(row["updated_at"])},
            )
            for row in rows
        ]

    def _terminal_run_candidates(self, connection) -> list[GovernanceCandidate]:
        rows = connection.execute(
            """
            SELECT run.run_id, job.session_id, job.status AS job_status, job.exit_code,
                   job.process_tree_live_pids_json
            FROM cargo_job_runs AS run
            JOIN cargo_jobs AS job ON job.job_id=run.job_id
            WHERE run.status='running'
              AND job.process_tree_live_pids_json='[]'
              AND (
                  job.status='orphaned'
                  OR (job.status='released' AND job.exit_code IS NULL)
              )
            ORDER BY run.started_at, run.run_id
            """
        ).fetchall()
        candidates: list[GovernanceCandidate] = []
        for row in rows:
            job_status = str(row["job_status"])
            candidates.append(
                GovernanceCandidate(
                    kind="cargo_run",
                    identity=str(row["run_id"]),
                    action="complete",
                    reason=(
                        "terminal_orphaned_job"
                        if job_status == "orphaned"
                        else "released_job_missing_exit_code"
                    ),
                    expected={
                        "status": "running",
                        "jobStatus": job_status,
                        "sessionId": str(row["session_id"]),
                        "livePids": str(row["process_tree_live_pids_json"]),
                        "exitCode": row["exit_code"],
                    },
                )
            )
        return candidates

    def _expired_reservation_candidates(self, connection, now: str) -> list[GovernanceCandidate]:
        rows = connection.execute(
            """
            SELECT reservation.reservation_id, reservation.session_id, reservation.lane_scope,
                   reservation.expires_at, owner.status AS owner_status
            FROM cargo_lane_reservations AS reservation
            LEFT JOIN sessions AS owner ON owner.session_id=reservation.session_id
            WHERE reservation.status='pending' AND reservation.job_id IS NULL
              AND (
                  owner.status IS NULL
                  OR owner.status NOT IN (
                      'registered', 'active', 'waiting_lease', 'resolving_failure',
                      'waiting_validation', 'finalizing'
                  )
                  OR reservation.expires_at <= ?
              )
            ORDER BY reservation.created_at, reservation.reservation_id
            """,
            (now,),
        ).fetchall()
        candidates: list[GovernanceCandidate] = []
        for row in rows:
            owner_status = row["owner_status"]
            candidates.append(
                GovernanceCandidate(
                    kind="reservation",
                    identity=str(row["reservation_id"]),
                    action="expire",
                    reason=(
                        "owner_not_executable"
                        if owner_status is None
                        or str(owner_status)
                        not in _EXECUTABLE_SESSION_STATUSES
                        else "absolute_ttl_elapsed"
                    ),
                    expected={
                        "status": "pending",
                        "sessionId": str(row["session_id"]),
                        "jobId": None,
                        "ownerStatus": owner_status,
                        "expiresAt": str(row["expires_at"]),
                        "laneScope": str(row["lane_scope"]),
                    },
                )
            )
        return candidates

    def _session_note_candidates(self, connection) -> list[GovernanceCandidate]:
        if self.repo_root is None:
            return []
        sessions_root = self.repo_root / ".codex" / "sessions"
        if not sessions_root.is_dir():
            return []

        candidates: list[GovernanceCandidate] = []
        for note_path in sorted(sessions_root.glob("*.md"), key=lambda path: path.name.casefold()):
            try:
                content = note_path.read_text(encoding="utf-8")
            except OSError:
                continue
            metadata = self._frontmatter(content)
            session_id = metadata.get("session")
            relative_path = note_path.relative_to(self.repo_root).as_posix()
            expected = {
                "contentSha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
                "sessionId": session_id,
            }
            if not session_id:
                candidates.append(
                    GovernanceCandidate(
                        kind="session_note",
                        identity=relative_path,
                        action="report_unowned",
                        reason="session_metadata_missing",
                        expected=expected,
                    )
                )
                continue
            owner = connection.execute(
                "SELECT status FROM sessions WHERE session_id=?", (session_id,)
            ).fetchone()
            if owner is None:
                candidates.append(
                    GovernanceCandidate(
                        kind="session_note",
                        identity=relative_path,
                        action="report_unowned",
                        reason="session_not_registered",
                        expected=expected,
                    )
                )
                continue
            owner_status = str(owner["status"])
            if owner_status not in {"completed", "archived", "cancelled"}:
                continue
            candidates.append(
                GovernanceCandidate(
                    kind="session_note",
                    identity=relative_path,
                    action="archive",
                    reason="owner_terminal",
                    expected={**expected, "ownerStatus": owner_status},
                )
            )
        return candidates

    def _apply_mark_stale(self, connection, candidate: GovernanceCandidate, now: str) -> bool:
        expected = candidate.expected
        changed = connection.execute(
            """
            UPDATE sessions
            SET status='stale', status_reason='heartbeat expired', updated_at=?
            WHERE session_id=? AND status=? AND last_heartbeat_at=?
              AND NOT EXISTS (
                  SELECT 1 FROM leases
                  WHERE leases.session_id=sessions.session_id AND leases.expires_at > ?
              )
              AND NOT EXISTS (
                  SELECT 1 FROM patches
                  WHERE patches.session_id=sessions.session_id
                    AND patches.status IN ('queued', 'applying', 'needs_rebase')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_jobs
                  WHERE cargo_jobs.session_id=sessions.session_id
                    AND cargo_jobs.status IN ('leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_lane_reservations
                  WHERE cargo_lane_reservations.session_id=sessions.session_id
                    AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM validation_copies
                  WHERE validation_copies.session_id=sessions.session_id
                    AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
              )
            """,
            (now, candidate.identity, expected["status"], expected["heartbeat"], now),
        ).rowcount
        if changed:
            self._event(
                connection,
                candidate.identity,
                "session.status_changed",
                {"from": expected["status"], "to": "stale", "reason": candidate.reason},
                now,
            )
        return changed == 1

    def _apply_archive_session(self, connection, candidate: GovernanceCandidate, now: str) -> bool:
        expected = candidate.expected
        changed = connection.execute(
            """
            UPDATE sessions
            SET status='archived', status_reason='stale retention elapsed',
                updated_at=?, archived_at=?
            WHERE session_id=? AND status=? AND updated_at=?
              AND NOT EXISTS (
                  SELECT 1 FROM leases
                  WHERE leases.session_id=sessions.session_id AND leases.expires_at > ?
              )
              AND NOT EXISTS (
                  SELECT 1 FROM patches
                  WHERE patches.session_id=sessions.session_id
                    AND patches.status IN ('queued', 'applying', 'needs_rebase')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_jobs
                  WHERE cargo_jobs.session_id=sessions.session_id
                    AND cargo_jobs.status IN ('leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM cargo_lane_reservations
                  WHERE cargo_lane_reservations.session_id=sessions.session_id
                    AND cargo_lane_reservations.status IN ('pending', 'leased', 'running')
              )
              AND NOT EXISTS (
                  SELECT 1 FROM validation_copies
                  WHERE validation_copies.session_id=sessions.session_id
                    AND validation_copies.status IN ('planned', 'materialized', 'running', 'cleanup_pending')
              )
            """,
            (now, now, candidate.identity, expected["status"], expected["updatedAt"], now),
        ).rowcount
        if changed:
            self._event(
                connection,
                candidate.identity,
                "session.status_changed",
                {"from": "stale", "to": "archived", "reason": candidate.reason},
                now,
            )
        return changed == 1

    def _apply_terminal_run(self, connection, candidate: GovernanceCandidate, now: str) -> bool:
        expected = candidate.expected
        error_code = (
            "cargo_run_reconciled_from_orphaned_job"
            if expected["jobStatus"] == "orphaned"
            else "cargo_run_reconciled_from_released_job_missing_exit_code"
        )
        changed = connection.execute(
            """
            UPDATE cargo_job_runs
            SET status='completed', error_code=?, completed_at=?
            WHERE run_id=? AND status=?
              AND EXISTS (
                  SELECT 1 FROM cargo_jobs
                  WHERE cargo_jobs.job_id=cargo_job_runs.job_id
                    AND cargo_jobs.session_id=? AND cargo_jobs.status=?
                    AND cargo_jobs.process_tree_live_pids_json=?
                    AND cargo_jobs.exit_code IS ?
              )
            """,
            (
                error_code,
                now,
                candidate.identity,
                expected["status"],
                expected["sessionId"],
                expected["jobStatus"],
                expected["livePids"],
                expected["exitCode"],
            ),
        ).rowcount
        if changed:
            self._event(
                connection,
                str(expected["sessionId"]),
                "cargo.run_reconciled",
                {"runId": candidate.identity, "reason": candidate.reason},
                now,
            )
        return changed == 1

    def _apply_expired_reservation(self, connection, candidate: GovernanceCandidate, now: str) -> bool:
        expected = candidate.expected
        owner_status = expected["ownerStatus"]
        owner_guard = (
            "NOT EXISTS (SELECT 1 FROM sessions WHERE session_id=cargo_lane_reservations.session_id)"
            if owner_status is None
            else "(SELECT status FROM sessions WHERE session_id=cargo_lane_reservations.session_id) IS ?"
        )
        parameters: list[object] = [
            now,
            candidate.identity,
            expected["status"],
            expected["sessionId"],
            expected["laneScope"],
            expected["expiresAt"],
        ]
        if owner_status is not None:
            parameters.append(owner_status)
        changed = connection.execute(
            f"""
            UPDATE cargo_lane_reservations
            SET status='expired', completed_at=COALESCE(completed_at, ?)
            WHERE reservation_id=? AND status=? AND session_id=? AND lane_scope=?
              AND job_id IS NULL AND expires_at=?
              AND {owner_guard}
            """,
            parameters,
        ).rowcount
        if changed:
            self._event(
                connection,
                str(expected["sessionId"]),
                "cargo.reservation_reconciled",
                {
                    "reservationId": candidate.identity,
                    "reason": candidate.reason,
                    "status": "expired",
                },
                now,
            )
        return changed == 1

    def _archive_session_note(self, candidate: GovernanceCandidate, now: str) -> bool:
        if self.repo_root is None:
            return False
        source = (self.repo_root / candidate.identity).resolve()
        sessions_root = (self.repo_root / ".codex" / "sessions").resolve()
        try:
            source.relative_to(sessions_root)
        except ValueError:
            return False
        if source.parent != sessions_root or source.suffix != ".md":
            return False
        try:
            content = source.read_text(encoding="utf-8")
        except OSError:
            return False
        expected_hash = candidate.expected["contentSha256"]
        if hashlib.sha256(content.encode("utf-8")).hexdigest() != expected_hash:
            return False

        archive_root = sessions_root / "archive"
        destination = archive_root / source.name
        if destination.exists():
            return False
        archived_content = self._archived_note_content(content, now)
        temporary = source.with_name(f".{source.name}.{uuid4().hex}.governance")
        try:
            temporary.write_text(archived_content, encoding="utf-8")
            temporary.replace(source)
            archive_root.mkdir(parents=True, exist_ok=True)
            source.replace(destination)
        except OSError:
            try:
                if source.exists():
                    source.write_text(content, encoding="utf-8")
            except OSError:
                pass
            return False
        finally:
            try:
                temporary.unlink(missing_ok=True)
            except OSError:
                pass
        return True

    def _record_apply(
        self, *, actor: str, result: GovernanceApplyResult, created_at: str
    ) -> None:
        with self.database.transaction() as connection:
            candidate_count = len(result.applied) + len(result.skipped) + len(result.conflicts)
            connection.execute(
                """
                INSERT INTO governance_applies(
                    fingerprint, actor, candidate_count, applied_count, skipped_count,
                    conflict_count, result_json, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    result.fingerprint,
                    actor,
                    candidate_count,
                    len(result.applied),
                    len(result.skipped),
                    len(result.conflicts),
                    json.dumps(result.to_dict(), sort_keys=True, separators=(",", ":")),
                    created_at,
                ),
            )
            self._event(
                connection,
                None,
                "governance.converge_applied",
                {
                    "actor": actor,
                    "fingerprint": result.fingerprint,
                    "candidateCount": candidate_count,
                    "appliedCount": len(result.applied),
                    "skippedCount": len(result.skipped),
                    "conflictCount": len(result.conflicts),
                    "result": result.to_dict(),
                },
                created_at,
            )

    @staticmethod
    def _find_recorded_preview(connection, fingerprint: str) -> dict[str, object] | None:
        row = connection.execute(
            """
            SELECT operation, candidates_json FROM governance_previews
            WHERE fingerprint=?
            """
            ,
            (fingerprint,),
        ).fetchone()
        if row is None:
            return None
        try:
            candidates = json.loads(str(row["candidates_json"]))
        except (TypeError, ValueError):
            return None
        operation = row["operation"]
        if not isinstance(operation, str) or not isinstance(candidates, list):
            return None
        return {
            "fingerprint": fingerprint,
            "operation": operation,
            "candidates": candidates,
        }

    @staticmethod
    def _event(connection, session_id: str | None, event_type: str, payload: dict[str, object], now: str) -> None:
        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (session_id, event_type, json.dumps(payload, sort_keys=True), now),
        )

    @staticmethod
    def _candidate_label(candidate: GovernanceCandidate) -> str:
        return f"{candidate.kind}:{candidate.identity}"

    @staticmethod
    def _frontmatter(content: str) -> dict[str, str]:
        lines = content.splitlines()
        if not lines or lines[0].strip() != "---":
            return {}
        metadata: dict[str, str] = {}
        for line in lines[1:]:
            if line.strip() == "---":
                break
            key, separator, value = line.partition(":")
            if separator and key.strip():
                metadata[key.strip()] = value.strip()
        return metadata

    @staticmethod
    def _archived_note_content(content: str, now: str) -> str:
        lines = content.splitlines(keepends=True)
        if not lines or lines[0].strip() != "---":
            return content
        replaced_status = False
        replaced_updated_at = False
        for index, line in enumerate(lines[1:], start=1):
            if line.strip() == "---":
                break
            newline = "\r\n" if line.endswith("\r\n") else "\n"
            if line.startswith("status:"):
                lines[index] = f"status: archived{newline}"
                replaced_status = True
            elif line.startswith("updated_at:"):
                lines[index] = f"updated_at: {now}{newline}"
                replaced_updated_at = True
        if not replaced_status:
            return content
        if not replaced_updated_at:
            closing = next((index for index, line in enumerate(lines) if index and line.strip() == "---"), None)
            if closing is not None:
                newline = "\r\n" if lines[closing].endswith("\r\n") else "\n"
                lines.insert(closing, f"updated_at: {now}{newline}")
        return "".join(lines)
