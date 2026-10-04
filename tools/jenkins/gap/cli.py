"""Gap workflow CLI.

Usage:
  python -m tools.jenkins.gap extract [--dir DIR] [--priority P0|P1|P2]
  python -m tools.jenkins.gap list   [--priority P0] [--status open] [--limit 50]
  python -m tools.jenkins.gap claim  <issue_id> --session <id>
  python -m tools.jenkins.gap release <issue_id> --session <id>
  python -m tools.jenkins.gap submit  <issue_id> --session <id> --request <id> [--build <id>]
  python -m tools.jenkins.gap accept  <issue_id> --receipt <ref>
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .extractor import scan_directory
from .state import GapState

_REPO_ROOT = Path(__file__).parents[3]
_DEFAULT_DIR = _REPO_ROOT / "docs" / "plans" / "optimize"


def _state() -> GapState:
    s = GapState()
    s.ensure_schema()
    return s


def cmd_extract(args: argparse.Namespace) -> int:
    root = Path(args.dir) if args.dir else _DEFAULT_DIR
    if not root.exists():
        print(f"error: directory not found: {root}", file=sys.stderr)
        return 1
    state = _state()
    inserted = skipped = 0
    for issue in scan_directory(root, priority_filter=args.priority or None):
        existing = state.get_issue(issue["issue_id"])
        if existing is not None:
            skipped += 1
            continue
        state.upsert_issue(
            issue["issue_id"],
            issue["doc_path"],
            issue["priority"],
            issue["summary"],
            gate_id=issue.get("gate_id"),
            source_fingerprint=issue.get("source_fingerprint"),
        )
        inserted += 1
    print(f"extract: inserted={inserted} skipped={skipped}")
    return 0


def cmd_list(args: argparse.Namespace) -> int:
    state = _state()
    issues = state.list_issues(
        priority=args.priority or None,
        status=args.status or None,
        limit=args.limit,
    )
    if not issues:
        print("(no results)")
        return 0
    fmt = "{:<45}  {:<4}  {:<12}  {}"
    print(fmt.format("issue_id", "pri", "status", "summary"))
    print("-" * 100)
    for i in issues:
        print(fmt.format(
            i["issue_id"][:45],
            i["priority"],
            i["status"],
            i["summary"][:60],
        ))
    return 0


def cmd_claim(args: argparse.Namespace) -> int:
    state = _state()
    try:
        claim_id = state.claim(args.issue_id, args.session)
        print(json.dumps({"status": "claimed", "claim_id": claim_id,
                          "issue_id": args.issue_id, "session_id": args.session}))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def cmd_release(args: argparse.Namespace) -> int:
    state = _state()
    try:
        state.release(args.issue_id, args.session)
        print(json.dumps({"status": "released", "issue_id": args.issue_id}))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def cmd_submit(args: argparse.Namespace) -> int:
    """Record that a Jenkins job has been submitted for this issue."""
    state = _state()
    try:
        state.record_submission(
            args.issue_id,
            args.session,
            args.request,
            getattr(args, "build", None),
        )
        print(json.dumps({
            "status": "submitted",
            "issue_id": args.issue_id,
            "jenkins_request_id": args.request,
            "jenkins_build_id": getattr(args, "build", None),
        }))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def cmd_accept(args: argparse.Namespace) -> int:
    state = _state()
    try:
        state.accept(args.issue_id, args.receipt)
        print(json.dumps({"status": "accepted", "issue_id": args.issue_id,
                          "receipt_ref": args.receipt}))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="python -m tools.jenkins.gap",
                                description="Gap-issue workflow CLI")
    sub = p.add_subparsers(dest="command", required=True)

    ex = sub.add_parser("extract", help="Scan docs and populate gap_issues")
    ex.add_argument("--dir", help="Root directory to scan (default: docs/plans/optimize)")
    ex.add_argument("--priority", choices=["P0", "P1", "P2"], help="Filter by priority")

    ls = sub.add_parser("list", help="List gap issues")
    ls.add_argument("--priority", choices=["P0", "P1", "P2"])
    ls.add_argument("--status", choices=["open", "claimed", "fixing", "submitted", "accepted"])
    ls.add_argument("--limit", type=int, default=50)

    cl = sub.add_parser("claim", help="Claim an open issue for a session")
    cl.add_argument("issue_id")
    cl.add_argument("--session", required=True)

    rl = sub.add_parser("release", help="Release a claimed issue")
    rl.add_argument("issue_id")
    rl.add_argument("--session", required=True)

    sb = sub.add_parser("submit", help="Record Jenkins submission for a claimed issue")
    sb.add_argument("issue_id")
    sb.add_argument("--session", required=True)
    sb.add_argument("--request", required=True, help="Jenkins request ID")
    sb.add_argument("--build", help="Jenkins build ID (optional)")

    ac = sub.add_parser("accept", help="Mark a submitted issue as accepted")
    ac.add_argument("issue_id")
    ac.add_argument("--receipt", required=True, help="Jenkins receipt reference")

    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    dispatch = {
        "extract": cmd_extract,
        "list": cmd_list,
        "claim": cmd_claim,
        "release": cmd_release,
        "submit": cmd_submit,
        "accept": cmd_accept,
    }
    return dispatch[args.command](args)


if __name__ == "__main__":
    sys.exit(main())

def _state() -> GapState:
    s = GapState()
    s.ensure_schema()
    return s


def cmd_extract(args: argparse.Namespace) -> int:
    root = Path(args.dir) if args.dir else _DEFAULT_DIR
    if not root.exists():
        print(f"error: directory not found: {root}", file=sys.stderr)
        return 1
    state = _state()
    inserted = skipped = 0
    for issue in scan_directory(root, priority_filter=args.priority or None):
        existing = state.get_issue(issue["issue_id"])
        if existing is not None:
            skipped += 1
            continue
        state.upsert_issue(
            issue["issue_id"],
            issue["doc_path"],
            issue["priority"],
            issue["summary"],
            gate_id=issue.get("gate_id"),
            source_fingerprint=issue.get("source_fingerprint"),
        )
        inserted += 1
    print(f"extract: inserted={inserted} skipped={skipped}")
    return 0


def cmd_list(args: argparse.Namespace) -> int:
    state = _state()
    issues = state.list_issues(
        priority=args.priority or None,
        status=args.status or None,
        limit=args.limit,
    )
    if not issues:
        print("(no results)")
        return 0
    fmt = "{:<45}  {:<4}  {:<12}  {}"
    print(fmt.format("issue_id", "pri", "status", "summary"))
    print("-" * 100)
    for i in issues:
        print(fmt.format(
            i["issue_id"][:45],
            i["priority"],
            i["status"],
            i["summary"][:60],
        ))
    return 0


def cmd_claim(args: argparse.Namespace) -> int:
    state = _state()
    try:
        claim_id = state.claim(args.issue_id, args.session)
        print(json.dumps({"status": "claimed", "claim_id": claim_id,
                          "issue_id": args.issue_id, "session_id": args.session}))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def cmd_release(args: argparse.Namespace) -> int:
    state = _state()
    try:
        state.release(args.issue_id, args.session)
        print(json.dumps({"status": "released", "issue_id": args.issue_id}))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def cmd_accept(args: argparse.Namespace) -> int:
    state = _state()
    try:
        state.accept(args.issue_id, args.receipt)
        print(json.dumps({"status": "accepted", "issue_id": args.issue_id,
                          "receipt_ref": args.receipt}))
        return 0
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="python -m tools.jenkins.gap",
                                description="Gap-issue workflow CLI")
    sub = p.add_subparsers(dest="command", required=True)

    ex = sub.add_parser("extract", help="Scan docs and populate gap_issues")
    ex.add_argument("--dir", help="Root directory to scan (default: docs/plans/optimize)")
    ex.add_argument("--priority", choices=["P0", "P1", "P2"], help="Filter by priority")

    ls = sub.add_parser("list", help="List gap issues")
    ls.add_argument("--priority", choices=["P0", "P1", "P2"])
    ls.add_argument("--status", choices=["open", "claimed", "fixing", "submitted", "accepted"])
    ls.add_argument("--limit", type=int, default=50)

    cl = sub.add_parser("claim", help="Claim an open issue for a session")
    cl.add_argument("issue_id")
    cl.add_argument("--session", required=True)

    rl = sub.add_parser("release", help="Release a claimed issue")
    rl.add_argument("issue_id")
    rl.add_argument("--session", required=True)

    ac = sub.add_parser("accept", help="Mark a submitted issue as accepted")
    ac.add_argument("issue_id")
    ac.add_argument("--receipt", required=True, help="Jenkins receipt reference")

    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    dispatch = {
        "extract": cmd_extract,
        "list": cmd_list,
        "claim": cmd_claim,
        "release": cmd_release,
        "accept": cmd_accept,
    }
    return dispatch[args.command](args)


if __name__ == "__main__":
    sys.exit(main())
