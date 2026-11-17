"""tools.jenkins.gap — gap-issue tracking and claim workflow."""
def __getattr__(name):
    """Preserve library exports without loading workflow code for ``-m``."""
    from importlib import import_module
    if name == "GapState":
        module = import_module(".state", __name__)
    elif name in {"extract_issues", "scan_directory"}:
        module = import_module(".extractor", __name__)
    elif name == "main":
        module = import_module(".cli", __name__)
    else:
        raise AttributeError(name)
    value = getattr(module, name)
    globals()[name] = value
    return value

__all__ = ["GapState", "extract_issues", "scan_directory", "main"]
