from __future__ import annotations


def render_entry_static_dependencies_markdown(
    entry_static_dependencies: dict[str, object],
) -> list[str]:
    entry_deps = entry_static_dependencies["cargo_path_dependencies"]
    plugin_deps = entry_static_dependencies["optional_runtime_plugin_path_dependencies"]
    plugin_feature_mentions = entry_static_dependencies[
        "optional_runtime_plugin_feature_mentions"
    ]
    builtin_modules = entry_static_dependencies["builtin_module_crates"]

    lines = [
        "## Entry Static Dependencies",
        f"- `zircon_app/Cargo.toml` path dependencies ({len(entry_deps)}): {', '.join(entry_deps)}",
        "- optional runtime plugin path dependencies "
        f"({len(plugin_deps)}): {', '.join(plugin_deps) if plugin_deps else 'none'}",
        "- optional runtime plugin feature mentions "
        f"({len(plugin_feature_mentions)}): "
        f"{', '.join(plugin_feature_mentions) if plugin_feature_mentions else 'none'}",
    ]
    lines.extend(f"- risk: {risk}" for risk in entry_static_dependencies["risks"])
    lines.append(
        "- built-in entry/runtime module crates "
        f"({len(builtin_modules)}): {', '.join(builtin_modules)}"
    )
    return lines
