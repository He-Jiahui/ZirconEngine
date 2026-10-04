"""Read-only Rust syntax and format evidence for sealed inputs."""
from __future__ import annotations

import os
import subprocess
import tempfile
import shutil
from pathlib import Path

from ..contracts import JenkinsError, digest


def _run_rustfmt(source: str, *, edition: str, check: bool) -> dict:
    if edition not in {"2015", "2018", "2021", "2024"}:
        raise JenkinsError("edition_invalid", "unsupported Rust edition")
    temp_root = Path(os.environ.get("JENKINS_TMP", ".jenkins/tmp")).resolve()
    temp_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="jenkins-syntax-", dir=str(temp_root)) as temp:
        path = Path(temp) / "input.rs"
        path.write_text(source, encoding="utf-8")
        rustfmt = shutil.which("rustfmt")
        if not rustfmt:
            raise JenkinsError("rustfmt_missing", "absolute rustfmt executable is unavailable")
        args = [str(Path(rustfmt).resolve()), "--edition", edition]
        if check:
            args.append("--check")
        else:
            args.extend(["--emit", "stdout"])
        args.append(str(path))
        completed = subprocess.run(args, capture_output=True, text=True, check=False)
    return {"exitCode": completed.returncode, "stdout": completed.stdout[-65536:],
            "stderr": completed.stderr[-65536:], "command": args,
            "commandDigest": digest(args), "tool": "rustfmt"}


def syntax_receipt(source: str, *, edition: str = "2021", source_digest: str | None = None) -> dict:
    result = _run_rustfmt(source, edition=edition, check=False)
    if result["exitCode"] != 0:
        raise JenkinsError("rust_syntax_error", "rustfmt rejected sealed Rust input", details=result)
    return {"status": "passed", "kind": "syntax", "sourceDigest": source_digest or digest(source),
            "edition": edition, **result}


def format_receipt(source: str, *, edition: str = "2021", source_digest: str | None = None) -> dict:
    result = _run_rustfmt(source, edition=edition, check=True)
    if result["exitCode"] != 0:
        raise JenkinsError("rust_format_error", "sealed Rust input is not formatted", details=result)
    return {"status": "passed", "kind": "format", "sourceDigest": source_digest or digest(source),
            "edition": edition, **result}
