"""Ordinary Jenkins management client. Submission is never build acceptance."""
from __future__ import annotations

import argparse
import contextlib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sqlite3
import sys
import urllib.parse

CLIENT_SOURCES = frozenset({
    "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1",
    "tools/dev/dev-fast-build.ps1", "tools/dev/submit-jenkins-management.ps1",
    "tools/jenkins/frontend.py", "tools/jenkins-management/entry.py",
})
CAPABILITY = {"enabled": True, "actions": ["validate"],
              "formalAcceptance": False, "m8Accepted": False}
ACTIONS = {"create-branch", "create-worktree", "prepare-merge", "apply-patch",
           "commit-worktree", "release-worktree", "validate", "generate", "submit-flow"}
AUTH = {name: "implementation" for name in ACTIONS}
AUTH.update({"validate": "validation", "submit-flow": "validation", "commit-worktree": "commit",
             "release-worktree": "gc"})
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
RECIPE_FIELDS = frozenset({
    "verb", "packages", "features", "profile", "offline", "noDefaultFeatures",
    "allFeatures", "locked", "jobs", "lib", "tests", "testFilter", "noRun", "testThreads",
    "buildRoot", "requestedTargetDir",
})


class EntryError(RuntimeError):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require(value, message, code="ordinary_entry_unproven"):
    if not value:
        raise EntryError(code, message)


def plain(value):
    path = Path(value).absolute()
    require(".." not in Path(value).parts, "Physical paths cannot contain parent traversal")
    for item in (path, *path.parents):
        if item.exists() or item.is_symlink():
            require(not item.is_symlink() and not getattr(item.lstat(), "st_file_attributes", 0) & 0x400,
                    "A control or source path uses an alias")
    return path


def document(path):
    path = plain(path)
    require(path.is_file() and path.stat().st_size <= 8 * 1024**2, "A bounded input file is required")
    raw = path.read_bytes()
    def pairs(items):
        result = {}
        for name, value in items:
            require(name not in result, "Duplicate JSON fields are not accepted")
            result[name] = value
        return result
    value = json.loads(raw, object_pairs_hook=pairs)
    require(isinstance(value, dict) and path.read_bytes() == raw, "Input changed during read")
    return value, raw


@contextlib.contextmanager
def connection(repo):
    path = plain(repo / ".jenkins/state/coordination.sqlite3")
    with contextlib.closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as value:
        value.row_factory = sqlite3.Row
        value.execute("PRAGMA query_only=ON")
        value.execute("BEGIN")
        yield value


def record(repo, domain, key):
    with connection(repo) as value:
        row = value.execute("SELECT domain,key,payload,version FROM records WHERE domain=? AND key=?",
                            (domain, key)).fetchone()
        return None if row is None else {**dict(row), "payload": json.loads(row["payload"])}


def capability(repo):
    """Verify the complete installed capsule before importing its runtime reader."""
    repo = plain(repo)
    installed, installed_raw = document(repo / ".jenkins/state/management/installed.json")
    require(installed.get("schemaVersion") == 1 and installed.get("job") == "zircon-management"
            and installed.get("ordinaryEntry") == CAPABILITY and installed.get("formalAcceptance") is False,
            "Ordinary Jenkins entry capability is pending installation", "ordinary_entry_pending")
    checksum = installed.get("capsuleSha256")
    require(isinstance(checksum, str) and SHA.fullmatch(checksum), "Invalid installed capsule hash")
    capsule = plain(installed.get("capsule", ""))
    require(capsule == repo / ".jenkins/runtime/management" / checksum, "Capsule escaped its runtime")
    config, raw = document(capsule / "capsule.json")
    require(sha(raw) == checksum and config.get("repositoryRoot") == str(repo)
            and config.get("ordinaryEntry") == CAPABILITY, "Capsule capability or repository changed")
    require(set(config.get("files", {})) == {"runner.py", "source_preparation.py", "zircon-management.groovy"}
            and set(config.get("clientSources", {})) == CLIENT_SOURCES, "Capsule has an incomplete code inventory")
    for name, expected in config["files"].items():
        require(isinstance(expected, str) and SHA.fullmatch(expected)
                and sha(plain(capsule / name).read_bytes()) == expected, "Pinned capsule source changed")
    for name, expected in config["clientSources"].items():
        require(isinstance(expected, str) and SHA.fullmatch(expected)
                and sha(plain(repo / name).read_bytes()) == expected, "Pinned public entry source changed")
    for name in ("driverDigest", "runtimeOperationId"):
        require(config.get(name) == installed.get(name), "Installed capability differs from its runtime")
    spec = importlib.util.spec_from_file_location("verified_ordinary_management_runner", capsule / "runner.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # Public adapters can already have imported canonical contracts. Rebind
    # this client process to the sealed runtime before boot imports its tools.
    # boot verifies every driver file before any driver module is imported.
    for name in tuple(sys.modules):
        if name == "tools.jenkins" or name.startswith("tools.jenkins."):
            del sys.modules[name]
    verified, bound_repo, manager = module.boot(capsule)
    require(verified == config and bound_repo == repo, "Capsule boot changed its binding")
    from tools.jenkins.deployment.drain_observations import owner_observation
    owner = owner_observation(manager)
    health = manager.health()
    require(owner["operationId"] == config["runtimeOperationId"]
            and all(health.get(k) is True for k in ("controller", "agent", "plugins", "ready", "acceptingRequests"))
            and health.get("quietingDown") is False, "Selected Jenkins runtime is not ready", "ordinary_runtime_pending")
    require(plain(repo / ".jenkins/state/management/installed.json").read_bytes() == installed_raw,
            "Installed capability changed during observation")
    return {"repo": repo, "installed": installed, "config": config, "module": module,
            "manager": manager, "owner": owner}


def authorize(repo, payload):
    require(payload.get("action") in ACTIONS, "Unsupported management action")
    for name in ("sessionId", "requestId", "operationId"):
        require(isinstance(payload.get(name), str) and ID.fullmatch(payload[name])
                and not payload[name].endswith("."), "An exact bounded " + name + " is required")
    session = os.environ.get("ZIRCON_JENKINS_SESSION_ID")
    require(not session or session == payload["sessionId"], "Input belongs to another current session")
    expected_ids = {sha(str(repo.resolve()).casefold().encode())[:32],
                    sha(str(repo).casefold().encode()), sha(encoded(str(repo).casefold()))}
    require(payload.get("repositoryId") in expected_ids, "Repository identity differs from the physical checkout")
    owned = payload.get("ownedPaths")
    require(isinstance(owned, list) and owned and len(set(owned)) == len(owned),
            "Nonempty original ownedPaths are required")
    def relative(name):
        require(isinstance(name, str) and name and "\\" not in name and ":" not in name
                and all(p not in {"", ".", "..", ".git"} for p in name.split("/")),
                "Owned/source paths must be canonical repository-relative paths")
        return name.casefold()
    with connection(repo) as value:
        row = value.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                            (payload["repositoryId"], payload["sessionId"])).fetchone()
        require(row is not None and row["status"] == "active", "Current session authorization is pending")
        allowed, actions, evidence = (json.loads(row[name]) for name in ("paths", "actions", "evidence"))
        snapshot = {"repositoryId": payload["repositoryId"], "sessionId": payload["sessionId"],
                    "principal": row["principal"], "paths": allowed, "actions": actions, "evidence": evidence}
        require(sha(encoded(snapshot)) == row["snapshot_digest"] and evidence.get("source") == "user"
                and evidence.get("task") and AUTH[payload["action"]] in actions, "Original action authorization changed")
        for name in owned:
            path = relative(name)
            require(any(path == scope or path.startswith(scope + "/") for scope in allowed),
                    "ownedPaths exceed the current session grant")
    return relative


def parse_cargo(arguments):
    """Accept bounded Cargo options; reject every unrepresented argument."""
    args = list(arguments)
    require(args and args[0] in {"check", "build", "test"}, "Only check/build/test have a Jenkins recipe")
    recipe = {"verb": args.pop(0), "packages": [], "features": [], "profile": "dev",
              "offline": False, "noDefaultFeatures": False, "allFeatures": False,
              "locked": True, "jobs": 1, "lib": False, "tests": [], "noRun": False}
    flags = {"--offline": "offline", "--no-default-features": "noDefaultFeatures",
             "--all-features": "allFeatures", "--locked": "locked", "--lib": "lib", "--no-run": "noRun"}
    options = {"-p": "packages", "--package": "packages", "--features": "features", "-F": "features",
               "--profile": "profile", "-j": "jobs", "--jobs": "jobs", "--test": "tests"}
    seen, after, index = set(), False, 0
    while index < len(args):
        token = args[index]
        index += 1
        if token == "--":
            require(not after and recipe["verb"] == "test", "Only test harness arguments may follow --")
            after = True
            continue
        if not after and len(token) > 2 and token[:2] in {"-p", "-j", "-F"} and not token.startswith("--"):
            token = token[:2] + "=" + token[2:]
        key, equal, inline = token.partition("=")
        if after:
            require(key == "--test-threads", "Unsupported test harness argument: " + token)
            if not equal:
                require(index < len(args), "--test-threads requires a value")
                inline = args[index]; index += 1
            require("testThreads" not in seen and inline.isascii() and inline.isdigit()
                    and 1 <= int(inline) <= 8, "Invalid or repeated test thread count")
            recipe["testThreads"] = int(inline); seen.add("testThreads")
            continue
        if token == "--release":
            require("profile" not in seen, "Cargo profile was supplied more than once")
            recipe["profile"] = "release"; seen.add("profile")
        elif key in flags and not equal:
            field = flags[key]
            require(field not in seen, "Repeated Cargo flag: " + token)
            recipe[field] = True; seen.add(field)
        elif key in options:
            field = options[key]
            if not equal:
                require(index < len(args), key + " requires a value")
                inline = args[index]; index += 1
            require(inline and not inline.startswith("-"), "Missing Cargo option value")
            if field in {"packages", "tests"}:
                require(re.fullmatch(r"[A-Za-z0-9_.-]{1,80}" if field == "tests" else r"[A-Za-z0-9_-]{1,80}", inline)
                        and inline not in recipe[field],
                        "Invalid or repeated Cargo selector")
                recipe[field].append(inline)
            elif field == "features":
                features = re.split(r"[,\s]+", inline)
                require(all(re.fullmatch(r"[A-Za-z0-9_:/.-]{1,120}", f) for f in features),
                        "Invalid feature selector")
                recipe[field].extend(features)
            else:
                require(field not in seen, "Repeated Cargo option: " + key)
                if field == "jobs":
                    require(inline.isascii() and inline.isdigit() and 1 <= int(inline) <= 8, "Invalid Cargo jobs")
                    recipe[field] = int(inline)
                else:
                    require(inline in {"dev", "release"}, "Unsupported Cargo profile: " + inline)
                    recipe[field] = inline
                seen.add(field)
        elif not token.startswith("-") and recipe["verb"] == "test" and "testFilter" not in recipe:
            require(len(token) <= 256 and not any(c in token for c in "\0\r\n"), "Invalid test filter")
            recipe["testFilter"] = token
        else:
            raise EntryError("cargo_option_unsupported", "No Jenkins recipe for Cargo argument: " + token)
    require(1 <= len(recipe["packages"]) <= 8, "Select one to eight scoped packages with -p/--package")
    require(not (recipe["allFeatures"] and recipe["features"]), "--all-features cannot be mixed with --features")
    require(recipe["verb"] == "test" or not (recipe["noRun"] or "testFilter" in recipe),
            "Test selectors require cargo test")
    require(not (recipe["noRun"] and "testThreads" in seen), "--no-run cannot use test harness arguments")
    if recipe["verb"] == "test":
        recipe.setdefault("testThreads", 1)
    require(not (recipe["allFeatures"] and recipe["noDefaultFeatures"]),
            "--all-features cannot be combined with --no-default-features")
    return recipe


def output_policy(repo, payload, *, target_dir=None, build_root=None):
    from tools.jenkins.resources.paths import canonical_build_root, build_root_containing, physical_path_under
    target = target_dir or payload.get("requestedTargetDir")
    requested_root = build_root or payload.get("buildRoot")
    if target:
        root = build_root_containing(target)
        physical = physical_path_under(root, target)
        require(physical != root.path, "Target directory must be below its build root")
        require(not requested_root or canonical_build_root(requested_root).path == root.path,
                "Explicit target and buildRoot are on different roots")
        requested_root = str(root.path)
    else:
        root = canonical_build_root(requested_root or document(repo / ".jenkins/deployment-spec.json")[0]["storage"]["buildRoot"])
    result = {"buildRoot": str(root.path)}
    if target:
        result["requestedTargetDir"] = str(physical)
    return result


def source_digest(context, payload):
    """Reuse the pinned job's complete read-only source capture; create no copy."""
    from tools.jenkins.state import State
    actor = object.__new__(context["module"].Management)
    actor.config, actor.repo, actor.manager, actor.payload = (
        context["config"], context["repo"], context["manager"], payload)
    actor.repo_id, actor.session, actor.owned = payload["repositoryId"], payload["sessionId"], payload["ownedPaths"]
    actor.worktrees = plain(actor.repo / ".jenkins/worktrees")
    actor.state = object.__new__(State)
    actor.state.path = actor.repo / ".jenkins/state/coordination.sqlite3"
    actor.state.connect = lambda: connection(actor.repo)
    previous = os.environ.get("GIT_OPTIONAL_LOCKS")
    os.environ["GIT_OPTIONAL_LOCKS"] = "0"
    try:
        with contextlib.redirect_stdout(sys.stderr):
            source_root, before = actor.validation_with_cache()
        hashes = payload.get("sourceHashes", {})
        require(isinstance(hashes, dict), "sourceHashes must be a file hash map")
        for name, expected in hashes.items():
            require(any(name == p or name.startswith(p.rstrip("/") + "/") for p in actor.owned)
                    and (expected is None or isinstance(expected, str) and SHA.fullmatch(expected))
                    and before["source"]["files"].get(name) == expected, "An authored source hash changed")
        with contextlib.redirect_stdout(sys.stderr):
            _after_root, after = actor.validation_with_cache()
        require(_after_root == source_root and after == before, "Source changed during pre-submit capture")
        actor.verify_declared_inputs(before, payload)
        return before["sourceDigest"], source_root, context["module"].source_components(before)
    finally:
        if previous is None: os.environ.pop("GIT_OPTIONAL_LOCKS", None)
        else: os.environ["GIT_OPTIONAL_LOCKS"] = previous


def input_payload(context, payload, *, identity=None, recipe=None, target_dir=None, build_root=None):
    value = dict(payload)
    if identity:
        for name, expected in identity.items():
            require(value.get(name) == expected, "Input differs from submit argument: " + name)
    for name in ("driverDigest", "runtimeOperationId"):
        require(value.get(name) == context["config"][name], "Original request differs from installed " + name)
    authorize(context["repo"], value)
    if value["action"] == "validate":
        if recipe:
            for name, expected in recipe.items():
                require(name not in value or value[name] == expected, "Input differs from Cargo option: " + name)
                value[name] = expected
        else:
            require(value.get("verb") in {"check", "build", "test"} and value.get("packages"),
                    "An explicit scoped validation recipe is required")
        policy = output_policy(context["repo"], value, target_dir=target_dir, build_root=build_root)
        for name, expected in policy.items():
            if name in value:
                require(plain(value[name]) == Path(expected), "Input differs from explicit output option: " + name)
            value[name] = expected
        validator = object.__new__(context["module"].Management)
        validator.payload = value
        validator.validation_recipe()
        require(value.get("sourceView") in {"checkout", "worktree"}, "An explicit sourceView is required")
        if value["sourceView"] == "checkout":
            require(not value.get("worktreeId") and value.get("externalRoots") == [str(plain(context["repo"].parent / "zr_vm"))],
                    "Checkout validation needs its explicit canonical external dependency")
    require(value.get("observationOnly") in (None, False), "Use QueryOnly to observe the original input")
    value.pop("observationOnly", None)
    return value


def submit_payload(context, payload, *, query_only=False, dry_run=False):
    repo, manager = context["repo"], context["manager"]
    captured_components = None
    operation_key = sha(encoded({name: payload[name] for name in ("repositoryId", "sessionId", "requestId", "operationId")}))
    marker_key = sha("|".join(payload[name] for name in ("sessionId", "requestId", "operationId")).encode())
    marker = plain(repo / ".jenkins/state/management/submissions" / (marker_key + ".json"))
    old = document(marker)[0] if marker.exists() else None
    existing = record(repo, "management_operation", operation_key)
    original = (old or {}).get("payload") or (existing["payload"].get("payload") if existing else None)
    if original is not None:
        candidate = dict(payload)
        if (payload["action"] == "validate" and "expectedSourceDigest" not in candidate
                and "expectedSourceDigest" in original):
            candidate["expectedSourceDigest"] = original.get("expectedSourceDigest")
        require(candidate == original, "Original identity already has different action/input/source/runtime")
        payload = candidate
    elif old or query_only:
        return {"status": "pending", "reasonCode": "original_delivery_input_unavailable" if old else "original_operation_missing",
                "requestId": payload["requestId"], "operationId": payload["operationId"],
                "resubmitted": False, "formalAcceptance": False, "m8Accepted": False}
    elif payload["action"] == "validate":
        # Admission requests bind the owned engine and let the queued Jenkins
        # actor select and seal the complete external closure. A supplied full
        # digest retains its original strict pre-submit capture semantics.
        if (payload.get("externalCapturePolicy") != "jenkins-admission"
                or "expectedSourceDigest" in payload):
            captured, _root, captured_components = source_digest(context, payload)
            require("expectedSourceDigest" not in payload or payload["expectedSourceDigest"] == captured,
                    "Complete source input differs from its declared digest")
            payload = {**payload, "expectedSourceDigest": captured}
    input_digest = sha(encoded(payload))
    envelope = {"repositoryId": payload["repositoryId"], "sessionId": payload["sessionId"],
                "requestId": payload["requestId"], "operationId": payload["operationId"],
                "driverDigest": payload["driverDigest"], "runtimeOperationId": payload["runtimeOperationId"],
                "inputDigest": input_digest, "formalAcceptance": False, "m8Accepted": False}
    if existing:
        value = existing["payload"]
        require(value.get("inputDigest") == input_digest, "Original operation input differs")
        return {**envelope, "status": "observed" if value["phase"] == "complete" else "pending",
                "phase": value["phase"], "result": value.get("result"),
                "resubmitted": False}
    if old:
        require(old.get("inputDigest", input_digest) == input_digest, "Original delivery input differs")
        return {**envelope, "status": "pending", "phase": old["phase"], "queueUrl": old.get("queueUrl"),
                "reasonCode": "original_delivery_requires_reconciliation", "resubmitted": False,
                "nextAction": "Query the same identity; do not submit a replacement"}
    if query_only:
        return {**envelope, "status": "pending", "reasonCode": "original_operation_missing", "resubmitted": False}
    if dry_run:
        return {**envelope, "status": "prepared-not-submitted", "payload": payload,
                "targetPolicy": "managed-compatibility-pool",
                "requestedTargetDir": payload.get("requestedTargetDir"), "buildRoot": payload.get("buildRoot"),
                "actualTargetDir": None}
    parameters = {"SESSION_ID": payload["sessionId"], "REQUEST_ID": payload["requestId"],
                  "OPERATION_ID": payload["operationId"], "ACTION": payload["action"],
                  "INPUT_JSON": encoded(payload).decode(), "EXPECTED_DRIVER_DIGEST": payload["driverDigest"],
                  "RUNTIME_OPERATION_ID": payload["runtimeOperationId"], "QUERY_ONLY": "false"}
    body = urllib.parse.urlencode(parameters).encode("ascii")
    saved = {**envelope, "payload": payload, "bodyDigest": sha(body), "phase": "dispatching", "queueUrl": None}
    if captured_components is not None:
        saved["sourceComponents"] = captured_components
    # Retain one durable delivery claim before HTTP. A lost acknowledgement is
    # reconciled with this same identity and is never sent as a second request.
    marker.parent.mkdir(parents=True, exist_ok=True)
    try:
        with marker.open("xb") as stream:
            stream.write(encoded(saved) + b"\n"); stream.flush(); os.fsync(stream.fileno())
    except FileExistsError:
        return submit_payload(context, payload, query_only=True)
    try:
        from tools.jenkins.deployment.drain_observations import owner_observation
        owner_observation(manager, expected=context["owner"])
        status, raw = manager._request("crumbIssuer/api/json")
        require(status == 200, "Jenkins crumb is unavailable")
        crumb = json.loads(raw)
        status, _raw, headers = manager._request_with_headers(
            "job/zircon-management/buildWithParameters", method="POST", data=body,
            headers={"Content-Type": "application/x-www-form-urlencoded",
                     crumb["crumbRequestField"]: crumb["crumb"]}, timeout=30)
        queue = next((value for name, value in headers.items() if name.casefold() == "location"), None)
        require(status == 201 and isinstance(queue, str) and queue.startswith(manager.base_url.rstrip("/") + "/queue/item/"),
                "Submission acknowledgement is unknown")
        saved.update(phase="submitted", queueUrl=queue)
    except Exception:
        saved.update(phase="delivery_unknown")
    with marker.open("wb") as stream:
        stream.write(encoded(saved) + b"\n"); stream.flush(); os.fsync(stream.fileno())
    return {**envelope, "status": "submitted" if saved["phase"] == "submitted" else "pending",
            "phase": saved["phase"], "queueUrl": saved["queueUrl"], "resubmitted": False,
            "nextAction": "Query this same identity", "targetPolicy": "managed-compatibility-pool",
            "buildRoot": payload.get("buildRoot"), "requestedTargetDir": payload.get("requestedTargetDir"),
            "actualTargetDir": None}


def dispatch(repo, payload, *, identity=None, recipe=None, target_dir=None, build_root=None,
             query_only=False, dry_run=False):
    context = capability(repo)
    value = input_payload(context, payload, identity=identity, recipe=recipe,
                          target_dir=target_dir, build_root=build_root)
    return submit_payload(context, value, query_only=query_only, dry_run=dry_run)


def cargo_entry(repo, arguments, *, identity_file=None, target_dir=None, build_root=None,
                query_only=False, dry_run=False, runtime_profile=None, unlocked=False):
    recipe = parse_cargo(arguments)
    if unlocked:
        require("--locked" not in arguments, "Unlocked input conflicts with --locked")
        recipe["locked"] = False
    if runtime_profile:
        import tomllib
        presets = tomllib.loads(plain(Path(repo) / "zircon_runtime/runtime-feature-presets.toml").read_text(encoding="utf-8"))
        choices = [p for p in presets["profiles"] if p["id"] == runtime_profile]
        require(len(choices) == 1 and not recipe["features"] and not recipe["allFeatures"],
                "Runtime profile conflicts with explicit features")
        require(recipe["packages"] in (["zircon_app"], ["zircon_runtime"]),
                "Runtime profile needs zircon_app/zircon_runtime; other packages require explicit FeatureOverride")
        field = "app_features" if recipe["packages"] == ["zircon_app"] else "runtime_features"
        recipe["features"] = list(choices[0][field])
        recipe["noDefaultFeatures"] = True
    path = identity_file or os.environ.get("ZIRCON_JENKINS_IDENTITY_FILE")
    require(path, "Original Jenkins identity file is required", "ordinary_identity_pending")
    payload, _raw = document(path)
    require(payload.get("action") == "validate", "Cargo entry requires action=validate")
    return dispatch(repo, payload, recipe=recipe, target_dir=target_dir, build_root=build_root,
                    query_only=query_only, dry_run=dry_run)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for command in ("cargo", "submit", "capability"):
        sub = commands.add_parser(command)
        sub.add_argument("--repo-root", required=True)
        if command == "cargo":
            sub.add_argument("--identity-file")
            sub.add_argument("--target-dir")
            sub.add_argument("--build-root")
            sub.add_argument("--runtime-profile")
            sub.add_argument("--unlocked", action="store_true")
            sub.add_argument("--query-only", action="store_true")
            sub.add_argument("--dry-run", action="store_true")
            sub.add_argument("cargo_args", nargs=argparse.REMAINDER)
        elif command == "submit":
            sub.add_argument("--input-file", required=True)
            for name in ("session-id", "request-id", "operation-id", "action"):
                sub.add_argument("--" + name, required=True)
            sub.add_argument("--query-only", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.command == "capability":
            context = capability(args.repo_root)
            result = {"status": "ready", "ordinaryEntry": CAPABILITY,
                      "driverDigest": context["config"]["driverDigest"],
                      "runtimeOperationId": context["config"]["runtimeOperationId"],
                      "formalAcceptance": False, "m8Accepted": False}
        elif args.command == "cargo":
            values = args.cargo_args[1:] if args.cargo_args[:1] == ["--"] else args.cargo_args
            result = cargo_entry(args.repo_root, values, identity_file=args.identity_file,
                                 target_dir=args.target_dir, build_root=args.build_root,
                                 runtime_profile=args.runtime_profile, unlocked=args.unlocked,
                                 query_only=args.query_only, dry_run=args.dry_run)
        else:
            payload, _raw = document(args.input_file)
            identity = {"sessionId": args.session_id, "requestId": args.request_id,
                        "operationId": args.operation_id, "action": args.action}
            result = dispatch(args.repo_root, payload, identity=identity, query_only=args.query_only)
        print(encoded(result).decode())
        return 2 if result["status"] == "pending" else 0
    except Exception as error:
        print(encoded({"status": "pending", "reasonCode": getattr(error, "code", "ordinary_entry_unproven"),
                       "message": str(error), "formalAcceptance": False, "m8Accepted": False}).decode())
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
