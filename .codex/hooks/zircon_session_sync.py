from __future__ import annotations

import argparse
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))


def main() -> int:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument(
        "--event",
        required=True,
        choices=(
            "SessionStart",
            "UserPromptSubmit",
            "Stop",
            "SubagentStart",
            "SubagentStop",
        ),
    )
    arguments = parser.parse_args()
    try:
        from tools.session_coordinator.codex_sync.hook import run_hook

        return run_hook(
            arguments.event,
            sys.stdin.buffer,
            sys.stdout,
            repo_root=REPO_ROOT,
        )
    except Exception:
        if arguments.event == "Stop":
            sys.stdout.write('{"continue":true}\n')
            sys.stdout.flush()
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
