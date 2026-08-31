from __future__ import annotations

import argparse
import difflib
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Mapping, Sequence

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import frameworks_01_resource_hard_cut_move_manifest as move_owner
from tools import frameworks_01_resource_hard_cut_manifest as source_owner
from tools.frameworks_01_resource_consumer_manifest import (
    _manifest_sha256,
    _sha256_bytes,
)
from tools.frameworks_01_resource_hard_cut_spec import (
    ASSEMBLY_CONSUMER_RULES,
    ASSEMBLY_SOURCE,
    ASSEMBLY_VISIBILITY_PATHS,
    CONSUMER_REPLACEMENTS,
    CONSUMER_USAGE_MARKERS,
    IO_ASSEMBLY_PROJECTION,
    OWNER_SOURCE_REPLACEMENTS,
    REQUIRED_CONSUMER_PATCHES,
    ROOT_ASSEMBLY_PROJECTION,
    VISIBILITY_REPLACEMENTS,
    ZR_RESOURCE_LOCK_PACKAGE,
    ZR_RESOURCE_MANIFEST,
)
from tools.runtime_domain_dependency_audit import _rust_code_view


SCHEMA_VERSION = 3
RUSTFMT_TOOLCHAIN = "1.94.1"
RESOURCE_ROOT = f"{move_owner.RESOURCE_IMPLEMENTATION_ROOT}/mod.rs"
RESOURCE_IO_ROOT = f"{move_owner.RESOURCE_IMPLEMENTATION_ROOT}/io/mod.rs"


class HardCutPatchError(RuntimeError):
    pass


class HardCutPatchStabilityError(HardCutPatchError):
    pass


def _replace_exact(source: str, old: str, new: str, *, label: str, count: int = 1) -> str:
    actual = source.count(old)
    if actual != count:
        raise HardCutPatchError(
            f"unexpected {label} source shape: expected {count}, found {actual}"
        )
    return source.replace(old, new)


def _replace_code_exact(
    source: str, old: str, new: str, *, label: str, count: int = 1
) -> str:
    code_view = _rust_code_view(source)
    matches = list(re.finditer(re.escape(old), code_view))
    if len(matches) != count:
        raise HardCutPatchError(
            f"unexpected {label} source shape: expected {count}, found {len(matches)}"
        )
    for match in reversed(matches):
        source = source[: match.start()] + new + source[match.end() :]
    return source


def rewrite_resource_crate_root(source: str) -> str:
    code_view = _rust_code_view(source)
    matches = list(move_owner.CRATE_RESOURCE_PATH.finditer(code_view))
    for match in reversed(matches):
        source = source[: match.start()] + "crate" + source[match.end() :]
    return source


def _promote_test_support_cfg(source: str) -> str:
    code_view = _rust_code_view(source)
    matches = []
    for match in re.finditer(r"#\[cfg\(test\)\]", code_view):
        tail = code_view[match.end() :].lstrip()
        if tail.startswith("mod tests"):
            continue
        matches.append(match)
    replacement = '#[cfg(any(test, feature = "test-support"))]'
    for match in reversed(matches):
        source = source[: match.start()] + replacement + source[match.end() :]
    return source


def promote_assembly_visibility(sources: Mapping[str, str]) -> dict[str, str]:
    missing = sorted(set(ASSEMBLY_VISIBILITY_PATHS).difference(sources))
    if missing:
        raise HardCutPatchError("assembly visibility source is missing: " + ", ".join(missing))
    outputs = dict(sources)
    for path, replacements in VISIBILITY_REPLACEMENTS.items():
        source = outputs[path]
        for old, new in replacements:
            source = _replace_code_exact(source, old, new, label=path)
        outputs[path] = source
    for path, replacements in OWNER_SOURCE_REPLACEMENTS.items():
        source = outputs[path]
        for old, new in replacements:
            source = _replace_exact(source, old, new, label=path)
        outputs[path] = source
    transaction_root = f"{move_owner.RESOURCE_IMPLEMENTATION_ROOT}/io/transaction/"
    for path, source in tuple(outputs.items()):
        if path.startswith(transaction_root):
            outputs[path] = _promote_test_support_cfg(source)
    return outputs


def _public_use_declarations(source: str, target_root: str) -> list[str]:
    code_view = _rust_code_view(source)
    matches = list(re.finditer(r"(?m)^pub\s+use\b[\s\S]*?;\n?", code_view))
    declarations = []
    for match in matches:
        declaration = source[match.start() : match.end()].strip()
        if re.search(r"::\s*\*\s*;", _rust_code_view(declaration)):
            raise HardCutPatchError("product Resource facade cannot contain a glob export")
        declaration, replaced = re.subn(
            r"^pub\s+use\s+[A-Za-z_][A-Za-z0-9_:]*::",
            f"pub use {target_root}::",
            declaration,
            count=1,
        )
        if replaced != 1:
            raise HardCutPatchError("unexpected public Resource use declaration")
        declarations.append(declaration)
    if not declarations:
        raise HardCutPatchError("Resource product facade has no public use declarations")
    return declarations


def generated_resource_surfaces(
    resource_root_source: str, resource_io_source: str
) -> tuple[str, str, str, str, str]:
    lib_source = resource_root_source
    lib_source = _replace_code_exact(
        lib_source,
        "pub mod io;",
        'pub mod io;\n\n#[doc(hidden)]\npub mod assembly;',
        label="Resource crate root",
    )
    product_uses = _public_use_declarations(resource_root_source, "zr_resource")
    runtime_facade = (
        "//! Curated Runtime projection of the canonical Resource foundation.\n\n"
        "pub mod io;\n\n"
        + "\n".join(product_uses)
        + "\n\n"
        + ROOT_ASSEMBLY_PROJECTION
    )

    io_root = resource_io_source
    io_root = _replace_code_exact(
        io_root,
        "mod atomic_file;",
        "pub(crate) mod atomic_file;",
        label="Resource I/O crate root",
    )
    io_product_uses = _public_use_declarations(resource_io_source, "zr_resource::io")
    runtime_io_facade = (
        "//! Curated Runtime projection of Resource I/O.\n\n"
        + "\n".join(io_product_uses)
        + "\n\n"
        + IO_ASSEMBLY_PROJECTION
    )
    return lib_source, ASSEMBLY_SOURCE, runtime_facade, io_root, runtime_io_facade


def _patch_consumers(sources: Mapping[str, str]) -> dict[str, str]:
    outputs = {}
    for path in REQUIRED_CONSUMER_PATCHES:
        source = sources[path]
        if CONSUMER_USAGE_MARKERS[path] not in _rust_code_view(source):
            raise HardCutPatchError(f"unexpected consumer source shape: {path}")
        for old, new in CONSUMER_REPLACEMENTS[path]:
            source = _replace_code_exact(
                source,
                old,
                new,
                label=f"consumer {path}",
            )
        outputs[path] = source
    return outputs


def _patch_workspace_manifest(source: str) -> str:
    uses_crlf = "\r\n" in source
    canonical = source.replace("\r\n", "\n")
    if "\r" in canonical:
        raise HardCutPatchError("workspace manifest contains unsupported mixed newlines")
    member = '    "zircon_runtime/crates/zr_resource",\n'
    dependency = (
        'zr_resource = { path = "zircon_runtime/crates/zr_resource", '
        'default-features = false }\n'
    )
    member_count = canonical.count(member)
    dependency_count = canonical.count(dependency)
    if member_count > 1 or dependency_count > 1 or member_count != dependency_count:
        raise HardCutPatchError("partial or duplicate workspace Resource wiring")
    if member_count == 1:
        return source
    canonical = _replace_exact(
        canonical,
        '    "zircon_runtime/crates/zr_math",\n',
        '    "zircon_runtime/crates/zr_math",\n    "zircon_runtime/crates/zr_resource",\n',
        label="workspace member",
    )
    canonical = _replace_exact(
        canonical,
        'zr_math = { path = "zircon_runtime/crates/zr_math", default-features = false }\n',
        'zr_math = { path = "zircon_runtime/crates/zr_math", default-features = false }\n'
        'zr_resource = { path = "zircon_runtime/crates/zr_resource", default-features = false }\n',
        label="workspace Resource dependency",
    )
    return canonical.replace("\n", "\r\n") if uses_crlf else canonical


def _verify_composed_write_set(
    planned_paths: Sequence[str],
    changes: Sequence[dict[str, object]],
    before: Mapping[str, str],
    after: Mapping[str, str],
) -> list[str]:
    changed_paths = [str(entry["path"]) for entry in changes]
    changed = set(changed_paths)
    preapplied = [
        path
        for path in planned_paths
        if path not in changed
        and path in before
        and path in after
        and before[path] == after[path]
    ]
    resolved = sorted(
        changed.union(preapplied), key=lambda path: (path.casefold(), path)
    )
    if resolved != list(planned_paths):
        raise HardCutPatchError(
            "composed changes and exact preapplied paths do not match the sealed write set"
        )
    return preapplied


def _patch_runtime_manifest(source: str) -> str:
    source = _replace_exact(
        source,
        "profiling = []",
        'profiling = ["zr_resource/profiling"]',
        label="Runtime profiling feature",
    )
    source = _replace_exact(
        source,
        "zr_math.workspace = true\n",
        "zr_math.workspace = true\nzr_resource.workspace = true\n",
        label="Runtime Resource dependency",
    )
    return _replace_exact(
        source,
        'ttf2woff2 = { version = "0.13.1", default-features = false }\n',
        'ttf2woff2 = { version = "0.13.1", default-features = false }\n'
        'zr_resource = { workspace = true, features = ["test-support"] }\n',
        label="Runtime Resource test-support dependency",
    )


def _patch_cargo_lock(source: str) -> str:
    runtime_match = re.search(
        r'(?ms)^\[\[package\]\]\nname = "zircon_runtime"\n.*?(?=^\[\[package\]\]|\Z)',
        source,
    )
    if runtime_match is None:
        raise HardCutPatchError("unexpected Cargo.lock Runtime package shape")
    runtime_block = runtime_match.group(0)
    patched_runtime = _replace_exact(
        runtime_block,
        ' "zr_math",\n',
        ' "zr_math",\n "zr_resource",\n',
        label="Cargo.lock Runtime Resource dependency",
    )
    source = source[: runtime_match.start()] + patched_runtime + source[runtime_match.end() :]
    marker = '[[package]]\nname = "zr_rhi"\n'
    if source.count(marker) != 1 or 'name = "zr_resource"' in source:
        raise HardCutPatchError("unexpected Cargo.lock zr_resource package shape")
    return source.replace(marker, ZR_RESOURCE_LOCK_PACKAGE + marker, 1)


def _patch_absorption_root(source: str) -> str:
    addition = (
        '#[path = "resource_foundation/resource_owner_boundary/mod.rs"]\n'
        "mod resource_owner_boundary;\n"
    )
    if addition in source:
        raise HardCutPatchError("Resource owner boundary is already wired")
    if not source.endswith("\n"):
        source += "\n"
    return source + addition


def _relocate_guard_support(source: str) -> str:
    old = """        let mut normalized = vec!["crate".to_owned()];
        normalized.extend_from_slice(&path[1..]);
        return normalized;
"""
    return _replace_exact(
        source,
        old,
        "        return path.to_vec();\n",
        label="relocated Resource guard external-crate normalization",
    )


def _relocate_guard(source: str) -> str:
    replacements = (
        (
            "use super::super::*;\nuse super::support::*;",
            "mod support;\n\nuse support::*;",
        ),
        (
            'fn is_higher_layer_runtime_path(path: &[String]) -> bool {\n'
            '    path.first().is_some_and(|segment| segment == "crate")\n'
            '        && !path.starts_with(&["crate".to_owned(), "core".to_owned(), "resource".to_owned()])\n'
            '}',
            'fn is_higher_layer_runtime_path(path: &[String]) -> bool {\n'
            '    path.first().is_some_and(|segment| segment == "zircon_runtime")\n'
            '        || path.starts_with(&["crate".to_owned(), "core".to_owned()])\n'
            '}',
        ),
        ('include_str!("../../../framework/asset.rs")', 'include_str!("../../../../core/framework/asset.rs")'),
        ('include_str!("../../../framework/mod.rs")', 'include_str!("../../../../core/framework/mod.rs")'),
        ('include_str!("../../manager/management_projection.rs")', 'include_str!("../../../../../crates/zr_resource/src/manager/management_projection.rs")'),
        ('include_str!("../../manager/resource_manager.rs")', 'include_str!("../../../../../crates/zr_resource/src/manager/resource_manager.rs")'),
        ('zircon_runtime/src/core/resource/management_generation.rs', 'zircon_runtime/crates/zr_resource/src/management_generation.rs'),
        ('zircon_runtime/src/core/resource', 'zircon_runtime/crates/zr_resource/src'),
        ('["crate", "core", "resource", "management_generation"]', '["crate", "management_generation"]'),
        ('"use crate::asset::AssetUri;"', '"use zircon_runtime::asset::AssetUri;"'),
        ('"use super::super::diagnostics::profiling;"', '"use zircon_runtime::core::diagnostics::profiling;"'),
        ('"use crate as runtime_root;"', '"use zircon_runtime as runtime_root;"'),
        ('"use crate::core::resource::{ResourceId, ResourceRecord};"', '"use crate::{ResourceId, ResourceRecord};"'),
    )
    for old, new in replacements:
        source = _replace_exact(source, old, new, label="relocated Resource owner guard")
    return source


def _require_consumers(inputs: Sequence[dict[str, object]]) -> None:
    input_by_path = {str(entry["path"]): entry for entry in inputs}
    missing = []
    for path in REQUIRED_CONSUMER_PATCHES:
        entry = input_by_path.get(path)
        roles = entry.get("roles") if entry is not None else None
        if not isinstance(roles, list) or "rust_consumer" not in roles:
            missing.append(path)
    if missing:
        raise HardCutPatchError("required Rust consumer is missing: " + ", ".join(missing))


def _require_assembly_consumer_closure(sources: Mapping[str, str]) -> None:
    rust_consumers = {
        path: _rust_code_view(source)
        for path, source in sources.items()
        if path.endswith(".rs")
        and not path.startswith(f"{move_owner.RESOURCE_IMPLEMENTATION_ROOT}/")
    }
    for label, rule in ASSEMBLY_CONSUMER_RULES.items():
        usage_pattern = re.compile(str(rule["usage_pattern"]))
        anchors = tuple(str(anchor) for anchor in rule["anchors"])
        expected = {str(path) for path in rule["paths"]}
        actual = {
            path
            for path, code_view in rust_consumers.items()
            if usage_pattern.search(code_view)
            and any(anchor in code_view for anchor in anchors)
        }
        if actual != expected:
            missing = sorted(expected.difference(actual))
            unexpected = sorted(actual.difference(expected))
            raise HardCutPatchError(
                f"unregistered assembly consumer closure for {label}: "
                f"missing={missing}, unexpected={unexpected}"
            )


def _unified_patch(before: Mapping[str, str], after: Mapping[str, str]) -> bytes:
    paths = sorted(set(before).union(after), key=lambda path: (path.casefold(), path))
    chunks: list[str] = []
    for path in paths:
        old = before.get(path, "")
        new = after.get(path, "")
        if old == new:
            continue
        from_file = f"a/{path}" if path in before else "/dev/null"
        to_file = f"b/{path}" if path in after else "/dev/null"
        chunks.extend(
            difflib.unified_diff(
                old.splitlines(keepends=True),
                new.splitlines(keepends=True),
                fromfile=from_file,
                tofile=to_file,
                lineterm="\n",
            )
        )
    payload = "".join(chunks)
    if payload and not payload.endswith("\n"):
        payload += "\n"
    return payload.encode("utf-8")


def _format_rust_outputs(
    repo_root: Path, outputs: Mapping[str, str]
) -> dict[str, str]:
    formatted = dict(outputs)
    rust_paths = sorted(
        (path for path in outputs if path.endswith(".rs")),
        key=lambda path: (path.casefold(), path),
    )
    if not rust_paths:
        return formatted

    with tempfile.TemporaryDirectory(
        prefix="frameworks01-zr-resource-rustfmt-", dir=repo_root.parent
    ) as scratch:
        scratch_root = Path(scratch)
        scratch_paths = []
        for index, path in enumerate(rust_paths):
            scratch_path = scratch_root / f"{index:04d}.rs"
            scratch_path.write_bytes(outputs[path].encode("utf-8"))
            scratch_paths.append(scratch_path)
        try:
            result = subprocess.run(
                [
                    "rustfmt",
                    f"+{RUSTFMT_TOOLCHAIN}",
                    "--edition",
                    "2021",
                    "--emit",
                    "files",
                    "--config",
                    "skip_children=true",
                    *(str(path) for path in scratch_paths),
                ],
                cwd=repo_root,
                capture_output=True,
                check=False,
                text=True,
            )
        except OSError as error:
            raise HardCutPatchError(
                f"pinned rustfmt +{RUSTFMT_TOOLCHAIN} is unavailable: {error}"
            ) from error
        if result.returncode != 0:
            diagnostic = (result.stderr or result.stdout).strip()
            raise HardCutPatchError(
                f"pinned rustfmt +{RUSTFMT_TOOLCHAIN} rejected generated Rust"
                + (f": {diagnostic}" if diagnostic else "")
            )
        for path, scratch_path in zip(rust_paths, scratch_paths, strict=True):
            formatted[path] = scratch_path.read_bytes().decode("utf-8").replace("\r\n", "\n")
    return formatted


def compose_resource_hard_cut_patch(
    repo_root: Path,
    source_report: dict[str, object],
    move_report: dict[str, object],
) -> tuple[dict[str, object], bytes]:
    repo_root = repo_root.resolve()
    try:
        inputs = move_owner._require_source_report(source_report)
    except move_owner.MoveManifestError as error:
        raise HardCutPatchError(str(error)) from error
    _require_consumers(inputs)
    try:
        expected_move = move_owner.build_resource_hard_cut_move_manifest(
            repo_root, source_report
        )
    except move_owner.MoveManifestError as error:
        raise HardCutPatchStabilityError(str(error)) from error
    if move_report != expected_move:
        raise HardCutPatchError("move manifest does not match the sealed current source")
    try:
        sources = move_owner._read_verified_sources(
            repo_root,
            inputs,
            reason="atomic_input_content_changed",
        )
    except move_owner.MoveManifestError as error:
        raise HardCutPatchStabilityError(str(error)) from error
    patched_consumers = _patch_consumers(sources)
    _require_assembly_consumer_closure(sources)

    owner_paths = {
        str(entry["path"])
        for entry in inputs
        if "resource_implementation_owner" in entry["roles"]
    }
    owner_sources = promote_assembly_visibility(
        {path: sources[path] for path in owner_paths}
    )
    lib, assembly, runtime_facade, io_root, runtime_io_facade = (
        generated_resource_surfaces(sources[RESOURCE_ROOT], sources[RESOURCE_IO_ROOT])
    )
    generated = {
        f"{move_owner.CRATE_ROOT}/Cargo.toml": ZR_RESOURCE_MANIFEST,
        f"{move_owner.CRATE_SOURCE_ROOT}/assembly.rs": assembly,
        f"{move_owner.CRATE_SOURCE_ROOT}/io/mod.rs": io_root,
        f"{move_owner.CRATE_SOURCE_ROOT}/lib.rs": lib,
    }

    before: dict[str, str] = {}
    after: dict[str, str] = {}
    for operation in move_report["operations"]:
        kind = str(operation["kind"])
        source_path = operation["source"]
        destination = operation["destination"]
        if kind == "replace_runtime_facade":
            path = str(source_path)
            before[path] = sources[path]
            after[path] = runtime_facade if path == RESOURCE_ROOT else runtime_io_facade
        elif kind == "relocate_runtime_guard":
            source_path = str(source_path)
            destination = str(destination)
            before[source_path] = sources[source_path]
            relocated = owner_sources[source_path]
            if source_path.endswith("/hard_cut.rs"):
                relocated = _relocate_guard(relocated)
            else:
                relocated = _relocate_guard_support(relocated)
            after[destination] = relocated
        elif kind.startswith("move_"):
            source_path = str(source_path)
            destination = str(destination)
            before[source_path] = sources[source_path]
            moved = owner_sources[source_path]
            if kind == "move_rewrite_crate_root":
                moved = rewrite_resource_crate_root(moved)
            elif kind == "move_rewrite_module_set":
                moved = _replace_code_exact(
                    moved, "mod hard_cut;\n", "", label="Resource guard module set"
                )
                moved = _replace_code_exact(
                    moved, "mod support;\n", "", label="Resource guard module set"
                )
            after[destination] = moved
        elif kind == "generate_crate_surface":
            destination = str(destination)
            after[destination] = generated[destination]
        elif kind == "patch_required":
            path = str(source_path)
            before[path] = sources[path]
            if path == "Cargo.toml":
                after[path] = _patch_workspace_manifest(sources[path])
            elif path == "Cargo.lock":
                after[path] = _patch_cargo_lock(sources[path])
            elif path == "zircon_runtime/Cargo.toml":
                after[path] = _patch_runtime_manifest(sources[path])
            else:
                after[path] = _patch_absorption_root(sources[path])
        elif kind == "patch_consumer":
            path = str(source_path)
            before[path] = sources[path]
            after[path] = patched_consumers[path]
        else:
            raise HardCutPatchError(f"unsupported move operation: {kind}")

    after = _format_rust_outputs(repo_root, after)
    patch = _unified_patch(before, after)
    if not patch:
        raise HardCutPatchError("hard-cut patch is empty")
    changes = []
    for path in sorted(set(before).union(after), key=lambda item: (item.casefold(), item)):
        old = before.get(path)
        new = after.get(path)
        if old == new:
            continue
        changes.append(
            {
                "after_sha256": _sha256_bytes(new.encode("utf-8")) if new is not None else None,
                "before_sha256": _sha256_bytes(old.encode("utf-8")) if old is not None else None,
                "kind": "add" if old is None else "delete" if new is None else "modify",
                "path": path,
            }
        )
    planned_write_paths = [entry["path"] for entry in move_report["write_paths"]]
    preapplied_write_paths = _verify_composed_write_set(
        planned_write_paths, changes, before, after
    )
    report = {
        "change_count": len(changes),
        "change_manifest_sha256": _manifest_sha256(changes),
        "changes": changes,
        "consumer_patch_count": len(REQUIRED_CONSUMER_PATCHES),
        "move_operation_manifest_sha256": move_report["operation_manifest_sha256"],
        "patch_bytes": len(patch),
        "patch_sha256": _sha256_bytes(patch),
        "preapplied_write_path_count": len(preapplied_write_paths),
        "preapplied_write_paths": preapplied_write_paths,
        "schema_version": SCHEMA_VERSION,
        "source_atomic_input_manifest_sha256": source_report[
            "atomic_input_manifest_sha256"
        ],
        "stability": {
            "atomic_inputs": True,
            "move_manifest": True,
            "source_manifest": True,
            "source_shape": True,
        },
        "write_path_manifest_sha256": move_report[
            "write_path_manifest_sha256"
        ],
    }
    try:
        final_source_report = source_owner.build_resource_hard_cut_manifest(repo_root)
    except (
        source_owner.HardCutManifestError,
        source_owner.ResourceConsumerManifestError,
    ) as error:
        raise HardCutPatchStabilityError(
            f"sealed current source changed during patch composition: {error}"
        ) from error
    if final_source_report != source_report:
        raise HardCutPatchStabilityError(
            "sealed current source changed during patch composition"
        )
    return report, patch


def _atomic_write(path: Path, payload: bytes) -> None:
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
    )
    temporary_path = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, path)
    finally:
        if temporary_path.exists():
            temporary_path.unlink()


def write_resource_hard_cut_patch(
    report: dict[str, object],
    patch: bytes,
    *,
    report_output: Path,
    patch_output: Path,
) -> None:
    report_payload = json.dumps(
        report,
        ensure_ascii=False,
        indent=2,
        sort_keys=True,
    ).encode("utf-8") + b"\n"
    _atomic_write(patch_output, patch)
    _atomic_write(report_output, report_payload)


def _read_json(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise HardCutPatchError(f"failed to read {label} {path}: {error}") from error
    if not isinstance(value, dict):
        raise HardCutPatchError(f"{label} root must be an object")
    return value


def _parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compose a sealed unified patch for the Frameworks01 zr_resource hard cut."
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--source-report", type=Path, required=True)
    parser.add_argument("--move-report", type=Path, required=True)
    parser.add_argument("--report-output", type=Path, required=True)
    parser.add_argument("--patch-output", type=Path, required=True)
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    parsed = _parse_arguments(sys.argv[1:] if arguments is None else arguments)
    try:
        source_report = _read_json(parsed.source_report, "source report")
        move_report = _read_json(parsed.move_report, "move report")
        report, patch = compose_resource_hard_cut_patch(
            parsed.repo_root,
            source_report,
            move_report,
        )
        write_resource_hard_cut_patch(
            report,
            patch,
            report_output=parsed.report_output,
            patch_output=parsed.patch_output,
        )
    except HardCutPatchError as error:
        print(str(error), file=sys.stderr)
        return 2
    print(
        json.dumps(
            {
                "change_count": report["change_count"],
                "patch_bytes": report["patch_bytes"],
                "patch_output": str(parsed.patch_output.resolve()),
                "patch_sha256": report["patch_sha256"],
                "report_output": str(parsed.report_output.resolve()),
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
