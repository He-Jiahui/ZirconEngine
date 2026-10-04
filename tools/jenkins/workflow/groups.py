"""Persistent multi-session grouping for the Jenkins workflow.

Grouping is deliberately a projection over sealed inputs.  It never changes a
request's acceptance state and it never treats a joint run as proof for each
consumer.  The latter distinction is important when one consumer is cancelled
or when a later session observes a different source generation.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Iterable, Mapping

from ..contracts import JenkinsError, canonical_json, digest, identifier
from ..state import State
from ..source import require_sealed_consumer, compute_source_digest


_IDENTITY_FIELDS = (
    "repositoryId", "baseHead", "toolchainDigest", "lockDigest",
    "compilerIdentity", "wrapperDigest", "profile", "features", "buildRoot",
)


def _text(value: Any, name: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise JenkinsError("group_identity_missing", f"{name} is required")
    return value


def _paths(session: Mapping[str, Any]) -> tuple[str, ...]:
    values = session.get("paths")
    if values is None:
        values = [c.get("path") for c in session.get("claims", []) if isinstance(c, Mapping)]
    if not isinstance(values, (list, tuple)) or not values:
        raise JenkinsError("group_paths_missing", "a session must carry sealed source paths")
    out = tuple(sorted({_text(p, "path").replace("\\", "/") for p in values}))
    for left in out:
        for right in out:
            if left != right and (right.startswith(left + "/") or left.startswith(right + "/")):
                raise JenkinsError("group_path_overlap", "one session contains overlapping paths")
    return out


def _sealed(state: State, session: Mapping[str, Any]) -> Mapping[str, Any]:
    ref = session.get("sealedInputRef") or session.get("sourceDigest")
    _text(ref, "sealedInputRef")
    record = state.get("sealed_input", ref)
    if not record or not isinstance(record.get("payload"), Mapping):
        raise JenkinsError("sealed_input_missing", "grouping requires a State sealed_input record")
    payload = record["payload"]
    if payload.get("status") != "sealed" or payload.get("sourceDigest") != ref:
        raise JenkinsError("sealed_input_untrusted", "sealed input is not an authoritative sealed record")
    manifest = payload.get("sourceManifest") or payload.get("manifest")
    if not isinstance(manifest, Mapping) or compute_source_digest(manifest) != ref:
        raise JenkinsError("sealed_input_corrupt", "grouping source manifest differs from its digest")
    return payload


def _session(session: Mapping[str, Any], state: State) -> dict[str, Any]:
    for field in ("repositoryId", "sessionId", "requestId", "generation"):
        if field not in session:
            raise JenkinsError("group_identity_missing", f"{field} is required")
    identifier(str(session["sessionId"]), "sessionId")
    identifier(str(session["requestId"]), "requestId")
    generation = session["generation"]
    if not isinstance(generation, int) or generation < 1:
        raise JenkinsError("group_generation_invalid", "generation must be a positive integer")
    sealed = _sealed(state, session)
    if session["repositoryId"] != sealed.get("repositoryId"):
        raise JenkinsError("group_repository_mismatch", "session belongs to another repository")
    owner = session.get("attemptId") or session.get("owner")
    require_sealed_consumer(state, repository_id_value=session["repositoryId"],
        session_id=session["sessionId"], owner=_text(owner, "attemptId"),
        source_digest=sealed["sourceDigest"], manifest_digest=sealed["sourceDigest"])
    source_digest = sealed["sourceDigest"]
    if session.get("sourceDigest", source_digest) != source_digest:
        raise JenkinsError("group_identity_mismatch", "session sourceDigest differs from sealed input")
    coverage = sealed.get("coverage")
    if not isinstance(coverage, Mapping) or coverage.get("sourceDigest") not in (None, source_digest):
        raise JenkinsError("group_coverage_mismatch", "sealed coverage is not bound to the source")
    manifest = sealed.get("sourceManifest") or sealed["manifest"]
    evidence = (state.get("sealed_cargo_evidence", source_digest) or {}).get("payload", {})
    profile = (state.get("validation_profile", source_digest) or {}).get("payload", {})
    environment = (state.get("cargo_environment", "windows-native") or {}).get("payload", {})
    trusted = {**sealed, **evidence, "baseHead": manifest.get("baseHead"),
               "profile": profile.get("profile", sealed.get("profile", "dev")),
               "features": digest(coverage.get("features", [])),
               "wrapperDigest": sealed.get("wrapperDigest", digest(environment.get("environmentDigest", "")))}
    values = {field: trusted.get(field) for field in _IDENTITY_FIELDS}
    for field, value in values.items():
        _text(value, field)
        if field in session and session[field] != value:
            raise JenkinsError("group_identity_mismatch", f"caller cannot replace sealed {field}")
        values[field] = value
    paths = _paths({"paths": sealed.get("ownedPaths") or [c["path"] for c in sealed.get("claims", [])]})
    if "paths" in session and _paths(session) != paths:
        raise JenkinsError("group_paths_mismatch", "caller cannot replace sealed source ownership")
    # A caller may provide a before hash snapshot, but cannot replace the
    # authoritative hash stored by the claim.
    for claim in sealed.get("claims", ()):
        if isinstance(claim, Mapping) and claim.get("beforeHash") and claim.get("currentBeforeHash"):
            if claim["beforeHash"] != claim["currentBeforeHash"]:
                raise JenkinsError("foreign_edit_detected", "source changed after the sealed claim")
    return {
        **values,
        "sessionId": str(session["sessionId"]),
        "requestId": str(session["requestId"]),
        "attemptId": owner,
        "generation": generation,
        "sourceDigest": source_digest,
        "coverageDigest": sealed.get("coverageDigest") or digest(coverage or {}),
        "sealedInputRef": source_digest,
        "paths": list(paths),
        "coverage": dict(coverage or {}),
        "acceptanceState": "pending",
    }


@dataclass(frozen=True)
class GroupProjection:
    group_id: str
    status: str
    mode: str
    sessions: tuple[dict[str, Any], ...]
    joint_manifest_digest: str | None
    version: int

    def as_dict(self) -> dict[str, Any]:
        return {"groupId": self.group_id, "status": self.status, "mode": self.mode,
                "sessions": [dict(s) for s in self.sessions],
                "jointManifestDigest": self.joint_manifest_digest, "version": self.version}


class SessionGroups:
    """CAS-backed grouping authority used by workflow/execution projections."""

    DOMAIN = "workflow_session_group"

    def __init__(self, state: State):
        self.state = state

    @staticmethod
    def _compatible(items: list[dict[str, Any]]) -> None:
        if not items:
            raise JenkinsError("group_empty", "at least one session is required")
        for field in _IDENTITY_FIELDS:
            if len({item[field] for item in items}) != 1:
                raise JenkinsError("group_identity_conflict", f"sessions disagree on {field}")

    def prepare(self, sessions: Iterable[Mapping[str, Any]], *, mode: str = "same-input",
                joint_validation: Mapping[str, Any] | None = None) -> GroupProjection:
        items = [_session(s, self.state) for s in sessions]
        if mode not in {"same-input", "joint"}:
            raise JenkinsError("group_mode_invalid", "mode must be same-input or joint")
        self._compatible(items)
        if mode == "same-input" and len({i["sourceDigest"] for i in items}) != 1:
            raise JenkinsError("group_input_conflict", "same-input grouping requires one sealed input")
        paths: dict[str, str] = {}
        same_input = len({i["sourceDigest"] for i in items}) == 1
        for item in items:
            for path in item["paths"]:
                if path in paths and (mode == "joint" or not same_input) and paths[path] != item["sessionId"]:
                    raise JenkinsError("group_path_conflict", "sessions claim the same source path")
                paths[path] = item["sessionId"]
        if mode == "joint":
            from .joint import merge_joint_inputs, verify_joint_acceptance
            joint = merge_joint_inputs(self.state, items)
            joint_digest = joint["sourceDigest"]
            status = "ready" if verify_joint_acceptance(self.state, joint, joint_validation) else "pending-joint-validation"
        else:
            joint_digest, status = None, "ready"
        payload = {"mode": mode, "status": status,
                   "identity": {field: items[0][field] for field in _IDENTITY_FIELDS},
                   "sessions": items, "jointManifestDigest": joint_digest,
                   "acceptedConsumers": [], "cancelledConsumers": [],
                   "generation": max(i["generation"] for i in items)}
        group_id = digest({"mode": mode, "identity": payload["identity"], "sessions": items,
                           "jointManifestDigest": joint_digest})
        old = self.state.get(self.DOMAIN, group_id)
        if old:
            if status == "ready" and old["payload"].get("status") == "pending-joint-validation":
                old = self.state.put(self.DOMAIN, group_id, {**old["payload"], "status": "ready"}, expected_version=old["version"])
            return self._projection(old)
        return self._projection(self.state.put(self.DOMAIN, group_id, payload, expected_version=0))

    def seal_joint(self, group_id: str, *, session_id: str, owner: str) -> dict:
        from .joint import seal_joint_input
        group = self.observe(group_id)
        if group.mode != "joint":
            raise JenkinsError("joint_group_required", "combined sealing requires a joint group")
        return seal_joint_input(self.state, group, session_id=session_id, owner=owner)

    def observe(self, group_id: str) -> GroupProjection:
        record = self.state.get(self.DOMAIN, _text(group_id, "groupId"))
        if not record:
            raise JenkinsError("group_missing", "session group does not exist")
        return self._projection(record)

    def cancel(self, group_id: str, session_id: str, generation: int) -> GroupProjection:
        record = self.state.get(self.DOMAIN, _text(group_id, "groupId"))
        if not record:
            raise JenkinsError("group_missing", "session group does not exist")
        payload = record["payload"]
        sessions = payload["sessions"]
        found = next((s for s in sessions if s["sessionId"] == session_id), None)
        if found is None:
            raise JenkinsError("group_session_missing", "session is not part of this group")
        if found["generation"] != generation:
            raise JenkinsError("group_generation_mismatch", "late cancellation targets an old generation")
        if session_id not in payload["cancelledConsumers"]:
            payload = {**payload, "cancelledConsumers": [*payload["cancelledConsumers"], session_id]}
            active = [s for s in sessions if s["sessionId"] not in payload["cancelledConsumers"]]
            payload["status"] = "cancelled" if not active else payload["status"]
            record = self.state.put(self.DOMAIN, group_id, payload, expected_version=record["version"])
        return self._projection(record)

    @staticmethod
    def _projection(record: Mapping[str, Any]) -> GroupProjection:
        p = record["payload"]
        return GroupProjection(record["key"], p["status"], p["mode"],
                               tuple(p["sessions"]), p.get("jointManifestDigest"), record["version"])


__all__ = ["GroupProjection", "SessionGroups"]
