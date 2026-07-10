from __future__ import annotations

import re
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path


LEGACY_STANDALONE_CRATES = (
    "zircon_module",
    "zircon_scene",
    "zircon_manager",
    "zircon_framework",
    "zircon_asset",
    "zircon_graphics",
    "zircon_ui",
    "zircon_core",
    "zircon_server",
    "zircon_render_server",
)


@dataclass
class Location:
    path: str
    line: int
    snippet: str


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _architecture_doc_files(root: Path) -> list[Path]:
    docs_root = root / "docs" / "engine-architecture"
    if not docs_root.exists():
        return []
    return sorted(
        path
        for path in docs_root.glob("*.md")
        if not path.name.startswith("runtime-architecture-review-")
    )


def legacy_standalone_references(
    root: Path,
    max_locations_per_term: int = 5,
) -> dict[str, object]:
    counts: dict[str, int] = defaultdict(int)
    samples: dict[str, list[Location]] = defaultdict(list)
    patterns = {
        crate_name: re.compile(rf"\b{re.escape(crate_name)}\b")
        for crate_name in LEGACY_STANDALONE_CRATES
    }

    for path in _architecture_doc_files(root):
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            for crate_name, pattern in patterns.items():
                if not pattern.search(line):
                    continue
                counts[crate_name] += 1
                if len(samples[crate_name]) < max_locations_per_term:
                    samples[crate_name].append(
                        Location(
                            path=_relative(root, path),
                            line=line_no,
                            snippet=line.strip(),
                        )
                    )

    return {
        "counts": dict(sorted((key, value) for key, value in counts.items() if value)),
        "sample_locations": {
            key: [asdict(location) for location in locations]
            for key, locations in sorted(samples.items())
            if locations
        },
    }
