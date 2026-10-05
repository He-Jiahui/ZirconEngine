"""Small Jenkins-facing orchestration adapter.

The support state/CLI modules remain authoritative.  This module only builds
validated stage plans and translates them into short control operations.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

from ..contracts import JenkinsError, digest

FLOW_JOB = "zircon-flow"
EXECUTION_JOB = "zircon-execution"
MAINTENANCE_JOB = "zircon-maintenance"

TEMPLATES = {
    "comments_only": ("patch", "syntax", "format", "acceptance", "commit"),
    "module_unit": ("patch", "syntax", "format", "build", "unit_test", "acceptance", "commit", "notify", "gc"),
    "cross_module": ("patch", "syntax", "format", "build", "unit_test", "integration_test", "acceptance", "commit", "notify", "gc"),
    "failure_repair": ("patch", "syntax", "format", "build", "unit_test", "regression_test", "acceptance", "commit", "notify", "gc"),
    "cache_maintenance": ("gc",),
}

@dataclass(frozen=True)
class StagePlan:
    template: str
    stages: tuple[str, ...]
    identity: Mapping[str, Any]
    execution_id: str | None = None


def build_dag(template: str, identity: Mapping[str, Any], *, execution_id: str | None = None) -> StagePlan:
    if template not in TEMPLATES:
        raise ValueError(f"unknown workflow template: {template}")
    required = {"sessionId", "requestId", "attemptId", "generation", "sourceInputDigest", "coverageDigest", "stageImplementationDigest"}
    missing = sorted(required - set(identity))
    if missing:
        raise ValueError("missing workflow identity: " + ",".join(missing))
    return StagePlan(template, TEMPLATES[template], dict(identity), execution_id)


def validate_pipeline_identity(identity: Mapping[str, Any], *, build_root: str | None = None) -> dict[str, Any]:
    """Validate the immutable identity passed from a Jenkins job.

    This adapter does not become a second State authority: it only rejects
    incomplete or conflicting request data before a control call is made.
    The authoritative build root is supplied by State; an explicit value is
    carried through unchanged for the dispatcher to verify physically.
    """
    required = ("repositoryId", "sessionId", "requestId", "attemptId", "generation",
                "sourceInputDigest", "coverageDigest", "stageImplementationDigest")
    missing = [name for name in required if identity.get(name) in (None, "")]
    if missing:
        raise JenkinsError("pipeline_identity_missing", "Pipeline identity is incomplete", details={"fields": missing})
    generation = identity.get("generation")
    if isinstance(generation, bool) or not isinstance(generation, int) or generation < 1:
        raise JenkinsError("pipeline_generation_invalid", "Pipeline generation must be a positive integer")
    normalized = dict(identity)
    if build_root is not None:
        if not isinstance(build_root, str) or not build_root:
            raise JenkinsError("build_root_invalid", "Pipeline buildRoot must be a non-empty string")
        normalized["buildRoot"] = build_root
    normalized["identityDigest"] = digest({k: normalized[k] for k in sorted(normalized) if k != "identityDigest"})
    return normalized


def run_control(operation: str, payload: Mapping[str, Any], *, cli: Any) -> Mapping[str, Any]:
    """Invoke an existing support CLI function without creating local state.

    ``cli`` is injected by the root dispatcher; this prevents Pipeline code
    from becoming a second authority.  The adapter retries only operations
    explicitly marked retryable by the authoritative response.
    """
    method = getattr(cli, operation, None)
    if method is None and "-" in operation:
        method = getattr(cli, operation.replace("-", "_"), None)
    if method is None or not callable(method):
        raise ValueError(f"unsupported Jenkins control operation: {operation}")
    result = method(dict(payload))
    if not isinstance(result, Mapping):
        raise TypeError("Jenkins control operation must return a mapping")
    return result


def submit_execution(cli: Any, identity: Mapping[str, Any], *, template: str,
                     sealed_input_ref: str, build_root: str) -> Mapping[str, Any]:
    """Submit one execution through the authoritative flow adapter.

    Jenkins parameters are treated as data.  The returned ``recipeRef`` and
    ``executionId`` are the only values a later execution stage may use.
    """
    validated = validate_pipeline_identity(identity, build_root=build_root)
    if not sealed_input_ref:
        raise JenkinsError("sealed_input_missing", "Execution requires a sealed input reference")
    result = run_control("submit-execution", {
        "identity": validated, "template": template,
        "sealedInputRef": sealed_input_ref, "buildRoot": build_root,
    }, cli=cli)
    for field in ("executionId", "recipeRef"):
        if not result.get(field):
            raise JenkinsError("execution_response_incomplete", f"Control response lacks {field}")
    return result
