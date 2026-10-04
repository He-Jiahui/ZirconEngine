"""Small, dependency-free helpers shared by the build entrypoint.

The build runner owns orchestration and policy.  Platform naming, selection
normalisation, and command rendering live here so the entrypoint stays a
reviewable coordinator rather than becoming another utility grab bag.
"""

from __future__ import annotations

import os
import platform
from typing import Iterable, Sequence


def platform_executable_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def platform_runtime_library_name() -> str:
    if os.name == "nt":
        return "zircon_runtime.dll"
    if platform.system().lower() == "darwin":
        return "libzircon_runtime.dylib"
    return "libzircon_runtime.so"


def platform_dynamic_library_name(crate_name: str) -> str:
    if os.name == "nt":
        return f"{crate_name}.dll"
    if platform.system().lower() == "darwin":
        return f"lib{crate_name}.dylib"
    return f"lib{crate_name}.so"


def sanitize_path_component(value: str) -> str:
    sanitized = "".join(
        ch if ch.isascii() and (ch.isalnum() or ch in "-_") else "_" for ch in value
    )
    return sanitized or "_"


def toml_string(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def resolve_number_tokens(raw: str, labels: Sequence[str]) -> str:
    resolved: list[str] = []
    for token in parse_csv(raw):
        if token.isdigit():
            index = int(token)
            if index < 1 or index > len(labels):
                raise SystemExit(f"Selection index out of range: {index}")
            resolved.append(labels[index - 1])
        else:
            resolved.append(token)
    return ",".join(resolved)


def parse_csv(raw: str) -> list[str]:
    return [part.strip().lower() for part in raw.split(",") if part.strip()]


def unique_in_order(values: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def quote_command(command: Sequence[str]) -> str:
    return " ".join(quote_arg(part) for part in command)


def quote_arg(value: str) -> str:
    if not value or any(ch.isspace() for ch in value):
        return '"' + value.replace('"', '\\"') + '"'
    return value
