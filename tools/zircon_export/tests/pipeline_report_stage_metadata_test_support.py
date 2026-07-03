from __future__ import annotations

from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


def write_library_embed_reports(out: Path) -> None:
    _write_validate_report_with_strategies(out, ["library_embed"])
    _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
    _write_stage_report(out, "cook_assets", fatal=False)
    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
    _write_stage_report(out, "platform_bundle", fatal=False)