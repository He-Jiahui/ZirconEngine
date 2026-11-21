"""Generate a tiny real-Git Cargo A/B fixture, or check its real command receipts.

This driver never runs Cargo. Use the declared local-cargo commands in the plan.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


PACKAGE = "ze_local_cargo_source_probe"
FILTER = "source_generation_"
MANIFEST = f'[package]\nname = "{PACKAGE}"\nversion = "0.1.0"\nedition = "2021"\n'
LOCK = f'version = 4\n\n[[package]]\nname = "{PACKAGE}"\nversion = "0.1.0"\n'


def proof_path(text: str) -> Path:
    path = Path(text)
    if not path.is_absolute() or path.drive.upper() not in {"D:", "E:", "F:"} or ".." in path.parts:
        raise ValueError("choose an absolute path on the Windows D, E, or F drive")
    namespace = Path(path.anchor) / "cargo-targets" / "zircon-local" / "reports"
    if path.parent != namespace:
        raise ValueError("choose a new task-named directory directly under the physical reports root")
    for part in (path, *path.parents):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if part.is_symlink() or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError(f"probe paths must not be aliases: {part}")
    if path.resolve() != path:
        raise ValueError("probe path must resolve to itself")
    return path


def git(tree: Path, *args: str) -> str:
    env = {key: val for key, val in os.environ.items() if not key.upper().startswith("GIT_")}
    env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_TERMINAL_PROMPT="0")
    command = ["git", "-c", "user.name=Zircon Cargo probe", "-c", "user.email=probe@example.invalid", *args]
    if args[0] in {"add", "commit", "worktree"}:
        index = Path(git(tree, "rev-parse", "--absolute-git-dir")) / "index.lock"
        if index.exists():
            raise ValueError(f"preserving fixture index lock: {index}")
    return subprocess.run(command, cwd=tree, env=env, check=True, text=True, stdout=subprocess.PIPE).stdout.strip()


def witness(letter: str) -> str:
    return (
        "#[cfg(test)]\n"
        "#[test]\n"
        f"fn {FILTER}{letter}() {{\n"
        '    println!("compiled-source={}", env!("CARGO_MANIFEST_DIR"));\n'
        "}\n"
    )


def prepare(proof: Path) -> None:
    proof.mkdir(parents=True, exist_ok=False)
    a, b = proof / "source-a", proof / "source-b"
    a.mkdir()
    (a / "src").mkdir()
    (a / "Cargo.toml").write_text(MANIFEST, encoding="utf-8", newline="\n")
    (a / "Cargo.lock").write_text(LOCK, encoding="utf-8", newline="\n")
    (a / "src/lib.rs").write_text(witness("a"), encoding="utf-8", newline="\n")
    git(a, "init", "--template=", "--initial-branch=source-a")
    git(a, "add", "--", "Cargo.toml", "Cargo.lock", "src/lib.rs")
    git(a, "commit", "--quiet", "-m", "Fixture: compile witness A")
    git(a, "worktree", "add", "--quiet", "-b", "source-b", str(b), "HEAD")
    (b / "src/lib.rs").write_text(witness("b"), encoding="utf-8", newline="\n")
    git(b, "add", "--", "src/lib.rs")
    git(b, "commit", "--quiet", "-m", "Fixture: compile witness B")
    print(json.dumps({"a": str(a), "b": str(b), "aHead": git(a, "rev-parse", "HEAD"),
                      "bHead": git(b, "rev-parse", "HEAD")}, ensure_ascii=False))


def observe(proof: Path, scenario: str, requested: str) -> tuple[tuple[str, Path], Path, Path]:
    directory = proof / scenario
    record = json.loads((directory / "receipt.json").read_bytes())
    if (record.get("schema") != "zircon_agent_local_cargo_command_v1"
            or record.get("commandExecuted") is not True or record.get("exitCode") != 0
            or Path(record["cwd"]) != proof / f"source-{requested}"
            or record["owner"].get("package") != PACKAGE or record["owner"].get("testFilter") != FILTER):
        raise ValueError(f"{scenario}: not the declared successful Cargo command")
    raw = (directory / "stdout.bin").read_bytes()
    if hashlib.sha256(raw).hexdigest() != record["stdout"]["sha256"]:
        raise ValueError(f"{scenario}: stdout hash does not match receipt")
    stdout = raw.decode("utf-8")
    names = re.findall(r"^test source_generation_([ab]) \.\.\.", stdout, re.MULTILINE)
    paths = re.findall(r"compiled-source=([^\r\n]+)", stdout)
    if len(names) != 1 or len(paths) != 1 or "test result: ok. 1 passed; 0 failed; 0 ignored;" not in stdout:
        raise ValueError(f"{scenario}: expected exactly one executed source witness")
    found = (names[0], Path(paths[0]))
    print(f"{scenario}: requested={record['cwd']} executed={FILTER}{found[0]} compiled={found[1]}")
    return found, Path(record["targetDirectory"]), Path(record["buildDirectory"])


def verify(proof: Path) -> int:
    a, a_target, a_build = observe(proof, "a-shared", "a")
    shared_b, b_target, b_build = observe(proof, "b-shared", "b")
    isolated_b, control_target, control_build = observe(proof, "b-isolated", "b")
    if (a_target != b_target or a_build != b_build
            or a_target == control_target or a_build == control_build):
        raise ValueError("A/B must share one pool, with a physically separate B control pool")
    if a != ("a", proof / "source-a") or isolated_b != ("b", proof / "source-b"):
        raise ValueError("the source A or isolated source B control did not establish its identity")
    if shared_b != isolated_b:
        print("FAIL: the shared-pool test did not execute the requested source B", file=sys.stderr)
        return 1
    print("PASS: source B agrees with the isolated control")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "verify"))
    parser.add_argument("proof_directory")
    args = parser.parse_args()
    proof = proof_path(args.proof_directory)
    if args.action == "prepare":
        prepare(proof)
        return 0
    return verify(proof)


if __name__ == "__main__":
    raise SystemExit(main())
