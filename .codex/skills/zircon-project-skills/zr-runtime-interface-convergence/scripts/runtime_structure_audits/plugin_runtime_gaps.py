from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable


WITH_PLUGIN_RE = re.compile(r"\.with_plugin\s*\(")
RESOLVE_PLUGIN_RE = re.compile(r"\bresolve_plugin(?:\s*::<|\s*\()")
PLUGIN_CONTEXT_RE = re.compile(r"\bPluginContext\b")


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _find_locations(
    root: Path,
    files: Iterable[Path],
    pattern: re.Pattern[str],
) -> list[dict[str, object]]:
    results: list[dict[str, object]] = []
    for path in files:
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            if pattern.search(line):
                results.append(
                    {
                        "path": _relative(root, path),
                        "line": line_no,
                        "snippet": line.strip(),
                    }
                )
    return results


def plugin_runtime_gaps(
    root: Path,
    all_rs_files: Iterable[Path],
    zircon_crates: Iterable[str],
) -> list[str]:
    all_rs_files = list(all_rs_files)
    zircon_crates = set(zircon_crates)

    with_plugin_sites = _find_locations(root, all_rs_files, WITH_PLUGIN_RE)
    resolve_plugin_sites = _find_locations(root, all_rs_files, RESOLVE_PLUGIN_RE)
    plugin_context_sites = [
        site
        for site in _find_locations(root, all_rs_files, PLUGIN_CONTEXT_RE)
        if not str(site["path"]).startswith("zircon_core/")
        and not str(site["path"]).startswith("zircon_module/")
    ]

    gaps: list[str] = []
    if not with_plugin_sites:
        gaps.append(
            "No module currently registers runtime plugins through ModuleDescriptor::with_plugin."
        )
    if not resolve_plugin_sites:
        gaps.append(
            "No runtime call site currently resolves plugins through CoreHandle::resolve_plugin."
        )
    if not plugin_context_sites:
        gaps.append(
            "PluginContext is still effectively a core-only abstraction and is not consumed by higher-level runtime code."
        )

    if "zircon_render_server" in zircon_crates and "zircon_server" not in zircon_crates:
        gaps.append(
            "Roadmap naming still points at zircon_server while the current workspace exposes zircon_render_server."
        )

    return gaps
