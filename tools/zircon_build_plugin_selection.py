"""Plugin catalog selection helpers for zircon_build."""

from __future__ import annotations

from typing import Iterable, Sequence

try:
    from .zircon_build_plugin_packages import PluginPackage
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_plugin_packages import PluginPackage


def filter_plugins_by_carrier(
    packages: Sequence[PluginPackage], plugin_carrier: str
) -> list[PluginPackage]:
    if plugin_carrier == "all":
        return list(packages)
    return [package for package in packages if plugin_carrier in package.carriers]


def select_plugins(candidates: Sequence[PluginPackage], raw: str) -> list[PluginPackage]:
    if not candidates:
        return []
    by_id: dict[str, PluginPackage] | None = None
    native_candidates: list[PluginPackage] | None = None
    static_candidates: list[PluginPackage] | None = None
    selected: list[PluginPackage] = []
    for token in _parse_csv(raw):
        if token == "all":
            selected.extend(candidates)
        elif token in ("native", "native_dynamic"):
            if native_candidates is None:
                native_candidates = [
                    package for package in candidates if package.native_dynamic_crates
                ]
            selected.extend(native_candidates)
        elif token in ("rlib", "rlib_static", "static"):
            if static_candidates is None:
                static_candidates = [
                    package for package in candidates if package.rlib_static_crates
                ]
            selected.extend(static_candidates)
        elif "-" in token and token.replace("-", "").isdigit():
            selected.extend(select_range(candidates, token))
        elif token.isdigit():
            selected.append(select_index(candidates, int(token)))
        else:
            if by_id is None:
                by_id = {package.plugin_id.lower(): package for package in candidates}
            package = by_id.get(token)
            if package is None:
                raise SystemExit(f"Unknown plugin selector: {token}")
            selected.append(package)
    return unique_plugins(selected)


def select_index(candidates: Sequence[PluginPackage], index: int) -> PluginPackage:
    if index < 1 or index > len(candidates):
        raise SystemExit(f"Plugin index out of range: {index}")
    return candidates[index - 1]


def select_range(candidates: Sequence[PluginPackage], token: str) -> list[PluginPackage]:
    start_raw, end_raw = token.split("-", 1)
    start = int(start_raw)
    end = int(end_raw)
    if start > end:
        start, end = end, start
    return [select_index(candidates, index) for index in range(start, end + 1)]


def unique_plugins(packages: Iterable[PluginPackage]) -> list[PluginPackage]:
    seen: set[str] = set()
    result: list[PluginPackage] = []
    for package in packages:
        plugin_id = package.plugin_id
        if plugin_id in seen:
            continue
        seen.add(plugin_id)
        result.append(package)
    return result


def print_plugin_catalog(packages: Sequence[PluginPackage]) -> None:
    print("Discovered plugins:")
    for index, package in enumerate(packages, start=1):
        carriers = ",".join(package.carriers) or "manifest_only"
        crate_names = ",".join(crate.name for crate in package.crates) or "no matched crate"
        print(f"  {index:2d}) {package.plugin_id:32s} [{carriers}] {crate_names}")


def _parse_csv(raw: str) -> list[str]:
    return [part.strip().lower() for part in raw.split(",") if part.strip()]
