from __future__ import annotations

import json
from pathlib import Path

from tools.zircon_export.tests import native_dynamic_export_test_support as native_dynamic_support


def json_dumps(value: object) -> str:
    return json.dumps(value, indent=2)


def json_loads(value: str) -> object:
    return json.loads(value)


def _write_platform_bundle_report_with_native_plugins_payload(
    out: Path,
    native_plugins_payload: dict[str, object],
    *,
    fatal: bool = False,
    profile: str = "windows-release",
    host_executable: Path | None = None,
    host_source: Path | None = None,
    host_source_origin: str | None = None,
    pack: Path | None = None,
    pack_source: Path | None = None,
    pack_source_origin: str | None = None,
    delta_pack: Path | None = None,
    delta_pack_source: Path | None = None,
    delta_pack_source_origin: str | None = None,
    bundle_manifest: Path | None = None,
    write_bundle_manifest: bool = True,
    bundle_manifest_overrides: dict[str, object] | None = None,
    write_output_files: bool = True,
    missing_output_fields: set[str] | None = None,
) -> None:
    report_dir = out / "stages" / "platform_bundle"
    report_dir.mkdir(parents=True, exist_ok=True)
    if not fatal:
        if (
            host_executable is None
            and host_source is None
            and host_source_origin is None
        ):
            host_executable = out / "bundle" / profile / "zircon_runtime.exe"
            host_source = _stage_report_path_value(
                out,
                "compile_host",
                "host_executable",
                out / "stages" / "compile_host" / "zircon_runtime.exe",
            )
            host_source_origin = "compile_host_report"
        if host_source is not None and host_source_origin is None:
            host_source_origin = "compile_host_report"
        if pack is None and pack_source is None:
            pack = out / "bundle" / profile / "assets.zrpack"
            pack_source = _stage_report_path_value(
                out,
                "pack",
                "pack",
                out / "stages" / "pack" / "assets.zrpack",
            )
        if pack_source is not None and pack_source_origin is None:
            pack_source_origin = "pack_report"
        if delta_pack_source is not None and delta_pack_source_origin is None:
            delta_pack_source_origin = "pack_report"
    report: dict[str, object] = {
        "stage": "PlatformBundle",
        "profile": profile,
        "bundle": str(out / "bundle" / profile),
        "fatal": fatal,
        "diagnostics": ["platform bundle failed"] if fatal else [],
        "template_resolution": None,
        "template": None,
        "native_plugins_payload": native_plugins_payload if fatal else None,
        "template_files": [],
    }
    if native_plugins_payload and not fatal:
        plugins_dir = native_dynamic_support._write_native_dynamic_stage_plugins(out / "bundle" / profile)
        report["native_plugins"] = str(plugins_dir)
        payload_overrides = dict(native_plugins_payload)
        native_plugins_payload.clear()
        native_plugins_payload.update(
            _platform_bundle_native_plugins_payload(
                plugins_dir,
                payload_overrides,
            )
        )
        report["native_plugins_payload"] = native_plugins_payload
    if host_executable is not None:
        report["host_executable"] = str(host_executable)
    if host_source is not None:
        report["host_source"] = str(host_source)
    if host_source_origin is not None:
        report["host_source_origin"] = host_source_origin
    if pack is not None:
        report["pack"] = str(pack)
    if pack_source is not None:
        report["pack_source"] = str(pack_source)
    if pack_source_origin is not None:
        report["pack_source_origin"] = pack_source_origin
    if delta_pack is not None:
        report["delta_pack"] = str(delta_pack)
    if delta_pack_source is not None:
        report["delta_pack_source"] = str(delta_pack_source)
    if delta_pack_source_origin is not None:
        report["delta_pack_source_origin"] = delta_pack_source_origin
    if not fatal and write_output_files:
        _write_platform_bundle_output_files(
            report,
            missing_output_fields=missing_output_fields or set(),
        )
    if not fatal and (bundle_manifest is not None or write_bundle_manifest):
        manifest_path = bundle_manifest or out / "bundle" / profile / "bundle.json"
        report["bundle_manifest"] = str(manifest_path)
        if write_bundle_manifest:
            manifest = _platform_bundle_manifest_from_report(report)
            if bundle_manifest_overrides:
                manifest.update(bundle_manifest_overrides)
            manifest_path.parent.mkdir(parents=True, exist_ok=True)
            manifest_path.write_text(json_dumps(manifest), encoding="utf-8")
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def _stage_report_path_value(
    out: Path,
    stage: str,
    field: str,
    fallback: Path,
) -> Path:
    report_path = out / "stages" / stage / "report.json"
    if report_path.is_file():
        report = json_loads(report_path.read_text(encoding="utf-8"))
        if isinstance(report, dict):
            value = report.get(field)
            if isinstance(value, str) and value:
                return Path(value)
    return fallback


def _write_platform_bundle_output_files(
    report: dict[str, object],
    *,
    missing_output_fields: set[str],
) -> None:
    source_fields = {
        "host_executable": "host_source",
        "pack": "pack_source",
        "delta_pack": "delta_pack_source",
    }
    for field in ("host_executable", "pack", "delta_pack"):
        if field in missing_output_fields:
            continue
        value = report.get(field)
        if not isinstance(value, str) or not value:
            continue
        path = Path(value)
        path.parent.mkdir(parents=True, exist_ok=True)
        source_value = report.get(source_fields[field])
        if isinstance(source_value, str) and source_value and Path(source_value).is_file():
            path.write_bytes(Path(source_value).read_bytes())
        else:
            path.write_text(f"{field} placeholder", encoding="utf-8")


def _platform_bundle_native_plugins_payload(
    plugins_dir: Path,
    payload_overrides: dict[str, object],
) -> dict[str, object]:
    file_manifest = native_dynamic_support._native_dynamic_plugins_file_manifest(plugins_dir)
    payload: dict[str, object] = {
        "stage_report": payload_overrides.get("stage_report"),
        "source": str(plugins_dir),
        "bundle_path": str(plugins_dir),
        "loader_manifest": str(plugins_dir / "native_plugins.toml"),
        "content_hash": native_dynamic_support._native_dynamic_content_hash(file_manifest),
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": 1,
        "materialized_packages": [
            {
                "package_id": "animation",
                "destination": str(plugins_dir / "animation"),
                "package_report": str(
                    plugins_dir / "animation" / "native_dynamic_package.toml"
                ),
                "loadable_artifact_count": 1,
                "loadable_artifacts": [
                    "plugins/animation/native/zircon_plugin_animation.dll"
                ],
            }
        ],
    }
    for key in ("native_signing", "native_notarization"):
        if key in payload_overrides:
            payload[key] = payload_overrides[key]
    return payload


def _platform_bundle_manifest_from_report(
    report: dict[str, object],
) -> dict[str, object]:
    return {
        "profile": report.get("profile"),
        "template_resolution": report.get("template_resolution"),
        "template": report.get("template"),
        "host_executable": report.get("host_executable"),
        "host_source": report.get("host_source"),
        "host_source_origin": report.get("host_source_origin"),
        "pack": report.get("pack"),
        "pack_source": report.get("pack_source"),
        "pack_source_origin": report.get("pack_source_origin"),
        "delta_pack": report.get("delta_pack"),
        "delta_pack_source": report.get("delta_pack_source"),
        "delta_pack_source_origin": report.get("delta_pack_source_origin"),
        "native_plugins": report.get("native_plugins"),
        "native_plugins_payload": report.get("native_plugins_payload"),
        "template_files": report.get("template_files", []),
    }