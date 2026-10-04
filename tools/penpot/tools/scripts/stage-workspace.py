"""Copy versioned Penpot inputs into a physical zircon-local build directory."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
from pathlib import Path

EXCLUDED = {".git", "node_modules", "dist", ".angular", ".vite", "coverage", "__pycache__"}


def approved_directory(value: str) -> Path:
    path = Path(value).absolute()
    roots = [Path(f"{drive}:/cargo-targets/zircon-local") for drive in "DEF"]
    root = next((root for root in roots if path.is_relative_to(root)), None)
    if root is None or not root.is_dir():
        raise ValueError("Output must be below D/E/F:/cargo-targets/zircon-local")
    for parent in [path, *path.parents]:
        if parent.is_symlink() or getattr(parent, "is_junction", lambda: False)():
            raise ValueError(f"Aliased output path: {parent}")
    if path.resolve() != path:
        raise ValueError(f"Output has an unexpected physical path: {path}")
    return path


def copy_inputs(source: Path, target: Path) -> list[dict[str, str]]:
    records = []
    for directory, names, files in os.walk(source, followlinks=False):
        names[:] = [name for name in names if name not in EXCLUDED]
        for name in files:
            original = Path(directory) / name
            relative = original.relative_to(source)
            if original.suffix in {".log", ".tsbuildinfo"}:
                continue
            destination = target / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, destination)
            digest = hashlib.sha256(original.read_bytes()).hexdigest()
            if hashlib.sha256(destination.read_bytes()).hexdigest() != digest:
                raise RuntimeError(f"Input changed during staging: {original}")
            records.append({"path": relative.as_posix(), "sha256": digest})
    return records


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifact-root", required=True)
    parser.add_argument("--cache-source", type=Path)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[4]
    artifact = approved_directory(args.artifact_root)
    if artifact.exists():
        raise FileExistsError(f"Use a fresh validation directory: {artifact}")
    upstream = repository / "third_party/penpot"
    json.loads((upstream / "UPSTREAM.json").read_text(encoding="utf-8"))
    artifact.mkdir(parents=True)
    workspace = artifact / "workspace"
    inputs = {
        "tools/penpot": copy_inputs(repository / "tools/penpot", workspace / "tools/penpot"),
    }
    # Only these upstream modules and browser fixtures are runtime inputs here.
    # The complete pinned source archive remains versioned in third_party.
    for relative in (
        "plugins/libs",
        "plugins/apps/plugin-api-test-suite/ci/fixtures",
        "frontend/playwright/data",
        "frontend/playwright/scripts",
    ):
        source = upstream / relative
        inputs[f"third_party/penpot/{relative}"] = copy_inputs(
            source, workspace / "third_party/penpot" / relative
        )
    for name in ("LICENSE", "UPSTREAM.json", "ZIRCON_UPSTREAM.md"):
        source = upstream / name
        if source.is_file():
            target = workspace / "third_party/penpot" / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            inputs[f"third_party/penpot/{name}"] = [{"path": name, "sha256": hashlib.sha256(source.read_bytes()).hexdigest()}]
    # Tests also load source assets through module-relative URLs.
    for relative in (
        "zircon_editor/assets",
        "zircon_runtime/assets",
        "zircon_runtime/tests/fixtures",
        "examples/woc/assets",
    ):
        source = repository / relative
        if source.is_dir():
            inputs[relative] = copy_inputs(source, workspace / relative)
    if args.cache_source:
        # Read an existing cache, but never ask pnpm to write back to that path.
        shutil.copytree(args.cache_source, artifact / "pnpm-store", dirs_exist_ok=True)
    receipt = {
        "repository": str(repository),
        "artifactRoot": str(artifact),
        "workspace": str(workspace),
        "inputs": inputs,
    }
    (artifact / "inputs.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in receipt.items() if key != "inputs"}))


if __name__ == "__main__":
    main()
