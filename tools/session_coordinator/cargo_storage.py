from __future__ import annotations

import os
import subprocess
import threading
from pathlib import Path


_MANAGED_ROOT_NAMES = {"cargo-targets", "targets", "zirconbuilds"}
_MANAGED_SCCACHE_PORTS = {
    ("D:", "cargo-targets"): 42260,
    ("E:", "cargo-targets"): 42261,
    ("F:", "cargo-targets"): 42262,
    ("D:", "targets"): 42263,
    ("E:", "targets"): 42264,
    ("F:", "targets"): 42265,
    ("D:", "zirconbuilds"): 42266,
    ("E:", "zirconbuilds"): 42267,
    ("F:", "zirconbuilds"): 42268,
}
_CARGO_HOME_LAYOUT_LOCK = threading.Lock()


def managed_cargo_storage_root(target_directory: str | Path) -> Path:
    """Return the stable per-drive storage root shared by managed Cargo jobs."""
    target = Path(target_directory).resolve()
    engine_root = next(
        (
            candidate
            for candidate in (target, *target.parents)
            if candidate.name.casefold() == "zircon-engine"
        ),
        None,
    )
    if engine_root is not None:
        return engine_root
    approved_root = next(
        (
            candidate
            for candidate in (target, *target.parents)
            if candidate.name.casefold() in _MANAGED_ROOT_NAMES
        ),
        None,
    )
    return approved_root / "zircon-engine" if approved_root is not None else target.parent


def managed_cargo_cache_paths(target_directory: str | Path) -> tuple[Path, Path]:
    cache_root = managed_cargo_storage_root(target_directory) / "cache"
    return cache_root / "cargo-home", cache_root / "sccache"


def managed_cargo_metadata_home_path(target_directory: str | Path) -> Path:
    return managed_cargo_storage_root(target_directory) / "cache" / "cargo-metadata-home"


def prepare_isolated_cargo_home(
    target_directory: str | Path,
    control_home: str | Path,
) -> Path:
    """Create a config-free Cargo home with private consumable source trees."""
    shared_home, _sccache = managed_cargo_cache_paths(target_directory)
    shared_home.mkdir(parents=True, exist_ok=True)
    isolated = Path(control_home).absolute()
    _prepare_cargo_home_layout(isolated, shared_home, share_consumable_sources=False)
    return isolated


def prepare_shared_metadata_cargo_home(target_directory: str | Path) -> Path:
    """Return the stable Cargo home used only by non-building metadata commands."""
    shared_home, _sccache = managed_cargo_cache_paths(target_directory)
    shared_home.mkdir(parents=True, exist_ok=True)
    metadata_home = managed_cargo_metadata_home_path(target_directory)
    _prepare_cargo_home_layout(metadata_home, shared_home, share_consumable_sources=True)
    return metadata_home


def _prepare_cargo_home_layout(
    home: Path,
    shared_home: Path,
    *,
    share_consumable_sources: bool,
) -> None:
    with _CARGO_HOME_LAYOUT_LOCK:
        _prepare_cargo_home_layout_locked(
            home,
            shared_home,
            share_consumable_sources=share_consumable_sources,
        )


def _prepare_cargo_home_layout_locked(
    home: Path,
    shared_home: Path,
    *,
    share_consumable_sources: bool,
) -> None:
    isolated = home.absolute()
    if isolated.is_symlink() or isolated.is_junction():
        raise OSError(f"private Cargo home cannot be a filesystem link: {isolated}")
    isolated.mkdir(parents=True, exist_ok=True)
    for config_name in ("config", "config.toml"):
        if os.path.lexists(isolated / config_name):
            raise OSError(
                f"isolated Cargo home contains forbidden configuration: {isolated / config_name}"
            )
    # Share verified downloads/indexes in every mode. Build jobs get private
    # extracted sources because build scripts may mutate them; metadata jobs
    # explicitly opt into the stable source view below and never run scripts.
    for relative in (
        Path("registry/cache"),
        Path("registry/index"),
        Path("git/db"),
    ):
        shared_directory = shared_home / relative
        shared_directory.mkdir(parents=True, exist_ok=True)
        link = isolated / relative
        link.parent.mkdir(parents=True, exist_ok=True)
        _ensure_directory_link(link, shared_directory)
    for relative in (Path("registry/src"), Path("git/checkouts")):
        directory = isolated / relative
        if share_consumable_sources:
            shared_directory = shared_home / relative
            shared_directory.mkdir(parents=True, exist_ok=True)
            _ensure_directory_link(directory, shared_directory)
        else:
            if directory.is_symlink() or directory.is_junction():
                raise OSError(f"private Cargo source directory cannot be a filesystem link: {directory}")
            directory.mkdir(parents=True, exist_ok=True)
    for lock_name in (".package-cache", ".package-cache-mutate"):
        shared_lock = shared_home / lock_name
        shared_lock.touch(exist_ok=True)
        isolated_lock = isolated / lock_name
        _ensure_file_link(isolated_lock, shared_lock)


def _ensure_file_link(link: Path, target: Path) -> None:
    if os.path.lexists(link):
        if not os.path.samefile(link, target):
            raise OSError(
                f"isolated Cargo cache lock has an unexpected identity: {link}"
            )
        return
    try:
        os.link(target, link)
    except FileExistsError:
        if not os.path.samefile(link, target):
            raise OSError(
                f"isolated Cargo cache lock has an unexpected identity: {link}"
            )


def _ensure_directory_link(link: Path, target: Path) -> None:
    if os.path.lexists(link):
        if not link.is_dir() or link.resolve() != target.resolve():
            raise OSError(f"isolated Cargo cache link has an unexpected target: {link}")
        return
    try:
        os.symlink(target, link, target_is_directory=True)
        return
    except OSError:
        if link.is_dir() and link.resolve() == target.resolve():
            return
        if os.name != "nt":
            raise
    system_root = Path(os.environ.get("SystemRoot", r"C:\Windows")).resolve()
    command = system_root / "System32" / "cmd.exe"
    if not command.is_file() or '"' in str(link) or '"' in str(target):
        raise OSError(f"could not create isolated Cargo cache link: {link}")
    result = subprocess.run(
        [str(command), "/d", "/c", "mklink", "/J", str(link), str(target)],
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if not link.is_dir() or link.resolve() != target.resolve():
        raise OSError(
            f"could not create isolated Cargo cache junction {link}: {result.stderr.strip()}"
        )


def managed_cargo_server_temp_path(target_directory: str | Path) -> Path:
    return managed_cargo_storage_root(target_directory) / "cache" / "sccache-temporary"


def managed_native_dynamic_cas_path(target_directory: str | Path) -> Path:
    """Return the stable native artifact CAS shared by managed validations."""
    return managed_cargo_storage_root(target_directory) / "cache" / "native-dynamic"


def managed_cargo_server_port(target_directory: str | Path) -> int:
    target = Path(target_directory).resolve()
    drive = target.drive.upper()
    root_name = next(
        (
            candidate.name.casefold()
            for candidate in (target, *target.parents)
            if candidate.name.casefold() in _MANAGED_ROOT_NAMES
        ),
        "",
    )
    try:
        return _MANAGED_SCCACHE_PORTS[(drive, root_name)]
    except KeyError as error:
        raise ValueError(
            "unsupported managed Cargo storage root: "
            f"{drive or '<none>'}\\{root_name or '<none>'}"
        ) from error


def managed_cargo_scratch_path(target_directory: str | Path, job_id: str) -> Path:
    return managed_cargo_storage_root(target_directory) / "scratch" / job_id
