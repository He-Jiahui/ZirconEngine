"""Public service entry. Windows owns this process; Codex is only a client."""

# Retired public workflow: stop before legacy imports or state access.
if __name__ == "__main__":
    import sys as _retired_sys
    from pathlib import Path as _RetiredPath
    _retired_sys.dont_write_bytecode = True
    _retired_sys.path.insert(0, str(_RetiredPath(__file__).absolute().parents[2]))
    from tools.jenkins.retirement import retired_cli as _retired_cli
    raise SystemExit(_retired_cli())

from pathlib import Path
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).absolute().parents[2]))

from tools.jenkins.deployment.independent_service import main

if __name__ == "__main__":
    raise SystemExit(main())
