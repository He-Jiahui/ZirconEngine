"""Single-plugin standalone validation command."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Sequence

from .native_build_workspace import (
    native_dynamic_cdylib_crate_index_from_workspace,
    native_dynamic_workspace_crate_index,
)
from .plugin_package_source import (
    default_repo_root,
    resolve_plugin_package_path,
)
from .plugin_validate_common import (
    PLUGIN_VALIDATE_DIST_FORM,
)
from .plugin_validate_asset_importer_global_ids import (
    validate_plugin_asset_importer_global_ids,
)
from .plugin_validate_engine_version import plugin_validate_engine_version
from .plugin_validate_option_global_keys import validate_plugin_option_global_keys
from .plugin_validate_retired_ui_assets import validate_plugin_retired_ui_asset_files
from .plugin_validate_report import (
    plugin_validate_all_report,
    render_plugin_validate_all_report,
    render_plugin_validate_report,
)
from .plugin_validate_single_target import plugin_validate_single_report
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
        choices=(PLUGIN_VALIDATE_DIST_FORM,),
        default=PLUGIN_VALIDATE_DIST_FORM,
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
        resolve_plugin_package_path("repo_root", Path(args.repo_root), diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    plugin_root = repo_root / "zircon_plugins" if repo_root else None
    workspace_manifest = plugin_root / "Cargo.toml" if plugin_root else None
    workspace_crate_index = (
        native_dynamic_workspace_crate_index(workspace_manifest, diagnostics)
        if workspace_manifest is not None
        else {}
    )
    crate_index = native_dynamic_cdylib_crate_index_from_workspace(workspace_crate_index)
    engine_version = plugin_validate_engine_version(repo_root, diagnostics)
    if args.all:
        report = plugin_validate_all_targets(
            args=args,
            repo_root=repo_root,
            plugin_root=plugin_root,
            workspace_manifest=workspace_manifest,
            crate_index=crate_index,
            workspace_crate_index=workspace_crate_index,
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
        workspace_crate_index=workspace_crate_index,
        engine_version=engine_version,
        requested_plugin_id=requested_plugin_id,
        diagnostics=diagnostics,
    )
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_plugin_validate_report(report), end="")
    return 2 if report["fatal"] else 0


def plugin_validate_all_targets(
    *,
    args: argparse.Namespace,
    repo_root: Path | None,
    plugin_root: Path | None,
    workspace_manifest: Path | None,
    crate_index: dict[str, dict[str, Any]],
    workspace_crate_index: dict[str, dict[str, Any]],
    engine_version: str | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    target_ids = (
        plugin_validate_discover_target_ids(plugin_root, diagnostics)
        if plugin_root is not None
        else []
    )
    if plugin_root is not None:
        validate_plugin_option_global_keys(plugin_root, diagnostics)
        validate_plugin_asset_importer_global_ids(plugin_root, diagnostics)
    if repo_root is not None:
        validate_plugin_retired_ui_asset_files(repo_root, diagnostics)
    items = [
        plugin_validate_single_report(
            args=args,
            repo_root=repo_root,
            plugin_root=plugin_root,
            workspace_manifest=workspace_manifest,
            crate_index=crate_index,
            workspace_crate_index=workspace_crate_index,
            engine_version=engine_version,
            requested_plugin_id=target_id,
            scan_retired_ui_assets=False,
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
