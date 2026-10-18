from pathlib import Path


SOURCE = Path("tools/session_coordinator/cleanup.py")


def test_retained_owner_filters_tree_overlap_in_sql() -> None:
    source = SOURCE.read_text(encoding="utf-8")
    helper = source.split("def _overlapping_retained_owner", 1)[1].split(
        "def cleanup_job_now", 1
    )[0]

    assert "substr(target_key, 1, length(?) + 1) = ? || '\\\\'" in helper
    assert "substr(?, 1, length(target_key) + 1) = target_key || '\\\\'" in helper
    assert "excluded_placeholders" in helper
    assert "targets_overlap" not in helper
