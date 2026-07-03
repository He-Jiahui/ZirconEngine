from __future__ import annotations

from pathlib import Path

from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _native_plugins_content_hash,
    _native_plugins_file_manifest,
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_stage_report,
)


def _refresh_platform_native_plugins_payload(
    out: Path,
    native_plugins: Path,
) -> None:
    platform_report = _read_stage_report(out, "platform_bundle")
    payload = platform_report["native_plugins_payload"]
    assert isinstance(payload, dict)
    file_manifest = _native_plugins_file_manifest(native_plugins)
    payload["file_manifest"] = file_manifest
    payload["file_count"] = len(file_manifest)
    payload["content_hash"] = _native_plugins_content_hash(file_manifest)
    _write_stage_report(out, "platform_bundle", platform_report)
    _write_bundle_manifest_from_platform_report(
        out / "bundle" / "windows-release" / "bundle.json",
        platform_report,
    )
