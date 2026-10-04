"""Tests for tools.jenkins.gap.extractor (M2)."""
from __future__ import annotations
import textwrap
from pathlib import Path

import pytest

from tools.jenkins.gap.extractor import extract_issues, _doc_slug, _issue_id


def write_doc(tmp_path: Path, name: str, content: str) -> Path:
    p = tmp_path / name
    p.write_text(textwrap.dedent(content), encoding="utf-8")
    return p


def test_extract_gate_ids(tmp_path):
    doc = write_doc(tmp_path, "223-runtime-render-graph.md", """
        # Runtime223

        P0 缺口：RT-VIS-P0-001 journal 未接产品，RT-VIS-P0-002 CPU bounds 单位球。
        P1 `49 Open/11 Partial`：RT-VIS-P1-007 grid COW 未实现。
    """)
    issues = extract_issues(doc)
    gate_ids = [i["gate_id"] for i in issues if i["gate_id"]]
    assert "RT-VIS-P0-001" in gate_ids
    assert "RT-VIS-P0-002" in gate_ids
    assert "RT-VIS-P1-007" in gate_ids


def test_extract_finding_lines(tmp_path):
    doc = write_doc(tmp_path, "099-editor-review.md", """
        ## Findings

        - **P1:** command document lacks typed executor contract
        - **P0:** session teardown has no quiescent barrier
        - **P2:** icon atlas rebuild triggered on every frame
    """)
    issues = extract_issues(doc)
    priorities = [i["priority"] for i in issues]
    assert "P0" in priorities
    assert "P1" in priorities
    assert "P2" in priorities


def test_issue_ids_are_stable(tmp_path):
    doc = write_doc(tmp_path, "010-runtime-core.md", """
        - **P1:** first gap
        - **P1:** second gap
    """)
    issues = extract_issues(doc)
    p1 = [i for i in issues if i["priority"] == "P1"]
    assert len(p1) == 2
    ids = [i["issue_id"] for i in p1]
    # seq numbers differ
    assert ids[0] != ids[1]
    assert ids[0].endswith(":001")
    assert ids[1].endswith(":002")


def test_summary_truncated(tmp_path):
    doc = write_doc(tmp_path, "001-x.md", f"""
        - **P1:** {"x" * 300}
    """)
    issues = extract_issues(doc)
    assert all(len(i["summary"]) <= 200 for i in issues)


def test_empty_doc_yields_no_issues(tmp_path):
    doc = write_doc(tmp_path, "empty.md", "# Title\n\nNo gaps here.\n")
    assert extract_issues(doc) == []


def test_doc_slug():
    from pathlib import Path
    p = Path("docs/plans/optimize/zircon_runtime/223-runtime-render-graph-gpu-scene.md")
    slug = _doc_slug(p)
    assert slug.startswith("223-runtime-render")
    assert len(slug) <= 40
