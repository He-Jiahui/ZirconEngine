"""Run local Cargo with physical output checks and no coordinator dependency."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

from tools.jenkins.contracts import JenkinsError


MIN_FREE_BYTES = 35 * 1024**3
ARTIFACT_COMMANDS = {"build", "check", "test", "run", "bench", "clippy", "doc", "rustc"}


def storage_roots() -> tuple[Path, ...]:
    if os.name == "nt":
        return tuple(Path(f"{drive}:\\cargo-targets") for drive in "DEF")
    return tuple(Path(f"/mnt/{drive}/cargo-targets") for drive in "def")


def validate_output_path(value: str | Path) -> Path:
    requested = Path(value)
    if not requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Output must be an absolute physical path below D/E/F:\\cargo-targets")
    physical = requested.resolve()
    if physical != requested:
        raise ValueError(f"Output path aliases are not allowed: {requested}")
    for root in storage_roots():
        if root.resolve() == root and physical != root and physical.is_relative_to(root):
            return physical
    raise ValueError(f"Output must physically be below D/E/F drive-root cargo-targets: {requested}")


def free_bytes(path: Path) -> int:
    probe = path
    while not probe.exists() and probe != probe.parent:
        probe = probe.parent
    return shutil.disk_usage(probe).free


def prepare_command(arguments: list[str], target: Path) -> list[str]:
    args = list(arguments)
    separator = args.index("--") if "--" in args else len(args)
    options = args[:separator]
    if any(arg in {"--target-dir", "--config", "--artifact-dir", "--out-dir"} or arg.startswith(("--target-dir=", "--config=", "--artifact-dir=", "--out-dir=")) for arg in options):
        raise ValueError("Pass --target-dir to the local wrapper; Cargo --config output overrides are not accepted")
    command_index = 1 if options and options[0].startswith("+") else 0
    if len(options) <= command_index or options[command_index] not in ARTIFACT_COMMANDS:
        raise ValueError("Select build/check/test/run/bench/clippy/doc/rustc; use ordinary Cargo for read-only commands")
    if options[command_index] == "rustc" and separator < len(args):
        raise ValueError("Raw rustc flags are not admitted by this bounded wrapper")
    generated = ["--target-dir", str(validate_output_path(target))]
    if "--locked" not in options:
        generated.append("--locked")
    return ["cargo", *options, *generated, *args[separator:]]


def _default_target(repo: Path, args: list[str], *, dry_run: bool) -> Path:
    # This namespace never reuses the retired coordinator's pools or job scratch.
    root = storage_roots()[0] if dry_run else max(
        (root for root in storage_roots() if root.parent.exists()), key=free_bytes
    )
    toolchain = repo / "rust-toolchain.toml"
    identity = json.dumps({"repo": str(repo).casefold(), "platform": sys.platform,
                           "toolchain": toolchain.read_text(encoding="utf-8") if toolchain.exists() else "default",
                           "cargoArgs": args}, sort_keys=True).encode("utf-8")
    key = hashlib.sha256(identity).hexdigest()[:20]
    return root / "zircon-local" / sys.platform / key / "target"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=str(Path(__file__).resolve().parents[2]))
    parser.add_argument("--target-dir")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--jenkins-identity-file")
    parser.add_argument("--independent-preview", action="store_true")
    parser.add_argument("cargo_args", nargs=argparse.REMAINDER)
    options = parser.parse_args(argv)
    args = options.cargo_args[1:] if options.cargo_args[:1] == ["--"] else options.cargo_args
    try:
        repo = Path(options.repo_root).resolve()
        if options.jenkins_identity_file:
            os.environ["ZIRCON_JENKINS_IDENTITY_FILE"] = str(Path(options.jenkins_identity_file).resolve())
        # M8 owns all known build/test entry points once activated.  The
        # identity file is supplied by the caller/session; without it we fail
        # before Cargo can start.  Prior to M8 this returns None and preserves
        # the independent local evidence path below.
        from tools.jenkins.frontend import entry_from_environment, sole_entry_enforced
        gate_repo = Path(__file__).resolve().parents[2]
        if options.independent_preview and sole_entry_enforced(gate_repo):
            raise ValueError("Jenkins sole-entry gate active; independent preview refuses service dispatch")
        forwarded = None if options.independent_preview else entry_from_environment(
            gate_repo, job="zircon-flow", parameters={"cargoVerb": args[0] if args else ""})
        if forwarded is not None:
            print(json.dumps(forwarded, ensure_ascii=False, sort_keys=True))
            return 0
        target = validate_output_path(options.target_dir or _default_target(repo, args, dry_run=options.dry_run))
        command = prepare_command(args, target)
        if options.dry_run:
            print(json.dumps({"status": "dry_run", "command": command, "formalAcceptance": False}, ensure_ascii=False))
            return 0
        if free_bytes(target) < MIN_FREE_BYTES:
            raise ValueError("The target drive must have at least 35 GiB free before Cargo starts")
        root = next(root for root in storage_roots() if target.is_relative_to(root))
        cache = validate_output_path(root / "zircon-local/cache")
        scratch = validate_output_path(root / "zircon-local/scratch" / uuid.uuid4().hex)
        directories = [target, cache / "cargo-home", cache / "sccache", scratch / "temporary", target.parent / "build"]
        for directory in directories:
            validate_output_path(directory).mkdir(parents=True, exist_ok=True)
            validate_output_path(directory)
        environment = dict(os.environ)
        environment.update({
            "CARGO_TARGET_DIR": str(target), "CARGO_HOME": str(cache / "cargo-home"),
            "CARGO_BUILD_BUILD_DIR": str(target.parent / "build"),
            "SCCACHE_DIR": str(cache / "sccache"),
            "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
            "TEMP": str(scratch / "temporary"), "TMP": str(scratch / "temporary"),
            "TMPDIR": str(scratch / "temporary"),
        })
        # Cargo retains its native target locks; no shared index or old pool is changed.
        return subprocess.run(command, cwd=repo, env=environment, check=False).returncode
    except (OSError, ValueError, JenkinsError) as error:
        print(f"Local Cargo preflight failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
