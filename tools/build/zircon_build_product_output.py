"""Final staging, archive and size-gate ownership for product builds."""

from __future__ import annotations

import shutil
import os
from pathlib import Path
from typing import Any

from .zircon_build_abi import (
    AbiValidationError,
    validate_product_pe_image,
    validate_runtime_abi,
)
from .zircon_build_feature_closure import audit_product_feature_closure
from .zircon_build_size_report import (
    ProductSizeError,
    collect_product_files,
    enforce_product_size_report,
    is_reparse_point,
    product_report_reference,
    write_deterministic_archive,
    write_product_size_report,
)


def clean_product_output(config: object) -> None:
    """Remove only the exact generation roots owned by a shipping invocation."""

    unresolved_out_root = Path(config.out_root).expanduser()
    _assert_no_reparse_ancestors(unresolved_out_root, "product output root")
    out_root = unresolved_out_root.resolve()
    _assert_safe_product_root(out_root)
    owned_paths = [
        Path(config.engine_root),
        Path(config.symbols_root),
        Path(config.reports_root),
    ]
    editor_asset_root = getattr(config, "editor_asset_root", None)
    if editor_asset_root is not None:
        owned_paths.append(Path(editor_asset_root))
    archive_path = Path(config.archive_path)
    owned_paths.append(archive_path)
    for path in owned_paths:
        _assert_owned_child(out_root, path)
    if getattr(config, "dry_run", False):
        print(f"DRY-RUN reset {config.engine_root}")
        print(f"DRY-RUN reset {config.symbols_root}")
        print(f"DRY-RUN reset {config.reports_root}")
        if editor_asset_root is not None:
            print(f"DRY-RUN reset {editor_asset_root}")
        print(f"DRY-RUN reset {config.archive_path}")
        return
    for path in owned_paths[:-1]:
        # ``exists`` misses broken symlinks; include them so the preflight
        # below rejects rather than silently preserving a redirector.
        if path.exists() or path.is_symlink() or is_reparse_point(path):
            _assert_tree_without_reparse(path)
            if path.is_dir() and not is_reparse_point(path):
                shutil.rmtree(path)
            else:
                path.unlink()
    archive = owned_paths[-1]
    if archive.exists() or archive.is_symlink() or is_reparse_point(archive):
        if is_reparse_point(archive):
            raise ProductSizeError(f"refusing to remove archive reparse point: {archive}")
        if archive.is_dir():
            raise ProductSizeError(f"refusing to remove archive directory: {archive}")
        archive.unlink()


def finalize_product_output(config: object) -> dict[str, object] | None:
    """Write the size report/archive for a product profile and enforce its gate."""

    profile = getattr(config, "product_profile", None)
    if profile is None:
        return None
    if getattr(config, "dry_run", False):
        print(f"DRY-RUN write product size report {config.size_report_path}")
        if profile.archive:
            print(f"DRY-RUN archive {config.engine_root} -> {config.archive_path}")
        return None

    # A manifest carries a canonical report-summary digest.  The manifests are
    # themselves inventoried product files, so update them and regenerate the
    # archive/report until that summary reaches a stable fixed point.  The
    # summary projection intentionally excludes per-file hashes and archive
    # bytes, avoiding a cryptographic self-reference while keeping the full
    # inventory auditable.
    try:
        from .zircon_build_runtime_manifest import write_runtime_artifact_manifest
        from .zircon_build_staging_manifest import write_staging_manifest
    except ImportError:  # pragma: no cover - direct script import path.
        from .zircon_build_runtime_manifest import write_runtime_artifact_manifest
        from .zircon_build_staging_manifest import write_staging_manifest

    build_set_id = getattr(config, "build_set_id", None)
    if not isinstance(build_set_id, str) or not build_set_id:
        build_set_id = profile.build_set_id
    cargo_profile = getattr(config, "cargo_profile_name", profile.cargo_profile)
    if callable(cargo_profile):
        cargo_profile = cargo_profile()
    app_features = getattr(config, "app_feature_arg", " ".join(profile.app_features))
    if callable(app_features):
        app_features = app_features()

    # Validate the required host/DLL pair before writing any derived report or
    # archive.  A size-only pass must never be able to publish an incomplete
    # product or hide a wrong-architecture native dependency.
    try:
        abi_report = _validate_product_abis(config, profile)
    except AbiValidationError as error:
        raise SystemExit(f"could not validate shipping PE closure: {error}") from error

    report: dict[str, object] | None = None
    previous_reference: dict[str, object] | None = None
    for _attempt in range(4):
        archive_bytes: int | None = None
        if profile.archive:
            archive_bytes = write_deterministic_archive(
                config.engine_root,
                config.archive_path,
                product_kind=profile.product_kind,
                forbidden_path_tokens=profile.forbidden_path_tokens,
                include_project_pack=profile.include_project_pack,
            )
        try:
            feature_closure = audit_product_feature_closure(
                config.repo_root,
                app_features=profile.app_features,
                runtime_features=profile.runtime_features,
            )
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise SystemExit(f"could not audit shipping feature closure: {error}") from error
        report = write_product_size_report(
            config.size_report_path,
            config.engine_root,
            build_set_id=build_set_id,
            product_profile=profile.name,
            product_kind=profile.product_kind,
            target_triple=profile.target_triple,
            cargo_profile=cargo_profile,
            runtime_features=getattr(config, "runtime_features", profile.runtime_features),
            app_features=str(app_features).split(),
            plugins=[package.plugin_id for package in getattr(config, "plugins", ())],
            warning_bytes=profile.warning_bytes,
            hard_limit_bytes=profile.hard_limit_bytes,
            hard_gate=profile.hard_gate,
            forbidden_path_tokens=profile.forbidden_path_tokens,
            include_project_pack=profile.include_project_pack,
            symbols_root=config.symbols_root,
            project_pack_root=getattr(config, "project_pack_root", None),
            archive_path=config.archive_path if profile.archive else None,
            archive_bytes=archive_bytes,
            feature_closure=feature_closure,
            abi_report=abi_report,
        )
        current_reference = product_report_reference(config.size_report_path, report)
        if current_reference == previous_reference:
            break
        previous_reference = current_reference
        write_runtime_artifact_manifest(
            config, product_report_reference=current_reference
        )
        write_staging_manifest(config, product_report_reference=current_reference)
    else:
        raise SystemExit(
            "product report summary did not stabilize while linking manifests"
        )
    assert report is not None
    try:
        enforce_product_size_report(report)
    except ProductSizeError as error:
        raise SystemExit(str(error)) from error
    print(
        "Product size report: "
        f"{report['totals']['uncompressed_bytes']} bytes "
        f"size_status={report['budget']['status']} "
        f"publication_status={report['publication']['status']} "
        f"path={config.size_report_path}"
    )
    return report


def _validate_product_abis(config: object, profile: object) -> dict[str, object]:
    """Validate every native image in the staged product tree.

    Runtime's v8 export/no-unwind contract receives the richer validator.  The
    host and optional plugin images use the same PE machine/import policy, but
    do not need a particular export table.  Keeping this at the final product
    boundary catches an x86 or host-installed DLL even when the byte budget is
    otherwise healthy.
    """

    engine_root = Path(config.engine_root)
    runtime_name = str(profile.runtime_library).replace("\\", "/")
    runtime_path = engine_root.joinpath(*runtime_name.split("/"))
    host_name = str(profile.app_binary).replace("\\", "/")
    if _target_is_windows(str(profile.target_triple)) and not host_name.lower().endswith(
        ".exe"
    ):
        host_name += ".exe"
    host_path = engine_root.joinpath(*host_name.split("/"))
    missing = [
        path
        for path in (runtime_path, host_path)
        if not path.is_file() or is_reparse_point(path)
    ]
    if missing:
        rendered = ", ".join(str(path) for path in missing)
        raise AbiValidationError(f"required product artifact(s) are missing or unsafe: {rendered}")

    expected_machine = str(profile.target_triple)
    runtime_report = validate_runtime_abi(
        runtime_path,
        repo_root=config.repo_root,
        product_root=engine_root,
        expected_machine=expected_machine,
        expected_kind="dll",
    )
    host_report = validate_product_pe_image(
        host_path,
        product_root=engine_root,
        expected_machine=expected_machine,
        expected_kind="exe",
    )

    # Build a secure inventory once and validate every staged PE image.  The
    # runtime DLL is already covered above; retaining its entry separately
    # gives consumers a stable ``runtime``/``host``/``native_images`` shape.
    native_images: list[dict[str, object]] = []
    for entry in collect_product_files(engine_root, product_kind=str(profile.product_kind)):
        suffix = Path(entry.path).suffix.lower()
        if suffix not in {".dll", ".exe", ".ocx", ".sys"}:
            continue
        candidate = engine_root.joinpath(*entry.path.split("/"))
        if candidate.resolve() == runtime_path.resolve() or candidate.resolve() == host_path.resolve():
            continue
        native_images.append(
            validate_product_pe_image(
                candidate,
                product_root=engine_root,
                expected_machine=expected_machine,
                expected_kind=_expected_pe_kind(suffix),
            )
        )
    return {
        "runtime": runtime_report,
        "host": host_report,
        "native_images": native_images,
        "target_triple": expected_machine,
    }


def _target_is_windows(target_triple: str) -> bool:
    return "-windows-" in target_triple.lower() or target_triple.lower().endswith("-windows")


def _expected_pe_kind(suffix: str) -> str | None:
    """Return the PE shape implied by a published native-image suffix.

    OCX files are DLL-form ActiveX controls.  System drivers use the PE
    ``IMAGE_FILE_SYSTEM`` characteristic instead of necessarily advertising
    ``IMAGE_FILE_DLL``, so the suffix alone must not force them to look like an
    executable.  Machine/import checks still apply to every native image.
    """

    if suffix in {".dll", ".ocx"}:
        return "dll"
    if suffix == ".exe":
        return "exe"
    return None


def _assert_safe_product_root(path: Path) -> None:
    """Reject drive roots and shared managed roots as destructive targets."""

    if path.parent == path or path.name.lower() in {
        "",
        "cargo-targets",
        "targets",
        "zirconbuilds",
    }:
        raise ProductSizeError(f"refusing to clean unsafe product output root: {path}")


def _assert_owned_child(out_root: Path, candidate: Path) -> None:
    candidate = Path(candidate).expanduser()
    _assert_no_reparse_ancestors(candidate, "product output path")
    resolved_out_root = out_root.resolve()
    resolved_candidate = candidate.resolve()
    if resolved_candidate == resolved_out_root:
        raise ProductSizeError(
            f"product output path must be a child of its output root: {candidate}"
        )
    try:
        resolved_candidate.relative_to(resolved_out_root)
    except ValueError as error:
        raise ProductSizeError(
            f"product output path escapes its output root: {candidate}"
        ) from error


def _assert_no_reparse_ancestors(path: Path, label: str) -> None:
    current = path
    if not current.is_absolute():
        current = Path.cwd() / current
    while True:
        if is_reparse_point(current):
            raise ProductSizeError(f"{label} contains a symlink/reparse point: {current}")
        if current.parent == current:
            break
        current = current.parent


def _assert_tree_without_reparse(path: Path) -> None:
    """Preflight an owned tree before deletion so a junction cannot redirect rmtree."""

    if is_reparse_point(path):
        raise ProductSizeError(f"refusing to remove product reparse point: {path}")
    if not path.is_dir():
        return
    try:
        entries = list(os.scandir(path))
    except OSError as error:
        raise ProductSizeError(f"could not inspect product output {path}: {error}") from error
    for entry in entries:
        child = Path(entry.path)
        if is_reparse_point(child):
            raise ProductSizeError(f"refusing to remove nested product reparse point: {child}")
        if entry.is_dir(follow_symlinks=False):
            _assert_tree_without_reparse(child)
