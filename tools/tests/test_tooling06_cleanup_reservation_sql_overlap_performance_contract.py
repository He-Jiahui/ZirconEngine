from pathlib import Path


SOURCE = Path("tools/session_coordinator/cargo_jobs.py")


def test_cleanup_reservation_filters_tree_overlap_in_sql() -> None:
    source = SOURCE.read_text(encoding="utf-8")
    helper = source.split("def overlapping_cleanup_reservation", 1)[1].split(
        "@dataclass", 1
    )[0]

    assert "substr(target_key, 1, length(?) + 1) = ? || '\\\\'" in helper
    assert "substr(?, 1, length(target_key) + 1) = target_key || '\\\\'" in helper
    assert "ORDER BY reserved_at LIMIT 1" in helper
    assert "targets_overlap" not in helper
