from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .manifest_schema import is_non_empty_trimmed_string


SHADING_MODEL_PLUGIN_ID_START = 16
SHADING_MODEL_ID_MAX = 255
SHADING_MODEL_FIELDS = frozenset(
    {
        "id",
        "token",
        "forward_include",
        "gbuffer_encode_include",
        "deferred_include",
        "required_channels",
    }
)
SHADING_MODEL_REQUIRED_FIELDS = (
    "id",
    "token",
    "forward_include",
    "gbuffer_encode_include",
    "deferred_include",
    "required_channels",
)
SHADER_PERMUTATION_SHADING_MODEL_ID_FIELDS = frozenset({"token", "id"})
U16_MAX = 2**16 - 1


@dataclass(frozen=True)
class ShadingModelAssignment:
    token: str
    id_value: int


class ShadingModelRegistry:
    def __init__(self) -> None:
        self._by_token: dict[str, ShadingModelAssignment] = {}
        self._by_id: dict[int, ShadingModelAssignment] = {}

    def add(
        self,
        display_path: str,
        field_label: str,
        token: str | None,
        id_value: int | None,
        violations: list[str],
    ) -> None:
        if token is None or id_value is None:
            return
        existing_by_token = self._by_token.get(token)
        if existing_by_token is not None:
            if existing_by_token.id_value != id_value:
                violations.append(
                    f"{display_path}: {field_label}.token {token} was already "
                    f"assigned id {existing_by_token.id_value} and cannot be "
                    f"reused by id {id_value}"
                )
            return
        existing_by_id = self._by_id.get(id_value)
        if existing_by_id is not None:
            if existing_by_id.token != token:
                violations.append(
                    f"{display_path}: {field_label}.id {id_value} was already "
                    f"assigned to {existing_by_id.token} and cannot be reused by "
                    f"{token}"
                )
            return
        assignment = ShadingModelAssignment(token=token, id_value=id_value)
        self._by_token[token] = assignment
        self._by_id[id_value] = assignment


def collect_shading_model_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    registry = ShadingModelRegistry()
    collect_shading_model_descriptor_schema_violations(
        display_path, manifest.get("shading_models"), violations, registry
    )
    collect_shader_permutation_shading_model_schema_violations(
        display_path, manifest.get("shader_permutation"), violations, registry
    )


def collect_shading_model_descriptor_schema_violations(
    display_path: str,
    entries: object,
    violations: list[str],
    registry: ShadingModelRegistry,
) -> None:
    if entries is None:
        return
    if not isinstance(entries, list):
        violations.append(f"{display_path}: shading_models must be an array")
        return
    for index, entry in enumerate(entries):
        field_label = f"shading_models[{index}]"
        if not isinstance(entry, dict):
            violations.append(f"{display_path}: {field_label} must be a table")
            continue
        collect_known_field_violations(
            display_path,
            field_label,
            entry,
            SHADING_MODEL_FIELDS,
            "shading model",
            violations,
        )
        for field in SHADING_MODEL_REQUIRED_FIELDS:
            if field not in entry:
                violations.append(f"{display_path}: missing {field_label}.{field}")

        id_value = shading_model_id_value(
            display_path, f"{field_label}.id", entry.get("id"), violations
        )
        token = shading_model_token(
            display_path, f"{field_label}.token", entry.get("token"), violations
        )
        for field in (
            "forward_include",
            "gbuffer_encode_include",
            "deferred_include",
        ):
            collect_wgsl_include_violations(
                display_path, f"{field_label}.{field}", entry.get(field), violations
            )
        collect_required_channels_violations(
            display_path,
            f"{field_label}.required_channels",
            entry.get("required_channels"),
            violations,
        )
        registry.add(display_path, field_label, token, id_value, violations)


def collect_shader_permutation_shading_model_schema_violations(
    display_path: str,
    permutation: object,
    violations: list[str],
    registry: ShadingModelRegistry,
) -> None:
    if not isinstance(permutation, dict):
        return
    entries = permutation.get("shading_model_ids")
    if entries is None:
        return
    if not isinstance(entries, list):
        violations.append(
            f"{display_path}: shader_permutation.shading_model_ids must be an array"
        )
        return
    for index, entry in enumerate(entries):
        field_label = f"shader_permutation.shading_model_ids[{index}]"
        if not isinstance(entry, dict):
            violations.append(f"{display_path}: {field_label} must be a table")
            continue
        collect_known_field_violations(
            display_path,
            field_label,
            entry,
            SHADER_PERMUTATION_SHADING_MODEL_ID_FIELDS,
            "shader_permutation shading model id",
            violations,
        )
        for field in ("token", "id"):
            if field not in entry:
                violations.append(f"{display_path}: missing {field_label}.{field}")
        token = shading_model_token(
            display_path, f"{field_label}.token", entry.get("token"), violations
        )
        id_value = shading_model_id_value(
            display_path, f"{field_label}.id", entry.get("id"), violations
        )
        registry.add(display_path, field_label, token, id_value, violations)


def collect_known_field_violations(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    known_fields: frozenset[str],
    field_name: str,
    violations: list[str],
) -> None:
    for field in sorted(table):
        if field not in known_fields:
            violations.append(
                f"{display_path}: {field_label}.{field} is not a known "
                f"{field_name} field"
            )


def shading_model_id_value(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> int | None:
    if type(value) is not int:
        violations.append(f"{display_path}: {field_label} must be an integer")
        return None
    if value < SHADING_MODEL_PLUGIN_ID_START:
        violations.append(
            f"{display_path}: {field_label} {value} must be a plugin shading "
            f"model id >= {SHADING_MODEL_PLUGIN_ID_START}"
        )
        return None
    if value > SHADING_MODEL_ID_MAX:
        violations.append(
            f"{display_path}: {field_label} {value} must be a plugin shading "
            f"model id <= {SHADING_MODEL_ID_MAX}"
        )
        return None
    return value


def shading_model_token(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> str | None:
    if not is_non_empty_trimmed_string(value):
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )
        return None
    if not value.startswith("custom:") or not value.removeprefix("custom:"):
        violations.append(f"{display_path}: {field_label} {value} must use custom:<name>")
        return None
    name = value.removeprefix("custom:")
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "-"})
        for char in name
    ):
        violations.append(
            f"{display_path}: {field_label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and hyphens after custom:"
        )
        return None
    return value


def collect_wgsl_include_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    if not is_non_empty_trimmed_string(value):
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )
        return
    if not value.endswith(".wgsl"):
        violations.append(f"{display_path}: {field_label} {value} must end with .wgsl")


def collect_required_channels_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    if type(value) is not int or value < 0 or value > U16_MAX:
        violations.append(f"{display_path}: {field_label} must be a u16 integer")
