"""Read-only first-party engine path budget guard (128 total, 48 reserved)."""
from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path, PurePosixPath
import subprocess

if __package__:
    from .review_debt import baseline_sources
else:
    from review_debt import baseline_sources

TOTAL_LIMIT = 128
PREFIX_RESERVE = 48
RELATIVE_LIMIT = TOTAL_LIMIT - PREFIX_RESERVE
ENGINE_ROOTS = (
    "zircon_app", "zircon_editor", "zircon_runtime", "zircon_runtime_interface",
    "zircon_runtime_host", "zircon_hub", "zircon_reflect_derive", "zircon_plugins",
)
EXCLUDED_PARTS = frozenset({"vendor", "node_modules", ".git", ".cache", ".zircon-cache"})
TEMP_SUFFIXES = (".tmp", ".bak", ".swp")
ROOT = Path(__file__).resolve().parents[2]


def path_units(value: str) -> int:
    """Windows path lengths count UTF-16 code units, including separators."""
    return len(value.encode("utf-16-le")) // 2


def owned_path(value: str) -> bool:
    path = PurePosixPath(value.replace("\\", "/"))
    if any(a==".zircon" and b=="cache" for a,b in zip(path.parts,path.parts[1:])):
        return False
    if "target" in path.parts:
        first=path.parts.index("target")
        if not any(p in {"src","tests","assets","fixtures"} for p in path.parts[:first]):
            return False
    return (
        bool(path.parts) and path.parts[0] in ENGINE_ROOTS
        and not EXCLUDED_PARTS.intersection(path.parts)
        and not path.name.endswith(TEMP_SUFFIXES)
    )


def files(root: Path = ROOT) -> list[str]:
    roots = [name for name in ENGINE_ROOTS if (root / name).is_dir()]
    result = subprocess.run(
        ["rg", "--files", "--hidden", "--no-ignore", "-g", "!**/vendor/**",
         "-g", "!**/node_modules/**", "-g", "!**/.git/**", "-g", "!**/.cache/**",
         "-g", "!**/.zircon-cache/**", "-g", "!**/.zircon/cache/**", *roots], cwd=root,
        capture_output=True, check=True,
    )
    return sorted({
        line.decode("utf-8").replace("\\", "/")
        for line in result.stdout.splitlines()
        if owned_path(line.decode("utf-8").replace("\\", "/"))
    })


def audit(paths: list[str], checkout: str = "") -> dict:
    entries = set(paths)
    for value in paths:
        entries.update(parent.as_posix() for parent in PurePosixPath(value).parents
                       if parent.as_posix() != ".")
    violations = [
        {"path":value, "relative_units":path_units(value),
         "absolute_units":path_units(checkout.rstrip("/\\") + "/" + value) if checkout else None}
        for value in sorted(entries)
        if path_units(value) > RELATIVE_LIMIT or (
            checkout and path_units(checkout.rstrip("/\\") + "/" + value) > TOTAL_LIMIT
        )
    ]
    return {
        "total_limit":TOTAL_LIMIT, "prefix_reserve":PREFIX_RESERVE,
        "relative_limit":RELATIVE_LIMIT, "files":len(paths),
        "long_files":sum(path_units(p)>RELATIVE_LIMIT for p in paths),
        "max_relative_units":max(map(path_units, paths), default=0),
        "violations":violations, "passed":not violations,
        "excluded":"third-party vendor trees, ignored build/cache data and noncanonical temporary files",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--summary", action="store_true")
    args = parser.parse_args()
    report = audit(files(args.repo_root), str(args.repo_root))
    baseline, _ = baseline_sources(args.repo_root, [])
    violations = report["violations"]
    # Existing path debt does not excuse a new name, even at the same length.
    report["violations"] = [v for v in violations if v["path"] not in baseline]
    report["baseline_violation_count"] = len(violations) - len(report["violations"])
    report["passed"] = not report["violations"]
    if args.summary:
        report["violations_by_root"] = dict(Counter(v["path"].split("/")[0] for v in report["violations"]))
        report.pop("violations")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
