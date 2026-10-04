"""Local command entry for the reversible, isolated Jenkins pilot."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
from .contracts import RequestIdentity, canonical_json
from .storage import DEFAULT_PILOT_ROOT, ManagedStorage


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_PILOT_ROOT)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    sub = parser.add_subparsers(dest="action", required=True)
    allocate = sub.add_parser("allocate")
    allocate.add_argument("--state-file", type=Path, required=True)
    allocate.add_argument("--prefix", default="jenkins-pilot-independent")
    prepare = sub.add_parser("prepare")
    prepare.add_argument("--reuse-from", type=Path, required=True)
    for name in ("start", "stop", "install"):
        sub.add_parser(name)
    seal = sub.add_parser("seal")
    seal.add_argument("--path", action="append", required=True)
    seal.add_argument("--allow-new", action="append", default=[])
    for name in ("submit", "reconcile", "cancel", "verify"):
        command = sub.add_parser(name)
        command.add_argument("--request-file", type=Path, required=True)
        command.add_argument("--job", default="zircon-pilot")
        if name == "submit":
            command.add_argument("--bundle", type=Path, required=True)
    args = parser.parse_args()
    if args.action == "allocate":
        from .governance import allocate_storage
        result = allocate_storage(args.repo_root, args.state_file, args.prefix)
    elif args.action == "prepare":
        from .assets import prepare_preserved_assets
        result = prepare_preserved_assets(args.root, args.reuse_from, repo_root=args.repo_root)
    elif args.action == "start":
        from .bootstrap import start_controller, start_agent
        from .deployment import seal_driver, install_job
        controller = start_controller(args.root)
        driver = seal_driver(args.root, args.repo_root)
        job = install_job(args.root)
        agent = start_agent(args.root)
        result = {"controller": controller, "driver": driver, "job": job, "agent": agent}
    elif args.action == "stop":
        from .bootstrap import stop
        result = stop(args.root)
    elif args.action == "install":
        from .deployment import install_job
        result = install_job(args.root)
    elif args.action == "seal":
        from .snapshot import capture
        sealed = capture(args.repo_root, ManagedStorage(args.root), paths=args.path, untracked_allowlist=args.allow_new)
        result = {"bundle": str(sealed.bundle), "bundleHash": sealed.bundle_hash, "inputHash": sealed.input_hash,
                  "baseCommit": sealed.manifest["baseCommit"], "fileCount": len(sealed.manifest["entries"])}
    else:
        from .deployment import client
        from .journal import SubmissionJournal
        identity = RequestIdentity.from_dict(json.loads(args.request_file.read_bytes()))
        journal = SubmissionJournal(args.root / "submissions.sqlite3")
        api = client(args.root)
        if args.action == "verify":
            from .acceptance import verify_build
            result = verify_build(root=args.root, repo_root=args.repo_root, identity=identity,
                                  job=args.job, journal=journal)
        elif args.action == "submit":
            result = api.submit(args.job, identity, args.bundle, journal)
        else:
            result = getattr(api, args.action)(args.job, identity, journal)
    print(canonical_json(result).decode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
