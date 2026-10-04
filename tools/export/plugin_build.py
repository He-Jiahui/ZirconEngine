"""Single-plugin standalone build command."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Sequence

from .plugin_build_command import (
    default_target_dir,
    plugin_build_cargo_command,
    plugin_build_features,
    run_plugin_build_command,
    shell_join,
)
from .native_build_workspace import native_dynamic_cdylib_crate_index
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    native_dynamic_package_directory,
)
from .native_signing import native_dynamic_signing_command_template
from .plugin_build_package import materialize_plugin_build_package
from .plugin_build_preflight import (
    plugin_build_failure_report,
    plugin_build_optional_trimmed_string,
    plugin_build_string_array,
    plugin_distribution_abi_version,
    plugin_distribution_dist_crate,
)
from .plugin_package_source import (
    default_repo_root,
    resolve_plugin_package_path,
    resolve_plugin_package_source,
)


PLUGIN_BUILD_DEFAULT_OUT = "zircon-plugin-build"
PLUGIN_BUILD_DEFAULT_MODE = "debug"
PLUGIN_BUILD_DIST_FORM = "dist"


def parse_plugin_build_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="zircon_export plugin build",
        description="Build one standalone Zircon plugin package.",
    )
    parser.add_argument("plugin_id", help="Plugin package id from plugin.toml.")
    parser.add_argument(
        "--form",
        choices=(PLUGIN_BUILD_DIST_FORM,),
        default=PLUGIN_BUILD_DIST_FORM,
        help="Standalone package form. Default: dist.",
    )
    parser.add_argument(
        "--platform",
        "--target-platform",
        dest="target_platform",
        default=None,
        help="Target platform id, for example windows-x86_64.",
    )
    parser.add_argument(
        "--mode",
        choices=("debug", "release"),
        default=PLUGIN_BUILD_DEFAULT_MODE,
        help="Cargo build mode. Default: debug.",
    )
    parser.add_argument(
        "--out",
        "--output",
        default=PLUGIN_BUILD_DEFAULT_OUT,
        help=f"Output directory for package folders. Default: {PLUGIN_BUILD_DEFAULT_OUT}.",
    )
    parser.add_argument(
        "--repo-root",
        default=None,
        help="Repository root. Default: auto-detect from this package.",
    )
    parser.add_argument(
        "--cargo",
        default="cargo",
        help="Cargo executable. Default: cargo.",
    )
    parser.add_argument(
        "--packer",
        default=None,
        help="Prebuilt zircon_export_pack executable. Default: cargo run zircon_export_pack.",
    )
    parser.add_argument(
        "--sign-command",
        "--native-dynamic-sign-command",
        dest="sign_command",
        default=None,
        help="External signer executable for the loadable plugin artifact.",
    )
    parser.add_argument(
        "--sign-arg",
        "--native-dynamic-sign-arg",
        dest="sign_arg",
        action="append",
        default=[],
        help=(
            "Argument appended to --sign-command. May be repeated; supports "
            "{artifact}, {package_id}, {package_dir}, {target_platform}, "
            "and {signing_profile}."
        ),
    )
    parser.add_argument(
        "--sign-profile",
        "--native-dynamic-sign-profile",
        dest="sign_profile",
        default=None,
        help="Audit label for the plugin signing profile.",
    )
    parser.add_argument(
        "--sign-platform",
        "--native-dynamic-sign-platform",
        dest="sign_platform",
        action="append",
        default=[],
        help="Allowed target platform for --sign-command. May be repeated.",
    )
    parser.add_argument(
        "--target-dir",
        default=None,
        help="Isolated Cargo target directory. Default: <out>/.target/<plugin-id>.",
    )
    parser.add_argument(
        "--build-feature",
        action="append",
        default=[],
        help="Additional Cargo feature for the dist crate. May be repeated.",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Pass --offline to Cargo.",
    )
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Do not pass --locked to Cargo. Locked mode is the default.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the selected build command without executing it.",
    )
    return parser.parse_args(argv)


def run_plugin_build(args: argparse.Namespace) -> int:
    diagnostics: list[str] = []
    repo_root = (
        resolve_plugin_package_path("repo_root", Path(args.repo_root), diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    out_root = resolve_plugin_package_path("out", Path(args.out), diagnostics)
    plugin_root = repo_root / "zircon_plugins" if repo_root else None
    workspace_manifest = plugin_root / "Cargo.toml" if plugin_root else None
    build_source = (
        resolve_plugin_package_source(plugin_root, args.plugin_id, diagnostics)
        if plugin_root is not None
        else None
    )
    plugin_manifest_path = (
        build_source.plugin_manifest_path if build_source is not None else None
    )
    package_id = build_source.package_id if build_source is not None else args.plugin_id
    distribution = build_source.distribution if build_source is not None else None
    package_manifest_text = (
        build_source.package_manifest_text if build_source is not None else None
    )
    dist_crate = plugin_distribution_dist_crate(distribution, package_id, diagnostics)
    abi_version = plugin_distribution_abi_version(distribution, package_id, diagnostics)
    features = plugin_build_features(args.build_feature, diagnostics)
    signing_enabled = args.sign_command is not None
    signing_command = plugin_build_optional_trimmed_string(
        args.sign_command,
        "plugin build signing command",
        diagnostics,
    )
    signing_args = plugin_build_string_array(
        args.sign_arg,
        "plugin build signing args",
        diagnostics,
    )
    signing_profile = plugin_build_optional_trimmed_string(
        args.sign_profile,
        "plugin build signing profile",
        diagnostics,
    )
    signing_platforms = plugin_build_string_array(
        args.sign_platform,
        "plugin build signing platforms",
        diagnostics,
        lowercase=True,
    )
    signing_command_template = native_dynamic_signing_command_template(
        command=signing_command,
        extra_args=signing_args,
    )
    if signing_enabled and not signing_command_template:
        diagnostics.append("plugin build signing command is enabled but has no command parts")
    packer = (
        resolve_plugin_package_path("packer", Path(args.packer), diagnostics)
        if args.packer
        else None
    )
    target_dir = resolve_plugin_package_path(
        "target_dir",
        Path(args.target_dir) if args.target_dir else default_target_dir(out_root, package_id),
        diagnostics,
    )
    crate_index = (
        native_dynamic_cdylib_crate_index(workspace_manifest, diagnostics)
        if workspace_manifest is not None
        else {}
    )
    if dist_crate and dist_crate not in crate_index:
        diagnostics.append(
            f"plugin {package_id} distribution dist_crate {dist_crate} is not a cdylib workspace member"
        )
    command = (
        plugin_build_cargo_command(
            cargo=args.cargo,
            workspace_manifest=workspace_manifest,
            dist_crate=dist_crate,
            target_dir=target_dir,
            mode=args.mode,
            locked=not args.no_locked,
            offline=args.offline,
            features=features,
        )
        if workspace_manifest is not None and dist_crate and target_dir is not None
        else []
    )

    print(f"zircon_export plugin build id={args.plugin_id} form={args.form}")
    print(f"repo_root={repo_root if repo_root else '<invalid>'}")
    print(f"plugin_manifest={plugin_manifest_path if plugin_manifest_path else '<invalid>'}")
    print(f"out={out_root if out_root else '<invalid>'}")
    print(f"target_dir={target_dir if target_dir else '<invalid>'}")
    print(shell_join(command) if command else "command=<skipped>")
    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if diagnostics else 0
    if diagnostics:
        print(json.dumps(plugin_build_failure_report(args, diagnostics), indent=2))
        return 2
    if (
        repo_root is None
        or out_root is None
        or plugin_manifest_path is None
        or distribution is None
        or dist_crate is None
        or abi_version is None
        or workspace_manifest is None
        or target_dir is None
    ):
        diagnostics.append("plugin build preflight did not resolve all required inputs")
        print(json.dumps(plugin_build_failure_report(args, diagnostics), indent=2))
        return 2

    completed = run_plugin_build_command(command, repo_root, diagnostics)
    if completed is None or completed.returncode != 0:
        print(json.dumps(plugin_build_failure_report(args, diagnostics), indent=2))
        return completed.returncode if completed and completed.returncode != 0 else 2

    package_dir = materialize_plugin_build_package(
        out_root=out_root,
        package_id=package_id,
        plugin_manifest_path=plugin_manifest_path,
        package_manifest_text=package_manifest_text,
        repo_root=repo_root,
        target_dir=target_dir,
        dist_crate=dist_crate,
        mode=args.mode,
        target_platform=args.target_platform,
        abi_version=abi_version,
        distribution=distribution,
        cargo=args.cargo,
        locked=not args.no_locked,
        offline=args.offline,
        packer=packer,
        signing_enabled=signing_enabled,
        signing_command_template=signing_command_template,
        signing_profile=signing_profile,
        signing_platforms=signing_platforms,
        diagnostics=diagnostics,
    )
    if package_dir is None or diagnostics:
        print(json.dumps(plugin_build_failure_report(args, diagnostics), indent=2))
        return 2

    report = {
        "command": "plugin build",
        "plugin_id": package_id,
        "form": args.form,
        "target_platform": args.target_platform,
        "mode": args.mode,
        "dist_crate": dist_crate,
        "package_dir": str(package_dir),
        "loader_manifest": str(out_root / NATIVE_DYNAMIC_LOADER_MANIFEST),
        "signature": str(
            package_dir / f"{native_dynamic_package_directory(package_id)}.sig"
        ),
        "diagnostics": [],
        "fatal": False,
    }
    print(json.dumps(report, indent=2))
    return 0
