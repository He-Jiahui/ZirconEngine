"""Check resolved portable-core and native text dependency boundaries."""
from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NATIVE_PACKAGES = {
    "libloading", "notify", "notify-types", "meshopt", "zstd", "zstd-sys",
    "zstd-safe", "ring", "goblin", "windows-sys", "joltc-sys",
}
FONT_FAMILIES = {"glam", "ttf-parser", "read-fonts", "skrifa", "swash"}
RETIRED_PACKAGES = {"bincode", "serde_yaml"}


def package_versions(output: str) -> dict[str, set[str]]:
    packages: dict[str, set[str]] = {}
    for line in output.splitlines():
        match = re.match(r"^([A-Za-z0-9_-]+) v([^\s]+)", line)
        if match:
            packages.setdefault(match[1], set()).add(match[2])
    return packages


def tree(
    features: str | None,
    target: str | None = None,
    *,
    manifest: Path = ROOT / "Cargo.toml",
    package: str = "zircon_runtime",
) -> dict[str, set[str]]:
    command = [
        "cargo", "tree", "--manifest-path", str(manifest), "--locked", "-p",
        package, "--no-default-features", "-e", "normal", "--prefix", "none",
    ]
    if features:
        command.extend(["--features", features])
    if target:
        command.extend(["--target", target])
    result = subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True)
    packages = package_versions(result.stdout)
    if package not in packages:
        raise RuntimeError("Cargo did not return the requested runtime graph")
    retired = sorted(RETIRED_PACKAGES.intersection(packages))
    if retired:
        raise RuntimeError(f"{package} includes retired dependencies: {retired}")
    return packages


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--text", action="store_true", help="also check the native text graph")
    args = parser.parse_args()
    portable = tree("core-min", "wasm32-unknown-unknown")
    native = sorted(NATIVE_PACKAGES.intersection(portable))
    if native:
        raise RuntimeError(f"portable core includes native packages: {', '.join(native)}")
    print(f"portable core: {len(portable)} package names, no native host dependencies")
    if args.text:
        plugin_manifest = ROOT / "zircon_plugins" / "Cargo.toml"
        graphs = {
            "runtime": tree("native-host,text"),
            "plugins": tree(
                None, manifest=plugin_manifest,
                package="zircon_plugin_ui_document_importer_runtime",
            ),
        }
        for label, fonts in graphs.items():
            duplicates = {
                name: sorted(fonts.get(name, set()))
                for name in sorted(FONT_FAMILIES)
                if len(fonts.get(name, set())) != 1
            }
            if duplicates:
                raise RuntimeError(f"{label} text dependencies diverged: {duplicates}")
            print(f"{label}: text dependency families each resolve to one version")
        tree(None, manifest=plugin_manifest, package="zircon_plugin_asset_importer_data_runtime")
        print("runtime and data importer graphs exclude retired codecs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
