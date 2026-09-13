from __future__ import annotations

from pathlib import Path
from typing import Mapping

from .cargo_command_policy import cargo_subcommand, is_direct_cargo_command


_LIBRARY_TARGET_KINDS = frozenset(
    {"lib", "proc-macro", "rlib", "dylib", "cdylib", "staticlib"}
)


def cargo_command_includes_test_code(command: tuple[str, ...]) -> bool:
    if not is_direct_cargo_command(command):
        return True
    if cargo_subcommand(command) in {"test", "bench"}:
        return True
    arguments = command[: command.index("--")] if "--" in command else command
    if any(
        argument in {"--tests", "--benches", "--all-targets"}
        for argument in arguments
    ):
        return True
    return any(
        argument in {"--test", "--bench"}
        or argument.startswith(("--test=", "--bench="))
        for argument in arguments
    )


def cargo_target_sources_for_command(
    package: Mapping[str, object],
    command: tuple[str, ...],
    *,
    selected_package: bool,
) -> tuple[Path, ...]:
    """Select metadata targets that the requested Cargo command can compile."""

    raw_targets = package.get("targets", [])
    if not isinstance(raw_targets, list):
        return ()
    if not is_direct_cargo_command(command):
        return _all_target_sources(raw_targets)

    arguments = command[: command.index("--")] if "--" in command else command
    subcommand = cargo_subcommand(command)
    all_targets = "--all-targets" in arguments
    library_only = "--lib" in arguments
    selected_bins = set(_option_values(command, frozenset({"--bin"})))
    selected_examples = set(_option_values(command, frozenset({"--example"})))
    selected_tests = set(_option_values(command, frozenset({"--test"})))
    selected_benches = set(_option_values(command, frozenset({"--bench"})))
    all_bins = "--bins" in arguments
    all_examples = "--examples" in arguments
    all_tests = "--tests" in arguments
    all_benches = "--benches" in arguments
    explicit_selector = bool(
        all_targets
        or library_only
        or selected_bins
        or selected_examples
        or selected_tests
        or selected_benches
        or all_bins
        or all_examples
        or all_tests
        or all_benches
    )

    selected_sources: list[Path] = []
    for raw_target in raw_targets:
        if not isinstance(raw_target, Mapping) or not raw_target.get("src_path"):
            continue
        source = Path(str(raw_target["src_path"])).resolve()
        kinds = _target_kinds(raw_target)
        # Older hand-authored metadata fixtures omitted `kind`; keep their
        # fail-closed behavior. Real Cargo metadata always supplies it.
        if not kinds:
            selected_sources.append(source)
            continue
        name = str(raw_target.get("name") or "")
        if "custom-build" in kinds:
            selected_sources.append(source)
            continue
        if not selected_package:
            if kinds & _LIBRARY_TARGET_KINDS:
                selected_sources.append(source)
            continue
        if all_targets:
            selected_sources.append(source)
            continue

        explicit_kinds: set[str] = set()
        if library_only:
            explicit_kinds.update(_LIBRARY_TARGET_KINDS)
        if selected_bins or all_bins:
            if "bin" in kinds and (all_bins or name in selected_bins):
                explicit_kinds.add("bin")
            explicit_kinds.update(_LIBRARY_TARGET_KINDS)
        if selected_examples or all_examples:
            if "example" in kinds and (all_examples or name in selected_examples):
                explicit_kinds.add("example")
            explicit_kinds.update(_LIBRARY_TARGET_KINDS)
        if selected_tests or all_tests:
            if "test" in kinds and (all_tests or name in selected_tests):
                explicit_kinds.add("test")
            explicit_kinds.update(_LIBRARY_TARGET_KINDS)
        if selected_benches or all_benches:
            if "bench" in kinds and (all_benches or name in selected_benches):
                explicit_kinds.add("bench")
            explicit_kinds.update(_LIBRARY_TARGET_KINDS)
        if explicit_kinds:
            if kinds & explicit_kinds:
                selected_sources.append(source)
            continue
        if explicit_selector:
            continue
        if subcommand == "test":
            if kinds & {"lib", "proc-macro", "bin", "example", "test"}:
                selected_sources.append(source)
        elif subcommand == "bench":
            if kinds & {"lib", "proc-macro", "bin", "example", "bench"}:
                selected_sources.append(source)
        elif kinds & (_LIBRARY_TARGET_KINDS | {"bin"}):
            selected_sources.append(source)
    return tuple(dict.fromkeys(selected_sources))


def cargo_test_target_sources_for_command(
    package: Mapping[str, object],
    command: tuple[str, ...],
    *,
    selected_package: bool,
) -> tuple[Path, ...]:
    """Return target sources compiled with cfg(test) for a direct Cargo command."""

    raw_targets = package.get("targets", [])
    if not isinstance(raw_targets, list):
        return ()
    if not is_direct_cargo_command(command):
        return _all_target_sources(raw_targets)
    if not selected_package or not cargo_command_includes_test_code(command):
        return ()
    arguments = command[: command.index("--")] if "--" in command else command
    subcommand = cargo_subcommand(command)
    all_targets = "--all-targets" in arguments
    library_only = "--lib" in arguments
    selected_bins = set(_option_values(command, frozenset({"--bin"})))
    selected_examples = set(_option_values(command, frozenset({"--example"})))
    selected_tests = set(_option_values(command, frozenset({"--test"})))
    selected_benches = set(_option_values(command, frozenset({"--bench"})))
    all_bins = "--bins" in arguments
    all_examples = "--examples" in arguments
    all_tests = "--tests" in arguments
    all_benches = "--benches" in arguments
    if "--doc" in arguments:
        return ()
    explicit_selector = bool(all_targets or library_only or selected_bins or selected_examples
                             or selected_tests or selected_benches or all_bins or all_examples
                             or all_tests or all_benches)
    selected_sources: list[Path] = []
    for raw_target in raw_targets:
        if not isinstance(raw_target, Mapping) or not raw_target.get("src_path"):
            continue
        kinds = _target_kinds(raw_target)
        if "custom-build" in kinds:
            continue
        name = str(raw_target.get("name") or "")
        test_enabled = bool(raw_target.get("test", bool(kinds & (_LIBRARY_TARGET_KINDS | {"bin", "test"}))))
        bench_enabled = bool(raw_target.get("bench", bool(kinds & (_LIBRARY_TARGET_KINDS | {"bin", "bench"}))))
        if not explicit_selector:
            selected = test_enabled if subcommand == "test" else bench_enabled
        else:
            selected = bool(
                all_targets
                or (library_only and kinds & _LIBRARY_TARGET_KINDS)
                or ("bin" in kinds and (all_bins or name in selected_bins))
                or ("example" in kinds and (all_examples or name in selected_examples))
                or ("test" in kinds and name in selected_tests)
                or ("bench" in kinds and name in selected_benches)
                or (all_tests and test_enabled)
                or (all_benches and bench_enabled)
            )
        if selected or not kinds:
            selected_sources.append(Path(str(raw_target["src_path"])).resolve())
    return tuple(dict.fromkeys(selected_sources))


def _all_target_sources(raw_targets: list[object]) -> tuple[Path, ...]:
    return tuple(
        dict.fromkeys(
            Path(str(target["src_path"])).resolve()
            for target in raw_targets
            if isinstance(target, Mapping) and target.get("src_path")
        )
    )


def _target_kinds(target: Mapping[str, object]) -> set[str]:
    raw_kinds = target.get("kind", ())
    if isinstance(raw_kinds, str):
        return {raw_kinds.casefold()}
    if isinstance(raw_kinds, (list, tuple, set, frozenset)):
        return {str(kind).casefold() for kind in raw_kinds}
    return set()


def _option_values(
    command: tuple[str, ...], options: frozenset[str]
) -> tuple[str, ...]:
    values: list[str] = []
    arguments = command[: command.index("--")] if "--" in command else command
    index = 0
    while index < len(arguments):
        part = arguments[index]
        matched = False
        for option in options:
            if part == option and index + 1 < len(arguments):
                value = arguments[index + 1].strip()
                if value:
                    values.append(value)
                index += 2
                matched = True
                break
            prefix = option + "="
            if part.startswith(prefix):
                value = part[len(prefix) :].strip()
                if value:
                    values.append(value)
                index += 1
                matched = True
                break
        if not matched:
            index += 1
    return tuple(values)
