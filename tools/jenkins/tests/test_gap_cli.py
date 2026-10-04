"""Tests for tools.jenkins.gap.cli (M3)."""
from __future__ import annotations
import json
import tempfile
from pathlib import Path

import pytest

from tools.jenkins.gap.cli import main
from tools.jenkins.gap.state import GapState


@pytest.fixture
def db_path(tmp_path):
    return tmp_path / "test.sqlite3"


@pytest.fixture
def seeded_db(db_path):
    s = GapState(db_path)
    s.ensure_schema()
    s.upsert_issue("doc:P1:001", "docs/x.md", "P1", "gap summary one")
    s.upsert_issue("doc:P0:001", "docs/x.md", "P0", "critical gap")
    return db_path


def _run(argv, db_path, capsys):
    """Patch the default DB path and run main()."""
    import tools.jenkins.gap.state as state_mod
    orig = state_mod._DEFAULT_DB
    state_mod._DEFAULT_DB = db_path
    try:
        rc = main(argv)
    finally:
        state_mod._DEFAULT_DB = orig
    out, err = capsys.readouterr()
    return rc, out, err


def test_list_all(seeded_db, capsys):
    rc, out, _ = _run(["list"], seeded_db, capsys)
    assert rc == 0
    assert "doc:P1:001" in out
    assert "doc:P0:001" in out


def test_list_priority_filter(seeded_db, capsys):
    rc, out, _ = _run(["list", "--priority", "P0"], seeded_db, capsys)
    assert rc == 0
    assert "doc:P0:001" in out
    assert "doc:P1:001" not in out


def test_list_status_filter(seeded_db, capsys):
    rc, out, _ = _run(["list", "--status", "open"], seeded_db, capsys)
    assert rc == 0
    assert "doc:P1:001" in out


def test_claim_success(seeded_db, capsys):
    rc, out, _ = _run(["claim", "doc:P1:001", "--session", "sess-1"], seeded_db, capsys)
    assert rc == 0
    data = json.loads(out)
    assert data["status"] == "claimed"
    assert data["issue_id"] == "doc:P1:001"


def test_claim_conflict(seeded_db, capsys):
    _run(["claim", "doc:P1:001", "--session", "sess-1"], seeded_db, capsys)
    rc, _, err = _run(["claim", "doc:P1:001", "--session", "sess-2"], seeded_db, capsys)
    assert rc == 1
    assert "gap_already_claimed" in err or "claimed" in err.lower()


def test_release(seeded_db, capsys):
    _run(["claim", "doc:P1:001", "--session", "sess-1"], seeded_db, capsys)
    rc, out, _ = _run(["release", "doc:P1:001", "--session", "sess-1"], seeded_db, capsys)
    assert rc == 0
    data = json.loads(out)
    assert data["status"] == "released"


def test_accept_wrong_state(seeded_db, capsys):
    rc, _, err = _run(["accept", "doc:P1:001", "--receipt", "ref-001"], seeded_db, capsys)
    assert rc == 1  # not in submitted state


def test_extract_scan(tmp_path, capsys):
    import textwrap
    doc = tmp_path / "001-review.md"
    doc.write_text(textwrap.dedent("""
        - **P1:** first finding with enough text to pass the minimum length
        - **P0:** critical finding with enough text to pass the minimum length
    """), encoding="utf-8")
    db = tmp_path / "coord.sqlite3"
    import tools.jenkins.gap.state as state_mod
    orig = state_mod._DEFAULT_DB
    state_mod._DEFAULT_DB = db
    try:
        rc, out, _ = _run(["extract", "--dir", str(tmp_path)], db, capsys)
    finally:
        state_mod._DEFAULT_DB = orig
    assert rc == 0
    assert "inserted=" in out
