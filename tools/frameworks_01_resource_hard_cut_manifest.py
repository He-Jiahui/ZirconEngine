from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Sequence

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.frameworks_01_resource_consumer_manifest import (
    ResourceConsumerManifestError,
    _bounded_ordered_map,
    _git_resource_reference_inventory,
    _manifest_sha256,
    _path_key,
    _safe_relative_path,
    _sha256_bytes,
    capture_resource_consumer_snapshot,
    finalize_resource_consumer_snapshot,
    revalidate_resource_consumer_snapshot,
)


SCHEMA_VERSION = 3
RESOURCE_IMPLEMENTATION_ROOT = "zircon_runtime/src/core/resource"
INTERFACE_RESOURCE_ROOT = "zircon_runtime_interface/src/resource"
TEXTUAL_REFERENCE_ROOTS = ("docs", "examples", "tools")
TEXTUAL_REFERENCE_SUFFIXES = frozenset(
    {".json", ".md", ".ps1", ".py", ".rs", ".sh", ".toml", ".wgsl", ".yaml", ".yml"}
)
TEXTUAL_REFERENCE_TOKENS = (
    b"core/resource",
    b"core::resource",
    b"zircon_resource",
    b"zr_resource",
)
FIXED_WORKSPACE_INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "zircon_runtime/Cargo.toml",
    "zircon_runtime/src/core/mod.rs",
    "zircon_runtime/src/core/resource/mod.rs",
    "zircon_runtime/src/lib.rs",
    "zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs",
    "zircon_runtime_interface/Cargo.toml",
    "zircon_runtime_interface/src/lib.rs",
    "zircon_runtime_interface/src/resource/mod.rs",
    "zircon_runtime_interface/src/tests/mod.rs",
    "zircon_runtime_interface/src/tests/resource_contracts.rs",
)
FUTURE_CRATE_PATHS = (
    "zircon_runtime/crates/zr_resource/Cargo.toml",
    "zircon_runtime/crates/zr_resource/src/assembly.rs",
    "zircon_runtime/crates/zr_resource/src/lib.rs",
)


class HardCutManifestError(RuntimeError):
    pass


class HardCutManifestStabilityError(HardCutManifestError):
    def __init__(self, reason: str, changed_paths: Sequence[str] = ()) -> None:
        self.reason = reason
        self.changed_paths = list(changed_paths)
        suffix = (
            f": {', '.join(self.changed_paths)}" if self.changed_paths else ""
        )
        super().__init__(f"resource hard-cut manifest is unstable ({reason}){suffix}")


def _current_root_paths(repo_root: Path, roots: Sequence[str]) -> tuple[Path, ...]:
    paths: set[Path] = set()
    for raw_root in roots:
        relative_root = _safe_relative_path(raw_root)
        absolute_root = repo_root / relative_root
        if not absolute_root.exists():
            continue
        candidates = (absolute_root,) if absolute_root.is_file() else absolute_root.rglob("*")
        for absolute_path in candidates:
            if not absolute_path.is_file():
                continue
            try:
                resolved_path = absolute_path.resolve(strict=True)
            except OSError as error:
                raise HardCutManifestError(
                    f"failed to resolve hard-cut input {absolute_path}: {error}"
                ) from error
            if not resolved_path.is_relative_to(repo_root):
                raise HardCutManifestError(
                    f"hard-cut input resolves outside the repository: {absolute_path}"
                )
            paths.add(absolute_path.relative_to(repo_root))
    return tuple(sorted(paths, key=_path_key))


def _textual_reference_candidates(repo_root: Path) -> tuple[Path, ...]:
    return _git_resource_reference_inventory(
        repo_root,
        textual_roots=TEXTUAL_REFERENCE_ROOTS,
        textual_suffixes=TEXTUAL_REFERENCE_SUFFIXES,
        textual_tokens=TEXTUAL_REFERENCE_TOKENS,
    ).textual_candidates


def _inside_root(path: Path, root: str) -> bool:
    display_path = path.as_posix().casefold()
    folded_root = root.casefold()
    return display_path == folded_root or display_path.startswith(folded_root + "/")


def _is_textual_reference_candidate(path: Path) -> bool:
    return path.suffix.casefold() in TEXTUAL_REFERENCE_SUFFIXES and any(
        _inside_root(path, root) for root in TEXTUAL_REFERENCE_ROOTS
    )


def _supplemental_candidate_paths(
    repo_root: Path,
    *,
    textual_candidates: Sequence[Path] | None = None,
) -> tuple[Path, ...]:
    candidates = set(
        _current_root_paths(
            repo_root,
            (RESOURCE_IMPLEMENTATION_ROOT, INTERFACE_RESOURCE_ROOT),
        )
    )
    candidates.update(
        _textual_reference_candidates(repo_root)
        if textual_candidates is None
        else textual_candidates
    )
    candidates.update(
        Path(path)
        for path in FIXED_WORKSPACE_INPUTS
        if (repo_root / path).is_file()
    )
    return tuple(sorted(candidates, key=_path_key))


def _future_path_collisions(repo_root: Path) -> list[str]:
    return [path for path in FUTURE_CRATE_PATHS if (repo_root / path).exists()]


def _read_supplemental_candidates(
    repo_root: Path, candidates: Sequence[Path]
) -> tuple[list[dict[str, object]], set[str]]:
    def read(path: Path) -> tuple[dict[str, object], bool]:
        try:
            source_bytes = (repo_root / path).read_bytes()
        except FileNotFoundError as error:
            raise HardCutManifestStabilityError(
                "supplemental_content_changed", [path.as_posix()]
            ) from error
        except OSError as error:
            raise HardCutManifestError(
                f"failed to read hard-cut input {path.as_posix()}: {error}"
            ) from error
        fingerprint = {
            "bytes": len(source_bytes),
            "path": path.as_posix(),
            "sha256": _sha256_bytes(source_bytes),
        }
        textual_reference = _is_textual_reference_candidate(path) and any(
            token in source_bytes for token in TEXTUAL_REFERENCE_TOKENS
        )
        return fingerprint, textual_reference

    fingerprints: list[dict[str, object]] = []
    textual_references: set[str] = set()
    for fingerprint, textual_reference in _bounded_ordered_map(read, candidates):
        fingerprints.append(fingerprint)
        if textual_reference:
            textual_references.add(str(fingerprint["path"]))
    return fingerprints, textual_references


def _changed_supplemental_paths(
    repo_root: Path, fingerprints: Sequence[dict[str, object]]
) -> list[str]:
    def changed_state(fingerprint: dict[str, object]) -> tuple[str, bool]:
        display_path = str(fingerprint["path"])
        try:
            source_bytes = (repo_root / display_path).read_bytes()
        except OSError:
            return display_path, True
        changed = (
            len(source_bytes) != fingerprint["bytes"]
            or _sha256_bytes(source_bytes) != fingerprint["sha256"]
        )
        return display_path, changed

    return [
        path
        for path, changed in _bounded_ordered_map(changed_state, fingerprints)
        if changed
    ]


def _revalidate_supplemental_snapshot(
    repo_root: Path,
    candidates: Sequence[Path],
    fingerprints: Sequence[dict[str, object]],
    *,
    textual_candidates: Sequence[Path] | None = None,
) -> None:
    current_candidates = _supplemental_candidate_paths(
        repo_root, textual_candidates=textual_candidates
    )
    if tuple(candidates) != current_candidates:
        changed_paths = sorted(
            {
                path.as_posix()
                for path in set(candidates).symmetric_difference(current_candidates)
            },
            key=lambda path: (path.casefold(), path),
        )
        raise HardCutManifestStabilityError(
            "supplemental_candidate_set_changed", changed_paths
        )
    changed_paths = _changed_supplemental_paths(repo_root, fingerprints)
    if changed_paths:
        raise HardCutManifestStabilityError(
            "supplemental_content_changed", changed_paths
        )


def _merge_input_role(
    inputs: dict[str, dict[str, object]],
    *,
    path: str,
    byte_count: int,
    sha256: str,
    role: str,
) -> None:
    entry = inputs.get(path)
    if entry is None:
        inputs[path] = {
            "bytes": byte_count,
            "path": path,
            "roles": {role},
            "sha256": sha256,
        }
        return
    if entry["bytes"] != byte_count or entry["sha256"] != sha256:
        raise HardCutManifestStabilityError("merged_input_hash_changed", [path])
    roles = entry["roles"]
    if not isinstance(roles, set):
        raise HardCutManifestError(f"invalid internal role state for {path}")
    roles.add(role)


def _role_summaries(inputs: Sequence[dict[str, object]]) -> dict[str, dict[str, int]]:
    summaries: dict[str, dict[str, int]] = {}
    for entry in inputs:
        for role in entry["roles"]:
            summary = summaries.setdefault(role, {"bytes": 0, "files": 0})
            summary["bytes"] += int(entry["bytes"])
            summary["files"] += 1
    return dict(sorted(summaries.items()))


def build_resource_hard_cut_manifest(repo_root: Path) -> dict[str, object]:
    repo_root = repo_root.resolve()
    collisions = _future_path_collisions(repo_root)
    if collisions:
        raise HardCutManifestError(
            "future hard-cut path already exists: " + ", ".join(collisions)
        )

    initial_inventory = _git_resource_reference_inventory(
        repo_root,
        textual_roots=TEXTUAL_REFERENCE_ROOTS,
        textual_suffixes=TEXTUAL_REFERENCE_SUFFIXES,
        textual_tokens=TEXTUAL_REFERENCE_TOKENS,
    )
    consumer_capture = capture_resource_consumer_snapshot(
        repo_root, rust_candidates=initial_inventory.rust_candidates
    )
    supplemental_candidates = _supplemental_candidate_paths(
        repo_root, textual_candidates=initial_inventory.textual_candidates
    )
    missing_fixed_inputs = [
        path for path in FIXED_WORKSPACE_INPUTS if Path(path) not in supplemental_candidates
    ]
    if missing_fixed_inputs:
        raise HardCutManifestError(
            "required hard-cut input is missing: " + ", ".join(missing_fixed_inputs)
        )
    supplemental_fingerprints, textual_references = _read_supplemental_candidates(
        repo_root, supplemental_candidates
    )
    fingerprint_by_path = {
        str(fingerprint["path"]): fingerprint
        for fingerprint in supplemental_fingerprints
    }

    final_inventory = _git_resource_reference_inventory(
        repo_root,
        textual_roots=TEXTUAL_REFERENCE_ROOTS,
        textual_suffixes=TEXTUAL_REFERENCE_SUFFIXES,
        textual_tokens=TEXTUAL_REFERENCE_TOKENS,
    )
    consumer_snapshot = finalize_resource_consumer_snapshot(
        repo_root,
        consumer_capture,
        rust_candidates=final_inventory.rust_candidates,
    )
    _revalidate_supplemental_snapshot(
        repo_root,
        supplemental_candidates,
        supplemental_fingerprints,
        textual_candidates=final_inventory.textual_candidates,
    )
    collisions = _future_path_collisions(repo_root)
    if collisions:
        raise HardCutManifestStabilityError("future_path_appeared", collisions)
    terminal_inventory = _git_resource_reference_inventory(
        repo_root,
        textual_roots=TEXTUAL_REFERENCE_ROOTS,
        textual_suffixes=TEXTUAL_REFERENCE_SUFFIXES,
        textual_tokens=TEXTUAL_REFERENCE_TOKENS,
    )
    revalidate_resource_consumer_snapshot(
        repo_root,
        consumer_snapshot,
        rust_candidates=terminal_inventory.rust_candidates,
    )
    _revalidate_supplemental_snapshot(
        repo_root,
        supplemental_candidates,
        supplemental_fingerprints,
        textual_candidates=terminal_inventory.textual_candidates,
    )
    collisions = _future_path_collisions(repo_root)
    if collisions:
        raise HardCutManifestStabilityError("future_path_appeared", collisions)

    inputs: dict[str, dict[str, object]] = {}

    def add_supplemental_role(path: str, role: str) -> None:
        fingerprint = fingerprint_by_path[path]
        _merge_input_role(
            inputs,
            path=path,
            byte_count=int(fingerprint["bytes"]),
            sha256=str(fingerprint["sha256"]),
            role=role,
        )

    for path in FIXED_WORKSPACE_INPUTS:
        add_supplemental_role(path, "fixed_workspace_input")
    for path in supplemental_candidates:
        if _inside_root(path, RESOURCE_IMPLEMENTATION_ROOT):
            add_supplemental_role(
                path.as_posix(), "resource_implementation_owner"
            )
        if _inside_root(path, INTERFACE_RESOURCE_ROOT):
            add_supplemental_role(path.as_posix(), "interface_resource_dto")
    for consumer in consumer_snapshot.report["consumers"]:
        _merge_input_role(
            inputs,
            path=str(consumer["path"]),
            byte_count=int(consumer["bytes"]),
            sha256=str(consumer["sha256"]),
            role="rust_consumer",
        )
    for path in textual_references:
        add_supplemental_role(path, "textual_reference")

    rendered_inputs: list[dict[str, object]] = []
    for path in sorted(inputs, key=lambda value: (value.casefold(), value)):
        entry = inputs[path]
        rendered_inputs.append(
            {
                "bytes": entry["bytes"],
                "path": path,
                "roles": sorted(entry["roles"]),
                "sha256": entry["sha256"],
            }
        )

    return {
        "atomic_input_count": len(rendered_inputs),
        "atomic_input_manifest_sha256": _manifest_sha256(rendered_inputs),
        "consumer_count": consumer_snapshot.report["consumer_count"],
        "consumer_manifest_sha256": consumer_snapshot.report[
            "consumer_manifest_sha256"
        ],
        "future_paths": list(FUTURE_CRATE_PATHS),
        "inputs": rendered_inputs,
        "role_summaries": _role_summaries(rendered_inputs),
        "schema_version": SCHEMA_VERSION,
        "stability": {
            "consumer_snapshot": True,
            "future_paths_absent": True,
            "supplemental_candidates": True,
            "supplemental_content": True,
            "supplemental_terminal_snapshot": True,
        },
        "supplemental_candidate_count": len(supplemental_fingerprints),
        "supplemental_candidate_manifest_sha256": _manifest_sha256(
            supplemental_fingerprints
        ),
    }


def write_resource_hard_cut_manifest(
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


def _parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a stable Frameworks01 zr_resource atomic hard-cut manifest."
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    parsed = _parse_arguments(sys.argv[1:] if arguments is None else arguments)
    try:
        report = build_resource_hard_cut_manifest(parsed.repo_root)
        write_resource_hard_cut_manifest(report, parsed.output)
    except (HardCutManifestError, ResourceConsumerManifestError) as error:
        print(str(error), file=sys.stderr)
        return 2
    print(
        json.dumps(
            {
                "atomic_input_count": report["atomic_input_count"],
                "atomic_input_manifest_sha256": report[
                    "atomic_input_manifest_sha256"
                ],
                "consumer_count": report["consumer_count"],
                "consumer_manifest_sha256": report["consumer_manifest_sha256"],
                "output": str(parsed.output.resolve()),
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
