from __future__ import annotations

from dataclasses import dataclass
from pathlib import PurePosixPath
from typing import Any

from .manifest_schema import is_non_empty_trimmed_string


GEOMETRY_SOURCE_PLUGIN_ID_START = 4
GEOMETRY_SOURCE_ID_MAX = 255
GEOMETRY_SOURCE_FIELDS = frozenset(
    {
        "id",
        "token",
        "wgsl_include",
        "vertex_attributes",
        "required_bindings",
        "shader_defines",
    }
)
GEOMETRY_SOURCE_REQUIRED_FIELDS = (
    "id",
    "token",
    "wgsl_include",
    "vertex_attributes",
    "required_bindings",
    "shader_defines",
)
GEOMETRY_SOURCE_VERTEX_ATTRIBUTES = (
    "position",
    "normal",
    "tangent",
    "uv0",
    "color0",
    "joint_indices",
    "joint_weights",
    "morph_position_delta",
    "morph_normal_delta",
)
GEOMETRY_SOURCE_BINDING_FIELDS = frozenset({"kind", "slot_token"})
GEOMETRY_SOURCE_BINDING_KINDS = (
    "gpu_scene_instance",
    "skinning_palette_storage",
    "morph_weights_storage",
    "morph_target_storage",
    "virtual_geometry_pages",
    "virtual_geometry_clusters",
)
GEOMETRY_SOURCE_SHADER_DEFINE_FIELDS = frozenset({"kind", "name", "value"})
GEOMETRY_SOURCE_SHADER_DEFINE_KINDS = ("bool", "int", "uint")
SHADER_PERMUTATION_FIELDS = frozenset(
    {"geometry_source_ids", "shading_model_ids", "shader_modules"}
)
SHADER_PERMUTATION_ID_FIELDS = frozenset({"token", "id"})
SHADER_PERMUTATION_MODULE_FIELDS = frozenset({"import_path", "source"})
I32_MIN = -(2**31)
I32_MAX = 2**31 - 1
U32_MAX = 2**32 - 1


@dataclass(frozen=True)
class GeometrySourceAssignment:
    token: str
    id_value: int


class GeometrySourceRegistry:
    def __init__(self) -> None:
        self._by_token: dict[str, GeometrySourceAssignment] = {}
        self._by_id: dict[int, GeometrySourceAssignment] = {}

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
        assignment = GeometrySourceAssignment(token=token, id_value=id_value)
        self._by_token[token] = assignment
        self._by_id[id_value] = assignment


def collect_geometry_source_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    registry = GeometrySourceRegistry()
    collect_geometry_source_descriptor_schema_violations(
        display_path, manifest.get("geometry_sources"), violations, registry
    )
    collect_shader_permutation_schema_violations(
        display_path, manifest.get("shader_permutation"), violations, registry
    )


def collect_geometry_source_descriptor_schema_violations(
    display_path: str,
    entries: object,
    violations: list[str],
    registry: GeometrySourceRegistry,
) -> None:
    if entries is None:
        return
    if not isinstance(entries, list):
        violations.append(f"{display_path}: geometry_sources must be an array")
        return
    for index, entry in enumerate(entries):
        field_label = f"geometry_sources[{index}]"
        if not isinstance(entry, dict):
            violations.append(f"{display_path}: {field_label} must be a table")
            continue
        collect_known_field_violations(
            display_path,
            field_label,
            entry,
            GEOMETRY_SOURCE_FIELDS,
            "geometry source",
            violations,
        )
        for field in GEOMETRY_SOURCE_REQUIRED_FIELDS:
            if field not in entry:
                violations.append(f"{display_path}: missing {field_label}.{field}")

        id_value = geometry_source_id_value(
            display_path, f"{field_label}.id", entry.get("id"), violations
        )
        token = geometry_source_token(
            display_path, f"{field_label}.token", entry.get("token"), violations
        )
        collect_geometry_source_wgsl_include_violations(
            display_path, field_label, entry, violations
        )
        collect_vertex_attribute_schema_violations(
            display_path, field_label, entry.get("vertex_attributes"), violations
        )
        collect_required_binding_schema_violations(
            display_path, field_label, entry.get("required_bindings"), violations
        )
        collect_shader_define_schema_violations(
            display_path, field_label, entry.get("shader_defines"), violations
        )
        registry.add(display_path, field_label, token, id_value, violations)


def collect_shader_permutation_schema_violations(
    display_path: str,
    permutation: object,
    violations: list[str],
    registry: GeometrySourceRegistry,
) -> None:
    if permutation is None:
        return
    if not isinstance(permutation, dict):
        violations.append(f"{display_path}: shader_permutation must be a table")
        return
    collect_known_field_violations(
        display_path,
        "shader_permutation",
        permutation,
        SHADER_PERMUTATION_FIELDS,
        "shader_permutation",
        violations,
    )
    collect_shader_permutation_geometry_source_ids(
        display_path,
        permutation.get("geometry_source_ids"),
        violations,
        registry,
    )
    collect_shader_permutation_shader_modules(
        display_path,
        permutation.get("shader_modules"),
        violations,
    )


def collect_shader_permutation_geometry_source_ids(
    display_path: str,
    entries: object,
    violations: list[str],
    registry: GeometrySourceRegistry,
) -> None:
    if entries is None:
        return
    if not isinstance(entries, list):
        violations.append(
            f"{display_path}: shader_permutation.geometry_source_ids must be an array"
        )
        return
    for index, entry in enumerate(entries):
        field_label = f"shader_permutation.geometry_source_ids[{index}]"
        if not isinstance(entry, dict):
            violations.append(f"{display_path}: {field_label} must be a table")
            continue
        collect_known_field_violations(
            display_path,
            field_label,
            entry,
            SHADER_PERMUTATION_ID_FIELDS,
            "shader_permutation geometry source id",
            violations,
        )
        for field in ("token", "id"):
            if field not in entry:
                violations.append(f"{display_path}: missing {field_label}.{field}")
        token = geometry_source_token(
            display_path, f"{field_label}.token", entry.get("token"), violations
        )
        id_value = geometry_source_id_value(
            display_path, f"{field_label}.id", entry.get("id"), violations
        )
        registry.add(display_path, field_label, token, id_value, violations)


def collect_shader_permutation_shader_modules(
    display_path: str,
    entries: object,
    violations: list[str],
) -> None:
    if entries is None:
        return
    if not isinstance(entries, list):
        violations.append(
            f"{display_path}: shader_permutation.shader_modules must be an array"
        )
        return
    seen: dict[str, int] = {}
    for index, entry in enumerate(entries):
        field_label = f"shader_permutation.shader_modules[{index}]"
        if not isinstance(entry, dict):
            violations.append(f"{display_path}: {field_label} must be a table")
            continue
        collect_known_field_violations(
            display_path,
            field_label,
            entry,
            SHADER_PERMUTATION_MODULE_FIELDS,
            "shader_permutation shader module",
            violations,
        )
        for field in ("import_path", "source"):
            if field not in entry:
                violations.append(f"{display_path}: missing {field_label}.{field}")
        import_path = shader_module_import_path(
            display_path, f"{field_label}.import_path", entry.get("import_path"), violations
        )
        shader_module_source(
            display_path, f"{field_label}.source", entry.get("source"), violations
        )
        if import_path is None:
            continue
        previous_index = seen.get(import_path)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {field_label}.import_path {import_path} "
                f"duplicates shader_permutation.shader_modules[{previous_index}]"
            )
            continue
        seen[import_path] = index


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


def geometry_source_id_value(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> int | None:
    if type(value) is not int:
        violations.append(f"{display_path}: {field_label} must be an integer")
        return None
    if value < GEOMETRY_SOURCE_PLUGIN_ID_START:
        violations.append(
            f"{display_path}: {field_label} {value} must be a plugin geometry "
            f"source id >= {GEOMETRY_SOURCE_PLUGIN_ID_START}"
        )
        return None
    if value > GEOMETRY_SOURCE_ID_MAX:
        violations.append(
            f"{display_path}: {field_label} {value} must be a plugin geometry "
            f"source id <= {GEOMETRY_SOURCE_ID_MAX}"
        )
        return None
    return value


def geometry_source_token(
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


def shader_module_import_path(
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
    segments = value.split("::")
    if len(segments) < 2 or any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {field_label} {value} must use namespace::module form"
        )
        return None
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for segment in segments
        for char in segment
    ):
        violations.append(
            f"{display_path}: {field_label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and namespace separators"
        )
        return None
    return value


def shader_module_source(
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
    if "\\" in value:
        violations.append(f"{display_path}: {field_label} {value} must use forward slashes")
        return None
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        violations.append(
            f"{display_path}: {field_label} {value} must be a package-relative shader path"
        )
        return None
    if path.suffix not in {".zshader", ".wgsl"}:
        violations.append(
            f"{display_path}: {field_label} {value} must end with .zshader or .wgsl"
        )
        return None
    return value


def collect_geometry_source_wgsl_include_violations(
    display_path: str,
    field_label: str,
    entry: dict[str, Any],
    violations: list[str],
) -> None:
    value = entry.get("wgsl_include")
    item_label = f"{field_label}.wgsl_include"
    if not is_non_empty_trimmed_string(value):
        violations.append(
            f"{display_path}: {item_label} must be a non-empty trimmed string"
        )
        return
    if not value.endswith(".wgsl"):
        violations.append(f"{display_path}: {item_label} {value} must end with .wgsl")


def collect_vertex_attribute_schema_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    item_label = f"{field_label}.vertex_attributes"
    if not isinstance(value, list) or not value:
        violations.append(f"{display_path}: {item_label} must be a non-empty array")
        return
    allowed = set(GEOMETRY_SOURCE_VERTEX_ATTRIBUTES)
    expected = ", ".join(GEOMETRY_SOURCE_VERTEX_ATTRIBUTES)
    seen: dict[str, int] = {}
    for index, entry in enumerate(value):
        entry_label = f"{item_label}[{index}]"
        if not is_non_empty_trimmed_string(entry):
            violations.append(
                f"{display_path}: {entry_label} must be a non-empty trimmed string"
            )
            continue
        if entry not in allowed:
            violations.append(
                f'{display_path}: {entry_label} "{entry}" is unsupported; '
                f"expected one of {expected}"
            )
            continue
        previous_index = seen.get(entry)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {entry_label} {entry} duplicates "
                f"vertex_attributes[{previous_index}]"
            )
            continue
        seen[entry] = index


def collect_required_binding_schema_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    item_label = f"{field_label}.required_bindings"
    if not isinstance(value, list):
        violations.append(f"{display_path}: {item_label} must be an array")
        return
    seen: dict[tuple[str, str], int] = {}
    for index, binding in enumerate(value):
        binding_label = f"{item_label}[{index}]"
        if not isinstance(binding, dict):
            violations.append(f"{display_path}: {binding_label} must be a table")
            continue
        collect_known_field_violations(
            display_path,
            binding_label,
            binding,
            GEOMETRY_SOURCE_BINDING_FIELDS,
            "geometry source binding",
            violations,
        )
        for field in ("kind", "slot_token"):
            if field not in binding:
                violations.append(f"{display_path}: missing {binding_label}.{field}")
        kind = geometry_source_binding_kind(
            display_path, f"{binding_label}.kind", binding.get("kind"), violations
        )
        slot_token = geometry_source_slot_token(
            display_path,
            f"{binding_label}.slot_token",
            binding.get("slot_token"),
            violations,
        )
        if kind is None or slot_token is None:
            continue
        identity = (kind, slot_token)
        previous_index = seen.get(identity)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {binding_label} duplicates "
                f"required_bindings[{previous_index}]"
            )
            continue
        seen[identity] = index


def geometry_source_binding_kind(
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
    allowed = set(GEOMETRY_SOURCE_BINDING_KINDS)
    if value not in allowed:
        expected = ", ".join(GEOMETRY_SOURCE_BINDING_KINDS)
        violations.append(
            f'{display_path}: {field_label} "{value}" is unsupported; '
            f"expected one of {expected}"
        )
        return None
    return value


def geometry_source_slot_token(
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
    if any(segment == "" for segment in value.split(".")):
        violations.append(
            f"{display_path}: {field_label} {value} should not contain empty "
            "namespace segments"
        )
        return None
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {field_label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and dots"
        )
        return None
    return value


def collect_shader_define_schema_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    item_label = f"{field_label}.shader_defines"
    if not isinstance(value, list):
        violations.append(f"{display_path}: {item_label} must be an array")
        return
    seen: dict[str, int] = {}
    for index, define in enumerate(value):
        define_label = f"{item_label}[{index}]"
        name: str | None
        if isinstance(define, str):
            name = shader_define_name(display_path, define_label, define, violations)
        elif isinstance(define, dict):
            collect_known_field_violations(
                display_path,
                define_label,
                define,
                GEOMETRY_SOURCE_SHADER_DEFINE_FIELDS,
                "geometry source shader define",
                violations,
            )
            for field in ("kind", "name", "value"):
                if field not in define:
                    violations.append(f"{display_path}: missing {define_label}.{field}")
            kind = shader_define_kind(
                display_path, f"{define_label}.kind", define.get("kind"), violations
            )
            name = shader_define_name(
                display_path, f"{define_label}.name", define.get("name"), violations
            )
            collect_shader_define_value_violations(
                display_path, f"{define_label}.value", kind, define.get("value"), violations
            )
        else:
            violations.append(
                f"{display_path}: {define_label} must be a string or table"
            )
            continue
        if name is None:
            continue
        previous_index = seen.get(name)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {define_label}.name {name} duplicates "
                f"shader_defines[{previous_index}]"
            )
            continue
        seen[name] = index


def shader_define_kind(
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
    allowed = set(GEOMETRY_SOURCE_SHADER_DEFINE_KINDS)
    if value not in allowed:
        expected = ", ".join(GEOMETRY_SOURCE_SHADER_DEFINE_KINDS)
        violations.append(
            f'{display_path}: {field_label} "{value}" is unsupported; '
            f"expected one of {expected}"
        )
        return None
    return value


def shader_define_name(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> str | None:
    if not is_non_empty_trimmed_string(value) or not all(
        char.isascii() and (char.isupper() or char.isdigit() or char == "_")
        for char in value
    ):
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed shader "
            "define name"
        )
        return None
    return value


def collect_shader_define_value_violations(
    display_path: str,
    field_label: str,
    kind: str | None,
    value: object,
    violations: list[str],
) -> None:
    if kind == "bool":
        if type(value) is not bool:
            violations.append(f"{display_path}: {field_label} must be a bool")
        return
    if kind == "int":
        if type(value) is not int or value < I32_MIN or value > I32_MAX:
            violations.append(f"{display_path}: {field_label} must be an i32 integer")
        return
    if kind == "uint":
        if type(value) is not int or value < 0 or value > U32_MAX:
            violations.append(f"{display_path}: {field_label} must be a u32 integer")
