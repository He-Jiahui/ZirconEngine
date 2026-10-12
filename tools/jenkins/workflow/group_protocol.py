"""Public reference-only protocol for frozen multi-author Jenkins groups."""
from __future__ import annotations

from pathlib import Path

from ..contracts import JenkinsError, digest, require_digest
from ..resources import canonical_build_root
from .groups import (MAX_GROUP_SESSIONS, REQUEST_FIELDS, SessionGroups, _external_scope,
                     _session, request_identity, require_request)

_BASE = frozenset((*REQUEST_FIELDS, "driverDigest", "identity", "operationId", "groupId",
                   "expectedVersion", "runtimeOperationId", "ownedPaths", "buildRoot"))
_EXTRA = {"prepare-group": {"sessions", "mode"}, "observe-group": set(),
          "seal-joint": set(), "accept-joint": {"receiptRef", "flowId"},
          "consume-joint": set(), "cancel-group": set()}
_AUTHOR = frozenset(("identity", "sealedInputRef", *REQUEST_FIELDS, "driverDigest"))


def _runtime(root, identity, payload):
    from ..deployment.driver import active_driver_binding
    formal = ((root / ".jenkins/state/deployment/driver.json").exists()
              or (root / ".jenkins/deployment-spec.json").exists())
    if not formal:
        return None
    binding = active_driver_binding(root)
    current = require_digest(binding.get("digest"), "active driver digest")
    operation = binding.get("runtimeOperationId")
    if identity["stageImplementationDigest"] != current or identity.get("driverDigest", current) != current:
        raise JenkinsError("driver_context_mismatch", "New group operations require the current verified sealed driver")
    if not isinstance(operation, str) or not operation:
        raise JenkinsError("runtime_binding_missing", "Selected group runtime has no operation identity")
    if payload.get("runtimeOperationId", operation) != operation:
        raise JenkinsError("runtime_binding_mismatch", "Group request differs from the selected runtime")
    return operation


def _version(payload):
    value = payload.get("expectedVersion")
    if type(value) is not int or value < 1:
        raise JenkinsError("group_version_required", "Existing group mutations require a positive expectedVersion")
    return value


def _member(record, identity):
    author = next((value for value in record["payload"]["sessions"] if value["sessionId"] == identity["sessionId"]), None)
    if author is None or author.get("identity") != identity:
        raise JenkinsError("group_author_mismatch", "Operation requires an original group author identity")
    return author


def _authorize_member(groups, record, identity, *, allow_cancelled=False):
    author = _member(record, identity)
    with groups.state.transaction() as connection:
        require_request(groups.state, identity, author["paths"], connection=connection, allow_cancelled=allow_cancelled)
    return author


def _union_consumer(groups, record, identity, payload):
    row = groups.state.get("joint_input", record["payload"].get("jointManifestDigest"))
    if record["payload"]["mode"] != "joint" or not row:
        raise JenkinsError("joint_group_required", "Operation requires a frozen joint source")
    joint = row["payload"]
    from .joint import verify_joint_identity
    verify_joint_identity(joint)
    if (identity["repositoryId"] != joint["repositoryId"] or identity["sourceInputDigest"] != joint["sourceDigest"]
            or identity["coverageDigest"] != joint["coverageDigest"]):
        raise JenkinsError("joint_consumer_mismatch", "Union consumer must reference the frozen source and coverage")
    with groups.state.transaction() as connection:
        request = require_request(groups.state, identity,
            [*joint["ownedPaths"], *_external_scope(joint["sourceManifest"])], connection=connection)
    intent = request["payload"]
    if intent.get("jointGroupId") != record["key"]:
        raise JenkinsError("joint_consumer_mismatch", "Union request must bind its immutable jointGroupId")
    if not isinstance(intent.get("ownedPaths"), list) or sorted(intent["ownedPaths"]) != sorted(joint["ownedPaths"]):
        raise JenkinsError("joint_consumer_mismatch", "Union request requires its complete frozen owned scope")
    if canonical_build_root(intent.get("buildRoot")).path != canonical_build_root(joint["buildRoot"]).path:
        raise JenkinsError("joint_consumer_mismatch", "Union consumer must retain the registered build root")
    if "buildRoot" in payload and canonical_build_root(payload["buildRoot"]).path != canonical_build_root(joint["buildRoot"]).path:
        raise JenkinsError("joint_consumer_mismatch", "Caller cannot replace the union build root")
    if "ownedPaths" in payload and payload["ownedPaths"] != joint["ownedPaths"]:
        raise JenkinsError("joint_consumer_mismatch", "Caller cannot replace frozen union ownership")
    return joint, request


def _revalidate_authors(groups, record, *, include_cancelled=True):
    payload = record["payload"]
    with groups.state.transaction() as connection:
        for author in payload["sessions"]:
            cancelled = author["sessionId"] in payload.get("cancelledConsumers", [])
            if cancelled and not include_cancelled:
                continue
            if not isinstance(author.get("identity"), dict):
                raise JenkinsError("group_author_untrusted", "Historical fixture groups cannot acquire new acceptance")
            fresh = _session({"identity": author["identity"]}, groups.state, strict=True,
                repo_root=groups.repo_root, connection=connection, allow_cancelled=True)
            if fresh != author:
                raise JenkinsError("group_author_mismatch", "Frozen author source or request binding changed")


def resolve_joint_checkout(state, connection, context: Path, payload: dict,
                           intent: dict, identity: dict, source: str) -> tuple[Path, dict]:
    """Resolve a registered union consumer without adopting an author's scope.

    Callers dispatch here only for an explicit jointGroupId in the immutable
    request. This validates the original authors through their existing source
    checkout authority; it never manufactures or selects a checkout binding.
    """
    from ..deployment.driver import _plain
    from ..source.consumer_intake import _objects
    from ..source import _consumer_key, compute_source_digest
    from .joint import _authors, verify_joint_identity
    reference = require_digest(payload.get("jointGroupId"), "jointGroupId")
    if intent.get("jointGroupId") != reference:
        raise JenkinsError("joint_checkout_mismatch", "Union checkout must reference the immutable registered group")
    group = state.get(SessionGroups.DOMAIN, reference, connection=connection)
    if not group or group["payload"].get("schemaVersion") != 2 or group["payload"].get("mode") != "joint":
        raise JenkinsError("joint_checkout_mismatch", "Union requires its authoritative attributed joint group")
    grouped = group["payload"]
    union = state.get("joint_input", source, connection=connection)
    sealed_row = state.get("sealed_input", source, connection=connection)
    if not union or not sealed_row:
        raise JenkinsError("joint_checkout_mismatch", "Union checkout requires its existing sealed input")
    joint, sealed = union["payload"], sealed_row["payload"]
    verify_joint_identity(joint)
    if (grouped.get("jointManifestDigest") != source or grouped.get("jointConsumer") != identity
            or grouped.get("jointCoverageDigest") != identity["coverageDigest"]
            or source != identity["sourceInputDigest"] or joint.get("sourceDigest") != source
            or joint.get("coverageDigest") != identity["coverageDigest"]
            or joint.get("repositoryId") != identity["repositoryId"]
            or compute_source_digest(joint.get("sourceManifest", {})) != source
            or joint.get("authors") != joint.get("sourceManifest", {}).get("authorInputs")
            or joint.get("authors") != _authors(grouped["sessions"])
            or sealed.get("status") != "sealed" or sealed.get("jointGroupId") != reference
            or sealed.get("sourceDigest") != source or sealed.get("identity") != identity
            or sealed.get("manifestDigest") != source or sealed.get("coverageDigest") != identity["coverageDigest"]
            or sealed.get("coverage") != joint.get("coverage") or sealed.get("buildRoot") != joint.get("buildRoot")
            or sealed.get("owner") != identity["attemptId"] or sealed.get("sessionId") != identity["sessionId"]
            or sealed.get("manifest") != joint["sourceManifest"] or sealed.get("sourceManifest") != joint["sourceManifest"]
            or sealed.get("repositoryId") != identity["repositoryId"]
            or sealed.get("repositoryRoot") != joint.get("repositoryRoot")
            or sealed.get("objectRoot") != joint.get("objectRoot") or sealed.get("ownedPaths") != joint.get("ownedPaths")):
        raise JenkinsError("joint_checkout_mismatch", "Union source, author attribution or aggregator identity changed")
    frozen = {name: grouped["sessions"][0].get(name) for name in
              ("toolchainDigest", "lockDigest", "compilerIdentity", "wrapperDigest",
               "environmentPolicyDigest", "resourceProfile", "buildRoot")}
    if joint["coverage"].get("validationIdentity") != frozen:
        raise JenkinsError("joint_checkout_mismatch", "Union validation identity differs from its original authors")
    owned = joint["ownedPaths"]
    request = require_request(state, identity, [*owned, *_external_scope(joint["sourceManifest"])], connection=connection)
    if request["payload"] != intent or intent.get("ownedPaths") != owned:
        raise JenkinsError("joint_checkout_mismatch", "Union differs from the original aggregator request")
    consumer = state.get("sealed_consumer", _consumer_key(identity["repositoryId"], identity["sessionId"],
                         identity["attemptId"], source), connection=connection)
    if not consumer or consumer["payload"] != {"repositoryId": identity["repositoryId"], "sessionId": identity["sessionId"],
            "owner": identity["attemptId"], "sourceDigest": source, "manifestDigest": source, "status": "active"}:
        raise JenkinsError("joint_checkout_mismatch", "Union lacks its exact registered aggregator sealed consumer")
    root = Path(joint.get("repositoryRoot", ""))
    _plain(context)
    _plain(root)
    if not root.is_absolute() or not root.is_dir() or root.resolve() != root:
        raise JenkinsError("joint_checkout_mismatch", "Union checkout has no physical registered identity")
    for field in ("repositoryRoot", "objectRoot", "buildRoot"):
        if field in payload and payload[field] != sealed[field]:
            raise JenkinsError("joint_checkout_mismatch", "Caller cannot replace the frozen union storage identity")
    _objects(sealed, {})
    authors = []
    for author in grouped["sessions"]:
        fresh = _session({"identity": author["identity"]}, state, strict=True, repo_root=context,
                         connection=connection, allow_cancelled=True)
        if fresh != author:
            raise JenkinsError("joint_checkout_mismatch", "Original group author binding changed")
        authors.append({"identity": author["identity"], "requestPayloadDigest": author["requestPayloadDigest"],
                        "authorizationSnapshot": author["authorizationSnapshot"], "sourceDigest": author["sourceDigest"]})
    proof = {"jointGroupId": reference, "jointManifestDigest": source, "jointCoverageDigest": joint["coverageDigest"],
             "unionIdentity": identity, "authorBindingsDigest": digest(authors), "repositoryRoot": str(root),
             "originalAuthors": authors}
    return root, proof


def _metadata(groups, group_id, identity, joint):
    """Finish coordinator preparation against this exact frozen union only."""
    from ..patches import _ensure_patch_metadata
    from ..state.locks import process_lock
    lock_path = groups.state.path.parent / "joint-preparation-locks" / (group_id + ".lock")
    with process_lock(lock_path, reason="joint_preparation_busy"):
        record = groups.record(group_id)
        if record["payload"].get("jointPreparationStatus") == "prepared":
            return record
        try:
            _ensure_patch_metadata(groups.state, Path(joint["repositoryRoot"]),
                {"identity": identity, "buildRoot": joint["buildRoot"]},
                {"sourceDigest": joint["sourceDigest"]}, policy_root=groups.repo_root)
        except BaseException as error:
            with groups.state.transaction() as connection:
                current = groups.record(group_id, connection=connection)
                groups.state.put(groups.DOMAIN, group_id, {**current["payload"],
                    "jointPreparationStatus": "metadata-pending",
                    "jointPreparationError": {"reasonCode": getattr(error, "code", type(error).__name__),
                                              "retryable": getattr(error, "retryable", False)}},
                    expected_version=current["version"], connection=connection)
            raise
        with groups.state.transaction() as connection:
            current = groups.record(group_id, connection=connection)
            updated = {**current["payload"], "jointPreparationStatus": "prepared"}
            updated.pop("jointPreparationError", None)
            return groups.state.put(groups.DOMAIN, group_id, updated,
                                    expected_version=current["version"], connection=connection)


def handle(action, payload, state, repo_root: Path, *, domain="group"):
    if action not in _EXTRA:
        raise JenkinsError("operation_unknown", "Unknown public group operation")
    if not isinstance(payload, dict) or set(payload) - (_BASE | _EXTRA[action]):
        raise JenkinsError("group_input_invalid", "Group operations accept bounded identities and references only")
    groups = SessionGroups(state, repo_root=Path(repo_root).absolute(), strict=True)
    if action == "observe-group":
        # Observation preserves old groups without inventing current authority.
        record = groups.record(require_digest(payload.get("groupId"), "groupId"))
        if "identity" in payload:
            identity = request_identity(payload)
            if record["payload"].get("jointConsumer") != identity:
                _member(record, identity)
        return {**groups._projection(record).as_dict(),
                "jointPreparationStatus": record["payload"].get("jointPreparationStatus", "unsealed"),
                "jointPreparationError": record["payload"].get("jointPreparationError")}
    identity = request_identity(payload)
    if action == "prepare-group":
        sessions = payload.get("sessions")
        if (not isinstance(sessions, list) or not sessions or len(sessions) > MAX_GROUP_SESSIONS
                or any(not isinstance(value, dict) or set(value) - _AUTHOR for value in sessions)):
            raise JenkinsError("group_input_invalid", "Group authors must be a bounded list of original request references")
        identities = [request_identity(value) for value in sessions]
        if identity not in identities:
            raise JenkinsError("group_author_mismatch", "Only an original author can prepare this source group")
        for author_identity in identities:
            _runtime(groups.repo_root, author_identity, payload)
        group = groups.prepare(sessions, mode=payload.get("mode", "same-input"))
        return {**group.as_dict(), "sourceStatus": "draft" if group.mode == "joint" else "sealed",
                "standaloneSourceAccepted": False}
    group_id = require_digest(payload.get("groupId"), "groupId")
    version = _version(payload)
    record = groups.record(group_id)
    if action == "cancel-group":
        _authorize_member(groups, record, identity, allow_cancelled=True)
        group = groups.cancel(group_id, identity["sessionId"], identity["generation"], expected_version=version)
        return {**group.as_dict(), "standaloneSourceAccepted": False}
    _runtime(groups.repo_root, identity, payload)
    if action == "seal-joint":
        joint, _request = _union_consumer(groups, record, identity, payload)
        _revalidate_authors(groups, record)
        consumer = record["payload"].get("jointConsumer")
        if consumer and consumer != identity:
            raise JenkinsError("joint_consumer_conflict", "Group already has a different immutable union consumer")
        result = groups.seal_joint(group_id, session_id=identity["sessionId"], owner=identity["attemptId"],
                                  identity=identity, expected_version=version)
        prepared = _metadata(groups, group_id, identity, joint)
        return {**groups._projection(prepared).as_dict(), "status": "sealed", "sourceStatus": "sealed",
                "jointPreparationStatus": prepared["payload"]["jointPreparationStatus"],
                "sealedInputRef": result["sourceDigest"], "identity": identity,
                "coverageDigest": result["coverageDigest"], "buildRoot": result["buildRoot"],
                "standaloneSourceAccepted": False}
    if action == "accept-joint":
        _union_consumer(groups, record, identity, payload)
        if record["payload"].get("jointConsumer") != identity:
            raise JenkinsError("joint_consumer_mismatch", "Acceptance requires the registered sealed union consumer")
        if record["payload"].get("jointPreparationStatus") != "prepared":
            raise JenkinsError("joint_preparation_pending", "Union source metadata is still pending", retryable=True)
        _revalidate_authors(groups, record, include_cancelled=False)
        group = groups.accept_joint(group_id, {"receiptRef": payload.get("receiptRef"), "flowId": payload.get("flowId")},
                                    expected_version=version)
        return {**group.as_dict(), "standaloneSourceAccepted": False}
    if action == "consume-joint":
        _authorize_member(groups, record, identity)
        return groups.consume_joint(group_id, identity, expected_version=version)
    raise JenkinsError("operation_unknown", "Unknown public group operation")


__all__ = ["handle", "resolve_joint_checkout"]
