"""Canonical validation ticket input parsing, independent of queue persistence."""

from __future__ import annotations

import json
import re
from typing import Mapping

from .models import CoordinatorError
from .portable_paths import normalize_portable_relative_path, portable_path_key


def require_text(field: str, value: object) -> str:
    if not isinstance(value, str) or not value.strip():
        raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be non-empty text")
    return value.strip()


def manifest(value: Mapping[str, str | None]) -> dict[str, str | None]:
    if not isinstance(value, Mapping) or not value:
        raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest must be non-empty")
    normalized: dict[str, str | None] = {}
    path_keys: set[str] = set()
    for raw_path, raw_hash in value.items():
        path = normalize_portable_relative_path(
            raw_path, code="validation_ticket_manifest_invalid", message="source_manifest path is unsafe",
        )
        folded = path.casefold()
        path_key = portable_path_key(path)
        protected = any(folded == root or folded.startswith(root + "/") for root in (".git", "target", ".codex/state"))
        if protected or path_key in path_keys:
            raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest path is unsafe")
        path_keys.add(path_key)
        if raw_hash is None:
            normalized[path] = None
        elif isinstance(raw_hash, str) and re.fullmatch(r"[0-9a-f]{64}", raw_hash.casefold()):
            normalized[path] = raw_hash.casefold()
        else:
            raise CoordinatorError("validation_ticket_manifest_invalid", "source_manifest values must be SHA-256 or null deletion tombstones")
    return dict(sorted(normalized.items(), key=lambda item: item[0].casefold()))


def command(value: tuple[str, ...] | list[str]) -> tuple[str, ...]:
    if not isinstance(value, (tuple, list)) or not value:
        raise CoordinatorError("validation_ticket_command_invalid", "command must be a non-empty string sequence")
    return tuple(require_text("command", item) for item in value)


def mapping(field: str, value: Mapping[str, object]) -> dict[str, object]:
    if not isinstance(value, Mapping):
        raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be an object")
    result = json_value(field, value, set())
    if not isinstance(result, dict):
        raise AssertionError("mapping normalization must preserve object shape")
    return result


def json_value(field: str, value: object, active: set[int]) -> object:
    if isinstance(value, (Mapping, list, tuple)):
        identity = id(value)
        if identity in active:
            raise CoordinatorError("validation_ticket_input_invalid", f"{field} must not contain a circular JSON value")
        active.add(identity)
        try:
            if isinstance(value, Mapping):
                result = {}
                for key, item in value.items():
                    if not isinstance(key, str):
                        raise CoordinatorError("validation_ticket_input_invalid", f"{field} object keys must be strings")
                    result[key] = json_value(field, item, active)
                return result
            return [json_value(field, item, active) for item in value]
        finally:
            active.remove(identity)
    try:
        json.dumps(value, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise CoordinatorError("validation_ticket_input_invalid", f"{field} must be JSON serializable") from error
    return value


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False)
