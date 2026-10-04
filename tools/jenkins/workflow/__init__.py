"""Jenkins workflow planning and trusted change classification."""

from .classification import classify_change, classify_rust_source, classify_path
from .syntax import syntax_receipt, format_receipt
from .orchestrator import (
    FLOW_JOB,
    EXECUTION_JOB,
    MAINTENANCE_JOB,
    build_dag,
    run_control,
)
from .planning import RecipePlan, RecipePlanner, get_recipe_plan


def handle(*args, **kwargs):
    # Keep workflow package importable by process/lifecycle modules without
    # eagerly importing the CLI handler and its execution dependencies.
    from .handler import handle as _handle
    return _handle(*args, **kwargs)

__all__ = [
    "classify_change", "classify_rust_source", "classify_path",
    "FLOW_JOB", "EXECUTION_JOB", "MAINTENANCE_JOB", "build_dag", "run_control",
    "handle", "syntax_receipt", "format_receipt", "RecipePlan", "RecipePlanner", "get_recipe_plan",
]
