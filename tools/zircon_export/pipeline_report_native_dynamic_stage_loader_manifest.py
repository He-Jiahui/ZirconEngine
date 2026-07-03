"""NativeDynamic stage loader manifest package diagnostics."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .pipeline_report_native_dynamic_loader_manifest import (
    native_dynamic_loader_manifest_plugins_or_diagnostics,
    native_dynamic_loader_manifest_row_field_diagnostics,
)


def native_dynamic_loader_manifest_package_diagnostics(
    loader_manifest: Path,
    materialized_package_ids: list[str],
    *,
    expected_package_exports: list[dict[str, Any]] | None = None,
    expected_package_exports_label: str | None = None,
) -> list[str]:
    plugins, diagnostics = native_dynamic_loader_manifest_plugins_or_diagnostics(
        loader_manifest,
        label="native_dynamic loader_manifest",
    )
    if diagnostics:
        return diagnostics
    assert plugins is not None
    plugin_ids = [str(plugin["id"]) for plugin in plugins]
    if plugin_ids != materialized_package_ids:
        return [
            "native_dynamic loader_manifest plugin ids "
            f"{plugin_ids} do not match materialized package ids "
            f"{materialized_package_ids}"
        ]

    if expected_package_exports is None or expected_package_exports_label is None:
        return []
    return native_dynamic_loader_manifest_package_export_diagnostics(
        plugins,
        expected_package_exports,
        expected_package_exports_label,
    )


def native_dynamic_loader_manifest_package_export_diagnostics(
    loader_plugins: list[dict[str, Any]],
    expected_package_exports: list[dict[str, Any]],
    expected_label: str,
) -> list[str]:
    expected_exports_by_id = {
        str(package_export["package_id"]): package_export
        for package_export in expected_package_exports
        if isinstance(package_export.get("package_id"), str)
    }
    return native_dynamic_loader_manifest_row_field_diagnostics(
        loader_plugins,
        expected_exports_by_id,
        label="native_dynamic loader_manifest",
        expected_label=expected_label,
        require_fields=True,
    )
