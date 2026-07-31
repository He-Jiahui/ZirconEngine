from __future__ import annotations

import argparse
import json
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _export_args as _base_export_args,
    _validate_runtime_plugin_availability,
)


def _export_args(
    *,
    out: Path,
    stage: str,
    dry_run: bool,
) -> argparse.Namespace:
    args = _base_export_args(out=out, stage=stage, dry_run=dry_run)
    args.native_dynamic_build = False
    args.native_dynamic_build_feature = []
    args.native_dynamic_sign_command = None
    args.native_dynamic_sign_arg = []
    args.native_dynamic_sign_profile = None
    args.native_dynamic_sign_platform = []
    args.native_dynamic_notarize_command = None
    args.native_dynamic_notarize_arg = []
    args.native_dynamic_notarize_profile = None
    args.native_dynamic_notarize_platform = []
    return args


def _write_validate_report_with_native_dynamic_exports(
    out: Path,
    *,
    profile: str,
    target_platform: str,
    build_mode: str | None = None,
    abi_overrides: dict[str, object] | None = None,
    native_dynamic_packages: list[str] | None = None,
    package_export_overrides: dict[str, object] | None = None,
) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": "Validate",
                "profile": profile,
                "project_manifest": str(out / "zircon-project.toml"),
                "stage_output": str(report_dir),
                "profile_found": True,
                "fatal": False,
                "diagnostics": [],
                "fatal_diagnostics": [],
                "profile_summary": {
                    "name": profile,
                    "target_mode": "client_runtime",
                    "strategies": ["native_dynamic"],
                    "target_platform": target_platform,
                    "build_mode": build_mode or "debug",
                    "selected_plugins": [],
                    "features": {},
                },
                "plan_summary": {
                    "enabled_runtime_plugins": [],
                    "linked_runtime_crates": [],
                    "native_dynamic_packages": native_dynamic_packages or ["animation"],
                    "generated_files": [],
                    "runtime_plugin_availability": _validate_runtime_plugin_availability(),
                    "native_dynamic_package_exports": [
                        _native_dynamic_package_export(
                            abi_overrides,
                            package_export_overrides,
                        )
                    ],
                },
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _native_dynamic_package_export(
    abi_overrides: dict[str, object] | None = None,
    package_export_overrides: dict[str, object] | None = None,
) -> dict[str, object]:
    abi: dict[str, object] = {
        "abi_version": 3,
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "descriptor_contract": "NativePluginAbiV3",
        "runtime_entry_source": "NativePluginAbiV3.runtime_entry_name",
        "editor_entry_source": "NativePluginAbiV3.editor_entry_name",
        "host_function_table": "NativePluginHostFunctionTableV3",
        "entry_report_contract": "NativePluginEntryReportV3",
        "behavior_contract": "NativePluginBehaviorV4",
        "state_snapshot_contract": "NativePluginBehaviorV4.save_state/restore_state",
        "bridge_method_table": "NativePluginBridgeMethodTableV3",
    }
    if abi_overrides:
        abi.update(abi_overrides)
    package_export: dict[str, object] = {
        "package_id": "animation",
        "directory": "animation",
        "path": "plugins/animation",
        "manifest": "plugins/animation/plugin.toml",
        "package_report": "plugins/animation/native_dynamic_package.toml",
        "abi": abi,
    }
    if package_export_overrides:
        package_export.update(package_export_overrides)
    return package_export


def _write_macos_native_dynamic_package_fixture(repo_root: Path) -> None:
    package = repo_root / "zircon_plugins" / "animation"
    native_dir = package / "native"
    dsym_dwarf_dir = (
        native_dir
        / "zircon_plugin_animation.dSYM"
        / "Contents"
        / "Resources"
        / "DWARF"
    )
    native_dir.mkdir(parents=True)
    dsym_dwarf_dir.mkdir(parents=True)
    (package / "plugin.toml").write_text(
        "\n".join(
            [
                'id = "animation"',
                'name = "Animation"',
                'default_packaging = ["native_dynamic"]',
            ]
        ),
        encoding="utf-8",
    )
    (native_dir / "libzircon_plugin_animation.dylib").write_text(
        "native dynamic placeholder",
        encoding="utf-8",
    )
    (dsym_dwarf_dir / "zircon_plugin_animation").write_text(
        "debug symbols placeholder",
        encoding="utf-8",
    )


def _write_windows_native_dynamic_package_fixture_at(
    repo_root: Path,
    relative_package_path: Path,
    *,
    package_id: str,
    module_crate_names: list[str] | None = None,
    write_native_artifact: bool = True,
) -> None:
    package = repo_root / "zircon_plugins" / relative_package_path
    native_dir = package / "native"
    package.mkdir(parents=True, exist_ok=True)
    if write_native_artifact:
        native_dir.mkdir(parents=True, exist_ok=True)
    plugin_manifest_lines = [
        f'id = "{package_id}"',
        'name = "Animation"',
        'default_packaging = ["native_dynamic"]',
    ]
    for crate_name in module_crate_names or []:
        plugin_manifest_lines.extend(
            [
                "",
                "[[modules]]",
                f'name = "{package_id}.runtime"',
                'kind = "runtime"',
                f'crate_name = "{crate_name}"',
            ]
        )
    (package / "plugin.toml").write_text(
        "\n".join(plugin_manifest_lines),
        encoding="utf-8",
    )
    if write_native_artifact:
        (native_dir / "zircon_plugin_animation.dll").write_text(
            "native dynamic placeholder",
            encoding="utf-8",
        )


def _write_native_dynamic_cdylib_workspace(
    repo_root: Path,
    member: Path,
    *,
    crate_name: str,
) -> None:
    plugins_root = repo_root / "zircon_plugins"
    plugins_root.mkdir(parents=True, exist_ok=True)
    (plugins_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                f'members = ["{member.as_posix()}"]',
                'resolver = "2"',
            ]
        ),
        encoding="utf-8",
    )
    crate_dir = plugins_root / member
    crate_dir.mkdir(parents=True, exist_ok=True)
    (crate_dir / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{crate_name}"',
                'version = "0.1.0"',
                'edition = "2021"',
                "",
                "[lib]",
                'crate-type = ["cdylib"]',
            ]
        ),
        encoding="utf-8",
    )


def _write_native_dynamic_fake_cargo_build_script(repo_root: Path, crate_name: str) -> None:
    (repo_root / "build").write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                "target_dir = Path(sys.argv[sys.argv.index('--target-dir') + 1])",
                "profile = 'release' if '--release' in sys.argv else 'debug'",
                f"artifact = target_dir / profile / '{crate_name}.dll'",
                "artifact.parent.mkdir(parents=True, exist_ok=True)",
                "artifact.write_text('built native dynamic artifact', encoding='utf-8')",
            ]
        ),
        encoding="utf-8",
    )


def _write_native_dynamic_fake_sign_script(repo_root: Path, exit_code: int = 0) -> Path:
    script = repo_root / "sign_native.py"
    script.write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                f"exit_code = {exit_code}",
                "if exit_code:",
                "    print('signing failed', file=sys.stderr)",
                "    sys.exit(exit_code)",
                "artifact = Path(sys.argv[1])",
                "package_id = sys.argv[2] if len(sys.argv) > 2 else ''",
                "target_platform = sys.argv[3] if len(sys.argv) > 3 else ''",
                "signing_profile = sys.argv[4] if len(sys.argv) > 4 else ''",
                "with artifact.open('a', encoding='utf-8') as output:",
                "    output.write(f'\\nsigned:{package_id}:{target_platform}:{signing_profile}')",
                "print(f'signed {artifact.name}')",
            ]
        ),
        encoding="utf-8",
    )
    return script


def _write_native_dynamic_fake_notarize_script(
    repo_root: Path,
    exit_code: int = 0,
) -> Path:
    script = repo_root / "notarize_native.py"
    script.write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                f"exit_code = {exit_code}",
                "if exit_code:",
                "    print('notarization failed', file=sys.stderr)",
                "    sys.exit(exit_code)",
                "artifact = Path(sys.argv[1])",
                "package_id = sys.argv[2] if len(sys.argv) > 2 else ''",
                "target_platform = sys.argv[3] if len(sys.argv) > 3 else ''",
                "signing_profile = sys.argv[4] if len(sys.argv) > 4 else ''",
                "notarization_profile = sys.argv[5] if len(sys.argv) > 5 else ''",
                "with artifact.open('a', encoding='utf-8') as output:",
                "    output.write(f'\\nnotarized:{package_id}:{target_platform}:{signing_profile}:{notarization_profile}')",
                "print(f'notarized {artifact.name}')",
            ]
        ),
        encoding="utf-8",
    )
    return script
