"""Command-line contract for the staged ZirconEngine build runner.

Keeping argument parsing separate from build orchestration makes the runner's
product/profile behavior easier to audit and prevents the entrypoint from
becoming a second configuration module.
"""

from __future__ import annotations

import argparse
import sys
from typing import Sequence


TARGETS = ("hub", "editor", "runtime", "plugins", "font-sdf")
MODES = ("debug", "release", "profiling", "shipping", "shipping-symbols")
PLUGIN_CARRIERS = ("all", "native_dynamic", "rlib_static")


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    argv = normalize_target_options(argv)
    parser = argparse.ArgumentParser(
        description="Build staged ZirconEngine hub/editor/runtime/plugin artifacts.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python tools/build/zircon_build.py --targets hub,editor,runtime --out E:\\cargo-targets\\zircon --mode debug
  python tools/build/zircon_build.py --targets editor,runtime --out E:\\cargo-targets\\zircon --mode debug
  python tools/build/zircon_build.py --targets runtime --out E:\\cargo-targets\\zircon-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy
  python tools/build/zircon_build.py --product-profile runtime-windows --out E:\\cargo-targets\\zircon-shipping --mode shipping
  python tools/build/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\\cargo-targets\\zircon --mode debug
  python tools/build/zircon_build.py --targets plugins --plugins all --plugin-carrier native_dynamic --out E:\\cargo-targets\\zircon --mode release
  python tools/build/zircon_build.py --targets font-sdf --font-sdf-manifest E:\\cargo-targets\\project\\font-sdf.json --out E:\\cargo-targets\\zircon --mode release

Plugin carrier boundary:
  native_dynamic crates are cdylib plugins copied into ZirconEngine/plugins.
  rlib_static crates are built into targets/plugins/<id> and remain static-link inputs.
""".strip(),
    )
    parser.add_argument(
        "--targets",
        help="Comma-separated build targets: hub,editor,runtime,plugins,font-sdf.",
    )
    parser.add_argument(
        "--target-triple",
        help="Rust target triple. Shipping profiles currently require x86_64-pc-windows-msvc.",
    )
    parser.add_argument(
        "--product-profile",
        help=(
            "Build-only product profile from tools/export/product_build_profiles.toml "
            "(runtime-windows or editor-windows)."
        ),
    )
    parser.add_argument(
        "--product-profile-file",
        help="Override the product profile TOML file.",
    )
    parser.add_argument(
        "--out",
        "--output",
        help="Build output directory under an approved Windows build root.",
    )
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
        "--clean-output",
        action="store_true",
        help="Reset the exact product staging roots before building (shipping defaults to this).",
    )
    parser.add_argument(
        "--project-pack-root",
        help=(
            "Optional cooked project-pack directory to reference in the product size report. "
            "It is never copied into the Runtime/Editor engine baseline."
        ),
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


def normalize_target_options(argv: Sequence[str] | None) -> list[str]:
    """Accept both the legacy build-target alias and Cargo target triples."""

    values = list(sys.argv[1:] if argv is None else argv)
    normalized: list[str] = []
    target_names = set(TARGETS) | {"all"}
    index = 0
    while index < len(values):
        value = values[index]
        if value.startswith("--target="):
            candidate = value.split("=", 1)[1]
            parsed = {
                part.strip().lower()
                for part in candidate.split(",")
                if part.strip()
            }
            normalized.append(
                "--targets" if parsed and parsed <= target_names else "--target-triple"
            )
            normalized.append(candidate)
            index += 1
            continue
        if value == "--target" and index + 1 < len(values):
            candidate = values[index + 1]
            parsed = {
                part.strip().lower()
                for part in candidate.split(",")
                if part.strip()
            }
            normalized.append(
                "--targets" if parsed and parsed <= target_names else "--target-triple"
            )
            normalized.append(candidate)
            index += 2
            continue
        normalized.append(value)
        index += 1
    return normalized
