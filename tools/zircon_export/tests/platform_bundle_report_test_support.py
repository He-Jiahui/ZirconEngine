from __future__ import annotations

import hashlib
import json
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _compile_host_link_plan,
    _compile_host_plan,
    _native_dynamic_build_execution_report,
    _native_dynamic_build_plan_report,
    _native_dynamic_operation_audit_report,
    _native_dynamic_operation_audit_summary_report,
    _native_dynamic_package_export,
    _pack_binary_bytes,
    _write_validate_report_with_strategies,
)
from tools.zircon_export.tests.pack_test_support import (
    empty_delta_manifest,
    empty_pack_document_manifest,
)


TEMPLATE_FORMAT_VERSION = 1
TEMPLATE_ID = "fixture-template"
TEMPLATE_ENGINE_VERSION = "0.1.0"
TEMPLATE_TARGET_PLATFORM = "windows-x86_64"
TEMPLATE_HOST_KIND = "desktop"
TEMPLATE_HOST_ARTIFACT = "precompiled"
TEMPLATE_RESOURCE_STRATEGY = "filesystem_bundle"
TEMPLATE_PLUGIN_STRATEGY = "native_dynamic_allowed"
TEMPLATE_BUNDLE_FORMAT = "directory"
TEMPLATE_COMPATIBLE_PROFILES = ("windows-release",)


def _write_platform_bundle_fixture(
    out: Path,
    *,
    payload_overrides: dict[str, object] | None = None,
    with_template_file: bool = False,
    with_delta: bool = False,
    include_bundle: bool = True,
    bundle_dir: Path | None = None,
    bundle_manifest: Path | None = None,
    host_output: Path | None = None,
    pack_output: Path | None = None,
    template_output: Path | None = None,
) -> dict[str, Path]:
    profile = "windows-release"
    host = out / "compile" / "zircon_runtime.exe"
    pack = out / "pack-output" / "assets.zrpack"
    bundle_dir = bundle_dir or out / "bundle" / profile
    platform_host = host_output or bundle_dir / "zircon_runtime.exe"
    platform_pack = pack_output or bundle_dir / "assets.zrpack"
    delta_pack = out / "pack-output" / "assets.delta.zrpd"
    previous_pack = out / "pack-output" / "previous.zrpack"
    platform_delta = bundle_dir / "assets.delta.zrpd"
    native_stage_plugins = out / "stages" / "native_dynamic" / "plugins"
    native_plugins = bundle_dir / "plugins"
    native_source_package = out / "zircon_plugins" / "animation"
    bundle_manifest = bundle_manifest or bundle_dir / "bundle.json"
    template_source = out / "template" / "Info.plist"
    template_file = template_output or bundle_dir / "Contents" / "Info.plist"
    _write_text(host, "host")
    pack.parent.mkdir(parents=True, exist_ok=True)
    pack.write_bytes(_pack_binary_bytes(empty_pack_document_manifest(), b"ZRPK"))
    _copy_file(host, platform_host)
    _copy_file(pack, platform_pack)
    if with_delta:
        delta_pack.parent.mkdir(parents=True, exist_ok=True)
        delta_pack.write_bytes(_pack_binary_bytes(empty_delta_manifest(), b"ZRPD"))
        previous_pack.write_bytes(
            _pack_binary_bytes(empty_pack_document_manifest(), b"ZRPK")
        )
        _copy_file(delta_pack, platform_delta)
    _write_native_plugins(native_stage_plugins)
    _write_native_plugins(native_plugins)
    _write_text(native_source_package / "plugin.toml", 'id = "animation"\n')
    native_payload = _native_plugins_payload(native_plugins)
    native_payload["stage_report"] = str(
        out / "stages" / "native_dynamic" / "report.json"
    )
    native_payload["source"] = str(native_stage_plugins)
    if payload_overrides:
        native_payload.update(payload_overrides)
    cooked_manifest = out / "stages" / "cook_assets" / "assets.json"
    _write_text(cooked_manifest, json.dumps({"roots": [], "assets": []}, indent=2))
    copied_template_files: list[dict[str, object]] = []
    template_report: dict[str, object] | None = None
    if with_template_file:
        _write_text(template_source, "<plist>zircon</plist>")
        _write_text(template_file, "<plist>zircon</plist>")
        template_payload = template_source.read_bytes()
        template_hash = hashlib.sha256(template_payload).hexdigest()
        embedded_template_files = [
            {
                "path": template_source.name,
                "bundle_path": "Contents/Info.plist",
                "sha256": template_hash,
                "purpose": "platform_metadata",
            }
        ]
        template_content_hash = _template_content_hash(embedded_template_files)
        _write_template_manifest(template_source.parent, template_content_hash, template_hash)
        template_report = {
            "bundle": {
                "delta_pack_path": "",
                "host_path": "",
                "manifest_path": "bundle.json",
                "pack_path": "",
                "root": ".",
            },
            "bundle_format": TEMPLATE_BUNDLE_FORMAT,
            "compatible_profiles": list(TEMPLATE_COMPATIBLE_PROFILES),
            "computed_content_hash": template_content_hash,
            "content_hash": template_content_hash,
            "diagnostics": [],
            "engine_version": TEMPLATE_ENGINE_VERSION,
            "expected_engine_version": TEMPLATE_ENGINE_VERSION,
            "expected_format_version": TEMPLATE_FORMAT_VERSION,
            "expected_target_platform": TEMPLATE_TARGET_PLATFORM,
            "fatal": False,
            "files": embedded_template_files,
            "format_version": TEMPLATE_FORMAT_VERSION,
            "host_artifact": TEMPLATE_HOST_ARTIFACT,
            "host_executable": str(template_source),
            "host_kind": TEMPLATE_HOST_KIND,
            "manifest": str(template_source.parent / "template.toml"),
            "plugin_strategy": TEMPLATE_PLUGIN_STRATEGY,
            "profile": profile,
            "resource_strategy": TEMPLATE_RESOURCE_STRATEGY,
            "target_platform": TEMPLATE_TARGET_PLATFORM,
            "template_dir": str(template_source.parent),
            "template_id": TEMPLATE_ID,
        }
        copied_template_files.append(
            {
                "source": str(template_source),
                "destination": str(template_file),
            }
        )

    _write_validate_report_with_strategies(out, ["native_dynamic"], profile=profile)
    native_stage_materialized_packages = _native_plugins_payload(native_stage_plugins)[
        "materialized_packages"
    ]
    for package in native_stage_materialized_packages:
        if isinstance(package, dict) and package.get("package_id") == "animation":
            package["source"] = str(native_source_package)
    _write_report(
        out,
        "native_dynamic",
        {
            "stage": "NativeDynamic",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "stage_output": str(out / "stages" / "native_dynamic"),
            "validate_report": str(out / "stages" / "validate" / "report.json"),
            "target_platform": "windows-x86_64",
            "artifact_extensions": [".dll"],
            "native_plugin_root": str(out / "zircon_plugins"),
            "plugins_dir": str(native_stage_plugins),
            "loader_manifest": str(native_stage_plugins / "native_plugins.toml"),
            "content_hash": _native_plugins_content_hash(
                _native_plugins_file_manifest(native_stage_plugins)
            ),
            "file_manifest": _native_plugins_file_manifest(native_stage_plugins),
            "native_dynamic_packages": ["animation"],
            "package_exports": [_native_dynamic_package_export()],
            "package_count": 1,
            "native_build_plan": _native_dynamic_build_plan_report(),
            "native_build_execution": _native_dynamic_build_execution_report(),
            "native_signing": _native_dynamic_operation_audit_report(),
            "native_notarization": _native_dynamic_operation_audit_report(),
            "materialized_packages": native_stage_materialized_packages,
        },
    )
    _write_report(
        out,
        "compile_host",
        {
            "stage": "CompileHost",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "command": list(_compile_host_plan()["command"]),
            "exit_code": 0,
            "host_executable": str(host),
            "link_plan": _compile_host_link_plan(),
            "stdout_lines": [],
            "stderr_lines": [],
        },
    )
    _write_report(
        out,
        "cook_assets",
        {
            "stage": "CookAssets",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "source_asset_manifest": None,
            "project_manifest": None,
            "generated_from_project": False,
            "project_default_scene": None,
            "cooked_asset_manifest": str(cooked_manifest),
            "cooked_asset_manifest_sha256": hashlib.sha256(
                cooked_manifest.read_bytes()
            ).hexdigest(),
            "asset_count": 0,
            "root_count": 0,
            "asset_filter": None,
        },
    )
    _write_report(
        out,
        "pack",
        {
            "stage": "Pack",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "asset_manifest": str(out / "stages" / "cook_assets" / "assets.json"),
            "pack": str(pack),
            "stage_output": str(out / "stages" / "pack"),
            "trim_report": {
                "included_assets": [],
                "trimmed_assets": [],
                "missing_dependencies": [],
                "duplicate_assets": [],
                "diagnostics": [],
            },
            "manifest": {
                "pack": {
                    "version": 1,
                    "chunks": [],
                    "total_size": 0,
                },
                "assets": [],
            },
            "asset_count": 0,
            "chunk_count": 0,
            "deduplicated_assets": [],
            "deterministic_double_run": False,
            "previous_pack": str(previous_pack) if with_delta else None,
            "delta_pack": str(delta_pack) if with_delta else None,
            "delta_manifest": empty_delta_manifest() if with_delta else None,
            "delta_asset_count": 0,
            "delta_chunk_count": 0,
            "delta_removed_assets": [],
            "delta_reused_assets": [],
            "delta_apply_verified": True if with_delta else None,
        },
    )
    platform_report = {
        "stage": "PlatformBundle",
        "profile": profile,
        "fatal": False,
        "diagnostics": [],
        "template_resolution": None,
        "template": template_report,
        "host_executable": str(platform_host),
        "host_source": str(host),
        "host_source_origin": "compile_host_report",
        "pack": str(platform_pack),
        "pack_source": str(pack),
        "pack_source_origin": "pack_report",
        "delta_pack": str(platform_delta) if with_delta else None,
        "delta_pack_source": str(delta_pack) if with_delta else None,
        "delta_pack_source_origin": "pack_report" if with_delta else None,
        "native_plugins": str(native_plugins),
        "native_plugins_payload": native_payload,
        "template_files": copied_template_files,
        "bundle_manifest": str(bundle_manifest),
    }
    if include_bundle:
        platform_report["bundle"] = str(bundle_dir)
    _write_bundle_manifest_from_platform_report(bundle_manifest, platform_report)
    _write_report(out, "platform_bundle", platform_report)
    return {
        "native_plugins": native_plugins,
        "template_file": template_file,
        "host_source": host,
        "pack_source": pack,
        "delta_source": delta_pack,
        "platform_host": platform_host,
        "platform_pack": platform_pack,
        "platform_delta": platform_delta,
        "bundle_manifest": bundle_manifest,
    }


def _write_template_manifest(
    template_dir: Path,
    content_hash: str,
    file_hash: str,
) -> None:
    _write_text(
        template_dir / "template.toml",
        "\n".join(
            [
                f"format_version = {TEMPLATE_FORMAT_VERSION}",
                f'template_id = "{TEMPLATE_ID}"',
                f'engine_version = "{TEMPLATE_ENGINE_VERSION}"',
                f'target_platform = "{TEMPLATE_TARGET_PLATFORM}"',
                f'host_kind = "{TEMPLATE_HOST_KIND}"',
                f'host_artifact = "{TEMPLATE_HOST_ARTIFACT}"',
                f'resource_strategy = "{TEMPLATE_RESOURCE_STRATEGY}"',
                f'plugin_strategy = "{TEMPLATE_PLUGIN_STRATEGY}"',
                f'bundle_format = "{TEMPLATE_BUNDLE_FORMAT}"',
                f'content_hash = "{content_hash}"',
                'compatible_profiles = ["windows-release"]',
                "",
                "[paths]",
                'host_executable = "Info.plist"',
                "[[files]]",
                'path = "Info.plist"',
                'bundle_path = "Contents/Info.plist"',
                'purpose = "platform_metadata"',
                f'sha256 = "{file_hash}"',
                "",
            ]
        ),
    )


def _template_content_hash(files: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in sorted(files, key=lambda value: str(value["path"])):
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry.get("bundle_path", "")).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def _write_native_plugins(native_plugins: Path) -> None:
    package_export = _native_dynamic_package_export()
    abi = package_export["abi"]
    _write_text(
        native_plugins / "native_plugins.toml",
        "\n".join(
            [
                "[[plugins]]",
                'id = "animation"',
                'path = "plugins/animation"',
                'manifest = "plugins/animation/plugin.toml"',
                'package_report = "plugins/animation/native_dynamic_package.toml"',
                "",
                "[plugins.abi]",
                f'abi_version = {abi["abi_version"]}',
                f'descriptor_symbol = "{abi["descriptor_symbol"]}"',
                f'descriptor_contract = "{abi["descriptor_contract"]}"',
                f'runtime_entry_source = "{abi["runtime_entry_source"]}"',
                f'editor_entry_source = "{abi["editor_entry_source"]}"',
                f'host_function_table = "{abi["host_function_table"]}"',
                f'entry_report_contract = "{abi["entry_report_contract"]}"',
                f'behavior_contract = "{abi["behavior_contract"]}"',
                f'state_snapshot_contract = "{abi["state_snapshot_contract"]}"',
                f'bridge_method_table = "{abi["bridge_method_table"]}"',
            ]
        )
        + "\n",
    )
    _write_text(
        native_plugins / "animation" / "native" / "zircon_plugin_animation.dll",
        "plugin dll",
    )
    _write_native_dynamic_package_report(native_plugins / "animation")


def _native_plugins_payload(native_plugins: Path) -> dict[str, object]:
    file_manifest = _native_plugins_file_manifest(native_plugins)
    materialized_package = {
        "package_id": "animation",
        "destination": str(native_plugins / "animation"),
        "package_report": str(
            native_plugins / "animation" / "native_dynamic_package.toml"
        ),
        "loadable_artifact_count": 1,
        "loadable_artifacts": [
            "plugins/animation/native/zircon_plugin_animation.dll"
        ],
    }
    return {
        "stage_report": None,
        "source": str(native_plugins),
        "bundle_path": str(native_plugins),
        "loader_manifest": str(native_plugins / "native_plugins.toml"),
        "content_hash": _native_plugins_content_hash(file_manifest),
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": 1,
        "native_signing": _native_dynamic_operation_audit_summary_report(),
        "native_notarization": _native_dynamic_operation_audit_summary_report(),
        "materialized_packages": [materialized_package],
    }


def _native_plugins_file_manifest(native_plugins: Path) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    for file_path in sorted(native_plugins.rglob("*")):
        if not file_path.is_file():
            continue
        payload = file_path.read_bytes()
        entries.append(
            {
                "path": f"plugins/{file_path.relative_to(native_plugins).as_posix()}",
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(entries, key=lambda entry: str(entry["path"]))


def _native_plugins_content_hash(file_manifest: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in file_manifest:
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["bytes"]).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def _native_plugin_package_payload_file_manifest(
    package_dir: Path,
) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    for file_path in sorted(package_dir.rglob("*")):
        if not file_path.is_file() or file_path.name == "native_dynamic_package.toml":
            continue
        payload = file_path.read_bytes()
        entries.append(
            {
                "path": file_path.relative_to(package_dir).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(entries, key=lambda entry: str(entry["path"]))


def _write_native_dynamic_package_report(package_dir: Path) -> None:
    payload_files = _native_plugin_package_payload_file_manifest(package_dir)
    _write_text(
        package_dir / "native_dynamic_package.toml",
        _native_dynamic_package_report_toml(payload_files),
    )


def _native_dynamic_package_report_toml(
    payload_files: list[dict[str, object]],
) -> str:
    lines = [
        "# Generated by Zircon export. Native dynamic package report.",
        "format_version = 1",
        'package_id = "animation"',
        'directory = "animation"',
        'path = "plugins/animation"',
        'manifest = "plugins/animation/plugin.toml"',
        "",
        "[abi]",
        "abi_version = 3",
        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
        'descriptor_contract = "NativePluginAbiV3"',
        'runtime_entry_source = "NativePluginAbiV3.runtime_entry_name"',
        'editor_entry_source = "NativePluginAbiV3.editor_entry_name"',
        'host_function_table = "NativePluginHostFunctionTableV3"',
        'entry_report_contract = "NativePluginEntryReportV3"',
        'behavior_contract = "NativePluginBehaviorV3"',
        'state_snapshot_contract = "NativePluginBehaviorV3.save_state/restore_state"',
        'bridge_method_table = "NativePluginBridgeMethodTableV3"',
        "",
        "[payload]",
        f"file_count = {len(payload_files)}",
        f'content_hash = "{_native_plugins_content_hash(payload_files)}"',
    ]
    for entry in payload_files:
        lines.extend(
            [
                "",
                "[[payload.files]]",
                f'path = "{entry["path"]}"',
                f'bytes = {entry["bytes"]}',
                f'sha256 = "{entry["sha256"]}"',
            ]
        )
    return "\n".join(lines) + "\n"


def _read_stage_report(out: Path, stage: str) -> dict[str, object]:
    return json.loads(
        (out / "stages" / stage / "report.json").read_text(encoding="utf-8")
    )


def _write_stage_report(out: Path, stage: str, report: dict[str, object]) -> None:
    _write_report(out, stage, report)


def _write_bundle_manifest_from_platform_report(
    bundle_manifest: Path,
    platform_report: dict[str, object],
) -> None:
    bundle_manifest.parent.mkdir(parents=True, exist_ok=True)
    bundle_manifest.write_text(
        json.dumps(
            {
                "profile": platform_report["profile"],
                "template_resolution": platform_report["template_resolution"],
                "template": platform_report["template"],
                "host_executable": platform_report.get("host_executable"),
                "host_source": platform_report.get("host_source"),
                "host_source_origin": platform_report.get("host_source_origin"),
                "pack": platform_report.get("pack"),
                "pack_source": platform_report.get("pack_source"),
                "pack_source_origin": platform_report.get("pack_source_origin"),
                "delta_pack": platform_report.get("delta_pack"),
                "delta_pack_source": platform_report.get("delta_pack_source"),
                "delta_pack_source_origin": platform_report.get(
                    "delta_pack_source_origin"
                ),
                "native_plugins": platform_report.get("native_plugins"),
                "native_plugins_payload": platform_report.get("native_plugins_payload"),
                "template_files": platform_report.get("template_files"),
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _copy_file(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(source.read_bytes())


def _remove_tree(path: Path) -> None:
    for child in sorted(path.rglob("*"), reverse=True):
        if child.is_file():
            child.unlink()
        elif child.is_dir():
            child.rmdir()
    path.rmdir()


def _write_report(out: Path, stage: str, report: dict[str, object]) -> None:
    report_dir = out / "stages" / stage
    report_dir.mkdir(parents=True, exist_ok=True)
    (report_dir / "report.json").write_text(
        json.dumps(report, indent=2),
        encoding="utf-8",
    )
