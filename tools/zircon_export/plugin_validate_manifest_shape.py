"""Root manifest scalar shape validation for plugin packages."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_trimmed_string


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_MANIFEST_ROOT_FIELDS = frozenset(
    """
    asset_importers asset_roots capabilities capability_statuses category components
    content_roots default_packaging dependencies description display_name distribution
    event_catalogs feature_extensions geometry_sources id maturity modules optional_features
    options package_company package_kind package_name package_prefix provides_interfaces
    sdk_api_version shader_permutation shading_models supported_platforms supported_targets
    ui_components version
    """.split()
)
PLUGIN_VALIDATE_MANIFEST_VERSION_FIELDS = ("version", "sdk_api_version")
PLUGIN_VALIDATE_MANIFEST_SEMVER_COMPONENTS = ("major", "minor", "patch")
PLUGIN_VALIDATE_U32_MAX = (1 << 32) - 1
PLUGIN_VALIDATE_MANIFEST_ID_CHARSET_MESSAGE = "must contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments"
PLUGIN_VALIDATE_MANIFEST_ID_START_MESSAGE = "must start with a lowercase ASCII letter"
PLUGIN_VALIDATE_MANIFEST_ID_UNDERSCORE_MESSAGE = "segments must not end with an underscore or contain repeated underscores"
PLUGIN_VALIDATE_MANIFEST_SEMVER_SHAPE_MESSAGE = "must use MAJOR.MINOR.PATCH form"


def validate_plugin_manifest_shape(
    *,
    plugin_manifest_path: Path | None,
    package_label: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    validate_plugin_manifest_known_fields(manifest, package_label, diagnostics)
    plugin_validate_manifest_identity(manifest, package_label, diagnostics)
    for field_name in PLUGIN_VALIDATE_MANIFEST_VERSION_FIELDS:
        plugin_validate_manifest_semver(
            manifest,
            field_name,
            package_label,
            diagnostics,
        )
    plugin_validate_trimmed_string(
        manifest,
        "display_name",
        f"plugin {package_label} display_name",
        diagnostics,
    )


def validate_plugin_manifest_known_fields(
    manifest: Manifest, package_label: str, diagnostics: Diagnostics
) -> None:
    for field in sorted(manifest):
        if field not in PLUGIN_VALIDATE_MANIFEST_ROOT_FIELDS:
            diagnostics.append(
                f"plugin {package_label} {field} is not a known manifest root field"
            )


def plugin_validate_manifest_identity(
    manifest: Manifest,
    package_label: str,
    diagnostics: Diagnostics,
) -> None:
    value = plugin_validate_trimmed_string(
        manifest,
        "id",
        f"plugin {package_label} id",
        diagnostics,
    )
    if value is None:
        return
    segments = value.split(".")
    if any(not segment for segment in segments) or any(
        not plugin_validate_manifest_identity_char(char)
        for segment in segments
        for char in segment
    ):
        diagnostics.append(
            f"plugin {package_label} id {value} "
            f"{PLUGIN_VALIDATE_MANIFEST_ID_CHARSET_MESSAGE}"
        )
    if not ("a" <= value[0] <= "z"):
        diagnostics.append(
            f"plugin {package_label} id {value} "
            f"{PLUGIN_VALIDATE_MANIFEST_ID_START_MESSAGE}"
        )
    if any(segment.endswith("_") or "__" in segment for segment in segments):
        diagnostics.append(
            f"plugin {package_label} id {value} "
            f"{PLUGIN_VALIDATE_MANIFEST_ID_UNDERSCORE_MESSAGE}"
        )


def plugin_validate_manifest_identity_char(char: str) -> bool:
    return ("a" <= char <= "z") or char.isdigit() or char in {"_", "."}


def plugin_validate_manifest_semver(
    manifest: Manifest,
    field_name: str,
    package_label: str,
    diagnostics: Diagnostics,
) -> None:
    value = plugin_validate_trimmed_string(
        manifest,
        field_name,
        f"plugin {package_label} {field_name}",
        diagnostics,
    )
    if value is None:
        return
    segments = value.split(".")
    if len(segments) != len(PLUGIN_VALIDATE_MANIFEST_SEMVER_COMPONENTS):
        diagnostics.append(
            f"plugin {package_label} {field_name} {value} "
            f"{PLUGIN_VALIDATE_MANIFEST_SEMVER_SHAPE_MESSAGE}"
        )
        return
    for component_name, segment in zip(
        PLUGIN_VALIDATE_MANIFEST_SEMVER_COMPONENTS,
        segments,
    ):
        plugin_validate_manifest_semver_component(
            package_label,
            field_name,
            value,
            component_name,
            segment,
            diagnostics,
        )


def plugin_validate_manifest_semver_component(
    package_label: str,
    field_name: str,
    value: str,
    component_name: str,
    segment: str,
    diagnostics: Diagnostics,
) -> None:
    if not segment.isascii() or not segment.isdigit():
        diagnostics.append(
            f"plugin {package_label} {field_name} {value} {component_name} "
            f"component {segment} must contain ASCII digits"
        )
        return
    if len(segment) > 1 and segment.startswith("0"):
        diagnostics.append(
            f"plugin {package_label} {field_name} {value} {component_name} "
            f"component {segment} must not use leading zeroes"
        )
        return
    if int(segment) > PLUGIN_VALIDATE_U32_MAX:
        diagnostics.append(
            f"plugin {package_label} {field_name} {value} {component_name} "
            f"component {segment} must fit in u32"
        )
