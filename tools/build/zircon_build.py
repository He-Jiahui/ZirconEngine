#!/usr/bin/env python3
"""Build and stage Zircon editor, runtime, and plugin artifacts."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Sequence

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - exercised only on old Python.
    print("Python 3.11 or newer is required because this tool uses tomllib.", file=sys.stderr)
    raise

try:
    from .zircon_build_config import BuildConfig
    from .zircon_build_arguments import (
        MODES,
        PLUGIN_CARRIERS,
        TARGETS,
        parse_args,
    )
    from .zircon_build_cargo_environment import (
        assert_managed_windows_build_root,
        managed_cargo_environment,
    )
    from .zircon_build_asset_staging import (
        copy_resource_dirs,
        stage_engine_assets,
    )
    from .zircon_build_artifact_io import copy_file, copy_sidecars
    from .zircon_build_staging_manifest import write_staging_manifest
    from .zircon_build_runtime_manifest import write_runtime_artifact_manifest
    from .zircon_build_product_profile import (
        ProductBuildProfile,
        ProductBuildProfileError,
        load_product_build_profile,
    )
    from .zircon_build_product_output import (
        clean_product_output,
        finalize_product_output,
    )
    from .zircon_build_abi import AbiValidationError, validate_runtime_abi
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
    from .zircon_build_support import (
        parse_csv,
        platform_dynamic_library_name,
        platform_executable_name,
        platform_runtime_library_name,
        quote_command,
        resolve_number_tokens,
        sanitize_path_component,
        toml_string,
        unique_in_order,
    )
except ImportError:  # pragma: no cover - exercised when run as a script.
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
    from tools.build.zircon_build_config import BuildConfig
    from tools.build.zircon_build_arguments import (
        MODES,
        PLUGIN_CARRIERS,
        TARGETS,
        parse_args,
    )
    from tools.build.zircon_build_cargo_environment import (
        assert_managed_windows_build_root,
        managed_cargo_environment,
    )
    from tools.build.zircon_build_asset_staging import (
        copy_resource_dirs,
        stage_engine_assets,
    )
    from tools.build.zircon_build_artifact_io import copy_file, copy_sidecars
    from tools.build.zircon_build_staging_manifest import write_staging_manifest
    from tools.build.zircon_build_runtime_manifest import write_runtime_artifact_manifest
    from tools.build.zircon_build_product_profile import (
        ProductBuildProfile,
        ProductBuildProfileError,
        load_product_build_profile,
    )
    from tools.build.zircon_build_product_output import clean_product_output, finalize_product_output
    from tools.build.zircon_build_abi import AbiValidationError, validate_runtime_abi
    from tools.build.zircon_build_hub import build_hub
    from tools.build.zircon_build_font_sdf import bake_font_sdf_manifest
    from tools.build.zircon_build_plugin_assets import collect_plugin_asset_roots
    from tools.build.zircon_build_plugin_manifest_contract import (
        collect_module_crate_names,
        distribution_table,
        normalize_optional_string,
        require_distribution_forms,
    )
    from tools.build.zircon_build_plugin_packages import PluginPackage
    from tools.build.zircon_build_plugin_selection import (
        filter_plugins_by_carrier,
        print_plugin_catalog,
        select_plugins,
    )
    from tools.build.zircon_build_plugin_shader_descriptors import (
        collect_geometry_source_descriptor_id_specs,
        collect_geometry_source_descriptors,
        collect_shader_module_specs,
        collect_shader_permutation_id_specs,
        collect_shading_model_descriptors,
        shading_model_descriptor_id_specs,
    )
    from tools.build.zircon_build_plugin_workspace_crates import discover_plugin_workspace_crates
    from tools.build.zircon_build_shader_prewarm import (
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
    from tools.build.zircon_build_shader_prewarm_acceptance import (
        validate_staged_shader_prewarm_acceptance_contract,
    )
    from tools.build.zircon_build_support import (
        parse_csv,
        platform_dynamic_library_name,
        platform_executable_name,
        platform_runtime_library_name,
        quote_command,
        resolve_number_tokens,
        sanitize_path_component,
        toml_string,
        unique_in_order,
    )


PLUGIN_LOAD_MANIFEST = "plugins/native_plugins.toml"


def main(argv: Sequence[str] | None = None) -> int:
    # The product builder is a known M8 entry point.  Dispatch before parsing
    # interactive build options so sole-entry mode cannot execute Cargo or
    # publish products without the caller's immutable Jenkins identity.
    try:
        from tools.jenkins.frontend import entry_from_environment
    except ModuleNotFoundError:
        sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
        from tools.jenkins.frontend import entry_from_environment
    forwarded = entry_from_environment(resolve_repo_root(), job="zircon-flow",
                                       parameters={"entry": "zircon_build"})
    if forwarded is not None:
        print(json.dumps(forwarded, sort_keys=True))
        return 0
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


def resolve_repo_root() -> Path:
    root = Path(__file__).resolve().parents[2]
    if not (root / "Cargo.toml").exists():
        raise SystemExit(f"Cannot locate repository Cargo.toml from {__file__}.")
    return root


def resolve_config(
    args: argparse.Namespace, repo_root: Path, plugin_catalog: Sequence[PluginPackage]
) -> BuildConfig:
    product_profile: ProductBuildProfile | None = None
    if getattr(args, "product_profile_file", None) and not args.product_profile:
        raise SystemExit("--product-profile-file requires --product-profile")
    if args.product_profile:
        try:
            product_profile = load_product_build_profile(
                args.product_profile,
                path=args.product_profile_file,
            )
        except ProductBuildProfileError as error:
            raise SystemExit(str(error)) from error
        expected_target = ("runtime",) if product_profile.is_runtime else ("editor",)
        if args.targets:
            requested_targets = parse_targets(args.targets)
            if requested_targets != expected_target:
                raise SystemExit(
                    f"product profile {product_profile.name} owns target "
                    f"{','.join(expected_target)}; do not combine it with "
                    f"{','.join(requested_targets)}"
                )
            targets = requested_targets
        else:
            targets = expected_target
    else:
        targets = parse_targets(args.targets) if args.targets else prompt_targets()
    out_root = resolve_out_root(args.out) if args.out else prompt_out_root()
    if product_profile is not None:
        allowed_product_modes = {product_profile.cargo_profile}
        if product_profile.cargo_profile == "shipping":
            allowed_product_modes.add("shipping-symbols")
        if args.mode and args.mode not in allowed_product_modes:
            raise SystemExit(
                f"--mode {args.mode} does not match product profile "
                f"{product_profile.name} cargo_profile {product_profile.cargo_profile}"
            )
        mode = args.mode or product_profile.cargo_profile
        runtime_features = product_profile.runtime_features
        if args.runtime_features:
            requested_features = parse_feature_list(args.runtime_features)
            if requested_features != runtime_features:
                raise SystemExit(
                    f"product profile {product_profile.name} owns the runtime feature set "
                    f"({','.join(runtime_features)}); ad-hoc --runtime-features are not allowed"
                )
        if args.plugin_carrier != "all" and args.plugin_carrier != product_profile.plugin_carrier:
            raise SystemExit(
                f"--plugin-carrier {args.plugin_carrier} conflicts with product profile "
                f"{product_profile.name} ({product_profile.plugin_carrier})"
            )
        plugin_carrier = product_profile.plugin_carrier
        target_triple = args.target_triple or product_profile.target_triple
        if target_triple != product_profile.target_triple:
            raise SystemExit(
                f"product profile {product_profile.name} requires target "
                f"{product_profile.target_triple}"
            )
    else:
        mode = args.mode or prompt_mode()
        runtime_features = (
            parse_feature_list(args.runtime_features)
            if args.runtime_features
            else default_runtime_features(targets)
        )
        plugin_carrier = args.plugin_carrier
        target_triple = args.target_triple
    font_sdf_manifest = resolve_optional_path(args.font_sdf_manifest)

    if mode == "profiling" and "hub" in targets:
        raise SystemExit("--mode profiling is not supported for the hub/Tauri target.")
    if mode == "profiling" and "plugins" in targets:
        raise SystemExit("--mode profiling is not supported for the plugin workspace target.")
    if product_profile is not None and any(
        target in targets for target in ("hub", "plugins", "font-sdf")
    ):
        raise SystemExit(
            f"product profile {product_profile.name} can only build its own product target"
        )

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

    if product_profile is not None and args.plugins:
        raise SystemExit(
            f"product profile {product_profile.name} does not accept ad-hoc plugins; "
            "select an explicit optional plugin profile instead"
        )

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
        product_profile=product_profile,
        target_triple=target_triple,
        clean_output=bool(args.clean_output or product_profile is not None),
        asset_scope=(product_profile.asset_scope if product_profile is not None else "all"),
        project_pack_root=resolve_optional_path(getattr(args, "project_pack_root", None)),
        editor_asset_root=(
            out_root / "EditorAssets" if product_profile is not None and product_profile.is_editor else None
        ),
        cargo_profile_override=(mode if mode == "shipping-symbols" else None),
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
    if config.product_profile_name:
        print(f"  product: {config.product_profile_name}")
        print(f"  target:  {config.effective_target_triple}")
        print(f"  build set: {config.build_set_id}")
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
    if config.is_shipping:
        print(f"  symbols: {config.symbols_root}")
        print(f"  size report: {config.size_report_path}")
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
        assert_managed_windows_build_root(config.out_root)
        assert_managed_windows_build_root(config.engine_root)
        assert_managed_windows_build_root(config.targets_root)
    if getattr(config, "clean_output", False):
        clean_product_output(config)
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
        stage_engine_assets(config, getattr(config, "asset_scope", None))
        if config.prewarm_shaders:
            prewarm_shaders(config)
    if "plugins" in config.targets:
        ensure_plugin_base_artifacts(config)
        build_plugins(config)
    if "font-sdf" in config.targets:
        bake_font_sdf_manifest(config, config.font_sdf_manifest)
    if "editor" in config.targets or "runtime" in config.targets:
        write_runtime_artifact_manifest(config)
        write_staging_manifest(config)
        finalize_product_output(config)


def _runtime_product_features_include_abi(runtime_feature_arg: str) -> bool:
    tokens = runtime_feature_arg.replace(",", " ").split()
    return any(
        token
        in {
            "target-client",
            "target-editor-host",
            "dynamic-api",
            "zircon_runtime/dynamic-api",
            "shipping-runtime",
            "runtime-shipping",
            "shipping-editor",
            "editor-shipping",
            "zircon_runtime/shipping-runtime",
            "zircon_runtime/shipping-editor",
        }
        for token in tokens
    )


def build_runtime(config: BuildConfig, runtime_feature_arg: str, include_preview: bool) -> None:
    shipping_product = bool(
        getattr(config, "is_shipping", False)
        or getattr(config, "mode", "") in {"shipping", "shipping-symbols"}
        or getattr(config, "product_profile", None) is not None
    )
    if shipping_product:
        if "dev-dynamic-linking" in runtime_feature_arg:
            raise ValueError("Runtime C ABI products cannot enable development DLL features.")
        if not _runtime_product_features_include_abi(runtime_feature_arg):
            raise ValueError(
                "Runtime C ABI products require the runtime dynamic-api feature "
                "(use target-client, target-editor-host, or dynamic-api)."
            )
    runtime_root = config.targets_root / "runtime"
    lib_target_dir = runtime_root / "lib"
    bin_target_dir = runtime_root / "bin"
    preview_feature_arg = config.runtime_preview_feature_arg
    cargo_preview_binary = _cargo_host_binary(config, "zircon_runtime")
    published_preview_binary = _published_host_binary(config, "zircon_runtime")
    run_cargo(
        config,
        [
            "rustc",
            "-p",
            "zircon_runtime",
            "--lib",
            "--crate-type",
            "cdylib",
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
                cargo_preview_binary,
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
    if shipping_product:
        runtime_library = Path(config.engine_root) / platform_runtime_library_name()
        expected_machine = getattr(config, "effective_target_triple", None)
        if callable(expected_machine):
            expected_machine = expected_machine()
        try:
            abi_report = validate_runtime_abi(
                runtime_library,
                repo_root=config.repo_root,
                expected_machine=expected_machine or "x86_64-pc-windows-msvc",
                expected_kind="dll",
            )
        except AbiValidationError as error:
            raise SystemExit(f"Shipping Runtime ABI validation failed: {error}") from error
        print(
            "Shipping Runtime ABI: "
            f"exports={len(abi_report['exports'])} required={abi_report['required_exports']}"
        )
    if include_preview:
        source_name = platform_executable_name(cargo_preview_binary)
        published_name = platform_executable_name(published_preview_binary)
        if source_name == published_name:
            copy_artifact(config, bin_target_dir, source_name)
        else:
            copy_artifact(
                config, bin_target_dir, source_name, published_name=published_name
            )


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
    environment = managed_cargo_environment(
        config.targets_root / "shader_prewarm", config.targets_root
    )
    print(quote_command(command))
    result = subprocess.run(
        command, cwd=config.repo_root, check=False, env=environment
    )
    print_shader_prewarm_report_dimensions(config.shader_prewarm_report_path)
    if result.returncode == 0:
        validate_staged_shader_prewarm_acceptance_contract(config)
    result.check_returncode()


def build_editor(config: BuildConfig, editor_feature_arg: str) -> None:
    target_dir = config.targets_root / "editor"
    cargo_binary = _cargo_host_binary(config, "zircon_editor")
    published_binary = _published_host_binary(config, "zircon_editor")
    run_cargo(
        config,
        [
            "build",
            "-p",
            "zircon_app",
            "--bin",
            cargo_binary,
            "--no-default-features",
            "--features",
            editor_feature_arg,
            "--target-dir",
            str(target_dir),
        ],
    )
    if config.dry_run:
        return
    source_name = platform_executable_name(cargo_binary)
    published_name = platform_executable_name(published_binary)
    if source_name == published_name:
        copy_artifact(config, target_dir, source_name)
    else:
        copy_artifact(config, target_dir, source_name, published_name=published_name)


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
    elif config.mode in {"shipping", "shipping-symbols"}:
        command.extend(["--profile", config.mode])
    target_triple = getattr(config, "effective_target_triple", None)
    if callable(target_triple):
        target_triple = target_triple()
    if target_triple and "--target" not in command:
        command.extend(["--target", str(target_triple)])
    if config.jobs:
        command.extend(["--jobs", config.jobs])
    if config.dry_run:
        print("DRY-RUN", quote_command(command))
        return
    try:
        target_index = args.index("--target-dir") + 1
    except ValueError as error:
        raise ValueError("Cargo build command must declare --target-dir.") from error
    if target_index >= len(args):
        raise ValueError("Cargo build command is missing a target directory value.")
    target_dir = Path(args[target_index])
    if not target_dir.is_absolute():
        target_dir = config.repo_root / target_dir
    environment = managed_cargo_environment(target_dir, config.targets_root)
    print(quote_command(command))
    subprocess.run(command, cwd=config.repo_root, check=True, env=environment)


def _cargo_host_binary(config: object, default: str) -> str:
    profile = getattr(config, "product_profile", None)
    return getattr(profile, "cargo_app_binary", default) if profile is not None else default


def _published_host_binary(config: object, default: str) -> str:
    profile = getattr(config, "product_profile", None)
    return getattr(profile, "app_binary", default) if profile is not None else default


def copy_artifact(
    config: BuildConfig,
    target_dir: Path,
    artifact_name: str,
    *,
    published_name: str | None = None,
) -> None:
    artifact = find_artifact(target_dir, config.profile_dir, artifact_name)
    destination_name = published_name or artifact.name
    copy_file(artifact, config.engine_root / destination_name, config)
    shipping = bool(
        getattr(config, "is_shipping", False)
        or getattr(config, "mode", "") in {"shipping", "shipping-symbols"}
    )
    sidecar_destination = config.symbols_root if shipping else config.engine_root
    copy_sidecars(
        artifact,
        sidecar_destination,
        config,
        published_name=destination_name,
    )


def find_artifact(target_dir: Path, profile_dir: str, artifact_name: str) -> Path:
    profile_roots = [target_dir / profile_dir]
    # Cargo places target-specific products under <target>/<triple>/<profile>
    # when --target is supplied.  Keep the legacy layout as the first lookup
    # so existing development builds remain unchanged.
    profile_roots.extend(
        candidate
        for candidate in sorted(target_dir.glob("*/" + profile_dir))
        if candidate not in profile_roots
    )
    for profile_root in profile_roots:
        candidates = [profile_root / artifact_name, profile_root / "deps" / artifact_name]
        candidates.extend(profile_root.rglob(artifact_name) if profile_root.exists() else [])
        for candidate in candidates:
            if candidate.exists() and candidate.is_file():
                return candidate
    roots = ", ".join(str(root) for root in profile_roots)
    raise SystemExit(f"Built artifact not found under {roots}: {artifact_name}")


def write_native_plugin_load_manifest(
    config: BuildConfig, native_packages: Sequence[PluginPackage]
) -> None:
    manifest_path = config.engine_root / PLUGIN_LOAD_MANIFEST
    lines = ["# Generated by tools/build/zircon_build.py.\n"]
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


if __name__ == "__main__":
    raise SystemExit(main())
