"""Tests for tools.jenkins.gap.state (M1)."""
from __future__ import annotations
import sqlite3
import tempfile
from pathlib import Path

import pytest

from tools.jenkins.gap.state import GapState
from tools.jenkins.contracts import JenkinsError


@pytest.fixture
def state(tmp_path):
    db = tmp_path / "test.sqlite3"
    s = GapState(db)
    s.ensure_schema()
    return s


def test_upsert_and_get(state):
    state.upsert_issue("doc:P1:001", "docs/plans/optimize/x.md", "P1",
                       "some gap summary", gate_id="RT-P1-001")
    issue = state.get_issue("doc:P1:001")
    assert issue["priority"] == "P1"
    assert issue["status"] == "open"
    assert issue["gate_id"] == "RT-P1-001"


def test_upsert_is_idempotent(state):
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "summary a")
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "summary b")  # ignored
    issue = state.get_issue("doc:P1:001")
    assert issue["summary"] == "summary a"  # first write wins


def test_list_issues_filter(state):
    state.upsert_issue("doc:P0:001", "docs/x.md", "P0", "p0 gap")
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "p1 gap")
    p0 = state.list_issues(priority="P0")
    assert len(p0) == 1 and p0[0]["priority"] == "P0"


def test_claim_and_release(state):
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "gap")
    claim_id = state.claim("doc:P1:001", "session-A")
    assert claim_id
    issue = state.get_issue("doc:P1:001")
    assert issue["status"] == "claimed"

    # second claim from different session is rejected
    with pytest.raises(JenkinsError) as exc_info:
        state.claim("doc:P1:001", "session-B")
    assert exc_info.value.code == "gap_already_claimed"

    state.release("doc:P1:001", "session-A")
    issue = state.get_issue("doc:P1:001")
    assert issue["status"] == "open"


def test_release_wrong_session_rejected(state):
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "gap")
    state.claim("doc:P1:001", "session-A")
    with pytest.raises(JenkinsError) as exc_info:
        state.release("doc:P1:001", "session-B")
    assert exc_info.value.code == "gap_not_owner"


def test_accept_lifecycle(state):
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "gap")
    state.claim("doc:P1:001", "session-A")
    state.record_submission("doc:P1:001", "session-A", "req-001")
    state.accept("doc:P1:001", "receipt-abc")
    issue = state.get_issue("doc:P1:001")
    assert issue["status"] == "accepted"


def test_accept_requires_submitted(state):
    state.upsert_issue("doc:P1:001", "docs/x.md", "P1", "gap")
    with pytest.raises(JenkinsError) as exc_info:
        state.accept("doc:P1:001", "receipt-abc")
    assert exc_info.value.code == "gap_wrong_state"


def test_not_found(state):
    assert state.get_issue("nonexistent") is None
    with pytest.raises(JenkinsError) as exc_info:
        state.claim("nonexistent", "session-A")
    assert exc_info.value.code == "gap_not_found"
