from pathlib import Path


SOURCE = Path("tools/session_coordinator/control_plane/snapshot.py")


def test_control_cargo_target_rank_does_not_materialize_wide_job_rows() -> None:
    text = SOURCE.read_text(encoding="utf-8")
    start = text.index("WITH latest_target AS (")
    end = text.index("        ).fetchall()", start)
    query = text[start:end]

    assert "SELECT *," not in query
    for column in (
        "job_id",
        "session_id",
        "lane_kind",
        "status",
        "created_at",
        "started_at",
        "finished_at",
        "released_at",
        "cleanup_policy",
        "cleanup_status",
        "process_tree_observed_at",
        "process_tree_exited_at",
        "target_dir",
    ):
        assert column in query


if __name__ == "__main__":
    test_control_cargo_target_rank_does_not_materialize_wide_job_rows()
