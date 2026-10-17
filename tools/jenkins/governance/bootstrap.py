"""Hash-pinned Jenkins entry. No State or output is touched before verification."""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import sys
import zlib


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pins-base64")
    parser.add_argument("--request-base64")
    parser.add_argument("--from-environment", action="store_true")
    args = parser.parse_args()
    pins_b64 = os.environ["ZGOV_PINS"] if args.from_environment else args.pins_base64
    request_b64 = os.environ["ZGOV_REQUEST"] if args.from_environment else args.request_base64
    pins = json.loads(zlib.decompress(base64.b64decode(pins_b64, validate=True)))
    root = Path(pins["controlRoot"])
    # Verify using only the standard library before importing repository code.
    # In particular, a changed paths.py must not choose its own verification result.
    def verified_file(path):
        for candidate in (path, *path.parents):
            info = candidate.lstat()
            if candidate.is_symlink() or getattr(info, "st_file_attributes", 0) & 0x400:
                raise ValueError("bootstrap_reparse_path_rejected")
        if os.path.normcase(str(path.resolve())) != os.path.normcase(str(path)):
            raise ValueError("bootstrap_alias_rejected")
        before = path.stat()
        with path.open("rb") as stream:
            opened = os.fstat(stream.fileno())
            identity = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns)
            if identity(opened) != identity(before):
                raise ValueError("bootstrap_file_replaced")
            checksum = hashlib.file_digest(stream, "sha256").hexdigest()
            if identity(os.fstat(stream.fileno())) != identity(before) or identity(path.stat()) != identity(before):
                raise ValueError("bootstrap_file_changed")
        return checksum
    if not root.is_absolute() or str(Path(__file__)) != str(root / "tools/jenkins/governance/bootstrap.py"):
        raise ValueError("unexpected_governance_bootstrap")
    for relative, expected in pins["sourcePins"].items():
        path = Path(relative)
        if path.is_absolute() or ".." in path.parts:
            raise ValueError("invalid_source_pin")
        if verified_file(root / path) != expected:
            raise ValueError("governance_source_changed:" + relative)
    if verified_file(Path(sys.executable)) != pins["pythonSha256"]:
        raise ValueError("governance_python_changed")
    if os.environ.get("JOB_NAME") != pins["jobName"] or not os.environ.get("BUILD_NUMBER"):
        raise ValueError("jenkins_governance_job_required")
    request = json.loads(base64.b64decode(request_b64, validate=True))
    sys.path.insert(0, str(root))
    from tools.jenkins.governance.paths import digest
    from tools.jenkins.governance.worker import execute
    return execute(pins, request, capsule_digest=digest(pins))


if __name__ == "__main__":
    raise SystemExit(main())
