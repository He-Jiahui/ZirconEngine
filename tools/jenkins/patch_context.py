"""Resolve original acceptance checkouts from immutable deployment-owned bindings."""
from __future__ import annotations

from pathlib import Path

from .contracts import JenkinsError, digest
from .deployment.driver import _plain, active_driver_binding
from .source import repository_id

_FIELDS = ("repositoryId", "sessionId", "requestId", "attemptId", "generation", "stageImplementationDigest")


def _identity(value):
    identity = {key: value.get(key) for key in _FIELDS}
    if (any(not isinstance(identity[key], str) or not identity[key] for key in _FIELDS if key != "generation")
            or type(identity["generation"]) is not int or identity["generation"] != 1):
        raise JenkinsError("acceptance_checkout_identity", "Acceptance checkout needs a complete request identity")
    return identity


def _key(identity):
    return digest({key: identity[key] for key in _FIELDS[:5]})


def _physical_fixture(context: Path, value: str | Path) -> Path:
    context = context.absolute()
    root = Path(value).absolute()
    permitted = context / ".jenkins/tmp/acceptance"
    for path in (context, permitted, root):
        _plain(path)
    if (root.parent != permitted or not root.is_dir()
            or root.resolve() != root or context.resolve() != context):
        raise JenkinsError("acceptance_checkout_path", "Only an original physical acceptance checkout is allowed")
    return root


def register_acceptance_checkout(state, context_root: Path, payload: dict) -> dict:
    identity = _identity(payload.get("identity", payload))
    context = context_root.absolute()
    root = _physical_fixture(context, payload.get("checkoutRoot", ""))
    if identity["repositoryId"] != repository_id(root):
        raise JenkinsError("acceptance_checkout_identity", "Repository identity differs from the physical checkout")
    binding = active_driver_binding(context)
    if identity["stageImplementationDigest"] != binding["digest"]:
        raise JenkinsError("driver_context_mismatch", "Checkout registration must use the selected driver")
    scope = payload.get("ownedPaths")
    if not isinstance(scope, list) or not scope or not all(isinstance(path, str) and path for path in scope):
        raise JenkinsError("acceptance_checkout_scope", "Checkout registration needs an explicit task scope")
    from .source import canonical_paths
    scope = canonical_paths(root, scope)
    auth = state.check_authorization(identity["repositoryId"], identity["sessionId"], "implementation", scope)
    state.check_authorization(identity["repositoryId"], identity["sessionId"], "validation", scope)
    state.check_authorization(identity["repositoryId"], identity["sessionId"], "deployment", scope)
    record = {"schemaVersion": 1, "kind": "original-acceptance-checkout", "identity": identity,
              "contextRoot": str(context), "checkoutRoot": str(root), "ownedPaths": scope,
              "runtimeOperationId": binding["runtimeOperationId"], "driverDigest": binding["digest"],
              "authorizationSnapshot": auth["snapshotDigest"],
              "productRepositoryAcceptance": False}
    key = _key(identity)
    previous = state.get("acceptance_checkout", key)
    if previous:
        if previous["payload"] != record:
            raise JenkinsError("acceptance_checkout_conflict", "Acceptance checkout binding is immutable")
    else:
        state.put("acceptance_checkout", key, record, expected_version=0)
    return {"status": "registered", "checkoutBindingRef": key, "checkoutRoot": str(root),
            "runtimeOperationId": binding["runtimeOperationId"], "productRepositoryAcceptance": False}


def resolve_patch_checkout(state, context_root: Path, intent: dict) -> Path:
    context = context_root.absolute()
    supplied = Path(intent["repositoryRoot"]).absolute()
    if str(supplied).casefold() == str(context).casefold():
        return context
    identity = _identity(intent["identity"])
    previous = state.get("acceptance_checkout", _key(identity))
    if not previous:
        raise JenkinsError("patch_identity_mismatch", "Patch checkout has no deployment-owned binding")
    record = previous["payload"]
    root = _physical_fixture(context, supplied)
    binding = active_driver_binding(context)
    if (record.get("schemaVersion") != 1 or record.get("kind") != "original-acceptance-checkout"
            or record.get("identity") != identity or record.get("contextRoot") != str(context)
            or record.get("checkoutRoot") != str(root)
            or identity["repositoryId"] != repository_id(root)
            or record.get("runtimeOperationId") != binding["runtimeOperationId"]
            or record.get("driverDigest") != binding["digest"]
            or identity["stageImplementationDigest"] != binding["digest"]):
        raise JenkinsError("acceptance_checkout_conflict", "Patch checkout no longer matches its formal runtime")
    from .source import canonical_paths
    paths = canonical_paths(root, sorted(set(intent["beforeHashes"]) | set(intent.get("declaredDependencies", []))))
    allowed = record.get("ownedPaths", [])
    if any(not any(path == grant or path.startswith(grant + "/") for grant in allowed) for path in paths):
        raise JenkinsError("acceptance_checkout_scope", "Patch exceeds the registered acceptance scope")
    auth = state.check_authorization(identity["repositoryId"], identity["sessionId"], "implementation",
                                     list(intent["beforeHashes"]))
    state.check_authorization(identity["repositoryId"], identity["sessionId"], "validation", paths)
    if auth["snapshotDigest"] != record.get("authorizationSnapshot"):
        raise JenkinsError("patch_authorization_changed", "Acceptance checkout authorization changed")
    return root


def patch_checkout_for_registration(state, context_root: Path, payload: dict) -> Path:
    context = context_root.absolute()
    ref = payload.get("checkoutBindingRef")
    supplied = payload.get("repositoryRoot")
    if not ref:
        if supplied and str(Path(supplied).absolute()).casefold() != str(context).casefold():
            raise JenkinsError("acceptance_checkout_ref_required", "Alternate patch checkout requires its registered binding")
        return context
    identity = _identity(payload.get("identity", payload))
    if ref != _key(identity):
        raise JenkinsError("acceptance_checkout_conflict", "Checkout reference belongs to another request")
    previous = state.get("acceptance_checkout", ref)
    if not previous or not isinstance(previous["payload"].get("checkoutRoot"), str):
        raise JenkinsError("acceptance_checkout_missing", "Registered checkout binding is unavailable")
    root = Path(previous["payload"]["checkoutRoot"]).absolute()
    if supplied and str(Path(supplied).absolute()).casefold() != str(root).casefold():
        raise JenkinsError("acceptance_checkout_conflict", "Caller checkout differs from its registered binding")
    before = payload.get("beforeHashes")
    dependencies = payload.get("declaredDependencies", [])
    if not isinstance(before, dict) or not before:
        raise JenkinsError("patch_scope_mismatch", "Patch registration requires exact before hashes")
    if not isinstance(dependencies, list) or not all(isinstance(value, str) for value in dependencies):
        raise JenkinsError("patch_dependency_invalid", "Patch dependencies must be an explicit file list")
    return resolve_patch_checkout(state, context, {"identity": identity, "repositoryRoot": str(root),
                                  "beforeHashes": before, "declaredDependencies": dependencies})
