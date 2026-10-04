"""Absolute-path bootstrap for python/pythonw; independent of working directory."""
from pathlib import Path
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.jenkins.tray.__main__ import main

if __name__ == "__main__":
    raise SystemExit(main())
