#!/usr/bin/env python3
"""Model the remaining full-presentation diagnostic refresh pressure.

This is a source-bound operation model. It intentionally does not claim CPU,
allocator, GPU, RSS, or measured input latency.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/app/runtime_diagnostics_visibility.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs",
    "zircon_editor/src/ui/retained_host/ui/shell_content_presentation.rs",
    "zircon_editor/src/ui/retained_host/ui/scoped_presentation.rs",
    "zircon_editor/src/ui/retained_host/host_contract/data/pane_presentation_patch.rs",
    "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs",
)

SOURCE_GUARDS = {
    SOURCE_PATHS[0]: (
        "RuntimeDiagnosticsRefreshTarget::FullPresentation",
        "active_diagnostic_document_tab",
        "RuntimeDiagnosticsRefreshTarget::ShellContent",
    ),
    SOURCE_PATHS[1]: (
        "self.schedule_runtime_diagnostics_refresh();",
        "self.mark_presentation_dirty();",
        "self.invalidate_host_for_shell_content(scope, HostInvalidationMask::SHELL_CONTENT);",
    ),
    SOURCE_PATHS[2]: (
        "RecomputeInvalidationTarget::ShellContent(scope)",
        "RecomputeInvalidationTarget::ViewPresentation(view_ids)",
        "self.build_recompute_shell_snapshot(requested_shell_layout_reuse)",
    ),
    SOURCE_PATHS[3]: (
        "patch_shell_content_presentation_from_state(",
        "ShellContentPatchFallback::Document",
        "ShellContentPatchFallback::HitIndex",
    ),
    SOURCE_PATHS[4]: (
        "ui.patch_host_presentation_panes(",
        "HostPanePresentationLocation::Floating",
        "HostPanePresentationLocation::NativeFloating",
    ),
    SOURCE_PATHS[5]: (
        "pub(crate) struct HostPanePresentationPatch",
        ".with_row_patches(floating_rows)",
        ".with_row_patches(native_floating_rows)",
    ),
    SOURCE_PATHS[6]: (
        "pub(crate) fn patch_host_presentation_panes(",
        ".rebind_paint_models(&model_replacements)",
    ),
}


def git_output(root: Path, *args: str) -> str:
    return subprocess.check_output(("git", *args), cwd=root, text=True).strip()


def source_binding(root: Path) -> dict[str, Any]:
    files: list[dict[str, str]] = []
    for relative in SOURCE_PATHS:
        data = (root / relative).read_bytes()
        files.append({"path": relative, "sha256": hashlib.sha256(data).hexdigest().upper()})
    manifest = "\n".join(f"{item['path']}:{item['sha256']}" for item in files).encode()
    return {
        "head": git_output(root, "rev-parse", "HEAD"),
        "files": files,
        "manifest_sha256": hashlib.sha256(manifest).hexdigest().upper(),
    }


def validate_source_contract(root: Path) -> None:
    for relative, guards in SOURCE_GUARDS.items():
        source = (root / relative).read_text(encoding="utf-8")
        for guard in guards:
            if guard not in source:
                raise AssertionError(f"missing source guard {guard!r} in {relative}")

    visibility = (root / SOURCE_PATHS[0]).read_text(encoding="utf-8")
    if "if has_non_drawer_target" not in visibility or "return RuntimeDiagnosticsRefreshTarget::FullPresentation" not in visibility:
        raise AssertionError("document/floating diagnostics must retain explicit full fallback")

    scoped = (root / SOURCE_PATHS[4]).read_text(encoding="utf-8")
    production = scoped.split("#[cfg(test)]\n#[derive(Default)]", 1)[0]
    if "update_host_presentation_if(" in production:
        raise AssertionError("scoped pane patch must not reenter the generic updater")


def pressure_report(*, refresh_count: int = 200, unrelated_pane_count: int = 8) -> dict[str, Any]:
    if min(refresh_count, unrelated_pane_count) < 0:
        raise ValueError("counts must be non-negative")

    legacy_full_refreshes = refresh_count
    legacy_unrelated_pane_visits = refresh_count * unrelated_pane_count
    target_pane_payload_builds = refresh_count
    target_unrelated_pane_visits = 0
    return {
        "schema": "zircon.editor.runtime_diagnostics_scope_pressure.v2",
        "inputs": {
            "successful_render_refresh_count": refresh_count,
            "unrelated_pane_count": unrelated_pane_count,
        },
        "current_document_or_floating_fallback": {
            "full_presentation_refresh_count": legacy_full_refreshes,
            "unrelated_pane_visit_count": legacy_unrelated_pane_visits,
            "global_hit_index_rebind_boundary_count": legacy_full_refreshes,
        },
        "target_pane_scoped_publication": {
            "diagnostic_pane_payload_build_count": target_pane_payload_builds,
            "unrelated_pane_visit_count": target_unrelated_pane_visits,
            "row_level_floating_storage_replacements": target_pane_payload_builds,
            "damage_region_count_per_refresh": 1,
        },
        "delta": {
            "avoided_full_presentation_refresh_count": legacy_full_refreshes,
            "avoided_unrelated_pane_visits": legacy_unrelated_pane_visits,
            "avoided_global_hit_index_rebind_boundary_count": legacy_full_refreshes,
        },
        "interpretation": {
            "timing_claim": False,
            "authority": "publication target is selected before render submission; pane patch owns the published surface frame",
            "required_fallback": "missing pane identity, changed layout/topology, or hit-index mismatch remains a typed full fallback",
            "excluded": "CPU time, allocator latency, GPU time, RSS, and measured input latency",
        },
    }


def run(root: Path | None = None) -> dict[str, Any]:
    repo = root or Path(__file__).resolve().parents[4]
    validate_source_contract(repo)
    result = pressure_report()
    result["source_binding"] = source_binding(repo)
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--refresh-count", type=int, default=200)
    parser.add_argument("--unrelated-pane-count", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = pressure_report(
        refresh_count=args.refresh_count,
        unrelated_pane_count=args.unrelated_pane_count,
    )
    repo = Path(__file__).resolve().parents[4]
    validate_source_contract(repo)
    result["source_binding"] = source_binding(repo)
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        if args.output.drive.upper() == "C:":
            raise ValueError("profile artifacts must not be written to C:")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
