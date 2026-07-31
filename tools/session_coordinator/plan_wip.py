from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from sqlite3 import Connection, Row

from .models import CoordinatorError
from .plans import PlanRepository


SESSION_ROLES = frozenset({"primary", "reviewer"})
TERMINAL_SESSION_STATUSES = ("completed", "archived", "cancelled")


@dataclass(frozen=True, slots=True)
class PlanWipAdmission:
    plan_family_key: str | None
    session_role: str
    parent_session_id: str | None


class PlanWipGate:
    """Serialize numbered-plan ownership inside Session registration transactions."""

    def __init__(self, repo_root: str | Path) -> None:
        self.plans = PlanRepository(repo_root)

    def admit_in_connection(
        self,
        connection: Connection,
        *,
        session_id: str,
        plan_path: str | None,
        session_role: str,
        parent_session_id: str | None,
        write_scope: tuple[str, ...],
        existing: Row | None,
    ) -> PlanWipAdmission:
        if session_role not in SESSION_ROLES:
            raise CoordinatorError(
                "plan_wip_session_role_invalid",
                "Session role must be primary or reviewer",
                details={"role": session_role},
            )
        family = self._family_key(plan_path)
        existing_family = str(existing["plan_family_key"]) if existing and existing["plan_family_key"] else None
        existing_role = str(existing["session_role"]) if existing else None
        existing_parent = str(existing["parent_session_id"]) if existing and existing["parent_session_id"] else None
        if existing_family is not None and family not in {None, existing_family}:
            raise CoordinatorError(
                "plan_wip_family_immutable",
                "An existing Session cannot change numbered Plan family",
                details={"existingFamily": existing_family, "requestedFamily": family},
            )
        if existing_role is not None and existing_role != session_role:
            raise CoordinatorError(
                "plan_wip_role_immutable",
                "An existing Session role is immutable",
                details={"existingRole": existing_role, "requestedRole": session_role},
            )
        if existing_parent is not None and existing_parent != parent_session_id:
            raise CoordinatorError(
                "plan_wip_parent_immutable",
                "An existing reviewer parent is immutable",
                details={"existingParentSessionId": existing_parent},
            )
        family = existing_family or family
        parent = existing_parent or parent_session_id
        if family is None:
            if session_role == "reviewer":
                raise CoordinatorError(
                    "plan_wip_reviewer_plan_required",
                    "A reviewer must target a numbered Plan family",
                )
            return PlanWipAdmission(None, session_role, None)
        if session_role == "primary":
            self._require_primary_slot(connection, session_id, family)
            return PlanWipAdmission(family, session_role, None)
        if write_scope:
            raise CoordinatorError(
                "plan_wip_reviewer_write_scope_forbidden",
                "A reviewer Session must not own a write scope",
            )
        if not parent:
            raise CoordinatorError(
                "plan_wip_reviewer_parent_required",
                "A reviewer Session must name its primary parent Session",
            )
        self._require_reviewer_parent(connection, parent, family)
        self._require_reviewer_slot(connection, session_id, family)
        return PlanWipAdmission(family, session_role, parent)

    def _family_key(self, plan_path: str | None) -> str | None:
        if not plan_path:
            return None
        try:
            return self.plans.resolve_owner(plan_path).child_dir
        except CoordinatorError as error:
            if error.code in {"plan_not_found", "not_numbered_plan"}:
                return None
            raise

    @staticmethod
    def _require_primary_slot(connection: Connection, session_id: str, family: str) -> None:
        placeholders = ", ".join("?" for _ in TERMINAL_SESSION_STATUSES)
        existing = connection.execute(
            f"""
            SELECT session_id, status, last_heartbeat_at
            FROM sessions
            WHERE plan_family_key=? AND session_role='primary' AND session_id<>?
              AND status NOT IN ({placeholders})
            ORDER BY last_heartbeat_at DESC, session_id
            LIMIT 1
            """,
            (family, session_id, *TERMINAL_SESSION_STATUSES),
        ).fetchone()
        if existing is None:
            return
        raise CoordinatorError(
            "plan_wip_limit_reached",
            f"Plan family {family} already has an executable primary Session",
            details={
                "planFamily": family,
                "primarySessionId": str(existing["session_id"]),
                "primaryStatus": str(existing["status"]),
                "primaryLastHeartbeatAt": str(existing["last_heartbeat_at"]),
            },
        )

    @staticmethod
    def _require_reviewer_parent(connection: Connection, parent: str, family: str) -> None:
        placeholders = ", ".join("?" for _ in TERMINAL_SESSION_STATUSES)
        row = connection.execute(
            f"""
            SELECT session_id FROM sessions
            WHERE session_id=? AND plan_family_key=? AND session_role='primary'
              AND status NOT IN ({placeholders})
            """,
            (parent, family, *TERMINAL_SESSION_STATUSES),
        ).fetchone()
        if row is None:
            raise CoordinatorError(
                "plan_wip_reviewer_parent_invalid",
                "Reviewer parent must be an executable primary in the same Plan family",
                details={"parentSessionId": parent, "planFamily": family},
            )

    @staticmethod
    def _require_reviewer_slot(connection: Connection, session_id: str, family: str) -> None:
        placeholders = ", ".join("?" for _ in TERMINAL_SESSION_STATUSES)
        existing = connection.execute(
            f"""
            SELECT session_id, parent_session_id, status
            FROM sessions
            WHERE plan_family_key=? AND session_role='reviewer' AND session_id<>?
              AND status NOT IN ({placeholders})
            ORDER BY last_heartbeat_at DESC, session_id
            LIMIT 1
            """,
            (family, session_id, *TERMINAL_SESSION_STATUSES),
        ).fetchone()
        if existing is None:
            return
        raise CoordinatorError(
            "plan_wip_reviewer_limit_reached",
            f"Plan family {family} already has a reviewer Session",
            details={
                "planFamily": family,
                "reviewerSessionId": str(existing["session_id"]),
                "parentSessionId": str(existing["parent_session_id"]),
                "reviewerStatus": str(existing["status"]),
            },
        )
