from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile
from collections import Counter
from pathlib import Path
from typing import Sequence

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.frameworks_01_resource_consumer_manifest import (
    _bounded_ordered_map,
    _manifest_sha256,
    _path_key,
    _sha256_bytes,
)
from tools.frameworks_01_resource_hard_cut_manifest import (
    FUTURE_CRATE_PATHS,
    RESOURCE_IMPLEMENTATION_ROOT,
    SCHEMA_VERSION as SOURCE_REPORT_SCHEMA_VERSION,
    _current_root_paths,
)
from tools.frameworks_01_resource_hard_cut_spec import REQUIRED_CONSUMER_PATCHES
from tools.runtime_domain_dependency_audit import _rust_code_view


SCHEMA_VERSION = 3
SOURCE_REPORT_FUTURE_PATHS = FUTURE_CRATE_PATHS
CRATE_ROOT = "zircon_runtime/crates/zr_resource"
CRATE_SOURCE_ROOT = f"{CRATE_ROOT}/src"
RETAINED_RUNTIME_FACADES = (
    f"{RESOURCE_IMPLEMENTATION_ROOT}/mod.rs",
    f"{RESOURCE_IMPLEMENTATION_ROOT}/io/mod.rs",
)
RUNTIME_GUARD_RELOCATIONS = {
    f"{RESOURCE_IMPLEMENTATION_ROOT}/management_generation/tests/hard_cut.rs": (
        "zircon_runtime/src/tests/runtime_absorption/resource_foundation/"
        "resource_owner_boundary/mod.rs"
    ),
    f"{RESOURCE_IMPLEMENTATION_ROOT}/management_generation/tests/support.rs": (
        "zircon_runtime/src/tests/runtime_absorption/resource_foundation/"
        "resource_owner_boundary/support.rs"
    ),
}
MODULE_SET_REWRITE = (
    f"{RESOURCE_IMPLEMENTATION_ROOT}/management_generation/tests/mod.rs"
)
GENERATED_CRATE_SURFACES = (
    f"{CRATE_ROOT}/Cargo.toml",
    f"{CRATE_SOURCE_ROOT}/assembly.rs",
    f"{CRATE_SOURCE_ROOT}/io/mod.rs",
    f"{CRATE_SOURCE_ROOT}/lib.rs",
)
REQUIRED_PATCH_INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "zircon_runtime/Cargo.toml",
    "zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs",
)
CRATE_RESOURCE_PATH = re.compile(
    r"(?<![A-Za-z0-9_])crate\s*::\s*core\s*::\s*resource\b"
)
RUNTIME_ROOT_PATH = re.compile(
    r"(?<![A-Za-z0-9_])(?:crate|zircon_runtime)\s*::"
)
SOURCE_STABILITY_KEYS = frozenset(
    {
        "consumer_snapshot",
        "future_paths_absent",
        "supplemental_candidates",
        "supplemental_content",
        "supplemental_terminal_snapshot",
    }
)


class MoveManifestError(RuntimeError):
    pass


class MoveManifestStabilityError(MoveManifestError):
    def __init__(self, reason: str, changed_paths: Sequence[str] = ()) -> None:
        self.reason = reason
        self.changed_paths = list(changed_paths)
        suffix = (
            f": {', '.join(self.changed_paths)}" if self.changed_paths else ""
        )
        super().__init__(f"resource hard-cut move manifest is unstable ({reason}){suffix}")


def _require_source_report(source_report: dict[str, object]) -> list[dict[str, object]]:
    if source_report.get("schema_version") != SOURCE_REPORT_SCHEMA_VERSION:
        raise MoveManifestError("unsupported resource hard-cut source report schema")
    if source_report.get("future_paths") != list(SOURCE_REPORT_FUTURE_PATHS):
        raise MoveManifestError("source report future-path contract does not match")
    stability = source_report.get("stability")
    if (
        not isinstance(stability, dict)
        or set(stability) != SOURCE_STABILITY_KEYS
        or any(value is not True for value in stability.values())
    ):
        raise MoveManifestError("source report is not fully stable")
    inputs = source_report.get("inputs")
    if not isinstance(inputs, list) or not all(isinstance(entry, dict) for entry in inputs):
        raise MoveManifestError("source report inputs must be a list of objects")
    paths = [entry.get("path") for entry in inputs]
    if not all(isinstance(path, str) for path in paths) or len(paths) != len(set(paths)):
        raise MoveManifestError("source report input paths must be unique strings")
    for entry in inputs:
        byte_count = entry.get("bytes")
        sha256 = entry.get("sha256")
        roles = entry.get("roles")
        if (
            not isinstance(byte_count, int)
            or isinstance(byte_count, bool)
            or byte_count < 0
            or not isinstance(sha256, str)
            or re.fullmatch(r"[0-9a-f]{64}", sha256) is None
            or not isinstance(roles, list)
            or not all(isinstance(role, str) for role in roles)
            or roles != sorted(set(roles))
        ):
            raise MoveManifestError(
                f"source report input metadata is invalid: {entry.get('path')}"
            )
    canonical_inputs = sorted(
        inputs,
        key=lambda entry: (str(entry["path"]).casefold(), str(entry["path"])),
    )
    if inputs != canonical_inputs:
        raise MoveManifestError("source report inputs are not in canonical path order")
    if source_report.get("atomic_input_count") != len(inputs):
        raise MoveManifestError("source report atomic input count does not match")
    if source_report.get("atomic_input_manifest_sha256") != _manifest_sha256(inputs):
        raise MoveManifestError("source report atomic input manifest hash does not match")
    return inputs


def _resource_owner_paths(repo_root: Path) -> tuple[Path, ...]:
    return tuple(
        path
        for path in _current_root_paths(repo_root, (RESOURCE_IMPLEMENTATION_ROOT,))
        if path.suffix.casefold() == ".rs"
    )


def _changed_membership_paths(
    expected: Sequence[str], current: Sequence[Path]
) -> list[str]:
    return sorted(
        set(expected).symmetric_difference(path.as_posix() for path in current),
        key=lambda path: (path.casefold(), path),
    )


def _read_verified_sources(
    repo_root: Path,
    entries: Sequence[dict[str, object]],
    *,
    reason: str,
) -> dict[str, str]:
    def read(entry: dict[str, object]) -> tuple[str, str, bool]:
        path = str(entry["path"])
        try:
            source_bytes = (repo_root / path).read_bytes()
            source = source_bytes.decode("utf-8")
        except (OSError, UnicodeError):
            return path, "", True
        changed = (
            len(source_bytes) != entry.get("bytes")
            or _sha256_bytes(source_bytes) != entry.get("sha256")
        )
        return path, source, changed

    sources: dict[str, str] = {}
    changed_paths: list[str] = []
    for path, source, changed in _bounded_ordered_map(read, entries):
        if changed:
            changed_paths.append(path)
        else:
            sources[path] = source
    if changed_paths:
        raise MoveManifestStabilityError(reason, changed_paths)
    return sources


def _entry_map(inputs: Sequence[dict[str, object]]) -> dict[str, dict[str, object]]:
    return {str(entry["path"]): entry for entry in inputs}


def _entries_with_role(
    inputs: Sequence[dict[str, object]], role: str
) -> list[dict[str, object]]:
    return [
        entry
        for entry in inputs
        if isinstance(entry.get("roles"), list) and role in entry["roles"]
    ]


def _move_destination(source: str) -> str:
    relative = Path(source).relative_to(RESOURCE_IMPLEMENTATION_ROOT)
    return f"{CRATE_SOURCE_ROOT}/{relative.as_posix()}"


def _source_operation(
    source: str,
    entry: dict[str, object],
    source_text: str,
) -> dict[str, object]:
    if source in RETAINED_RUNTIME_FACADES:
        kind = "replace_runtime_facade"
        destination = source
    elif source in RUNTIME_GUARD_RELOCATIONS:
        kind = "relocate_runtime_guard"
        destination = RUNTIME_GUARD_RELOCATIONS[source]
    else:
        code_view = _rust_code_view(source_text)
        unsupported_runtime_root = next(
            (
                match
                for match in RUNTIME_ROOT_PATH.finditer(code_view)
                if CRATE_RESOURCE_PATH.match(code_view, match.start()) is None
            ),
            None,
        )
        if unsupported_runtime_root is not None:
            raise MoveManifestError(
                f"moved owner has a higher-layer Runtime dependency: {source}"
            )
        if source == MODULE_SET_REWRITE:
            kind = "move_rewrite_module_set"
        elif CRATE_RESOURCE_PATH.search(code_view) is not None:
            kind = "move_rewrite_crate_root"
        else:
            kind = "move_verbatim"
        destination = _move_destination(source)
    return {
        "bytes": entry["bytes"],
        "destination": destination,
        "kind": kind,
        "sha256": entry["sha256"],
        "source": source,
    }


def _generated_operation(destination: str) -> dict[str, object]:
    return {
        "bytes": None,
        "destination": destination,
        "kind": "generate_crate_surface",
        "sha256": None,
        "source": None,
    }


def _patch_operation(path: str, entry: dict[str, object]) -> dict[str, object]:
    return {
        "bytes": entry["bytes"],
        "destination": path,
        "kind": "patch_required",
        "sha256": entry["sha256"],
        "source": path,
    }


def _consumer_patch_operation(
    path: str, entry: dict[str, object]
) -> dict[str, object]:
    return {
        "bytes": entry["bytes"],
        "destination": path,
        "kind": "patch_consumer",
        "sha256": entry["sha256"],
        "source": path,
    }


def _operation_key(operation: dict[str, object]) -> tuple[str, str, str]:
    source = str(operation["source"] or "")
    destination = str(operation["destination"] or "")
    return source.casefold(), destination.casefold(), str(operation["kind"])


def _destination_collisions(repo_root: Path) -> list[str]:
    destinations = set(GENERATED_CRATE_SURFACES)
    destinations.update(RUNTIME_GUARD_RELOCATIONS.values())
    return sorted(
        (path for path in destinations if (repo_root / path).exists()),
        key=lambda path: (path.casefold(), path),
    )


def _write_path_manifest(
    operations: Sequence[dict[str, object]],
) -> list[dict[str, object]]:
    roles_by_path: dict[str, set[str]] = {}
    current_by_path: dict[str, tuple[object, object]] = {}
    for operation in operations:
        source = operation["source"]
        destination = operation["destination"]
        if isinstance(source, str):
            current_by_path[source] = (operation["bytes"], operation["sha256"])
            if source != destination:
                roles_by_path.setdefault(source, set()).add("delete_source")
        if isinstance(destination, str):
            roles_by_path.setdefault(destination, set()).add("write_destination")

    write_paths = []
    for path in sorted(roles_by_path, key=lambda item: (item.casefold(), item)):
        current = current_by_path.get(path)
        write_paths.append(
            {
                "current_bytes": current[0] if current is not None else None,
                "current_sha256": current[1] if current is not None else None,
                "path": path,
                "roles": sorted(roles_by_path[path]),
            }
        )
    return write_paths


def build_resource_hard_cut_move_manifest(
    repo_root: Path, source_report: dict[str, object]
) -> dict[str, object]:
    repo_root = repo_root.resolve()
    inputs = _require_source_report(source_report)
    owner_entries = _entries_with_role(inputs, "resource_implementation_owner")
    owner_entries.sort(
        key=lambda entry: (str(entry["path"]).casefold(), str(entry["path"]))
    )
    owner_paths = [str(entry["path"]) for entry in owner_entries]
    current_owner_paths = _resource_owner_paths(repo_root)
    if owner_paths != [path.as_posix() for path in current_owner_paths]:
        raise MoveManifestStabilityError(
            "resource_owner_membership_changed",
            _changed_membership_paths(owner_paths, current_owner_paths),
        )

    input_by_path = _entry_map(inputs)
    fixed_workspace_paths = {
        str(entry["path"])
        for entry in _entries_with_role(inputs, "fixed_workspace_input")
    }
    missing_patch_inputs = [
        path for path in REQUIRED_PATCH_INPUTS if path not in fixed_workspace_paths
    ]
    if missing_patch_inputs:
        raise MoveManifestError(
            "required patch input is missing: " + ", ".join(missing_patch_inputs)
        )
    missing_consumer_patch_inputs = [
        path for path in REQUIRED_CONSUMER_PATCHES if path not in input_by_path
    ]
    if missing_consumer_patch_inputs:
        raise MoveManifestError(
            "required consumer patch input is missing: "
            + ", ".join(missing_consumer_patch_inputs)
        )
    owner_sources = _read_verified_sources(
        repo_root,
        owner_entries,
        reason="resource_owner_content_changed",
    )
    patch_entries = [input_by_path[path] for path in REQUIRED_PATCH_INPUTS]
    _read_verified_sources(
        repo_root,
        patch_entries,
        reason="patch_input_content_changed",
    )
    consumer_patch_entries = [
        input_by_path[path] for path in REQUIRED_CONSUMER_PATCHES
    ]
    _read_verified_sources(
        repo_root,
        consumer_patch_entries,
        reason="consumer_patch_content_changed",
    )

    collisions = _destination_collisions(repo_root)
    if collisions:
        raise MoveManifestError("destination already exists: " + ", ".join(collisions))

    operations = [
        _source_operation(path, input_by_path[path], owner_sources[path])
        for path in owner_paths
    ]
    operations.extend(_generated_operation(path) for path in GENERATED_CRATE_SURFACES)
    operations.extend(
        _patch_operation(path, input_by_path[path]) for path in REQUIRED_PATCH_INPUTS
    )
    operations.extend(
        _consumer_patch_operation(path, input_by_path[path])
        for path in REQUIRED_CONSUMER_PATCHES
    )
    operations.sort(key=_operation_key)

    destinations = [
        str(operation["destination"])
        for operation in operations
        if operation["destination"] is not None
    ]
    duplicate_destinations = sorted(
        (path for path, count in Counter(destinations).items() if count > 1),
        key=lambda path: (path.casefold(), path),
    )
    if duplicate_destinations:
        raise MoveManifestError(
            "multiple operations target the same destination: "
            + ", ".join(duplicate_destinations)
        )

    final_owner_paths = _resource_owner_paths(repo_root)
    if current_owner_paths != final_owner_paths:
        raise MoveManifestStabilityError(
            "resource_owner_membership_changed",
            _changed_membership_paths(owner_paths, final_owner_paths),
        )
    _read_verified_sources(
        repo_root,
        owner_entries,
        reason="resource_owner_content_changed",
    )
    _read_verified_sources(
        repo_root,
        patch_entries,
        reason="patch_input_content_changed",
    )
    _read_verified_sources(
        repo_root,
        consumer_patch_entries,
        reason="consumer_patch_content_changed",
    )
    collisions = _destination_collisions(repo_root)
    if collisions:
        raise MoveManifestStabilityError("destination_appeared", collisions)

    operation_counts = dict(
        sorted(
            Counter(str(operation["kind"]) for operation in operations).items()
        )
    )
    write_paths = _write_path_manifest(operations)
    return {
        "operation_count": len(operations),
        "operation_counts": operation_counts,
        "operation_manifest_sha256": _manifest_sha256(operations),
        "operations": operations,
        "resource_owner_input_count": len(owner_entries),
        "resource_owner_manifest_sha256": _manifest_sha256(owner_entries),
        "schema_version": SCHEMA_VERSION,
        "source_atomic_input_manifest_sha256": source_report[
            "atomic_input_manifest_sha256"
        ],
        "stability": {
            "destinations_absent": True,
            "resource_owner_content": True,
            "resource_owner_membership": True,
            "source_report": True,
            "write_path_plan": True,
        },
        "write_path_count": len(write_paths),
        "write_path_manifest_sha256": _manifest_sha256(write_paths),
        "write_paths": write_paths,
    }


def write_resource_hard_cut_move_manifest(
    report: dict[str, object], output_path: Path
) -> None:
    output_path = output_path.resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(
        report,
        ensure_ascii=False,
        indent=2,
        sort_keys=True,
    ).encode("utf-8") + b"\n"
    descriptor, temporary_name = tempfile.mkstemp(
        dir=output_path.parent,
        prefix=f".{output_path.name}.",
        suffix=".tmp",
    )
    temporary_path = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, output_path)
    finally:
        if temporary_path.exists():
            temporary_path.unlink()


def _read_source_report(path: Path) -> dict[str, object]:
    try:
        report = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise MoveManifestError(f"failed to read source report {path}: {error}") from error
    if not isinstance(report, dict):
        raise MoveManifestError("source report root must be an object")
    return report


def _parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compose a deterministic Frameworks01 zr_resource move manifest."
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--source-report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    parsed = _parse_arguments(sys.argv[1:] if arguments is None else arguments)
    try:
        source_report = _read_source_report(parsed.source_report)
        report = build_resource_hard_cut_move_manifest(
            parsed.repo_root,
            source_report,
        )
        write_resource_hard_cut_move_manifest(report, parsed.output)
    except MoveManifestError as error:
        print(str(error), file=sys.stderr)
        return 2
    print(
        json.dumps(
            {
                "operation_count": report["operation_count"],
                "operation_manifest_sha256": report["operation_manifest_sha256"],
                "output": str(parsed.output.resolve()),
                "resource_owner_input_count": report["resource_owner_input_count"],
                "write_path_count": report["write_path_count"],
                "write_path_manifest_sha256": report[
                    "write_path_manifest_sha256"
                ],
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
