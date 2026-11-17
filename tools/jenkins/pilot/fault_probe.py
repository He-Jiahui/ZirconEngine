"""An owned parent/child pair used only for lifecycle fault acceptance."""
from __future__ import annotations

# Retired public workflow: stop before legacy imports or state access.
if __name__ == "__main__":
    import sys as _retired_sys
    from pathlib import Path as _RetiredPath
    _retired_sys.dont_write_bytecode = True
    _retired_sys.path.insert(0, str(_RetiredPath(__file__).absolute().parents[3]))
    from tools.jenkins.retirement import retired_cli as _retired_cli
    raise SystemExit(_retired_cli())


import json
import os
import subprocess
import sys
import time
from pathlib import Path

from tools.jenkins.pilot.native.process_identity import popen_process_creation_time, process_creation_time
from .storage import require_managed_root


def main() -> int:
    path = require_managed_root(Path(sys.argv[1]), allow_root=False)
    seconds = float(sys.argv[2])
    if not 1 <= seconds <= 120:
        raise ValueError("probe duration must be between one and 120 seconds")
    child = subprocess.Popen(
        [sys.executable, "-B", "-c", "import time; time.sleep(180)"],
        stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
    )
    path.write_text(json.dumps({
        "parentPid": os.getpid(), "parentCreationTime": process_creation_time(os.getpid()),
        "childPid": child.pid, "childCreationTime": popen_process_creation_time(child),
    }), encoding="utf-8")
    print("owned probe parent and child started", flush=True)
    try:
        time.sleep(seconds)
    finally:
        child.terminate()
        child.wait(timeout=10)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
