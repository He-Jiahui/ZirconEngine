"""Hub/Tauri build and installer staging for zircon_build."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Sequence

try:
    from .zircon_build_cargo_environment import managed_cargo_environment
    from .zircon_build_hub_outputs import (
        HUB_TAURI_BUNDLE_TARGET,
        stage_hub_tauri_installers,
        stage_hub_tauri_outputs,
    )
except ImportError:  # pragma: no cover - direct script import path.
    from zircon_build_cargo_environment import managed_cargo_environment
    from zircon_build_hub_outputs import (
        HUB_TAURI_BUNDLE_TARGET,
        stage_hub_tauri_installers,
        stage_hub_tauri_outputs,
    )


def build_hub(config: object) -> None:
    if config.mode == "profiling":
        raise SystemExit("--mode profiling is not supported for the hub/Tauri target.")
    target_dir = config.targets_root / "hub"
    run_tauri_build(config, target_dir)
    stage_hub_tauri_outputs(config, target_dir)


def tauri_cli_path(config: object) -> Path:
    cli_path = (
        config.repo_root
        / "zircon_hub"
        / "node_modules"
        / "@tauri-apps"
        / "cli"
        / "tauri.js"
    )
    if not cli_path.exists():
        raise SystemExit(
            "Tauri CLI is missing. Run npm install in zircon_hub before "
            f"building the Hub bundle: {cli_path}"
        )
    return cli_path


def run_tauri_build(config: object, target_dir: Path) -> None:
    command = [
        "node",
        str(tauri_cli_path(config)),
        "build",
        "--runner",
        config.cargo,
        "--bundles",
        HUB_TAURI_BUNDLE_TARGET,
        "--ci",
        "--no-sign",
    ]
    if config.mode == "debug":
        command.append("--debug")

    runner_args: list[str] = []
    if config.locked:
        runner_args.append("--locked")
    if config.jobs:
        runner_args.extend(["--jobs", config.jobs])
    if runner_args:
        command.append("--")
        command.extend(runner_args)

    if config.dry_run:
        print("DRY-RUN", f"CARGO_TARGET_DIR={target_dir}", _quote_command(command))
        return
    env = hub_cargo_environment(target_dir)
    print(f"CARGO_TARGET_DIR={target_dir} {_quote_command(command)}")
    subprocess.run(command, cwd=config.repo_root / "zircon_hub", check=True, env=env)


def hub_cargo_environment(target_dir: Path) -> dict[str, str]:
    environment = managed_cargo_environment(target_dir, target_dir)
    # npm otherwise defaults to LocalAppData on Windows before Tauri invokes Cargo.
    npm_cache = target_dir.resolve() / "npm-cache"
    npm_cache.mkdir(parents=True, exist_ok=True)
    environment["npm_config_cache"] = str(npm_cache)
    return environment


def _quote_command(command: Sequence[str]) -> str:
    return " ".join(_quote_arg(part) for part in command)


def _quote_arg(value: str) -> str:
    if not value or any(ch.isspace() for ch in value):
        return '"' + value.replace('"', '\\"') + '"'
    return value
