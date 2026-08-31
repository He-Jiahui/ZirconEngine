"""Managed filesystem environment for staged Cargo build children."""

from __future__ import annotations

import os
from functools import lru_cache
from pathlib import Path


APPROVED_WINDOWS_BUILD_ROOTS = (
    r"D:\cargo-targets",
    r"E:\cargo-targets",
    r"F:\cargo-targets",
    r"D:\targets",
    r"E:\targets",
    r"F:\targets",
    r"D:\ZirconBuilds",
    r"E:\ZirconBuilds",
    r"F:\ZirconBuilds",
)


@lru_cache(maxsize=1)
def _resolved_approved_windows_build_roots() -> tuple[Path, ...]:
    return tuple(Path(root).resolve() for root in APPROVED_WINDOWS_BUILD_ROOTS)


def assert_managed_windows_build_root(out_root: Path) -> None:
    """Require Windows staging roots to physically resolve under an approved root."""

    if os.name != "nt":
        return
    resolved_root = out_root.resolve()
    for approved_root in _resolved_approved_windows_build_roots():
        try:
            resolved_root.relative_to(approved_root)
            return
        except ValueError:
            continue
    allowed_roots = ", ".join(APPROVED_WINDOWS_BUILD_ROOTS)
    raise ValueError(
        "Windows build output must physically resolve below an approved build root "
        f"({allowed_roots}), not {resolved_root}"
    )


def managed_cargo_environment(target_dir: Path, cache_root: Path) -> dict[str, str]:
    """Keep Cargo output, user cache, compiler cache, and temporary files in staging."""

    target_dir = target_dir.resolve()
    cache_root = cache_root.resolve()
    assert_managed_windows_build_root(cache_root)
    try:
        target_dir.relative_to(cache_root)
    except ValueError as error:
        raise ValueError(
            f"Cargo target must physically resolve beneath build targets root: "
            f"{target_dir} is outside {cache_root}"
        ) from error

    cargo_home = cache_root / "cargo-home"
    sccache = cache_root / "sccache"
    temporary = target_dir / "temporary"
    for directory in (target_dir, cargo_home, sccache, temporary):
        directory.mkdir(parents=True, exist_ok=True)

    environment = os.environ.copy()
    environment.update(
        {
            "CARGO_TARGET_DIR": str(target_dir),
            "CARGO_HOME": str(cargo_home),
            "SCCACHE_DIR": str(sccache),
            "TEMP": str(temporary),
            "TMP": str(temporary),
            "TMPDIR": str(temporary),
        }
    )
    return environment
