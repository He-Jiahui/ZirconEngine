from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CANDIDATES = ROOT / "tools" / "session_coordinator" / "integration_candidates.py"


def test_candidate_lease_evidence_uses_prefix_trie() -> None:
    source = CANDIDATES.read_text(encoding="utf-8")
    lease_evidence = source.split("    def _lease_evidence", 1)[1].split(
        "    def _tree_with_candidate_blobs", 1
    )[0]

    assert "lease_trie" in lease_evidence
    assert "node.setdefault(part" in lease_evidence
    assert '"/".join(path_parts[:part_count])' not in lease_evidence
