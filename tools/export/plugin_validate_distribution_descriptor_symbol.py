"""Distribution descriptor symbol checks for plugin validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_trimmed_string


PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3 = "zircon_native_plugin_descriptor_v3"


def plugin_validate_descriptor_symbol(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> None:
    distribution_label = distribution_label or f"plugin {package_id} distribution"
    label = f"{distribution_label}.descriptor_symbol"
    descriptor_symbol = plugin_validate_trimmed_string(
        distribution,
        "descriptor_symbol",
        label,
        diagnostics,
    )
    if descriptor_symbol is None:
        return
    if descriptor_symbol != PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3:
        diagnostics.append(
            f"{label} must equal {PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3}"
        )
