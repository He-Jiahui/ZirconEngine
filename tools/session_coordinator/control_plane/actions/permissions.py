from __future__ import annotations

from ...models import CoordinatorError, WebControlRole
from .models import ActionContext, ActionSpec


ROLE_RANK = {
    WebControlRole.OBSERVER: 0,
    WebControlRole.OPERATOR: 1,
    WebControlRole.COMMITTER: 2,
    WebControlRole.MAINTAINER: 3,
}


def require_permission(
    context: ActionContext, spec: ActionSpec, target_session_id: str | None
) -> None:
    if not spec.enabled:
        raise CoordinatorError("action_disabled", "Action is registered but not enabled yet")
    if ROLE_RANK[context.role] < ROLE_RANK[spec.required_role]:
        raise CoordinatorError(
            "action_permission_denied",
            f"Action requires {spec.required_role.value} permission",
        )
    if context.daemon_instance_id == "":
        raise CoordinatorError("action_instance_invalid", "Action identity has no daemon instance")
    if spec.session_bound:
        if not target_session_id or context.bound_session_id != target_session_id:
            raise CoordinatorError(
                "action_session_scope_mismatch",
                "Elevated web session is not bound to the target Session",
            )
