from __future__ import annotations

from ...models import CoordinatorError, WebControlRole
from .models import ActionContext, ActionKind, ActionSpec, SessionParameters


ROLE_RANK = {
    WebControlRole.OBSERVER: 0,
    WebControlRole.OPERATOR: 1,
    WebControlRole.COMMITTER: 2,
    WebControlRole.MAINTAINER: 3,
}


def require_permission(
    context: ActionContext,
    spec: ActionSpec,
    target_session_id: str | None,
    *,
    parameters: object | None = None,
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
        bootstrap = _scoped_bootstrap(parameters, spec)
        if bootstrap is not None:
            if ROLE_RANK[context.role] < ROLE_RANK[WebControlRole.MAINTAINER]:
                raise CoordinatorError(
                    "action_permission_denied",
                    "Scoped Session bootstrap requires maintainer permission",
                )
            if (
                context.web_session_id is not None
                and context.bound_session_id not in {None, bootstrap.maintenance_session_id}
            ):
                raise CoordinatorError(
                    "action_session_scope_mismatch",
                    "Elevated web session is bound to another maintenance Session",
                )
            return
        if not target_session_id or context.bound_session_id != target_session_id:
            raise CoordinatorError(
                "action_session_scope_mismatch",
                "Elevated web session is not bound to the target Session",
            )


def _scoped_bootstrap(parameters: object | None, spec: ActionSpec) -> SessionParameters | None:
    if spec.kind is not ActionKind.SESSION_ACTIVATE or not isinstance(parameters, SessionParameters):
        return None
    if parameters.maintenance_session_id is None:
        return None
    if parameters.display_name is None or parameters.plan_path is None or not parameters.write_scope:
        raise CoordinatorError(
            "action_parameters_invalid",
            "Scoped Session bootstrap requires displayName, planPath, and writeScope",
        )
    return parameters
