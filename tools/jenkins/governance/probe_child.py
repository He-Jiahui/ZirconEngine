"""Actual permission probes, including inherited restrictions in a child process."""
from __future__ import annotations

import base64
import json
import os
from pathlib import Path
import subprocess
import sys


def check(specification, *, child=False):
    root = Path(specification["allowedRoot"])
    prefix = "child-" if child else "parent-"
    results = []
    for label, directory in (("own-temporary", root), ("own-patch-handoff", Path(specification["handoffRoot"]))):
        positive = directory / (".governance-positive-" + specification["operationId"] + "-" + prefix)
        try:
            with positive.open("xb") as stream:
                stream.write(b"probe")
            positive.write_bytes(b"probe-updated")
            renamed = positive.with_suffix(".renamed")
            positive.rename(renamed)
            renamed.unlink()
            results.append({"scope": label, "passed": True})
        except OSError as error:
            results.append({"scope": label, "passed": False, "errno": error.errno, "winerror": getattr(error, "winerror", None)})
    targets = {"git": Path(specification["commonDir"]),
               "jenkins-state": Path(specification["controlRoot"]) / ".jenkins/state",
               "build-pool": Path(specification["buildNamespace"])}
    for item in specification["sourceTargets"]:
        path = Path(item["path"])
        if item["kind"] == "directory":
            targets[item["scope"]] = path
            continue
        # OPEN_EXISTING without truncation probes individually granted files
        # while leaving their bytes and shared-index ownership untouched.
        try:
            descriptor = os.open(path, os.O_WRONLY)
            os.close(descriptor)
            results.append({"scope": item["scope"], "passed": False, "reason": "file_write_access_granted"})
        except OSError as error:
            code = getattr(error, "winerror", None)
            denied = code == 5 or code is None and error.errno == 13
            results.append({"scope": item["scope"], "passed": denied, "errno": error.errno, "winerror": code})
    for label, target in targets.items():
        if not target.is_dir():
            results.append({"scope": label, "passed": False, "reason": "missing_probe_destination"})
            continue
        path = target / (".governance-canary-" + specification["operationId"] + "-" + prefix)
        if path.exists():
            results.append({"scope": label, "passed": False, "reason": "canary_already_exists"})
            continue
        try:
            with path.open("xb") as stream:
                stream.write(b"permission-probe-canary")
            results.append({"scope": label, "passed": False, "reason": "write_succeeded", "residual": str(path)})
        except OSError as error:
            code = getattr(error, "winerror", None)
            denied = code == 5 or code is None and error.errno == 13
            results.append({"scope": label, "passed": denied, "errno": error.errno, "winerror": getattr(error, "winerror", None)})
    return results


if __name__ == "__main__":
    specification = json.loads(base64.b64decode(sys.argv[1], validate=True))
    inherited = len(sys.argv) > 2 and sys.argv[2] == "child"
    values = check(specification, child=inherited)
    if inherited:
        print(json.dumps(values))
        raise SystemExit(0 if all(row["passed"] for row in values) else 1)
    child = subprocess.run([sys.executable, "-B", "-X", "utf8", __file__, sys.argv[1], "child"], capture_output=True, timeout=60)
    try:
        children = json.loads(child.stdout)
    except (ValueError, UnicodeError):
        children = [{"scope": "child", "passed": False, "reason": "child_did_not_execute_probes"}]
    value = {"parent": values, "child": children, "passed": child.returncode == 0 and all(row["passed"] for row in values + children)}
    (Path(specification["allowedRoot"]) / "probe.json").write_text(json.dumps(value), encoding="utf-8")
    raise SystemExit(0 if value["passed"] else 1)
