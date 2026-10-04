"""Gap-issue tracking: SQLite state layer.

Two tables are added to the existing coordination database:
  gap_issues  — one row per extracted P0/P1/P2 gap item
  gap_claims  — one row per claim attempt (at most one active per issue)
"""
from __future__ import annotations

import sqlite3
import uuid
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterator

from ..contracts import JenkinsError

# Resolved at import time; callers may override via State(db_path=...).
_DEFAULT_DB = Path(__file__).parents[3] / ".jenkins" / "state" / "coordination.sqlite3"

_DDL = """
CREATE TABLE IF NOT EXISTS gap_issues (
    issue_id           TEXT PRIMARY KEY,
    doc_path           TEXT NOT NULL,
    priority           TEXT NOT NULL CHECK(priority IN ('P0','P1','P2')),
    gate_id            TEXT,
    summary            TEXT NOT NULL,
    status             TEXT NOT NULL DEFAULT 'open'
                           CHECK(status IN ('open','claimed','fixing','submitted','accepted')),
    source_fingerprint TEXT,
    extracted_at       TEXT NOT NULL,
    updated_at         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS gap_claims (
    claim_id           TEXT PRIMARY KEY,
    issue_id           TEXT NOT NULL REFERENCES gap_issues(issue_id),
    session_id         TEXT NOT NULL,
    claimed_at         TEXT NOT NULL,
    released_at        TEXT,
    jenkins_request_id TEXT,
    jenkins_build_id   TEXT,
    receipt_ref        TEXT,
    outcome            TEXT CHECK(outcome IS NULL OR outcome IN ('passed','failed','abandoned'))
);

CREATE INDEX IF NOT EXISTS gap_issues_status  ON gap_issues(status);
CREATE INDEX IF NOT EXISTS gap_issues_priority ON gap_issues(priority);
CREATE INDEX IF NOT EXISTS gap_claims_issue   ON gap_claims(issue_id);
"""


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


class GapState:
    """Thin transactional wrapper around the two gap tables."""

    def __init__(self, db_path: str | Path | None = None) -> None:
        self._path = Path(db_path) if db_path else _DEFAULT_DB

    @contextmanager
    def _conn(self) -> Iterator[sqlite3.Connection]:
        con = sqlite3.connect(str(self._path), timeout=10)
        con.execute("PRAGMA journal_mode=WAL")
        con.execute("PRAGMA foreign_keys=ON")
        try:
            yield con
            con.commit()
        except Exception:
            con.rollback()
            raise
        finally:
            con.close()

    def ensure_schema(self) -> None:
        with self._conn() as con:
            con.executescript(_DDL)

    # ── Issues ──────────────────────────────────────────────────────────────

    def upsert_issue(
        self,
        issue_id: str,
        doc_path: str,
        priority: str,
        summary: str,
        *,
        gate_id: str | None = None,
        source_fingerprint: str | None = None,
    ) -> str:
        """Insert or ignore (existing rows are not overwritten)."""
        now = _now()
        with self._conn() as con:
            con.execute(
                """
                INSERT OR IGNORE INTO gap_issues
                    (issue_id, doc_path, priority, gate_id, summary,
                     source_fingerprint, extracted_at, updated_at)
                VALUES (?,?,?,?,?,?,?,?)
                """,
                (issue_id, doc_path, priority, gate_id, summary[:200],
                 source_fingerprint, now, now),
            )
        return issue_id

    def get_issue(self, issue_id: str) -> dict | None:
        with self._conn() as con:
            row = con.execute(
                "SELECT * FROM gap_issues WHERE issue_id = ?", (issue_id,)
            ).fetchone()
        if row is None:
            return None
        cols = ["issue_id", "doc_path", "priority", "gate_id", "summary",
                "status", "source_fingerprint", "extracted_at", "updated_at"]
        return dict(zip(cols, row))

    def list_issues(
        self,
        *,
        priority: str | None = None,
        status: str | None = None,
        limit: int = 50,
    ) -> list[dict]:
        clauses, params = [], []
        if priority:
            clauses.append("priority = ?"); params.append(priority)
        if status:
            clauses.append("status = ?"); params.append(status)
        where = ("WHERE " + " AND ".join(clauses)) if clauses else ""
        with self._conn() as con:
            rows = con.execute(
                f"SELECT * FROM gap_issues {where} ORDER BY priority, issue_id LIMIT ?",
                params + [limit],
            ).fetchall()
        cols = ["issue_id", "doc_path", "priority", "gate_id", "summary",
                "status", "source_fingerprint", "extracted_at", "updated_at"]
        return [dict(zip(cols, r)) for r in rows]

    def _set_status(self, con: sqlite3.Connection, issue_id: str, status: str) -> None:
        con.execute(
            "UPDATE gap_issues SET status = ?, updated_at = ? WHERE issue_id = ?",
            (status, _now(), issue_id),
        )

    # ── Claims ───────────────────────────────────────────────────────────────

    def _active_claim(self, con: sqlite3.Connection, issue_id: str) -> dict | None:
        row = con.execute(
            "SELECT * FROM gap_claims WHERE issue_id = ? AND outcome IS NULL",
            (issue_id,),
        ).fetchone()
        if row is None:
            return None
        cols = ["claim_id", "issue_id", "session_id", "claimed_at", "released_at",
                "jenkins_request_id", "jenkins_build_id", "receipt_ref", "outcome"]
        return dict(zip(cols, row))

    def claim(self, issue_id: str, session_id: str) -> str:
        """Claim an open issue. Returns claim_id or raises GapConflict."""
        with self._conn() as con:
            issue = con.execute(
                "SELECT status FROM gap_issues WHERE issue_id = ?", (issue_id,)
            ).fetchone()
            if issue is None:
                raise JenkinsError("gap_not_found", f"Issue {issue_id!r} not found")
            if issue[0] != "open":
                active = self._active_claim(con, issue_id)
                owner = active["session_id"] if active else "unknown"
                raise JenkinsError(
                    "gap_already_claimed",
                    f"Issue {issue_id!r} is {issue[0]!r} (owner: {owner})",
                )
            claim_id = str(uuid.uuid4())
            con.execute(
                """INSERT INTO gap_claims (claim_id, issue_id, session_id, claimed_at)
                   VALUES (?,?,?,?)""",
                (claim_id, issue_id, session_id, _now()),
            )
            self._set_status(con, issue_id, "claimed")
        return claim_id

    def release(self, issue_id: str, session_id: str) -> None:
        """Release a claim owned by session_id, return issue to open."""
        with self._conn() as con:
            active = self._active_claim(con, issue_id)
            if active is None or active["session_id"] != session_id:
                raise JenkinsError(
                    "gap_not_owner",
                    f"Session {session_id!r} does not own claim on {issue_id!r}",
                )
            now = _now()
            con.execute(
                "UPDATE gap_claims SET released_at = ?, outcome = 'abandoned' WHERE claim_id = ?",
                (now, active["claim_id"]),
            )
            self._set_status(con, issue_id, "open")

    def record_submission(
        self,
        issue_id: str,
        session_id: str,
        jenkins_request_id: str,
        jenkins_build_id: str | None = None,
    ) -> None:
        """Transition claimed → submitted."""
        with self._conn() as con:
            active = self._active_claim(con, issue_id)
            if active is None or active["session_id"] != session_id:
                raise JenkinsError("gap_not_owner",
                                   f"Session {session_id!r} does not own {issue_id!r}")
            con.execute(
                """UPDATE gap_claims
                   SET jenkins_request_id = ?, jenkins_build_id = ?
                   WHERE claim_id = ?""",
                (jenkins_request_id, jenkins_build_id, active["claim_id"]),
            )
            self._set_status(con, issue_id, "submitted")

    def accept(self, issue_id: str, receipt_ref: str) -> None:
        """Transition submitted → accepted, record receipt."""
        with self._conn() as con:
            issue = con.execute(
                "SELECT status FROM gap_issues WHERE issue_id = ?", (issue_id,)
            ).fetchone()
            if issue is None:
                raise JenkinsError("gap_not_found", f"Issue {issue_id!r} not found")
            if issue[0] != "submitted":
                raise JenkinsError(
                    "gap_wrong_state",
                    f"Issue {issue_id!r} is {issue[0]!r}, not submitted",
                )
            active = self._active_claim(con, issue_id)
            if active:
                con.execute(
                    "UPDATE gap_claims SET receipt_ref = ?, outcome = 'passed' WHERE claim_id = ?",
                    (receipt_ref, active["claim_id"]),
                )
            self._set_status(con, issue_id, "accepted")
