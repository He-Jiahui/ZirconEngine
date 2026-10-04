"""Small Jenkins-facing orchestration adapter.

The support state/CLI modules remain authoritative.  This module only builds
validated stage plans and translates them into short control operations.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

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


def run_control(operation: str, payload: Mapping[str, Any], *, cli: Any) -> Mapping[str, Any]:
    """Invoke an existing support CLI function without creating local state.

    ``cli`` is injected by the root dispatcher; this prevents Pipeline code
    from becoming a second authority.  The adapter retries only operations
    explicitly marked retryable by the authoritative response.
    """
    method = getattr(cli, operation, None)
    if method is None or not callable(method):
        raise ValueError(f"unsupported Jenkins control operation: {operation}")
    result = method(dict(payload))
    if not isinstance(result, Mapping):
        raise TypeError("Jenkins control operation must return a mapping")
    return result
