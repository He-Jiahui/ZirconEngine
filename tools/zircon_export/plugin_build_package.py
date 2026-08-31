"""Plugin build package directory materialization."""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Any

from .native_build_command import platform_dynamic_library_name
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    native_dynamic_package_directory,
)
from .native_dynamic_payload_file_manifest import (
    native_dynamic_package_payload_file_manifest,
)
from .native_dynamic_templates import (
    native_dynamic_package_report_template,
    native_plugin_load_manifest_template,
)
from .plugin_build_asset_pack import materialize_plugin_asset_pack
from .plugin_build_signature import (
    plugin_build_signing_audit,
    write_plugin_build_signature,
)
from .plugin_package_source import resolve_plugin_package_path


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
    resolved_out_root = resolve_plugin_package_path("out", out_root, diagnostics)
    resolved_package_dir = resolve_plugin_package_path(
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
    if not built_artifact.is_file():
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


def plugin_build_abi_contract(
    abi_version: int,
    distribution: dict[str, Any],
) -> dict[str, object]:
    abi = {"abi_version": abi_version, **NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS}
    descriptor_symbol = distribution.get("descriptor_symbol")
    if isinstance(descriptor_symbol, str) and descriptor_symbol.strip():
        abi["descriptor_symbol"] = descriptor_symbol
    return abi
