import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs",
        (
            "pub(crate) fn replace_close_prompt",
            "pub(crate) fn replace_asset_deletion_blocker",
            "ui.overlay_presentation.close_prompt_commit_count",
            "ui.overlay_presentation.asset_deletion_blocker_commit_count",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/window/presentation/"
        "close_prompt.rs",
        ("state.replace_close_prompt(prompt);", "HostRedrawRequest::region(damage)"),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/window/presentation/"
        "asset_deletion_blocker.rs",
        (
            "state.replace_asset_deletion_blocker(blocker);",
            "HostRedrawRequest::region(damage)",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
        "button_dispatch/entry/sequence/steps.rs",
        ("dispatch_close_prompt_step", "dispatch_asset_deletion_blocker_step"),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/"
        "paint_workbench_renderer/scene_layers/overlay/modal.rs",
        ("draw_asset_deletion_blocker", "draw_close_prompt"),
    ),
    (
        "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
        "SlateInvalidationRoot.cpp",
        ("bool FSlateInvalidationRoot::ProcessInvalidation", "FinalUpdateList"),
    ),
    (
        "dev/slint/internal/core/partial_renderer.rs",
        ("pub struct DirtyRegion", "pub fn mark_dirty_region(&self, region: DirtyRegion)"),
    ),
)

FORBIDDEN_SOURCE_TOKENS = {
    "zircon_editor/src/ui/retained_host/host_contract/window/presentation/"
    "close_prompt.rs": ("update_host_presentation",),
    "zircon_editor/src/ui/retained_host/host_contract/window/presentation/"
    "asset_deletion_blocker.rs": ("update_host_presentation",),
}


class SourceContractError(RuntimeError):
    pass


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest().upper()


def _git_output(repo_root: Path, *args: str) -> str | None:
    try:
        completed = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout.strip()


def _method_body(source: str, name: str) -> str:
    marker = f"pub(crate) fn {name}"
    try:
        body = source.split(marker, 1)[1]
    except IndexError as error:
        raise SourceContractError(f"missing method: {name}") from error
    return body.split("pub(crate) fn ", 1)[0]


def source_binding_report(repo_root: Path) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    critical_sources = []
    source_set = hashlib.sha256()
    relative_paths = []
    for relative_path, required_tokens in CRITICAL_SOURCE_CONTRACTS:
        path = repo_root / relative_path
        try:
            payload = path.read_bytes()
        except OSError as error:
            raise SourceContractError(f"missing critical source: {relative_path}") from error
        source = payload.decode("utf-8")
        missing = [token for token in required_tokens if token not in source]
        if missing:
            raise SourceContractError(
                f"critical source contract changed: {relative_path}: {missing}"
            )
        obsolete = [
            token
            for token in FORBIDDEN_SOURCE_TOKENS.get(relative_path, ())
            if token in source
        ]
        if obsolete:
            raise SourceContractError(
                f"obsolete source contract returned: {relative_path}: {obsolete}"
            )
        if relative_path.endswith("globals/state.rs"):
            for method in ("replace_close_prompt", "replace_asset_deletion_blocker"):
                body = _method_body(source, method)
                if "indexes_presentation" in body or "HostWorkbenchHitIndex::from_presentation" in body:
                    raise SourceContractError(
                        f"overlay publisher reentered hit-index validation: {method}"
                    )
        digest = _sha256(payload)
        critical_sources.append(
            {
                "relative_path": relative_path,
                "sha256": digest,
                "byte_length": len(payload),
            }
        )
        relative_paths.append(relative_path)
        source_set.update(relative_path.encode("utf-8"))
        source_set.update(b"\0")
        source_set.update(bytes.fromhex(digest))

    dirty = _git_output(repo_root, "status", "--short", "--", *relative_paths) or ""
    return {
        "ready": True,
        "git_revision": _git_output(repo_root, "rev-parse", "HEAD") or "unavailable",
        "critical_source_dirty": bool(dirty),
        "critical_source_dirty_entry_count": len(dirty.splitlines()) if dirty else 0,
        "source_set_sha256": source_set.hexdigest().upper(),
        "critical_sources": critical_sources,
    }


def run(*, overlay_update_count: int, paint_model_count: int) -> dict[str, Any]:
    if overlay_update_count < 0:
        raise ValueError("overlay_update_count must be non-negative")
    if paint_model_count <= 0:
        raise ValueError("paint_model_count must be positive")

    legacy_identity_visits = overlay_update_count * paint_model_count
    return {
        "schema": "zircon.editor.overlay_presentation_pressure.v1",
        "model_scope": "deterministic paint-model identity and temporary directory operations; not elapsed time or memory",
        "overlay_update_count": overlay_update_count,
        "paint_model_count": paint_model_count,
        "legacy_generic_presentation_update": {
            "paint_model_directory_materialization_count": overlay_update_count,
            "paint_model_identity_visit_count": legacy_identity_visits,
            "hit_index_full_build_count_when_models_are_stable": 0,
            "complexity": "O(U * P)",
        },
        "current_typed_overlay_publication": {
            "paint_model_directory_materialization_count": 0,
            "paint_model_identity_visit_count": 0,
            "hit_index_full_build_count_when_models_are_stable": 0,
            "structure_generation_increment_count": overlay_update_count,
            "geometry_generation_increment_count": overlay_update_count,
            "complexity": "O(U) plus payload copy-on-write when an external snapshot is retained",
        },
        "implemented_delta": {
            "avoided_paint_model_directory_materialization_count": overlay_update_count,
            "avoided_paint_model_identity_visit_count": legacy_identity_visits,
        },
        "interpretation": {
            "is_product_timing": False,
            "included": "source-proven generic presentation model enumeration removed from close-prompt and asset-deletion overlay publication",
            "excluded": "elapsed CPU, allocator bytes, Arc snapshot COW depth, paint cost, GPU time, and input-to-present latency",
            "required_product_evidence": "overlay update count, presentation COW count, paint-model visits, allocation bytes, and overlay click-to-present p95",
        },
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.casefold() == "c:":
        raise ValueError("profile artifacts must not be written to the C drive")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--overlay-update-count", type=int, default=10_000)
    parser.add_argument("--paint-model-count", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        overlay_update_count=args.overlay_update_count,
        paint_model_count=args.paint_model_count,
    )
    result["source_binding"] = source_binding_report(Path(__file__).resolve().parents[4])
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
