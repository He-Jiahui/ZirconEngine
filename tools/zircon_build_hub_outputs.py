"""Hub/Tauri executable and installer output staging."""

from __future__ import annotations

import os
import shutil
from pathlib import Path

try:
    from .zircon_build_cargo_environment import assert_managed_windows_build_root
except ImportError:  # pragma: no cover - direct script import path.
    from zircon_build_cargo_environment import assert_managed_windows_build_root


HUB_TAURI_BUNDLE_TARGET = "nsis"
HUB_INSTALLERS_DIR_NAME = "installers"


def stage_hub_tauri_outputs(config: object, target_dir: Path) -> None:
    if not config.dry_run:
        assert_managed_windows_build_root(config.engine_root)
    bundle_root = target_dir / config.profile_dir / "bundle" / HUB_TAURI_BUNDLE_TARGET
    installers_dir = config.engine_root / HUB_INSTALLERS_DIR_NAME
    if config.dry_run:
        print(
            "DRY-RUN copy "
            f"{target_dir / config.profile_dir / _platform_executable_name('zircon_hub')} "
            f"-> {config.engine_root / _platform_executable_name('zircon_hub')}"
        )
        print(f"DRY-RUN reset {installers_dir}")
        print(f"DRY-RUN copytree {bundle_root} -> {installers_dir}")
        return

    _copy_artifact(config, target_dir, _platform_executable_name("zircon_hub"))
    stage_hub_tauri_installers(bundle_root, installers_dir, config)


def stage_hub_tauri_installers(
    bundle_root: Path, installers_dir: Path, config: object
) -> None:
    if not config.dry_run:
        assert_managed_windows_build_root(installers_dir)
    if not bundle_root.exists() or not bundle_root.is_dir():
        raise SystemExit(f"Tauri bundle output is missing: {bundle_root}")

    if installers_dir.exists():
        shutil.rmtree(installers_dir)
    installers_dir.mkdir(parents=True, exist_ok=True)

    bundle_directories: list[Path] = []
    bundle_files: list[Path] = []
    for directory, subdirectories, file_names in os.walk(bundle_root):
        directory_path = Path(directory)
        bundle_directories.extend(
            directory_path / name for name in subdirectories
        )
        bundle_files.extend(directory_path / name for name in file_names)

    for source in sorted(bundle_directories):
        relative = source.relative_to(bundle_root)
        destination = installers_dir / relative
        destination.mkdir(parents=True, exist_ok=True)
    for source in sorted(bundle_files):
        relative = source.relative_to(bundle_root)
        destination = installers_dir / relative
        _copy_file(source, destination, config)

    if not bundle_files:
        raise SystemExit(f"Tauri bundle output has no files: {bundle_root}")


def _copy_artifact(config: object, target_dir: Path, artifact_name: str) -> None:
    artifact = _find_artifact(target_dir, config.profile_dir, artifact_name)
    _copy_file(artifact, config.engine_root / artifact.name, config)
    _copy_sidecars(artifact, config.engine_root, config)


def _find_artifact(target_dir: Path, profile_dir: str, artifact_name: str) -> Path:
    profile_root = target_dir / profile_dir
    for candidate in (
        profile_root / artifact_name,
        profile_root / "deps" / artifact_name,
    ):
        if candidate.exists() and candidate.is_file():
            return candidate
    if profile_root.exists():
        for candidate in profile_root.rglob(artifact_name):
            if candidate.exists() and candidate.is_file():
                return candidate
    raise SystemExit(f"Built artifact not found under {profile_root}: {artifact_name}")


def _copy_file(source: Path, destination: Path, config: object) -> None:
    if config.dry_run:
        print(f"DRY-RUN copy {source} -> {destination}")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    print(f"Copied {source} -> {destination}")


def _copy_sidecars(source: Path, destination_dir: Path, config: object) -> None:
    sidecars = [
        source.with_suffix(".pdb"),
        source.with_suffix(".dbg"),
        Path(str(source) + ".dSYM"),
    ]
    for sidecar in sidecars:
        if not sidecar.exists():
            continue
        destination = destination_dir / sidecar.name
        if sidecar.is_dir():
            if config.dry_run:
                print(f"DRY-RUN copytree {sidecar} -> {destination}")
            else:
                if destination.exists():
                    shutil.rmtree(destination)
                shutil.copytree(sidecar, destination)
                print(f"Copied {sidecar} -> {destination}")
        else:
            _copy_file(sidecar, destination, config)


def _platform_executable_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem
