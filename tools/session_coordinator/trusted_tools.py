from __future__ import annotations

import hashlib
import json
import os
import subprocess
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Mapping

from .models import CoordinatorError


@lru_cache(maxsize=32)
def _resolve_cached(name: str, repo_root_text: str, path_value: str) -> str:
    repo_root = Path(repo_root_text).resolve()
    suffixes = (".exe",) if os.name == "nt" else ("",)
    for raw_directory in path_value.split(os.pathsep):
        if not raw_directory.strip():
            continue
        directory = Path(raw_directory).expanduser()
        if not directory.is_absolute():
            continue
        for suffix in suffixes:
            candidate = directory / f"{name}{suffix}"
            try:
                resolved = candidate.resolve(strict=True)
            except OSError:
                continue
            if (
                not resolved.is_file()
                or candidate.is_symlink()
                or resolved.is_relative_to(repo_root)
            ):
                continue
            return str(resolved)
    raise CoordinatorError(
        "trusted_tool_unavailable",
        f"Coordinator could not resolve trusted {name} outside the repository",
        details={"tool": name},
    )


def trusted_executable(name: str, repo_root: str | Path) -> str:
    resolved_repo = Path(repo_root).resolve()
    executable = Path(
        _resolve_cached(name.casefold(), str(resolved_repo), os.environ.get("PATH", ""))
    )
    try:
        refreshed = executable.resolve(strict=True)
    except OSError:
        _resolve_cached.cache_clear()
        return trusted_executable(name, resolved_repo)
    if (
        not refreshed.is_file()
        or executable.is_symlink()
        or refreshed.is_relative_to(resolved_repo)
    ):
        _resolve_cached.cache_clear()
        raise CoordinatorError(
            "trusted_tool_unavailable",
            f"Coordinator trusted {name} path changed after resolution",
            details={"tool": name},
        )
    return str(refreshed)


@dataclass(frozen=True, slots=True)
class _RustToolBinding:
    rustup: str
    cargo_launcher: str
    rustc_launcher: str
    cargo_binary: str
    rustc_binary: str
    active_toolchain: str


def _run_rustup(
    rustup: str,
    arguments: tuple[str, ...],
    *,
    working_directory: Path,
    environment: Mapping[str, str],
) -> str:
    try:
        result = subprocess.run(
            [rustup, *arguments],
            cwd=working_directory,
            env=dict(environment),
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CoordinatorError(
            "trusted_toolchain_unavailable",
            "Coordinator could not resolve the managed Rustup toolchain",
            details={"operation": "rustup " + " ".join(arguments)},
        ) from error
    if result.returncode != 0 or not result.stdout.strip():
        raise CoordinatorError(
            "trusted_toolchain_unavailable",
            "Rustup could not resolve the managed toolchain",
            details={
                "operation": "rustup " + " ".join(arguments),
                "exitCode": int(result.returncode),
            },
        )
    return result.stdout.strip()


def _validated_tool_path(
    value: str | Path, repo_root: Path, *, tool: str
) -> str:
    candidate = Path(value)
    try:
        resolved = candidate.resolve(strict=True)
    except OSError as error:
        raise CoordinatorError(
            "trusted_toolchain_unavailable",
            f"Rustup returned an unavailable {tool} executable",
            details={"tool": tool},
        ) from error
    if (
        not resolved.is_file()
        or candidate.is_symlink()
        or resolved.is_relative_to(repo_root)
    ):
        raise CoordinatorError(
            "trusted_toolchain_unavailable",
            f"Rustup returned an untrusted {tool} executable",
            details={"tool": tool},
        )
    return str(resolved)


def _rust_tool_binding(
    command: tuple[str, ...],
    repo_root: str | Path,
    working_directory: str | Path | None,
) -> _RustToolBinding:
    resolved_repo = Path(repo_root).resolve()
    query_root = Path(working_directory or resolved_repo).resolve()
    rustup = trusted_executable("rustup", resolved_repo)
    suffix = ".exe" if os.name == "nt" else ""
    proxy_directory = Path(rustup).parent
    cargo_launcher = _validated_tool_path(
        proxy_directory / f"cargo{suffix}", resolved_repo, tool="cargo proxy"
    )
    rustc_launcher = _validated_tool_path(
        proxy_directory / f"rustc{suffix}", resolved_repo, tool="rustc proxy"
    )
    environment = dict(os.environ)
    for name in tuple(environment):
        if name.casefold() == "rustup_toolchain":
            del environment[name]
    selector = next((part[1:] for part in command[1:2] if part.startswith("+")), None)
    if selector:
        environment["RUSTUP_TOOLCHAIN"] = selector
    active_output = _run_rustup(
        rustup,
        ("show", "active-toolchain"),
        working_directory=query_root,
        environment=environment,
    )
    active_toolchain = active_output.split()[0]
    if not active_toolchain:
        raise CoordinatorError(
            "trusted_toolchain_unavailable",
            "Rustup returned an empty active toolchain identity",
        )
    environment["RUSTUP_TOOLCHAIN"] = active_toolchain
    cargo_binary = _validated_tool_path(
        _run_rustup(
            rustup,
            ("which", "cargo"),
            working_directory=query_root,
            environment=environment,
        ),
        resolved_repo,
        tool="cargo",
    )
    rustc_binary = _validated_tool_path(
        _run_rustup(
            rustup,
            ("which", "rustc"),
            working_directory=query_root,
            environment=environment,
        ),
        resolved_repo,
        tool="rustc",
    )
    return _RustToolBinding(
        rustup=rustup,
        cargo_launcher=cargo_launcher,
        rustc_launcher=rustc_launcher,
        cargo_binary=cargo_binary,
        rustc_binary=rustc_binary,
        active_toolchain=active_toolchain,
    )


def bind_trusted_cargo(
    command: tuple[str, ...],
    repo_root: str | Path,
    *,
    working_directory: str | Path | None = None,
) -> tuple[str, ...]:
    if not command or command[0].casefold() not in {"cargo", "cargo.exe"}:
        return command
    binding = _rust_tool_binding(command, repo_root, working_directory)
    return (binding.cargo_launcher, *command[1:])


def bind_trusted_rust_environment(
    environment: Mapping[str, str],
    command: tuple[str, ...],
    repo_root: str | Path,
    *,
    working_directory: str | Path | None = None,
) -> dict[str, str]:
    """Bind Cargo's compiler and remove checkout-local PATH entries."""
    resolved_repo = Path(repo_root).resolve()
    binding = _rust_tool_binding(command, resolved_repo, working_directory)
    cargo = Path(binding.cargo_launcher)
    rustc = Path(binding.rustc_launcher)
    result = {
        name: value
        for name, value in environment.items()
        if name.casefold() != "path"
    }
    directories: list[str] = []
    seen: set[str] = set()

    def include(directory: Path) -> None:
        try:
            resolved = directory.expanduser().resolve(strict=True)
        except OSError:
            return
        if not resolved.is_dir() or resolved.is_relative_to(resolved_repo):
            return
        key = str(resolved).casefold() if os.name == "nt" else str(resolved)
        if key in seen:
            return
        seen.add(key)
        directories.append(str(resolved))

    include(cargo.parent)
    include(rustc.parent)
    inherited_path = next(
        (value for name, value in environment.items() if name.casefold() == "path"),
        "",
    )
    for raw_directory in inherited_path.split(os.pathsep):
        directory = Path(raw_directory.strip())
        if directory.is_absolute():
            include(directory)
    result["PATH"] = os.pathsep.join(directories)
    result["RUSTC"] = str(rustc)
    result["RUSTUP_TOOLCHAIN"] = binding.active_toolchain
    return result


def trusted_git_command(repo_root: str | Path, *arguments: str) -> list[str]:
    return [trusted_executable("git", repo_root), *arguments]


@lru_cache(maxsize=16)
def _tool_identity_cached(
    executable: str,
    arguments: tuple[str, ...],
    file_identity: tuple[int, int, int, int, int],
    working_directory: str,
    workspace_toolchain_identity: str,
) -> Mapping[str, object]:
    path = Path(executable)
    try:
        content_hash = hashlib.sha256(path.read_bytes()).hexdigest()
        result = subprocess.run(
            [executable, *arguments],
            cwd=working_directory,
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as error:
        raise CoordinatorError(
            "trusted_tool_identity_unavailable",
            "Coordinator could not establish the managed Rust tool identity",
            details={"executable": executable},
        ) from error
    if result.returncode != 0:
        raise CoordinatorError(
            "trusted_tool_identity_unavailable",
            "Managed Rust tool version query failed",
            details={"executable": executable, "exitCode": result.returncode},
        )
    version = result.stdout.strip()
    return {
        "path": str(path.resolve()),
        "sha256": content_hash,
        "versionSha256": hashlib.sha256(version.encode("utf-8")).hexdigest(),
        "version": version[:2048],
    }


def _tool_identity(
    executable: str,
    arguments: tuple[str, ...],
    working_directory: Path,
) -> Mapping[str, object]:
    path = Path(executable)
    try:
        current = path.stat()
    except OSError as error:
        raise CoordinatorError(
            "trusted_tool_identity_unavailable",
            "Coordinator could not inspect the managed Rust tool identity",
            details={"executable": executable},
        ) from error
    return _tool_identity_cached(
        executable,
        arguments,
        (
            int(current.st_dev),
            int(current.st_ino),
            int(current.st_size),
            int(current.st_mtime_ns),
            int(current.st_ctime_ns),
        ),
        str(working_directory.resolve()),
        _workspace_toolchain_identity(working_directory),
    )


def _workspace_toolchain_identity(working_directory: Path) -> str:
    current = working_directory.resolve()
    while True:
        for name in ("rust-toolchain.toml", "rust-toolchain"):
            candidate = current / name
            try:
                content = candidate.read_bytes()
            except FileNotFoundError:
                continue
            except OSError as error:
                raise CoordinatorError(
                    "trusted_tool_identity_unavailable",
                    "Coordinator could not read the workspace Rust toolchain selector",
                    details={"path": str(candidate)},
                ) from error
            return json.dumps(
                {
                    "path": candidate.relative_to(current).as_posix(),
                    "sha256": hashlib.sha256(content).hexdigest(),
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        if current == current.parent:
            return "workspace-default"
        current = current.parent


def trusted_rust_toolchain_identity(
    command: tuple[str, ...],
    repo_root: str | Path,
    *,
    working_directory: str | Path | None = None,
) -> str:
    query_root = Path(working_directory or repo_root).resolve()
    binding = _rust_tool_binding(command, repo_root, query_root)
    payload = {
        "rustup": _tool_identity(binding.rustup, ("-V",), query_root),
        "cargo": _tool_identity(binding.cargo_binary, ("-vV",), query_root),
        "rustc": _tool_identity(binding.rustc_binary, ("-vV",), query_root),
        "selector": next(
            (part for part in command[1:2] if part.startswith("+")),
            "workspace-default",
        ),
        "activeToolchain": binding.active_toolchain,
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":"))


def trusted_file_identity(executable: str | Path) -> Mapping[str, object]:
    path = Path(executable).resolve(strict=True)
    current = path.stat()
    return {
        "path": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "size": int(current.st_size),
        "modifiedNs": int(current.st_mtime_ns),
    }
