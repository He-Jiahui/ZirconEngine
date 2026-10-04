"""Single-target orchestration for plugin validation."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

from .plugin_package_source import resolve_plugin_package_source
from .plugin_validate_common import PLUGIN_VALIDATE_FEATURE_SOURCE, PLUGIN_VALIDATE_ROOT_SOURCE
from .plugin_validate_asset_importers import validate_plugin_asset_importers
from .plugin_validate_capabilities import validate_plugin_capabilities
from .plugin_validate_capability_statuses import validate_plugin_capability_statuses
from .plugin_validate_components import validate_plugin_components
from .plugin_validate_dependencies import validate_plugin_dependencies
from .plugin_validate_default_packaging import validate_plugin_default_packaging
from .plugin_validate_distribution_contract import validate_plugin_distribution
from .plugin_validate_distribution_modules import validate_plugin_distribution_modules
from .plugin_validate_dist_crate import validate_plugin_dist_crate_workspace_member
from .plugin_validate_event_catalogs import validate_plugin_event_catalogs
from .plugin_validate_feature_extensions import validate_plugin_feature_extensions
from .plugin_validate_feature_provider import validate_plugin_feature_provider_package_projection
from .plugin_validate_layout import validate_plugin_layout
from .plugin_validate_manifest_classification import validate_plugin_manifest_classification
from .plugin_validate_manifest_shape import validate_plugin_manifest_shape
from .plugin_validate_modules import validate_plugin_modules
from .plugin_validate_optional_feature_dependencies import validate_plugin_optional_feature_dependencies
from .plugin_validate_optional_feature_distribution import validate_plugin_optional_feature_distribution
from .plugin_validate_optional_features import validate_plugin_optional_features
from .plugin_validate_options import validate_plugin_options
from .plugin_validate_package_kind import validate_plugin_package_kind
from .plugin_validate_report import plugin_validate_report
from .plugin_validate_retired_ui_assets import validate_plugin_target_retired_ui_asset_files


def plugin_validate_single_report(
    *,
    args: argparse.Namespace,
    repo_root: Path | None,
    plugin_root: Path | None,
    workspace_manifest: Path | None,
    crate_index: dict[str, dict[str, Any]],
    workspace_crate_index: dict[str, dict[str, Any]],
    engine_version: str | None,
    requested_plugin_id: str,
    diagnostics: list[str] | None = None,
    scan_retired_ui_assets: bool = True,
) -> dict[str, Any]:
    diagnostics = list(diagnostics or [])
    build_source = resolve_plugin_package_source(plugin_root, requested_plugin_id, diagnostics) if plugin_root is not None else None
    plugin_manifest_path = build_source.plugin_manifest_path if build_source is not None else None
    package_id = build_source.package_id if build_source is not None else requested_plugin_id
    source_kind = PLUGIN_VALIDATE_FEATURE_SOURCE if build_source is not None and build_source.package_manifest_text is not None else PLUGIN_VALIDATE_ROOT_SOURCE
    distribution = build_source.distribution if build_source is not None else None
    distribution_contract = validate_plugin_distribution(
        distribution,
        package_id,
        diagnostics,
        plugin_manifest_path=plugin_manifest_path,
        engine_version=engine_version,
    )
    validate_plugin_feature_provider_package_projection(
        plugin_manifest_path=plugin_manifest_path,
        package_manifest_text=build_source.package_manifest_text if build_source is not None else None,
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    dist_crate = distribution_contract.dist_crate
    validate_plugin_distribution_modules(
        plugin_manifest_path=plugin_manifest_path,
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        source_kind=source_kind,
        dist_crate=dist_crate,
        runtime_entry=distribution_contract.runtime_entry,
        editor_entry=distribution_contract.editor_entry,
        diagnostics=diagnostics,
    )
    if source_kind == PLUGIN_VALIDATE_ROOT_SOURCE:
        validate_plugin_manifest_classification(plugin_manifest_path=plugin_manifest_path, package_label=requested_plugin_id, diagnostics=diagnostics)
        validate_plugin_manifest_shape(plugin_manifest_path=plugin_manifest_path, package_label=requested_plugin_id, diagnostics=diagnostics)
        validate_plugin_package_kind(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_capabilities(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_optional_features(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_feature_extensions(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_default_packaging(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_layout(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_optional_feature_dependencies(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            diagnostics=diagnostics,
        )
        validate_plugin_optional_feature_distribution(plugin_manifest_path=plugin_manifest_path, package_id=package_id, engine_version=engine_version, diagnostics=diagnostics)
        validate_plugin_capability_statuses(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
        validate_plugin_components(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            diagnostics=diagnostics,
        )
        validate_plugin_modules(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            workspace_crate_index=workspace_crate_index,
            diagnostics=diagnostics,
        )
        validate_plugin_dependencies(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            diagnostics=diagnostics,
        )
        validate_plugin_event_catalogs(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            diagnostics=diagnostics,
        )
        validate_plugin_options(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            diagnostics=diagnostics,
        )
        validate_plugin_asset_importers(
            plugin_manifest_path=plugin_manifest_path,
            plugin_root=plugin_root,
            package_id=package_id,
            diagnostics=diagnostics,
        )
    if scan_retired_ui_assets: validate_plugin_target_retired_ui_asset_files(plugin_manifest_path=plugin_manifest_path, package_id=package_id, diagnostics=diagnostics)
    dist_crate_manifest = validate_plugin_dist_crate_workspace_member(crate_index, package_id, dist_crate, diagnostics)

    return plugin_validate_report(
        args=args,
        requested_plugin_id=requested_plugin_id,
        repo_root=repo_root,
        workspace_manifest=workspace_manifest,
        plugin_manifest_path=plugin_manifest_path,
        engine_version=engine_version,
        package_id=package_id,
        source_kind=source_kind,
        dist_crate=dist_crate,
        dist_crate_manifest=dist_crate_manifest,
        abi_version=distribution_contract.abi_version,
        diagnostics=diagnostics,
    )
