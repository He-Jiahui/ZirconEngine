"""Deterministic, auditable size reports for Zircon product bundles.

The report deliberately measures the *published tree* rather than Cargo's
target directory.  Build caches, source files and symbols therefore cannot
silently inflate (or accidentally enter) the Runtime/Editor budget.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping, Sequence


PRODUCT_SIZE_REPORT_SCHEMA_VERSION = 1
PRODUCT_SIZE_REPORT_KIND = "ProductSizeReportV1"
DEFAULT_LARGEST_FILE_COUNT = 20
DEFAULT_CONTRIBUTOR_COUNT = 20
REPORT_SUMMARY_HASH_SCOPE = "canonical-product-summary-v1"
SUPPORTED_PRODUCT_KINDS = frozenset({"runtime", "editor"})


class ProductSizeError(ValueError):
    """Raised when a product tree violates its publication or size policy."""


@dataclass(frozen=True)
class ProductFileEntry:
    path: str
    category: str
    bytes: int
    sha256: str

    def as_dict(self) -> dict[str, object]:
        return {
            "path": self.path,
            "category": self.category,
            "bytes": self.bytes,
            "sha256": self.sha256,
        }


def collect_product_files(
    product_root: Path | str,
    *,
    product_kind: str = "runtime",
    forbidden_path_tokens: Sequence[str] = (),
    include_project_pack: bool = True,
    exclude_paths: Iterable[str] = (),
) -> tuple[ProductFileEntry, ...]:
    """Collect and hash files in stable relative-path order.

    Symlinks/reparse points are rejected instead of followed.  This prevents a
    bundle from escaping its declared root and makes repeated reports
    comparable on Windows and POSIX hosts.
    """

    unresolved_root = Path(product_root).expanduser()
    _assert_safe_output_path(unresolved_root, "product root")
    root = unresolved_root.resolve()
    if root.parent == root:
        raise ProductSizeError(f"refusing to inventory filesystem root as a product: {root}")
    if not root.exists() or not root.is_dir():
        raise ProductSizeError(f"product root does not exist or is not a directory: {root}")
    excluded = {
        _normalize_relative_path(path).casefold() for path in exclude_paths
    }
    entries: list[ProductFileEntry] = []
    seen_paths: set[str] = set()
    for candidate in _walk_product_tree(root):
        relative = candidate.relative_to(root)
        relative_text = _normalize_relative_path(relative.as_posix())
        folded_relative = relative_text.casefold()
        if folded_relative in seen_paths:
            raise ProductSizeError(
                f"product bundle contains case-colliding paths: {relative_text}"
            )
        seen_paths.add(folded_relative)
        if relative_text.casefold() in excluded:
            continue
        if _is_reparse_point(candidate):
            raise ProductSizeError(f"product bundle contains a symlink/reparse point: {relative_text}")
        if not candidate.is_file():
            continue
        category = classify_product_path(
            relative_text,
            product_kind=product_kind,
            forbidden_path_tokens=forbidden_path_tokens,
        )
        if category == "project_pack" and not include_project_pack:
            category = "forbidden"
        entries.append(
            ProductFileEntry(
                path=relative_text,
                category=category,
                bytes=candidate.stat(follow_symlinks=False).st_size,
                sha256=file_sha256(candidate),
            )
        )
    return tuple(entries)


def classify_product_path(
    relative_path: str,
    *,
    product_kind: str = "runtime",
    forbidden_path_tokens: Sequence[str] = (),
) -> str:
    """Classify a bundle path and return ``forbidden`` for policy violations."""

    normalized = _normalize_relative_path(relative_path)
    lowered = normalized.lower()
    parts = tuple(part.lower() for part in normalized.split("/"))
    if _contains_forbidden_path_token(normalized, forbidden_path_tokens):
        return "forbidden"
    # Symbols are a separately published product.  Seeing one in the product
    # tree is therefore a publication violation, not an in-budget file.
    if lowered.endswith((".pdb", ".dbg", ".dsym")):
        return "forbidden"
    if parts and parts[0] in {
        "cache",
        "source",
        "sources",
        "target",
        "targets",
        "symbols",
        "reports",
        "archives",
        "editor-assets",
    }:
        return "forbidden"
    if parts and parts[0] in {"plugins", "plugin"}:
        return "plugin"
    if lowered.endswith((".zrpack", ".pck", ".pak")) or parts and parts[0] in {
        "project-pack",
        "project_pack",
        "project",
    }:
        return "project_pack"
    if lowered.endswith((".json", ".toml", ".manifest", ".manifest.json")):
        return "manifest"
    if product_kind == "editor" and (
        "zircon_editor" in lowered or lowered.endswith("editor.exe")
    ):
        return "editor_binary"
    if lowered.endswith((".exe", ".dll", ".so", ".dylib")):
        return "runtime_binary"
    if parts and parts[0] in {"assets", "asset", "engine-assets", "engine_assets"}:
        return "engine_asset"
    return "payload"


def evaluate_budget(
    total_bytes: int,
    *,
    warning_bytes: int,
    hard_limit_bytes: int,
    hard_gate: bool = True,
) -> dict[str, object]:
    """Evaluate decimal-byte warning and hard limits."""

    if warning_bytes <= 0 or hard_limit_bytes <= 0:
        raise ProductSizeError("warning_bytes and hard_limit_bytes must be positive")
    warning_exceeded = total_bytes > warning_bytes
    hard_exceeded = total_bytes > hard_limit_bytes
    if hard_exceeded and hard_gate:
        status = "fail"
    elif warning_exceeded:
        status = "warning"
    else:
        status = "pass"
    return {
        "warning_bytes": warning_bytes,
        "hard_limit_bytes": hard_limit_bytes,
        "warning_exceeded": warning_exceeded,
        "hard_exceeded": hard_exceeded,
        "hard_gate": hard_gate,
        "passed": not hard_exceeded or not hard_gate,
        "status": status,
    }


def build_product_size_report(
    product_root: Path | str,
    *,
    build_set_id: str,
    product_profile: str,
    product_kind: str,
    target_triple: str,
    cargo_profile: str,
    runtime_features: Sequence[str] = (),
    app_features: Sequence[str] = (),
    plugins: Sequence[str] = (),
    warning_bytes: int,
    hard_limit_bytes: int,
    hard_gate: bool = True,
    forbidden_path_tokens: Sequence[str] = (),
    include_project_pack: bool = True,
    symbols_root: Path | str | None = None,
    project_pack_root: Path | str | None = None,
    archive_path: Path | str | None = None,
    archive_bytes: int | None = None,
    feature_closure: Mapping[str, object] | None = None,
    abi_report: Mapping[str, object] | None = None,
    largest_file_count: int = DEFAULT_LARGEST_FILE_COUNT,
    contributor_count: int = DEFAULT_CONTRIBUTOR_COUNT,
) -> dict[str, object]:
    """Build a JSON-serializable ProductSizeReportV1 payload."""

    if product_kind not in SUPPORTED_PRODUCT_KINDS:
        raise ProductSizeError(
            f"product_kind must be one of {sorted(SUPPORTED_PRODUCT_KINDS)}"
        )
    if type(largest_file_count) is not int or largest_file_count < 0:
        raise ProductSizeError("largest_file_count must be a non-negative integer")
    if type(contributor_count) is not int or contributor_count < 0:
        raise ProductSizeError("contributor_count must be a non-negative integer")
    if feature_closure is not None and not isinstance(feature_closure, Mapping):
        raise ProductSizeError("feature_closure must be an object or None")
    if abi_report is not None and not isinstance(abi_report, Mapping):
        raise ProductSizeError("abi_report must be an object or None")

    product_root_path = Path(product_root).expanduser().resolve()
    if not product_root_path.is_dir():
        raise ProductSizeError(
            f"product root does not exist or is not a directory: {product_root_path}"
        )
    for external_root, label in (
        (symbols_root, "symbols"),
        (project_pack_root, "project pack"),
        (archive_path, "archive"),
    ):
        _assert_outside_product_root(external_root, product_root_path, label)
    entries = collect_product_files(
        product_root,
        product_kind=product_kind,
        forbidden_path_tokens=forbidden_path_tokens,
        include_project_pack=include_project_pack,
    )
    forbidden = [entry for entry in entries if entry.category == "forbidden"]
    total_bytes = sum(entry.bytes for entry in entries if entry.category != "symbols")
    budget = evaluate_budget(
        total_bytes,
        warning_bytes=warning_bytes,
        hard_limit_bytes=hard_limit_bytes,
        hard_gate=hard_gate,
    )
    if forbidden:
        budget = dict(budget)
        budget["status"] = "fail"
        budget["passed"] = False
    symbol_summary = _external_tree_summary(symbols_root, "symbols")
    project_summary = _external_tree_summary(project_pack_root, "project_pack")
    if archive_bytes is None and archive_path is not None:
        archive_candidate = Path(archive_path).expanduser()
        _assert_safe_output_path(archive_candidate, "archive")
        archive_bytes = (
            archive_candidate.stat(follow_symlinks=False).st_size
            if archive_candidate.is_file() and not is_reparse_point(archive_candidate)
            else None
        )
    if archive_bytes is not None and (
        type(archive_bytes) is not int or archive_bytes < 0
    ):
        raise ProductSizeError("archive_bytes must be a non-negative integer or None")
    archive_sha256 = None
    if archive_path is not None:
        archive_candidate = Path(archive_path).expanduser()
        _assert_safe_output_path(archive_candidate, "archive")
        if archive_candidate.is_file() and not is_reparse_point(archive_candidate):
            archive_sha256 = file_sha256(archive_candidate)
    sorted_entries = sorted(entries, key=lambda entry: (-entry.bytes, entry.path))
    contributors = _category_contributors(entries, contributor_count)
    closure_status = _feature_closure_status(feature_closure)
    feature_closure_gate = {
        "status": closure_status,
        "passed": closure_status in {None, "core-only"},
        "required_for_final_release": feature_closure is not None,
    }
    report: dict[str, object] = {
        "schema_version": PRODUCT_SIZE_REPORT_SCHEMA_VERSION,
        "report_kind": PRODUCT_SIZE_REPORT_KIND,
        "build_set_id": _require_digest(build_set_id, "build_set_id"),
        "product_profile": _require_text(product_profile, "product_profile"),
        "product_kind": _require_text(product_kind, "product_kind"),
        "target": _target_summary(target_triple),
        "cargo_profile": _require_text(cargo_profile, "cargo_profile"),
        "features": {
            "runtime": sorted({_require_text(value, "runtime feature") for value in runtime_features}),
            "app": sorted({_require_text(value, "app feature") for value in app_features}),
        },
        "plugins": sorted({_require_text(value, "plugin id") for value in plugins}),
        "feature_closure": dict(feature_closure) if feature_closure is not None else None,
        "feature_closure_gate": feature_closure_gate,
        "abi": dict(abi_report) if abi_report is not None else None,
        "files": [entry.as_dict() for entry in sorted(entries, key=lambda item: item.path)],
        "totals": {
            "file_count": len(entries),
            "uncompressed_bytes": total_bytes,
            "uncompressed_mib": round(total_bytes / (1024 * 1024), 6),
            "compressed_bytes": archive_bytes,
            "compressed_mib": (
                round(archive_bytes / (1024 * 1024), 6)
                if archive_bytes is not None
                else None
            ),
        },
        "budget": budget,
        "largest_files": [
            entry.as_dict() for entry in sorted_entries[: max(0, largest_file_count)]
        ],
        "category_contributors": contributors,
        # ``contributors`` is the concise public spelling; retain the
        # category-qualified key for consumers that want to distinguish this
        # aggregate from a future baseline-delta list.
        "contributors": contributors,
        "forbidden_paths": [entry.path for entry in forbidden],
        "symbols": symbol_summary,
        "project_pack": project_summary,
        "archive": {
            "path": _portable_optional_path(archive_path),
            "bytes": archive_bytes,
            "sha256": archive_sha256,
        },
    }
    # A byte-budget violation is the primary publication failure.  Preserve a
    # separate feature-closure gate for the architectural debt, but do not let
    # ``needs-split`` hide a hard size failure in the headline status.
    publication_status = str(budget["status"])
    if bool(budget["passed"]) and closure_status == "needs-split":
        publication_status = "needs-split"
    report["publication"] = {
        "status": publication_status,
        "size_passed": bool(budget["passed"]),
        "feature_closure_passed": bool(feature_closure_gate["passed"]),
        "passed": bool(budget["passed"])
        and bool(feature_closure_gate["passed"]),
    }
    summary = product_report_summary(report)
    report["summary"] = summary
    report["summary_sha256"] = product_report_summary_sha256(report)
    report["summary_hash_scope"] = REPORT_SUMMARY_HASH_SCOPE
    return report


def product_report_summary(report: Mapping[str, object]) -> dict[str, object]:
    """Project stable publication facts for manifest linkage.

    The summary intentionally excludes per-file hashes and archive bytes. A
    manifest carries this digest, and its own report-reference field changes
    those hashes; excluding the self-referential fields gives us a stable,
    independently verifiable identity while retaining the full inventory in
    ``files``.
    """

    totals = report.get("totals")
    budget = report.get("budget")
    if not isinstance(totals, Mapping) or not isinstance(budget, Mapping):
        raise ProductSizeError("product size report is missing totals or budget")
    target = report.get("target")
    features = report.get("features")
    symbols = report.get("symbols")
    project_pack = report.get("project_pack")
    feature_closure = report.get("feature_closure")
    feature_closure_gate = report.get("feature_closure_gate")
    abi = report.get("abi")
    publication = report.get("publication")
    return {
        "schema_version": report.get("schema_version"),
        "report_kind": report.get("report_kind"),
        "build_set_id": report.get("build_set_id"),
        "product_profile": report.get("product_profile"),
        "product_kind": report.get("product_kind"),
        "target": dict(target) if isinstance(target, Mapping) else target,
        "cargo_profile": report.get("cargo_profile"),
        "features": dict(features) if isinstance(features, Mapping) else features,
        "plugins": list(report.get("plugins", []))
        if isinstance(report.get("plugins"), list)
        else report.get("plugins"),
        "totals": {
            "file_count": totals.get("file_count"),
            "uncompressed_bytes": totals.get("uncompressed_bytes"),
        },
        "budget": {
            key: budget.get(key)
            for key in (
                "warning_bytes",
                "hard_limit_bytes",
                "warning_exceeded",
                "hard_exceeded",
                "hard_gate",
                "passed",
                "status",
            )
        },
        "category_contributors": list(report.get("category_contributors", []))
        if isinstance(report.get("category_contributors"), list)
        else report.get("category_contributors"),
        "contributors": list(report.get("contributors", []))
        if isinstance(report.get("contributors"), list)
        else report.get("contributors"),
        "forbidden_paths": list(report.get("forbidden_paths", []))
        if isinstance(report.get("forbidden_paths"), list)
        else report.get("forbidden_paths"),
        "symbols": dict(symbols) if isinstance(symbols, Mapping) else symbols,
        "project_pack": dict(project_pack)
        if isinstance(project_pack, Mapping)
        else project_pack,
        "feature_closure": _stable_feature_closure_summary(feature_closure),
        "feature_closure_gate": dict(feature_closure_gate)
        if isinstance(feature_closure_gate, Mapping)
        else feature_closure_gate,
        "abi": _stable_abi_summary(abi),
        "publication": dict(publication)
        if isinstance(publication, Mapping)
        else publication,
    }


def product_report_summary_sha256(report: Mapping[str, object]) -> str:
    """Hash the canonical summary projection used by product manifests."""

    encoded = json.dumps(
        product_report_summary(report),
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def product_report_reference(
    report_path: Path | str,
    report: Mapping[str, object],
) -> dict[str, object]:
    """Return the portable report reference embedded in product manifests.

    ``sha256`` is deliberately the canonical summary hash rather than a hash
    of the complete JSON file. The latter would create an impossible
    self-reference because the manifest is itself one of the inventoried
    product files. ``hash_scope`` makes this distinction explicit to readers
    and downstream validators.
    """

    summary = product_report_summary(report)
    digest = product_report_summary_sha256(report)
    totals = summary["totals"]
    budget = summary["budget"]
    publication = summary.get("publication")
    return {
        "path": _portable_report_reference_path(report_path),
        "sha256": digest,
        "summary_sha256": digest,
        "hash_scope": REPORT_SUMMARY_HASH_SCOPE,
        "summary": {
            "file_count": totals["file_count"],
            "uncompressed_bytes": totals["uncompressed_bytes"],
            "warning_bytes": budget["warning_bytes"],
            "hard_limit_bytes": budget["hard_limit_bytes"],
            "status": budget["status"],
            "passed": budget["passed"],
            "publication_status": publication.get("status")
            if isinstance(publication, Mapping)
            else None,
            "publication_passed": publication.get("passed")
            if isinstance(publication, Mapping)
            else None,
        },
    }


def _portable_report_reference_path(value: Path | str | None) -> str | None:
    """Render a report path from the product tree without exposing its host root."""

    if value is None:
        return None
    raw = str(value).replace("\\", "/")
    components = [part for part in raw.split("/") if part]
    report_index = next(
        (index for index, component in enumerate(components) if component.casefold() == "reports"),
        None,
    )
    if report_index is not None and report_index + 1 < len(components):
        return "../" + "/".join(components[report_index:])
    return _portable_optional_path(value)


def write_product_size_report(
    report_path: Path | str,
    product_root: Path | str,
    **kwargs: object,
) -> dict[str, object]:
    """Build and atomically write a deterministic report."""

    unresolved_destination = Path(report_path).expanduser()
    _assert_safe_output_path(unresolved_destination, "size report")
    destination = unresolved_destination.resolve()
    product_root_path = Path(product_root).expanduser().resolve()
    _assert_outside_product_root(destination, product_root_path, "size report")
    report = build_product_size_report(product_root, **kwargs)  # type: ignore[arg-type]
    if is_reparse_point(destination) or is_reparse_point(destination.parent):
        raise ProductSizeError(f"size report destination contains a symlink/reparse point: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(destination.suffix + ".tmp")
    if is_reparse_point(temporary):
        raise ProductSizeError(f"size report temporary path contains a symlink/reparse point: {temporary}")
    try:
        temporary.write_text(
            json.dumps(report, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        os.replace(temporary, destination)
    finally:
        if temporary.exists() or is_reparse_point(temporary):
            if is_reparse_point(temporary):
                raise ProductSizeError(
                    f"size report temporary path became a symlink/reparse point: {temporary}"
                )
            temporary.unlink()
    return report


def write_deterministic_archive(
    product_root: Path | str,
    archive_path: Path | str,
    *,
    exclude_paths: Iterable[str] = (),
    product_kind: str = "runtime",
    forbidden_path_tokens: Sequence[str] = (),
    include_project_pack: bool = True,
) -> int:
    """Create a reproducible ZIP with fixed order, timestamp and permissions."""

    unresolved_root = Path(product_root).expanduser()
    _assert_safe_output_path(unresolved_root, "product root")
    root = unresolved_root.resolve()
    unresolved_archive = Path(archive_path).expanduser()
    _assert_safe_output_path(unresolved_archive, "archive")
    archive = unresolved_archive.resolve()
    if not root.is_dir():
        raise ProductSizeError(f"cannot archive missing product root: {root}")
    try:
        archive.relative_to(root)
    except ValueError:
        pass
    else:
        raise ProductSizeError(
            f"archive path must remain outside the product root: {archive}"
        )
    excluded = {
        _normalize_relative_path(path).casefold() for path in exclude_paths
    }
    files = [
        path
        for path in _walk_product_tree(root)
        if not _path_is_excluded(
            _normalize_relative_path(path.relative_to(root).as_posix()).casefold(),
            excluded,
        )
    ]
    seen_paths: set[str] = set()
    for source in files:
        relative = _normalize_relative_path(source.relative_to(root).as_posix())
        folded_relative = relative.casefold()
        if folded_relative in seen_paths:
            raise ProductSizeError(
                f"cannot archive case-colliding product paths: {relative}"
            )
        seen_paths.add(folded_relative)
        if _is_reparse_point(source):
            raise ProductSizeError(f"cannot archive a symlink/reparse point: {relative}")
        category = classify_product_path(
            relative,
            product_kind=product_kind,
            forbidden_path_tokens=forbidden_path_tokens,
        )
        if category == "forbidden" or (
            category == "project_pack" and not include_project_pack
        ):
            raise ProductSizeError(f"cannot archive forbidden product path: {relative}")
    archive.parent.mkdir(parents=True, exist_ok=True)
    temporary = archive.with_suffix(archive.suffix + ".tmp")
    if is_reparse_point(temporary):
        raise ProductSizeError(
            f"archive temporary path contains a symlink/reparse point: {temporary}"
        )
    try:
        with zipfile.ZipFile(
            temporary,
            "w",
            compression=zipfile.ZIP_DEFLATED,
            compresslevel=9,
        ) as output:
            for source in files:
                relative = _normalize_relative_path(source.relative_to(root).as_posix())
                info = zipfile.ZipInfo(relative, date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o644 << 16
                info.create_system = 0
                output.writestr(info, source.read_bytes())
        os.replace(temporary, archive)
    finally:
        if temporary.exists() or is_reparse_point(temporary):
            if is_reparse_point(temporary):
                raise ProductSizeError(
                    f"archive temporary path became a symlink/reparse point: {temporary}"
                )
            temporary.unlink()
    return archive.stat().st_size


def enforce_product_size_report(
    report: Mapping[str, object], *, require_core_feature_closure: bool = False
) -> None:
    """Raise a useful error when a report fails its publication gate."""

    budget = report.get("budget")
    if not isinstance(budget, Mapping):
        raise ProductSizeError("product size report has no budget object")
    if budget.get("status") == "fail" or budget.get("passed") is False:
        total = report.get("totals", {})
        forbidden = report.get("forbidden_paths", [])
        raise ProductSizeError(
            "product size gate failed: "
            f"uncompressed_bytes={total.get('uncompressed_bytes') if isinstance(total, Mapping) else '?'} "
            f"hard_limit_bytes={budget.get('hard_limit_bytes')} "
            f"forbidden_paths={forbidden}"
        )
    if require_core_feature_closure:
        gate = report.get("feature_closure_gate")
        if not isinstance(gate, Mapping) or gate.get("passed") is not True:
            raise ProductSizeError(
                "product feature-closure gate failed: shipping still includes "
                "heavy modules; split the runtime-api/graphics boundary before "
                "promoting the final size gate"
            )


def file_sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def is_reparse_point(path: Path) -> bool:
    """Return true for symlinks and Windows reparse-point directories/files."""

    if path.is_symlink():
        return True
    try:
        attributes = os.lstat(path).st_file_attributes
    except (OSError, AttributeError):
        return False
    return bool(attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400))


# Private compatibility name retained for the local walkers and older callers.
_is_reparse_point = is_reparse_point


def _walk_product_tree(root: Path) -> tuple[Path, ...]:
    """Return regular files without traversing links or reparse points."""

    if _is_reparse_point(root):
        raise ProductSizeError(f"product root is a symlink/reparse point: {root}")
    files: list[Path] = []

    def visit(directory: Path) -> None:
        try:
            with os.scandir(directory) as iterator:
                entries = sorted(iterator, key=lambda entry: entry.name.casefold())
        except OSError as error:
            raise ProductSizeError(f"could not inspect product directory {directory}: {error}") from error
        for entry in entries:
            candidate = Path(entry.path)
            if _is_reparse_point(candidate):
                relative = candidate.relative_to(root).as_posix()
                raise ProductSizeError(
                    f"product bundle contains a symlink/reparse point: {relative}"
                )
            try:
                if entry.is_dir(follow_symlinks=False):
                    visit(candidate)
                elif entry.is_file(follow_symlinks=False):
                    files.append(candidate)
            except OSError as error:
                raise ProductSizeError(f"could not inspect product path {candidate}: {error}") from error

    visit(root)
    return tuple(sorted(files, key=lambda path: path.relative_to(root).as_posix()))


def _external_tree_summary(root: Path | str | None, category: str) -> dict[str, object] | None:
    if root is None:
        return None
    unresolved_path = Path(root).expanduser()
    _assert_safe_output_path(unresolved_path, f"external {category}")
    path = unresolved_path.resolve()
    if not path.exists():
        return {"path": _portable_optional_path(root), "file_count": 0, "bytes": 0, "sha256": None}
    if not path.is_dir():
        raise ProductSizeError(f"external {category} root is not a directory: {path}")
    entries = collect_product_files(unresolved_path, product_kind="external")
    digest = hashlib.sha256()
    for entry in entries:
        digest.update(entry.path.encode("utf-8"))
        digest.update(b"\0")
        digest.update(entry.sha256.encode("ascii"))
        digest.update(b"\n")
    return {
        "path": _portable_optional_path(root),
        "file_count": len(entries),
        "bytes": sum(entry.bytes for entry in entries),
        "sha256": digest.hexdigest(),
    }


def _assert_safe_output_path(path: Path, label: str) -> None:
    """Reject output paths that would write through an existing reparse point."""

    current = path
    if not current.is_absolute():
        current = Path.cwd() / current
    while current != current.parent:
        if is_reparse_point(current):
            raise ProductSizeError(f"{label} path contains a symlink/reparse point: {current}")
        current = current.parent


def _assert_outside_product_root(
    candidate: Path | str | None,
    product_root: Path,
    label: str,
) -> None:
    """Require separately published artifacts to stay outside the product tree."""

    if candidate is None:
        return
    path = Path(candidate).expanduser().resolve()
    try:
        path.relative_to(product_root)
    except ValueError:
        return
    raise ProductSizeError(f"{label} path must remain outside the product root: {path}")


def _category_contributors(
    entries: Sequence[ProductFileEntry], count: int
) -> list[dict[str, object]]:
    totals: dict[str, int] = {}
    for entry in entries:
        if entry.category == "symbols":
            continue
        totals[entry.category] = totals.get(entry.category, 0) + entry.bytes
    return [
        {"category": category, "bytes": bytes_value}
        for category, bytes_value in sorted(
            totals.items(), key=lambda item: (-item[1], item[0])
        )[: max(0, count)]
    ]


def _normalize_relative_path(value: str) -> str:
    portable = str(value).replace("\\", "/")
    if (
        not portable
        or portable.startswith("/")
        or ":" in portable
        or any(ord(character) < 0x20 for character in portable)
    ):
        raise ProductSizeError(f"path must be relative and portable: {value!r}")
    raw_components = portable.split("/")
    components = [component for component in raw_components if component]
    if (
        not components
        or len(components) != len(raw_components)
        or any(component in {".", ".."} for component in components)
    ):
        raise ProductSizeError(f"path contains an unsafe component: {value!r}")
    return "/".join(components)


def _target_summary(target_triple: object) -> dict[str, object]:
    """Expand the stable target triple into auditable platform dimensions."""

    triple = _require_text(target_triple, "target_triple")
    parts = triple.lower().split("-")
    architecture = {
        "x86_64": "x86_64",
        "aarch64": "aarch64",
        "i686": "x86",
    }.get(parts[0], parts[0])
    if "windows" in parts:
        operating_system = "windows"
    elif "darwin" in parts or "macos" in parts:
        operating_system = "macos"
    elif "linux" in parts:
        operating_system = "linux"
    else:
        operating_system = parts[2] if len(parts) > 2 else "unknown"
    pointer_width = 64 if architecture in {"x86_64", "aarch64"} else 32
    return {
        "triple": triple,
        "target_triple": triple,
        "architecture": architecture,
        "operating_system": operating_system,
        "pointer_width": pointer_width,
    }


def _portable_optional_path(value: Path | str | None) -> str | None:
    if value is None:
        return None
    # Reports may refer to a separately published artifact, but never leak an
    # absolute host path into the product metadata.
    raw = str(value).replace("\\", "/")
    if not raw:
        return None
    # pathlib on POSIX does not recognize a Windows drive prefix as absolute;
    # handle both forms explicitly before attempting relative normalization.
    if raw.startswith("/") or (len(raw) >= 2 and raw[1] == ":"):
        return Path(raw).name or None
    try:
        return _normalize_relative_path(raw)
    except ProductSizeError:
        return Path(raw).name or None


def _contains_forbidden_path_token(
    normalized_path: str, forbidden_path_tokens: Sequence[str]
) -> bool:
    """Match policy tokens on path-component boundaries.

    A plain substring check would reject legitimate names such as
    ``assets/resources`` when the policy token is ``source``.  Tokens are
    therefore interpreted as one or more path components; a component may
    also carry an extension (``zircon_hub.exe``).
    """

    components = tuple(part.lower() for part in normalized_path.split("/"))
    for raw_token in forbidden_path_tokens:
        token = str(raw_token).strip().replace("\\", "/").strip("/").lower()
        if not token:
            continue
        token_parts = tuple(part for part in token.split("/") if part)
        if not token_parts:
            continue
        width = len(token_parts)
        for start in range(0, len(components) - width + 1):
            window = components[start : start + width]
            if window == token_parts:
                return True
            if width == 1 and window[0].startswith(token_parts[0] + "."):
                return True
    return False


def _path_is_excluded(relative_path: str, excluded: set[str]) -> bool:
    """Match archive exclusions on complete path-component boundaries."""
    return any(
        relative_path == candidate or relative_path.startswith(candidate + "/")
        for candidate in excluded
    )


def _require_text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ProductSizeError(f"{label} must be non-empty text")
    return value.strip()


def _require_digest(value: object, label: str) -> str:
    text = _require_text(value, label).lower()
    if len(text) != 64 or any(character not in "0123456789abcdef" for character in text):
        raise ProductSizeError(f"{label} must be a SHA-256 hexadecimal digest")
    return text


def _feature_closure_status(value: Mapping[str, object] | None) -> str | None:
    if value is None:
        return None
    status = value.get("status")
    return status.strip() if isinstance(status, str) and status.strip() else "unknown"


def _stable_feature_closure_summary(value: object) -> dict[str, object] | None:
    if not isinstance(value, Mapping):
        return None
    heavy = value.get("heavy_features", [])
    return {
        "status": _feature_closure_status(value),
        "heavy_features": sorted(
            {item.strip() for item in heavy if isinstance(item, str) and item.strip()}
        )
        if isinstance(heavy, list)
        else [],
    }


def _stable_abi_summary(value: object) -> dict[str, object] | None:
    if not isinstance(value, Mapping):
        return None
    # Keep the manifest linkage sensitive to the native closure and machine,
    # while intentionally excluding per-file hashes and host-local paths.
    summary: dict[str, object] = {}
    for key in (
        "target_triple",
        "machine",
        "machine_hex",
        "machine_name",
        "bitness",
        "imports",
        "delay_imports",
        "unexpected_imports",
        "forwarded_exports",
        "passed",
    ):
        if key in value:
            item = value[key]
            if isinstance(item, list):
                summary[key] = sorted(item)
            else:
                summary[key] = item
    for key in ("runtime", "host", "native_images"):
        nested = value.get(key)
        if isinstance(nested, Mapping):
            summary[key] = _stable_abi_summary(nested)
        elif isinstance(nested, list):
            summary[key] = [
                _stable_abi_summary(item) if isinstance(item, Mapping) else item
                for item in nested
            ]
    return summary
