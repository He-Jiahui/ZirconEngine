from __future__ import annotations

from bisect import bisect_left
from collections import OrderedDict
import json
import posixpath
import subprocess
import tempfile
import threading
import tomllib
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Callable, Iterable, Mapping

from .cargo_command_policy import (
    cargo_config_file_arguments,
    cargo_excluded_package_specs,
    cargo_manifest_path_argument,
    cargo_package_specs,
    cargo_selects_workspace,
    cargo_subcommand,
    is_direct_cargo_command,
)
from .cargo_target_selection import (
    cargo_command_includes_test_code,
    cargo_test_target_sources_for_command,
    cargo_target_sources_for_command,
)
from .models import CoordinatorError
from .trusted_tools import bind_trusted_cargo, trusted_git_command
from .validation_copy_external import (
    EXTERNAL_REPOSITORY_ROOT,
    ExternalGitSource,
    external_source_includes_path,
    external_topology_paths,
    external_tree_paths,
)


@dataclass(frozen=True, slots=True)
class CargoInputClosure:
    repository_paths: tuple[str, ...]
    external_sources: tuple[ExternalGitSource, ...]


_COMPILE_TIME_INCLUDE_MACROS = frozenset({"include", "include_bytes", "include_str"})
_CARGO_MANIFEST_DIR = "CARGO_MANIFEST_DIR"
_OUT_DIR = "OUT_DIR"
_GIT_PATHSPEC_COMMAND_CHAR_LIMIT = 24_000
_COMPILE_TIME_SOURCE_LIMIT = 32_768
_COMPILE_TIME_SOURCE_BATCH_SIZE = 128
_COMPILE_TIME_SOURCE_MAX_BYTES = 8 * 1024 * 1024
_COMPILE_TIME_SOURCE_TOTAL_MAX_BYTES = 128 * 1024 * 1024
_BASELINE_COMPILE_TIME_CACHE_LIMIT = 4
_RustToken = tuple[str, str]
_BaselineCompileTimeCacheKey = tuple[
    str, str, tuple[str, ...], tuple[str, ...], tuple[str, ...]
]
_BaselineCompileTimeCacheValue = tuple[tuple[str, tuple[str, ...]], ...]
_BASELINE_COMPILE_TIME_CACHE: OrderedDict[
    _BaselineCompileTimeCacheKey, _BaselineCompileTimeCacheValue
] = OrderedDict()
_BASELINE_COMPILE_TIME_INFLIGHT: dict[
    _BaselineCompileTimeCacheKey, threading.Event
] = {}
_BASELINE_COMPILE_TIME_CACHE_LOCK = threading.Lock()


@dataclass
class _CompileTimeSourceBudget:
    max_file_bytes: int = _COMPILE_TIME_SOURCE_MAX_BYTES
    max_total_bytes: int = _COMPILE_TIME_SOURCE_TOTAL_MAX_BYTES
    total_bytes: int = 0

    def account(self, relative: str, byte_count: int) -> None:
        if byte_count > self.max_file_bytes:
            raise CoordinatorError(
                "validation_copy_compile_time_source_too_large",
                "Compile-time Rust source exceeds its per-file byte budget",
                details={
                    "sourcePath": relative,
                    "byteCount": byte_count,
                    "maxByteCount": self.max_file_bytes,
                },
            )
        if self.total_bytes + byte_count > self.max_total_bytes:
            raise CoordinatorError(
                "validation_copy_compile_time_source_total_too_large",
                "Compile-time Rust sources exceed their aggregate byte budget",
                details={
                    "sourcePath": relative,
                    "totalByteCount": self.total_bytes + byte_count,
                    "maxTotalByteCount": self.max_total_bytes,
                },
            )
        self.total_bytes += byte_count


def _rust_tokens(source: str) -> tuple[_RustToken, ...]:
    """Tokenize enough Rust to locate real include macros without parsing source strings."""

    tokens: list[_RustToken] = []
    index = 0
    length = len(source)
    while index < length:
        current = source[index]
        following = source[index + 1] if index + 1 < length else ""
        if current.isspace():
            index += 1
            continue
        if current == "/" and following == "/":
            newline = source.find("\n", index + 2)
            index = length if newline < 0 else newline + 1
            continue
        if current == "/" and following == "*":
            index = _skip_rust_block_comment(source, index + 2)
            continue
        if current == "r":
            raw_end = _rust_raw_string_end(source, index)
            if raw_end is not None:
                content_start, end = raw_end
                tokens.append(("string", source[content_start:end]))
                hash_count = content_start - index - 2
                index = end + 1 + hash_count
                continue
        if current == '"':
            value, index = _rust_string(source, index + 1)
            tokens.append(("string", value))
            continue
        if current == "'":
            character_end = _rust_character_end(source, index)
            if character_end is not None:
                index = character_end
                continue
        if current.isalpha() or current == "_":
            end = index + 1
            while end < length and (source[end].isalnum() or source[end] == "_"):
                end += 1
            tokens.append(("ident", source[index:end]))
            index = end
            continue
        if current in "(){}!,#[]=":
            tokens.append((current, current))
        elif current == "$":
            tokens.append(("dollar", current))
        else:
            tokens.append(("other", current))
        index += 1
    return tuple(tokens)


def _skip_rust_block_comment(source: str, index: int) -> int:
    depth = 1
    while index < len(source) and depth:
        pair = source[index : index + 2]
        if pair == "/*":
            depth += 1
            index += 2
        elif pair == "*/":
            depth -= 1
            index += 2
        else:
            index += 1
    return index


def _rust_raw_string_end(source: str, index: int) -> tuple[int, int] | None:
    cursor = index + 1
    while cursor < len(source) and source[cursor] == "#":
        cursor += 1
    if cursor >= len(source) or source[cursor] != '"':
        return None
    hashes = source[index + 1 : cursor]
    content_start = cursor + 1
    closing = '"' + hashes
    end = source.find(closing, content_start)
    if end < 0:
        return content_start, len(source)
    return content_start, end


def _rust_string(source: str, index: int) -> tuple[str, int]:
    value: list[str] = []
    while index < len(source):
        current = source[index]
        if current == '"':
            return "".join(value), index + 1
        if current == "\\" and index + 1 < len(source):
            index += 1
            value.append(source[index])
        else:
            value.append(current)
        index += 1
    return "".join(value), index


def _rust_character_end(source: str, index: int) -> int | None:
    cursor = index + 1
    if cursor >= len(source) or source[cursor] in "\r\n":
        return None
    if source[cursor] == "\\":
        cursor += 1
        if cursor >= len(source) or source[cursor] in "\r\n":
            return None
        if (
            source[cursor] == "u"
            and cursor + 1 < len(source)
            and source[cursor + 1] == "{"
        ):
            closing = source.find("}", cursor + 2)
            if closing < 0:
                return None
            cursor = closing + 1
        elif source[cursor] == "x":
            cursor += 3
        else:
            cursor += 1
    else:
        cursor += 1
    return cursor + 1 if cursor < len(source) and source[cursor] == "'" else None


def _matching_delimiter(tokens: tuple[_RustToken, ...], opening: int) -> int | None:
    pairs = {"(": ")", "[": "]", "{": "}"}
    opening_kind = tokens[opening][0] if opening < len(tokens) else ""
    if opening_kind not in pairs:
        return None
    stack: list[str] = []
    for index in range(opening, len(tokens)):
        kind, _value = tokens[index]
        if kind in pairs:
            stack.append(pairs[kind])
        elif stack and kind == stack[-1]:
            stack.pop()
            if not stack:
                return index
    return None


def _macro_arguments(
    tokens: tuple[_RustToken, ...], name: str
) -> tuple[_RustToken, ...] | None:
    if len(tokens) < 4 or tokens[0] != ("ident", name) or tokens[1][0] != "!":
        return None
    if tokens[2][0] not in {"(", "[", "{"}:
        return None
    closing = _matching_delimiter(tokens, 2)
    if closing != len(tokens) - 1:
        return None
    return tokens[3:closing]


def _split_top_level_arguments(tokens: tuple[_RustToken, ...]) -> tuple[tuple[_RustToken, ...], ...]:
    arguments: list[tuple[_RustToken, ...]] = []
    start = 0
    depth = 0
    for index, (kind, _value) in enumerate(tokens):
        if kind in {"(", "[", "{"}:
            depth += 1
        elif kind in {")", "]", "}"}:
            depth -= 1
        elif kind == "," and depth == 0:
            arguments.append(tokens[start:index])
            start = index + 1
    if start < len(tokens):
        arguments.append(tokens[start:])
    return tuple(argument for argument in arguments if argument)


def _string_argument(tokens: tuple[_RustToken, ...]) -> str | None:
    if tokens and tokens[-1][0] == ",":
        tokens = tokens[:-1]
    if len(tokens) == 1 and tokens[0][0] == "string":
        return tokens[0][1]
    return None


def _is_environment_variable(tokens: tuple[_RustToken, ...], name: str) -> bool:
    arguments = _macro_arguments(tokens, "env")
    return arguments is not None and _string_argument(arguments) == name


def _is_cargo_manifest_dir(tokens: tuple[_RustToken, ...]) -> bool:
    return _is_environment_variable(tokens, _CARGO_MANIFEST_DIR)


def _is_build_output_resource(expression: tuple[_RustToken, ...]) -> bool:
    if _is_environment_variable(expression, _OUT_DIR):
        return True
    arguments = _macro_arguments(expression, "concat")
    if arguments is None:
        return False
    components = _split_top_level_arguments(arguments)
    return bool(components) and _is_environment_variable(components[0], _OUT_DIR)


def _compile_time_resource(
    expression: tuple[_RustToken, ...],
    *,
    source: Path,
    package_root: Path,
    repo_root: Path,
) -> Path:
    literal = _string_argument(expression)
    if literal is not None:
        candidate = (source.parent / literal).resolve()
    else:
        arguments = _macro_arguments(expression, "concat")
        if arguments is None:
            raise CoordinatorError(
                "validation_copy_compile_time_resource_unresolved",
                "Compile-time include expression cannot be resolved safely",
                details={"sourcePath": str(source)},
            )
        base = source.parent
        uses_manifest_dir = False
        dynamic_tail = False
        literal_prefix: list[str] = []
        for argument in _split_top_level_arguments(arguments):
            if _is_cargo_manifest_dir(argument):
                if literal_prefix or uses_manifest_dir:
                    raise CoordinatorError(
                        "validation_copy_compile_time_resource_unresolved",
                        "Compile-time include has an ambiguous manifest-directory prefix",
                        details={"sourcePath": str(source)},
                    )
                base = package_root
                uses_manifest_dir = True
                continue
            literal_argument = _string_argument(argument)
            if literal_argument is None:
                dynamic_tail = True
            elif not dynamic_tail:
                literal_prefix.append(literal_argument)
        if not literal_prefix:
            raise CoordinatorError(
                "validation_copy_compile_time_resource_unresolved",
                "Compile-time include has no repository-local static path prefix",
                details={"sourcePath": str(source)},
            )
        suffix = "".join(literal_prefix)
        if uses_manifest_dir:
            suffix = suffix.lstrip("/\\\\")
        candidate = (base / suffix).resolve()
        if dynamic_tail:
            candidate = (
                candidate if suffix.endswith(("/", "\\")) else candidate.parent
            )
    if not candidate.is_relative_to(repo_root):
        raise CoordinatorError(
            "validation_copy_compile_time_resource_outside_repository",
            "Compile-time include resolves outside the repository",
            details={"sourcePath": str(source), "resourcePath": str(candidate)},
        )
    return candidate


def _compile_time_include_expressions(
    tokens: tuple[_RustToken, ...],
) -> tuple[tuple[_RustToken, ...], ...]:
    return tuple(
        expression
        for expression, _is_path_attribute in _compile_time_include_candidates(tokens)
    )


def _compile_time_include_candidates(
    tokens: tuple[_RustToken, ...],
) -> tuple[tuple[tuple[_RustToken, ...], bool], ...]:
    expressions: list[tuple[tuple[_RustToken, ...], bool]] = []
    for index, token in enumerate(tokens):
        if token[0] != "ident" or token[1] not in _COMPILE_TIME_INCLUDE_MACROS:
            continue
        opening = index + 2
        if (
            opening >= len(tokens)
            or tokens[index + 1][0] != "!"
            or tokens[opening][0] not in {"(", "[", "{"}
        ):
            continue
        closing = _matching_delimiter(tokens, opening)
        if closing is not None:
            expressions.append((tokens[opening + 1 : closing], False))
    brace_depth = 0
    for index in range(len(tokens) - 5):
        if tokens[index][0] == "{":
            brace_depth += 1
            continue
        if tokens[index][0] == "}":
            brace_depth = max(0, brace_depth - 1)
            continue
        if (
            brace_depth == 0
            and
            tokens[index][0] == "#"
            and tokens[index + 1][0] == "["
            and tokens[index + 2] == ("ident", "path")
            and tokens[index + 3][0] == "="
            and tokens[index + 4][0] == "string"
            and tokens[index + 5][0] == "]"
        ):
            expressions.append(((tokens[index + 4],), True))
    return tuple(expressions)


def _cfg_expression_requires_test(tokens: tuple[_RustToken, ...]) -> bool:
    """Return whether a cfg expression can only be true for test builds."""

    if len(tokens) == 1 and tokens[0] == ("ident", "test"):
        return True
    if len(tokens) < 3 or tokens[0][0] != "ident":
        return False
    opening = 1
    if tokens[opening][0] != "(":
        return False
    closing = _matching_delimiter(tokens, opening)
    if closing != len(tokens) - 1:
        return False
    arguments = _split_top_level_arguments(tokens[opening + 1 : closing])
    operator = tokens[0][1]
    if operator == "all":
        return any(_cfg_expression_requires_test(argument) for argument in arguments)
    if operator == "any":
        return bool(arguments) and all(
            _cfg_expression_requires_test(argument) for argument in arguments
        )
    # A negated test predicate is active in non-test builds, so it cannot be
    # treated as test-only for closure planning.
    return False


def _rust_attribute_requires_test(attribute: tuple[_RustToken, ...]) -> bool:
    if not attribute or attribute[0] != ("ident", "cfg"):
        return False
    if len(attribute) < 3 or attribute[1][0] != "(":
        return False
    closing = _matching_delimiter(attribute, 1)
    if closing != len(attribute) - 1:
        return False
    return _cfg_expression_requires_test(attribute[2:closing])


def _rust_out_of_line_modules(
    source_relative: str,
    source_text: str,
    module_directory: str,
    *,
    include_test_modules: bool = True,
    skipped_test_modules: set[str] | None = None,
) -> tuple[tuple[str, str], ...]:
    tokens = _rust_tokens(source_text)
    source_parent = PurePosixPath(source_relative).parent
    found: set[tuple[str, str]] = set()

    def portable(candidate: PurePosixPath) -> str | None:
        normalized = posixpath.normpath(candidate.as_posix())
        if normalized == ".." or normalized.startswith("../"):
            raise CoordinatorError(
                "validation_copy_cargo_target_module_outside_repository",
                "Cargo target module resolves outside the repository",
                details={
                    "sourcePath": source_relative,
                    "modulePath": candidate.as_posix(),
                },
            )
        return normalized

    def record(candidate: PurePosixPath, *, test_only: bool) -> None:
        relative = portable(candidate)
        if relative is None:
            return
        if test_only and not include_test_modules:
            if skipped_test_modules is not None:
                skipped_test_modules.add(relative)
            return
        path = PurePosixPath(relative)
        child_directory = (
            path.parent if path.name == "mod.rs" else path.parent / path.stem
        ).as_posix()
        found.add((relative, child_directory))
        if len(found) > _COMPILE_TIME_SOURCE_LIMIT * 2:
            raise CoordinatorError(
                "validation_copy_compile_time_source_limit",
                "Cargo target module closure exceeded its candidate-file limit",
                details={"sourceLimit": _COMPILE_TIME_SOURCE_LIMIT},
            )

    def scan(
        start: int,
        end: int,
        current_directory: PurePosixPath,
        path_base: PurePosixPath,
    ) -> None:
        direct_path: str | None = None
        conditional_paths: set[str] = set()
        test_only = False
        index = start
        while index < end:
            if (
                tokens[index][0] == "#"
                and index + 1 < end
                and tokens[index + 1][0] == "["
            ):
                closing = _matching_delimiter(tokens, index + 1)
                if closing is not None and closing < end:
                    attribute = tokens[index + 2 : closing]
                    if (
                        len(attribute) == 3
                        and attribute[0] == ("ident", "path")
                        and attribute[1][0] == "="
                        and attribute[2][0] == "string"
                    ):
                        direct_path = attribute[2][1]
                        conditional_paths.clear()
                    elif attribute[:1] == (("ident", "cfg_attr"),):
                        conditional_paths.update(
                            attribute[cursor + 2][1]
                            for cursor in range(len(attribute) - 2)
                            if attribute[cursor] == ("ident", "path")
                            and attribute[cursor + 1][0] == "="
                            and attribute[cursor + 2][0] == "string"
                        )
                    elif _rust_attribute_requires_test(attribute):
                        test_only = True
                    index = closing + 1
                    continue
            if (
                tokens[index] == ("ident", "mod")
                and index + 2 < end
                and tokens[index + 1][0] == "ident"
            ):
                name = tokens[index + 1][1]
                terminator = index + 2
                if tokens[terminator][1] == ";":
                    if direct_path is not None:
                        record(
                            path_base / direct_path.replace("\\", "/"),
                            test_only=test_only,
                        )
                    else:
                        record(
                            current_directory / f"{name}.rs",
                            test_only=test_only,
                        )
                        record(
                            current_directory / name / "mod.rs",
                            test_only=test_only,
                        )
                        for conditional_path in conditional_paths:
                            record(
                                path_base / conditional_path.replace("\\", "/"),
                                test_only=test_only,
                            )
                    direct_path = None
                    conditional_paths.clear()
                    test_only = False
                    index = terminator + 1
                    continue
                if tokens[terminator][0] == "{":
                    closing = _matching_delimiter(tokens, terminator)
                    if closing is not None and closing < end:
                        if direct_path is not None:
                            inline_directories = {
                                path_base / direct_path.replace("\\", "/")
                            }
                        else:
                            inline_directories = {current_directory / name}
                            inline_directories.update(
                                path_base / value.replace("\\", "/")
                                for value in conditional_paths
                            )
                        for inline_directory in inline_directories:
                            if include_test_modules or not test_only:
                                scan(
                                    terminator + 1,
                                    closing,
                                    inline_directory,
                                    inline_directory,
                                )
                        direct_path = None
                        conditional_paths.clear()
                        test_only = False
                        index = closing + 1
                        continue
            if tokens[index][0] in {"(", "[", "{"}:
                closing = _matching_delimiter(tokens, index)
                if closing is not None and closing < end:
                    if include_test_modules or not test_only:
                        scan(index + 1, closing, current_directory, path_base)
                    if tokens[index][0] == "{":
                        direct_path = None
                        conditional_paths.clear()
                        test_only = False
                    index = closing + 1
                    continue
            if tokens[index][1] == ";":
                direct_path = None
                conditional_paths.clear()
                test_only = False
            index += 1

    try:
        scan(
            0,
            len(tokens),
            PurePosixPath(module_directory),
            source_parent,
        )
    finally:
        # The recursive closure otherwise retains every source's tokens until cyclic GC.
        del scan
    return tuple(sorted(found, key=lambda item: (item[0].casefold(), item[1].casefold())))


def _package_root_for_relative_source(
    source: str, package_roots: tuple[str, ...]
) -> str | None:
    candidates = [
        root
        for root in package_roots
        if root == "." or source == root or source.startswith(root.rstrip("/") + "/")
    ]
    if not candidates:
        return None
    return max(candidates, key=len)


class CargoInputClosurePlanner:
    """Derive local Cargo inputs from server-owned metadata and Git trees."""

    def __init__(
        self,
        repo_root: str | Path,
        *,
        metadata_runner: Callable[[tuple[str, ...]], Mapping[str, object]] | None = None,
    ) -> None:
        self.repo_root = Path(repo_root).resolve()
        self.metadata_runner = metadata_runner or self._cargo_metadata

    def plan(
        self,
        command: tuple[str, ...] | list[str],
        *,
        external_sources: tuple[ExternalGitSource, ...] | list[ExternalGitSource] = (),
        discover_external_sources: bool = False,
        overlay_paths: tuple[str, ...] | list[str] = (),
        baseline_commit: str | None = None,
    ) -> CargoInputClosure:
        command_tuple = tuple(str(part) for part in command if str(part))
        include_test_code = cargo_command_includes_test_code(command_tuple)
        package_names = set(cargo_package_specs(command_tuple))
        excluded_package_names = set(cargo_excluded_package_specs(command_tuple))
        metadata = self.metadata_runner(command_tuple)
        packages = {
            str(item["id"]): item
            for item in metadata.get("packages", [])
            if isinstance(item, Mapping) and "id" in item
        }
        workspace_members = {
            str(package_id)
            for package_id in metadata.get("workspace_members", [])
            if str(package_id) in packages
        }
        if package_names:
            selected = [
                package_id
                for package_id, package in packages.items()
                if str(package.get("name")) in package_names
            ]
        elif cargo_selects_workspace(command_tuple):
            selected = [
                package_id
                for package_id in workspace_members
                if str(packages[package_id].get("name"))
                not in excluded_package_names
            ]
        elif cargo_manifest_path_argument(command_tuple) is not None:
            resolve = metadata.get("resolve")
            resolve_root = (
                str(resolve.get("root") or "")
                if isinstance(resolve, Mapping)
                else ""
            )
            selected = [resolve_root] if resolve_root in packages else []
            if not selected:
                raw_manifest = cargo_manifest_path_argument(command_tuple)
                assert raw_manifest is not None
                requested_manifest = Path(raw_manifest)
                if not requested_manifest.is_absolute():
                    requested_manifest = self.repo_root / requested_manifest
                requested_manifest = requested_manifest.resolve()
                selected = [
                    package_id
                    for package_id, package in packages.items()
                    if package.get("manifest_path")
                    and Path(str(package["manifest_path"])).resolve()
                    == requested_manifest
                ]
            if not selected:
                default_members = [
                    str(package_id)
                    for package_id in metadata.get("workspace_default_members", [])
                    if str(package_id) in packages
                ]
                selected = default_members or list(workspace_members or packages)
        else:
            default_members = [
                str(package_id)
                for package_id in metadata.get("workspace_default_members", [])
                if str(package_id) in packages
            ]
            selected = default_members or list(workspace_members or packages)
        if not selected:
            raise CoordinatorError(
                "validation_copy_cargo_target_missing",
                "Cargo metadata did not contain the requested package",
                details={"packages": sorted(package_names)},
            )
        dependency_ids = {
            str(node["id"]): tuple(
                str(dep["pkg"])
                for dep in node.get("deps", [])
                if isinstance(dep, Mapping) and "pkg" in dep
            )
            for node in (metadata.get("resolve") or {}).get("nodes", [])
            if isinstance(node, Mapping) and "id" in node
        }
        build_queue = list(selected)
        build_closure_ids: set[str] = set()
        while build_queue:
            package_id = build_queue.pop()
            if package_id in build_closure_ids:
                continue
            build_closure_ids.add(package_id)
            build_queue.extend(dependency_ids.get(package_id, ()))
        declared_dependency_roots = set(
            self._validation_dependency_roots(
                (
                    packages[package_id]
                    for package_id in build_closure_ids
                    if package_id in packages
                ),
                keys=("dependency-roots", "dependencyRoots"),
                field_name="dependency-roots",
            )
        )
        if (
            is_direct_cargo_command(command_tuple)
            and cargo_subcommand(command_tuple) in {"test", "bench"}
        ):
            declared_dependency_roots.update(
                self._validation_dependency_roots(
                    (
                        packages[package_id]
                        for package_id in selected
                        if package_id in packages
                    ),
                    keys=("test-dependency-roots", "testDependencyRoots"),
                    field_name="test-dependency-roots",
                )
            )

        queue = list(set(selected) | workspace_members)
        closure_ids: set[str] = set()
        while queue:
            package_id = queue.pop()
            if package_id in closure_ids:
                continue
            closure_ids.add(package_id)
            queue.extend(dependency_ids.get(package_id, ()))

        descriptors = tuple(source.pinned() for source in external_sources)
        repository_roots: set[str] = set()
        build_repository_roots: set[str] = set()
        manifest_build_scopes = {
            Path(str(package["manifest_path"])).resolve(): package_id
            in build_closure_ids
            for package_id, package in packages.items()
            if package.get("source") is None and package.get("manifest_path")
        }
        manifest_target_sources = {
            Path(str(package["manifest_path"])).resolve(): cargo_target_sources_for_command(
                package,
                command_tuple,
                selected_package=package_id in selected,
            )
            for package_id, package in packages.items()
            if package.get("source") is None and package.get("manifest_path")
        }
        manifest_test_target_sources = {
            Path(str(package["manifest_path"])).resolve(): cargo_test_target_sources_for_command(
                package,
                command_tuple,
                selected_package=package_id in selected,
            )
            for package_id, package in packages.items()
            if package.get("source") is None and package.get("manifest_path")
        }
        used_external: dict[str, tuple[ExternalGitSource, set[str]]] = {}
        external_tree_cache: dict[tuple[str, str], frozenset[str]] = {}
        external_topology_cache: dict[tuple[str, str, str], set[str]] = {}
        discovered_inputs: dict[Path, dict[Path, bool]] = {}

        def external_include_path(
            manifest: Path, external_root: Path, include_sources: bool
        ) -> str:
            relative_manifest = manifest.relative_to(external_root)
            if not include_sources:
                return relative_manifest.as_posix()
            relative_root = relative_manifest.parent.as_posix()
            if relative_root in {"", "."}:
                return EXTERNAL_REPOSITORY_ROOT
            return relative_root

        def record_external(
            source: ExternalGitSource, manifest: Path, include_sources: bool
        ) -> None:
            key = source.mount_path.casefold()
            entry = used_external.get(key)
            if entry is not None and (
                entry[0].repo_root != source.repo_root
                or entry[0].commit != source.commit
            ):
                raise CoordinatorError(
                    "validation_copy_external_mount_conflict",
                    "External Git sources map different immutable identities to one mount",
                    details={
                        "mountPath": source.mount_path,
                        "existingRepoRoot": str(entry[0].repo_root),
                        "existingCommit": entry[0].commit,
                        "conflictingRepoRoot": str(source.repo_root),
                        "conflictingCommit": source.commit,
                    },
                )
            if entry is None:
                entry = (source, set())
                used_external[key] = entry
            if source.archive_hash is not None:
                entry[1].clear()
                entry[1].add(EXTERNAL_REPOSITORY_ROOT)
                return
            tree_key = (str(source.repo_root), source.commit)
            tracked_paths = external_tree_cache.get(tree_key)
            if tracked_paths is None:
                tracked_paths = external_tree_paths(source)
                external_tree_cache[tree_key] = tracked_paths
            cache_key = (str(source.repo_root), source.commit, str(manifest))
            topology_paths = external_topology_cache.get(cache_key)
            if topology_paths is None:
                topology_paths = external_topology_paths(
                    source, manifest, tracked_paths
                )
                external_topology_cache[cache_key] = topology_paths
            entry[1].update(topology_paths)
            if not include_sources:
                for target_source in manifest_target_sources.get(manifest, ()):
                    if not target_source.is_relative_to(source.repo_root):
                        continue
                    relative_target = target_source.relative_to(
                        source.repo_root
                    ).as_posix()
                    if relative_target in tracked_paths:
                        entry[1].add(relative_target)
            entry[1].add(
                external_include_path(manifest, source.repo_root, include_sources)
            )

        def discover_external(manifest: Path, include_sources: bool) -> None:
            external_root = self._external_git_root(manifest)
            manifests = discovered_inputs.setdefault(external_root, {})
            manifests[manifest] = manifests.get(manifest, False) or include_sources

        for package_id in closure_ids:
            package = packages.get(package_id)
            if package is None:
                continue
            manifest = Path(str(package["manifest_path"])).resolve()
            if manifest.is_relative_to(self.repo_root):
                relative_root = manifest.parent.relative_to(self.repo_root).as_posix()
                repository_roots.add(relative_root or ".")
                if package_id in build_closure_ids:
                    build_repository_roots.add(relative_root or ".")
                continue
            if package.get("source") is not None:
                # Registry and Git packages are fetched by Cargo; only source-null
                # local path packages require validation-copy external mounts.
                continue
            descriptor = next(
                (
                    source
                    for source in descriptors
                    if manifest.is_relative_to(source.repo_root)
                    and external_source_includes_path(
                        source,
                        manifest.relative_to(source.repo_root).as_posix(),
                    )
                ),
                None,
            )
            if descriptor is None:
                if discover_external_sources:
                    discover_external(manifest, package_id in build_closure_ids)
                    continue
                raise CoordinatorError(
                    "validation_copy_external_source_missing",
                    "Cargo local path dependency has no pinned external source descriptor",
                    details={"manifestPath": str(manifest)},
                )
            record_external(descriptor, manifest, package_id in build_closure_ids)

        root_manifest = self.repo_root / "Cargo.toml"
        manifest_queue = [(root_manifest, False)] if root_manifest.is_file() else []
        manifest_queue.extend(
            (
                Path(str(packages[package_id]["manifest_path"])).resolve(),
                package_id in build_closure_ids,
            )
            for package_id in closure_ids
            if package_id in packages
            and Path(str(packages[package_id]["manifest_path"]))
            .resolve()
            .is_relative_to(self.repo_root)
        )
        scanned_manifest_scopes: dict[Path, bool] = {}
        repository_manifests: set[str] = set()
        while manifest_queue:
            manifest, include_sources = manifest_queue.pop()
            previous_scope = scanned_manifest_scopes.get(manifest)
            if previous_scope is True or (
                previous_scope is False and not include_sources
            ):
                continue
            include_sources = bool(previous_scope) or include_sources
            scanned_manifest_scopes[manifest] = include_sources
            if manifest.is_relative_to(self.repo_root):
                repository_manifests.add(
                    manifest.relative_to(self.repo_root).as_posix()
                )
            for dependency_manifest in self._manifest_path_dependencies(manifest):
                dependency_includes_sources = manifest_build_scopes.get(
                    dependency_manifest, include_sources
                )
                if dependency_manifest.is_relative_to(self.repo_root):
                    relative_root = dependency_manifest.parent.relative_to(
                        self.repo_root
                    ).as_posix()
                    repository_roots.add(relative_root or ".")
                    if dependency_includes_sources:
                        build_repository_roots.add(relative_root or ".")
                    manifest_queue.append(
                        (dependency_manifest, dependency_includes_sources)
                    )
                    continue
                descriptor = next(
                    (
                        source
                        for source in descriptors
                        if dependency_manifest.is_relative_to(source.repo_root)
                        and external_source_includes_path(
                            source,
                            dependency_manifest.relative_to(
                                source.repo_root
                            ).as_posix(),
                        )
                    ),
                    None,
                )
                if descriptor is not None:
                    record_external(
                        descriptor,
                        dependency_manifest,
                        dependency_includes_sources,
                    )
                    continue
                if not discover_external_sources:
                    raise CoordinatorError(
                        "validation_copy_external_source_missing",
                        "Cargo manifest path dependency has no pinned external source descriptor",
                        details={"manifestPath": str(dependency_manifest)},
                    )
                discover_external(
                    dependency_manifest,
                    dependency_includes_sources,
                )

        for external_root, manifests in discovered_inputs.items():
            include_roots = {
                external_include_path(manifest, external_root, include_sources)
                for manifest, include_sources in manifests.items()
            }
            descriptor = self._discovered_sibling_source(
                external_root, include_roots
            )
            for manifest, include_sources in manifests.items():
                record_external(descriptor, manifest, include_sources)

        narrowed_external = tuple(
            ExternalGitSource.from_payload(
                {
                    "repoRoot": str(source.repo_root),
                    "commit": source.commit,
                    "mountPath": source.mount_path,
                    "includeRoots": (
                        [EXTERNAL_REPOSITORY_ROOT]
                        if source.archive_hash is not None
                        else sorted(include_roots, key=str.casefold)
                    ),
                    **(
                        {
                            "archiveHash": source.archive_hash,
                            "archiveByteCount": source.archive_byte_count,
                        }
                        if source.archive_hash is not None
                        else {}
                    ),
                }
            )
            for source, include_roots in used_external.values()
        )

        roots = tuple(sorted(build_repository_roots, key=str.casefold))
        if roots:
            tracked_package_paths = self._tracked_git_paths(
                set(roots),
                operation="git_ls_files_cargo_package_sources",
                count_key="packageRootCount",
                error_code="validation_copy_cargo_package_git_failed",
                message="Git could not enumerate Cargo package source files",
                baseline_commit=baseline_commit,
            )
            package_roots = tuple(sorted(repository_roots, key=str.casefold))
            selected_roots = set(build_repository_roots)
            paths = {
                line
                for line in tracked_package_paths
                if line
                and _package_root_for_relative_source(line, package_roots)
                in selected_roots
            }
        else:
            paths = set()
        if repository_manifests:
            paths.update(
                self._tracked_git_paths(
                    repository_manifests,
                    operation="git_ls_files_cargo_manifests",
                    count_key="manifestCount",
                    error_code="validation_copy_cargo_manifest_git_failed",
                    message="Git could not enumerate Cargo manifests",
                    baseline_commit=baseline_commit,
                )
            )
        manifest_target_entrypoints = {
            target_source.relative_to(self.repo_root).as_posix()
            for manifest in scanned_manifest_scopes
            for target_source in manifest_target_sources.get(manifest, ())
            if target_source.is_relative_to(self.repo_root)
        }
        if manifest_target_entrypoints:
            paths.update(
                self._tracked_git_paths(
                    manifest_target_entrypoints,
                    operation="git_ls_files_cargo_target_sources",
                    count_key="targetSourceCount",
                    error_code="validation_copy_cargo_target_git_failed",
                    message="Git could not enumerate Cargo target source files",
                    baseline_commit=baseline_commit,
                )
            )
        target_scope_roots: set[str] = set()
        target_source_owners: dict[str, set[str]] = {}
        target_source_test_owners: dict[str, set[str]] = {}
        for manifest, include_sources in scanned_manifest_scopes.items():
            if not include_sources or not manifest.is_relative_to(self.repo_root):
                continue
            owner = manifest.parent.relative_to(self.repo_root).as_posix() or "."
            for target_source in manifest_target_sources.get(manifest, ()):
                if not target_source.is_relative_to(self.repo_root):
                    continue
                scope = target_source.parent.relative_to(self.repo_root).as_posix() or "."
                relative_target = target_source.relative_to(self.repo_root).as_posix()
                target_scope_roots.add(scope)
                target_source_owners.setdefault(relative_target, set()).add(owner)
                if target_source in manifest_test_target_sources.get(manifest, ()):
                    target_source_test_owners.setdefault(relative_target, set()).add(owner)
        source_package_contexts: dict[str, set[str]] = {}
        skipped_test_sources: set[str] = set()
        if target_scope_roots:
            target_scope_paths = self._tracked_git_paths(
                target_scope_roots,
                operation="git_ls_files_cargo_target_scopes",
                count_key="targetScopeCount",
                error_code="validation_copy_cargo_target_git_failed",
                message="Git could not enumerate Cargo target source scopes",
                baseline_commit=baseline_commit,
            )
            paths.update(target_scope_paths)
            contextual_paths = target_scope_paths | {
                relative
                for relative in overlay_paths
                if (self.repo_root / relative).is_file()
                and any(
                    scope == "."
                    or relative.startswith(scope.rstrip("/") + "/")
                    for scope in target_scope_roots
                )
            }
            source_package_contexts = self._target_source_package_contexts(
                target_source_owners,
                contextual_paths,
                overlay_paths=set(overlay_paths),
                baseline_commit=baseline_commit,
                include_test_code=include_test_code,
                target_source_test_owners=(
                    target_source_test_owners if is_direct_cargo_command(command_tuple) else None
                ),
                skipped_test_sources=skipped_test_sources,
            )
            paths.update(source_package_contexts)
        root_cargo_inputs = {
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
            ".cargo/config",
            ".cargo/config.toml",
            *cargo_config_file_arguments(command_tuple),
        }
        paths.update(
            self._tracked_git_paths(
                root_cargo_inputs,
                operation="git_ls_files_cargo_root_files",
                count_key="rootFileCount",
                error_code="validation_copy_cargo_root_git_failed",
                message="Git could not enumerate Cargo root files",
                baseline_commit=baseline_commit,
            )
        )
        if declared_dependency_roots:
            paths.update(
                self._tracked_git_paths(
                    set(declared_dependency_roots),
                    operation="git_ls_files_cargo_declared_inputs",
                    count_key="dependencyRootCount",
                    error_code="validation_copy_cargo_dependency_root_git_failed",
                    message="Git could not enumerate declared Cargo validation inputs",
                    baseline_commit=baseline_commit,
                )
            )
        overlay_sources = {
            Path(relative).as_posix()
            for relative in overlay_paths
            if (self.repo_root / relative).is_file()
        }
        paths.update(
            self._compile_time_resource_paths(
                paths | overlay_sources,
                repository_roots,
                build_repository_roots,
                source_package_contexts=source_package_contexts,
                overlay_paths=overlay_sources,
                baseline_commit=baseline_commit,
                baseline_paths=paths,
                skipped_test_sources=skipped_test_sources,
            )
        )
        return CargoInputClosure(
            tuple(sorted(paths, key=str.casefold)),
            tuple(
                sorted(narrowed_external, key=lambda item: item.mount_path.casefold())
            ),
        )

    def _validation_dependency_roots(
        self,
        packages: Iterable[Mapping[str, object]],
        *,
        keys: tuple[str, str],
        field_name: str,
    ) -> tuple[str, ...]:
        roots: set[str] = set()
        for package in packages:
            if not isinstance(package, Mapping):
                continue
            manifest_value = package.get("manifest_path")
            if not isinstance(manifest_value, str):
                continue
            manifest = Path(manifest_value).resolve()
            if not manifest.is_relative_to(self.repo_root):
                continue
            metadata = package.get("metadata")
            if not isinstance(metadata, Mapping):
                continue
            zircon = metadata.get("zircon")
            validation = zircon.get("validation") if isinstance(zircon, Mapping) else None
            if not isinstance(validation, Mapping):
                continue
            values = validation.get(keys[0])
            if values is None:
                values = validation.get(keys[1])
            if values is None:
                continue
            if not isinstance(values, list) or any(
                not isinstance(value, str) or not value.strip() for value in values
            ):
                raise CoordinatorError(
                    "validation_copy_cargo_dependency_roots_invalid",
                    f"Cargo package validation {field_name} must be a string array",
                    details={"manifestPath": str(manifest), "field": field_name},
                )
            for value in values:
                candidate = (manifest.parent / value).resolve()
                if not candidate.is_relative_to(self.repo_root):
                    raise CoordinatorError(
                        "validation_copy_cargo_dependency_root_escape",
                        "Cargo validation dependency root must stay inside the repository",
                        details={
                            "manifestPath": str(manifest),
                            "dependencyRoot": value,
                            "field": field_name,
                        },
                    )
                relative = candidate.relative_to(self.repo_root).as_posix()
                if relative in {"", "."}:
                    raise CoordinatorError(
                        "validation_copy_cargo_dependency_root_too_broad",
                        "Cargo validation dependency root cannot select the whole repository",
                        details={"manifestPath": str(manifest), "field": field_name},
                    )
                roots.add(relative)
        return tuple(sorted(roots, key=str.casefold))

    def _target_source_package_contexts(
        self,
        target_source_owners: Mapping[str, set[str]],
        available_sources: set[str],
        *,
        overlay_paths: set[str],
        baseline_commit: str | None,
        include_test_code: bool = True,
        target_source_test_owners: Mapping[str, set[str]] | None = None,
        skipped_test_sources: set[str] | None = None,
    ) -> dict[str, set[str]]:
        available_rust_sources = {
            relative for relative in available_sources if relative.endswith(".rs")
        }
        source_budget = _CompileTimeSourceBudget()
        source_texts: dict[str, str] = {}
        contexts: dict[str, set[str]] = {}
        missing_sources: set[str] = set()
        required_module_sources: set[str] = set(target_source_owners)
        pending: list[tuple[str, str, str, bool]] = []
        for relative, owners in target_source_owners.items():
            if relative not in available_rust_sources:
                continue
            module_directory = PurePosixPath(relative).parent.as_posix()
            for owner in owners:
                test_context = include_test_code and (
                    target_source_test_owners is None
                    or owner in target_source_test_owners.get(relative, ())
                )
                pending.append((relative, owner, module_directory, test_context))
        visited: set[tuple[str, str, str, bool]] = set()
        visited_sources: set[str] = set()
        while pending:
            frontier = [item for item in pending if item not in visited]
            pending = []
            baseline_frontier = {
                relative
                for relative, _owner, _module_directory, _test_context in frontier
                if baseline_commit
                and relative not in overlay_paths
                and relative not in source_texts
            }
            if baseline_commit and baseline_frontier:
                source_texts.update(
                    self._baseline_texts_at_commit(
                        baseline_commit,
                        baseline_frontier,
                        byte_budget=source_budget,
                    )
                )
            candidate_modules: list[tuple[str, str, str, bool]] = []
            for relative, owner, module_directory, test_context in frontier:
                visit = (relative, owner, module_directory, test_context)
                if visit in visited:
                    continue
                visited.add(visit)
                if relative not in visited_sources:
                    if len(visited_sources) >= _COMPILE_TIME_SOURCE_LIMIT:
                        raise CoordinatorError(
                            "validation_copy_compile_time_source_limit",
                            "Cargo target module closure exceeded its source-file limit",
                            details={"sourceLimit": _COMPILE_TIME_SOURCE_LIMIT},
                        )
                    visited_sources.add(relative)
                contexts.setdefault(relative, set()).add(owner)
                text = source_texts.get(relative)
                if text is None:
                    source = self.repo_root / relative
                    if not source.is_file():
                        continue
                    size = source.stat().st_size
                    source_budget.account(relative, size)
                    content = source.read_bytes()
                    if len(content) != size:
                        raise CoordinatorError(
                            "validation_copy_compile_time_source_changed",
                            "Compile-time Rust source changed while its closure was planned",
                            details={"sourcePath": relative},
                        )
                    try:
                        text = content.decode("utf-8")
                    except UnicodeDecodeError as error:
                        raise CoordinatorError(
                            "validation_copy_compile_time_source_invalid",
                            "Compile-time Rust source is not valid UTF-8",
                            details={"sourcePath": relative},
                        ) from error
                    source_texts[relative] = text
                for module_source, child_directory in _rust_out_of_line_modules(
                    relative,
                    text,
                    module_directory,
                    include_test_modules=test_context,
                    skipped_test_modules=skipped_test_sources,
                ):
                    required_module_sources.add(module_source)
                    candidate_modules.append(
                        (module_source, owner, child_directory, test_context)
                    )
            unresolved = {
                module_source
                for module_source, _owner, _child_directory, _test_context in candidate_modules
                if module_source not in available_rust_sources
                and module_source not in missing_sources
            }
            overlay_sources = {
                relative
                for relative in unresolved
                if relative in overlay_paths
                and (self.repo_root / relative).is_file()
            }
            git_candidates = unresolved - overlay_sources
            tracked_candidates = (
                self._tracked_git_paths(
                    git_candidates,
                    operation="git_ls_files_cargo_target_modules",
                    count_key="moduleCandidateCount",
                    error_code="validation_copy_cargo_target_git_failed",
                    message="Git could not resolve Cargo target modules",
                    baseline_commit=baseline_commit,
                )
                if git_candidates
                else set()
            )
            available_rust_sources.update(overlay_sources)
            available_rust_sources.update(
                relative
                for relative in tracked_candidates
                if relative.endswith(".rs")
            )
            missing_sources.update(unresolved - available_rust_sources)
            pending.extend(
                item
                for item in candidate_modules
                if item[0] in available_rust_sources
            )
        if skipped_test_sources is not None:
            skipped_test_sources.difference_update(required_module_sources)
        return contexts

    def _compile_time_resource_paths(
        self,
        tracked_paths: set[str],
        package_roots: set[str],
        selected_package_roots: set[str],
        *,
        source_package_contexts: Mapping[str, set[str]] | None = None,
        overlay_paths: set[str] | frozenset[str] = frozenset(),
        baseline_commit: str | None = None,
        baseline_paths: set[str] | None = None,
        skipped_test_sources: set[str] | frozenset[str] = frozenset(),
    ) -> set[str]:
        roots = tuple(sorted(package_roots, key=str.casefold))
        selected_roots = set(selected_package_roots)
        explicit_contexts = {
            relative: tuple(sorted(contexts, key=str.casefold))
            for relative, contexts in (source_package_contexts or {}).items()
            if contexts
        }
        resource_sources: dict[str, str] = {}
        resource_contexts: dict[str, set[str]] = {}
        overlay_source_resources: set[str] = set()
        scanned_sources: set[tuple[str, str]] = set()

        def register_expressions(
            relative: str,
            package_root_relative: str,
            source_text: str,
            *,
            overlay_source: bool,
        ) -> None:
            source = self.repo_root / relative
            package_root = (
                self.repo_root
                if package_root_relative == "."
                else self.repo_root / package_root_relative
            )
            for expression, is_path_attribute in _compile_time_include_candidates(
                _rust_tokens(source_text)
            ):
                if _is_build_output_resource(expression):
                    continue
                resource = _compile_time_resource(
                    expression,
                    source=source,
                    package_root=package_root,
                    repo_root=self.repo_root,
                )
                resource_root = resource.relative_to(self.repo_root).as_posix()
                if is_path_attribute and resource_root in skipped_test_sources:
                    continue
                resource_sources.setdefault(resource_root, str(source))
                resource_contexts.setdefault(resource_root, set()).add(
                    package_root_relative
                )
                if overlay_source:
                    overlay_source_resources.add(resource_root)

        def materialize_resource_roots(resource_roots: set[str]) -> set[str]:
            if not resource_roots:
                return set()
            materialized = self._tracked_compile_time_resources(
                resource_roots, baseline_commit=baseline_commit
            )
            for resource_root in resource_roots:
                descendant_prefix = resource_root.rstrip("/") + "/"
                if resource_root in overlay_source_resources and (
                    self.repo_root / resource_root
                ).exists():
                    materialized.add(resource_root)
                materialized.update(
                    path
                    for path in overlay_paths
                    if path == resource_root or path.startswith(descendant_prefix)
                )
            ordered = sorted(materialized)
            for resource_root in resource_roots:
                descendant_prefix = resource_root.rstrip("/") + "/"
                resource_index = bisect_left(ordered, resource_root)
                if resource_index < len(ordered) and (
                    ordered[resource_index] == resource_root
                    or ordered[resource_index].startswith(descendant_prefix)
                ):
                    continue
                raise CoordinatorError(
                    "validation_copy_compile_time_resource_missing",
                    "Compile-time include resource is unavailable",
                    details={
                        "sourcePath": resource_sources[resource_root],
                        "resourcePath": str(
                            (self.repo_root / resource_root).resolve()
                        ),
                    },
                )
            return materialized

        baseline_sources = {
            relative
            for relative in (
                baseline_paths if baseline_paths is not None else tracked_paths
            )
            if relative.endswith(".rs") and relative not in skipped_test_sources
        }
        if baseline_commit:
            contextual_baseline_sources = baseline_sources & set(explicit_contexts)
            non_contextual_baseline_sources = (
                set()
                if explicit_contexts
                else baseline_sources - contextual_baseline_sources
            )
            baseline_resources = self._baseline_compile_time_resources_by_source(
                baseline_commit,
                non_contextual_baseline_sources,
                package_roots,
                selected_package_roots,
            )
            for relative, resource_roots in baseline_resources.items():
                if relative in overlay_paths:
                    continue
                package_root_relative = _package_root_for_relative_source(
                    relative, roots
                )
                if package_root_relative is None:
                    continue
                scanned_sources.add((relative, package_root_relative))
                for resource_root in resource_roots:
                    resource_sources.setdefault(
                        resource_root, str(self.repo_root / relative)
                    )
                    resource_contexts.setdefault(resource_root, set()).add(
                        package_root_relative
                    )
            contextual_candidates = self._baseline_compile_time_source_paths(
                baseline_commit, contextual_baseline_sources
            )
            contextual_texts = self._baseline_texts_at_commit(
                baseline_commit, contextual_candidates
            )
            for relative in sorted(contextual_candidates, key=str.casefold):
                if relative in overlay_paths:
                    continue
                for package_root_relative in explicit_contexts[relative]:
                    scanned_sources.add((relative, package_root_relative))
                    register_expressions(
                        relative,
                        package_root_relative,
                        contextual_texts[relative],
                        overlay_source=False,
                    )

        live_sources = (
            {relative for relative in tracked_paths if relative.endswith(".rs")}
            if baseline_commit is None
            else set(overlay_paths)
        )
        for relative in sorted(live_sources, key=str.casefold):
            if not relative.endswith(".rs"):
                continue
            if relative in skipped_test_sources:
                continue
            contexts = explicit_contexts.get(relative)
            if contexts is None:
                if explicit_contexts:
                    continue
                package_root_relative = _package_root_for_relative_source(
                    relative, roots
                )
                contexts = (
                    (package_root_relative,)
                    if package_root_relative is not None
                    and package_root_relative in selected_roots
                    else ()
                )
            if not contexts:
                continue
            source = self.repo_root / relative
            if not source.is_file():
                continue
            source_text = source.read_text(encoding="utf-8")
            for package_root_relative in contexts:
                scanned_sources.add((relative, package_root_relative))
                register_expressions(
                    relative,
                    package_root_relative,
                    source_text,
                    overlay_source=relative in overlay_paths,
                )
        if not resource_sources:
            return set()
        materialized_roots: set[str] = set(resource_sources)
        resources = materialize_resource_roots(materialized_roots)
        expanded_contexts: set[tuple[str, str]] = set()
        while True:
            pending: set[tuple[str, str]] = set()
            for resource_root, contexts in resource_contexts.items():
                for package_root_relative in contexts:
                    context = (resource_root, package_root_relative)
                    if context in expanded_contexts:
                        continue
                    expanded_contexts.add(context)
                    descendant_prefix = resource_root.rstrip("/") + "/"
                    pending.update(
                        (relative, package_root_relative)
                        for relative in resources
                        if relative.endswith(".rs")
                        and (
                            relative == resource_root
                            or relative.startswith(descendant_prefix)
                        )
                        and (relative, package_root_relative)
                        not in scanned_sources
                    )
            if not pending:
                break
            if len(scanned_sources) + len(pending) > _COMPILE_TIME_SOURCE_LIMIT:
                raise CoordinatorError(
                    "validation_copy_compile_time_source_limit",
                    "Compile-time include closure exceeded its source-file limit",
                    details={"sourceLimit": _COMPILE_TIME_SOURCE_LIMIT},
                )
            baseline_pending = {
                relative
                for relative, _package_root in pending
                if baseline_commit and relative not in overlay_paths
            }
            baseline_texts = (
                self._baseline_texts_at_commit(baseline_commit, baseline_pending)
                if baseline_commit and baseline_pending
                else {}
            )
            prior_roots = set(resource_sources)
            for relative, package_root_relative in sorted(
                pending, key=lambda item: (item[0].casefold(), item[1].casefold())
            ):
                scanned_sources.add((relative, package_root_relative))
                if relative in baseline_texts:
                    source_text = baseline_texts[relative]
                else:
                    source = self.repo_root / relative
                    if not source.is_file():
                        continue
                    source_text = source.read_text(encoding="utf-8")
                register_expressions(
                    relative,
                    package_root_relative,
                    source_text,
                    overlay_source=relative in overlay_paths,
                )
            new_roots = set(resource_sources) - prior_roots
            resources.update(materialize_resource_roots(new_roots))
            materialized_roots.update(new_roots)
        return resources

    def _baseline_compile_time_resources_by_source(
        self,
        baseline_commit: str,
        sources: set[str],
        package_roots: set[str],
        selected_package_roots: set[str],
    ) -> dict[str, tuple[str, ...]]:
        ordered_sources = tuple(sorted(sources, key=str.casefold))
        roots = tuple(sorted(package_roots, key=str.casefold))
        selected_roots = tuple(sorted(selected_package_roots, key=str.casefold))
        cache_key = (
            str(self.repo_root),
            baseline_commit,
            ordered_sources,
            roots,
            selected_roots,
        )
        while True:
            with _BASELINE_COMPILE_TIME_CACHE_LOCK:
                cached = _BASELINE_COMPILE_TIME_CACHE.get(cache_key)
                if cached is not None:
                    _BASELINE_COMPILE_TIME_CACHE.move_to_end(cache_key)
                    return dict(cached)
                pending = _BASELINE_COMPILE_TIME_INFLIGHT.get(cache_key)
                if pending is None:
                    pending = threading.Event()
                    _BASELINE_COMPILE_TIME_INFLIGHT[cache_key] = pending
                    break
            pending.wait()

        try:
            candidates = self._baseline_compile_time_source_paths(
                baseline_commit, set(ordered_sources)
            )
            resources_by_source: dict[str, tuple[str, ...]] = {}
            selected_root_set = set(selected_roots)
            ordered_candidates = tuple(sorted(candidates, key=str.casefold))
            source_budget = _CompileTimeSourceBudget()
            for offset in range(0, len(ordered_candidates), _COMPILE_TIME_SOURCE_BATCH_SIZE):
                batch = ordered_candidates[offset : offset + _COMPILE_TIME_SOURCE_BATCH_SIZE]
                baseline_texts = self._baseline_texts_at_commit(
                    baseline_commit, set(batch), byte_budget=source_budget
                )
                for relative in batch:
                    package_root_relative = _package_root_for_relative_source(
                        relative, roots
                    )
                    if (
                        package_root_relative is None
                        or package_root_relative not in selected_root_set
                    ):
                        continue
                    source = self.repo_root / relative
                    package_root = (
                        self.repo_root
                        if package_root_relative == "."
                        else self.repo_root / package_root_relative
                    )
                    resource_roots = {
                        _compile_time_resource(
                            expression,
                            source=source,
                            package_root=package_root,
                            repo_root=self.repo_root,
                        )
                        .relative_to(self.repo_root)
                        .as_posix()
                        for expression in _compile_time_include_expressions(
                            _rust_tokens(baseline_texts[relative])
                        )
                        if not _is_build_output_resource(expression)
                    }
                    resources_by_source[relative] = tuple(
                        sorted(resource_roots, key=str.casefold)
                    )
                del baseline_texts
            frozen = tuple(
                sorted(resources_by_source.items(), key=lambda item: item[0].casefold())
            )
        except BaseException:
            with _BASELINE_COMPILE_TIME_CACHE_LOCK:
                _BASELINE_COMPILE_TIME_INFLIGHT.pop(cache_key, pending).set()
            raise

        with _BASELINE_COMPILE_TIME_CACHE_LOCK:
            _BASELINE_COMPILE_TIME_CACHE[cache_key] = frozen
            _BASELINE_COMPILE_TIME_CACHE.move_to_end(cache_key)
            while (
                len(_BASELINE_COMPILE_TIME_CACHE)
                > _BASELINE_COMPILE_TIME_CACHE_LIMIT
            ):
                _BASELINE_COMPILE_TIME_CACHE.popitem(last=False)
            _BASELINE_COMPILE_TIME_INFLIGHT.pop(cache_key, pending).set()
        return dict(frozen)

    def _baseline_compile_time_source_paths(
        self, baseline_commit: str, paths: set[str]
    ) -> set[str]:
        if not paths:
            return set()
        try:
            result = subprocess.run(
                trusted_git_command(
                    self.repo_root,
                    "grep",
                    "-l",
                    "-e",
                    "include",
                    "-e",
                    "path",
                    baseline_commit,
                    "--",
                    "*.rs",
                ),
                cwd=self.repo_root,
                check=False,
                capture_output=True,
                encoding="utf-8",
            )
        except (OSError, subprocess.SubprocessError) as error:
            raise CoordinatorError(
                "validation_copy_compile_time_source_git_failed",
                "Git could not identify compile-time sources in the pinned baseline",
                details={
                    "operation": "git_grep_compile_time_sources",
                    "errorType": type(error).__name__,
                    "baselineCommit": baseline_commit,
                    "sourceCount": len(paths),
                },
            ) from error
        if result.returncode not in {0, 1}:
            raise CoordinatorError(
                "validation_copy_compile_time_source_git_failed",
                "Git could not identify compile-time sources in the pinned baseline",
                details={
                    "operation": "git_grep_compile_time_sources",
                    "exitCode": int(result.returncode),
                    "baselineCommit": baseline_commit,
                    "sourceCount": len(paths),
                },
            )
        candidates = {
            line.partition(":")[2]
            for line in result.stdout.splitlines()
            if ":" in line
        }
        return candidates & paths

    def _baseline_texts_at_commit(
        self,
        baseline_commit: str,
        paths: set[str],
        *,
        byte_budget: _CompileTimeSourceBudget | None = None,
    ) -> dict[str, str]:
        ordered_paths = tuple(sorted(paths, key=str.casefold))
        if not ordered_paths:
            return {}
        budget = byte_budget or _CompileTimeSourceBudget()
        with tempfile.TemporaryFile() as error_stream:
            process: subprocess.Popen[bytes] | None = None
            try:
                process = subprocess.Popen(
                    trusted_git_command(self.repo_root, "cat-file", "--batch"),
                    cwd=self.repo_root,
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=error_stream,
                )
                if process.stdin is None or process.stdout is None:
                    raise OSError("Git cat-file did not expose binary pipes")
                texts: dict[str, str] = {}
                for relative in ordered_paths:
                    query = f"{baseline_commit}:{relative}\n".encode("utf-8")
                    process.stdin.write(query)
                    process.stdin.flush()
                    header = process.stdout.readline(4097)
                    if not header.endswith(b"\n") or len(header) > 4096:
                        raise ValueError(f"invalid baseline header for {relative}")
                    header = header[:-1]
                    if header.endswith(b" missing"):
                        raise ValueError(f"missing baseline blob for {relative}")
                    _object_name, object_type, size_text = header.rsplit(b" ", 2)
                    if object_type != b"blob":
                        raise ValueError(
                            f"baseline object for {relative} is not a blob"
                        )
                    size = int(size_text)
                    budget.account(relative, size)
                    content = process.stdout.read(size)
                    if len(content) != size or process.stdout.read(1) != b"\n":
                        raise ValueError(f"truncated baseline blob for {relative}")
                    texts[relative] = content.decode("utf-8")
                process.stdin.close()
                process.wait()
                if process.returncode != 0:
                    raise subprocess.CalledProcessError(
                        process.returncode,
                        trusted_git_command(self.repo_root, "cat-file", "--batch"),
                    )
                return texts
            except CoordinatorError:
                if process is not None and process.poll() is None:
                    process.kill()
                    process.wait()
                raise
            except (
                OSError,
                subprocess.SubprocessError,
                UnicodeDecodeError,
                ValueError,
            ) as error:
                if process is not None and process.poll() is None:
                    process.kill()
                    process.wait()
                details: dict[str, object] = {
                    "operation": "git_cat_file_compile_time_sources",
                    "errorType": type(error).__name__,
                    "baselineCommit": baseline_commit,
                    "sourceCount": len(ordered_paths),
                }
                if isinstance(error, subprocess.CalledProcessError):
                    details["exitCode"] = int(error.returncode)
                error_stream.seek(0)
                stderr = error_stream.read().decode("utf-8", errors="replace")
                if stderr:
                    details["stderr"] = stderr[-4096:]
                raise CoordinatorError(
                    "validation_copy_compile_time_source_git_failed",
                    "Git could not read compile-time sources from the pinned baseline",
                    details=details,
                ) from error
            finally:
                if process is not None:
                    if process.stdin is not None and not process.stdin.closed:
                        process.stdin.close()
                    if process.stdout is not None:
                        process.stdout.close()

    def _tracked_compile_time_resources(
        self, resource_roots: set[str], *, baseline_commit: str | None = None
    ) -> set[str]:
        return self._tracked_git_paths(
            resource_roots,
            operation="git_ls_files_compile_time_resources",
            count_key="resourceRootCount",
            error_code="validation_copy_compile_time_resource_git_failed",
            message="Git could not enumerate compile-time resources",
            baseline_commit=baseline_commit,
        )

    def _tracked_git_paths(
        self,
        pathspecs: set[str],
        *,
        operation: str,
        count_key: str,
        error_code: str,
        message: str,
        baseline_commit: str | None = None,
    ) -> set[str]:
        ordered_pathspecs = tuple(sorted(pathspecs, key=str.casefold))
        batches: list[tuple[str, ...]] = []
        batch: list[str] = []
        command_prefix = tuple(
            trusted_git_command(
                self.repo_root,
                *(
                    ("ls-tree", "-r", "--name-only", baseline_commit, "--")
                    if baseline_commit
                    else ("ls-files", "--")
                ),
            )
        )
        prefix_length = len(subprocess.list2cmdline(command_prefix))
        batch_length = prefix_length
        for pathspec in ordered_pathspecs:
            argument_length = len(subprocess.list2cmdline((pathspec,))) + 1
            if (
                batch
                and batch_length + argument_length > _GIT_PATHSPEC_COMMAND_CHAR_LIMIT
            ):
                batches.append(tuple(batch))
                batch = []
                batch_length = prefix_length
            batch.append(pathspec)
            batch_length += argument_length
        if batch:
            batches.append(tuple(batch))

        tracked: set[str] = set()
        try:
            for roots in batches:
                arguments = (
                    ("ls-tree", "-r", "--name-only", baseline_commit, "--", *roots)
                    if baseline_commit
                    else ("ls-files", "--", *roots)
                )
                command = trusted_git_command(self.repo_root, *arguments)
                result = subprocess.run(
                    command,
                    cwd=self.repo_root,
                    check=True,
                    capture_output=True,
                    encoding="utf-8",
                )
                tracked.update(line for line in result.stdout.splitlines() if line)
        except (OSError, subprocess.SubprocessError) as error:
            details: dict[str, object] = {
                "operation": operation,
                "errorType": type(error).__name__,
                count_key: len(ordered_pathspecs),
            }
            for name in ("errno", "winerror"):
                value = getattr(error, name, None)
                if isinstance(value, int):
                    details[name] = value
            if isinstance(error, subprocess.CalledProcessError):
                details["exitCode"] = int(error.returncode)
            raise CoordinatorError(
                error_code,
                message,
                details=details,
            ) from error
        return tracked

    def _manifest_path_dependencies(self, manifest: Path) -> tuple[Path, ...]:
        if not manifest.is_file():
            raise CoordinatorError(
                "validation_copy_cargo_manifest_path_missing",
                "Cargo path dependency manifest is unavailable",
                details={"manifestPath": str(manifest)},
            )
        try:
            document = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise CoordinatorError(
                "validation_copy_cargo_manifest_invalid",
                "Cargo manifest could not be inspected for local path dependencies",
                details={"manifestPath": str(manifest)},
            ) from error
        dependencies: set[Path] = set()

        def visit(node: Mapping[str, object]) -> None:
            for key, value in node.items():
                if not isinstance(value, Mapping):
                    continue
                if key in {"dependencies", "dev-dependencies", "build-dependencies"}:
                    for specification in value.values():
                        if isinstance(specification, Mapping) and specification.get("path"):
                            dependencies.add(
                                (manifest.parent / str(specification["path"])).resolve()
                                / "Cargo.toml"
                            )
                    continue
                visit(value)

        visit(document)
        return tuple(sorted(dependencies, key=lambda path: str(path).casefold()))

    def _external_git_root(self, manifest: Path) -> Path:
        result = subprocess.run(
            trusted_git_command(self.repo_root, "rev-parse", "--show-toplevel"),
            cwd=manifest.parent,
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_external_source_missing",
                "Cargo local path dependency is not in a discoverable Git repository",
                details={"manifestPath": str(manifest)},
            )
        external_root = Path(result.stdout.strip()).resolve()
        if (
            not manifest.is_relative_to(external_root)
            or external_root.parent != self.repo_root.parent
            or external_root == self.repo_root
        ):
            raise CoordinatorError(
                "validation_copy_external_source_missing",
                "Automatic Cargo source discovery is restricted to sibling Git repositories",
                details={"manifestPath": str(manifest), "repoRoot": str(external_root)},
            )
        return external_root

    def _discovered_sibling_source(
        self, external_root: Path, package_roots: set[str]
    ) -> ExternalGitSource:
        head = subprocess.run(
            trusted_git_command(self.repo_root, "rev-parse", "HEAD"),
            cwd=external_root,
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        if head.returncode != 0 or not head.stdout.strip():
            raise CoordinatorError(
                "validation_copy_external_source_missing",
                "Discovered sibling Git repository has no pinnable HEAD",
                details={"repoRoot": str(external_root)},
            )
        commit = head.stdout.strip()
        include_roots = set(package_roots)
        return ExternalGitSource.from_payload(
            {
                "repoRoot": str(external_root),
                "commit": commit,
                "mountPath": external_root.name,
                "includeRoots": sorted(include_roots, key=str.casefold),
            }
        ).pinned()

    def _cargo_metadata(self, _command: tuple[str, ...]) -> Mapping[str, object]:
        result = subprocess.run(
            bind_trusted_cargo(
                ("cargo", "metadata", "--format-version", "1", "--locked"),
                self.repo_root,
                working_directory=self.repo_root,
            ),
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_cargo_metadata_failed",
                "Cargo metadata failed before validation-copy materialization",
                details={"stderr": result.stderr[-4096:]},
            )
        return json.loads(result.stdout)

    @staticmethod
    def _package_name(command: tuple[str, ...]) -> str | None:
        for index, part in enumerate(command):
            if part in {"-p", "--package"} and index + 1 < len(command):
                return command[index + 1]
            if part.startswith("--package="):
                return part.partition("=")[2]
        return None
