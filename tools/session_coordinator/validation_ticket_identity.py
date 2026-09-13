"""Versioned submission identity shared by ticket admission and evidence binding."""

from __future__ import annotations

import hashlib
import re
from typing import Mapping

from . import validation_ticket_inputs as inputs
from .models import CoordinatorError


_VERSION = 1
_FIELDS = frozenset({
    "version", "baselineEpoch", "baseHead", "validatorVersion",
    "runtimeIdentityHash", "executionEnvironmentHash",
})
_SHA256 = re.compile(r"[0-9a-f]{64}")


def create_identity(
    *,
    baseline_epoch: int | None,
    base_head: str | None,
    validator_version: str,
    runtime_identity: str | None,
    execution_environment_hash: str,
) -> dict[str, object]:
    return normalize_identity({
        "version": _VERSION,
        "baselineEpoch": baseline_epoch,
        "baseHead": base_head,
        "validatorVersion": validator_version,
        "runtimeIdentityHash": (
            hashlib.sha256(runtime_identity.encode("utf-8")).hexdigest()
            if runtime_identity else None
        ),
        "executionEnvironmentHash": execution_environment_hash,
    })


def normalize_identity(value: object) -> dict[str, object]:
    if (
        not isinstance(value, dict)
        or set(value) != _FIELDS
        or type(value.get("version")) is not int
        or value["version"] != _VERSION
    ):
        raise CoordinatorError(
            "validation_ticket_identity_revalidation_required",
            "Ticket lacks a complete supported submission identity; submit fresh managed validation",
        )
    epoch = value["baselineEpoch"]
    head = value["baseHead"]
    validator = value["validatorVersion"]
    if (
        (epoch is not None and (type(epoch) is not int or epoch < 0))
        or (head is not None and (not isinstance(head, str) or not head.strip()))
        or not isinstance(validator, str)
        or not validator.strip()
        or not _is_digest(value["executionEnvironmentHash"])
        or (value["runtimeIdentityHash"] is not None and not _is_digest(value["runtimeIdentityHash"]))
    ):
        raise CoordinatorError(
            "validation_ticket_identity_invalid", "Ticket submission identity fields are malformed",
        )
    return dict(value)


def dedupe_key(
    *,
    identity: object,
    source_manifest_hash: str,
    command: tuple[str, ...] | list[str],
    toolchain: Mapping[str, object],
    coverage: Mapping[str, object],
) -> str:
    if not _is_digest(source_manifest_hash):
        raise CoordinatorError("validation_ticket_identity_invalid", "Ticket source identity is malformed")
    payload = {
        "identity": normalize_identity(identity),
        "sourceManifestHash": source_manifest_hash,
        "command": inputs.command(command),
        "toolchain": inputs.mapping("toolchain", toolchain),
        "coverage": inputs.mapping("coverage", coverage),
    }
    return hashlib.sha256(inputs.canonical(payload).encode("utf-8")).hexdigest()


def _is_digest(value: object) -> bool:
    return isinstance(value, str) and _SHA256.fullmatch(value) is not None
