"""Feature-provider generated distribution schema validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_distribution_assets import plugin_validate_distribution_assets
from .plugin_validate_distribution_descriptor_symbol import (
    plugin_validate_descriptor_symbol,
)
from .plugin_validate_distribution_engine_compat import plugin_validate_engine_compat
from .plugin_validate_distribution_entries import plugin_validate_distribution_entries
from .plugin_validate_distribution_packaging import (
    plugin_validate_distribution_packaging,
)
from .plugin_validate_distribution_scalars import plugin_validate_distribution_scalars


def validate_plugin_feature_provider_distribution_schema(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} generated distribution"
    plugin_validate_distribution_packaging(
        distribution, package_id, diagnostics, distribution_label=label
    )
    plugin_validate_engine_compat(
        distribution,
        package_id,
        diagnostics,
        engine_version=None,
        distribution_label=label,
    )
    plugin_validate_distribution_scalars(
        distribution, package_id, diagnostics, distribution_label=label
    )
    plugin_validate_descriptor_symbol(
        distribution, package_id, diagnostics, distribution_label=label
    )
    plugin_validate_distribution_entries(
        distribution, package_id, diagnostics, distribution_label=label
    )
    plugin_validate_distribution_assets(
        distribution, package_id, diagnostics, distribution_label=label
    )
