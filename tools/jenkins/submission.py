"""Idempotent Jenkins flow submission frontend.

This module records the immutable request in State before contacting Jenkins.  It
is deliberately a small adapter: formal acceptance still belongs to State and
the Jenkins receipts, never to HTTP 201 or a build number alone.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Callable, Mapping
from urllib.parse import urlencode
from urllib.request import Request, urlopen

from .contracts import JenkinsError, identifier
from .state import State

DEFAULT_BASE_URL = "http://127.0.0.1:18080"
DEFAULT_JOB = "zircon-flow"

_PARAMETER_NAMES = {
    "repositoryId": "REPOSITORY_ID", "sessionId": "SESSION_ID", "requestId": "REQUEST_ID",
    "sourceRef": "SEALED_INPUT_REF", "sourceDigest": "SOURCE_INPUT_DIGEST",
    "sealedInputRef": "SEALED_INPUT_REF", "sourceInputDigest": "SOURCE_INPUT_DIGEST",
    "patchRef": "PATCH_REQUEST_REF", "patchRequestRef": "PATCH_REQUEST_REF",
    "coverageRef": "COVERAGE_REF", "coverageDigest": "COVERAGE_DIGEST",
    "recipeRef": "RECIPE_REF", "generation": "GENERATION", "attemptId": "ATTEMPT_ID",
    "executionId": "EXECUTION_ID", "buildRoot": "BUILD_ROOT",
}


def _submission_fields(payload: Mapping[str, object]) -> dict[str, str]:
    identity_keys = ("repositoryId", "sessionId", "requestId", "sourceRef", "sourceDigest",
                     "coverageRef", "coverageDigest")
    identity = {_PARAMETER_NAMES[key]: str(payload[key]) for key in identity_keys if key in payload}
    for key in ("patchRef", "patchRequestRef"):
        if key in payload:
            identity["PATCH_REQUEST_REF"] = str(payload[key])
    fields = dict(identity)
    parameters = payload.get("parameters", {})
    if not isinstance(parameters, Mapping):
        raise JenkinsError("submission_parameters_invalid", "Jenkins parameters must be an object")
    for key, value in parameters.items():
        name = _PARAMETER_NAMES.get(str(key), str(key))
        if name in identity and str(value) != identity[name]:
            raise JenkinsError("submission_identity_override", "Jenkins parameters cannot override request identity")
        fields[name] = str(value)
    return fields


def _require_registered_identity(recorded: Mapping[str, object], payload: Mapping[str, object]) -> None:
    bound = recorded.get("payload", {})
    bindings = {
        "repositoryId": ("repositoryId",), "sessionId": ("sessionId",), "requestId": ("requestId",),
        "sourceRef": ("sealedInputRef", "sourceRef"),
        "sourceDigest": ("sourceInputDigest", "sourceDigest", "sealedInputRef"),
        "coverageRef": ("coverageRef", "coverageDigest"),
        "coverageDigest": ("coverageDigest", "coverageRef"),
    }
    if not isinstance(bound, Mapping):
        raise JenkinsError("submission_identity_mismatch", "Registered request identity is unavailable")
    for key, aliases in bindings.items():
        value = next((bound.get(name) for name in aliases if bound.get(name) not in (None, "")), None)
        if value is None or str(value) != str(payload.get(key)):
            raise JenkinsError("submission_identity_mismatch", "Submission identity differs from registered request")


def request_payload(*, repository_id: str, session_id: str, request_id: str,
                    source_ref: str, source_digest: str, coverage_ref: str,
                    coverage_digest: str, parameters: Mapping[str, str] | None = None) -> dict:
    """Build a bounded, immutable request payload for State.submit_request."""
    for value, name in ((repository_id, "repositoryId"), (session_id, "sessionId"),
                        (request_id, "requestId"), (source_ref, "sourceRef"),
                        (coverage_ref, "coverageRef")):
        identifier(value, name)
    if not source_digest or not coverage_digest:
        raise JenkinsError("identity_incomplete", "source and coverage digests are required")
    params = {str(k): str(v) for k, v in (parameters or {}).items()}
    return {"repositoryId": repository_id, "sessionId": session_id, "requestId": request_id,
            "sourceRef": source_ref, "sourceDigest": source_digest,
            "coverageRef": coverage_ref, "coverageDigest": coverage_digest,
            "parameters": params}


def _marker_key(payload: Mapping[str, object]) -> str:
    submission = payload.get("parameters", {}).get("submissionRequestId", "flow") if isinstance(payload.get("parameters"), Mapping) else "flow"
    return f"{payload['repositoryId']}:{payload['sessionId']}:{payload['requestId']}:{submission}"


def _claim_dispatch(state: State, key: str, payload: Mapping[str, object]) -> dict:
    """Claim the external Jenkins side effect under the State write lock."""
    from .contracts import digest
    expected = digest(dict(payload))
    with state.transaction() as connection:
        prior = state.get("jenkins_submission", key, connection=connection)
        if prior is not None:
            old = prior["payload"]
            if old.get("payloadDigest") != expected:
                raise JenkinsError("submission_payload_mismatch", "Submission identity already has a different payload")
            if old.get("delivery") in {"submitted", "accepted", "completed"}:
                return {"claim": False, "record": prior}
            if old.get("delivery") in {"dispatching", "unknown"}:
                raise JenkinsError("delivery_unknown", "Jenkins delivery must be reconciled before retry", retryable=True)
        value = {**dict(payload), "payloadDigest": expected, "delivery": "dispatching",
                 "queueUrl": None, "buildRef": None, "formalAcceptance": False}
        return {"claim": True, "record": state.put("jenkins_submission", key, value, connection=connection)}


def _finish_dispatch(state: State, key: str, version: int, value: dict) -> dict:
    with state.transaction() as connection:
        current = state.get("jenkins_submission", key, connection=connection)
        if current is None:
            raise JenkinsError("submission_missing", "Submission claim disappeared")
        return state.put("jenkins_submission", key, value, expected_version=version, connection=connection)


def submit(state: State, payload: dict, *, base_url: str = DEFAULT_BASE_URL,
           job: str = DEFAULT_JOB, token: str | None = None,
           transport: Callable[[Request], object] | None = None,
           manager=None) -> dict:
    """Persist once, then submit once; return request/queue/build references.

    A pre-existing request is returned without replaying the side effect.  A
    transport may be injected for tests; no credential is included in output.
    """
    required = ("repositoryId", "sessionId", "requestId", "sourceRef", "sourceDigest", "coverageRef", "coverageDigest")
    if any(not payload.get(k) for k in required):
        raise JenkinsError("identity_incomplete", "Submission requires complete source and coverage identity")
    fields = _submission_fields(payload)
    try:
        recorded = state.submit_request(payload)
    except JenkinsError as error:
        if error.code != "request_payload_mismatch" or not payload.get("parameters", {}).get("submissionRequestId"):
            raise
        recorded = state.get_request(payload["repositoryId"], payload["sessionId"], payload["requestId"])
        if recorded is None:
            raise
        _require_registered_identity(recorded, payload)
    return dispatch_registered(state, payload, fields, registered_request=recorded,
                               base_url=base_url, job=job, token=token,
                               transport=transport, manager=manager)


def dispatch_registered(state: State, payload: Mapping[str, object], fields: Mapping[str, str], *,
                        registered_request: Mapping[str, object] | None = None,
                        base_url: str = DEFAULT_BASE_URL, job: str = DEFAULT_JOB,
                        token: str | None = None,
                        transport: Callable[[Request], object] | None = None,
                        manager=None) -> dict:
    """Dispatch an already registered intent exactly once.

    ``registered_request`` is supplied by the caller after it has checked the
    authoritative patch intent/request.  This deliberately does not call
    :meth:`State.submit_request`: a patch request cannot receive its final
    source identity until Jenkins applies and seals the patch.  The external
    marker still uses the same request/session/flow key and therefore protects
    both ordinary source submissions and raw patch intent dispatches.
    """
    required = ("repositoryId", "sessionId", "requestId")
    if any(not payload.get(key) for key in required):
        raise JenkinsError("identity_incomplete", "Registered submission identity is incomplete")
    if not isinstance(fields, Mapping):
        raise JenkinsError("submission_parameters_invalid", "Jenkins parameters must be an object")
    fields = {str(key): str(value) for key, value in fields.items()}
    recorded = registered_request if registered_request is not None else {"payload": dict(payload)}
    marker_key = _marker_key(payload)
    claim = _claim_dispatch(state, marker_key, payload)
    if not claim["claim"]:
        return {"request": recorded, **claim["record"]["payload"], "reused": True}
    body = urlencode(fields).encode()
    queue_url = None
    try:
        if manager is not None:
            status, _ = manager._request(f"job/{job}/buildWithParameters", method="POST", data=body,
                                         headers={"Content-Type": "application/x-www-form-urlencoded"})
        else:
            request = Request(f"{base_url.rstrip('/')}/job/{job}/buildWithParameters", data=body, method="POST")
            if token:
                import base64
                request.add_header("Authorization", "Basic " + base64.b64encode(f"admin:{token}".encode()).decode())
            response = (transport or urlopen)(request)
            queue_url = response.headers.get("Location") if hasattr(response, "headers") else None
            status = getattr(response, "status", None)
    except Exception as error:
        unknown = dict(claim["record"]["payload"])
        unknown.update({"delivery": "unknown", "deliveryError": type(error).__name__})
        _finish_dispatch(state, marker_key, claim["record"]["version"], unknown)
        raise JenkinsError("delivery_unknown", "Jenkins submission delivery is unknown; reconcile before retry", retryable=True) from error
    if isinstance(status, bool) or not isinstance(status, int) or not 200 <= status < 300:
        unknown = dict(claim["record"]["payload"])
        unknown.update({"delivery": "unknown", "deliveryError": "http_status_invalid"})
        _finish_dispatch(state, marker_key, claim["record"]["version"], unknown)
        raise JenkinsError("delivery_unknown", "Jenkins submission HTTP status is not a successful 2xx response; reconcile before retry", retryable=True)
    value = dict(claim["record"]["payload"])
    value.update({"delivery": "submitted", "status": status, "queueUrl": queue_url,
                  "buildRef": None, "formalAcceptance": False})
    finished = _finish_dispatch(state, marker_key, claim["record"]["version"], value)
    return {"request": recorded, **finished["payload"], "reused": False}


def reconcile(state: State, payload: Mapping[str, object], *, queue_url: str | None = None,
               build_ref: str | None = None, delivery: str = "submitted") -> dict:
    """Record an API/operator reconciliation without issuing another build."""
    from .contracts import digest
    key = _marker_key(payload)
    current = state.get("jenkins_submission", key)
    if current is None:
        raise JenkinsError("submission_missing", "No Jenkins submission exists for this request")
    if current["payload"].get("payloadDigest") != digest(dict(payload)):
        raise JenkinsError("submission_payload_mismatch", "Reconciliation identity differs from submitted payload")
    if delivery not in {"submitted", "accepted", "completed", "unknown"}:
        raise JenkinsError("delivery_state_invalid", "Invalid Jenkins delivery state")
    value = dict(current["payload"])
    value.update({"delivery": delivery, "queueUrl": queue_url or value.get("queueUrl"),
                  "buildRef": build_ref or value.get("buildRef")})
    return _finish_dispatch(state, key, current["version"], value)["payload"]


def load_payload(path: str | Path) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))
