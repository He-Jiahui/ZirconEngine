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
            SELECT session_id, plan_path, status
            FROM sessions
            WHERE status IN ('waiting_validation', 'waiting_lease')
              AND plan_path IS NOT NULL
            ORDER BY updated_at, session_id
            LIMIT ?
            """,
            (_MAX_CONTINUATIONS,),
        ).fetchall()
        continuations: list[dict[str, object]] = []
        for row in rows:
            candidate = self._next_implementation_slice(str(row["plan_path"]))
            if candidate is None:
                continue
            continuations.append(
                {
                    "sessionId": row["session_id"],
                    "planPath": row["plan_path"],
                    "waitKind": (
                        "validation"
                        if row["status"] == "waiting_validation"
                        else "lease"
                    ),
                    "candidate": {
                        "milestone": candidate.milestone,
                        "title": candidate.title,
                    },
                    "scopeClaimRequired": True,
                    "returnToPrimary": True,
                }
            )
        return continuations

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
                    return ContinuationCandidate(milestone=milestone, title=title)
        return None
