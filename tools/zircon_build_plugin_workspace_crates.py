"""Plugin workspace crate discovery for zircon_build."""

from __future__ import annotations

from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - exercised only on old Python.
    print("Python 3.11 or newer is required because this tool uses tomllib.")
    raise

try:
    from .zircon_build_plugin_packages import CargoPackage
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_plugin_packages import CargoPackage


def discover_plugin_workspace_crates(plugins_root: Path) -> tuple[CargoPackage, ...]:
    workspace = _read_toml(plugins_root / "Cargo.toml")
    members = workspace.get("workspace", {}).get("members", [])
    packages: list[CargoPackage] = []
    for member in members:
        manifest_path = plugins_root / member / "Cargo.toml"
        if not manifest_path.exists():
            continue
        data = _read_toml(manifest_path)
        package = data.get("package", {})
        name = package.get("name")
        if not name:
            continue
        crate_types = data.get("lib", {}).get("crate-type", [])
        packages.append(
            CargoPackage(
                name=str(name),
                member=str(member).replace("\\", "/"),
                manifest_path=manifest_path,
                crate_types=tuple(str(crate_type) for crate_type in crate_types),
            )
        )
    return tuple(packages)


def _read_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)
