"""Cargo feature-closure auditing for build-only Zircon product profiles.

Cargo's ``--features`` argument is intentionally small in the product profile,
but a local feature can expand to other local features and dependency feature
edges.  This module resolves that graph without invoking Cargo so the shipping
report can show exactly which boundary is still waiting for a real split.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping, Sequence

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]


SHIPPING_HEAVY_FEATURES = frozenset(
    {
        "text",
        "ui",
        "script",
        "navigation",
        "animation",
        "ai-contracts",
        "net-contracts",
        "physics-contracts",
        "sound-contracts",
        "default-platform",
        "platform-x11",
        "platform-wayland",
        "input-gamepad",
        "gamepad-gilrs",
        "first-party-runtime-plugins",
        "first-party-advanced-render-runtime-plugins",
        "first-party-navigation-runtime-plugin",
        "first-party-ui-document-importer",
        "dev-dynamic-linking",
    }
)


@dataclass(frozen=True)
class FeatureClosure:
    package: str
    requested: tuple[str, ...]
    local_features: tuple[str, ...]
    dependency_edges: tuple[str, ...]
    heavy_features: tuple[str, ...]

    @property
    def status(self) -> str:
        return "needs-split" if self.heavy_features else "core-only"

    def as_dict(self) -> dict[str, object]:
        return {
            "package": self.package,
            "requested": list(self.requested),
            "local_features": list(self.local_features),
            "dependency_edges": list(self.dependency_edges),
            "heavy_features": list(self.heavy_features),
            "status": self.status,
        }


def resolve_feature_closure(
    manifest_path: Path | str,
    requested: Iterable[str],
    *,
    package_name: str | None = None,
) -> FeatureClosure:
    """Resolve local Cargo feature aliases and record dependency edges."""

    if tomllib is None:  # pragma: no cover
        raise RuntimeError("Python 3.11 or newer is required to parse Cargo manifests")
    path = Path(manifest_path).expanduser().resolve()
    with path.open("rb") as handle:
        document = tomllib.load(handle)
    raw_features = document.get("features", {})
    if not isinstance(raw_features, Mapping):
        raise ValueError(f"Cargo manifest {path} features must be a table")
    requested_names = tuple(dict.fromkeys(_clean_name(value) for value in requested if _clean_name(value)))
    local: set[str] = set()
    dependency_edges: set[str] = set()
    pending = list(requested_names)
    while pending:
        feature = pending.pop()
        if feature in local:
            continue
        local.add(feature)
        values = raw_features.get(feature, [])
        if not isinstance(values, list):
            continue
        for raw_edge in values:
            if not isinstance(raw_edge, str) or not raw_edge.strip():
                continue
            edge = raw_edge.strip()
            if edge.startswith("dep:"):
                dependency_edges.add(edge)
                continue
            if "/" in edge:
                dependency_edges.add(edge)
                local_name = edge.split("/", 1)[0].removesuffix("?")
                if local_name in raw_features:
                    pending.append(local_name)
                continue
            if edge in raw_features:
                pending.append(edge)
            else:
                dependency_edges.add(edge)
    heavy = tuple(sorted(feature for feature in local if feature in SHIPPING_HEAVY_FEATURES))
    package_table = document.get("package")
    inferred_package = (
        package_table.get("name", path.stem)
        if isinstance(package_table, Mapping)
        else path.stem
    )
    return FeatureClosure(
        package=package_name or str(inferred_package),
        requested=tuple(sorted(requested_names)),
        local_features=tuple(sorted(local)),
        dependency_edges=tuple(sorted(dependency_edges)),
        heavy_features=heavy,
    )


def audit_product_feature_closure(
    repo_root: Path | str,
    *,
    app_features: Sequence[str],
    runtime_features: Sequence[str],
) -> dict[str, object]:
    """Return an auditable app/runtime closure summary."""

    root = Path(repo_root).expanduser().resolve()
    app = resolve_feature_closure(
        root / "zircon_app" / "Cargo.toml",
        app_features,
        package_name="zircon_app",
    )
    runtime = resolve_feature_closure(
        root / "zircon_runtime" / "Cargo.toml",
        runtime_features,
        package_name="zircon_runtime",
    )
    heavy = sorted(set(app.heavy_features) | set(runtime.heavy_features))
    return {
        "status": "needs-split" if heavy else "core-only",
        "heavy_features": heavy,
        "packages": [app.as_dict(), runtime.as_dict()],
    }


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=Path(__file__).resolve().parents[2])
    parser.add_argument("--app-features", required=True)
    parser.add_argument("--runtime-features", required=True)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    result = audit_product_feature_closure(
        args.repo_root,
        app_features=_split_features(args.app_features),
        runtime_features=_split_features(args.runtime_features),
    )
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(
            f"feature_closure status={result['status']} "
            f"heavy={','.join(result['heavy_features']) or '<none>'}"
        )
    return 0 if result["status"] == "core-only" else 2


def _split_features(value: str) -> tuple[str, ...]:
    return tuple(part.strip() for part in value.replace(",", " ").split() if part.strip())


def _clean_name(value: object) -> str:
    return value.strip() if isinstance(value, str) else ""


if __name__ == "__main__":
    raise SystemExit(main())
