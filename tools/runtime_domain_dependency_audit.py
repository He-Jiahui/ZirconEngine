from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path


RUNTIME_DOMAINS = frozenset(
    {
        "animation",
        "asset",
        "builtin",
        "core",
        "diagnostic_log",
        "dynamic_api",
        "engine_module",
        "foundation",
        "graphics",
        "input",
        "navigation",
        "platform",
        "plugin",
        "render_graph",
        "rhi",
        "rhi_wgpu",
        "scene",
        "script",
        "ui",
    }
)

CRATE_DOMAIN_REFERENCE = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*)::")


@dataclass(frozen=True, order=True)
class DomainReference:
    source_domain: str
    target_domain: str
    path: str
    line: int
    source: str


def is_production_rust_source(relative_path: Path) -> bool:
    if relative_path.suffix != ".rs":
        return False
    parts = relative_path.parts
    file_name = relative_path.name
    if (
        "tests" in parts
        or file_name in {"tests.rs", "test.rs"}
        or file_name.endswith("_tests.rs")
        or file_name.startswith("test_")
    ):
        return False
    return parts and parts[0] in RUNTIME_DOMAINS


def audit_runtime_domain_dependencies(repo_root: Path) -> dict[str, object]:
    source_root = repo_root / "zircon_runtime" / "src"
    references: set[DomainReference] = set()

    for source_path in sorted(source_root.rglob("*.rs")):
        relative_path = source_path.relative_to(source_root)
        if not is_production_rust_source(relative_path):
            continue

        source_domain = relative_path.parts[0]
        for line_number, line in enumerate(
            source_path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            targets = {
                match.group(1)
                for match in CRATE_DOMAIN_REFERENCE.finditer(line)
                if match.group(1) in RUNTIME_DOMAINS
                and match.group(1) != source_domain
            }
            for target_domain in targets:
                references.add(
                    DomainReference(
                        source_domain=source_domain,
                        target_domain=target_domain,
                        path=(Path("zircon_runtime/src") / relative_path).as_posix(),
                        line=line_number,
                        source=line.strip(),
                    )
                )

    ordered_references = sorted(references)
    matrix = Counter(
        (reference.source_domain, reference.target_domain)
        for reference in ordered_references
    )
    matrix_rows = [
        {
            "source_domain": source_domain,
            "target_domain": target_domain,
            "reference_count": count,
        }
        for (source_domain, target_domain), count in sorted(matrix.items())
    ]

    return {
        "schema_version": 1,
        "runtime_source_root": "zircon_runtime/src",
        "production_reference_count": len(ordered_references),
        "domain_edge_count": len(matrix_rows),
        "matrix": matrix_rows,
        "references": [asdict(reference) for reference in ordered_references],
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit direct cross-domain references in zircon_runtime production Rust."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root override.",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print JSON for review or baseline capture.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Write the JSON report to this path instead of stdout.",
    )
    args = parser.parse_args()

    report = audit_runtime_domain_dependencies(args.repo_root.resolve())
    rendered = json.dumps(
        report,
        ensure_ascii=False,
        indent=2 if args.pretty else None,
        sort_keys=True,
    )
    if args.output is None:
        print(rendered)
    else:
        output_path = args.output.resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(f"{rendered}\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
