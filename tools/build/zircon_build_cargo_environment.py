"""Managed filesystem environment for staged Cargo build children."""

from __future__ import annotations

import os
from functools import lru_cache
from pathlib import Path


APPROVED_WINDOWS_BUILD_ROOTS = (
    r"D:\cargo-targets",
    r"E:\cargo-targets",
    r"F:\cargo-targets",
)


@lru_cache(maxsize=1)
def _resolved_approved_windows_build_roots() -> tuple[Path, ...]:
    resolved_roots: list[Path] = []
    for root in APPROVED_WINDOWS_BUILD_ROOTS:
        requested = Path(root)
        resolved = requested.resolve()
        if resolved != requested:
            raise ValueError(f"Approved drive-root Cargo storage is redirected: {requested} -> {resolved}")
        resolved_roots.append(resolved)
    return tuple(resolved_roots)


def assert_managed_windows_build_root(out_root: Path) -> None:
    """Require Windows staging roots to physically resolve under an approved root."""

    if os.name != "nt":
        return
    # Admit the literal path before resolution: an alias into an approved root
    # is not an approved physical output path. Revalidate mutable ancestry on
    # every call; cached spelling is never native-storage admission.
    from tools.dev.local_cargo import validate_output_path
    try:
        validate_output_path(out_root)
    except ValueError as error:
        raise ValueError(f"Output is not below an approved build root: {error}") from error
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

    # Check the original spelling before .resolve() can erase an alias.
    assert_managed_windows_build_root(target_dir)
    assert_managed_windows_build_root(cache_root)
    target_dir = target_dir.resolve()
    cache_root = cache_root.resolve()
    try:
        target_dir.relative_to(cache_root)
    except ValueError as error:
        raise ValueError(
            f"Cargo target must physically resolve beneath build targets root: "
            f"{target_dir} is outside {cache_root}"
        ) from error

    from tools.dev.local_cargo import (
        MIN_FREE_BYTES, _configuration_preflight, free_bytes, local_environment,
    )
    cargo_home = cache_root / "cargo-home"
    sccache = cache_root / "sccache"
    temporary = target_dir / "temporary"
    build_dir = target_dir / "build-intermediates"
    # Inherit only the caller-prepared compiler environment, never ambient Cargo
    # output overrides, rustflags, runners, wrappers or retired service identity.
    environment, _unused = local_environment(target_dir) if os.name == "nt" else ({
        key: value for key, value in os.environ.items() if key.upper() in {
            "PATH", "HOME", "USERPROFILE", "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT",
            "INCLUDE", "LIB", "LIBPATH", "RUSTUP_HOME", "APPDATA", "LOCALAPPDATA",
        }
    }, [])
    environment.update({
        "CARGO_TARGET_DIR": str(target_dir), "CARGO_BUILD_BUILD_DIR": str(build_dir),
        "CARGO_HOME": str(cargo_home), "SCCACHE_DIR": str(sccache),
        "TEMP": str(temporary), "TMP": str(temporary), "TMPDIR": str(temporary),
        "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "", "CARGO_INCREMENTAL": "0",
        "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0",
        "CARGO_PROFILE_DEV_INCREMENTAL": "false", "CARGO_PROFILE_TEST_INCREMENTAL": "false",
        "PYTHONDONTWRITEBYTECODE": "1",
    })
    code_root = Path(__file__).resolve().parents[2]
    _configuration_preflight(code_root, environment)
    _configuration_preflight(code_root / "zircon_plugins", environment)
    directories = (target_dir, cargo_home, sccache, temporary, build_dir)
    for directory in directories:
        assert_managed_windows_build_root(directory)
        if free_bytes(directory) < MIN_FREE_BYTES:
            raise ValueError("Every output/cache volume requires at least 35 GiB free")
    for directory in directories:
        directory.mkdir(parents=True, exist_ok=True)
        assert_managed_windows_build_root(directory)
    return environment
