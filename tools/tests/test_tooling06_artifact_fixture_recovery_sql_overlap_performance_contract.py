from pathlib import Path


SOURCE = Path("tools/session_coordinator/artifact_governance.py")


def test_artifact_fixture_recovery_filters_path_overlap_in_sql() -> None:
    text = SOURCE.read_text(encoding="utf-8")
    start = text.index("    def _complete_candidate(")
    end = text.index("    @staticmethod\n    def _insert_fixture_event", start)
    body = text[start:end]

    assert "SELECT lease_id, target_dir FROM artifact_fixture_leases" in body
    assert "target_key=?" in body
    assert "substr(?, 1, length(target_key) + 1)" in body
    assert "substr(target_key, 1, length(?) + 1)" in body
    assert "targets_overlap(key" not in body


if __name__ == "__main__":
    test_artifact_fixture_recovery_filters_path_overlap_in_sql()
