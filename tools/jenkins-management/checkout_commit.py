"""Ordinary checkout commits without whole-tree hashes or per-chat grants.

The current user has authorized scoped local commits. This route deliberately
does not issue product acceptance, push changes, or replay old failed workflows.
"""
from __future__ import annotations

import argparse
import base64
from contextlib import contextmanager
import json
import os
from pathlib import Path, PurePosixPath
import re
import sqlite3
import stat
import subprocess
import sys
import time
import tomllib
import urllib.parse
import urllib.error
import urllib.request
import uuid
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
BEGIN = "// BEGIN ZIRCON SCOPED CHECKOUT COMMIT"
END = "// END ZIRCON SCOPED CHECKOUT COMMIT"


class CommitBusy(ValueError):
    code = "management_git_busy"


@contextmanager
def git_lock(path):
    """Use the same native byte lock as the other management Git routes."""
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+b") as handle:
        if path.stat().st_size == 0:
            handle.write(b"\0")
            handle.flush()
        handle.seek(0)
        try:
            if os.name == "nt":
                import msvcrt
                msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            raise CommitBusy("Another management operation owns Git; continue the same request") from error
        try:
            yield
        finally:
            handle.seek(0)
            if os.name == "nt":
                msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(handle.fileno(), fcntl.LOCK_UN)


def physical(path):
    path = Path(os.path.abspath(path))
    for parent in (path, *path.parents):
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("Commit paths must be physical paths")
    return path


def read_json(path):
    return json.loads(physical(path).read_text(encoding="utf-8-sig"))


def write_json(path, value):
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = physical(path.with_name(path.name + ".tmp"))
    with temporary.open("w", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, path)


def load_policy(repo):
    policy = read_json(repo / "tools/jenkins-management/local-commit-policy.json")
    if (policy.get("schemaVersion") != 1 or policy.get("enabled") is not True
            or physical(policy["repositoryRoot"]) != repo
            or policy.get("authorization", {}).get("source") != "user"
            or not policy["authorization"].get("instruction")
            or policy.get("pushAllowed") is not False
            or any(policy.get(field) is not False for field in
                   ("requireSessionGrant", "requireSourceDigest", "requireFormalAcceptance"))):
        raise ValueError("Scoped local commit policy is not enabled for this checkout")
    return policy


def selected_paths(repo, paths, policy, grants):
    if not isinstance(paths, list) or not paths or len(paths) > 256:
        raise ValueError("Select between 1 and 256 individual task files")
    result, seen = [], set()
    for name in paths:
        if (not isinstance(name, str) or not name or "\\" in name or ":" in name
                or name.startswith("/") or any(c in name for c in "\0\r\n*?[")
                or any(p in {"", ".", ".."} for p in name.split("/"))
                or any(p.rstrip(" .") != p for p in name.split("/"))
                or any(p.casefold() == ".git" for p in PurePosixPath(name).parts)
                or name.casefold().startswith(".jenkins/")
                or name.casefold() in seen):
            raise ValueError("Commit requires unique literal repository-relative file paths")
        target = physical(repo / name)
        if target.exists() and not target.is_file():
            raise ValueError("Select individual files, not directories")
        if not target.is_relative_to(repo):
            raise ValueError("Commit path escaped this checkout")
        matching = [(len(str(Path(root))), mode) for root, mode in grants.items()
                    if not root.startswith(":") and target.is_relative_to(Path(root))]
        allowed = max(matching, default=(0, "read"))[1] == "write"
        if not allowed and name not in policy.get("controlFiles", []):
            raise ValueError("Selected file has no project write grant: " + name)
        seen.add(name.casefold())
        result.append(name)
    return result


def git(repo, *args, check=True):
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    environment.update(GIT_LITERAL_PATHSPECS="1", GIT_TERMINAL_PROMPT="0",
                       GIT_OPTIONAL_LOCKS="0")
    result = subprocess.run(["git", "-c", "core.quotepath=false", *args], cwd=repo,
                            env=environment, capture_output=True, timeout=120)
    if check and result.returncode:
        raise ValueError(result.stderr.decode("utf-8", "replace")[-2500:].strip()
                         or "Git command failed")
    return result


def checkout_reader(repo):
    database = physical(repo / ".jenkins/state/coordination.sqlite3")
    if not database.exists():
        return None
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=5) as connection:
        for key, raw in connection.execute(
                "SELECT key,payload FROM records WHERE domain='management_checkout_read'"):
            value = json.loads(raw)
            if value.get("status") in {"active", "unknown"}:
                return key
    return None


def execute(repo, payload, *, policy=None, grants=None):
    """Called under the shared management Git lock by the Jenkins worker."""
    repo = physical(repo)
    policy = load_policy(repo) if policy is None else policy
    if grants is None:
        config = tomllib.loads((repo / ".codex/config.toml").read_text(encoding="utf-8-sig"))
        grants = config["permissions"]["zircon-strict"]["filesystem"]
    if payload.get("action") != "commit-checkout":
        raise ValueError("Unsupported commit action")
    for field in ("sessionId", "requestId", "operationId"):
        if not isinstance(payload.get(field), str) or not ID.fullmatch(payload[field]):
            raise ValueError("Invalid " + field)
    paths = selected_paths(repo, payload.get("paths"), policy, grants)
    message = payload.get("message")
    if not isinstance(message, str) or not message.strip() or len(message) > 4096 or "\0" in message:
        raise ValueError("A bounded commit message is required")
    if type(payload.get("observationOnly", False)) is not bool:
        raise ValueError("observationOnly must be boolean")
    request = {k: payload[k] for k in ("sessionId", "requestId", "operationId", "action", "paths", "message")}
    directory = physical(repo / ".jenkins/state/management/checkout-commits")
    record_path = physical(directory / (payload["operationId"] + ".json"))
    previous = read_json(record_path) if record_path.exists() else None
    if previous and previous.get("request") != request:
        raise ValueError("This operation ID already belongs to another commit request")
    if previous and previous["status"] in {"complete", "no-changes"}:
        return {**previous, "reusedReceipt": True}
    if payload.get("observationOnly") is True:
        return previous or {"status": "not-started", "request": request, "formalAcceptance": False}
    reader = checkout_reader(repo)
    if reader:
        return {"status": "waiting", "reasonCode": "checkout_source_busy", "blockingReader": reader,
                "request": request, "formalAcceptance": False}
    head = git(repo, "rev-parse", "HEAD").stdout.decode().strip()
    trailer = "Zircon-Commit-Operation: " + payload["operationId"]
    if previous:
        commits = git(repo, "log", "-100", "--format=%H%x00%P%x00%B%x00",
                      "--fixed-strings", "--grep=" + trailer).stdout.decode("utf-8", "replace").split("\0")
        for i in range(0, len(commits) - 2, 3):
            commit, parents, body = commits[i:i + 3]
            if trailer in body.splitlines() and parents == previous.get("headBefore"):
                result = {**previous, "status": "complete", "commit": commit.strip(), "recoveredReceipt": True}
                write_json(record_path, result)
                return result
        if head != previous.get("headBefore"):
            raise ValueError("Interrupted commit has a changed HEAD; retain its original request")
    tracked = set(git(repo, "ls-files", "-z", "--", *paths).stdout.decode("utf-8").split("\0"))
    for name in paths:
        if not (repo / name).exists() and name not in tracked:
            raise ValueError("Selected file does not exist in Git or the checkout: " + name)
        mode = git(repo, "ls-files", "--stage", "--", name).stdout
        if mode.startswith(b"160000 "):
            raise ValueError("Submodule commits require their separate integration workflow")
    dirty = git(repo, "status", "--porcelain=v1", "-z", "--untracked-files=all", "--", *paths).stdout
    result = {"schemaVersion": 1, "status": "committing" if dirty else "no-changes",
              "request": request, "headBefore": head, "paths": paths,
              "authorization": policy["authorization"], "sourceDigestRequired": False,
              "sessionGrantRequired": False, "formalAcceptance": False,
              "pushPerformed": False, "notificationPerformed": False, "startedAt": time.time()}
    write_json(record_path, result)
    if not dirty:
        return result
    # --only commits exactly these working files and retains other staged entries.
    # Add makes explicitly selected untracked files available to --only.
    git(repo, "add", "--", *paths)
    hooks = physical(repo / ".jenkins/state/management/empty-commit-hooks")
    hooks.mkdir(parents=True, exist_ok=True)
    if any(hooks.iterdir()):
        raise ValueError("The managed empty hooks directory must remain empty")
    git(repo, "-c", "core.hooksPath=" + str(hooks), "-c", "commit.gpgSign=false",
        "commit", "--only", "--no-gpg-sign", "-m", message.strip(), "-m", trailer, "--", *paths)
    commit = git(repo, "rev-parse", "HEAD").stdout.decode().strip()
    result.update(status="complete", commit=commit, finishedAt=time.time())
    write_json(record_path, result)
    return result


def transport(repo):
    # Keep the ordinary entry independent of uncommitted coordinator modules.
    credentials = read_json(repo / ".jenkins/state/credentials.json")
    token = credentials.get("apiToken") or credentials.get("password")
    if credentials.get("authenticationMode") != "api-token" or not token or not credentials.get("username"):
        raise ValueError("Configured local Jenkins API token is required")
    authorization = "Basic " + base64.b64encode((credentials["username"] + ":" + token).encode()).decode()

    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            return None

    class LocalTransport:
        def request(self, path, *, method="GET", data=None, content_type=None):
            if path.startswith(("http:", "https:", "//")):
                raise ValueError("Only the configured local Jenkins endpoint is allowed")
            headers = {"Authorization": authorization, "Accept": "application/json"}
            if content_type:
                headers["Content-Type"] = content_type
            request = urllib.request.Request("http://127.0.0.1:18080/" + path.lstrip("/"),
                data=data, method=method, headers=headers)
            try:
                with urllib.request.build_opener(NoRedirect()).open(request, timeout=30) as response:
                    return response.status, response.read(), dict(response.headers.items())
            except urllib.error.HTTPError as error:
                return error.code, error.read(), dict(error.headers.items())
    return LocalTransport()


def install(repo, client):
    """Extend the existing management job while retaining its other routes."""
    load_policy(repo)
    endpoint = "/job/zircon-management/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Existing Jenkins management job is unavailable")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if script is None or "zircon-windows-agent" not in script.text or "--capsule" not in script.text:
        raise ValueError("The existing management job does not have its expected core routes")
    addon = (repo / "tools/jenkins-management/checkout-commit.groovy").read_text(encoding="utf-8-sig")
    original_script = script.text
    old_script = original_script
    if BEGIN in old_script:
        start, end = old_script.index(BEGIN), old_script.index(END) + len(END)
        old_script = old_script[:start] + old_script[end:].lstrip("\r\n")
    script.text = addon.rstrip() + "\n\n" + old_script
    action = document.find(".//parameterDefinitions/*[name='ACTION']")
    if action is not None and action.tag.endswith("ChoiceParameterDefinition"):
        choices = action.find("choices/a")
        if choices is None:
            raise ValueError("Unrecognized management action choice list")
        if "commit-checkout" not in [x.text for x in choices]:
            ET.SubElement(choices, "string").text = "commit-checkout"
    if script.text == original_script:
        return {"status": "installed", "job": "zircon-management", "action": "commit-checkout", "unchanged": True}
    updated = ET.tostring(document, encoding="utf-8", xml_declaration=True)
    # Existing running Pipeline builds retain their CPS program and capsule.
    status, body, _ = client.request(endpoint, method="POST", data=updated, content_type="application/xml")
    if status not in (200, 201, 302):
        raise ValueError("Management action installation failed: HTTP " + str(status))
    check_status, after, _ = client.request(endpoint)
    if check_status != 200 or ET.fromstring(after).findtext("definition/script") != script.text:
        raise ValueError("Management action installation has no matching read-back")
    return {"status": "installed", "job": "zircon-management", "action": "commit-checkout"}


def install_tests(repo, client):
    endpoint = "/job/zircon-management-tests/config.xml"
    status, raw, _ = client.request(endpoint)
    if status != 200:
        raise ValueError("Existing management regression job is unavailable")
    document = ET.fromstring(raw)
    script = document.find("definition/script")
    if script is None or "zircon-windows-agent" not in script.text:
        raise ValueError("Existing management regression job differs")
    begin, end = "// BEGIN ZIRCON SCOPED COMMIT TESTS", "// END ZIRCON SCOPED COMMIT TESTS"
    original = script.text
    previous = original
    if begin in previous:
        a, b = previous.index(begin), previous.index(end) + len(end)
        previous = previous[:a] + previous[b:].lstrip("\r\n")
    addon = (repo / "tools/jenkins-management/checkout-commit-tests.groovy").read_text(encoding="utf-8-sig")
    script.text = addon.rstrip() + "\n\n" + previous
    if script.text != original:
        status, _, _ = client.request(endpoint, method="POST",
            data=ET.tostring(document, encoding="utf-8", xml_declaration=True), content_type="application/xml")
        if status not in (200, 201, 302):
            raise ValueError("Commit regression route installation failed")
        status, after, _ = client.request(endpoint)
        if status != 200 or ET.fromstring(after).findtext("definition/script") != script.text:
            raise ValueError("Commit regression route read-back differs")
    return {"job": "zircon-management-tests", "status": "installed"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "submit", "run"))
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--input-file", type=Path)
    parser.add_argument("--output-file", type=Path)
    parser.add_argument("--path", action="append", dest="paths")
    parser.add_argument("--message")
    parser.add_argument("--session-id", default="local-checkout")
    parser.add_argument("--request-id")
    parser.add_argument("--operation-id")
    parser.add_argument("--query-only", action="store_true")
    args = parser.parse_args(argv)
    repo = physical(args.repo_root)
    try:
        if repo != physical(ROOT):
            raise ValueError("The public entry is limited to the authorized checkout")
        if args.command == "run":
            if os.environ.get("JOB_NAME") != "zircon-management" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
                raise ValueError("Git mutations require the Jenkins management job and fixed agent")
            payload = read_json(args.input_file)
            for field, variable in (("sessionId", "ZR_COMMIT_SESSION"), ("requestId", "ZR_COMMIT_REQUEST"),
                                    ("operationId", "ZR_COMMIT_OPERATION")):
                if payload.get(field) != os.environ.get(variable):
                    raise ValueError("Commit request differs from authoritative Jenkins parameters")
            if payload.get("observationOnly", False) != (os.environ.get("ZR_COMMIT_QUERY", "false").lower() == "true"):
                raise ValueError("Commit query flag differs from Jenkins parameters")
            with git_lock(repo / ".jenkins/state/locks/management-git.lock"):
                result = execute(repo, payload)
        else:
            load_policy(repo)
            client = transport(repo)
            if args.command == "install":
                result = install(repo, client)
                result["regression"] = install_tests(repo, client)
            else:
                payload = read_json(args.input_file) if args.input_file else {
                    "action": "commit-checkout", "paths": args.paths, "message": args.message,
                    "sessionId": args.session_id, "requestId": args.request_id or "commit-" + uuid.uuid4().hex[:16],
                    "operationId": args.operation_id or "commit-" + uuid.uuid4().hex[:16]}
                payload["observationOnly"] = args.query_only
                for field in ("sessionId", "requestId", "operationId"):
                    if not isinstance(payload.get(field), str) or not ID.fullmatch(payload[field]):
                        raise ValueError("Invalid " + field)
                # Repair only this extension if a normal management reinstall removed it.
                status, job, _ = client.request("/job/zircon-management/config.xml")
                if status != 200:
                    raise ValueError("Jenkins management job is unavailable")
                if BEGIN not in ET.fromstring(job).findtext("definition/script", ""):
                    install(repo, client)
                selected = read_json(repo / ".jenkins/state/deployment/driver.json")
                fields = {"ACTION": "commit-checkout", "SESSION_ID": payload["sessionId"],
                          "REQUEST_ID": payload["requestId"], "OPERATION_ID": payload["operationId"],
                          "INPUT_JSON": json.dumps(payload, ensure_ascii=False),
                          "QUERY_ONLY": str(args.query_only).lower(),
                          "EXPECTED_DRIVER_DIGEST": selected["driverDigest"],
                          "RUNTIME_OPERATION_ID": selected["runtimeOperationId"]}
                status, body, headers = client.request("/job/zircon-management/buildWithParameters", method="POST",
                    data=urllib.parse.urlencode(fields).encode(), content_type="application/x-www-form-urlencoded")
                if status not in (201, 302):
                    raise ValueError("Jenkins commit submission failed: HTTP " + str(status))
                result = {"status": "queued", "queueUrl": headers.get("Location"), "request": payload}
        if args.output_file:
            write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 0 if result["status"] not in {"failed", "unknown", "waiting"} else 1
    except Exception as error:
        result = {"status": "waiting" if getattr(error, "code", None) == "management_git_busy" else "failed",
                  "reasonCode": getattr(error, "code", "scoped_commit_failed"),
                  "message": str(error), "formalAcceptance": False, "pushPerformed": False}
        if args.output_file:
            write_json(args.output_file, result)
        print(json.dumps(result, ensure_ascii=False))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
