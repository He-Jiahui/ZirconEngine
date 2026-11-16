"""Public client for Jenkins-owned, explicitly scoped local commits."""

# Retired public workflow: stop before legacy imports or state access.
if __name__ == "__main__":
    import sys as _retired_sys
    from pathlib import Path as _RetiredPath
    _retired_sys.dont_write_bytecode = True
    _retired_sys.path.insert(0, str(_RetiredPath(__file__).absolute().parents[2]))
    from tools.jenkins.retirement import retired_cli as _retired_cli
    raise SystemExit(_retired_cli())

from pathlib import Path
import runpy

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).resolve().parents[1] /
                       "jenkins-management/checkout_commit.py"), run_name="__main__")
