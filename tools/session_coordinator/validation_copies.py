from __future__ import annotations

import hashlib
import json
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Callable, Mapping, Sequence

from .models import CoordinatorError


def _relative_path(value: object, *, code: str) -> str:
    if not isinstance(value, str):
        raise CoordinatorError(code, "Validation source path must be text")
    normalized = value.strip().replace("\\", "/").strip("/")
    path = PurePosixPath(normalized)
    if not normalized or path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise CoordinatorError(code, "Validation source path must be a safe relative path")
    return path.as_posix()


@dataclass(frozen=True, slots=True)
class ExternalGitSource:
    repo_root: Path
    commit: str
    mount_path: str
    include_roots: tuple[str, ...]
    source_hash: str

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> "ExternalGitSource":
        required = {
            "repoRoot",
            "commit",
            "mountPath",
            "includeRoots",
        }
        if (
            not isinstance(payload, Mapping)
            or not required <= set(payload)
            or not set(payload) <= required | {"sourceHash"}
        ):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source requires repoRoot, commit, mountPath, and includeRoots",
            )
        repo_root = Path(str(payload["repoRoot"])).resolve()
        commit = str(payload["commit"]).strip()
        mount_path = _relative_path(
            payload["mountPath"], code="validation_copy_external_mount_escape"
        )
        raw_roots = payload["includeRoots"]
        if not isinstance(raw_roots, Sequence) or isinstance(raw_roots, (str, bytes)):
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git includeRoots must be a non-empty list",
            )
        include_roots = tuple(
            sorted(
                {
                    _relative_path(
                        root, code="validation_copy_external_include_root_invalid"
                    )
                    for root in raw_roots
                },
                key=str.casefold,
            )
        )
        if not include_roots:
            raise CoordinatorError(
                "validation_copy_external_source_invalid",
                "External Git source must declare include roots",
            )
        source_hash = hashlib.sha256(
            json.dumps(
                {
                    "repoRoot": str(repo_root),
                    "commit": commit,
                    "mountPath": mount_path,
                    "includeRoots": include_roots,
                },
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()
        return cls(repo_root, commit, mount_path, include_roots, source_hash)

    def pinned(self) -> "ExternalGitSource":
        if not self.repo_root.is_dir():
            raise CoordinatorError(
                "validation_copy_external_repository_missing",
                "External Git repository root does not exist",
                details={"repoRoot": str(self.repo_root)},
            )
        result = subprocess.run(
            ["git", "rev-parse", "--verify", f"{self.commit}^{{commit}}"],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            raise CoordinatorError(
                "validation_copy_external_commit_missing",
                "External Git source commit is unavailable",
                details={"repoRoot": str(self.repo_root), "commit": self.commit},
            )
        commit = result.stdout.strip()
        payload = {
            "repoRoot": str(self.repo_root),
            "commit": commit,
            "mountPath": self.mount_path,
            "includeRoots": list(self.include_roots),
        }
        return ExternalGitSource.from_payload(payload)

    def to_payload(self) -> dict[str, object]:
        return {
            "repoRoot": str(self.repo_root),
            "commit": self.commit,
            "mountPath": self.mount_path,
            "includeRoots": list(self.include_roots),
            "sourceHash": self.source_hash,
        }


@dataclass(frozen=True, slots=True)
class CargoInputClosure:
    repository_paths: tuple[str, ...]
    external_sources: tuple[ExternalGitSource, ...]


_COMPILE_TIME_INCLUDE_MACROS = frozenset({"include_bytes", "include_str"})
_CARGO_MANIFEST_DIR = "CARGO_MANIFEST_DIR"
_GIT_PATHSPEC_COMMAND_CHAR_LIMIT = 24_000
_RustToken = tuple[str, str]


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
        if current in "()!,":
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


def _matching_parenthesis(tokens: tuple[_RustToken, ...], opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(tokens)):
        kind, _value = tokens[index]
        if kind == "(":
            depth += 1
        elif kind == ")":
            depth -= 1
            if depth == 0:
                return index
    return None


def _macro_arguments(
    tokens: tuple[_RustToken, ...], name: str
) -> tuple[_RustToken, ...] | None:
    if len(tokens) < 4 or tokens[0] != ("ident", name) or tokens[1][0] != "!":
        return None
    if tokens[2][0] != "(":
        return None
    closing = _matching_parenthesis(tokens, 2)
    if closing != len(tokens) - 1:
        return None
    return tokens[3:closing]


def _split_top_level_arguments(tokens: tuple[_RustToken, ...]) -> tuple[tuple[_RustToken, ...], ...]:
    arguments: list[tuple[_RustToken, ...]] = []
    start = 0
    depth = 0
    for index, (kind, _value) in enumerate(tokens):
        if kind == "(":
            depth += 1
        elif kind == ")":
            depth -= 1
        elif kind == "," and depth == 0:
            arguments.append(tokens[start:index])
            start = index + 1
    if start < len(tokens):
        arguments.append(tokens[start:])
    return tuple(argument for argument in arguments if argument)


def _string_argument(tokens: tuple[_RustToken, ...]) -> str | None:
    if len(tokens) == 1 and tokens[0][0] == "string":
        return tokens[0][1]
    return None


def _is_cargo_manifest_dir(tokens: tuple[_RustToken, ...]) -> bool:
    arguments = _macro_arguments(tokens, "env")
    return arguments is not None and _string_argument(arguments) == _CARGO_MANIFEST_DIR


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
            candidate = candidate if candidate.is_dir() else candidate.parent
    if not candidate.is_relative_to(repo_root):
        raise CoordinatorError(
            "validation_copy_compile_time_resource_outside_repository",
            "Compile-time include resolves outside the repository",
            details={"sourcePath": str(source), "resourcePath": str(candidate)},
        )
    if not candidate.exists():
        raise CoordinatorError(
            "validation_copy_compile_time_resource_missing",
            "Compile-time include resource is unavailable",
            details={"sourcePath": str(source), "resourcePath": str(candidate)},
        )
    return candidate


def _compile_time_include_expressions(
    tokens: tuple[_RustToken, ...],
) -> tuple[tuple[_RustToken, ...], ...]:
    expressions: list[tuple[_RustToken, ...]] = []
    for index, token in enumerate(tokens):
        if token[0] != "ident" or token[1] not in _COMPILE_TIME_INCLUDE_MACROS:
            continue
        opening = index + 2
        if opening >= len(tokens) or tokens[index + 1][0] != "!" or tokens[opening][0] != "(":
            continue
        closing = _matching_parenthesis(tokens, opening)
        if closing is not None:
            expressions.append(tokens[opening + 1 : closing])
    return tuple(expressions)


def _package_root_for_source(source: Path, package_roots: tuple[Path, ...]) -> Path | None:
    candidates = [root for root in package_roots if source.is_relative_to(root)]
    if not candidates:
        return None
    return max(candidates, key=lambda root: len(root.parts))


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
    ) -> CargoInputClosure:
        command_tuple = tuple(str(part) for part in command if str(part))
        package_name = self._package_name(command_tuple)
        metadata = self.metadata_runner(command_tuple)
        packages = {
            str(item["id"]): item
            for item in metadata.get("packages", [])
            if isinstance(item, Mapping) and "id" in item
        }
        selected = [
            package_id
            for package_id, package in packages.items()
            if package_name is None or package.get("name") == package_name
        ]
        if not selected:
            raise CoordinatorError(
                "validation_copy_cargo_target_missing",
                "Cargo metadata did not contain the requested package",
                details={"package": package_name},
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
        workspace_members = {
            str(package_id)
            for package_id in metadata.get("workspace_members", [])
            if str(package_id) in packages
        }
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
        used_external: dict[str, ExternalGitSource] = {}
        discovered_roots: dict[Path, set[str]] = {}
        for package_id in closure_ids:
            package = packages.get(package_id)
            if package is None:
                continue
            manifest = Path(str(package["manifest_path"])).resolve()
            if manifest.is_relative_to(self.repo_root):
                relative_root = manifest.parent.relative_to(self.repo_root).as_posix()
                repository_roots.add(relative_root or ".")
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
                    and any(
                        manifest.relative_to(source.repo_root).as_posix() == root
                        or manifest.relative_to(source.repo_root).as_posix().startswith(root + "/")
                        for root in source.include_roots
                    )
                ),
                None,
            )
            if descriptor is None:
                if discover_external_sources:
                    external_root = self._external_git_root(manifest)
                    relative_root = manifest.parent.relative_to(external_root).as_posix()
                    if relative_root in {"", "."}:
                        raise CoordinatorError(
                            "validation_copy_external_source_layout_unsupported",
                            "Discovered external Cargo package must be below its sibling Git root",
                            details={"manifestPath": str(manifest)},
                        )
                    discovered_roots.setdefault(external_root, set()).add(relative_root)
                    continue
                raise CoordinatorError(
                    "validation_copy_external_source_missing",
                    "Cargo local path dependency has no pinned external source descriptor",
                    details={"manifestPath": str(manifest)},
                )
            used_external[descriptor.mount_path.casefold()] = descriptor

        root_manifest = self.repo_root / "Cargo.toml"
        manifest_queue = [root_manifest] if root_manifest.is_file() else []
        manifest_queue.extend(
            Path(str(packages[package_id]["manifest_path"])).resolve()
            for package_id in closure_ids
            if package_id in packages
            and Path(str(packages[package_id]["manifest_path"]))
            .resolve()
            .is_relative_to(self.repo_root)
        )
        scanned_manifests: set[Path] = set()
        while manifest_queue:
            manifest = manifest_queue.pop()
            if manifest in scanned_manifests:
                continue
            scanned_manifests.add(manifest)
            for dependency_manifest in self._manifest_path_dependencies(manifest):
                if dependency_manifest.is_relative_to(self.repo_root):
                    relative_root = dependency_manifest.parent.relative_to(
                        self.repo_root
                    ).as_posix()
                    repository_roots.add(relative_root or ".")
                    manifest_queue.append(dependency_manifest)
                    continue
                descriptor = next(
                    (
                        source
                        for source in descriptors
                        if dependency_manifest.is_relative_to(source.repo_root)
                        and any(
                            dependency_manifest.relative_to(
                                source.repo_root
                            ).as_posix()
                            == root
                            or dependency_manifest.relative_to(
                                source.repo_root
                            ).as_posix().startswith(root + "/")
                            for root in source.include_roots
                        )
                    ),
                    None,
                )
                if descriptor is not None:
                    used_external[descriptor.mount_path.casefold()] = descriptor
                    continue
                if not discover_external_sources:
                    raise CoordinatorError(
                        "validation_copy_external_source_missing",
                        "Cargo manifest path dependency has no pinned external source descriptor",
                        details={"manifestPath": str(dependency_manifest)},
                    )
                external_root = self._external_git_root(dependency_manifest)
                relative_root = dependency_manifest.parent.relative_to(
                    external_root
                ).as_posix()
                if relative_root in {"", "."}:
                    raise CoordinatorError(
                        "validation_copy_external_source_layout_unsupported",
                        "Discovered external Cargo package must be below its sibling Git root",
                        details={"manifestPath": str(dependency_manifest)},
                    )
                discovered_roots.setdefault(external_root, set()).add(relative_root)

        for external_root, include_roots in discovered_roots.items():
            descriptor = self._discovered_sibling_source(external_root, include_roots)
            used_external[descriptor.mount_path.casefold()] = descriptor

        roots = tuple(sorted(repository_roots, key=str.casefold))
        result = subprocess.run(
            ["git", "ls-files", "--", *roots],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            encoding="utf-8",
        )
        paths = {line for line in result.stdout.splitlines() if line}
        for root_file in ("Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml"):
            tracked = subprocess.run(
                ["git", "ls-files", "--error-unmatch", "--", root_file],
                cwd=self.repo_root,
                check=False,
                capture_output=True,
                encoding="utf-8",
            )
            if tracked.returncode == 0:
                paths.add(root_file)
        paths.update(self._compile_time_resource_paths(paths, repository_roots))
        return CargoInputClosure(
            tuple(sorted(paths, key=str.casefold)),
            tuple(sorted(used_external.values(), key=lambda item: item.mount_path.casefold())),
        )

    def _compile_time_resource_paths(
        self,
        tracked_paths: set[str],
        package_roots: set[str],
    ) -> set[str]:
        roots = tuple(
            sorted(
                {(self.repo_root / root).resolve() for root in package_roots},
                key=lambda path: str(path).casefold(),
            )
        )
        resource_roots: set[str] = set()
        for relative in tracked_paths:
            if not relative.endswith(".rs"):
                continue
            source = (self.repo_root / relative).resolve()
            package_root = _package_root_for_source(source, roots)
            if package_root is None or not source.is_file():
                continue
            expressions = _compile_time_include_expressions(
                _rust_tokens(source.read_text(encoding="utf-8"))
            )
            for expression in expressions:
                resource = _compile_time_resource(
                    expression,
                    source=source,
                    package_root=package_root,
                    repo_root=self.repo_root,
                )
                resource_roots.add(resource.relative_to(self.repo_root).as_posix())
        if not resource_roots:
            return set()
        return self._tracked_compile_time_resources(resource_roots)

    def _tracked_compile_time_resources(self, resource_roots: set[str]) -> set[str]:
        ordered_roots = tuple(sorted(resource_roots, key=str.casefold))
        batches: list[tuple[str, ...]] = []
        batch: list[str] = []
        for root in ordered_roots:
            candidate = ("git", "ls-files", "--", *batch, root)
            if (
                batch
                and len(subprocess.list2cmdline(candidate))
                > _GIT_PATHSPEC_COMMAND_CHAR_LIMIT
            ):
                batches.append(tuple(batch))
                batch = []
            batch.append(root)
        if batch:
            batches.append(tuple(batch))

        tracked: set[str] = set()
        try:
            for roots in batches:
                result = subprocess.run(
                    ["git", "ls-files", "--", *roots],
                    cwd=self.repo_root,
                    check=True,
                    capture_output=True,
                    encoding="utf-8",
                )
                tracked.update(line for line in result.stdout.splitlines() if line)
        except (OSError, subprocess.SubprocessError) as error:
            details: dict[str, object] = {
                "operation": "git_ls_files_compile_time_resources",
                "errorType": type(error).__name__,
                "resourceRootCount": len(ordered_roots),
            }
            for name in ("errno", "winerror"):
                value = getattr(error, name, None)
                if isinstance(value, int):
                    details[name] = value
            if isinstance(error, subprocess.CalledProcessError):
                details["exitCode"] = int(error.returncode)
            raise CoordinatorError(
                "validation_copy_compile_time_resource_git_failed",
                "Git could not enumerate compile-time resources",
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
            ["git", "rev-parse", "--show-toplevel"],
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
            ["git", "rev-parse", "HEAD"],
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
        for root_file in (
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
        ):
            tracked = subprocess.run(
                ["git", "cat-file", "-e", f"{commit}:{root_file}"],
                cwd=external_root,
                check=False,
                capture_output=True,
                encoding="utf-8",
            )
            if tracked.returncode == 0:
                include_roots.add(root_file)
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
            ["cargo", "metadata", "--format-version", "1", "--locked"],
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
