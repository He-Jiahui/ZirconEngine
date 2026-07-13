#!/usr/bin/env python3
"""Build and stage Zircon editor, runtime, and plugin artifacts."""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable, Sequence

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - exercised only on old Python.
    print("Python 3.11 or newer is required because this tool uses tomllib.", file=sys.stderr)
    raise

try:
    from .zircon_build_config import BuildConfig
    from .zircon_build_asset_staging import (
        copy_resource_dirs,
        stage_engine_assets,
    )
    from .zircon_build_hub import build_hub
    from .zircon_build_font_sdf import bake_font_sdf_manifest
    from .zircon_build_plugin_assets import collect_plugin_asset_roots
    from .zircon_build_plugin_manifest_contract import (
        collect_module_crate_names,
        distribution_table,
        normalize_optional_string,
        require_distribution_forms,
    )
    from .zircon_build_plugin_packages import PluginPackage
    from .zircon_build_plugin_selection import (
        filter_plugins_by_carrier,
        print_plugin_catalog,
        select_plugins,
    )
    from .zircon_build_plugin_shader_descriptors import (
        collect_geometry_source_descriptor_id_specs,
        collect_geometry_source_descriptors,
        collect_shader_module_specs,
        collect_shader_permutation_id_specs,
        collect_shading_model_descriptors,
        shading_model_descriptor_id_specs,
    )
    from .zircon_build_plugin_workspace_crates import discover_plugin_workspace_crates
    from .zircon_build_shader_prewarm import (
        build_shader_prewarm_command,
        parse_shader_geometry_source_ids,
        parse_shader_geometry_sources,
        parse_shader_quality_tiers,
        parse_shader_shading_model_ids,
        print_shader_prewarm_plan,
        print_shader_prewarm_report_dimensions,
        validate_shader_permutation_registry_export_contract,
        write_generated_shader_permutation_registry,
    )
    from .zircon_build_shader_prewarm_acceptance import (
        validate_staged_shader_prewarm_acceptance_contract,
    )
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_config import BuildConfig
    from zircon_build_asset_staging import (
        copy_resource_dirs,
        stage_engine_assets,
    )
    from zircon_build_hub import build_hub
    from zircon_build_font_sdf import bake_font_sdf_manifest
    from zircon_build_plugin_assets import collect_plugin_asset_roots
    from zircon_build_plugin_manifest_contract import (
        collect_module_crate_names,
        distribution_table,
        normalize_optional_string,
        require_distribution_forms,
    )
    from zircon_build_plugin_packages import PluginPackage
    from zircon_build_plugin_selection import (
        filter_plugins_by_carrier,
        print_plugin_catalog,
        select_plugins,
    )
    from zircon_build_plugin_shader_descriptors import (
        collect_geometry_source_descriptor_id_specs,
        collect_geometry_source_descriptors,
        collect_shader_module_specs,
        collect_shader_permutation_id_specs,
        collect_shading_model_descriptors,
        shading_model_descriptor_id_specs,
    )
    from zircon_build_plugin_workspace_crates import discover_plugin_workspace_crates
    from zircon_build_shader_prewarm import (
        build_shader_prewarm_command,
        parse_shader_geometry_source_ids,
        parse_shader_geometry_sources,
        parse_shader_quality_tiers,
        parse_shader_shading_model_ids,
        print_shader_prewarm_plan,
        print_shader_prewarm_report_dimensions,
        validate_shader_permutation_registry_export_contract,
        write_generated_shader_permutation_registry,
    )
    from zircon_build_shader_prewarm_acceptance import (
        validate_staged_shader_prewarm_acceptance_contract,
    )


TARGETS = ("hub", "editor", "runtime", "plugins", "font-sdf")
MODES = ("debug", "release", "profiling")
PLUGIN_CARRIERS = ("all", "native_dynamic", "rlib_static")
PLUGIN_LOAD_MANIFEST = "plugins/native_plugins.toml"


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    repo_root = resolve_repo_root()
    plugin_catalog = discover_plugins(repo_root)

    if args.list_plugins:
        print_plugin_catalog(plugin_catalog)
        return 0

    config = resolve_config(args, repo_root, plugin_catalog)
    print_plan(config)
    build(config)
    return 0


def parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build staged ZirconEngine hub/editor/runtime/plugin artifacts.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python tools/zircon_build.py --targets hub,editor,runtime --out E:\\zircon-build --mode debug
  python tools/zircon_build.py --targets editor,runtime --out E:\\zircon-build --mode debug
  python tools/zircon_build.py --targets runtime --out E:\\zircon-build-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy
  python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\\zircon-build --mode debug
  python tools/zircon_build.py --targets plugins --plugins all --plugin-carrier native_dynamic --out E:\\zircon-build --mode release
  python tools/zircon_build.py --targets font-sdf --font-sdf-manifest E:\\project\\font-sdf.json --out E:\\zircon-build --mode release

Plugin carrier boundary:
  native_dynamic crates are cdylib plugins copied into ZirconEngine/plugins.
  rlib_static crates are built into targets/plugins/<id> and remain static-link inputs.
""".strip(),
    )
    parser.add_argument(
        "--targets",
        "--target",
        help="Comma-separated build targets: hub,editor,runtime,plugins,font-sdf.",
    )
    parser.add_argument("--out", "--output", help="Build output directory.")
    parser.add_argument(
        "--font-sdf-manifest",
        help="Versioned JSON bake manifest required by the font-sdf target.",
    )
    parser.add_argument("--mode", choices=MODES, help="Cargo profile mode.")
    parser.add_argument(
        "--runtime-features",
        help=(
            "Comma-separated runtime/app feature set for runtime and editor targets. "
            "Defaults to target-client for runtime builds and target-editor-host for editor-only staging."
        ),
    )
    parser.add_argument(
        "--cargo",
        default="cargo",
        help="Cargo executable to invoke. Default: cargo.",
    )
    parser.add_argument(
        "--plugins",
        help="Plugin ids, numbers, ranges, all, native, or rlib when plugins target is selected.",
    )
    parser.add_argument(
        "--plugin-carrier",
        choices=PLUGIN_CARRIERS,
        default="all",
        help="Filter selected plugins by deployability carrier. Default: all.",
    )
    parser.add_argument(
        "--jobs",
        default="1",
        help="Forwarded Cargo jobs value. Default: 1. Use empty string to omit.",
    )
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Do not pass --locked to Cargo. Locked builds are the default.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print Cargo/copy actions without executing them.",
    )
    parser.add_argument(
        "--prewarm-shaders",
        action="store_true",
        help="Prewarm built-in shader variants into ZirconEngine/cache/shader_variants.",
    )
    parser.add_argument(
        "--validate-wgpu-shaders",
        action="store_true",
        help=(
            "When --prewarm-shaders is enabled, validate each prewarm WGSL source by "
            "creating an offscreen WGPU shader module before writing the cache."
        ),
    )
    parser.add_argument(
        "--validate-wgpu-pipelines",
        action="store_true",
        help=(
            "When --prewarm-shaders is enabled, validate each full-template mesh "
            "prewarm request by creating an offscreen WGPU render pipeline before "
            "writing the cache."
        ),
    )
    parser.add_argument(
        "--shader-quality-tier",
        action="append",
        choices=("low", "medium", "high", "ultra", "all"),
        default=[],
        help=(
            "Shader quality tier(s) to prewarm when --prewarm-shaders is enabled. "
            "Repeat for multiple tiers or use all. Default: medium."
        ),
    )
    parser.add_argument(
        "--shader-geometry-source",
        action="append",
        choices=("static", "skinned", "morphed", "skinned-morphed", "all"),
        default=[],
        help=(
            "Geometry source(s) to prewarm when --prewarm-shaders is enabled. "
            "Repeat for multiple sources or use all. Default: static."
        ),
    )
    parser.add_argument(
        "--shader-asset-root",
        action="append",
        default=[],
        help=(
            "Project shader asset root to scan during --prewarm-shaders and automatic "
            "shader resource registry export. Repeat for multiple project roots."
        ),
    )
    parser.add_argument(
        "--shader-geometry-source-id",
        action="append",
        default=[],
        metavar="CUSTOM=ID",
        help=(
            "Custom geometry source plugin id(s) to prewarm when --prewarm-shaders is enabled. "
            "Use custom:name=4 or name=4, repeat for multiple plugin geometry sources."
        ),
    )
    parser.add_argument(
        "--shader-shading-model-id",
        action="append",
        default=[],
        metavar="CUSTOM=ID",
        help=(
            "Custom shading model plugin id(s) to prewarm when --prewarm-shaders is enabled. "
            "Use custom:name=16 or name=16, repeat for multiple plugin models."
        ),
    )
    parser.add_argument(
        "--shader-permutation-registry",
        action="append",
        default=[],
        help=(
            "Project/plugin shader permutation registry JSON file to merge during "
            "--prewarm-shaders. Repeat for multiple registries. Asset roots also "
            "auto-discover shader_permutation_registry.json."
        ),
    )
    parser.add_argument(
        "--shader-resource-registry",
        help=(
            "ResourceRecord JSON array or {resources:[...]} file whose shader revisions "
            "override asset-root source-hash revisions during --prewarm-shaders. "
            "When omitted, --prewarm-shaders exports a staged shader registry automatically."
        ),
    )
    parser.add_argument(
        "--list-plugins",
        action="store_true",
        help="List discovered plugins and exit.",
    )
    return parser.parse_args(argv)


def resolve_repo_root() -> Path:
    root = Path(__file__).resolve().parents[1]
    if not (root / "Cargo.toml").exists():
        raise SystemExit(f"Cannot locate repository Cargo.toml from {__file__}.")
    return root


def resolve_config(
    args: argparse.Namespace, repo_root: Path, plugin_catalog: Sequence[PluginPackage]
) -> BuildConfig:
    targets = parse_targets(args.targets) if args.targets else prompt_targets()
    out_root = resolve_out_root(args.out) if args.out else prompt_out_root()
    mode = args.mode or prompt_mode()
    runtime_features = (
        parse_feature_list(args.runtime_features)
        if args.runtime_features
        else default_runtime_features(targets)
    )
    plugin_carrier = args.plugin_carrier
    font_sdf_manifest = resolve_optional_path(args.font_sdf_manifest)

    if mode == "profiling" and "hub" in targets:
        raise SystemExit("--mode profiling is not supported for the hub/Tauri target.")
    if mode == "profiling" and "plugins" in targets:
        raise SystemExit("--mode profiling is not supported for the plugin workspace target.")

    selected_plugins: tuple[PluginPackage, ...] = ()
    candidates = filter_plugins_by_carrier(plugin_catalog, plugin_carrier)
    if args.plugins:
        selected_plugins = tuple(select_plugins(candidates, args.plugins))
    elif "plugins" in targets:
        selected_plugins = tuple(prompt_plugins(candidates))
    if "plugins" in targets and not selected_plugins:
        raise SystemExit("No plugins selected for the plugins target.")
    if "font-sdf" in targets and font_sdf_manifest is None:
        raise SystemExit("The font-sdf target requires --font-sdf-manifest.")
    if "font-sdf" not in targets and font_sdf_manifest is not None:
        raise SystemExit("--font-sdf-manifest requires the font-sdf target.")

    return BuildConfig(
        repo_root=repo_root,
        out_root=out_root,
        cargo=args.cargo,
        mode=mode,
        targets=targets,
        runtime_features=runtime_features,
        plugins=selected_plugins,
        plugin_carrier=plugin_carrier,
        locked=not args.no_locked,
        jobs=args.jobs or None,
        dry_run=args.dry_run,
        prewarm_shaders=args.prewarm_shaders,
        validate_wgpu_shaders=args.validate_wgpu_shaders,
        validate_wgpu_pipelines=args.validate_wgpu_pipelines,
        shader_quality_tiers=parse_shader_quality_tiers(args.shader_quality_tier),
        shader_geometry_sources=parse_shader_geometry_sources(args.shader_geometry_source),
        shader_asset_roots=resolve_optional_paths(args.shader_asset_root),
        shader_geometry_source_ids=parse_shader_geometry_source_ids(
            args.shader_geometry_source_id
        ),
        shader_shading_model_ids=parse_shader_shading_model_ids(
            args.shader_shading_model_id
        ),
        shader_permutation_registries=resolve_optional_paths(
            args.shader_permutation_registry
        ),
        shader_resource_registry=resolve_optional_path(args.shader_resource_registry),
        font_sdf_manifest=font_sdf_manifest,
    )


def parse_targets(raw: str) -> tuple[str, ...]:
    values = parse_csv(raw)
    if not values:
        raise SystemExit("--targets must name at least one target.")
    if "all" in values:
        values = list(TARGETS)
    unknown = sorted(set(values) - set(TARGETS))
    if unknown:
        raise SystemExit(f"Unknown target(s): {', '.join(unknown)}")
    return tuple(unique_in_order(values))


def parse_csv(raw: str) -> list[str]:
    return [part.strip().lower() for part in raw.split(",") if part.strip()]


def parse_feature_list(raw: str) -> tuple[str, ...]:
    values = parse_csv(raw)
    if not values:
        raise SystemExit("--runtime-features must name at least one feature.")
    return tuple(unique_in_order(values))


def default_runtime_features(targets: Sequence[str]) -> tuple[str, ...]:
    if "runtime" in targets:
        return ("target-client",)
    if "editor" in targets:
        return ("target-editor-host",)
    return ("target-client",)


def resolve_out_root(raw: str) -> Path:
    path = Path(raw).expanduser()
    if not path.is_absolute():
        path = (Path.cwd() / path).resolve()
    return path


def resolve_optional_path(raw: str | None) -> Path | None:
    if not raw:
        return None
    path = Path(raw).expanduser()
    if not path.is_absolute():
        path = (Path.cwd() / path).resolve()
    return path


def resolve_optional_paths(raw_paths: Sequence[str]) -> tuple[Path, ...]:
    paths: list[Path] = []
    for raw in raw_paths:
        path = resolve_optional_path(raw)
        if path is not None:
            paths.append(path)
    return tuple(paths)


def prompt_targets() -> tuple[str, ...]:
    require_tty("--targets")
    print("Select build targets:")
    for index, target in enumerate(TARGETS, start=1):
        print(f"  {index}) {target}")
    raw = input("Targets (comma numbers or names, default hub,editor,runtime): ").strip()
    if not raw:
        return ("hub", "editor", "runtime")
    return parse_targets(resolve_number_tokens(raw, TARGETS))


def prompt_out_root() -> Path:
    require_tty("--out")
    raw = input("Build output directory: ").strip()
    if not raw:
        raise SystemExit("Build output directory is required.")
    return resolve_out_root(raw)


def prompt_mode() -> str:
    require_tty("--mode")
    raw = input("Build mode [debug/release/profiling] (default debug): ").strip().lower()
    if not raw:
        return "debug"
    if raw not in MODES:
        raise SystemExit(f"Unknown mode: {raw}")
    return raw


def prompt_plugins(candidates: Sequence[PluginPackage]) -> list[PluginPackage]:
    require_tty("--plugins")
    if not candidates:
        raise SystemExit("No plugins match the current carrier filter.")
    print_plugin_catalog(candidates)
    raw = input("Plugins (numbers, ids, ranges, all/native/rlib; default native): ").strip()
    if not raw:
        raw = "native"
    return select_plugins(candidates, raw)


def require_tty(option_name: str) -> None:
    if not sys.stdin.isatty():
        raise SystemExit(f"Missing {option_name}; interactive prompt is unavailable.")


def discover_plugins(repo_root: Path) -> tuple[PluginPackage, ...]:
    plugins_root = repo_root / "zircon_plugins"
    crates = discover_plugin_workspace_crates(plugins_root)
    crates_by_name = {crate.name: crate for crate in crates}
    packages: list[PluginPackage] = []
    for manifest_path in sorted(plugins_root.rglob("plugin.toml")):
        data = read_toml(manifest_path)
        plugin_id = str(data.get("id", manifest_path.parent.name))
        display_name = str(data.get("display_name", plugin_id))
        distribution = distribution_table(data)
        default_packaging = tuple(
            normalize_packaging(
                distribution.get("default_packaging", data.get("default_packaging", []))
            )
        )
        distribution_forms = require_distribution_forms(manifest_path, distribution)
        dist_crate_name = normalize_optional_string(distribution.get("dist_crate"))
        module_crate_names = tuple(unique_in_order(collect_module_crate_names(data)))
        asset_roots = collect_plugin_asset_roots(
            manifest_path,
            data,
            distribution,
            plugin_id,
        )
        shader_geometry_source_descriptors = collect_geometry_source_descriptors(
            manifest_path, data
        )
        shader_shading_model_descriptors = collect_shading_model_descriptors(
            manifest_path, data
        )
        shader_geometry_source_ids = tuple(
            unique_in_order(
                [
                    *collect_shader_permutation_id_specs(
                        manifest_path, data, "geometry_source_ids"
                    ),
                    *collect_geometry_source_descriptor_id_specs(
                        shader_geometry_source_descriptors
                    ),
                ]
            )
        )
        shader_shading_model_ids = tuple(
            unique_in_order(
                [
                    *collect_shader_permutation_id_specs(
                        manifest_path, data, "shading_model_ids"
                    ),
                    *shading_model_descriptor_id_specs(
                        shader_shading_model_descriptors
                    ),
                ]
            )
        )
        shader_modules = collect_shader_module_specs(manifest_path, data)
        matched_crates = tuple(
            crates_by_name[name] for name in module_crate_names if name in crates_by_name
        )
        packages.append(
            PluginPackage(
                plugin_id=plugin_id,
                display_name=display_name,
                manifest_path=manifest_path,
                package_root=manifest_path.parent,
                asset_roots=asset_roots,
                default_packaging=default_packaging,
                distribution_forms=distribution_forms,
                dist_crate_name=dist_crate_name,
                module_crate_names=module_crate_names,
                shader_geometry_source_ids=shader_geometry_source_ids,
                shader_geometry_source_descriptors=shader_geometry_source_descriptors,
                shader_shading_model_ids=shader_shading_model_ids,
                shader_shading_model_descriptors=shader_shading_model_descriptors,
                crates=matched_crates,
                shader_modules=shader_modules,
            )
        )
    return tuple(sorted(packages, key=lambda item: item.plugin_id))


def read_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def normalize_packaging(values: object) -> list[str]:
    if not isinstance(values, list):
        return []
    return [str(value).strip().lower() for value in values if str(value).strip()]


def print_plan(config: BuildConfig) -> None:
    print("Zircon build plan")
    print(f"  repo:    {config.repo_root}")
    print(f"  out:     {config.out_root}")
    print(f"  cargo:   {config.cargo}")
    print(f"  mode:    {config.mode}")
    print(f"  targets: {','.join(config.targets)}")
    if "runtime" in config.targets:
        print(f"  runtime features: {config.runtime_feature_arg}")
    if "editor" in config.targets:
        print(
            "  editor runtime features: "
            f"{config.feature_arg_for_target('target-editor-host')}"
        )
    print(f"  locked:  {config.locked}")
    if config.jobs:
        print(f"  jobs:    {config.jobs}")
    if config.dry_run:
        print("  dry-run: enabled")
    if config.prewarm_shaders:
        print_shader_prewarm_plan(config)
    if config.font_sdf_manifest is not None:
        print(f"  font-SDF manifest: {config.font_sdf_manifest}")
    if config.plugins:
        print("  plugins:")
        for package in config.plugins:
            print(f"    - {package.plugin_id} ({','.join(package.carriers) or 'manifest_only'})")


def build(config: BuildConfig) -> None:
    if not config.dry_run:
        config.engine_root.mkdir(parents=True, exist_ok=True)
        config.targets_root.mkdir(parents=True, exist_ok=True)

    if "hub" in config.targets:
        build_hub(config)

    runtime_staged = False
    if "runtime" in config.targets:
        build_runtime(config, config.runtime_feature_arg, include_preview=True)
        runtime_staged = True
    if "editor" in config.targets:
        editor_features = config.feature_arg_for_target("target-editor-host")
        if not runtime_staged:
            build_runtime(config, editor_features, include_preview=False)
            runtime_staged = True
        build_editor(config, editor_features)
    if "editor" in config.targets or "runtime" in config.targets:
        stage_engine_assets(config)
        if config.prewarm_shaders:
            prewarm_shaders(config)
    if "plugins" in config.targets:
        ensure_plugin_base_artifacts(config)
        build_plugins(config)
    if "font-sdf" in config.targets:
        bake_font_sdf_manifest(config, config.font_sdf_manifest)


def build_runtime(config: BuildConfig, runtime_feature_arg: str, include_preview: bool) -> None:
    runtime_root = config.targets_root / "runtime"
    lib_target_dir = runtime_root / "lib"
    bin_target_dir = runtime_root / "bin"
    preview_feature_arg = config.runtime_preview_feature_arg
    run_cargo(
        config,
        [
            "build",
            "-p",
            "zircon_runtime",
            "--lib",
            "--no-default-features",
            "--features",
            runtime_feature_arg,
            "--target-dir",
            str(lib_target_dir),
        ],
    )
    if include_preview:
        run_cargo(
            config,
            [
                "build",
                "-p",
                "zircon_app",
                "--bin",
                "zircon_runtime",
                "--no-default-features",
                "--features",
                preview_feature_arg,
                "--target-dir",
                str(bin_target_dir),
            ],
        )
    if config.dry_run:
        return
    copy_artifact(config, lib_target_dir, platform_runtime_library_name())
    if include_preview:
        copy_artifact(config, bin_target_dir, platform_executable_name("zircon_runtime"))


def prewarm_shaders(config: BuildConfig) -> None:
    if not config.dry_run:
        permutation_registry_path = write_generated_shader_permutation_registry(config)
        if permutation_registry_path is not None:
            validate_shader_permutation_registry_export_contract(
                permutation_registry_path,
                config=config,
            )
    command = build_shader_prewarm_command(config)
    if config.dry_run:
        print("DRY-RUN", quote_command(command))
        return
    print(quote_command(command))
    result = subprocess.run(command, cwd=config.repo_root, check=False)
    print_shader_prewarm_report_dimensions(config.shader_prewarm_report_path)
    if result.returncode == 0:
        validate_staged_shader_prewarm_acceptance_contract(config)
    result.check_returncode()


def build_editor(config: BuildConfig, editor_feature_arg: str) -> None:
    target_dir = config.targets_root / "editor"
    run_cargo(
        config,
        [
            "build",
            "-p",
            "zircon_app",
            "--bin",
            "zircon_editor",
            "--no-default-features",
            "--features",
            editor_feature_arg,
            "--target-dir",
            str(target_dir),
        ],
    )
    if config.dry_run:
        return
    copy_artifact(config, target_dir, platform_executable_name("zircon_editor"))


def ensure_plugin_base_artifacts(config: BuildConfig) -> None:
    if config.dry_run:
        return
    required = []
    if "editor" not in config.targets:
        required.append(config.engine_root / platform_executable_name("zircon_editor"))
    if "runtime" not in config.targets and "editor" not in config.targets:
        required.append(config.engine_root / platform_runtime_library_name())
    missing = [path for path in required if not path.exists()]
    if missing:
        missing_list = ", ".join(str(path) for path in missing)
        raise SystemExit(
            "Plugin builds require existing editor/runtime artifacts unless "
            "those targets are built in the same invocation; checked "
            f"{config.engine_root}; missing: {missing_list}"
        )


def build_plugins(config: BuildConfig) -> None:
    native_packages: list[PluginPackage] = []
    for package in config.plugins:
        if package.native_dynamic_crates:
            build_native_dynamic_plugin(config, package)
            native_packages.append(package)
        if package.rlib_static_crates:
            build_rlib_static_plugin(config, package)
    if native_packages:
        write_native_plugin_load_manifest(config, native_packages)


def build_native_dynamic_plugin(config: BuildConfig, package: PluginPackage) -> None:
    target_dir = plugin_target_dir(config, package)
    crate_names = [crate.name for crate in package.native_dynamic_crates]
    print(f"Building native_dynamic plugin {package.plugin_id}: {', '.join(crate_names)}")
    run_plugin_cargo(config, target_dir, crate_names)
    if config.dry_run:
        return
    package_out = config.engine_root / "plugins" / sanitize_path_component(package.plugin_id)
    native_out = package_out / "native"
    native_out.mkdir(parents=True, exist_ok=True)
    copy_file(package.manifest_path, package_out / "plugin.toml", config)
    copy_resource_dirs(package.package_root, package_out, config)
    for crate in package.native_dynamic_crates:
        artifact_name = platform_dynamic_library_name(crate.name)
        artifact = find_artifact(target_dir, config.profile_dir, artifact_name)
        copy_file(artifact, native_out / artifact.name, config)
        copy_sidecars(artifact, native_out, config)


def build_rlib_static_plugin(config: BuildConfig, package: PluginPackage) -> None:
    target_dir = plugin_target_dir(config, package)
    crate_names = [crate.name for crate in package.rlib_static_crates]
    print(
        "Building rlib_static plugin "
        f"{package.plugin_id}: {', '.join(crate_names)}"
    )
    print(
        "  Note: rlib_static crates are valid static-link inputs only; "
        "they are not copied into ZirconEngine/plugins."
    )
    run_plugin_cargo(config, target_dir, crate_names)


def run_plugin_cargo(config: BuildConfig, target_dir: Path, package_names: Sequence[str]) -> None:
    if not package_names:
        return
    args = [
        "build",
        "--manifest-path",
        str(config.repo_root / "zircon_plugins" / "Cargo.toml"),
        "--target-dir",
        str(target_dir),
    ]
    for package_name in package_names:
        args.extend(["-p", package_name])
    run_cargo(config, args)


def run_cargo(config: BuildConfig, args: list[str]) -> None:
    command = [config.cargo, *args]
    if config.locked:
        command.append("--locked")
    if config.mode == "release":
        command.append("--release")
    elif config.mode == "profiling":
        command.extend(["--profile", "profiling"])
    if config.jobs:
        command.extend(["--jobs", config.jobs])
    if config.dry_run:
        print("DRY-RUN", quote_command(command))
        return
    print(quote_command(command))
    subprocess.run(command, cwd=config.repo_root, check=True)


def copy_artifact(config: BuildConfig, target_dir: Path, artifact_name: str) -> None:
    artifact = find_artifact(target_dir, config.profile_dir, artifact_name)
    copy_file(artifact, config.engine_root / artifact.name, config)
    copy_sidecars(artifact, config.engine_root, config)


def find_artifact(target_dir: Path, profile_dir: str, artifact_name: str) -> Path:
    profile_root = target_dir / profile_dir
    candidates = [profile_root / artifact_name, profile_root / "deps" / artifact_name]
    candidates.extend(profile_root.rglob(artifact_name) if profile_root.exists() else [])
    for candidate in candidates:
        if candidate.exists() and candidate.is_file():
            return candidate
    raise SystemExit(f"Built artifact not found under {profile_root}: {artifact_name}")


def copy_file(source: Path, destination: Path, config: BuildConfig) -> None:
    if config.dry_run:
        print(f"DRY-RUN copy {source} -> {destination}")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    print(f"Copied {source} -> {destination}")


def copy_sidecars(source: Path, destination_dir: Path, config: BuildConfig) -> None:
    sidecars = [source.with_suffix(".pdb"), source.with_suffix(".dbg"), Path(str(source) + ".dSYM")]
    for sidecar in sidecars:
        if not sidecar.exists():
            continue
        destination = destination_dir / sidecar.name
        if sidecar.is_dir():
            if config.dry_run:
                print(f"DRY-RUN copytree {sidecar} -> {destination}")
            else:
                if destination.exists():
                    shutil.rmtree(destination)
                shutil.copytree(sidecar, destination)
                print(f"Copied {sidecar} -> {destination}")
        else:
            copy_file(sidecar, destination, config)


def write_native_plugin_load_manifest(
    config: BuildConfig, native_packages: Sequence[PluginPackage]
) -> None:
    manifest_path = config.engine_root / PLUGIN_LOAD_MANIFEST
    lines = ["# Generated by tools/zircon_build.py.\n"]
    seen_dirs: set[str] = set()
    for package in native_packages:
        package_dir = sanitize_path_component(package.plugin_id)
        if package_dir in seen_dirs:
            raise SystemExit(
                f"Native plugin output directory collision: plugins/{package_dir}"
            )
        seen_dirs.add(package_dir)
        lines.extend(
            [
                "\n[[plugins]]\n",
                f"id = {toml_string(package.plugin_id)}\n",
                f"path = {toml_string('plugins/' + package_dir)}\n",
                f"manifest = {toml_string('plugins/' + package_dir + '/plugin.toml')}\n",
            ]
        )
    if config.dry_run:
        print(f"DRY-RUN write {manifest_path}")
        return
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text("".join(lines), encoding="utf-8")
    print(f"Wrote {manifest_path}")


def plugin_target_dir(config: BuildConfig, package: PluginPackage) -> Path:
    return config.targets_root / "plugins" / sanitize_path_component(package.plugin_id)


def platform_executable_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def platform_runtime_library_name() -> str:
    if os.name == "nt":
        return "zircon_runtime.dll"
    if platform.system().lower() == "darwin":
        return "libzircon_runtime.dylib"
    return "libzircon_runtime.so"


def platform_dynamic_library_name(crate_name: str) -> str:
    if os.name == "nt":
        return f"{crate_name}.dll"
    if platform.system().lower() == "darwin":
        return f"lib{crate_name}.dylib"
    return f"lib{crate_name}.so"


def sanitize_path_component(value: str) -> str:
    sanitized = "".join(ch if ch.isascii() and (ch.isalnum() or ch in "-_") else "_" for ch in value)
    return sanitized or "_"


def toml_string(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def resolve_number_tokens(raw: str, labels: Sequence[str]) -> str:
    resolved: list[str] = []
    for token in parse_csv(raw):
        if token.isdigit():
            index = int(token)
            if index < 1 or index > len(labels):
                raise SystemExit(f"Selection index out of range: {index}")
            resolved.append(labels[index - 1])
        else:
            resolved.append(token)
    return ",".join(resolved)


def unique_in_order(values: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def quote_command(command: Sequence[str]) -> str:
    return " ".join(quote_arg(part) for part in command)


def quote_arg(value: str) -> str:
    if not value or any(ch.isspace() for ch in value):
        return '"' + value.replace('"', '\\"') + '"'
    return value


if __name__ == "__main__":
    raise SystemExit(main())
