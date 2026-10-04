"""asset_importers numeric range checks for plugin validation."""

from __future__ import annotations

from typing import Any

Importer = dict[str, Any]
Diagnostics = list[str]

I32_MIN = -(2**31)
I32_MAX = 2**31 - 1
U32_MAX = 2**32 - 1


def validate_plugin_asset_importer_numbers(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    validate_plugin_asset_importer_priority_range(
        importer.get("priority"), importer_label, diagnostics
    )
    validate_plugin_asset_importer_version_range(
        importer.get("importer_version"), importer_label, diagnostics
    )


def validate_plugin_asset_importer_priority_range(
    value: Any, importer_label: str, diagnostics: Diagnostics
) -> None:
    if not isinstance(value, int) or isinstance(value, bool):
        return
    if value < I32_MIN or value > I32_MAX:
        diagnostics.append(f"{importer_label}.priority must fit i32")


def validate_plugin_asset_importer_version_range(
    value: Any, importer_label: str, diagnostics: Diagnostics
) -> None:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        return
    if value > U32_MAX:
        diagnostics.append(f"{importer_label}.importer_version must be a positive u32")
