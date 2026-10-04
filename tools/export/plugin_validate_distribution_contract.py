"""Distribution manifest contract checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .plugin_validate_distribution_assets import (
    plugin_validate_distribution_assets,
)
from .plugin_validate_distribution_descriptor_symbol import (
    plugin_validate_descriptor_symbol,
)
from .plugin_validate_distribution_entries import (
    plugin_validate_distribution_entries,
)
from .plugin_validate_distribution_engine_compat import (
    plugin_validate_engine_compat,
)
from .plugin_validate_distribution_packaging import (
    plugin_validate_distribution_packaging,
)
from .plugin_validate_distribution_scalars import (
    plugin_validate_distribution_scalars,
)

PLUGIN_VALIDATE_DISTRIBUTION_FIELDS = frozenset(
    "abi_version assets default_packaging descriptor_symbol dist_crate editor_entry "
    "engine_compat forms runtime_entry".split()
)


class PluginValidateDistributionContract:
    def __init__(
        self,
        *,
        dist_crate: str | None,
        abi_version: int | None,
        runtime_entry: str | None,
        editor_entry: str | None,
    ) -> None:
        self.dist_crate = dist_crate
        self.abi_version = abi_version
        self.runtime_entry = runtime_entry
        self.editor_entry = editor_entry


def validate_plugin_distribution_known_fields(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> None:
    label = distribution_label or f"plugin {package_id} distribution"
    for field_name in distribution:
        if field_name in PLUGIN_VALIDATE_DISTRIBUTION_FIELDS:
            continue
        diagnostics.append(f"{label}.{field_name} is not a known distribution field")


def validate_plugin_distribution(
    distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
    *,
    plugin_manifest_path: Path | None = None,
    engine_version: str | None,
    distribution_label: str | None = None,
) -> PluginValidateDistributionContract:
    if distribution is None:
        return PluginValidateDistributionContract(
            dist_crate=None,
            abi_version=None,
            runtime_entry=None,
            editor_entry=None,
    )
    validate_plugin_distribution_known_fields(
        distribution,
        package_id,
        diagnostics,
        distribution_label=distribution_label,
    )
    plugin_validate_distribution_packaging(
        distribution, package_id, diagnostics, distribution_label=distribution_label
    )
    plugin_validate_engine_compat(
        distribution,
        package_id,
        diagnostics,
        engine_version,
        distribution_label=distribution_label,
    )
    scalars = plugin_validate_distribution_scalars(
        distribution, package_id, diagnostics, distribution_label=distribution_label
    )
    plugin_validate_descriptor_symbol(
        distribution, package_id, diagnostics, distribution_label=distribution_label
    )
    entries = plugin_validate_distribution_entries(
        distribution, package_id, diagnostics, distribution_label=distribution_label
    )
    plugin_validate_distribution_assets(
        distribution,
        package_id,
        diagnostics,
        plugin_manifest_path=plugin_manifest_path,
        distribution_label=distribution_label,
    )
    return PluginValidateDistributionContract(
        dist_crate=scalars.dist_crate,
        abi_version=scalars.abi_version,
        runtime_entry=entries.runtime_entry,
        editor_entry=entries.editor_entry,
    )
