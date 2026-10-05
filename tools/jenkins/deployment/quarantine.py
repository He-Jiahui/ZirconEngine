"""Fail-closed admission for a fresh execution broker generation.

This module is deliberately side-effect free. It classifies durable launch
records for a future broker integration; it never rewrites historical rows,
releases reservations, or infers termination from a missing process.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping


@dataclass(frozen=True)
class QuarantineBinding:
    """The exact lifecycle and sealed-driver identity accepted by a broker."""

    operation_id: str
    generation: str
    driver_digest: str

    def __post_init__(self) -> None:
        if not all(isinstance(value, str) and value for value in
                   (self.operation_id, self.generation, self.driver_digest)):
            raise ValueError("quarantine binding fields must be non-empty strings")


def matches_binding(payload: Mapping[str, object], execution: Mapping[str, object] | None,
                    driver_digest: str, operation_id: str) -> bool:
    """Strictly match an existing launch to one runtime operation."""
    if not isinstance(execution, Mapping):
        return False
    driver = execution.get("driver")
    if not isinstance(driver, Mapping):
        return False
    required = (
        payload.get("operationId") == operation_id,
        execution.get("operationId") == operation_id,
        payload.get("generation") is not None,
        payload.get("generation") == execution.get("generation"),
        payload.get("driverDigest") == driver_digest,
        execution.get("driverDigest") == driver_digest,
        driver.get("digest") == driver_digest,
        driver.get("runtimeOperationId") == operation_id,
        driver.get("generation") == execution.get("generation"),
    )
    return all(required)


def classify_launch(payload: Mapping[str, object], execution: Mapping[str, object] | None,
                    binding: QuarantineBinding) -> str:
    """Return ``eligible`` only for a fresh, fully bound launch.

    All other rows are ``preserve``. In particular, an old ``requested`` row
    must not be changed to ``failed`` merely because a new broker generation
    is running; reconciliation of that row belongs to its original authority.
    """
    if payload.get("status") != "requested" or not isinstance(execution, Mapping):
        return "preserve"
    driver = execution.get("driver")
    if not isinstance(driver, Mapping):
        return "preserve"
    return "eligible" if matches_binding(payload, execution, binding.driver_digest, binding.operation_id) \
        and payload.get("generation") == binding.generation else "preserve"


def classify_launches(rows: list[Mapping[str, object]], executions: Mapping[str, Mapping[str, object]],
                     binding: QuarantineBinding) -> dict[str, list[str]]:
    """Classify rows without mutating the durable state database."""
    result = {"eligible": [], "preserve": []}
    for row in rows:
        key = str(row.get("key", ""))
        payload = row.get("payload")
        if not isinstance(payload, Mapping):
            result["preserve"].append(key)
            continue
        execution_id = str(payload.get("executionId", key))
        outcome = classify_launch(payload, executions.get(execution_id), binding)
        result[outcome].append(key)
    return result
