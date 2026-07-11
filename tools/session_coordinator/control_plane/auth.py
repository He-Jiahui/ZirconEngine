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
ELEVATED_SESSION_SECONDS = 15 * 60
ELEVATION_GRANT_SECONDS = 60
ROLE_RANK = {
    WebControlRole.OBSERVER: 0,
    WebControlRole.OPERATOR: 1,
    WebControlRole.COMMITTER: 2,
    WebControlRole.MAINTAINER: 3,
}


@dataclass(frozen=True, slots=True)
class WebSessionRecord:
    session_id: str
    role: str
    actor: str
    daemon_instance_id: str
    expires_at: str
    bound_session_id: str | None = None
    elevated_until: str | None = None


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

    def issue_elevation_grant(
        self,
        actor: str,
        role: WebControlRole,
        instance_id: str,
        *,
        bound_session_id: str | None = None,
        ttl_seconds: int = ELEVATION_GRANT_SECONDS,
        maintenance_authorized: bool = False,
    ) -> str:
        if role is WebControlRole.OBSERVER:
            raise CoordinatorError(
                "elevation_role_invalid", "Observer is the default role and cannot be elevated"
            )
        if role is WebControlRole.COMMITTER and not bound_session_id:
            raise CoordinatorError(
                "elevation_session_required", "Committer elevation must bind to a Session"
            )
        if role is WebControlRole.MAINTAINER and not maintenance_authorized:
            raise CoordinatorError(
                "maintenance_unauthorized",
                "Maintainer elevation requires the separate local maintenance capability",
            )
        grant = secrets.token_urlsafe(32)
        now = utc_now()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO web_elevation_grants(
                       grant_hash, actor, role, bound_session_id, daemon_instance_id,
                       created_at, expires_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?)""",
                (
                    self._digest(grant),
                    actor,
                    role.value,
                    bound_session_id,
                    instance_id,
                    utc_text(now),
                    utc_text(now + timedelta(seconds=ttl_seconds)),
                ),
            )
        return grant

    def consume_elevation_grant(
        self, raw_grant: str, cookie_header: str, instance_id: str
    ) -> tuple[str, WebSessionRecord]:
        raw_session = self._raw_cookie(cookie_header)
        now = utc_now()
        csrf = secrets.token_urlsafe(32)
        with self.database.transaction() as connection:
            session = connection.execute(
                "SELECT * FROM web_control_sessions WHERE session_token_hash = ?",
                (self._digest(raw_session),),
            ).fetchone()
            if session is None or session["revoked_at"]:
                raise CoordinatorError("web_session_invalid", "Web session is invalid")
            if session["daemon_instance_id"] != instance_id:
                raise CoordinatorError(
                    "web_session_instance_mismatch", "Web session belongs to another daemon"
                )
            if parse_utc(session["expires_at"]) <= now:
                raise CoordinatorError("web_session_expired", "Web session has expired")
            grant = connection.execute(
                "SELECT * FROM web_elevation_grants WHERE grant_hash = ?",
                (self._digest(raw_grant),),
            ).fetchone()
            if grant is None:
                raise CoordinatorError("elevation_grant_invalid", "Elevation grant is invalid")
            if grant["daemon_instance_id"] != instance_id:
                raise CoordinatorError(
                    "elevation_instance_mismatch", "Elevation grant belongs to another daemon"
                )
            if grant["actor"] != session["actor"]:
                raise CoordinatorError(
                    "elevation_actor_mismatch", "Elevation grant belongs to another actor"
                )
            if grant["consumed_at"]:
                raise CoordinatorError(
                    "elevation_grant_consumed", "Elevation grant has already been consumed"
                )
            if parse_utc(grant["expires_at"]) <= now:
                raise CoordinatorError("elevation_grant_expired", "Elevation grant has expired")
            current_role = WebControlRole(session["role"])
            requested_role = WebControlRole(grant["role"])
            if ROLE_RANK[requested_role] < ROLE_RANK[current_role]:
                raise CoordinatorError(
                    "elevation_downgrade", "Elevation cannot lower an active web role"
                )
            elevated_until = now + timedelta(seconds=ELEVATED_SESSION_SECONDS)
            connection.execute(
                "UPDATE web_elevation_grants SET consumed_at = ? WHERE grant_hash = ?",
                (utc_text(now), self._digest(raw_grant)),
            )
            connection.execute(
                """UPDATE web_control_sessions
                   SET role = ?, bound_session_id = ?, csrf_token_hash = ?,
                       elevated_until = ?, last_seen_at = ?
                   WHERE session_token_hash = ?""",
                (
                    requested_role.value,
                    grant["bound_session_id"],
                    self._digest(csrf),
                    utc_text(elevated_until),
                    utc_text(now),
                    self._digest(raw_session),
                ),
            )
        return csrf, WebSessionRecord(
            session_id=session["session_id"],
            role=requested_role.value,
            actor=session["actor"],
            daemon_instance_id=instance_id,
            expires_at=session["expires_at"],
            bound_session_id=grant["bound_session_id"],
            elevated_until=utc_text(elevated_until),
        )

    def authenticate_cookie(self, cookie_header: str, instance_id: str) -> WebSessionRecord:
        raw_session = self._raw_cookie(cookie_header)
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
            role = row["role"]
            bound_session_id = row["bound_session_id"]
            elevated_until = row["elevated_until"]
            if elevated_until and parse_utc(elevated_until) <= now:
                role = WebControlRole.OBSERVER.value
                bound_session_id = None
                elevated_until = None
                connection.execute(
                    """UPDATE web_control_sessions
                       SET role = 'observer', bound_session_id = NULL,
                           csrf_token_hash = NULL, elevated_until = NULL
                       WHERE session_token_hash = ?""",
                    (self._digest(raw_session),),
                )
            connection.execute(
                "UPDATE web_control_sessions SET last_seen_at = ? WHERE session_token_hash = ?",
                (utc_text(now), self._digest(raw_session)),
            )
        return WebSessionRecord(
            session_id=row["session_id"],
            role=role,
            actor=row["actor"],
            daemon_instance_id=row["daemon_instance_id"],
            expires_at=row["expires_at"],
            bound_session_id=bound_session_id,
            elevated_until=elevated_until,
        )

    def validate_csrf(
        self, cookie_header: str, supplied_token: str, instance_id: str
    ) -> WebSessionRecord:
        session = self.authenticate_cookie(cookie_header, instance_id)
        if not supplied_token:
            raise CoordinatorError("csrf_invalid", "CSRF token is required")
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT csrf_token_hash FROM web_control_sessions WHERE session_id = ?",
                (session.session_id,),
            ).fetchone()
        if row is None or not row["csrf_token_hash"] or not secrets.compare_digest(
            row["csrf_token_hash"], self._digest(supplied_token)
        ):
            raise CoordinatorError("csrf_invalid", "CSRF token is missing or mismatched")
        return session

    @staticmethod
    def require_bound_session(session: WebSessionRecord, session_id: str) -> None:
        if session.bound_session_id != session_id:
            raise CoordinatorError(
                "web_session_scope_mismatch", "Web elevation is bound to another Session"
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

    @staticmethod
    def _raw_cookie(cookie_header: str) -> str:
        cookie = SimpleCookie()
        try:
            cookie.load(cookie_header)
            return cookie[COOKIE_NAME].value
        except (KeyError, AttributeError):
            raise CoordinatorError("web_session_missing", "Observer web session is required")
