"""Bounded, persistent grouping of immutable authored source inputs.

A group records a frozen union. Its consumption records attest to that union,
never to an author's independent source or ordinary request result.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Mapping

from ..contracts import JenkinsError, digest, identifier, require_digest
from ..state import State
from ..source import _consumer_key, canonical_path, compute_source_digest

MAX_GROUP_SESSIONS = 16
REQUEST_FIELDS = ("repositoryId", "sessionId", "requestId", "attemptId", "generation",
                  "sourceInputDigest", "coverageDigest", "stageImplementationDigest")
_IDENTITY_FIELDS = ("repositoryId", "baseHead", "toolchainDigest", "lockDigest",
                    "compilerIdentity", "wrapperDigest", "profile", "features", "buildRoot")


def _text(value: Any, name: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise JenkinsError("group_identity_missing", f"{name} is required")
    return value


def request_identity(value: Mapping[str, Any]) -> dict:
    supplied = value.get("identity", value)
    if not isinstance(supplied, Mapping):
        raise JenkinsError("identity_missing", "Grouping requires a complete request identity")
    if "identity" in value and set(supplied) - set((*REQUEST_FIELDS, "driverDigest")):
        raise JenkinsError("identity_invalid", "Group identity contains unsupported fields")
    identity = {name: supplied.get(name) for name in REQUEST_FIELDS}
    for name in REQUEST_FIELDS[:4]:
        identifier(identity[name], name)
    if type(identity["generation"]) is not int or identity["generation"] < 1:
        raise JenkinsError("group_generation_invalid", "generation must be a positive integer")
    for name in REQUEST_FIELDS[5:]:
        require_digest(identity[name], name)
    if "driverDigest" in supplied:
        identity["driverDigest"] = require_digest(supplied["driverDigest"], "driverDigest")
        if identity["driverDigest"] != identity["stageImplementationDigest"]:
            raise JenkinsError("driver_context_mismatch", "Request driver identities differ")
    for name, field in identity.items():
        if name in value and value[name] != field:
            raise JenkinsError("identity_conflict", "Control and request identities differ")
    return identity


def require_request(state: State, identity: Mapping[str, Any], paths: list[str], *,
                    connection=None, allow_cancelled: bool = False) -> dict:
    repo, session, request_id = (identity[name] for name in REQUEST_FIELDS[:3])
    record = state.get_request(repo, session, request_id, connection=connection)
    if not record:
        raise JenkinsError("request_not_registered", "Grouping requires the original registered request")
    intent = record["payload"]
    if (not isinstance(intent, dict) or record["payloadDigest"] != digest(intent)
            or type(intent.get("generation")) is not int
            or any(intent.get(name) != value for name, value in identity.items())):
        raise JenkinsError("request_identity_mismatch", "Grouping cannot replace the original request identity")
    if record["generation"] != identity["generation"]:
        raise JenkinsError("request_generation_mismatch", "Request generation changed")
    if any(intent[name] != identity["sourceInputDigest"] for name in
           ("sealedInputRef", "sourceDigest", "sourceRef", "manifestDigest") if name in intent):
        raise JenkinsError("request_identity_mismatch", "Registered request source references differ")
    if intent.get("driverDigest", identity["stageImplementationDigest"]) != identity["stageImplementationDigest"]:
        raise JenkinsError("request_identity_mismatch", "Registered driver references differ")
    cancelled = (record["status"] in {"cancelled", "cancel_requested"}
                 or state.get("cancellation", f"{repo}:{session}:{request_id}", connection=connection))
    if cancelled and not allow_cancelled:
        raise JenkinsError("request_cancelled", "Cancelled authors cannot acquire union acceptance")
    if record["status"] in {"failed", "rejected"} and not allow_cancelled:
        raise JenkinsError("request_terminal", "Failed authors cannot acquire union acceptance")
    grant = state.check_authorization(repo, session, "validation", paths, connection=connection)
    if intent.get("authorizationSnapshot", grant["snapshotDigest"]) != grant["snapshotDigest"]:
        raise JenkinsError("authorization_conflict", "Grouping authorization differs from the registered request")
    return {**record, "authorizationSnapshot": grant["snapshotDigest"]}


def _paths(session: Mapping[str, Any], root: str | Path) -> tuple[str, ...]:
    values = session.get("paths")
    if values is None:
        values = [claim.get("path") for claim in session.get("claims", []) if isinstance(claim, Mapping)]
    if not isinstance(values, (list, tuple)) or not values:
        raise JenkinsError("group_paths_missing", "A session must carry sealed source ownership")
    out = tuple(sorted(canonical_path(root, _text(path, "path")) for path in values))
    if len(set(out)) != len(out):
        raise JenkinsError("group_path_overlap", "One session contains duplicate path identities")
    for index, left in enumerate(out):
        if any(right.startswith(left + "/") for right in out[index + 1:]):
            raise JenkinsError("group_path_overlap", "One session contains overlapping paths")
    return out


def _sealed(state: State, session: Mapping[str, Any], *, connection=None) -> Mapping[str, Any]:
    identity = session.get("identity") or {}
    ref = session.get("sealedInputRef") or session.get("sourceDigest") or identity.get("sourceInputDigest")
    _text(ref, "sealedInputRef")
    record = state.get("sealed_input", ref, connection=connection)
    if not record or not isinstance(record.get("payload"), Mapping):
        raise JenkinsError("sealed_input_missing", "Grouping requires an authoritative sealed input")
    sealed = record["payload"]
    manifest = sealed.get("sourceManifest") or sealed.get("manifest")
    if (sealed.get("status") != "sealed" or sealed.get("sourceDigest") != ref
            or not isinstance(manifest, Mapping) or compute_source_digest(manifest) != ref
            or "manifest" in sealed and sealed["manifest"] != manifest):
        raise JenkinsError("sealed_input_corrupt", "Grouping source manifest differs from its immutable identity")
    return sealed


def _external_scope(manifest: Mapping) -> list[str]:
    return ["external/" + str(external["name"]) + "/" + entry["path"]
            for external in manifest.get("external", []) if isinstance(external, Mapping)
            and isinstance(external.get("manifest"), Mapping)
            for entry in external["manifest"].get("entries", [])]


def _session(session: Mapping[str, Any], state: State, *, strict: bool = False,
             repo_root: Path | None = None, connection=None, allow_cancelled: bool = False) -> dict[str, Any]:
    if not isinstance(session, Mapping):
        raise JenkinsError("group_input_invalid", "Each author must be a source reference object")
    identity = request_identity(session) if strict else None
    supplied = {**session, **(identity or {})}
    for field in ("repositoryId", "sessionId", "requestId", "generation"):
        if field not in supplied:
            raise JenkinsError("group_identity_missing", f"{field} is required")
    for field in ("repositoryId", "sessionId", "requestId"):
        identifier(supplied[field], field)
    if type(supplied["generation"]) is not int or supplied["generation"] < 1:
        raise JenkinsError("group_generation_invalid", "generation must be a positive integer")
    sealed = _sealed(state, {**supplied, "sealedInputRef": supplied.get("sealedInputRef") or supplied.get("sourceInputDigest")
                                      or supplied.get("sourceDigest")}, connection=connection)
    source = sealed["sourceDigest"]
    if supplied["repositoryId"] != sealed.get("repositoryId"):
        raise JenkinsError("group_repository_mismatch", "Session belongs to another repository")
    owner = _text(supplied.get("attemptId") or supplied.get("owner"), "attemptId")
    consumer = state.get("sealed_consumer", _consumer_key(supplied["repositoryId"],
        supplied["sessionId"], owner, source), connection=connection)
    if not consumer or consumer["payload"].get("status") != "active":
        raise JenkinsError("sealed_consumer_missing", "Grouping requires an active sealed consumer")
    if consumer["payload"] != {"repositoryId": supplied["repositoryId"], "sessionId": supplied["sessionId"],
            "owner": owner, "sourceDigest": source, "manifestDigest": source, "status": "active"}:
        raise JenkinsError("sealed_consumer_mismatch", "Grouping consumer differs from its original sealed reference")
    if supplied.get("sourceDigest", source) != source:
        raise JenkinsError("group_identity_mismatch", "Session source differs from its sealed input")
    manifest = sealed.get("sourceManifest") or sealed["manifest"]
    root = _text(sealed.get("repositoryRoot") or sealed.get("repoRoot"), "repositoryRoot")
    paths = _paths({"paths": sealed.get("ownedPaths") or [c["path"] for c in sealed.get("claims", [])]}, root)
    if "paths" in supplied and _paths(supplied, root) != paths:
        raise JenkinsError("group_paths_mismatch", "Caller cannot replace sealed source ownership")
    coverage = sealed.get("coverage")
    if not isinstance(coverage, Mapping) or coverage.get("sourceDigest") not in (None, source):
        raise JenkinsError("group_coverage_mismatch", "Sealed coverage is not bound to the source")
    if any(not isinstance(coverage.get(name, []), list) or any(not isinstance(value, str) or not value
                or "\n" in value or "\r" in value for value in coverage.get(name, []))
           for name in ("features", "requiredTests", "packages")):
        raise JenkinsError("group_coverage_mismatch", "Sealed feature, package and test scopes must be explicit lists")
    if "noDefaultFeatures" in coverage and type(coverage["noDefaultFeatures"]) is not bool:
        raise JenkinsError("group_coverage_mismatch", "Sealed default feature selection must be a boolean")
    request = None
    if strict:
        if identity["sourceInputDigest"] != source or identity["coverageDigest"] != sealed.get("coverageDigest"):
            raise JenkinsError("group_identity_mismatch", "Author identity differs from sealed source or coverage")
        if (coverage.get("sourceDigest") != source or coverage.get("coverageDigest") != identity["coverageDigest"]
                or digest({k: v for k, v in coverage.items() if k != "coverageDigest"}) != identity["coverageDigest"]):
            raise JenkinsError("group_coverage_mismatch", "Sealed coverage digest is corrupt")
        request = require_request(state, identity, [*paths, *_external_scope(manifest)],
                                  connection=connection, allow_cancelled=allow_cancelled)
        if "ownedPaths" in request["payload"] and _paths({"paths": request["payload"]["ownedPaths"]}, root) != paths:
            raise JenkinsError("request_identity_mismatch", "Original request scope differs from sealed ownership")
        from ..source.consumer_checkout import resolve_consumer_checkout
        context = repo_root or Path(root)
        selected, _ = resolve_consumer_checkout(state, connection, context, request["payload"],
                                                request["payload"], identity, source)
        if str(selected).casefold() != str(Path(root)).casefold():
            raise JenkinsError("group_repository_mismatch", "Author source differs from its registered checkout")
        from ..source.consumer_intake import _objects, _sealed_input
        _sealed_input(state, connection, selected, identity, source)
        _objects(dict(sealed), {})
    evidence = (state.get("sealed_cargo_evidence", source, connection=connection) or {}).get("payload", {})
    if evidence and (evidence.get("sourceDigest") != source
                     or evidence.get("repositoryId", supplied["repositoryId"]) != supplied["repositoryId"]):
        raise JenkinsError("cargo_evidence_source_mismatch", "Grouping Cargo evidence differs from its source")
    profile = (state.get("validation_profile", source, connection=connection) or {}).get("payload", {})
    environment = (state.get("environment_policy", "default", connection=connection) or
                   state.get("cargo_environment", "windows-native", connection=connection) or {}).get("payload", {})
    if strict and state.get("environment_policy", "default", connection=connection):
        from .planning import _cargo_evidence, _environment_policy
        environment_values, verified_policy, _ = _environment_policy(state, repo_root or Path(root))
        evidence = _cargo_evidence(state, source)
        if evidence.get("environmentPolicyDigest") != digest(verified_policy):
            raise JenkinsError("cargo_evidence_policy_mismatch", "Author Cargo evidence differs from verified environment policy")
        environment = {**verified_policy, "environment": environment_values}
    wrappers = {name: str(environment.get("environment", {}).get(name, ""))
                for name in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER")}
    trusted = {**sealed, **evidence, "baseHead": manifest.get("baseHead"),
               "profile": coverage.get("profile", sealed.get("profile", "dev")),
               "features": digest(sorted(coverage.get("features", []))),
               "wrapperDigest": digest(wrappers) if strict and environment else sealed.get("wrapperDigest", digest(wrappers))}
    values = {field: _text(trusted.get(field), field) for field in _IDENTITY_FIELDS}
    for field, value in values.items():
        if field in supplied and supplied[field] != value:
            raise JenkinsError("group_identity_mismatch", f"Caller cannot replace sealed {field}")
    for claim in sealed.get("claims", ()):
        if isinstance(claim, Mapping) and claim.get("beforeHash") and claim.get("currentBeforeHash"):
            if claim["beforeHash"] != claim["currentBeforeHash"]:
                raise JenkinsError("foreign_edit_detected", "Source changed after the sealed claim")
    result = {**values, "sessionId": supplied["sessionId"], "requestId": supplied["requestId"],
              "attemptId": owner, "generation": supplied["generation"], "sourceDigest": source,
              "coverageDigest": sealed.get("coverageDigest") or digest(coverage), "sealedInputRef": source,
              "paths": list(paths), "coverage": dict(coverage), "acceptanceState": "pending"}
    result["defaultFeatures"] = digest(bool(coverage.get("noDefaultFeatures", False)))
    if request:
        result.update(identity=identity, requestPayloadDigest=request["payloadDigest"],
                      authorizationSnapshot=request["authorizationSnapshot"],
                      environmentPolicyDigest=evidence.get("environmentPolicyDigest"),
                      resourceProfile=profile.get("profile", "default"))
    return result


@dataclass(frozen=True)
class GroupProjection:
    group_id: str
    status: str
    mode: str
    sessions: tuple[dict[str, Any], ...]
    joint_manifest_digest: str | None
    version: int
    joint_coverage_digest: str | None = None
    accepted_consumers: tuple[str, ...] = ()
    cancelled_consumers: tuple[str, ...] = ()
    consumption_refs: tuple[tuple[str, str], ...] = ()
    joint_receipt_ref: str | None = None

    def as_dict(self) -> dict[str, Any]:
        return {"groupId": self.group_id, "status": self.status, "mode": self.mode,
                "sessions": [dict(session) for session in self.sessions],
                "jointManifestDigest": self.joint_manifest_digest, "version": self.version,
                "unionSourceDigest": self.joint_manifest_digest, "unionCoverageDigest": self.joint_coverage_digest,
                "acceptedConsumers": list(self.accepted_consumers), "cancelledConsumers": list(self.cancelled_consumers),
                "consumptionRefs": dict(self.consumption_refs), "jointReceiptRef": self.joint_receipt_ref}


class SessionGroups:
    DOMAIN = "workflow_session_group"

    def __init__(self, state: State, *, repo_root: Path | None = None, strict: bool = False):
        self.state, self.repo_root, self.strict = state, repo_root, strict

    @staticmethod
    def _compatible(items: list[dict[str, Any]]) -> None:
        if not items or len(items) > MAX_GROUP_SESSIONS:
            raise JenkinsError("group_size_invalid", f"Groups require 1–{MAX_GROUP_SESSIONS} authors")
        if len({item["sessionId"] for item in items}) != len(items):
            raise JenkinsError("group_session_duplicate", "Each group author must have a distinct session")
        for field in (*_IDENTITY_FIELDS, "environmentPolicyDigest", "resourceProfile", "defaultFeatures"):
            if len({item.get(field) for item in items}) != 1:
                raise JenkinsError("group_identity_conflict", f"Sessions disagree on {field}")
        if len({item.get("identity", {}).get("stageImplementationDigest") for item in items}) != 1:
            raise JenkinsError("group_identity_conflict", "Sessions disagree on driver identity")

    def prepare(self, sessions: Iterable[Mapping[str, Any]], *, mode: str = "same-input",
                joint_validation: Mapping[str, Any] | None = None) -> GroupProjection:
        supplied = list(sessions)
        if not supplied or len(supplied) > MAX_GROUP_SESSIONS:
            raise JenkinsError("group_size_invalid", f"Groups require 1–{MAX_GROUP_SESSIONS} authors")
        if mode not in {"same-input", "joint"}:
            raise JenkinsError("group_mode_invalid", "mode must be same-input or joint")
        if mode == "joint" and len(supplied) < 2:
            raise JenkinsError("group_size_invalid", "Joint validation requires at least two authors")
        with self.state.transaction() as connection:
            items = sorted((_session(session, self.state, strict=self.strict, repo_root=self.repo_root,
                                      connection=connection) for session in supplied), key=lambda item: item["sessionId"])
            self._compatible(items)
            if mode == "same-input" and len({item["sourceDigest"] for item in items}) != 1:
                raise JenkinsError("group_input_conflict", "Same-input grouping requires one sealed input")
            if mode == "joint":
                from .joint import merge_joint_inputs
                joint = merge_joint_inputs(self.state, items, connection=connection)
                joint_digest, coverage_digest = joint["sourceDigest"], joint["coverageDigest"]
                status = "pending-joint-validation"
            else:
                joint_digest, coverage_digest, status = None, None, "ready"
            payload = {"schemaVersion": 2, "mode": mode, "status": status,
                       "identity": {field: items[0][field] for field in _IDENTITY_FIELDS}, "sessions": items,
                       "jointManifestDigest": joint_digest, "jointCoverageDigest": coverage_digest,
                       "acceptedConsumers": [], "cancelledConsumers": [], "consumptionRefs": {},
                       "generation": max(item["generation"] for item in items)}
            group_id = digest({"mode": mode, "identity": payload["identity"], "sessions": items,
                               "jointManifestDigest": joint_digest})
            old = self.state.get(self.DOMAIN, group_id, connection=connection)
            record = old or self.state.put(self.DOMAIN, group_id, payload, expected_version=0, connection=connection)
        if joint_validation:
            return self.accept_joint(group_id, joint_validation, expected_version=record["version"])
        return self._projection(record)

    def record(self, group_id: str, *, connection=None) -> dict:
        record = self.state.get(self.DOMAIN, _text(group_id, "groupId"), connection=connection)
        if not record:
            raise JenkinsError("group_missing", "Session group does not exist")
        return record

    def observe(self, group_id: str) -> GroupProjection:
        return self._projection(self.record(group_id))

    @staticmethod
    def _version(record: dict, expected_version: int | None) -> None:
        if expected_version is not None and record["version"] != expected_version:
            raise JenkinsError("group_version_conflict", "Group changed before the requested operation", retryable=True)

    def seal_joint(self, group_id: str, *, session_id: str, owner: str,
                   expected_version: int | None = None, identity: Mapping | None = None) -> dict:
        from .joint import seal_joint_input
        with self.state.transaction() as connection:
            record = self.record(group_id, connection=connection)
            if record["payload"]["mode"] != "joint":
                raise JenkinsError("joint_group_required", "Combined sealing requires a joint group")
            if record["payload"].get("status") == "cancelled":
                raise JenkinsError("group_cancelled", "Cancelled groups cannot seal a new consumer")
            old = record["payload"].get("jointConsumer")
            consumer = dict(identity) if identity else {"sessionId": session_id, "attemptId": owner}
            if self.strict:
                if not identity or identity.get("sessionId") != session_id or identity.get("attemptId") != owner:
                    raise JenkinsError("joint_consumer_mismatch", "Union sealing requires its full registered aggregator identity")
                joint = self.state.get("joint_input", record["payload"]["jointManifestDigest"], connection=connection)
                if not joint:
                    raise JenkinsError("joint_input_missing", "Frozen union input is unavailable")
                require_request(self.state, identity, [*joint["payload"]["ownedPaths"],
                    *_external_scope(joint["payload"]["sourceManifest"])], connection=connection)
                for author in record["payload"]["sessions"]:
                    fresh = _session({"identity": author["identity"]}, self.state, strict=True, repo_root=self.repo_root,
                                     connection=connection, allow_cancelled=True)
                    if fresh != author:
                        raise JenkinsError("group_author_mismatch", "Frozen author source binding changed before union sealing")
            if old and old != consumer:
                raise JenkinsError("joint_consumer_conflict", "Group sealing cannot replace its registered union consumer")
            if not old:
                self._version(record, expected_version)
            result = seal_joint_input(self.state, self._projection(record), session_id=session_id,
                                       owner=owner, identity=identity, connection=connection)
            if not old:
                record = self.state.put(self.DOMAIN, group_id, {**record["payload"], "jointConsumer": consumer,
                    "jointPreparationStatus": "metadata-pending"}, expected_version=record["version"], connection=connection)
            return {**result, "groupId": group_id, "version": record["version"]}

    def accept_joint(self, group_id: str, reference: Mapping, *, expected_version: int | None = None) -> GroupProjection:
        from .joint import verify_joint_acceptance
        record = self.record(group_id)
        payload = record["payload"]
        if payload["mode"] != "joint":
            raise JenkinsError("joint_group_required", "Acceptance requires a joint group")
        if payload["status"] == "cancelled":
            raise JenkinsError("group_cancelled", "Cancelled group cannot acquire acceptance")
        joint = self.state.get("joint_input", payload["jointManifestDigest"])
        if not joint:
            raise JenkinsError("joint_input_missing", "Frozen joint input is unavailable")
        verified = verify_joint_acceptance(self.state, joint["payload"], reference, repo_root=self.repo_root,
                                           consumer_identity=payload.get("jointConsumer"))
        with self.state.transaction() as connection:
            current = self.record(group_id, connection=connection)
            if self.strict:
                require_request(self.state, payload["jointConsumer"], [*joint["payload"]["ownedPaths"],
                    *_external_scope(joint["payload"]["sourceManifest"])], connection=connection)
                for author in current["payload"]["sessions"]:
                    if author["sessionId"] not in current["payload"].get("cancelledConsumers", []):
                        fresh = _session({"identity": author["identity"]}, self.state, strict=True,
                                         repo_root=self.repo_root, connection=connection, allow_cancelled=True)
                        if fresh != author:
                            raise JenkinsError("group_author_mismatch", "Frozen author source binding changed before union acceptance")
            old_ref = current["payload"].get("jointReceiptRef")
            if old_ref:
                if old_ref != verified["receiptRef"] or current["payload"].get("jointFlowId") != verified["flowId"]:
                    raise JenkinsError("joint_acceptance_conflict", "Group acceptance cannot replace its original receipt")
                return self._projection(current)
            self._version(current, expected_version)
            if current["payload"].get("jointConsumer") != payload.get("jointConsumer") or current["payload"]["status"] == "cancelled":
                raise JenkinsError("group_version_conflict", "Group changed during acceptance", retryable=True)
            updated = {**current["payload"], "status": "ready", "jointReceiptRef": verified["receiptRef"],
                       "jointFlowId": verified["flowId"], "jointEvidenceDigest": verified["evidenceDigest"]}
            return self._projection(self.state.put(self.DOMAIN, group_id, updated,
                expected_version=current["version"], connection=connection))

    def consume_joint(self, group_id: str, identity: Mapping, *, expected_version: int | None = None) -> dict:
        from .joint import consume_joint_acceptance
        return consume_joint_acceptance(self, group_id, identity, expected_version=expected_version)

    def cancel(self, group_id: str, session_id: str, generation: int, *, expected_version: int | None = None) -> GroupProjection:
        with self.state.transaction() as connection:
            record = self.record(group_id, connection=connection)
            payload = record["payload"]
            found = next((item for item in payload["sessions"] if item["sessionId"] == session_id), None)
            if found is None:
                raise JenkinsError("group_session_missing", "Session is not part of this group")
            if type(generation) is not int or found["generation"] != generation:
                raise JenkinsError("group_generation_mismatch", "Late cancellation targets another generation")
            if self.strict:
                require_request(self.state, found["identity"], found["paths"],
                                connection=connection, allow_cancelled=True)
            if session_id in payload.get("cancelledConsumers", []):
                return self._projection(record)
            if session_id in payload.get("acceptedConsumers", []):
                raise JenkinsError("group_consumer_accepted", "Consumption is immutable after union acceptance")
            self._version(record, expected_version)
            cancelled = sorted([*payload.get("cancelledConsumers", []), session_id])
            updated = {**payload, "cancelledConsumers": cancelled}
            if len(cancelled) == len(payload["sessions"]):
                updated["status"] = "cancelled"
            return self._projection(self.state.put(self.DOMAIN, group_id, updated,
                expected_version=record["version"], connection=connection))

    @staticmethod
    def _projection(record: Mapping[str, Any]) -> GroupProjection:
        payload = record["payload"]
        return GroupProjection(record["key"], payload["status"], payload["mode"], tuple(payload["sessions"]),
            payload.get("jointManifestDigest"), record["version"], payload.get("jointCoverageDigest"),
            tuple(payload.get("acceptedConsumers", [])), tuple(payload.get("cancelledConsumers", [])),
            tuple(sorted(payload.get("consumptionRefs", {}).items())), payload.get("jointReceiptRef"))


__all__ = ["GroupProjection", "SessionGroups", "MAX_GROUP_SESSIONS"]
