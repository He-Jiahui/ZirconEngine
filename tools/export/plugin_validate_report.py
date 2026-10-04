"""Report assembly and text rendering for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any


PLUGIN_VALIDATE_COMMAND = "plugin validate"


def plugin_validate_report(
    *,
    args: Any,
    requested_plugin_id: str,
    repo_root: Path | None,
    workspace_manifest: Path | None,
    plugin_manifest_path: Path | None,
    engine_version: str | None,
    package_id: str,
    source_kind: str,
    dist_crate: str | None,
    dist_crate_manifest: Path | None,
    abi_version: int | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    return {
        "command": PLUGIN_VALIDATE_COMMAND,
        "requested_plugin_id": requested_plugin_id,
        "package_id": package_id,
        "source_kind": source_kind,
        "form": args.form,
        "repo_root": str(repo_root) if repo_root else None,
        "workspace_manifest": str(workspace_manifest) if workspace_manifest else None,
        "plugin_manifest": str(plugin_manifest_path) if plugin_manifest_path else None,
        "engine_version": engine_version,
        "dist_crate": dist_crate,
        "dist_crate_manifest": (
            str(dist_crate_manifest) if dist_crate_manifest else None
        ),
        "abi_version": abi_version,
        "fatal": bool(diagnostics),
        "diagnostics": list(diagnostics),
    }


def plugin_validate_all_report(
    *,
    args: Any,
    repo_root: Path | None,
    workspace_manifest: Path | None,
    engine_version: str | None,
    diagnostics: list[str],
    items: list[dict[str, Any]],
) -> dict[str, Any]:
    failed_count = sum(1 for item in items if item["fatal"])
    return {
        "command": f"{PLUGIN_VALIDATE_COMMAND} --all",
        "form": args.form,
        "repo_root": str(repo_root) if repo_root else None,
        "workspace_manifest": str(workspace_manifest) if workspace_manifest else None,
        "engine_version": engine_version,
        "target_count": len(items),
        "failed_count": failed_count,
        "fatal": bool(diagnostics) or failed_count > 0,
        "diagnostics": list(diagnostics),
        "items": items,
    }


def render_plugin_validate_report(report: dict[str, Any]) -> str:
    lines = [
        f"zircon_export {PLUGIN_VALIDATE_COMMAND} id={report['requested_plugin_id']} "
        f"form={report['form']}",
        f"repo_root={report['repo_root'] or '<invalid>'}",
        f"workspace_manifest={report['workspace_manifest'] or '<invalid>'}",
        f"plugin_manifest={report['plugin_manifest'] or '<invalid>'}",
        f"engine_version={report['engine_version'] or '<invalid>'}",
        f"package_id={report['package_id']}",
        f"source_kind={report['source_kind']}",
        f"dist_crate={report['dist_crate'] or '<invalid>'}",
        f"dist_crate_manifest={report['dist_crate_manifest'] or '<invalid>'}",
        f"abi_version={report['abi_version'] or '<invalid>'}",
        "status=failed" if report["fatal"] else "status=ok",
    ]
    diagnostics = report.get("diagnostics", [])
    if isinstance(diagnostics, list):
        lines.extend(f"diagnostic={diagnostic}" for diagnostic in diagnostics)
    return "\n".join(lines) + "\n"


def render_plugin_validate_all_report(report: dict[str, Any]) -> str:
    lines = [
        f"zircon_export {PLUGIN_VALIDATE_COMMAND} --all form={report['form']}",
        f"repo_root={report['repo_root'] or '<invalid>'}",
        f"workspace_manifest={report['workspace_manifest'] or '<invalid>'}",
        f"engine_version={report['engine_version'] or '<invalid>'}",
        f"target_count={report['target_count']}",
        f"failed_count={report['failed_count']}",
        "status=failed" if report["fatal"] else "status=ok",
    ]
    diagnostics = report.get("diagnostics", [])
    if isinstance(diagnostics, list):
        lines.extend(f"diagnostic={diagnostic}" for diagnostic in diagnostics)
    items = report.get("items", [])
    if isinstance(items, list):
        for item in items:
            if not isinstance(item, dict):
                continue
            status = "failed" if item.get("fatal") else "ok"
            lines.append(
                "item="
                f"{item.get('package_id', '<invalid>')} "
                f"source_kind={item.get('source_kind', '<invalid>')} "
                f"dist_crate={item.get('dist_crate') or '<invalid>'} "
                f"status={status}"
            )
            item_diagnostics = item.get("diagnostics", [])
            if isinstance(item_diagnostics, list):
                lines.extend(
                    f"item_diagnostic={item.get('package_id', '<invalid>')}: {diagnostic}"
                    for diagnostic in item_diagnostics
                )
    return "\n".join(lines) + "\n"
