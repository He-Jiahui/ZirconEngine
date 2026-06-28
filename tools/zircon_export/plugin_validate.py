"""Single-plugin standalone validation command."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Sequence

from .native_build import native_dynamic_cdylib_crate_index
from .plugin_build import (
    PLUGIN_BUILD_DIST_FORM,
    default_repo_root,
    plugin_distribution_abi_version,
    plugin_distribution_dist_crate,
    resolve_plugin_build_path,
    resolve_plugin_build_source,
)
from .plugin_validate_common import (
    PLUGIN_VALIDATE_FEATURE_SOURCE,
    PLUGIN_VALIDATE_ROOT_SOURCE,
)
from .plugin_validate_distribution_contract import validate_plugin_distribution
from .plugin_validate_distribution_modules import (
    validate_plugin_distribution_modules,
)
from .plugin_validate_dist_crate import validate_plugin_dist_crate_workspace_member
from .plugin_validate_engine_version import plugin_validate_engine_version
from .plugin_validate_feature_provider import (
    validate_plugin_feature_provider_package_projection,
)
from .plugin_validate_report import (
    plugin_validate_all_report,
    plugin_validate_report,
    render_plugin_validate_all_report,
    render_plugin_validate_report,
)
from .plugin_validate_target_discovery import plugin_validate_discover_target_ids


def parse_plugin_validate_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="zircon_export plugin validate",
        description="Validate one standalone Zircon plugin package contract.",
    )
    parser.add_argument(
        "plugin_id",
        nargs="?",
        help="Plugin package id from plugin.toml. Omit when --all is supplied.",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Validate every root distribution and feature-provider distribution.",
    )
    parser.add_argument(
        "--form",
        choices=(PLUGIN_BUILD_DIST_FORM,),
        default=PLUGIN_BUILD_DIST_FORM,
        help="Standalone package form. Default: dist.",
    )
    parser.add_argument(
        "--repo-root",
        default=None,
        help="Repository root. Default: auto-detect from this package.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit a machine-readable validation report.",
    )
    args = parser.parse_args(argv)
    if args.all == (args.plugin_id is not None):
        parser.error("pass exactly one plugin_id or --all")
    return args


def run_plugin_validate(args: argparse.Namespace) -> int:
    diagnostics: list[str] = []
    repo_root = (
        resolve_plugin_build_path("repo_root", Path(args.repo_root), diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    plugin_root = repo_root / "zircon_plugins" if repo_root else None
    workspace_manifest = plugin_root / "Cargo.toml" if plugin_root else None
    crate_index = (
        native_dynamic_cdylib_crate_index(workspace_manifest, diagnostics)
        if workspace_manifest is not None
        else {}
    )
    engine_version = plugin_validate_engine_version(repo_root, diagnostics)
    if args.all:
        report = plugin_validate_all_targets(
            args=args,
            repo_root=repo_root,
            plugin_root=plugin_root,
            workspace_manifest=workspace_manifest,
            crate_index=crate_index,
            engine_version=engine_version,
            diagnostics=diagnostics,
        )
        if args.json:
            print(json.dumps(report, indent=2, sort_keys=True))
        else:
            print(render_plugin_validate_all_report(report), end="")
        return 2 if report["fatal"] else 0
    requested_plugin_id = str(args.plugin_id)
    report = plugin_validate_single_report(
        args=args,
        repo_root=repo_root,
        plugin_root=plugin_root,
        workspace_manifest=workspace_manifest,
        crate_index=crate_index,
        engine_version=engine_version,
        requested_plugin_id=requested_plugin_id,
        diagnostics=diagnostics,
    )
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_plugin_validate_report(report), end="")
    return 2 if report["fatal"] else 0


def plugin_validate_single_report(
    *,
    args: argparse.Namespace,
    repo_root: Path | None,
    plugin_root: Path | None,
    workspace_manifest: Path | None,
    crate_index: dict[str, dict[str, Any]],
    engine_version: str | None,
    requested_plugin_id: str,
    diagnostics: list[str] | None = None,
) -> dict[str, Any]:
    diagnostics = list(diagnostics or [])
    build_source = (
        resolve_plugin_build_source(plugin_root, requested_plugin_id, diagnostics)
        if plugin_root is not None
        else None
    )
    plugin_manifest_path = (
        build_source.plugin_manifest_path if build_source is not None else None
    )
    package_id = (
        build_source.package_id if build_source is not None else requested_plugin_id
    )
    source_kind = (
        PLUGIN_VALIDATE_FEATURE_SOURCE
        if build_source is not None and build_source.package_manifest_text is not None
        else PLUGIN_VALIDATE_ROOT_SOURCE
    )
    distribution = build_source.distribution if build_source is not None else None
    runtime_entry, editor_entry = validate_plugin_distribution(
        distribution,
        package_id,
        diagnostics,
        engine_version=engine_version,
    )
    validate_plugin_feature_provider_package_projection(
        plugin_manifest_path=plugin_manifest_path,
        package_manifest_text=(
            build_source.package_manifest_text if build_source is not None else None
        ),
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    dist_crate = plugin_distribution_dist_crate(distribution, package_id, diagnostics)
    abi_version = plugin_distribution_abi_version(distribution, package_id, diagnostics)
    validate_plugin_distribution_modules(
        plugin_manifest_path=plugin_manifest_path,
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        source_kind=source_kind,
        dist_crate=dist_crate,
        runtime_entry=runtime_entry,
        editor_entry=editor_entry,
        diagnostics=diagnostics,
    )
    dist_crate_manifest = validate_plugin_dist_crate_workspace_member(
        crate_index,
        package_id,
        dist_crate,
        diagnostics,
    )

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
        abi_version=abi_version,
        diagnostics=diagnostics,
    )


def plugin_validate_all_targets(
    *,
    args: argparse.Namespace,
    repo_root: Path | None,
    plugin_root: Path | None,
    workspace_manifest: Path | None,
    crate_index: dict[str, dict[str, Any]],
    engine_version: str | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    target_ids = (
        plugin_validate_discover_target_ids(plugin_root, diagnostics)
        if plugin_root is not None
        else []
    )
    items = [
        plugin_validate_single_report(
            args=args,
            repo_root=repo_root,
            plugin_root=plugin_root,
            workspace_manifest=workspace_manifest,
            crate_index=crate_index,
            engine_version=engine_version,
            requested_plugin_id=target_id,
        )
        for target_id in target_ids
    ]
    return plugin_validate_all_report(
        args=args,
        repo_root=repo_root,
        workspace_manifest=workspace_manifest,
        engine_version=engine_version,
        diagnostics=diagnostics,
        items=items,
    )
