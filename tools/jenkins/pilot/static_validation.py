"""The small declared Python input check; emits no bytecode or build products."""
from __future__ import annotations

import ast
import json
import sys
from pathlib import Path


def validate(source: Path) -> dict[str, int]:
    checked = 0
    for path in sorted(source.rglob("*.py")):
        ast.parse(path.read_bytes(), filename=path.relative_to(source).as_posix())
        checked += 1
    if not checked:
        raise ValueError("python-static requires Python files in the declared input closure")
    return {"pythonFiles": checked}


def main() -> int:
    if len(sys.argv) != 2:
        raise ValueError("python-static requires one materialized input directory")
    print(json.dumps(validate(Path(sys.argv[1])), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
