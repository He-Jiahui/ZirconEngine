"""SourceTemplate generated project materialization helpers."""

from __future__ import annotations

import hashlib
import re
import shutil
from pathlib import Path
from typing import Any

from .pipeline_report_source_template_validate_schema import (
    source_template_validate_generated_files_schema_diagnostics,
)


PATH_DEPENDENCY_RE = re.compile(
    r'(?P<prefix>^(?P<crate>zircon_[A-Za-z0-9_]+)\s*=\s*\{[^}]*path\s*=\s*")'
    r'(?P<path>[^"]+)'
    r'(?P<suffix>"[^}]*\}\s*$)',
    re.MULTILINE,
)


def generated_file_summaries(validate_payload: dict[str, Any] | None) -> list[dict[str, str]]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return []
    files = plan_summary.get("generated_files")
    if not isinstance(files, list):
        return []
    summaries: list[dict[str, str]] = []
    for file in files:
        if not isinstance(file, dict):
            continue
        path = file.get("path")
        purpose = file.get("purpose", "")
        if isinstance(path, str):
            summaries.append({"path": path, "purpose": purpose if isinstance(purpose, str) else ""})
    return summaries


def source_template_generated_files_plan_diagnostics(
    validate_payload: dict[str, Any] | None,
) -> list[str]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return []
    files = plan_summary.get("generated_files")
    diagnostics = source_template_validate_generated_files_schema_diagnostics(files)
    if diagnostics:
        return diagnostics
    return generated_file_path_duplicate_diagnostics(files)


def materialize_generated_files(
    project_dir: Path,
    files: list[dict[str, str]],
    diagnostics: list[str],
) -> bool:
    duplicate_diagnostics = generated_file_path_duplicate_diagnostics(files)
    if duplicate_diagnostics:
        diagnostics.extend(duplicate_diagnostics)
        return False
    try:
        project_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"SourceTemplate generated project {project_dir} could not be created: {error}"
        )
        return False
    resolved_root, root_resolution_error = resolve_project_root(project_dir)
    for file in files:
        path = file.get("path")
        contents = file.get("contents")
        if not isinstance(path, str):
            continue
        if not isinstance(contents, str):
            diagnostics.append(f"validate report generated file {path} has no contents; skipped")
            continue
        destination = resolve_project_child_from_root(
            project_dir,
            resolved_root,
            root_resolution_error,
            path,
            diagnostics,
        )
        if destination is None:
            continue
        try:
            destination.parent.mkdir(parents=True, exist_ok=True)
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} parent directory could not be created: {error}"
            )
            continue
        try:
            destination.write_text(contents, encoding="utf-8")
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} could not be written: {error}"
            )
    return True


def generated_file_path_safety_diagnostics(
    project_dir: Path,
    validate_payload: dict[str, Any] | None,
) -> list[str]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return []
    files = plan_summary.get("generated_files")
    if not isinstance(files, list):
        return []
    diagnostics: list[str] = []
    resolved_root, root_resolution_error = resolve_project_root(project_dir)
    for file in files:
        if not isinstance(file, dict):
            continue
        path = file.get("path")
        if isinstance(path, str):
            resolve_project_child_from_root(
                project_dir,
                resolved_root,
                root_resolution_error,
                path,
                diagnostics,
            )
    return diagnostics


def generated_file_path_duplicate_diagnostics(files: object) -> list[str]:
    if not isinstance(files, list):
        return []
    diagnostics: list[str] = []
    paths: set[str] = set()
    for file in files:
        if not isinstance(file, dict):
            continue
        path = file.get("path")
        if not isinstance(path, str):
            continue
        if path in paths:
            diagnostics.append(f"SourceTemplate generated file path {path} is duplicated")
            continue
        paths.add(path)
    return diagnostics


def source_template_generated_file_report(
    project_dir: Path,
    generated_files: list[dict[str, str]],
    diagnostics: list[str],
) -> list[dict[str, str | int]]:
    report: list[dict[str, str | int]] = []
    resolved_root, root_resolution_error = resolve_project_root(project_dir)
    for file in generated_files:
        path = file["path"]
        destination = resolve_project_child_from_root(
            project_dir,
            resolved_root,
            root_resolution_error,
            path,
            diagnostics,
        )
        if destination is None:
            continue
        if not destination.exists():
            diagnostics.append(f"SourceTemplate generated file {path} does not exist after materialization")
            continue
        if not destination.is_file():
            diagnostics.append(f"SourceTemplate generated file {path} is not a file after materialization")
            continue
        try:
            contents = destination.read_bytes()
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} could not be read: {error}"
            )
            continue
        report.append(
            {
                "path": path,
                "purpose": file.get("purpose", ""),
                "size": len(contents),
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
        )
    return report


def reset_generated_project_dir(project_dir: Path, diagnostics: list[str]) -> bool:
    if project_dir.exists():
        try:
            shutil.rmtree(project_dir)
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated project {project_dir} could not be removed: {error}"
            )
            return False
    return True


def rewrite_generated_manifest_paths(
    project_dir: Path,
    repo_root: Path,
    diagnostics: list[str],
) -> None:
    manifest_path = project_dir / "Cargo.toml"
    if not manifest_path.exists():
        diagnostics.append(f"SourceTemplate manifest {manifest_path} does not exist after materialization")
        return
    if not manifest_path.is_file():
        diagnostics.append(f"SourceTemplate manifest {manifest_path} is not a file after materialization")
        return
    try:
        source = manifest_path.read_text(encoding="utf-8")
    except OSError as error:
        diagnostics.append(f"SourceTemplate manifest {manifest_path} could not be read: {error}")
        return

    def replace(match: re.Match[str]) -> str:
        crate_name = match.group("crate")
        relative = match.group("path").replace("\\", "/")
        try:
            crate_path = (repo_root / relative.lstrip("./")).resolve()
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate dependency {crate_name} path {relative} could not be resolved: {error}"
            )
            return match.group(0)
        if not crate_path.exists():
            diagnostics.append(f"SourceTemplate dependency {crate_name} path {crate_path} does not exist")
        return f"{match.group('prefix')}{crate_path.as_posix()}{match.group('suffix')}"

    rewritten = PATH_DEPENDENCY_RE.sub(replace, source)
    try:
        manifest_path.write_text(rewritten, encoding="utf-8")
    except OSError as error:
        diagnostics.append(f"SourceTemplate manifest {manifest_path} could not be written: {error}")


def resolve_project_child(
    project_dir: Path,
    relative_path: str,
    diagnostics: list[str],
    *,
    kind: str = "generated file path",
) -> Path | None:
    resolved_root, root_resolution_error = resolve_project_root(project_dir)
    return resolve_project_child_from_root(
        project_dir,
        resolved_root,
        root_resolution_error,
        relative_path,
        diagnostics,
        kind=kind,
    )


def resolve_project_root(project_dir: Path) -> tuple[Path | None, OSError | None]:
    try:
        return project_dir.resolve(), None
    except OSError as error:
        return None, error


def resolve_project_child_from_root(
    project_dir: Path,
    resolved_root: Path | None,
    root_resolution_error: OSError | None,
    relative_path: str,
    diagnostics: list[str],
    *,
    kind: str = "generated file path",
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"{kind} {relative_path} must be relative")
        return None
    if root_resolution_error is not None:
        diagnostics.append(
            f"SourceTemplate project {project_dir} could not be resolved for "
            f"{kind} {relative_path}: {root_resolution_error}"
        )
        return None
    assert resolved_root is not None
    try:
        resolved = (resolved_root / child_path).resolve()
    except OSError as error:
        diagnostics.append(f"{kind} {relative_path} could not be resolved: {error}")
        return None
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        if kind == "generated file path":
            diagnostics.append(f"{kind} {relative_path} escapes the SourceTemplate project")
        else:
            diagnostics.append(f"{kind} {relative_path} escapes the generated project")
        return None
    return resolved
