"""Build-root admission and bounded resource accounting."""

from .paths import ApprovedBuildRoot, canonical_build_root, build_namespace, physical_path_under
from .capacity import Capacity, Reservation, ReservationBook, ResourceManager
from ..contracts import JenkinsError
def handle(action, payload, state, repo_root, *, domain="resources"):
    if action == "query" and payload.get("view") == "policy":
        from ..contracts import response
        return response("observed", result={"policy": state.get("resource_policy", "default"),
                        "registrations": state.list("managed_storage_registration")})
    if action == "activate-policy":
        from .activation import activate_policy
        from ..contracts import response
        return response("accepted", operation_id=payload.get("operationId"), result=activate_policy(state, repo_root, payload))
    from .paths import canonical_build_root
    root=canonical_build_root(payload.get("buildRoot"))
    policy=state.get("resource_policy", "default")
    if not policy: raise JenkinsError("resource_policy_unavailable", "Trusted resource policy is unavailable", retryable=True)
    p=policy["payload"]
    total=Capacity(int(p["cpuBudget"]), int(p["memoryBudget"]), int(p["diskBudget"]))
    payload={**payload,"buildRoot":str(root.path)}
    manager=ResourceManager(state, total)
    result = manager.handle(action, payload)
    from ..contracts import response
    status = "accepted" if result is not None else "waiting"
    return response(status, operation_id=payload.get("operationId"), result=result)

__all__ = ["ApprovedBuildRoot", "canonical_build_root", "build_namespace", "physical_path_under", "Capacity", "Reservation", "ReservationBook", "ResourceManager"]
