"""Distribution runtime/editor entry checks for plugin validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_optional_trimmed_string


class PluginValidateDistributionEntries:
    def __init__(
        self,
        *,
        runtime_entry: str | None,
        editor_entry: str | None,
    ) -> None:
        self.runtime_entry = runtime_entry
        self.editor_entry = editor_entry


def plugin_validate_distribution_entries(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> PluginValidateDistributionEntries:
    label = distribution_label or f"plugin {package_id} distribution"
    runtime_entry = plugin_validate_optional_trimmed_string(
        distribution,
        "runtime_entry",
        f"{label}.runtime_entry",
        diagnostics,
    )
    editor_entry = plugin_validate_optional_trimmed_string(
        distribution,
        "editor_entry",
        f"{label}.editor_entry",
        diagnostics,
    )
    if runtime_entry is None and editor_entry is None:
        diagnostics.append(f"{label} must declare runtime_entry or editor_entry")
    return PluginValidateDistributionEntries(
        runtime_entry=runtime_entry,
        editor_entry=editor_entry,
    )
