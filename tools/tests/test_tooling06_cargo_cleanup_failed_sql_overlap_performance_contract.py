from pathlib import Path


SOURCE = Path("tools/session_coordinator/cargo_jobs.py")


def test_cargo_cleanup_failed_blocker_filters_target_overlap_in_sql() -> None:
    text = SOURCE.read_text(encoding="utf-8")
    start = text.index("            cleanup_failed =")
    end = text.index("            if cleanup_failed is not None:", start)
    body = text[start:end]

    assert "targets_overlap" not in body
    assert "target_key=?" in body or "target_key = ?" in body
    assert "substr(target_key, 1, length(?) + 1)" in body
    assert "substr(?, 1, length(target_key) + 1)" in body
    assert "ORDER BY created_at DESC, job_id DESC" in body
    assert "LIMIT 1" in body


if __name__ == "__main__":
    test_cargo_cleanup_failed_blocker_filters_target_overlap_in_sql()
