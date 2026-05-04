#!/usr/bin/env python3
"""Build and stage Zircon editor, runtime, and plugin artifacts."""

from __future__ import annotations

import argparse
import dataclasses
import filecmp
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


TARGETS = ("editor", "runtime", "plugins")
MODES = ("debug", "release")
PLUGIN_CARRIERS = ("all", "native_dynamic", "rlib_static")
ENGINE_DIR_NAME = "ZirconEngine"
PLUGIN_LOAD_MANIFEST = "plugins/native_plugins.toml"
ENGINE_ASSET_ROOTS = (
    Path("zircon_editor") / "assets",
    Path("zircon_runtime") / "assets",
)


@dataclasses.dataclass(frozen=True)
class CargoPackage:
    name: str
    member: str
    manifest_path: Path
    crate_types: tuple[str, ...]

    @property
    def is_native_dynamic(self) -> bool:
        return "cdylib" in self.crate_types


@dataclasses.dataclass(frozen=True)
class PluginPackage:
    plugin_id: str
    display_name: str
    manifest_path: Path
    package_root: Path
    default_packaging: tuple[str, ...]
    module_crate_names: tuple[str, ...]
    crates: tuple[CargoPackage, ...]

    @property
    def native_dynamic_crates(self) -> tuple[CargoPackage, ...]:
        return tuple(crate for crate in self.crates if crate.is_native_dynamic)

    @property
    def rlib_static_crates(self) -> tuple[CargoPackage, ...]:
        return tuple(crate for crate in self.crates if not crate.is_native_dynamic)

    @property
    def carriers(self) -> tuple[str, ...]:
        carriers: list[str] = []
        if self.native_dynamic_crates:
            carriers.append("native_dynamic")
        if self.rlib_static_crates:
            carriers.append("rlib_static")
        return tuple(carriers)


@dataclasses.dataclass(frozen=True)
class BuildConfig:
    repo_root: Path
    out_root: Path
    mode: str
    targets: tuple[str, ...]
    plugins: tuple[PluginPackage, ...]
    plugin_carrier: str
    locked: bool
    jobs: str | None
    dry_run: bool

    @property
    def engine_root(self) -> Path:
        return self.out_root / ENGINE_DIR_NAME

    @property
    def targets_root(self) -> Path:
        return self.out_root / "targets"

    @property
    def profile_dir(self) -> str:
        return "release" if self.mode == "release" else "debug"


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
        description="Build staged ZirconEngine editor/runtime/plugin artifacts.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python tools/zircon_build.py --targets editor,runtime --out E:\\zircon-build --mode debug
  python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\\zircon-build --mode debug
  python tools/zircon_build.py --targets plugins --plugins all --plugin-carrier native_dynamic --out E:\\zircon-build --mode release

Plugin carrier boundary:
  native_dynamic crates are cdylib plugins copied into ZirconEngine/plugins.
  rlib_static crates are built into targets/plugins/<id> and remain static-link inputs.
""".strip(),
    )
    parser.add_argument(
        "--targets",
        "--target",
        help="Comma-separated build targets: editor,runtime,plugins.",
    )
    parser.add_argument("--out", "--output", help="Build output directory.")
    parser.add_argument("--mode", choices=MODES, help="Cargo profile mode.")
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
    plugin_carrier = args.plugin_carrier

    selected_plugins: tuple[PluginPackage, ...] = ()
    if "plugins" in targets:
        candidates = filter_plugins_by_carrier(plugin_catalog, plugin_carrier)
        if args.plugins:
            selected_plugins = tuple(select_plugins(candidates, args.plugins))
        else:
            selected_plugins = tuple(prompt_plugins(candidates))
        if not selected_plugins:
            raise SystemExit("No plugins selected for the plugins target.")

    return BuildConfig(
        repo_root=repo_root,
        out_root=out_root,
        mode=mode,
        targets=targets,
        plugins=selected_plugins,
        plugin_carrier=plugin_carrier,
        locked=not args.no_locked,
        jobs=args.jobs or None,
        dry_run=args.dry_run,
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


def resolve_out_root(raw: str) -> Path:
    path = Path(raw).expanduser()
    if not path.is_absolute():
        path = (Path.cwd() / path).resolve()
    return path


def prompt_targets() -> tuple[str, ...]:
    require_tty("--targets")
    print("Select build targets:")
    for index, target in enumerate(TARGETS, start=1):
        print(f"  {index}) {target}")
    raw = input("Targets (comma numbers or names, default editor,runtime): ").strip()
    if not raw:
        return ("editor", "runtime")
    return parse_targets(resolve_number_tokens(raw, TARGETS))


def prompt_out_root() -> Path:
    require_tty("--out")
    raw = input("Build output directory: ").strip()
    if not raw:
        raise SystemExit("Build output directory is required.")
    return resolve_out_root(raw)


def prompt_mode() -> str:
    require_tty("--mode")
    raw = input("Build mode [debug/release] (default debug): ").strip().lower()
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
        default_packaging = tuple(normalize_packaging(data.get("default_packaging", [])))
        module_crate_names = tuple(unique_in_order(collect_module_crate_names(data)))
        matched_crates = tuple(
            crates_by_name[name] for name in module_crate_names if name in crates_by_name
        )
        packages.append(
            PluginPackage(
                plugin_id=plugin_id,
                display_name=display_name,
                manifest_path=manifest_path,
                package_root=manifest_path.parent,
                default_packaging=default_packaging,
                module_crate_names=module_crate_names,
                crates=matched_crates,
            )
        )
    return tuple(sorted(packages, key=lambda item: item.plugin_id))


def discover_plugin_workspace_crates(plugins_root: Path) -> tuple[CargoPackage, ...]:
    workspace = read_toml(plugins_root / "Cargo.toml")
    members = workspace.get("workspace", {}).get("members", [])
    packages: list[CargoPackage] = []
    for member in members:
        manifest_path = plugins_root / member / "Cargo.toml"
        if not manifest_path.exists():
            continue
        data = read_toml(manifest_path)
        package = data.get("package", {})
        name = package.get("name")
        if not name:
            continue
        crate_types = data.get("lib", {}).get("crate-type", [])
        packages.append(
            CargoPackage(
                name=str(name),
                member=str(member).replace("\\", "/"),
                manifest_path=manifest_path,
                crate_types=tuple(str(crate_type) for crate_type in crate_types),
            )
        )
    return tuple(packages)


def read_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def normalize_packaging(values: object) -> list[str]:
    if not isinstance(values, list):
        return []
    return [str(value).strip().lower() for value in values if str(value).strip()]


def collect_module_crate_names(data: dict) -> list[str]:
    crate_names: list[str] = []
    for module in data.get("modules", []):
        append_module_crate(crate_names, module)
    for feature_key in ("optional_features", "feature_extensions"):
        for feature in data.get(feature_key, []):
            for module in feature.get("modules", []):
                append_module_crate(crate_names, module)
    return crate_names


def append_module_crate(crate_names: list[str], module: object) -> None:
    if not isinstance(module, dict):
        return
    crate_name = module.get("crate_name")
    if crate_name:
        crate_names.append(str(crate_name))


def filter_plugins_by_carrier(
    packages: Sequence[PluginPackage], plugin_carrier: str
) -> list[PluginPackage]:
    if plugin_carrier == "all":
        return list(packages)
    return [package for package in packages if plugin_carrier in package.carriers]


def select_plugins(candidates: Sequence[PluginPackage], raw: str) -> list[PluginPackage]:
    if not candidates:
        return []
    by_id = {package.plugin_id.lower(): package for package in candidates}
    selected: list[PluginPackage] = []
    for token in parse_csv(raw):
        if token == "all":
            selected.extend(candidates)
        elif token in ("native", "native_dynamic"):
            selected.extend(package for package in candidates if package.native_dynamic_crates)
        elif token in ("rlib", "rlib_static", "static"):
            selected.extend(package for package in candidates if package.rlib_static_crates)
        elif "-" in token and token.replace("-", "").isdigit():
            selected.extend(select_range(candidates, token))
        elif token.isdigit():
            selected.append(select_index(candidates, int(token)))
        elif token in by_id:
            selected.append(by_id[token])
        else:
            raise SystemExit(f"Unknown plugin selector: {token}")
    return unique_plugins(selected)


def select_index(candidates: Sequence[PluginPackage], index: int) -> PluginPackage:
    if index < 1 or index > len(candidates):
        raise SystemExit(f"Plugin index out of range: {index}")
    return candidates[index - 1]


def select_range(candidates: Sequence[PluginPackage], token: str) -> list[PluginPackage]:
    start_raw, end_raw = token.split("-", 1)
    start = int(start_raw)
    end = int(end_raw)
    if start > end:
        start, end = end, start
    return [select_index(candidates, index) for index in range(start, end + 1)]


def unique_plugins(packages: Iterable[PluginPackage]) -> list[PluginPackage]:
    seen: set[str] = set()
    result: list[PluginPackage] = []
    for package in packages:
        if package.plugin_id in seen:
            continue
        seen.add(package.plugin_id)
        result.append(package)
    return result


def print_plugin_catalog(packages: Sequence[PluginPackage]) -> None:
    print("Discovered plugins:")
    for index, package in enumerate(packages, start=1):
        carriers = ",".join(package.carriers) or "manifest_only"
        crate_names = ",".join(crate.name for crate in package.crates) or "no matched crate"
        print(f"  {index:2d}) {package.plugin_id:32s} [{carriers}] {crate_names}")


def print_plan(config: BuildConfig) -> None:
    print("Zircon build plan")
    print(f"  repo:    {config.repo_root}")
    print(f"  out:     {config.out_root}")
    print(f"  mode:    {config.mode}")
    print(f"  targets: {','.join(config.targets)}")
    print(f"  locked:  {config.locked}")
    if config.jobs:
        print(f"  jobs:    {config.jobs}")
    if config.dry_run:
        print("  dry-run: enabled")
    if config.plugins:
        print("  plugins:")
        for package in config.plugins:
            print(f"    - {package.plugin_id} ({','.join(package.carriers) or 'manifest_only'})")


def build(config: BuildConfig) -> None:
    if not config.dry_run:
        config.engine_root.mkdir(parents=True, exist_ok=True)
        config.targets_root.mkdir(parents=True, exist_ok=True)

    runtime_staged = False
    if "runtime" in config.targets:
        build_runtime(config, runtime_feature="target-client", include_preview=True)
        runtime_staged = True
    if "editor" in config.targets:
        if not runtime_staged:
            build_runtime(config, runtime_feature="target-editor-host", include_preview=False)
            runtime_staged = True
        build_editor(config)
    if "editor" in config.targets or "runtime" in config.targets:
        stage_engine_assets(config)
    if "plugins" in config.targets:
        ensure_plugin_base_artifacts(config)
        build_plugins(config)


def build_runtime(
    config: BuildConfig, runtime_feature: str, include_preview: bool
) -> None:
    runtime_root = config.targets_root / "runtime"
    lib_target_dir = runtime_root / "lib"
    bin_target_dir = runtime_root / "bin"
    run_cargo(
        config,
        [
            "build",
            "-p",
            "zircon_runtime",
            "--lib",
            "--no-default-features",
            "--features",
            runtime_feature,
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
                "target-client",
                "--target-dir",
                str(bin_target_dir),
            ],
        )
    if config.dry_run:
        return
    copy_artifact(config, lib_target_dir, platform_runtime_library_name())
    if include_preview:
        copy_artifact(config, bin_target_dir, platform_executable_name("zircon_runtime"))


def build_editor(config: BuildConfig) -> None:
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
            "target-editor-host",
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
    command = ["cargo", *args]
    if config.locked:
        command.append("--locked")
    if config.mode == "release":
        command.append("--release")
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


def stage_engine_assets(config: BuildConfig) -> None:
    destination_root = config.engine_root / "assets"
    if config.dry_run:
        print(f"DRY-RUN reset {destination_root}")
    else:
        if destination_root.exists():
            shutil.rmtree(destination_root)
        destination_root.mkdir(parents=True, exist_ok=True)

    for relative_root in ENGINE_ASSET_ROOTS:
        source_root = config.repo_root / relative_root
        if not source_root.exists() or not source_root.is_dir():
            raise SystemExit(f"Engine asset root is missing: {source_root}")
        print(f"Staging assets {source_root} -> {destination_root}")
        copy_tree_contents(source_root, destination_root, config)


def copy_tree_contents(source_root: Path, destination_root: Path, config: BuildConfig) -> None:
    for source in sorted(source_root.rglob("*")):
        relative = source.relative_to(source_root)
        destination = destination_root / relative
        if source.is_dir():
            if config.dry_run:
                print(f"DRY-RUN mkdir {destination}")
            else:
                destination.mkdir(parents=True, exist_ok=True)
            continue
        if not source.is_file():
            continue
        copy_asset_file(source, destination, config)


def copy_asset_file(source: Path, destination: Path, config: BuildConfig) -> None:
    if destination.exists():
        if destination.is_file() and filecmp.cmp(source, destination, shallow=False):
            return
        raise SystemExit(
            "Engine asset staging collision: "
            f"{source} cannot overwrite existing {destination} with different content."
        )
    copy_file(source, destination, config)


def copy_resource_dirs(source_root: Path, package_out: Path, config: BuildConfig) -> None:
    for name in ("assets", "asset", "resources", "resource"):
        source = source_root / name
        if not source.exists() or not source.is_dir():
            continue
        destination = package_out / name
        if config.dry_run:
            print(f"DRY-RUN copytree {source} -> {destination}")
            continue
        if destination.exists():
            shutil.rmtree(destination)
        shutil.copytree(source, destination)
        print(f"Copied {source} -> {destination}")


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
