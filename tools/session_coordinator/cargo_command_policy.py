from __future__ import annotations

import re
import tomllib
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Callable, Iterable, Mapping

from .models import CoordinatorError
from .portable_paths import normalize_portable_relative_path


_CARGO_EXECUTABLES = frozenset({"cargo", "cargo.exe"})
_SOURCE_PATH_OPTIONS = ("--manifest-path", "--config")
_OUTPUT_PATH_OPTIONS = (
    "--target-dir",
    "--build-dir",
    "--artifact-dir",
    "--lockfile-path",
)
_INLINE_CONFIG = re.compile(r"^\s*([A-Za-z0-9_.-]+)\s*=")
_FORBIDDEN_CONFIG_KEYS = frozenset(
    {
        "build.target",
        "build.target-dir",
        "build.build-dir",
        "build.rustc",
        "build.rustc-wrapper",
        "build.rustc-workspace-wrapper",
        "build.rustflags",
        "env.cargo_target_dir",
        "env.rustc",
        "env.rustc_wrapper",
        "env.rustflags",
    }
)
_FORBIDDEN_CONFIG_SUFFIXES = (
    ".target-dir",
    ".build-dir",
    ".rustc",
    ".rustc-wrapper",
    ".rustc-workspace-wrapper",
    ".rustflags",
    ".linker",
)
_PathMapper = Callable[[str, str], str]
_VALIDATION_SUBCOMMANDS = frozenset(
    {"bench", "build", "check", "clippy", "doc", "metadata", "run", "test"}
)
_PINNED_TOOLCHAIN = re.compile(
    r"^\+(?:\d+\.\d+(?:\.\d+)?(?:-[A-Za-z0-9_.-]+)?|(?:nightly|beta)-\d{4}-\d{2}-\d{2}(?:-[A-Za-z0-9_.-]+)?)$"
)
_PINNED_PACKAGE_SPEC = re.compile(
    r"^(?P<name>[A-Za-z0-9_-]+)(?:@[0-9][A-Za-z0-9.+-]*)?$"
)
_COMPILER_ENVIRONMENT_NAMES = frozenset(
    {
        "ar",
        "cc",
        "cflags",
        "cxx",
        "cxxflags",
        "ldflags",
        "makeflags",
        "pkg_config_path",
        "pkg_config_libdir",
        "vcpkg_root",
    }
)
_COMPILER_ENVIRONMENT_PREFIXES = (
    "cargo_",
    "rust",
    "sccache_",
    "cc_",
    "cxx_",
    "ar_",
)


def is_direct_cargo_command(command: Iterable[str]) -> bool:
    parts = tuple(command)
    if not parts:
        return False
    return parts[0].casefold() in _CARGO_EXECUTABLES


def normalize_cargo_ticket_command(
    command: tuple[str, ...], repo_root: Path | None
) -> tuple[str, ...]:
    """Make every Cargo source path repository-relative and reject owned overrides."""
    if not is_direct_cargo_command(command):
        raise CoordinatorError(
            "validation_ticket_cargo_command_opaque",
            "New Cargo validation tickets require a direct structured Cargo command",
        )
    subcommand = cargo_subcommand(command)
    if subcommand not in _VALIDATION_SUBCOMMANDS:
        raise CoordinatorError(
            "validation_ticket_cargo_subcommand_forbidden",
            "Cargo validation tickets may only run non-mutating build and validation subcommands",
            details={"subcommand": subcommand},
        )
    selector = cargo_toolchain_selector(command)
    if selector is not None and _PINNED_TOOLCHAIN.fullmatch(selector) is None:
        raise CoordinatorError(
            "validation_ticket_cargo_toolchain_unpinned",
            "Cargo toolchain selectors must be exact versions or dated channels",
            details={"selector": selector},
        )
    _validate_reproducible_cargo_arguments(command)
    validate_cargo_storage_arguments(command)
    target = cargo_target_argument(command)
    if target is not None:
        target_path = PureWindowsPath(target)
        if (
            "/" in target
            or "\\" in target
            or target_path.drive
            or target.casefold().endswith(".json")
        ):
            raise CoordinatorError(
                "validation_ticket_cargo_target_unsealed",
                "Custom Cargo target specification paths are not valid immutable ticket inputs",
                details={"target": target},
            )
    resolved_root = repo_root.resolve() if repo_root is not None else None

    def normalize(option: str, value: str) -> str:
        if option == "--config" and inline_cargo_config_key(value) is not None:
            validate_inline_cargo_config(value)
            return value
        return _repository_relative_path(
            value,
            resolved_root,
            manifest=(option == "--manifest-path"),
        )

    return rewrite_cargo_source_path_arguments(command, normalize)


def _validate_reproducible_cargo_arguments(command: tuple[str, ...]) -> None:
    arguments = tuple(_cargo_arguments_before_delimiter(command))
    if "--locked" not in arguments and "--frozen" not in arguments:
        raise CoordinatorError(
            "validation_ticket_cargo_lock_required",
            "Cargo validation tickets must use --locked or --frozen dependency resolution",
        )
    for argument in arguments:
        if argument in {"--fix", "--allow-dirty", "--allow-staged", "--allow-no-vcs"}:
            raise CoordinatorError(
                "validation_ticket_cargo_mutation_forbidden",
                "Cargo validation tickets cannot request source mutation",
                details={"argument": argument},
            )
        if argument == "-Z" or argument.startswith("-Z"):
            raise CoordinatorError(
                "validation_ticket_cargo_unstable_argument_forbidden",
                "Cargo unstable -Z options are outside the reproducible validation contract",
                details={"argument": argument},
            )
        if (
            argument in {"-j", "--jobs"}
            or argument.startswith("-j=")
            or (argument.startswith("-j") and argument[2:].isdigit())
            or argument.startswith("--jobs=")
        ):
            raise CoordinatorError(
                "validation_ticket_cargo_compute_override",
                "Cargo build parallelism is owned by the coordinator",
                details={"argument": argument},
            )
        if argument.startswith("-p") and argument not in {"-p"} and not argument.startswith("-p="):
            raise CoordinatorError(
                "validation_ticket_cargo_argument_unsupported",
                "Attached -p package values are unsupported; pass -p and the package separately",
                details={"argument": argument},
            )
        if argument.startswith("-F") and argument not in {"-F"} and not argument.startswith("-F="):
            raise CoordinatorError(
                "validation_ticket_cargo_argument_unsupported",
                "Attached -F feature values are unsupported; pass -F and the feature separately",
                details={"argument": argument},
            )
    for spec in (*_raw_cargo_package_specs(command), *_raw_cargo_excluded_package_specs(command)):
        if _PINNED_PACKAGE_SPEC.fullmatch(spec) is None:
            raise CoordinatorError(
                "validation_ticket_cargo_package_spec_unsupported",
                "Cargo package selectors must be a package name or package name with an exact version",
                details={"packageSpec": spec},
            )


def _cargo_arguments_before_delimiter(command: tuple[str, ...]) -> Iterable[str]:
    for part in command[1:]:
        if part == "--":
            return
        yield part


def validate_cargo_storage_arguments(command: tuple[str, ...]) -> None:
    direct = is_direct_cargo_command(command)
    for index, part in enumerate(command):
        if direct and part == "--":
            break
        for option in _OUTPUT_PATH_OPTIONS:
            if part == option or part.startswith(option + "="):
                raise CoordinatorError(
                    "validation_ticket_cargo_storage_override",
                    "Cargo output directories are owned by the coordinator",
                    details={"argument": part, "index": index},
                )
        config_value: str | None = None
        if part == "--config" and index + 1 < len(command):
            config_value = command[index + 1]
        elif part.startswith("--config="):
            config_value = part[len("--config=") :]
        if config_value is not None and inline_cargo_config_key(config_value) is not None:
            validate_inline_cargo_config(config_value)
    # Shell wrappers are retained for existing coordinator tickets, but any
    # embedded storage/compiler override is rejected because it cannot be
    # rewritten safely without a shell parser.
    if not is_direct_cargo_command(command):
        text = " ".join(command)
        lowered = text.casefold()
        if any(
            re.search(
                rf"(?:^|\s){re.escape(option)}(?:=|\s)", lowered
            )
            for option in _OUTPUT_PATH_OPTIONS
        ) or any(
            marker in lowered
            for marker in (
                "build.target-dir=",
                "build.build-dir=",
                "build.rustc-wrapper=",
                "build.rustc-workspace-wrapper=",
                "build.rustflags=",
            )
        ):
            raise CoordinatorError(
                "validation_ticket_cargo_storage_override",
                "Opaque Cargo wrappers cannot override coordinator-owned output or compiler settings",
            )


def validate_inline_cargo_config(value: str) -> None:
    keys = _inline_cargo_config_keys(value)
    if keys is None:
        return
    raise CoordinatorError(
        "validation_ticket_cargo_storage_override",
        "Inline Cargo configuration is not an immutable validation input; use a sealed config file",
        details={"configKeys": list(keys)},
    )


def inline_cargo_config_key(value: str) -> str | None:
    keys = _inline_cargo_config_keys(value)
    return keys[0] if keys else None


def _inline_cargo_config_keys(value: str) -> tuple[str, ...] | None:
    if "=" not in value:
        return None
    try:
        document = tomllib.loads(value)
    except tomllib.TOMLDecodeError as error:
        raise CoordinatorError(
            "validation_ticket_cargo_config_invalid",
            "Cargo --config values containing '=' must be valid inline TOML",
        ) from error
    leaves: list[str] = []

    def visit(node: dict[str, object], prefix: tuple[str, ...] = ()) -> None:
        for key, child in node.items():
            path = (*prefix, str(key))
            if isinstance(child, dict):
                visit(child, path)
            else:
                leaves.append(".".join(path))

    visit(document)
    if len(leaves) != 1:
        raise CoordinatorError(
            "validation_ticket_cargo_config_invalid",
            "Cargo inline --config must define exactly one value",
        )
    return tuple(leaves)


def rewrite_cargo_source_path_arguments(
    command: tuple[str, ...], mapper: _PathMapper
) -> tuple[str, ...]:
    rewritten = list(command)
    index = 0
    while index < len(rewritten):
        part = rewritten[index]
        if part == "--":
            break
        matched = False
        for option in _SOURCE_PATH_OPTIONS:
            if part == option:
                if index + 1 >= len(rewritten) or not rewritten[index + 1].strip():
                    raise CoordinatorError(
                        "cargo_source_path_argument_invalid",
                        f"{option} requires a non-empty path or inline configuration",
                    )
                rewritten[index + 1] = mapper(option, rewritten[index + 1])
                index += 2
                matched = True
                break
            prefix = option + "="
            if part.startswith(prefix):
                value = part[len(prefix) :]
                if not value.strip():
                    raise CoordinatorError(
                        "cargo_source_path_argument_invalid",
                        f"{option} requires a non-empty path or inline configuration",
                    )
                rewritten[index] = prefix + mapper(option, value)
                index += 1
                matched = True
                break
        if not matched:
            index += 1
    return tuple(rewritten)


def cargo_config_file_arguments(command: tuple[str, ...]) -> tuple[str, ...]:
    result: list[str] = []

    def capture(option: str, value: str) -> str:
        if option == "--config" and inline_cargo_config_key(value) is None:
            result.append(value)
        return value

    rewrite_cargo_source_path_arguments(command, capture)
    return tuple(result)


def cargo_manifest_path_argument(command: tuple[str, ...]) -> str | None:
    result: str | None = None

    def capture(option: str, value: str) -> str:
        nonlocal result
        if option == "--manifest-path":
            result = value
        return value

    rewrite_cargo_source_path_arguments(command, capture)
    return result


def cargo_toolchain_selector(command: tuple[str, ...]) -> str | None:
    for index, part in enumerate(command):
        if Path(part).name.casefold() not in _CARGO_EXECUTABLES:
            continue
        if index + 1 < len(command) and command[index + 1].startswith("+"):
            selector = command[index + 1].strip()
            return selector if len(selector) > 1 else None
        return None
    return None


def cargo_subcommand(command: tuple[str, ...]) -> str:
    if not is_direct_cargo_command(command):
        raise CoordinatorError(
            "validation_ticket_cargo_command_opaque",
            "Cargo command must be direct and structured",
        )
    index = 1
    if index < len(command) and command[index].startswith("+"):
        index += 1
    value_options = {"--color", "--config"}
    global_flags = {"-q", "--quiet", "-v", "--verbose", "--frozen", "--locked", "--offline"}
    while index < len(command):
        part = command[index]
        if part == "--":
            break
        if part in value_options:
            if index + 1 >= len(command):
                break
            index += 2
            continue
        if any(part.startswith(option + "=") for option in value_options):
            index += 1
            continue
        if part in global_flags:
            index += 1
            continue
        if part.startswith("-"):
            raise CoordinatorError(
                "validation_ticket_cargo_global_argument_unsupported",
                "Cargo global argument cannot be interpreted safely",
                details={"argument": part},
            )
        return part.casefold()
    raise CoordinatorError(
        "validation_ticket_cargo_subcommand_missing",
        "Cargo validation command must include a subcommand",
    )


def cargo_target_argument(command: tuple[str, ...]) -> str | None:
    targets: list[str] = []
    for index, part in enumerate(command):
        if part == "--":
            break
        if part == "--target" and index + 1 < len(command):
            targets.append(command[index + 1].strip())
        elif part.startswith("--target="):
            targets.append(part[len("--target=") :].strip())
    if len(targets) > 1:
        raise CoordinatorError(
            "validation_ticket_cargo_target_duplicate",
            "Cargo validation tickets may select exactly one compilation target",
            details={"targets": targets},
        )
    return (targets[0] or None) if targets else None


def cargo_package_specs(command: tuple[str, ...]) -> tuple[str, ...]:
    packages: list[str] = []
    for value in _raw_cargo_package_specs(command):
        match = _PINNED_PACKAGE_SPEC.fullmatch(value)
        normalized = match.group("name") if match is not None else value
        if normalized not in packages:
            packages.append(normalized)
    return tuple(packages)


def _raw_cargo_package_specs(command: tuple[str, ...]) -> tuple[str, ...]:
    packages: list[str] = []
    index = 0
    while index < len(command):
        part = command[index]
        if part == "--":
            break
        value: str | None = None
        if part in {"-p", "--package"} and index + 1 < len(command):
            value = command[index + 1]
            index += 2
        elif part.startswith("-p="):
            value = part[3:]
            index += 1
        elif part.startswith("--package="):
            value = part[len("--package=") :]
            index += 1
        else:
            index += 1
        if value is None:
            continue
        normalized = value.strip()
        if normalized and normalized not in packages:
            packages.append(normalized)
    return tuple(packages)


def cargo_excluded_package_specs(command: tuple[str, ...]) -> tuple[str, ...]:
    packages: list[str] = []
    for value in _raw_cargo_excluded_package_specs(command):
        match = _PINNED_PACKAGE_SPEC.fullmatch(value)
        normalized = match.group("name") if match is not None else value
        if normalized not in packages:
            packages.append(normalized)
    return tuple(packages)


def _raw_cargo_excluded_package_specs(command: tuple[str, ...]) -> tuple[str, ...]:
    packages: list[str] = []
    index = 0
    while index < len(command):
        part = command[index]
        if part == "--":
            break
        value: str | None = None
        if part == "--exclude" and index + 1 < len(command):
            value = command[index + 1]
            index += 2
        elif part.startswith("--exclude="):
            value = part[len("--exclude=") :]
            index += 1
        else:
            index += 1
        normalized = value.strip() if value is not None else ""
        if normalized and normalized not in packages:
            packages.append(normalized)
    return tuple(packages)


def cargo_selects_workspace(command: tuple[str, ...]) -> bool:
    for part in command:
        if part == "--":
            break
        if part in {"--workspace", "--all"}:
            return True
    return False


def scrub_inherited_cargo_environment(
    environment: Mapping[str, str],
) -> dict[str, str]:
    result: dict[str, str] = {}
    for name, value in environment.items():
        folded = name.casefold()
        if folded in _COMPILER_ENVIRONMENT_NAMES or folded.startswith(
            _COMPILER_ENVIRONMENT_PREFIXES
        ):
            continue
        result[name] = value
    return result


def validate_no_ambient_cargo_configs(source_root: Path) -> None:
    root = source_root.resolve()
    candidate = root.parent
    while True:
        for relative in (Path(".cargo/config"), Path(".cargo/config.toml")):
            config = candidate / relative
            if config.is_file():
                raise CoordinatorError(
                    "cargo_ambient_config_forbidden",
                    "Managed Cargo source ancestors cannot contain ambient configuration",
                    details={"sourceRoot": str(root), "configPath": str(config)},
                )
        if candidate == candidate.parent:
            return
        candidate = candidate.parent


def _repository_relative_path(
    value: str, repo_root: Path | None, *, manifest: bool
) -> str:
    stripped = value.strip()
    windows = PureWindowsPath(stripped)
    posix = PurePosixPath(stripped.replace("\\", "/"))
    absolute = Path(stripped).is_absolute() or windows.is_absolute()
    if absolute:
        if repo_root is None:
            raise CoordinatorError(
                "cargo_source_path_argument_invalid",
                "Absolute Cargo source paths require a configured repository root",
                details={"path": value},
            )
        try:
            relative = Path(stripped).resolve(strict=False).relative_to(repo_root)
        except ValueError as error:
            raise CoordinatorError(
                "cargo_source_path_argument_invalid",
                "Cargo source path must stay inside the pinned repository",
                details={"path": value},
            ) from error
        normalized = PurePosixPath(relative.as_posix())
    else:
        normalized = PurePosixPath(
            normalize_portable_relative_path(
                stripped,
                code="cargo_source_path_argument_invalid",
                message="Cargo source path must be a normalized repository-relative path",
            )
        )
    if manifest and normalized.name.casefold() != "cargo.toml":
        raise CoordinatorError(
            "cargo_source_path_argument_invalid",
            "--manifest-path must identify a Cargo.toml inside the pinned repository",
            details={"path": value},
        )
    return normalized.as_posix()
