from __future__ import annotations

import re
from pathlib import Path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _parse_entry_path_dependencies(root: Path) -> list[str]:
    manifest = _read_text(root / "zircon_app" / "Cargo.toml")
    dependencies_section = re.search(
        r"(?ms)^\[dependencies\]\s*(.*?)^(?:\[|$)",
        manifest,
    )
    if not dependencies_section:
        return []
    deps = re.findall(
        r"(?m)^(zircon_[a-z0-9_]+)\s*=\s*\{\s*path\s*=\s*\"[^\"]+\"",
        dependencies_section.group(1),
    )
    return sorted(deps)


def _builtin_entry_modules(root: Path) -> list[str]:
    search_roots = [
        root / "zircon_app" / "src" / "entry",
        root / "zircon_runtime" / "src",
    ]
    modules: set[str] = set()
    for search_root in search_roots:
        if not search_root.exists():
            continue
        for path in search_root.rglob("*.rs"):
            source = _read_text(path)
            modules.update(re.findall(r"Arc::new\((zircon_[a-z0-9_]+)::", source))
    return sorted(modules)


def entry_static_dependencies_audit(root: Path) -> dict[str, object]:
    manifest = _read_text(root / "zircon_app" / "Cargo.toml")
    path_dependencies = _parse_entry_path_dependencies(root)
    plugin_path_dependencies = sorted(
        dependency
        for dependency in path_dependencies
        if dependency.startswith("zircon_plugin_")
    )
    plugin_feature_mentions = sorted(
        set(re.findall(r'"dep:(zircon_plugin_[a-z0-9_]+)"', manifest))
    )

    risks: list[str] = []
    if plugin_path_dependencies:
        risks.append(
            "zircon_app has direct optional first-party runtime plugin path dependencies; "
            "M2 should move plugin implementation fan-out behind a runtime-owned catalog."
        )
    if plugin_feature_mentions:
        risks.append(
            "zircon_app feature flags still name first-party runtime plugin crates directly."
        )

    return {
        "cargo_path_dependencies": path_dependencies,
        "optional_runtime_plugin_path_dependencies": plugin_path_dependencies,
        "optional_runtime_plugin_feature_mentions": plugin_feature_mentions,
        "optional_runtime_plugin_path_dependency_count": len(plugin_path_dependencies),
        "optional_runtime_plugin_feature_mention_count": len(plugin_feature_mentions),
        "builtin_module_crates": _builtin_entry_modules(root),
        "risks": risks,
    }
