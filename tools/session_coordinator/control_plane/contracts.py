from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from typing import Any

from ..models import CoordinatorError


CONTROL_API_VERSION = 1


@dataclass(frozen=True, slots=True)
class ControlResponse:
    status: int
    data: dict[str, Any] | None = None
    error: dict[str, Any] | None = None
    headers: dict[str, str] = field(default_factory=dict)

    def body(self, correlation_id: str) -> dict[str, Any]:
        meta = {"apiVersion": CONTROL_API_VERSION, "correlationId": correlation_id}
        if self.error is not None:
            return {"ok": False, "error": self.error, "meta": meta}
        return {"ok": True, "data": self.data or {}, "meta": meta}


def new_correlation_id() -> str:
    return str(uuid.uuid4())


def error_payload(error: CoordinatorError, *, retryable: bool = False) -> dict[str, Any]:
    return {
        "code": error.code,
        "message": error.message,
        "retryable": retryable,
        "details": error.details,
    }
