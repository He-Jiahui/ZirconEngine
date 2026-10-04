"""Versioned identities and structured errors shared by every support module."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.:\-]{0,127}\Z")


class JenkinsError(RuntimeError):
    def __init__(self, code: str, message: str, *, retryable: bool = False,
                 details: dict | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.reason_code = code
        self.retryable = retryable
        self.details = details or {}


def canonical_json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value: Any) -> str:
    return hashlib.sha256(canonical_json(value)).hexdigest()


def file_digest(path: str | Path) -> str:
    result = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def identifier(value: Any, name: str = "identifier") -> str:
    if not isinstance(value, str) or not IDENTIFIER.fullmatch(value):
        raise JenkinsError("invalid_identity", f"Invalid {name}")
    return value


def require_digest(value: Any, name: str = "digest") -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        raise JenkinsError("invalid_digest", f"Invalid {name}")
    return value


def response(status: str, *, request_id: str | None = None,
             operation_id: str | None = None, reason_code: str | None = None,
             retryable: bool = False, receipt_ref: str | None = None,
             observed_generation: int | None = None, **data: Any) -> dict:
    return {"schemaVersion": SCHEMA_VERSION, "requestId": request_id,
            "operationId": operation_id, "status": status,
            "reasonCode": reason_code, "retryable": retryable,
            "receiptRef": receipt_ref, "observedGeneration": observed_generation,
            **data}


def execution_key(source_digest: str, recipe: dict, coverage: dict,
                  driver_digest: str, policy_version: str,
                  environment: dict | None = None) -> str:
    """Consumer/session authorization is deliberately outside shared identity."""
    return digest({"sourceDigest": require_digest(source_digest),
                   "recipe": recipe, "coverage": coverage,
                   "driverDigest": require_digest(driver_digest),
                   "policyVersion": policy_version, "environment": environment or {}})
