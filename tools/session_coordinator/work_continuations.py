"""Read-only same-plan continuations for Sessions waiting on a local resource."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from .models import CoordinatorError
from .plans import PlanRepository


_MAX_PLAN_BYTES = 2 * 1024 * 1024
_MAX_CONTINUATIONS = 20
_MAX_TITLE_CHARS = 500
_MILESTONE = re.compile(r"^#{2,6}\s+(M\d+)\b", re.IGNORECASE)
_SUBSECTION = re.compile(r"^#{3,6}\s+(?P<title>.+?)\s*$")
_UNCHECKED = re.compile(r"^\s*-\s*\[\s*\]\s*(?P<title>\S.*?)\s*$")
_IMPLEMENTATION = re.compile(r"implementation|实现|切片", re.IGNORECASE)
_TESTING = re.compile(r"test|验证|测试", re.IGNORECASE)


@dataclass(frozen=True, slots=True)
class ContinuationCandidate:
    kind: str
    plan_path: str
    milestone: str
    title: str


class WorkContinuationService:
    """Project one safe next slice without mutating a Session or resource queue.

    The recommendation is advisory. A worker still has to claim its concrete
    file scope before writing, so a validation wait can never steal another
    Session's lease or turn the coordinator into a cross-plan scheduler.
    """

    def __init__(self, repo_root: str | Path | None):
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else None
        self.plans = PlanRepository(self.repo_root) if self.repo_root is not None else None

    def project(self, connection) -> list[dict[str, object]]:
        if self.plans is None:
            return []
        rows = connection.execute(
            """
            SELECT session_id, plan_path, status,
                   CASE
                       WHEN status='waiting_lease' THEN 'lease'
                       WHEN EXISTS (
                           SELECT 1
                           FROM cargo_lane_reservations AS reservation
                           WHERE reservation.session_id=sessions.session_id
                             AND reservation.status='pending'
                       ) THEN 'validation'
                       ELSE 'external'
                   END AS wait_kind
            FROM sessions
            WHERE plan_path IS NOT NULL
              AND session_role='primary'
              AND (
                  status IN ('waiting_validation', 'waiting_lease')
                  OR (
                      status='active'
                      AND EXISTS (
                          SELECT 1
                          FROM cargo_lane_reservations AS reservation
                          WHERE reservation.session_id=sessions.session_id
                            AND reservation.status='pending'
                      )
                  )
              )
            ORDER BY updated_at, session_id
            LIMIT ?
            """,
            (_MAX_CONTINUATIONS,),
        ).fetchall()
        continuations: list[dict[str, object]] = []
        without_same_plan_work = []
        for row in rows:
            candidate = self._next_implementation_slice(str(row["plan_path"]))
            if candidate is None:
                without_same_plan_work.append(row)
                continue
            continuations.append(
                self._projection(row, candidate)
            )
        # A validation queue must never become the only work item.  When a
        # plan has no declared implementation slice left, expose one distinct
        # unowned code Failure per waiting Session.  The cards are advisory:
        # each worker still claims the target scope and returns to its primary
        # plan after the repair, so they cannot steal active work or turn the
        # queue into a global drain.
        if without_same_plan_work:
            fallbacks = self._next_unowned_code_failures(
                connection, limit=len(without_same_plan_work)
            )
            continuations.extend(
                self._projection(row, fallback)
                for row, fallback in zip(without_same_plan_work, fallbacks, strict=False)
            )
        return continuations

    @staticmethod
    def _projection(row, candidate: ContinuationCandidate) -> dict[str, object]:
        return {
            "sessionId": row["session_id"],
            "planPath": row["plan_path"],
            "waitKind": row["wait_kind"],
            "candidate": {
                "kind": candidate.kind,
                "planPath": candidate.plan_path,
                "milestone": candidate.milestone,
                "title": candidate.title,
            },
            "scopeClaimRequired": True,
            "returnToPrimary": True,
        }

    def _next_implementation_slice(self, plan_path: str) -> ContinuationCandidate | None:
        try:
            owner = self.plans.resolve_owner(plan_path)
        except CoordinatorError:
            return None
        source = self.repo_root / owner.path
        try:
            raw = source.read_bytes()
        except OSError:
            return None
        if len(raw) > _MAX_PLAN_BYTES:
            return None
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            return None
        milestone: str | None = None
        in_implementation = False
        for line in text.splitlines():
            if match := _MILESTONE.match(line):
                milestone = match.group(1).upper()
                in_implementation = False
                continue
            if match := _SUBSECTION.match(line):
                heading = match.group("title")
                in_implementation = bool(_IMPLEMENTATION.search(heading)) and not bool(
                    _TESTING.search(heading)
                )
                continue
            if milestone is None or not in_implementation:
                continue
            if match := _UNCHECKED.match(line):
                title = match.group("title").strip()
                if 0 < len(title) <= _MAX_TITLE_CHARS:
                    return ContinuationCandidate(
                        kind="same_plan",
                        plan_path=plan_path,
                        milestone=milestone,
                        title=title,
                    )
        return None

    @staticmethod
    def _next_unowned_code_failures(
        connection, *, limit: int
    ) -> list[ContinuationCandidate]:
        rows = connection.execute(
            """
            WITH ranked AS (
                SELECT failure.fixing_plan, failure.summary_slug, failure.priority,
                       failure.created_at, failure.node_id,
                       ROW_NUMBER() OVER (
                           PARTITION BY failure.fixing_plan
                           ORDER BY failure.priority, failure.created_at, failure.node_id
                       ) AS plan_rank
                FROM failure_nodes AS failure
                WHERE failure.status='open'
                  AND lower(failure.summary_slug) NOT LIKE '%plan-output%'
                  AND lower(failure.summary_slug) NOT LIKE '%archive-notice%'
                  AND lower(failure.summary_slug) NOT LIKE '%documentation%'
                  AND NOT EXISTS (
                      SELECT 1
                      FROM sessions AS owner
                      WHERE owner.plan_path=failure.fixing_plan
                        AND owner.status IN (
                            'registered', 'active', 'resolving_failure',
                            'waiting_lease', 'waiting_validation', 'finalizing'
                        )
                  )
            )
            SELECT fixing_plan, summary_slug, priority
            FROM ranked
            WHERE plan_rank=1
            ORDER BY priority, created_at, node_id
            LIMIT ?
            """
            ,
            (limit,),
        ).fetchall()
        return [
            ContinuationCandidate(
                kind="unowned_failure",
                plan_path=str(row["fixing_plan"]),
                milestone=f"Failure P{int(row['priority'])}",
                title=str(row["summary_slug"]),
            )
            for row in rows
        ]
