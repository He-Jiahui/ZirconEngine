"""Entry point: python -m tools.jenkins.gap <command>"""

# Retired public workflow: stop before legacy imports or state access.
if __name__ == "__main__":
    import sys as _retired_sys
    from pathlib import Path as _RetiredPath
    _retired_sys.dont_write_bytecode = True
    _retired_sys.path.insert(0, str(_RetiredPath(__file__).absolute().parents[3]))
    from tools.jenkins.retirement import retired_cli as _retired_cli
    raise SystemExit(_retired_cli())

import sys
from .cli import main

sys.exit(main())
