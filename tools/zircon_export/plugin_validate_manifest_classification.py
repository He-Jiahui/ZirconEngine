"""Root manifest classification validation for plugin packages."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_trimmed_string


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_MANIFEST_CATEGORY_VALUES = (
    "asset_importer",
    "authoring",
    "diagnostics",
    "platform",
    "rendering",
    "runtime",
    "sdk",
)
PLUGIN_VALIDATE_MANIFEST_CATEGORY_UNSUPPORTED_MESSAGE = (
    "is unsupported; expected one of asset_importer, authoring, diagnostics, platform, rendering, runtime, sdk"
)
PLUGIN_VALIDATE_MANIFEST_MATURITY_VALUES = ("stable", "beta", "experimental")
PLUGIN_VALIDATE_MANIFEST_MATURITY_UNSUPPORTED_MESSAGE = (
    "is unsupported; expected one of stable, beta, experimental"
)


def validate_plugin_manifest_classification(
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
    validate_plugin_manifest_category(manifest, package_label, diagnostics)
    validate_plugin_manifest_maturity(manifest, package_label, diagnostics)


def validate_plugin_manifest_category(
    manifest: Manifest,
    package_label: str,
    diagnostics: Diagnostics,
) -> None:
    value = manifest.get("category")
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return
    category = value
    if category not in PLUGIN_VALIDATE_MANIFEST_CATEGORY_VALUES:
        diagnostics.append(
            f"plugin {package_label} category {category} "
            f"{PLUGIN_VALIDATE_MANIFEST_CATEGORY_UNSUPPORTED_MESSAGE}"
        )


def validate_plugin_manifest_maturity(
    manifest: Manifest,
    package_label: str,
    diagnostics: Diagnostics,
) -> None:
    maturity = plugin_validate_trimmed_string(
        manifest,
        "maturity",
        f"plugin {package_label} maturity",
        diagnostics,
    )
    if maturity is None:
        return
    if maturity not in PLUGIN_VALIDATE_MANIFEST_MATURITY_VALUES:
        diagnostics.append(
            f"plugin {package_label} maturity {maturity} "
            f"{PLUGIN_VALIDATE_MANIFEST_MATURITY_UNSUPPORTED_MESSAGE}"
        )
