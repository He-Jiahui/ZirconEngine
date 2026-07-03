from __future__ import annotations

from typing import Any

from .manifest_schema import (
    MATURITY_VALUES,
    PACKAGING_VALUES,
    SUPPORTED_PLATFORM_ALIASES,
    SUPPORTED_PLATFORM_VALUES,
    SUPPORTED_TARGET_VALUES,
    collect_allowed_string_array_values,
    collect_allowed_string_value,
    is_non_empty_trimmed_string,
)


PACKAGE_KIND_VALUES = ("standard", "feature_extension")
CATEGORY_VALUES = (
    "asset_importer",
    "authoring",
    "diagnostics",
    "platform",
    "rendering",
    "runtime",
    "sdk",
)
MANIFEST_VERSION_FIELDS = ("version", "sdk_api_version")
MANIFEST_SEMVER_COMPONENTS = ("major", "minor", "patch")
MANIFEST_U32_MAX = (1 << 32) - 1
MANIFEST_ID_CHARSET_MESSAGE = (
    "must contain only lowercase ASCII letters, digits, underscores, "
    "and dots in non-empty segments"
)
MANIFEST_ID_START_MESSAGE = "must start with a lowercase ASCII letter"
MANIFEST_ID_UNDERSCORE_MESSAGE = (
    "segments must not end with an underscore or contain repeated underscores"
)
MANIFEST_SEMVER_SHAPE_MESSAGE = "must use MAJOR.MINOR.PATCH form"


def collect_root_metadata_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    from .manifest_schema_distribution import (
        collect_root_distribution_schema_violations,
    )

    collect_root_manifest_identity_violations(display_path, manifest, violations)
    collect_root_manifest_version_violations(display_path, manifest, violations)
    collect_root_distribution_schema_violations(display_path, manifest, violations)
    collect_allowed_string_value(
        display_path,
        "category",
        manifest,
        "category",
        CATEGORY_VALUES,
        violations,
    )
    collect_supported_target_values(display_path, manifest, violations)
    collect_supported_platform_values(display_path, manifest, violations)
    collect_root_capability_violations(display_path, manifest, violations)
    collect_root_package_kind_violations(display_path, manifest, violations)
    collect_root_default_packaging_violations(display_path, manifest, violations)
    from .manifest_schema_layout_coordinates import (
        collect_layout_coordinate_schema_violations,
    )

    collect_layout_coordinate_schema_violations(display_path, manifest, violations)
    collect_allowed_string_value(
        display_path,
        "maturity",
        manifest,
        "maturity",
        MATURITY_VALUES,
        violations,
    )


def collect_root_manifest_identity_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    value = manifest.get("id")
    if not is_non_empty_trimmed_string(value):
        return
    segments = value.split(".")
    if any(not segment for segment in segments) or any(
        not root_manifest_identity_char(char)
        for segment in segments
        for char in segment
    ):
        violations.append(f"{display_path}: id {value} {MANIFEST_ID_CHARSET_MESSAGE}")
    if not ("a" <= value[0] <= "z"):
        violations.append(f"{display_path}: id {value} {MANIFEST_ID_START_MESSAGE}")
    if any(segment.endswith("_") or "__" in segment for segment in segments):
        violations.append(
            f"{display_path}: id {value} {MANIFEST_ID_UNDERSCORE_MESSAGE}"
        )


def root_manifest_identity_char(char: str) -> bool:
    return ("a" <= char <= "z") or char.isdigit() or char in {"_", "."}


def collect_root_manifest_version_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    for field_name in MANIFEST_VERSION_FIELDS:
        collect_root_manifest_semver_violations(
            display_path,
            field_name,
            manifest.get(field_name),
            violations,
        )


def collect_root_manifest_semver_violations(
    display_path: str,
    field_name: str,
    value: object,
    violations: list[str],
) -> None:
    if not is_non_empty_trimmed_string(value):
        return
    segments = value.split(".")
    if len(segments) != len(MANIFEST_SEMVER_COMPONENTS):
        violations.append(
            f"{display_path}: {field_name} {value} {MANIFEST_SEMVER_SHAPE_MESSAGE}"
        )
        return
    for component_name, segment in zip(MANIFEST_SEMVER_COMPONENTS, segments):
        collect_root_manifest_semver_component_violations(
            display_path,
            field_name,
            value,
            component_name,
            segment,
            violations,
        )


def collect_root_manifest_semver_component_violations(
    display_path: str,
    field_name: str,
    value: str,
    component_name: str,
    segment: str,
    violations: list[str],
) -> None:
    if not segment.isascii() or not segment.isdigit():
        violations.append(
            f"{display_path}: {field_name} {value} {component_name} "
            f"component {segment} must contain ASCII digits"
        )
        return
    if len(segment) > 1 and segment.startswith("0"):
        violations.append(
            f"{display_path}: {field_name} {value} {component_name} "
            f"component {segment} must not use leading zeroes"
        )
        return
    if int(segment) > MANIFEST_U32_MAX:
        violations.append(
            f"{display_path}: {field_name} {value} {component_name} "
            f"component {segment} must fit in u32"
        )


def collect_root_package_kind_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    package_kind = "standard"
    if "package_kind" in manifest:
        value = manifest["package_kind"]
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            violations.append(
                f"{display_path}: package_kind must be a non-empty trimmed string"
            )
            return
        package_kind = value
    if package_kind not in PACKAGE_KIND_VALUES:
        violations.append(
            f"{display_path}: package_kind {package_kind} "
            "should be standard or feature_extension"
        )
        return
    optional_feature_count = root_metadata_table_array_count(manifest, "optional_features")
    feature_extension_count = root_metadata_table_array_count(
        manifest, "feature_extensions"
    )
    has_feature_extensions = "feature_extensions" in manifest
    if package_kind == "standard" and feature_extension_count:
        violations.append(
            f"{display_path}: standard package_kind "
            "should not declare feature_extensions rows"
        )
    if package_kind == "feature_extension":
        if not has_feature_extensions:
            violations.append(
                f"{display_path}: package_kind feature_extension "
                "should declare at least one feature_extensions row"
            )
        if optional_feature_count:
            violations.append(
                f"{display_path}: package_kind feature_extension "
                "should not declare optional_features rows"
            )


def root_metadata_table_array_count(
    manifest: dict[str, Any],
    field_name: str,
) -> int:
    rows = manifest.get(field_name)
    return len(rows) if isinstance(rows, list) else 0


def collect_supported_target_values(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    field_label = "supported_targets"
    collect_allowed_string_array_values(
        display_path,
        field_label,
        manifest,
        field_label,
        SUPPORTED_TARGET_VALUES,
        violations,
    )
    value = manifest.get(field_label)
    if not isinstance(value, list):
        return
    allowed = set(SUPPORTED_TARGET_VALUES)
    seen: dict[str, int] = {}
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip() or entry.strip() != entry:
            continue
        if entry not in allowed:
            continue
        previous_index = seen.get(entry)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {field_label}[{index}] {entry} "
                f"duplicates {field_label}[{previous_index}]"
            )
            continue
        seen[entry] = index


def collect_supported_platform_values(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    field_label = "supported_platforms"
    collect_allowed_string_array_values(
        display_path,
        field_label,
        manifest,
        field_label,
        SUPPORTED_PLATFORM_VALUES,
        violations,
    )
    value = manifest.get(field_label)
    if not isinstance(value, list):
        return
    allowed = set(SUPPORTED_PLATFORM_VALUES)
    seen: dict[str, int] = {}
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip() or entry.strip() != entry:
            continue
        if entry not in allowed:
            continue
        canonical = SUPPORTED_PLATFORM_ALIASES.get(entry, entry)
        previous_index = seen.get(canonical)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {field_label}[{index}] {entry} "
                f"duplicates {field_label}[{previous_index}]"
            )
            continue
        seen[canonical] = index


def collect_root_capability_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    value = manifest.get("capabilities")
    if not isinstance(value, list):
        return
    seen: dict[str, int] = {}
    for index, capability in enumerate(value):
        if not isinstance(capability, str) or capability.strip() != capability or not capability:
            continue
        item_label = f"capabilities[{index}]"
        collect_root_capability_namespace_violations(
            display_path, item_label, capability, violations
        )
        previous_index = seen.get(capability)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} {capability} "
                f"duplicates capabilities capabilities[{previous_index}]"
            )
            continue
        seen[capability] = index


def collect_root_capability_namespace_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} {value} "
            "should use at least two dot-separated namespace segments"
        )
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {label} {value} "
            "should not contain empty namespace segments"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {label} {value} should contain only lowercase ASCII "
            "letters, digits, underscores, and dots"
        )


def collect_root_default_packaging_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    field_label = "default_packaging"
    value = manifest.get(field_label)
    if not isinstance(value, list):
        return
    collect_allowed_string_array_values(
        display_path,
        field_label,
        manifest,
        field_label,
        PACKAGING_VALUES,
        violations,
    )
    seen: dict[str, int] = {}
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip() or entry.strip() != entry:
            continue
        previous_index = seen.get(entry)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {field_label}[{index}] {entry} "
                f"duplicates {field_label}[{previous_index}]"
            )
            continue
        seen[entry] = index
