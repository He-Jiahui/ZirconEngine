"""Plugin build asset subpackage materialization."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from .plugin_package_source import resolve_plugin_package_path
from .plugin_validate_distribution_assets import (
    plugin_validate_retired_ui_asset_pattern_suffix,
    plugin_validate_retired_ui_asset_suffix,
)
from .plugin_validate_distribution_zui_assets import (
    validate_plugin_distribution_zui_asset,
)


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
    initial_diagnostic_count = len(diagnostics)
    asset_entries = plugin_asset_pack_entries(
        plugin_root,
        distribution,
        package_id,
        diagnostics,
    )
    if len(diagnostics) != initial_diagnostic_count:
        return False
    if not asset_entries:
        return True

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
        if not pack_path.is_file():
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

    resolved_plugin_root = resolve_plugin_package_path("plugin root", plugin_root, diagnostics)
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
        if plugin_validate_retired_ui_asset_pattern_suffix(raw_pattern) is not None:
            diagnostics.append(
                f"{label} targets retired UI asset suffix {pattern_path.as_posix()}; use .zui"
            )
            continue

        matches = sorted(path for path in plugin_root.glob(raw_pattern) if path.is_file())
        if not matches:
            diagnostics.append(f"{label} matched no plugin asset files")
            continue
        for source_path in matches:
            resolved_source = resolve_plugin_package_path(
                "plugin asset source",
                source_path,
                diagnostics,
            )
            if resolved_source is None:
                continue
            try:
                relative_source = resolved_source.relative_to(resolved_plugin_root)
            except ValueError:
                diagnostics.append(
                    f"plugin asset source {resolved_source} is outside plugin root {resolved_plugin_root}"
                )
                continue
            relative_path = relative_source.as_posix()
            if plugin_validate_retired_ui_asset_suffix(relative_source) is not None:
                diagnostics.append(
                    f"{label} matched retired UI asset suffix {relative_path}; use .zui"
                )
                continue
            diagnostic_count = len(diagnostics)
            validate_plugin_distribution_zui_asset(
                item_label=label,
                relative_source=relative_source,
                source_path=source_path,
                diagnostics=diagnostics,
            )
            if len(diagnostics) != diagnostic_count:
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
