"""Gap-issue extractor: parse P0/P1/P2 entries from optimize/ ledger documents."""
from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Iterator

# Matches lines like:  P0 `5 Open`, P1 `43 Open/5 Partial`, P1 `7 Open/2 Partial/0 Closed`
# and standalone gate-id patterns like:  RT-VIS-P1-007
_GATE_RE = re.compile(
    r"\b([A-Z][A-Z0-9\-]{2,20}-(?:P0|P1|P2)-\d{3})\b"
)

# Matches  P0 `N Open` / P1 `N Open/M Partial` style summary lines in index docs
_SUMMARY_COUNT_RE = re.compile(
    r"(P[012])\s+[`\"]?(\d+)\s+Open(?:/(\d+)\s+Partial)?(?:/(\d+)\s+Closed)?[`\"]?",
    re.IGNORECASE,
)

# Matches explicit finding lines, e.g.  - **P1:** description text
_FINDING_RE = re.compile(
    r"^\s*[-*]\s+\*{0,2}(P[012])\*{0,2}[:\s]+(.{10,})", re.MULTILINE
)


def _file_sha256(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def _doc_slug(path: Path) -> str:
    """Turn a doc path into a stable, short identifier."""
    name = path.stem  # e.g. "223-runtime-render-graph-..."
    # keep only the numeric prefix and first two words for brevity
    m = re.match(r"(\d+)-([a-z]+)-([a-z]+)", name)
    if m:
        return f"{m.group(1)}-{m.group(2)}-{m.group(3)}"
    return re.sub(r"[^a-z0-9]+", "-", name.lower())[:40]


def _issue_id(doc_slug: str, priority: str, seq: int) -> str:
    return f"{doc_slug}:{priority}:{seq:03d}"


def extract_issues(doc_path: Path) -> list[dict]:
    """Return a list of issue dicts extracted from a single ledger document."""
    text = doc_path.read_text(encoding="utf-8", errors="replace")
    fingerprint = _file_sha256(doc_path)
    slug = _doc_slug(doc_path)
    rel = str(doc_path)

    issues: list[dict] = []
    seen_gate_ids: set[str] = set()

    # 1. Explicit gate-id mentions (most precise)
    for m in _GATE_RE.finditer(text):
        gate_id = m.group(1)
        if gate_id in seen_gate_ids:
            continue
        seen_gate_ids.add(gate_id)
        # Determine priority from gate_id
        priority_match = re.search(r"-(P[012])-", gate_id)
        priority = priority_match.group(1) if priority_match else "P1"
        # Extract surrounding context as summary (up to 200 chars)
        start = max(0, m.start() - 60)
        end = min(len(text), m.end() + 120)
        summary = text[start:end].replace("\n", " ").strip()[:200]
        issues.append({
            "doc_path": rel,
            "priority": priority,
            "gate_id": gate_id,
            "summary": summary,
            "source_fingerprint": fingerprint,
        })

    # 2. Explicit finding lines (- **P1:** description)
    for m in _FINDING_RE.finditer(text):
        priority = m.group(1).upper()
        summary = m.group(2).strip()[:200]
        # deduplicate by summary prefix
        key = summary[:40]
        if any(i["summary"][:40] == key for i in issues):
            continue
        issues.append({
            "doc_path": rel,
            "priority": priority,
            "gate_id": None,
            "summary": summary,
            "source_fingerprint": fingerprint,
        })

    # Assign stable seq numbers per priority
    seq_counters: dict[str, int] = {}
    result = []
    for item in issues:
        p = item["priority"]
        seq_counters[p] = seq_counters.get(p, 0) + 1
        result.append({
            "issue_id": _issue_id(slug, p, seq_counters[p]),
            **item,
        })

    return result


def scan_directory(
    root: Path,
    *,
    priority_filter: str | None = None,
) -> Iterator[dict]:
    """Yield issue dicts from all .md files under root."""
    for path in sorted(root.rglob("*.md")):
        for issue in extract_issues(path):
            if priority_filter and issue["priority"] != priority_filter:
                continue
            yield issue
