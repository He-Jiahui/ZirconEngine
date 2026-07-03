"""Command-line argument declarations for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import sys
from typing import Sequence


STAGES = (
    "validate",
    "source_template",
    "native_dynamic",
    "compile_host",
    "cook_assets",
    "pack",
    "platform_bundle",
    "report",
)
RESUMABLE_STAGES = STAGES
DEFAULT_OUT = "zircon-export"


def parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    argv_list = list(argv) if argv is not None else sys.argv[1:]
    stage_explicit = option_present(argv_list, "--stage")
    resume_from_explicit = option_present(argv_list, "--resume-from")
    pack_file_explicit = option_present(argv_list, "--pack-file")
    delta_pack_explicit = option_present(argv_list, "--delta-pack")
    host_executable_explicit = option_present(argv_list, "--host-executable")

    parser = argparse.ArgumentParser(
        description="Run the staged Zircon export pipeline.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python -m tools.zircon_export --profile windows-release --project zircon-project.toml --out E:\\zircon-export
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage validate --dry-run
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage source_template --dry-run
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage native_dynamic
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage compile_host --dry-run
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage cook_assets --asset-manifest cooked-assets.json
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage pack
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage platform_bundle --template-dir tools/zircon_export/export-templates\\windows-x86_64-library_embed-debug
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage platform_bundle --template-root tools/zircon_export/export-templates --target-platform windows-x86_64
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --stage report
  python -m tools.zircon_export --profile windows-release --out E:\\zircon-export --resume-from pack
""".strip(),
    )
    parser.add_argument("--profile", required=True, help="Export profile name.")
    parser.add_argument(
        "--project",
        default="zircon-project.toml",
        help="Project manifest path. Default: zircon-project.toml.",
    )
    parser.add_argument(
        "--out",
        "--output",
        default=DEFAULT_OUT,
        help=f"Export output directory. Default: {DEFAULT_OUT}.",
    )
    parser.add_argument(
        "--stage",
        choices=STAGES,
        default="validate",
        help="Single pipeline stage to run. Omit --stage to run the full main pipeline.",
    )
    parser.add_argument(
        "--resume-from",
        choices=RESUMABLE_STAGES,
        default=None,
        help=(
            "Run the main export pipeline from this stage through report. "
            "Cannot be combined with --stage."
        ),
    )
    parser.add_argument(
        "--repo-root",
        default=None,
        help="Repository root for Cargo commands. Default: auto-detect from this package.",
    )
    parser.add_argument(
        "--cargo",
        default="cargo",
        help="Cargo executable used when --validator is not supplied. Default: cargo.",
    )
    parser.add_argument(
        "--validator",
        default=None,
        help="Prebuilt zircon_export_validate executable. Skips cargo run when supplied.",
    )
    parser.add_argument(
        "--packer",
        default=None,
        help="Prebuilt zircon_export_pack executable. Skips cargo run when supplied.",
    )
    parser.add_argument(
        "--validate-report",
        default=None,
        help=(
            "Validate report consumed by CompileHost. "
            "Default: <out>/stages/validate/report.json."
        ),
    )
    parser.add_argument(
        "--native-plugin-root",
        default=None,
        help=(
            "Root directory containing NativeDynamic plugin packages. "
            "Default: <repo-root>/zircon_plugins."
        ),
    )
    parser.add_argument(
        "--asset-manifest",
        default=None,
        help=(
            "Cooked asset manifest input for CookAssets, or explicit Pack input. "
            "Pack defaults from a matching CookAssets report, then "
            "<out>/stages/cook_assets/assets.json."
        ),
    )
    parser.add_argument(
        "--asset-filter",
        default=None,
        help=(
            "Default asset filter written by CookAssets when the cooked manifest "
            "does not declare asset_filter. Defaults from a matching Validate report."
        ),
    )
    parser.add_argument(
        "--pack-file",
        default=None,
        help=(
            "Pack output file. PlatformBundle defaults from a matching Pack report, "
            "then <out>/stages/pack/assets.zrpack."
        ),
    )
    parser.add_argument(
        "--previous-pack",
        default=None,
        help="Previous full zrpack used to build a delta pack. Requires --delta-pack.",
    )
    parser.add_argument(
        "--delta-pack",
        default=None,
        help="Delta zrpack output file. Requires --previous-pack.",
    )
    parser.add_argument(
        "--host-executable",
        default=None,
        help="Compiled host executable copied into PlatformBundle when available.",
    )
    parser.add_argument(
        "--native-plugins-dir",
        default=None,
        help=(
            "NativeDynamic plugins directory copied into PlatformBundle. "
            "Defaults from a matching NativeDynamic report when reported."
        ),
    )
    parser.add_argument(
        "--template-dir",
        default=None,
        help=(
            "Export template package directory containing template.toml. "
            "When supplied, PlatformBundle validates the template before copying outputs."
        ),
    )
    parser.add_argument(
        "--template-root",
        default=None,
        help=(
            "Root directory containing export-template packages. "
            "Used by PlatformBundle to resolve a compatible template when --template-dir is omitted."
        ),
    )
    parser.add_argument(
        "--engine-version",
        default=None,
        help=(
            "Engine version expected by template.toml. "
            "Default: [workspace.package].version from Cargo.toml."
        ),
    )
    parser.add_argument(
        "--target-platform",
        default=None,
        help="Optional target platform id used to reject mismatched export templates.",
    )
    parser.add_argument(
        "--determinism-check",
        action="store_true",
        help="Run a second in-memory pack write and compare bytes.",
    )
    parser.add_argument(
        "--source-template-build",
        action="store_true",
        help=(
            "Execute cargo build for the materialized SourceTemplate project. "
            "Without this flag, SourceTemplate only writes the generated project and report."
        ),
    )
    parser.add_argument(
        "--native-dynamic-build",
        action="store_true",
        help=(
            "Execute Cargo cdylib builds for NativeDynamic packages and stage the built "
            "loadable artifacts. Without this flag, NativeDynamic only consumes existing "
            "package native artifacts."
        ),
    )
    parser.add_argument(
        "--native-dynamic-build-feature",
        action="append",
        default=[],
        help=(
            "Cargo feature passed to NativeDynamic cdylib package builds. May be repeated; "
            "recorded in the native_build_plan and applied when --native-dynamic-build runs."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-command",
        default=None,
        help=(
            "External signer executable for NativeDynamic loadable artifacts. "
            "Disabled when omitted."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-arg",
        action="append",
        default=[],
        help=(
            "Argument appended to --native-dynamic-sign-command. May be repeated; "
            "supports {artifact}, {package_id}, {package_dir}, {target_platform}, "
            "and {signing_profile}."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-profile",
        default=None,
        help=(
            "Audit label for the NativeDynamic signing profile. This does not select "
            "platform certificate stores by itself; it is passed to the external signer."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-platform",
        action="append",
        default=[],
        help=(
            "Target platform prefix allowed by the NativeDynamic signing profile. "
            "May be repeated, for example windows, linux, or macos."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-command",
        default=None,
        help=(
            "External notarization or platform post-processing executable for "
            "NativeDynamic loadable artifacts. Disabled when omitted."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-arg",
        action="append",
        default=[],
        help=(
            "Argument appended to --native-dynamic-notarize-command. May be repeated; "
            "supports {artifact}, {package_id}, {package_dir}, {target_platform}, "
            "{signing_profile}, and {notarization_profile}."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-profile",
        default=None,
        help=(
            "Audit label for the NativeDynamic notarization/profile post-processing "
            "step. This does not select platform services by itself; it is passed to "
            "the external command."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-platform",
        action="append",
        default=[],
        help=(
            "Target platform prefix allowed by the NativeDynamic notarization profile. "
            "May be repeated, for example windows, linux, or macos."
        ),
    )
    parser.add_argument(
        "--target-dir",
        default=None,
        help="Cargo target directory passed through to cargo run.",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Pass --offline to Cargo commands.",
    )
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Do not pass --locked to Cargo. Locked mode is the default.",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Emit pretty JSON from the Rust validate report generator.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the selected stage command without executing it.",
    )
    args = parser.parse_args(argv)
    args.stage_explicit = stage_explicit
    args.resume_from_explicit = resume_from_explicit
    args.pack_file_explicit = pack_file_explicit
    args.delta_pack_explicit = delta_pack_explicit
    args.host_executable_explicit = host_executable_explicit
    if args.resume_from and stage_explicit:
        parser.error("--resume-from runs the main pipeline and cannot be combined with --stage")
    return args


def option_present(argv: Sequence[str], option: str) -> bool:
    prefix = option + "="
    return any(value == option or value.startswith(prefix) for value in argv)
