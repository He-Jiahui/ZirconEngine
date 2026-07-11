from __future__ import annotations

import hashlib
import secrets
import uuid
from dataclasses import dataclass
from datetime import timedelta
from http.cookies import SimpleCookie

from ..database import Database
from ..models import (
    CoordinatorError,
    WebControlRole,
    parse_utc,
    utc_now,
    utc_text,
)


COOKIE_NAME = "zircon_control"
OBSERVER_SESSION_SECONDS = 8 * 60 * 60


@dataclass(frozen=True, slots=True)
class WebSessionRecord:
    session_id: str
    role: str
    actor: str
    daemon_instance_id: str
    expires_at: str


class WebControlAuth:
    """Issues opaque browser credentials while keeping the runtime bearer server-side."""

    def __init__(self, database: Database):
        self.database = database

    def issue_bootstrap_ticket(
        self,
        actor: str,
        instance_id: str,
        ttl_seconds: int = 30,
        *,
        role: WebControlRole = WebControlRole.OBSERVER,
    ) -> str:
        if role is not WebControlRole.OBSERVER:
            raise CoordinatorError(
                "observer_only", "M1 bootstrap tickets may issue only Observer sessions"
            )
        raw_ticket = secrets.token_urlsafe(32)
        now = utc_now()
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO web_bootstrap_tickets(
                    ticket_hash, role, actor, daemon_instance_id,
                    created_at, expires_at
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                (
                    self._digest(raw_ticket),
                    role.value,
                    actor,
                    instance_id,
                    utc_text(now),
                    utc_text(now + timedelta(seconds=ttl_seconds)),
                ),
            )
        return raw_ticket

    def consume_bootstrap_ticket(
        self, raw_ticket: str, instance_id: str
    ) -> tuple[str, WebSessionRecord]:
        now = utc_now()
        ticket_hash = self._digest(raw_ticket)
        raw_session = secrets.token_urlsafe(32)
        session_id = uuid.uuid4().hex
        expires_at = now + timedelta(seconds=OBSERVER_SESSION_SECONDS)
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT * FROM web_bootstrap_tickets WHERE ticket_hash = ?",
                (ticket_hash,),
            ).fetchone()
            if row is None:
                raise CoordinatorError("bootstrap_ticket_invalid", "Bootstrap ticket is invalid")
            if row["daemon_instance_id"] != instance_id:
                raise CoordinatorError(
                    "bootstrap_ticket_instance_mismatch",
                    "Bootstrap ticket belongs to a different daemon instance",
                )
            if row["consumed_at"]:
                raise CoordinatorError(
                    "bootstrap_ticket_consumed", "Bootstrap ticket has already been consumed"
                )
            if parse_utc(row["expires_at"]) <= now:
                raise CoordinatorError("bootstrap_ticket_expired", "Bootstrap ticket has expired")
            connection.execute(
                "UPDATE web_bootstrap_tickets SET consumed_at = ? WHERE ticket_hash = ?",
                (utc_text(now), ticket_hash),
            )
            connection.execute(
                """
                INSERT INTO web_control_sessions(
                    session_token_hash, session_id, role, actor,
                    daemon_instance_id, created_at, last_seen_at, expires_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    self._digest(raw_session),
                    session_id,
                    row["role"],
                    row["actor"],
                    instance_id,
                    utc_text(now),
                    utc_text(now),
                    utc_text(expires_at),
                ),
            )
        return raw_session, WebSessionRecord(
            session_id=session_id,
            role=row["role"],
            actor=row["actor"],
            daemon_instance_id=instance_id,
            expires_at=utc_text(expires_at),
        )

    def authenticate_cookie(self, cookie_header: str, instance_id: str) -> WebSessionRecord:
        cookie = SimpleCookie()
        try:
            cookie.load(cookie_header)
            raw_session = cookie[COOKIE_NAME].value
        except (KeyError, AttributeError):
            raise CoordinatorError("web_session_missing", "Observer web session is required")
        now = utc_now()
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT * FROM web_control_sessions WHERE session_token_hash = ?",
                (self._digest(raw_session),),
            ).fetchone()
            if row is None or row["revoked_at"]:
                raise CoordinatorError("web_session_invalid", "Observer web session is invalid")
            if row["daemon_instance_id"] != instance_id:
                raise CoordinatorError(
                    "web_session_instance_mismatch",
                    "Observer web session belongs to a different daemon instance",
                )
            if parse_utc(row["expires_at"]) <= now:
                raise CoordinatorError("web_session_expired", "Observer web session has expired")
            connection.execute(
                "UPDATE web_control_sessions SET last_seen_at = ? WHERE session_token_hash = ?",
                (utc_text(now), self._digest(raw_session)),
            )
        return WebSessionRecord(
            session_id=row["session_id"],
            role=row["role"],
            actor=row["actor"],
            daemon_instance_id=row["daemon_instance_id"],
            expires_at=row["expires_at"],
        )

    @staticmethod
    def cookie_header(raw_session: str) -> str:
        return (
            f"{COOKIE_NAME}={raw_session}; HttpOnly; SameSite=Strict; "
            f"Path=/control; Max-Age={OBSERVER_SESSION_SECONDS}"
        )

    @staticmethod
    def _digest(raw_value: str) -> str:
        return hashlib.sha256(raw_value.encode("utf-8")).hexdigest()
