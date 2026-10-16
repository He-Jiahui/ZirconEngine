"""Public client for Jenkins-owned, explicitly scoped local commits."""
from pathlib import Path
import runpy

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).resolve().parents[1] /
                       "jenkins-management/checkout_commit.py"), run_name="__main__")
