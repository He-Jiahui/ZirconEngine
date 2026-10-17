"""Submit scoped branch commits and inspect the scheduled Jenkins merge queue."""
from pathlib import Path
import runpy

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).resolve().parents[1] / "jenkins-management/branch_integration.py"), run_name="__main__")
