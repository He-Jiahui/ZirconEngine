"""Distribution scalar field checks for plugin validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import (
    plugin_validate_int,
    plugin_validate_trimmed_string,
)


PLUGIN_VALIDATE_ABI_VERSION_V3 = 3


class PluginValidateDistributionScalars:
    def __init__(
        self,
        *,
        dist_crate: str | None,
        abi_version: int | None,
    ) -> None:
        self.dist_crate = dist_crate
        self.abi_version = abi_version


def plugin_validate_distribution_scalars(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> PluginValidateDistributionScalars:
    return PluginValidateDistributionScalars(
        dist_crate=plugin_validate_distribution_dist_crate(
            distribution,
            package_id,
            diagnostics,
            distribution_label=distribution_label,
        ),
        abi_version=plugin_validate_distribution_abi_version(
            distribution,
            package_id,
            diagnostics,
            distribution_label=distribution_label,
        ),
    )


def plugin_validate_distribution_dist_crate(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> str | None:
    label = distribution_label or f"plugin {package_id} distribution"
    return plugin_validate_trimmed_string(
        distribution,
        "dist_crate",
        f"{label}.dist_crate",
        diagnostics,
    )


def plugin_validate_distribution_abi_version(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> int | None:
    distribution_label = distribution_label or f"plugin {package_id} distribution"
    label = f"{distribution_label}.abi_version"
    abi_version = plugin_validate_int(
        distribution,
        "abi_version",
        label,
        diagnostics,
    )
    if abi_version is None:
        return None
    if abi_version != PLUGIN_VALIDATE_ABI_VERSION_V3:
        diagnostics.append(f"{label} must be {PLUGIN_VALIDATE_ABI_VERSION_V3}")
        return None
    return abi_version
