"""PlatformBundle NativeDynamic payload loader-manifest diagnostics."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
)
from .pipeline_report_native_dynamic_loader_manifest import (
    native_dynamic_loader_manifest_plugins_or_diagnostics,
    native_dynamic_loader_manifest_row_field_diagnostics,
)


def platform_bundle_native_plugins_loader_manifest_diagnostics(
    payload: dict[str, Any],
    plugins_dir: Path,
) -> list[str]:
    loader_manifest = payload.get("loader_manifest")
    if not isinstance(loader_manifest, str):
        return []
    if not loader_manifest.strip():
        return [
            "PlatformBundle report native_plugins_payload loader_manifest "
            "must be a non-empty string"
        ]
    diagnostics: list[str] = []
    loader_manifest_path = _resolve_user_path_or_diagnostic(
        loader_manifest,
        diagnostics,
        "PlatformBundle report native_plugins_payload loader_manifest",
    )
    expected_manifest_path = _resolve_user_path_or_diagnostic(
        plugins_dir / NATIVE_DYNAMIC_LOADER_MANIFEST,
        diagnostics,
        "PlatformBundle current bundle loader manifest",
    )
    if loader_manifest_path is None or expected_manifest_path is None:
        return diagnostics
    if loader_manifest_path != expected_manifest_path:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"{loader_manifest_path} does not match current bundle loader manifest "
            f"{expected_manifest_path}"
        )
        return diagnostics
    if not loader_manifest_path.exists():
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"{loader_manifest_path} does not exist"
        )
    elif not loader_manifest_path.is_file():
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"{loader_manifest_path} is not a file"
        )
    return diagnostics


def platform_bundle_native_plugins_loader_manifest_package_diagnostics(
    payload: dict[str, Any],
    packages: list[dict[str, object]],
    *,
    stage_backed_payload: bool = False,
) -> list[str]:
    loader_manifest = payload.get("loader_manifest")
    if not isinstance(loader_manifest, str):
        return []
    if not loader_manifest.strip():
        return [
            "PlatformBundle report native_plugins_payload loader_manifest "
            "must be a non-empty string"
        ]
    diagnostics: list[str] = []
    loader_manifest_path = _resolve_user_path_or_diagnostic(
        loader_manifest,
        diagnostics,
        "PlatformBundle report native_plugins_payload loader_manifest",
    )
    if loader_manifest_path is None:
        return diagnostics
    plugins, plugin_diagnostics = native_dynamic_loader_manifest_plugins_or_diagnostics(
        loader_manifest_path,
        label="PlatformBundle report native_plugins_payload loader_manifest",
    )
    if plugin_diagnostics:
        return plugin_diagnostics
    assert plugins is not None
    plugin_ids = [str(plugin["id"]) for plugin in plugins]
    package_ids = [str(package["package_id"]) for package in packages]
    if plugin_ids != package_ids:
        return [
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"plugin ids {plugin_ids} do not match materialized package ids "
            f"{package_ids}"
        ]
    return native_dynamic_loader_manifest_row_field_diagnostics(
        plugins,
        platform_bundle_native_plugins_loader_manifest_expected_plugins_by_id(
            packages,
            loader_manifest_path.parent,
        ),
        label="PlatformBundle report native_plugins_payload loader_manifest",
        expected_label="materialized package",
        require_fields=stage_backed_payload,
    )


def platform_bundle_native_plugins_loader_manifest_expected_plugins_by_id(
    packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, dict[str, Any]]:
    expected_plugins: dict[str, dict[str, Any]] = {}
    try:
        plugins_root = plugins_dir.resolve()
    except OSError:
        return expected_plugins

    for package in packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(plugins_root)
        except (OSError, ValueError):
            continue
        path = f"plugins/{relative_destination.as_posix().rstrip('/')}"
        expected_plugin = {
            "path": path,
            "manifest": f"{path}/plugin.toml",
            "abi": {
                "abi_version": 3,
                **NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
            },
        }
        package_report = package.get("package_report")
        if isinstance(package_report, str):
            package_report_path = Path(package_report).expanduser()
            try:
                relative_package_report = package_report_path.resolve().relative_to(
                    plugins_root
                )
            except (OSError, ValueError):
                pass
            else:
                expected_plugin["package_report"] = (
                    f"plugins/{relative_package_report.as_posix()}"
                )
        expected_plugins[package_id] = expected_plugin
    return expected_plugins


def _resolve_user_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return Path(path).expanduser().resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None
