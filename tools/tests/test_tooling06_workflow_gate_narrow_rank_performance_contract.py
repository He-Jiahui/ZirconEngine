from pathlib import Path


SOURCE = Path("tools/session_coordinator/workflows/projections.py")


def test_commit_eligibility_rank_omits_unused_gate_payloads() -> None:
    source = SOURCE.read_text(encoding="utf-8")
    helper = source.split("def _commit_eligibilities", 1)[1].split(
        "def _attempt_projection", 1
    )[0]

    assert "SELECT node_id, gate_kind, decision, input_fingerprint FROM" in helper
    assert "SELECT source.node_id, source.gate_kind, source.decision," in helper
    assert "source.*" not in helper
    assert "evidence.*" not in helper
