#!/usr/bin/env python3
"""Stage an explicitly built Linux editor using the canonical asset/ABI tools.

This never compiles, edits Cargo freshness, or replaces an existing evidence
directory. The caller must build both products with compatible source/features.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
from types import SimpleNamespace


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--staging", type=Path, required=True)
    parser.add_argument("--profile", choices=("debug", "release", "profiling"), default="debug")
    parser.add_argument("--runtime-features", default="target-editor-host")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    sys.path.insert(0, str(repo))
    from tools.build.zircon_build_asset_staging import stage_engine_assets
    from tools.build.zircon_build_runtime_manifest import write_runtime_artifact_manifest

    products = args.target_dir.resolve() / args.profile
    stage = args.staging.resolve()
    names = ("zircon_editor", "libzircon_runtime.so")
    for name in names:
        if not (products / name).is_file():
            parser.error(f"missing built product: {products / name}")
    stage.mkdir(parents=True, exist_ok=False)
    records = []
    for name in names:
        source = products / name
        destination = stage / name
        shutil.copy2(source, destination)
        with destination.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        records.append({"source": str(source), "staged": str(destination),
                        "bytes": destination.stat().st_size, "sha256": digest})
    config = SimpleNamespace(repo_root=repo, engine_root=stage, dry_run=False,
                             mode="debug" if args.profile == "debug" else args.profile,
                             runtime_features=args.runtime_features.split(","))
    stage_engine_assets(config)
    manifest = write_runtime_artifact_manifest(config)
    receipt = {"scope": "local Linux staged editor, not Jenkins acceptance",
               "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
               "dirty": subprocess.check_output(["git", "status", "--porcelain=v1"], cwd=repo, text=True),
               "products": records, "runtime_manifest": str(manifest)}
    (stage / "stage-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
