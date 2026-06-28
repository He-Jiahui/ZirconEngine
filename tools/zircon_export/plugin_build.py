"""Single-plugin standalone build command."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence

from .native_build import (
    native_dynamic_cdylib_crate_index,
    platform_dynamic_library_name,
    read_toml,
    resolve_native_build_path,
)
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
    NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    native_dynamic_package_directory,
)
from .native_dynamic_payload import native_dynamic_package_payload_file_manifest
from .native_dynamic_templates import (
    native_dynamic_package_report_template,
    native_plugin_load_manifest_template,
    toml_string,
)
from .native_signing import (
    execute_native_dynamic_signing,
    native_dynamic_signing_command_template,
)


PLUGIN_BUILD_DEFAULT_OUT = "zircon-plugin-build"
PLUGIN_BUILD_DEFAULT_MODE = "debug"
PLUGIN_BUILD_DIST_FORM = "dist"
PLUGIN_BUILD_DIST_FEATURE = "dist"


@dataclass(frozen=True)
class PluginBuildSource:
    package_id: str
    plugin_manifest_path: Path
    distribution: dict[str, Any] | None
    package_manifest_text: str | None = None


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
        resolve_plugin_build_path("repo_root", Path(args.repo_root), diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    out_root = resolve_plugin_build_path("out", Path(args.out), diagnostics)
    plugin_root = repo_root / "zircon_plugins" if repo_root else None
    workspace_manifest = plugin_root / "Cargo.toml" if plugin_root else None
    build_source = (
        resolve_plugin_build_source(plugin_root, args.plugin_id, diagnostics)
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
        resolve_plugin_build_path("packer", Path(args.packer), diagnostics)
        if args.packer
        else None
    )
    target_dir = resolve_plugin_build_path(
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


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_plugin_build_path(
    label: str,
    path: Path | None,
    diagnostics: list[str],
) -> Path | None:
    if path is None:
        diagnostics.append(f"{label} is required")
        return None
    return resolve_native_build_path(label, path.expanduser(), diagnostics)


def default_target_dir(out_root: Path | None, plugin_id: str) -> Path:
    base = out_root if out_root is not None else Path(PLUGIN_BUILD_DEFAULT_OUT)
    return base / ".target" / native_dynamic_package_directory(plugin_id)


def resolve_plugin_build_source(
    plugin_root: Path,
    plugin_id: str,
    diagnostics: list[str],
) -> PluginBuildSource | None:
    direct = plugin_root / plugin_id / "plugin.toml"
    if direct.exists():
        manifest = read_toml(direct, diagnostics)
        return root_plugin_build_source(direct, manifest, plugin_id, diagnostics)
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        if manifest.get("id") == plugin_id:
            return root_plugin_build_source(manifest_path, manifest, plugin_id, diagnostics)
        feature_source = feature_provider_plugin_build_source(
            manifest_path,
            manifest,
            plugin_id,
            diagnostics,
        )
        if feature_source is not None:
            return feature_source
    diagnostics.append(f"plugin {plugin_id} plugin.toml was not found under {plugin_root}")
    return None


def root_plugin_build_source(
    plugin_manifest_path: Path,
    plugin_manifest: dict[str, Any] | None,
    requested_plugin_id: str,
    diagnostics: list[str],
) -> PluginBuildSource | None:
    package_id = plugin_package_id(plugin_manifest, requested_plugin_id, diagnostics)
    distribution = plugin_distribution(plugin_manifest, package_id, diagnostics)
    return PluginBuildSource(
        package_id=package_id,
        plugin_manifest_path=plugin_manifest_path,
        distribution=distribution,
    )


def feature_provider_plugin_build_source(
    plugin_manifest_path: Path,
    plugin_manifest: dict[str, Any],
    requested_plugin_id: str,
    diagnostics: list[str],
) -> PluginBuildSource | None:
    owner_plugin_id = plugin_manifest.get("id")
    optional_features = plugin_manifest.get("optional_features", [])
    if not isinstance(owner_plugin_id, str) or not owner_plugin_id.strip():
        return None
    if not isinstance(optional_features, list):
        return None
    for feature in optional_features:
        if not isinstance(feature, dict):
            continue
        feature_id = feature.get("id")
        if not isinstance(feature_id, str) or not feature_id.strip():
            continue
        provider_package_id = feature_provider_package_id(feature, feature_id)
        if requested_plugin_id not in {feature_id, provider_package_id}:
            continue
        if not provider_package_id:
            diagnostics.append(
                f"plugin feature {feature_id} provider_package_id must be a string"
            )
            return PluginBuildSource(
                package_id=requested_plugin_id,
                plugin_manifest_path=plugin_manifest_path,
                distribution=None,
            )
        distribution = feature_provider_distribution(
            feature,
            provider_package_id,
            diagnostics,
        )
        package_manifest_text = (
            feature_provider_package_manifest_template(
                owner_manifest=plugin_manifest,
                feature=feature,
                provider_package_id=provider_package_id,
                distribution=distribution,
            )
            if distribution is not None
            else None
        )
        return PluginBuildSource(
            package_id=provider_package_id,
            plugin_manifest_path=plugin_manifest_path,
            distribution=distribution,
            package_manifest_text=package_manifest_text,
        )
    return None


def feature_provider_package_id(feature: dict[str, Any], feature_id: str) -> str | None:
    provider_package_id = feature.get("provider_package_id")
    if provider_package_id is None:
        return native_dynamic_package_directory(feature_id)
    if not isinstance(provider_package_id, str) or not provider_package_id.strip():
        return None
    if provider_package_id.strip() != provider_package_id:
        return None
    return provider_package_id


def feature_provider_distribution(
    feature: dict[str, Any],
    provider_package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    feature_id = feature.get("id")
    distribution = feature.get("distribution")
    if not isinstance(distribution, dict):
        diagnostics.append(
            f"plugin feature {feature_id} provider {provider_package_id} has no distribution table"
        )
        return None
    return plugin_distribution_contract(distribution, provider_package_id, diagnostics)


def plugin_package_id(
    plugin_manifest: dict[str, Any] | None,
    requested_plugin_id: str,
    diagnostics: list[str],
) -> str:
    if plugin_manifest is None:
        return requested_plugin_id
    package_id = plugin_manifest.get("id")
    if not isinstance(package_id, str) or not package_id.strip():
        diagnostics.append(f"plugin {requested_plugin_id} plugin.toml id must be a string")
        return requested_plugin_id
    if package_id.strip() != package_id:
        diagnostics.append(f"plugin {requested_plugin_id} plugin.toml id must be trimmed")
        return requested_plugin_id
    if package_id != requested_plugin_id:
        diagnostics.append(
            f"plugin manifest id {package_id} does not match requested id {requested_plugin_id}"
        )
    return package_id


def plugin_distribution(
    plugin_manifest: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if plugin_manifest is None:
        return None
    distribution = plugin_manifest.get("distribution")
    if not isinstance(distribution, dict):
        diagnostics.append(f"plugin {package_id} has no [distribution] table")
        return None
    return plugin_distribution_contract(distribution, package_id, diagnostics)


def plugin_distribution_contract(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any]:
    forms = distribution.get("forms", [])
    if not isinstance(forms, list) or PLUGIN_BUILD_DIST_FORM not in forms:
        diagnostics.append(f"plugin {package_id} distribution.forms must include dist")
    return distribution


def plugin_distribution_dist_crate(
    distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> str | None:
    if distribution is None:
        return None
    dist_crate = distribution.get("dist_crate")
    if not isinstance(dist_crate, str) or not dist_crate.strip():
        diagnostics.append(f"plugin {package_id} distribution.dist_crate must be a string")
        return None
    if dist_crate.strip() != dist_crate:
        diagnostics.append(f"plugin {package_id} distribution.dist_crate must be trimmed")
        return None
    return dist_crate


def plugin_distribution_abi_version(
    distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> int | None:
    if distribution is None:
        return None
    abi_version = distribution.get("abi_version")
    if not isinstance(abi_version, int):
        diagnostics.append(f"plugin {package_id} distribution.abi_version must be an integer")
        return None
    if abi_version != 3:
        diagnostics.append(f"plugin {package_id} distribution.abi_version must be 3")
        return None
    return abi_version


def plugin_build_features(
    extra_features: list[str],
    diagnostics: list[str],
) -> list[str]:
    features = [PLUGIN_BUILD_DIST_FEATURE]
    for index, feature in enumerate(extra_features):
        label = f"plugin build features[{index}]"
        if not isinstance(feature, str) or not feature.strip():
            diagnostics.append(f"{label} must be a non-empty string")
            continue
        if feature.strip() != feature:
            diagnostics.append(f"{label} must be trimmed")
            continue
        if feature not in features:
            features.append(feature)
    return features


def plugin_build_optional_trimmed_string(
    value: object,
    field: str,
    diagnostics: list[str],
) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str):
        diagnostics.append(f"{field} must be a string")
        return None
    if not value or value.strip() != value:
        diagnostics.append(f"{field} must be a non-empty trimmed string")
        return None
    return value


def plugin_build_string_array(
    value: object,
    field: str,
    diagnostics: list[str],
    *,
    lowercase: bool = False,
) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        value = [value]
    values: list[str] = []
    seen: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str):
            diagnostics.append(f"{field}[{index}] must be a string")
            continue
        if not item or item.strip() != item:
            diagnostics.append(f"{field}[{index}] must be a non-empty trimmed string")
            continue
        normalized = item.lower() if lowercase else item
        if normalized in seen:
            continue
        seen.add(normalized)
        values.append(normalized)
    return values


def feature_provider_package_manifest_template(
    *,
    owner_manifest: dict[str, Any],
    feature: dict[str, Any],
    provider_package_id: str,
    distribution: dict[str, Any],
) -> str:
    feature_id = feature_string(feature, "id", provider_package_id)
    display_name = feature_string(feature, "display_name", feature_id)
    capabilities = feature_string_array(feature.get("capabilities"))
    supported_targets = feature_provider_supported_targets(owner_manifest, feature)
    supported_platforms = feature_string_array(owner_manifest.get("supported_platforms")) or [
        "windows",
        "linux",
        "macos",
    ]
    default_packaging = feature_string_array(
        distribution.get("default_packaging")
    ) or ["native_dynamic"]
    owner_plugin_id = feature_string(feature, "owner_plugin_id", owner_manifest.get("id", ""))
    runtime_module = feature_provider_runtime_module(feature, distribution)
    lines = [
        "# Generated by Zircon export. Feature provider package manifest.",
        f"id = {toml_string(provider_package_id)}",
        f"version = {toml_string(feature_string(owner_manifest, 'version', '0.1.0'))}",
        'package_kind = "feature_extension"',
        f"display_name = {toml_string(display_name + ' Provider')}",
        f"description = {toml_string(f'Native dynamic provider for optional feature {feature_id}.')}",
        f"sdk_api_version = {toml_string(feature_string(owner_manifest, 'sdk_api_version', '0.1.0'))}",
        f"category = {toml_string(feature_string(owner_manifest, 'category', 'runtime'))}",
        f"maturity = {toml_string(feature_string(owner_manifest, 'maturity', 'beta'))}",
        f"supported_targets = {toml_string_array(supported_targets)}",
        f"supported_platforms = {toml_string_array(supported_platforms)}",
        f"capabilities = {toml_string_array(capabilities)}",
        f"default_packaging = {toml_string_array(default_packaging)}",
        "",
        "[distribution]",
        f"forms = {toml_string_array(feature_string_array(distribution.get('forms')))}",
        f"default_packaging = {toml_string_array(default_packaging)}",
    ]
    abi_version = distribution.get("abi_version")
    if type(abi_version) is int:
        lines.append(f"abi_version = {abi_version}")
    for field in ("engine_compat", "dist_crate", "descriptor_symbol", "runtime_entry"):
        lines.append(f"{field} = {toml_string(feature_string(distribution, field, ''))}")
    editor_entry = distribution.get("editor_entry")
    if isinstance(editor_entry, str) and editor_entry.strip():
        lines.append(f"editor_entry = {toml_string(editor_entry)}")
    assets = feature_string_array(distribution.get("assets"))
    if assets:
        lines.append(f"assets = {toml_string_array(assets)}")

    lines.extend(
        [
            "",
            "[[feature_extensions]]",
            f"id = {toml_string(feature_id)}",
            f"display_name = {toml_string(display_name)}",
            f"owner_plugin_id = {toml_string(owner_plugin_id)}",
            f"capabilities = {toml_string_array(capabilities)}",
            f"default_packaging = {toml_string_array(default_packaging)}",
            f"enabled_by_default = {toml_bool(feature.get('enabled_by_default'))}",
            "",
        ]
    )
    for dependency in feature_provider_dependencies(feature):
        lines.extend(
            [
                "[[feature_extensions.dependencies]]",
                f"plugin_id = {toml_string(dependency['plugin_id'])}",
                f"capability = {toml_string(dependency['capability'])}",
                f"primary = {toml_bool(dependency['primary'])}",
                "",
            ]
        )
    lines.extend(
        [
            "[[feature_extensions.modules]]",
            f"name = {toml_string(runtime_module['name'])}",
            'kind = "runtime"',
            f"crate_name = {toml_string(runtime_module['crate_name'])}",
            f"target_modes = {toml_string_array(runtime_module['target_modes'])}",
            f"capabilities = {toml_string_array(runtime_module['capabilities'])}",
            "",
        ]
    )
    return "\n".join(lines)


def feature_provider_supported_targets(
    owner_manifest: dict[str, Any],
    feature: dict[str, Any],
) -> list[str]:
    runtime_module = first_feature_module(feature, "runtime")
    if runtime_module is not None:
        target_modes = feature_string_array(runtime_module.get("target_modes"))
        if target_modes:
            return target_modes
    return feature_string_array(owner_manifest.get("supported_targets")) or [
        "client_runtime",
        "editor_host",
    ]


def feature_provider_runtime_module(
    feature: dict[str, Any],
    distribution: dict[str, Any],
) -> dict[str, list[str] | str]:
    feature_id = feature_string(feature, "id", "feature")
    dist_crate = feature_string(distribution, "dist_crate", "")
    feature_capabilities = feature_string_array(feature.get("capabilities"))
    source_module = first_feature_module(feature, "runtime")
    if source_module is None:
        return {
            "name": f"{feature_id}.runtime",
            "crate_name": dist_crate,
            "target_modes": ["client_runtime", "editor_host"],
            "capabilities": feature_capabilities,
        }
    return {
        "name": feature_string(source_module, "name", f"{feature_id}.runtime"),
        "crate_name": dist_crate,
        "target_modes": feature_string_array(source_module.get("target_modes"))
        or ["client_runtime", "editor_host"],
        "capabilities": feature_string_array(source_module.get("capabilities"))
        or feature_capabilities,
    }


def first_feature_module(
    feature: dict[str, Any],
    module_kind: str,
) -> dict[str, Any] | None:
    modules = feature.get("modules")
    if not isinstance(modules, list):
        return None
    for module in modules:
        if isinstance(module, dict) and module.get("kind") == module_kind:
            return module
    return None


def feature_provider_dependencies(feature: dict[str, Any]) -> list[dict[str, object]]:
    dependencies = feature.get("dependencies")
    if not isinstance(dependencies, list):
        return []
    output: list[dict[str, object]] = []
    for dependency in dependencies:
        if not isinstance(dependency, dict):
            continue
        plugin_id = dependency.get("plugin_id")
        capability = dependency.get("capability")
        if not isinstance(plugin_id, str) or not isinstance(capability, str):
            continue
        output.append(
            {
                "plugin_id": plugin_id,
                "capability": capability,
                "primary": dependency.get("primary") is True,
            }
        )
    return output


def feature_string(mapping: dict[str, Any], field: str, default: object) -> str:
    value = mapping.get(field)
    if isinstance(value, str) and value.strip():
        return value
    return str(default)


def feature_string_array(value: object) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str) and item.strip() == item]


def toml_string_array(values: Sequence[str]) -> str:
    return "[" + ", ".join(toml_string(value) for value in values) + "]"


def plugin_build_cargo_command(
    *,
    cargo: str,
    workspace_manifest: Path,
    dist_crate: str,
    target_dir: Path,
    mode: str,
    locked: bool,
    offline: bool,
    features: list[str],
) -> list[str]:
    command = [
        cargo,
        "build",
        "--manifest-path",
        str(workspace_manifest),
        "-p",
        dist_crate,
        "--target-dir",
        str(target_dir),
        "--no-default-features",
        "--features",
        ",".join(features),
    ]
    if locked:
        command.append("--locked")
    if mode == "release":
        command.append("--release")
    if offline:
        command.append("--offline")
    return command


def run_plugin_build_command(
    command: list[str],
    repo_root: Path,
    diagnostics: list[str],
) -> subprocess.CompletedProcess[str] | None:
    try:
        completed = subprocess.run(
            command,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        diagnostics.append(f"plugin build cargo command could not start: {error}")
        return None
    if completed.returncode != 0:
        diagnostics.append(
            f"plugin build cargo command exited with code {completed.returncode}"
        )
        if completed.stderr:
            diagnostics.append(completed.stderr.strip())
    return completed


def materialize_plugin_build_package(
    *,
    out_root: Path,
    package_id: str,
    plugin_manifest_path: Path,
    package_manifest_text: str | None,
    repo_root: Path,
    target_dir: Path,
    dist_crate: str,
    mode: str,
    target_platform: str | None,
    abi_version: int,
    distribution: dict[str, Any],
    cargo: str,
    locked: bool,
    offline: bool,
    packer: Path | None,
    signing_enabled: bool,
    signing_command_template: list[str],
    signing_profile: str | None,
    signing_platforms: list[str],
    diagnostics: list[str],
) -> Path | None:
    directory = native_dynamic_package_directory(package_id)
    package_dir = out_root / directory
    resolved_out_root = resolve_plugin_build_path("out", out_root, diagnostics)
    resolved_package_dir = resolve_plugin_build_path(
        "plugin package directory",
        package_dir,
        diagnostics,
    )
    if resolved_out_root is None or resolved_package_dir is None:
        return None
    if not resolved_package_dir.is_relative_to(resolved_out_root):
        diagnostics.append(
            f"plugin package directory {resolved_package_dir} is outside output root {resolved_out_root}"
        )
        return None
    try:
        if resolved_package_dir.exists():
            shutil.rmtree(resolved_package_dir)
        resolved_package_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"plugin package directory {resolved_package_dir} could not be prepared: {error}"
        )
        return None

    built_artifact = target_dir / mode / platform_dynamic_library_name(
        dist_crate,
        target_platform,
    )
    if not built_artifact.exists() or not built_artifact.is_file():
        diagnostics.append(f"plugin build artifact {built_artifact} does not exist")
        return None
    loadable_name = platform_dynamic_library_name(directory, target_platform)
    try:
        native_loadable_dir = resolved_package_dir / "native"
        native_loadable_dir.mkdir(parents=True, exist_ok=True)
        package_manifest_destination = resolved_package_dir / "plugin.toml"
        if package_manifest_text is None:
            shutil.copy2(plugin_manifest_path, package_manifest_destination)
        else:
            package_manifest_destination.write_text(
                package_manifest_text,
                encoding="utf-8",
            )
        shutil.copy2(built_artifact, resolved_package_dir / loadable_name)
        shutil.copy2(
            built_artifact,
            native_loadable_dir / platform_dynamic_library_name(dist_crate, target_platform),
        )
    except OSError as error:
        diagnostics.append(f"plugin package files could not be copied: {error}")
        return None

    if not materialize_plugin_asset_pack(
        package_id=package_id,
        directory=directory,
        plugin_root=plugin_manifest_path.parent,
        repo_root=repo_root,
        package_dir=resolved_package_dir,
        target_dir=target_dir,
        distribution=distribution,
        cargo=cargo,
        locked=locked,
        offline=offline,
        packer=packer,
        diagnostics=diagnostics,
    ):
        return None

    package_export = {
        "package_id": package_id,
        "directory": directory,
        "path": directory,
        "manifest": f"{directory}/plugin.toml",
        "package_report": f"{directory}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}",
        "abi": plugin_build_abi_contract(abi_version, distribution),
    }
    signing = plugin_build_signing_audit(
        package_id=package_id,
        package_dir=resolved_package_dir,
        target_platform=target_platform,
        signing_enabled=signing_enabled,
        signing_command_template=signing_command_template,
        signing_profile=signing_profile,
        signing_platforms=signing_platforms,
        diagnostics=diagnostics,
    )
    if diagnostics:
        return None
    if not write_plugin_build_signature(
        package_id=package_id,
        directory=directory,
        package_dir=resolved_package_dir,
        target_platform=target_platform,
        signing=signing,
        diagnostics=diagnostics,
    ):
        return None
    payload_manifest = native_dynamic_package_payload_file_manifest(
        resolved_package_dir,
        diagnostics,
    )
    report_text = native_dynamic_package_report_template(
        package_export,
        payload_manifest,
    )
    try:
        (resolved_package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE).write_text(
            report_text,
            encoding="utf-8",
        )
    except OSError as error:
        diagnostics.append(f"plugin package report could not be written: {error}")
        return None
    if not write_plugin_build_load_manifest(
        out_root=resolved_out_root,
        package_export=package_export,
        diagnostics=diagnostics,
    ):
        return None
    return resolved_package_dir


def plugin_build_signing_audit(
    *,
    package_id: str,
    package_dir: Path,
    target_platform: str | None,
    signing_enabled: bool,
    signing_command_template: list[str],
    signing_profile: str | None,
    signing_platforms: list[str],
    diagnostics: list[str],
) -> dict[str, object]:
    if not signing_enabled:
        return {
            "enabled": False,
            "profile": signing_profile,
            "target_platform": target_platform,
            "allowed_platforms": signing_platforms,
            "platform_allowed": True,
            "fatal": False,
            "diagnostics": [],
            "package_count": 0,
            "packages": [],
        }

    signing_diagnostics: list[str] = []
    signing = execute_native_dynamic_signing(
        materialized_packages=[
            {
                "package_id": package_id,
                "destination": str(package_dir),
            }
        ],
        loadable_artifact_extensions=NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
        command_template=signing_command_template,
        target_platform=target_platform,
        signing_profile=signing_profile,
        allowed_platforms=signing_platforms,
        diagnostics=signing_diagnostics,
    )
    diagnostics.extend(signing_diagnostics)
    return signing


def write_plugin_build_signature(
    *,
    package_id: str,
    directory: str,
    package_dir: Path,
    target_platform: str | None,
    signing: dict[str, object],
    diagnostics: list[str],
) -> bool:
    loadable_manifest = plugin_build_loadable_file_manifest(package_dir, diagnostics)
    if not loadable_manifest:
        if not diagnostics:
            diagnostics.append(f"plugin {package_id} has no loadable artifact to sign")
        return False
    signature_path = package_dir / f"{directory}.sig"
    signature_text = plugin_build_signature_template(
        package_id=package_id,
        target_platform=target_platform,
        signing=signing,
        loadable_manifest=loadable_manifest,
    )
    try:
        signature_path.write_text(signature_text, encoding="utf-8")
    except OSError as error:
        diagnostics.append(f"plugin signature {signature_path} could not be written: {error}")
        return False
    return True


def plugin_build_loadable_file_manifest(
    package_dir: Path,
    diagnostics: list[str],
) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    try:
        file_paths = sorted(package_dir.rglob("*"))
    except OSError as error:
        diagnostics.append(f"plugin package directory {package_dir} could not be listed: {error}")
        return entries
    for file_path in file_paths:
        if not file_path.is_file():
            continue
        if file_path.suffix.lower() not in NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS:
            continue
        try:
            payload = file_path.read_bytes()
        except OSError as error:
            diagnostics.append(f"plugin loadable artifact {file_path} could not be read: {error}")
            continue
        entries.append(
            {
                "path": file_path.relative_to(package_dir).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return entries


def plugin_build_signature_template(
    *,
    package_id: str,
    target_platform: str | None,
    signing: dict[str, object],
    loadable_manifest: list[dict[str, object]],
) -> str:
    output = "# Generated by Zircon export. Native dynamic plugin signature/hash sidecar.\n"
    output += "format_version = 1\n"
    output += f"package_id = {toml_string(package_id)}\n"
    if target_platform is not None:
        output += f"target_platform = {toml_string(target_platform)}\n"
    output += f"loadable_artifact_count = {len(loadable_manifest)}\n"
    for entry in loadable_manifest:
        output += "\n[[loadable_artifacts]]\n"
        output += f"path = {toml_string(entry['path'])}\n"
        output += f"bytes = {entry['bytes']}\n"
        output += f"sha256 = {toml_string(entry['sha256'])}\n"

    output += "\n[signing]\n"
    output += f"enabled = {toml_bool(signing.get('enabled'))}\n"
    profile = signing.get("profile")
    if isinstance(profile, str):
        output += f"profile = {toml_string(profile)}\n"
    signing_target_platform = signing.get("target_platform")
    if isinstance(signing_target_platform, str):
        output += f"target_platform = {toml_string(signing_target_platform)}\n"
    output += f"platform_allowed = {toml_bool(signing.get('platform_allowed'))}\n"
    output += f"fatal = {toml_bool(signing.get('fatal'))}\n"
    package_count = signing.get("package_count")
    output += f"package_count = {package_count if type(package_count) is int else 0}\n"
    artifacts = plugin_build_signing_artifacts(signing)
    output += f"artifact_count = {len(artifacts)}\n"
    for artifact in artifacts:
        output += "\n[[signing.artifacts]]\n"
        output += f"path = {toml_string(artifact['path'])}\n"
        exit_code = artifact.get("exit_code")
        if type(exit_code) is int:
            output += f"exit_code = {exit_code}\n"
        before_sha256 = artifact.get("before_sha256")
        if isinstance(before_sha256, str):
            output += f"before_sha256 = {toml_string(before_sha256)}\n"
        after_sha256 = artifact.get("after_sha256")
        if isinstance(after_sha256, str):
            output += f"after_sha256 = {toml_string(after_sha256)}\n"
    return output


def plugin_build_signing_artifacts(
    signing: dict[str, object],
) -> list[dict[str, object]]:
    artifacts: list[dict[str, object]] = []
    packages = signing.get("packages")
    if not isinstance(packages, list):
        return artifacts
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_artifacts = package.get("artifacts")
        if not isinstance(package_artifacts, list):
            continue
        for artifact in package_artifacts:
            if not isinstance(artifact, dict):
                continue
            path = artifact.get("package_relative_artifact")
            if not isinstance(path, str):
                continue
            artifacts.append(
                {
                    "path": path,
                    "exit_code": artifact.get("exit_code"),
                    "before_sha256": artifact.get("before_sha256"),
                    "after_sha256": artifact.get("after_sha256"),
                }
            )
    return sorted(artifacts, key=lambda entry: str(entry["path"]))


def toml_bool(value: object) -> str:
    return "true" if value is True else "false"


def write_plugin_build_load_manifest(
    *,
    out_root: Path,
    package_export: dict[str, Any],
    diagnostics: list[str],
) -> bool:
    loader_manifests = [
        out_root / NATIVE_DYNAMIC_LOADER_MANIFEST,
        out_root / "plugins" / NATIVE_DYNAMIC_LOADER_MANIFEST,
    ]
    manifest_text = native_plugin_load_manifest_template([package_export])
    try:
        for loader_manifest in loader_manifests:
            loader_manifest.parent.mkdir(parents=True, exist_ok=True)
            loader_manifest.write_text(manifest_text, encoding="utf-8")
    except OSError as error:
        diagnostics.append(f"plugin load manifest {loader_manifest} could not be written: {error}")
        return False
    return True


def materialize_plugin_asset_pack(
    *,
    package_id: str,
    directory: str,
    plugin_root: Path,
    repo_root: Path,
    package_dir: Path,
    target_dir: Path,
    distribution: dict[str, Any],
    cargo: str,
    locked: bool,
    offline: bool,
    packer: Path | None,
    diagnostics: list[str],
) -> bool:
    asset_entries = plugin_asset_pack_entries(
        plugin_root,
        distribution,
        package_id,
        diagnostics,
    )
    if not asset_entries:
        return not diagnostics

    pack_path = package_dir / f"{directory}.zrpack"
    with tempfile.TemporaryDirectory(prefix=f"zircon-plugin-{directory}-pack-") as temp_dir:
        temp_root = Path(temp_dir)
        asset_manifest_path = temp_root / "assets.json"
        pack_report_path = temp_root / "pack-report.json"
        asset_manifest = {
            "roots": [entry["path"] for entry in asset_entries],
            "assets": asset_entries,
        }
        try:
            asset_manifest_path.write_text(
                json.dumps(asset_manifest, indent=2, sort_keys=True),
                encoding="utf-8",
            )
        except OSError as error:
            diagnostics.append(f"plugin asset manifest could not be written: {error}")
            return False

        command = plugin_asset_pack_command(
            package_id=package_id,
            cargo=cargo,
            repo_root=repo_root,
            target_dir=target_dir,
            asset_manifest_path=asset_manifest_path,
            pack_path=pack_path,
            pack_report_path=pack_report_path,
            locked=locked,
            offline=offline,
            packer=packer,
        )
        completed = run_plugin_asset_pack_command(command, repo_root, diagnostics)
        if completed is None or completed.returncode != 0:
            return False
        if not pack_path.exists() or not pack_path.is_file():
            diagnostics.append(f"plugin asset pack {pack_path} was not written")
            return False
        return plugin_asset_pack_report_is_clean(pack_report_path, diagnostics)


def plugin_asset_pack_entries(
    plugin_root: Path,
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> list[dict[str, str]]:
    assets = distribution.get("assets", [])
    if assets is None:
        return []
    if not isinstance(assets, list):
        diagnostics.append(f"plugin {package_id} distribution.assets must be an array")
        return []

    resolved_plugin_root = resolve_plugin_build_path("plugin root", plugin_root, diagnostics)
    if resolved_plugin_root is None:
        return []

    entries: list[dict[str, str]] = []
    seen_paths: set[str] = set()
    for index, raw_pattern in enumerate(assets):
        label = f"plugin {package_id} distribution.assets[{index}]"
        if not isinstance(raw_pattern, str) or not raw_pattern.strip():
            diagnostics.append(f"{label} must be a non-empty string")
            continue
        if raw_pattern.strip() != raw_pattern:
            diagnostics.append(f"{label} must be trimmed")
            continue
        pattern_path = Path(raw_pattern)
        if pattern_path.is_absolute() or ".." in pattern_path.parts:
            diagnostics.append(f"{label} must be a plugin-relative glob")
            continue

        matches = sorted(path for path in plugin_root.glob(raw_pattern) if path.is_file())
        if not matches:
            diagnostics.append(f"{label} matched no plugin asset files")
            continue
        for source_path in matches:
            resolved_source = resolve_plugin_build_path(
                "plugin asset source",
                source_path,
                diagnostics,
            )
            if resolved_source is None:
                continue
            try:
                relative_path = resolved_source.relative_to(resolved_plugin_root).as_posix()
            except ValueError:
                diagnostics.append(
                    f"plugin asset source {resolved_source} is outside plugin root {resolved_plugin_root}"
                )
                continue
            if relative_path in seen_paths:
                continue
            seen_paths.add(relative_path)
            entries.append({"path": relative_path, "source": str(resolved_source)})
    return sorted(entries, key=lambda entry: entry["path"])


def plugin_asset_pack_command(
    *,
    package_id: str,
    cargo: str,
    repo_root: Path,
    target_dir: Path,
    asset_manifest_path: Path,
    pack_path: Path,
    pack_report_path: Path,
    locked: bool,
    offline: bool,
    packer: Path | None,
) -> list[str]:
    packer_args = [
        "--profile",
        f"plugin-{package_id}",
        "--manifest",
        str(asset_manifest_path),
        "--pack",
        str(pack_path),
        "--report",
        str(pack_report_path),
        "--determinism-check",
    ]
    if packer is not None:
        return [str(packer), *packer_args]

    command = [
        cargo,
        "run",
        "-p",
        "zircon_runtime",
        "--bin",
        "zircon_export_pack",
        "--manifest-path",
        str(repo_root / "Cargo.toml"),
        "--target-dir",
        str(target_dir),
    ]
    if locked:
        command.append("--locked")
    if offline:
        command.append("--offline")
    command.extend(["--", *packer_args])
    return command


def run_plugin_asset_pack_command(
    command: list[str],
    repo_root: Path,
    diagnostics: list[str],
) -> subprocess.CompletedProcess[str] | None:
    try:
        completed = subprocess.run(
            command,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        diagnostics.append(f"plugin asset pack command could not start: {error}")
        return None
    if completed.returncode != 0:
        diagnostics.append(
            f"plugin asset pack command exited with code {completed.returncode}"
        )
        if completed.stderr:
            diagnostics.append(completed.stderr.strip())
    return completed


def plugin_asset_pack_report_is_clean(
    report_path: Path,
    diagnostics: list[str],
) -> bool:
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(f"plugin asset pack report {report_path} could not be read: {error}")
        return False
    except json.JSONDecodeError as error:
        diagnostics.append(f"plugin asset pack report {report_path} is invalid JSON: {error}")
        return False
    if not isinstance(report, dict):
        diagnostics.append(f"plugin asset pack report {report_path} must be an object")
        return False
    if report.get("fatal") is True:
        report_diagnostics = report.get("diagnostics")
        if isinstance(report_diagnostics, list):
            diagnostics.extend(str(diagnostic) for diagnostic in report_diagnostics)
        diagnostics.append(f"plugin asset pack report {report_path} is fatal")
        return False
    return True


def plugin_build_abi_contract(
    abi_version: int,
    distribution: dict[str, Any],
) -> dict[str, object]:
    abi = {"abi_version": abi_version, **NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS}
    descriptor_symbol = distribution.get("descriptor_symbol")
    if isinstance(descriptor_symbol, str) and descriptor_symbol.strip():
        abi["descriptor_symbol"] = descriptor_symbol
    return abi


def plugin_build_failure_report(
    args: argparse.Namespace,
    diagnostics: list[str],
) -> dict[str, object]:
    return {
        "command": "plugin build",
        "plugin_id": args.plugin_id,
        "form": args.form,
        "target_platform": args.target_platform,
        "mode": args.mode,
        "fatal": True,
        "diagnostics": diagnostics,
    }


def shell_join(command: Sequence[str]) -> str:
    return " ".join(shell_quote(part) for part in command)


def shell_quote(value: str) -> str:
    if sys.platform == "win32":
        return subprocess.list2cmdline([value])
    import shlex

    return shlex.quote(value)
