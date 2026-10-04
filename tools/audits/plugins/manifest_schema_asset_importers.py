from __future__ import annotations

from typing import Any


ASSET_IMPORTER_FIELDS = frozenset(
    """
    additional_output_kinds full_suffixes id importer_version output_kind plugin_id
    priority required_capabilities source_extensions
    """.split()
)
ASSET_IMPORTER_ID_CHARSET_DIAGNOSTIC = (
    "must contain only lowercase ASCII letters, digits, underscores, and dots"
)
ASSET_IMPORTER_OUTPUT_KINDS = frozenset(
    (
        "Data",
        "Model",
        "Mesh",
        "Material",
        "MaterialGraph",
        "Texture",
        "Shader",
        "Scene",
        "Sound",
        "Font",
        "PhysicsMaterial",
        "NavMesh",
        "NavigationSettings",
        "Terrain",
        "TerrainLayerStack",
        "TileSet",
        "TileMap",
        "Prefab",
        "AnimationSkeleton",
        "AnimationClip",
        "AnimationSequence",
        "AnimationGraph",
        "AnimationStateMachine",
        "UiLayout",
        "UiWidget",
        "UiStyle",
    )
)
ASSET_IMPORTER_METADATA_ARRAYS = (
    "additional_output_kinds",
    "required_capabilities",
)
ASSET_IMPORTER_REQUIRED_CAPABILITY_CHARSET_DIAGNOSTIC = (
    "must contain only lowercase ASCII letters, digits, underscores, and dots"
)
RETIRED_UI_ASSET_SUFFIXES = (".v2.ui.toml", ".ui.toml")
I32_MIN = -(2**31)
I32_MAX = 2**31 - 1
U32_MAX = 2**32 - 1


def collect_asset_importers_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    asset_importers: object,
    violations: list[str],
) -> None:
    if not isinstance(asset_importers, list):
        violations.append(f"{display_path}: asset_importers must be an array of tables")
        return
    if not asset_importers:
        violations.append(
            f"{display_path}: asset_importers must not be empty when declared"
        )
        return

    package_id = manifest.get("id")
    for importer_index, importer in enumerate(asset_importers):
        importer_label = f"asset_importers[{importer_index}]"
        if not isinstance(importer, dict):
            violations.append(f"{display_path}: {importer_label} must be a table")
            continue
        collect_asset_importer_schema_violations(
            display_path,
            importer_label,
            importer,
            package_id,
            violations,
        )


def collect_asset_importer_schema_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    package_id: object,
    violations: list[str],
) -> None:
    collect_asset_importer_known_field_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    for field in ("id", "plugin_id", "output_kind"):
        collect_asset_importer_required_string_violation(
            display_path,
            f"{importer_label}.{field}",
            importer,
            field,
            violations,
        )
    collect_asset_importer_id_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    collect_asset_importer_plugin_id_match_violations(
        display_path,
        importer_label,
        importer,
        package_id,
        violations,
    )
    collect_asset_importer_required_integer_violation(
        display_path,
        f"{importer_label}.priority",
        importer,
        "priority",
        violations,
    )
    collect_asset_importer_positive_integer_violation(
        display_path,
        f"{importer_label}.importer_version",
        importer,
        "importer_version",
        violations,
    )
    collect_asset_importer_number_range_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    for field in (
        "source_extensions",
        "additional_output_kinds",
        "required_capabilities",
    ):
        collect_asset_importer_string_array_violations(
            display_path,
            f"{importer_label}.{field}",
            importer,
            field,
            violations,
        )
    collect_asset_importer_output_kind_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    collect_asset_importer_required_capability_namespace_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    collect_asset_importer_metadata_array_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    collect_asset_importer_source_selector_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    collect_asset_importer_source_extensions_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )
    collect_asset_importer_full_suffixes_violations(
        display_path,
        importer_label,
        importer,
        violations,
    )


def collect_asset_importer_known_field_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    for field in sorted(importer):
        if field not in ASSET_IMPORTER_FIELDS:
            violations.append(
                f"{display_path}: {importer_label}.{field} "
                "is not a known asset_importer field"
            )


def collect_asset_importer_required_string_violation(
    display_path: str,
    field_label: str,
    importer: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> None:
    value = importer.get(field_name)
    if not isinstance(value, str) or not value.strip():
        violations.append(f"{display_path}: {field_label} must be a non-empty string")
        return
    if value.strip() != value:
        violations.append(f"{display_path}: {field_label} must be trimmed")


def collect_asset_importer_required_integer_violation(
    display_path: str,
    field_label: str,
    importer: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> None:
    value = importer.get(field_name)
    if type(value) is not int:
        violations.append(f"{display_path}: {field_label} must be an integer")


def collect_asset_importer_positive_integer_violation(
    display_path: str,
    field_label: str,
    importer: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> None:
    value = importer.get(field_name)
    if type(value) is not int or value <= 0:
        violations.append(f"{display_path}: {field_label} must be a positive integer")


def collect_asset_importer_plugin_id_match_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    package_id: object,
    violations: list[str],
) -> None:
    plugin_id = importer.get("plugin_id")
    if not (
        isinstance(plugin_id, str)
        and plugin_id.strip()
        and plugin_id.strip() == plugin_id
        and isinstance(package_id, str)
        and package_id.strip()
        and package_id.strip() == package_id
    ):
        return
    if plugin_id != package_id:
        violations.append(
            f"{display_path}: {importer_label}.plugin_id "
            f"must match package id {package_id}"
        )


def collect_asset_importer_string_array_violations(
    display_path: str,
    field_label: str,
    importer: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> None:
    values = importer.get(field_name)
    if values is None:
        return
    if not isinstance(values, list):
        violations.append(f"{display_path}: {field_label} must be an array")
        return
    for value_index, value in enumerate(values):
        item_label = f"{field_label}[{value_index}]"
        if not isinstance(value, str) or not value.strip():
            violations.append(
                f"{display_path}: {item_label} must be a non-empty string"
            )
            continue
        if value.strip() != value:
            violations.append(f"{display_path}: {item_label} must be trimmed")


def collect_asset_importer_id_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    value = importer.get("id")
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return
    label = f"{importer_label}.id"
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} "
            "must use at least two dot-separated namespace segments"
        )
        return
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {label} must not contain empty namespace segments"
        )
        return
    if any(not asset_importer_lowercase_segment(segment) for segment in segments):
        violations.append(
            f"{display_path}: {label} {value} {ASSET_IMPORTER_ID_CHARSET_DIAGNOSTIC}"
        )


def asset_importer_lowercase_segment(segment: str) -> bool:
    return all(
        byte.isascii() and (byte.islower() or byte.isdigit() or byte == "_")
        for byte in segment
    )


def collect_asset_importer_source_selector_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    selector_fields = ("source_extensions", "full_suffixes")
    if all(field not in importer for field in selector_fields):
        violations.append(
            f"{display_path}: {importer_label} "
            "must declare source_extensions or full_suffixes"
        )
    for field in selector_fields:
        value = importer.get(field)
        if isinstance(value, list) and not value:
            violations.append(
                f"{display_path}: {importer_label}.{field} "
                "must not be empty when declared"
            )


def collect_asset_importer_source_extensions_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    values = importer.get("source_extensions")
    if not isinstance(values, list):
        return
    label = f"{importer_label}.source_extensions"
    seen: dict[str, int] = {}
    for value_index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            continue
        item_label = f"{label}[{value_index}]"
        duplicate_index = seen.get(value)
        if duplicate_index is not None:
            violations.append(
                f"{display_path}: {item_label} duplicates entry {duplicate_index}"
            )
            continue
        seen[value] = value_index
        if "." in value:
            violations.append(
                f"{display_path}: {item_label} must be a lowercase extension "
                "without dots; use full_suffixes for dotted suffixes"
            )
            continue
        if value != value.lower():
            violations.append(f"{display_path}: {item_label} must be lowercase")


def collect_asset_importer_output_kind_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    collect_asset_importer_known_output_kind_violation(
        display_path,
        f"{importer_label}.output_kind",
        importer.get("output_kind"),
        violations,
    )
    values = importer.get("additional_output_kinds")
    if not isinstance(values, list):
        return
    for value_index, value in enumerate(values):
        collect_asset_importer_known_output_kind_violation(
            display_path,
            f"{importer_label}.additional_output_kinds[{value_index}]",
            value,
            violations,
        )


def collect_asset_importer_known_output_kind_violation(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return
    if value not in ASSET_IMPORTER_OUTPUT_KINDS:
        violations.append(f"{display_path}: {field_label} must be a known ResourceKind")


def collect_asset_importer_metadata_array_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    for field_name in ASSET_IMPORTER_METADATA_ARRAYS:
        values = importer.get(field_name)
        if not isinstance(values, list):
            continue
        label = f"{importer_label}.{field_name}"
        if not values:
            violations.append(
                f"{display_path}: {label} must not be empty when declared"
            )
            continue
        seen: dict[str, int] = {}
        for value_index, value in enumerate(values):
            if not isinstance(value, str) or not value.strip() or value.strip() != value:
                continue
            duplicate_index = seen.get(value)
            if duplicate_index is not None:
                violations.append(
                    f"{display_path}: {label}[{value_index}] "
                    f"duplicates entry {duplicate_index}"
                )
                continue
            seen[value] = value_index


def collect_asset_importer_number_range_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    priority = importer.get("priority")
    if type(priority) is int and (priority < I32_MIN or priority > I32_MAX):
        violations.append(f"{display_path}: {importer_label}.priority must fit i32")
    importer_version = importer.get("importer_version")
    if (
        type(importer_version) is int
        and importer_version > 0
        and importer_version > U32_MAX
    ):
        violations.append(
            f"{display_path}: {importer_label}.importer_version "
            "must be a positive u32"
        )


def collect_asset_importer_required_capability_namespace_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    values = importer.get("required_capabilities")
    if not isinstance(values, list):
        return
    label = f"{importer_label}.required_capabilities"
    for value_index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            continue
        collect_asset_importer_required_capability_namespace_violation(
            display_path,
            f"{label}[{value_index}]",
            value,
            violations,
        )


def collect_asset_importer_required_capability_namespace_violation(
    display_path: str,
    field_label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {field_label} "
            "must use at least two dot-separated namespace segments"
        )
        return
    if any(not asset_importer_lowercase_segment(segment) for segment in segments):
        violations.append(
            f"{display_path}: {field_label} "
            f"{ASSET_IMPORTER_REQUIRED_CAPABILITY_CHARSET_DIAGNOSTIC}"
        )


def collect_asset_importer_full_suffixes_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    violations: list[str],
) -> None:
    values = importer.get("full_suffixes")
    if values is None:
        return
    label = f"{importer_label}.full_suffixes"
    if not isinstance(values, list):
        violations.append(f"{display_path}: {label} must be an array")
        return
    seen: dict[str, int] = {}
    for value_index, value in enumerate(values):
        item_label = f"{label}[{value_index}]"
        if not isinstance(value, str) or not value.strip():
            violations.append(
                f"{display_path}: {item_label} must be a non-empty string"
            )
            continue
        if value.strip() != value:
            violations.append(f"{display_path}: {item_label} must be trimmed")
            continue
        duplicate_index = seen.get(value)
        if duplicate_index is not None:
            violations.append(
                f"{display_path}: {item_label} duplicates entry {duplicate_index}"
            )
            continue
        seen[value] = value_index
        if not value.startswith(".") or value == ".":
            violations.append(f"{display_path}: {item_label} must be a dotted suffix")
            continue
        if value != value.lower():
            violations.append(f"{display_path}: {item_label} must be lowercase")
            continue
        retired_suffix = retired_ui_asset_suffix(value)
        if retired_suffix is not None:
            violations.append(
                f"{display_path}: {item_label} declares retired UI asset suffix "
                f"{retired_suffix}; use .zui"
            )


def retired_ui_asset_suffix(value: str) -> str | None:
    for suffix in RETIRED_UI_ASSET_SUFFIXES:
        if value.endswith(suffix):
            return suffix
    return None
