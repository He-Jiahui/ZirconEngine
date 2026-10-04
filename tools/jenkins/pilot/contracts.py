"""Immutable request identities shared by the pilot CLI and Jenkins runner."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from typing import Mapping

SCHEMA_VERSION = 2
TEMPLATES = frozenset({"python-static", "managed-cargo-check-v2", "fault-probe"})
_IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z")
_SHA256 = re.compile(r"[0-9a-f]{64}\Z")


class PilotError(ValueError):
    """A failed pilot contract, reported without credentials or source bytes."""


def canonical_json(value: object) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False
    ).encode("utf-8")


def digest(value: object) -> str:
    return hashlib.sha256(canonical_json(value)).hexdigest()


def identifier(value: object, field: str) -> str:
    if not isinstance(value, str) or not _IDENTIFIER.fullmatch(value):
        raise PilotError(f"{field} must be a bounded portable identifier")
    return value


def sha256_identity(value: object, field: str) -> str:
    if not isinstance(value, str) or not _SHA256.fullmatch(value):
        raise PilotError(f"{field} must be a lowercase SHA-256 digest")
    return value


@dataclass(frozen=True, slots=True)
class RequestIdentity:
    session_id: str
    request_id: str
    attempt_id: str
    generation: int
    input_hash: str
    template: str

    def __post_init__(self) -> None:
        for field in ("session_id", "request_id", "attempt_id"):
            identifier(getattr(self, field), field)
        if type(self.generation) is not int or self.generation < 1:
            raise PilotError("generation must be a positive integer")
        sha256_identity(self.input_hash, "input_hash")
        if self.template not in TEMPLATES:
            raise PilotError("template is not registered for the pilot")

    def to_dict(self) -> dict[str, object]:
        return {
            "schemaVersion": SCHEMA_VERSION,
            "sessionId": self.session_id,
            "requestId": self.request_id,
            "attemptId": self.attempt_id,
            "generation": self.generation,
            "inputHash": self.input_hash,
            "template": self.template,
        }

    @classmethod
    def from_dict(cls, value: Mapping[str, object]) -> RequestIdentity:
        if not isinstance(value, Mapping):
            raise PilotError("request must be an object")
        expected = {
            "schemaVersion", "sessionId", "requestId", "attemptId", "generation",
            "inputHash", "template",
        }
        if set(value) != expected or type(value.get("schemaVersion")) is not int:
            raise PilotError("request fields do not match the versioned contract")
        if value["schemaVersion"] != SCHEMA_VERSION:
            raise PilotError("request schema version is unsupported")
        return cls(
            session_id=value["sessionId"], request_id=value["requestId"],
            attempt_id=value["attemptId"], generation=value["generation"],
            input_hash=value["inputHash"], template=value["template"],
        )

    @property
    def fingerprint(self) -> str:
        return digest(self.to_dict())

    def parameters(self) -> dict[str, str]:
        return {
            "SESSION_ID": self.session_id,
            "REQUEST_ID": self.request_id,
            "ATTEMPT_ID": self.attempt_id,
            "GENERATION": str(self.generation),
            "INPUT_HASH": self.input_hash,
            "TEMPLATE": self.template,
        }
