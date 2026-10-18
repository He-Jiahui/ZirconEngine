from pathlib import Path


SOURCE = Path("tools/session_coordinator/cargo_reservations.py")


def test_invalid_pending_reconciliation_filters_candidates_in_sql() -> None:
    source = SOURCE.read_text(encoding="utf-8")

    assert "owners.status IS NULL" in source
    assert "owners.status NOT IN ({executable_placeholders})" in source
    assert "reservations.expires_at <= ?" in source
    assert "*EXECUTABLE_CARGO_SESSION_STATUSES," in source
