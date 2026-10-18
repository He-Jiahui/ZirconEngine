"""Public service entry. Windows owns this process; Codex is only a client."""
from pathlib import Path
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).absolute().parents[2]))

from tools.jenkins.deployment.independent_service import main

if __name__ == "__main__":
    raise SystemExit(main())
