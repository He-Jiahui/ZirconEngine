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


def _review_python(source: str, path: str) -> None:
    import ast
    try:
        ast.parse(source, filename=path)
    except (SyntaxError, ValueError) as error:
        raise JenkinsError("python_syntax_error", "sealed Python input did not parse",
                           details={"path": path, "line": getattr(error, "lineno", None)}) from error


def _review_json(source: str, path: str) -> None:
    import json
    try:
        json.loads(source)
    except ValueError as error:
        raise JenkinsError("json_syntax_error", "sealed JSON input did not parse",
                           details={"path": path, "line": getattr(error, "lineno", None)}) from error


def _review_toml(source: str, path: str) -> None:
    import tomllib
    try:
        tomllib.loads(source)
    except (tomllib.TOMLDecodeError, ValueError) as error:
        raise JenkinsError("toml_syntax_error", "sealed TOML input did not parse",
                           details={"path": path}) from error


def _review_powershell(source: str, path: str) -> dict:
    """Parse PowerShell without running any of it.

    ``Parser::ParseFile`` builds the AST only; the script is never invoked.  The
    text is fed from the already-sealed bytes, so the reviewed content cannot
    differ from what was sealed.
    """
    script = (
        "$ErrorActionPreference='Stop';\n"
        "$tokens=$null;$errors=$null;\n"
        "$ast=[System.Management.Automation.Language.Parser]::ParseInput("
        "[Console]::In.ReadToEnd(),[ref]$tokens,[ref]$errors);\n"
        "if($errors.Count -gt 0){$errors|ForEach-Object{$_.ToString()};exit 1};exit 0\n"
    )
    executable = shutil.which("pwsh") or shutil.which("powershell")
    if not executable:
        raise JenkinsError("powershell_parser_missing",
                           "a PowerShell parser is unavailable for sealed script review")
    completed = subprocess.run([str(Path(executable).resolve()), "-NoProfile", "-NonInteractive",
                                "-Command", script],
                               input=source, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise JenkinsError("powershell_syntax_error", "sealed PowerShell input did not parse",
                           details={"path": path, "stderr": completed.stderr[-4096:]})
    return {"tool": "powershell-parser", "exitCode": 0,
            "stdout": completed.stdout[-65536:], "stderr": completed.stderr[-65536:],
            "command": ["powershell-parser", "ParseInput"]}


REVIEW_SUFFIXES = frozenset({".rs", ".py", ".json", ".toml", ".ps1"})


def review_receipt(path: str, source: str, *, edition: str = "2021",
                   source_digest: str | None = None) -> dict:
    """Check one sealed file's syntax and formatting for its own language.

    Rust keeps the existing rustfmt pair.  The other languages are checked with
    their own parsers, all read-only: parsing a file is never allowed to execute
    it, which is the whole point of reviewing sealed input before a build.
    """
    suffix = Path(path).suffix.lower()
    if suffix not in REVIEW_SUFFIXES:
        raise JenkinsError("review_language_unsupported",
                           "Review covers only its registered languages",
                           details={"path": path, "suffix": suffix})
    if suffix == ".rs":
        syntax = syntax_receipt(source, edition=edition, source_digest=source_digest)
        formatting = format_receipt(source, edition=edition, source_digest=source_digest)
        return {"status": "passed", "kind": "review", "path": path, "sourceDigest": syntax["sourceDigest"],
                "tool": "rustfmt", "exitCode": 0, "edition": edition,
                "commandDigest": digest([syntax["commandDigest"], formatting["commandDigest"]]),
                "checks": {"syntax": syntax["status"], "format": formatting["status"]}}
    if suffix == ".py":
        _review_python(source, path)
        tool, detail = "ast.parse", {"parsed": True}
    elif suffix == ".json":
        _review_json(source, path)
        tool, detail = "json.loads", {"parsed": True}
    elif suffix == ".toml":
        _review_toml(source, path)
        tool, detail = "tomllib.loads", {"parsed": True}
    else:
        parsed = _review_powershell(source, path)
        tool, detail = parsed["tool"], {"parsed": True}
    digest_value = source_digest or digest(source)
    return {"status": "passed", "kind": "review", "path": path, "sourceDigest": digest_value,
            "tool": tool, "exitCode": 0,
            "commandDigest": digest([tool, path, digest_value]), "checks": detail}
