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
PREVIEW_ACTION = "preview-current-editor-r34"

ACTIONS = {"create-branch", "create-worktree", "prepare-merge", "apply-patch",
           "commit-worktree", "release-worktree", "recover-worktree", "validate", "generate",
           "prepare-flow", "submit-flow", "prepare-worktree", "inspect-worktree", "retire-worktree-reader", PREVIEW_ACTION}
AUTH = {name: "implementation" for name in ACTIONS}
AUTH.update({PREVIEW_ACTION: "validation", "validate": "validation", "submit-flow": "validation", "prepare-flow": "validation",
             "prepare-worktree": "validation", "inspect-worktree": "validation", "retire-worktree-reader": "validation",
             "commit-worktree": "commit", "release-worktree": "gc", "recover-worktree": "gc"})
MANAGED_WORKTREE_CAPABILITY = {
    "enabled": True, "schemaVersion": 1, "actions": sorted(ACTIONS - {PREVIEW_ACTION}),
    "sourcePreparation": "managed-worktree-source-preparation-v1",
    "compilerInputs": "managed-worktree-product-inputs-v1", "formalAcceptance": False,
}

ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
RECIPE_FIELDS = frozenset({
    "verb", "packages", "features", "profile", "offline", "noDefaultFeatures",
    "allFeatures", "locked", "jobs", "lib", "tests", "testFilter", "noRun", "testThreads",
    "buildRoot", "requestedTargetDir", "bins", "allTests", "harnessFilters",
    "exact", "ignored", "nocapture", "manifestPath",
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


def _public_transport_manager(repo, config, capsule, installed):
    """Bind the pinned HTTP receiver; native lifecycle admission stays server-side."""
    import base64
    import urllib.error
    import urllib.request
    import xml.etree.ElementTree as ET

    selected, _ = document(repo / ".jenkins/state/deployment/driver.json")
    driver = plain(repo / ".jenkins/runtime/drivers" / config["driverDigest"])
    require(plain(selected["root"]) == driver
            and all(selected[k] == config[k] for k in ("driverDigest", "runtimeOperationId")),
            "Selected runtime changed")
    manifest, _ = document(driver / "driver-manifest.json")
    require(sha(encoded({k: manifest[k] for k in ("schemaVersion", "files")}))
            == config["driverDigest"], "Driver manifest changed")
    for name, fact in manifest["files"].items():
        require(isinstance(name, str) and name and "\\" not in name and ":" not in name
                and all(p not in {"", ".", ".."} for p in name.split("/")),
                "Driver member escaped its root")
        data = plain(driver / name).read_bytes()
        require(len(data) == fact["bytes"] and sha(data) == fact["sha256"],
                "Pinned driver source changed")
    if config.get("managedWorktree") is not None:
        require({"tools/jenkins/managed_worktree.py", "tools/jenkins/managed_worktree_lifetime.py",
                 "tools/jenkins/product_inputs.py"} <= set(manifest["files"]),
                "Managed worktree capability lacks its sealed implementation")
    sys.path.insert(0, str(driver))
    from tools.jenkins.deployment.driver import driver_environment
    from tools.jenkins.deployment.spec import load_spec
    from tools.jenkins.deployment.paths import resolve_paths
    from tools.jenkins.deployment.manager import DeploymentManager
    os.environ.update(driver_environment({"root": str(driver), "driverDigest": config["driverDigest"]},
                                        repo / ".jenkins/runtime/python/python.exe", repo))
    deployment = load_spec(repo / ".jenkins/deployment-spec.json")
    require(deployment.controller["listenAddress"] == "127.0.0.1"
            and type(deployment.controller["httpPort"]) is int
            and 1 <= deployment.controller["httpPort"] <= 65535, "Loopback HTTP binding differs")

    class Direct(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            return None

    class Transport(DeploymentManager):
        # Retain the existing read-only spec/paths/journal interface for legacy
        # source capture and delivery reconciliation. Never call lifecycle methods.
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), Direct())

        def _request_with_headers(self, path, *, method="GET", data=None, headers=None, timeout=30):
            user, token = self._credentials()
            authorization = "Basic " + base64.b64encode(
                (user + ":" + token).encode()).decode("ascii")
            request = urllib.request.Request(self.base_url.rstrip("/") + "/" + path.lstrip("/"),
                data=data, method=method,
                headers={"Authorization": authorization, "Accept": "application/json", **(headers or {})})
            try:
                response = self.opener.open(request, timeout=min(timeout, 30))
            except urllib.error.HTTPError as error:
                response = error
            with response:
                body = response.read(8 * 1024**2 + 1)
                require(len(body) <= 8 * 1024**2, "HTTP response exceeds the input bound")
                return response.status, body, dict(response.headers.items())

        def verify_public_transport(self):
            current, _ = document(repo / ".jenkins/state/management/installed.json")
            selected, _ = document(repo / ".jenkins/state/deployment/driver.json")
            runtime, _ = document(repo / ".jenkins/state/deployment-operation.json")
            require(current == installed and plain(selected["root"]) == driver
                    and all(selected[k] == config[k] for k in ("driverDigest", "runtimeOperationId"))
                    and runtime.get("operationId") == runtime.get("generation") == config["runtimeOperationId"]
                    and runtime.get("state") == "running", "Selected HTTP runtime changed")
            status, raw = self._request("job/zircon-management/config.xml")
            require(status == 200, "Management HTTP receiver unavailable")
            job = ET.fromstring(raw)
            definition = job.find("definition")
            require(job.tag == "flow-definition" and definition is not None
                    and definition.get("class") == "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition"
                    and definition.findtext("sandbox") == "true"
                    and job.findtext("./disabled") == "false", "Management HTTP receiver owner differs")
            script = definition.findtext("script")
            require(isinstance(script, str) and script, "Management HTTP receiver script is missing")
            codex = re.findall(r"(?m)^def codex = '([A-Za-z]:/[^'\r\n\"]+)'$", script)
            require(len(codex) == 1, "Pinned management launcher differs")
            template = plain(capsule / "zircon-management.groovy").read_text(
                encoding="utf-8-sig").replace("\r\n", "\n")
            values = {"AGENT": config["agentLabel"], "REPOSITORY": str(repo),
                      "PYTHON": str(repo / ".jenkins/runtime/python/python.exe"), "CODEX": codex[0],
                      "CAPSULE": str(capsule), "DRIVER": config["driverDigest"],
                      "RUNTIME": config["runtimeOperationId"]}
            for key, value in values.items():
                template = template.replace("@@" + key + "@@", value.replace("\\", "/").replace("'", "\\'"))
            require("@@" not in template and script.replace("\r\n", "\n").count(template) == 1,
                    "Management HTTP receiver differs from its pinned capsule")
            require({v.findtext("name") for v in job.findall(
                "./properties/hudson.model.ParametersDefinitionProperty/parameterDefinitions/*")} ==
                {"SESSION_ID", "REQUEST_ID", "OPERATION_ID", "ACTION", "INPUT_JSON",
                 "EXPECTED_DRIVER_DIGEST", "RUNTIME_OPERATION_ID", "QUERY_ONLY"},
                "Management parameters differ")

    manager = Transport(deployment, resolve_paths(deployment))
    require(manager.paths.repo.absolute() == repo
            and manager.paths.state.absolute() == repo / ".jenkins/state"
            and manager.paths.home.absolute() == repo / ".jenkins/jenkins_home",
            "HTTP deployment paths differ from the canonical repository")
    manager.verify_public_transport()
    return manager


def capability(repo, *, public_transport=False):
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
    capsule_files = set(config.get("files", {}))
    legacy_files = {"runner.py", "source_preparation.py", "zircon-management.groovy"}
    checkout_files = legacy_files | {"checkout_guard.py"}
    product_files = checkout_files | {"checkout_product.py"}
    require(capsule_files in (legacy_files, checkout_files, product_files)
            and set(config.get("clientSources", {})) == CLIENT_SOURCES, "Capsule has an incomplete code inventory")
    checkout_capability = {"enabled": True, "policy": "actual-checkout", "guard": "checkout_guard.py"}
    require(config.get("checkoutValidation") == (checkout_capability if "checkout_guard.py" in capsule_files else None),
            "Capsule checkout validation capability differs from its pinned code")
    product_capabilities = (
        {"enabled": True, "kind": "zircon-development-editor-preview", "schemaVersion": 1, "helper": "checkout_product.py"},
        {"enabled": True, "kind": "zircon-development-editor-preview", "schemaVersion": 2, "helper": "checkout_product.py"},
    )
    configured_product = config.get("checkoutProduct")
    require((capsule_files == product_files and configured_product in product_capabilities)
            or (capsule_files != product_files and configured_product is None),
            "Capsule checkout product differs from its exact pinned inventory")
    for name, expected in config["files"].items():
        require(isinstance(expected, str) and SHA.fullmatch(expected)
                and sha(plain(capsule / name).read_bytes()) == expected, "Pinned capsule source changed")
    spec = importlib.util.spec_from_file_location("verified_ordinary_management_runner", capsule / "runner.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    for name, expected in config["clientSources"].items():
        path = (module.client_path(config, repo, name, capsule=capsule)
                if "clientRoot" in config else plain(repo / name))
        require(isinstance(expected, str) and SHA.fullmatch(expected)
                and sha(path.read_bytes()) == expected, "Pinned public entry source changed")
    managed = config.get("managedWorktree")
    require(managed in (None, MANAGED_WORKTREE_CAPABILITY) and installed.get("managedWorktree") == managed,
            "Managed worktree capability differs from its exact installed inventory")
    for name in ("driverDigest", "runtimeOperationId"):
        require(config.get(name) == installed.get(name), "Installed capability differs from its runtime")
    # Public adapters can already have imported canonical contracts. Rebind
    # this client process to the sealed runtime before boot imports its tools.
    # boot verifies every driver file before any driver module is imported.
    for name in tuple(sys.modules):
        if name == "tools" or name == "tools.jenkins" or name.startswith("tools.jenkins."):
            del sys.modules[name]
    if public_transport:
        manager = _public_transport_manager(repo, config, capsule, installed)
    else:
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
    context = {"repo": repo, "installed": installed, "config": config, "module": module, "manager": manager}
    return {**context, "publicTransport": True} if public_transport else {**context, "owner": owner}


def authorize(repo, payload, *, context=None, automatic=False, origin_session=None, read_connection=None):
    require(payload.get("action") in ACTIONS, "Unsupported management action")
    for name in ("sessionId", "requestId", "operationId"):
        require(isinstance(payload.get(name), str) and ID.fullmatch(payload[name])
                and not payload[name].endswith("."), "An exact bounded " + name + " is required")
    session = os.environ.get("ZIRCON_JENKINS_SESSION_ID")
    require(not session or session == (origin_session or payload["sessionId"]),
            "Input belongs to another current session")
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
    for name in owned:
        relative(name)
    public_transport = context is not None and context.get("publicTransport") is True
    if automatic and context is not None and not public_transport:
        context["module"].ensure_request_context(
            context["config"], repo, context["manager"], payload, record_request=False)
    with (contextlib.nullcontext(read_connection) if read_connection is not None else connection(repo)) as value:
        row = value.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                            (payload["repositoryId"], payload["sessionId"])).fetchone()
        if row is None and automatic and public_transport and AUTH[payload["action"]] in {"implementation", "validation"}:
            # Only the pinned server may establish this context under its native owner.
            return relative
        require(row is not None and row["status"] == "active", "Current session authorization is pending")
        allowed, actions, evidence = (json.loads(row[name]) for name in ("paths", "actions", "evidence"))
        snapshot = {"repositoryId": payload["repositoryId"], "sessionId": payload["sessionId"],
                    "principal": row["principal"], "paths": allowed, "actions": actions, "evidence": evidence}
        manual = evidence.get("source") == "user" and evidence.get("task")
        kind = evidence.get("kind")
        writer_context = (kind in {"ordinary-validation", "managed-worktree-writer"}
            and evidence.get("ownerPrincipal") == row["principal"]
            and actions == ["implementation", "validation"])
        reader_context = False
        if kind == "managed-worktree-reader" and context is not None:
            if public_transport:
                # This is a read-only issued-grant check, never a live owner claim.
                owner_matches = isinstance(evidence.get("ownerPrincipal"), str) and re.fullmatch(
                    r"windows:S-1-[0-9-]+", evidence["ownerPrincipal"]) is not None
            else:
                helper = context["module"].source_helper()
                owner_matches = evidence.get("ownerPrincipal") == helper.local_request_principal(context["manager"])
            reader_context = (actions == ["validation"]
                and evidence.get("writerPrincipal") == row["principal"] and owner_matches)
        automatic_context = (evidence.get("source") == "automatic-local-request"
            and evidence.get("policy") == "local-validation-v1"
            and evidence.get("repositoryId") == payload["repositoryId"]
            and evidence.get("repositoryRoot") == str(repo)
            and isinstance(evidence.get("inputDigest"), str) and SHA.fullmatch(evidence["inputDigest"])
            and (writer_context or reader_context))
        require(sha(encoded(snapshot)) == row["snapshot_digest"] and (manual or automatic_context)
                and AUTH[payload["action"]] in actions, "Original action authorization changed")
        for name in owned:
            path = relative(name)
            require(any(path == scope or path.startswith(scope + "/") for scope in allowed),
                    "ownedPaths exceed the current session grant")
    return relative


def normalize_cargo_manifest(value, *, repo=None):
    """Bind the CLI manifest to a catalog entry below the admitted source root.

    Native cwd stays the admitted source root. Caller-selected cwd and arbitrary
    manifests are not part of this bounded recipe.
    """
    choices = ("Cargo.toml", "zircon_plugins/Cargo.toml")
    require(isinstance(value, str) and value and not any(c in value for c in "\0\r\n"),
            "A catalog Cargo manifest is required", "cargo_manifest_unsupported")
    if value in choices:
        return value
    if repo is not None:
        root = plain(repo)
        for relative in choices:
            expected = root / relative
            if value in {str(expected), expected.as_posix()}:
                plain(expected)
                return relative
    raise EntryError("cargo_manifest_unsupported",
                     "Select Cargo.toml or zircon_plugins/Cargo.toml below the selected repository")


def parse_cargo(arguments, *, enforce_policy=True, repo=None):
    """Accept bounded Cargo options; explicit test profile requires cargo test."""
    args = list(arguments)
    require(args and args[0] in {"check", "build", "test"}, "Only check/build/test have a Jenkins recipe")
    recipe = {"verb": args.pop(0), "packages": [], "features": [], "profile": "dev",
              "offline": False, "noDefaultFeatures": False, "allFeatures": False,
              "locked": True, "jobs": 1, "lib": False, "tests": [], "noRun": False}
    flags = {"--offline": "offline", "--no-default-features": "noDefaultFeatures",
             "--all-features": "allFeatures", "--locked": "locked", "--lib": "lib", "--no-run": "noRun",
             "--bins": "bins", "--tests": "allTests"}
    options = {"-p": "packages", "--package": "packages", "--features": "features", "-F": "features",
               "--profile": "profile", "-j": "jobs", "--jobs": "jobs", "--test": "tests",
               "--manifest-path": "manifestPath"}
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
            harness_flags = {"--exact": "exact", "--ignored": "ignored", "--nocapture": "nocapture"}
            if token in harness_flags:
                field = harness_flags[token]
                require(field not in seen, "Repeated test harness flag: " + token)
                recipe[field] = True; seen.add(field)
                continue
            if not token.startswith("-"):
                require(token and len(token.encode("utf-8")) <= 256
                        and not any(c in token for c in "\0\r\n"), "Invalid harness filter")
                filters = recipe.setdefault("harnessFilters", [])
                require(len(filters) < 512 and token not in filters, "Repeated or excessive harness filters")
                filters.append(token)
                continue
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
            elif field == "manifestPath":
                require(field not in seen, "Repeated Cargo option: " + key)
                recipe[field] = normalize_cargo_manifest(inline, repo=repo)
                seen.add(field)
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
                    require(inline in {"dev", "release", "test"}, "Unsupported Cargo profile: " + inline)
                    require(inline != "test" or recipe["verb"] == "test",
                            "The test profile requires cargo test")
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
    require(not (recipe["noRun"] and after), "--no-run cannot use test harness arguments")
    if recipe["verb"] == "test":
        recipe.setdefault("testThreads", 1)
    if recipe.get("manifestPath") == "zircon_plugins/Cargo.toml":
        require(recipe["packages"] == ["zircon_plugin_material_editor_editor"]
                and recipe["verb"] == "test" and recipe["profile"] == "test"
                and recipe["noDefaultFeatures"] and not recipe["features"]
                and recipe["locked"] and recipe["lib"] and not recipe["tests"]
                and recipe["jobs"] == recipe["testThreads"] == 1
                and recipe.get("testFilter") == "material_graph_"
                and not any(recipe.get(name) for name in (
                    "allFeatures", "allTests", "bins", "noRun", "harnessFilters", "exact", "ignored", "nocapture")),
                "The plugin manifest permits only the material_graph_ single-package lib test recipe",
                "build_scope_policy")
    require(not (recipe["allFeatures"] and recipe["noDefaultFeatures"]),
            "--all-features cannot be combined with --no-default-features")
    if enforce_policy:
        require(recipe["noDefaultFeatures"] and not recipe["allFeatures"] and not recipe.get("allTests"),
                "Compact builds require --no-default-features and explicit features; --all-features/--tests are prohibited",
                "build_scope_policy")
        require(recipe["verb"] != "test" or (recipe["lib"] and not recipe["tests"])
                or (not recipe["lib"] and len(recipe["tests"]) == 1),
                "Select --lib or exactly one --test target before compiling tests", "build_scope_policy")
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
    require(payload.get("manifestPath") != "zircon_plugins/Cargo.toml"
            or root.path == plain(Path("E:/cargo-targets")),
            "The plugin recipe requires the physical E:/cargo-targets build root", "build_scope_policy")
    result = {"buildRoot": str(root.path)}
    if target:
        result["requestedTargetDir"] = str(physical)
    return result


def capture_drift_diagnostics(module, source_root, before, after_root, after):
    """Explain the two existing captures without changing their rejection contract."""
    try:
        def checksum(value):
            return sha(encoded(value))
        def identity(value):
            if value is None or isinstance(value, str) and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value):
                return value
            if isinstance(value, dict) and set(value) == {"sha256"} and SHA.fullmatch(value["sha256"]):
                return value
            return {"sha256": checksum(value)}
        def path(value):
            text = str(value)
            return {"path": text[:256], "pathSha256": sha(text.encode("utf-8")), "truncated": len(text) > 256}
        def roots(snapshot):
            result = {}
            def visit(root, source):
                result[root] = source
                for name, value in source["files"].items():
                    if isinstance(value, dict) and isinstance(value.get("source"), dict):
                        visit(root.rstrip("/\\") + "/" + name, value["source"])
            visit(snapshot["sourceRoot"], snapshot["source"])
            for external in snapshot["externalRoots"]:
                visit(external["path"], external["source"])
            return result
        def summary(source):
            if source is None:
                return None
            files = source["files"]
            value = {"head": identity(source.get("head")), "sourceDigest": identity(source.get("sourceDigest")),
                     "filesDigest": checksum(files), "fileCount": len(files),
                     "membershipDigest": checksum(sorted(files))}
            if "gitIndex" in source:
                index = source["gitIndex"]
                value.update(gitIndexDigest=checksum(index), gitIndexEntryCount=len(index["entries"]),
                             gitIndexMetadataSha256=identity(index.get("metadataSha256")))
            if "diffSha256" in source:
                value["diffSha256"] = identity(source["diffSha256"])
            return value
        old, new = roots(before), roots(after)
        names = sorted(set(old) | set(new))
        changed = [name for name in names if old.get(name) != new.get(name)]
        components = ("sourceView", "sourceRoot", "sourceDigest", "source", "externalRoots", "dependencyCache")
        details = {"captureRootBefore": path(source_root), "captureRootAfter": path(after_root),
                   "sourceDigestBefore": identity(before.get("sourceDigest")),
                   "sourceDigestAfter": identity(after.get("sourceDigest")),
                   "changedComponents": [name for name in components if before.get(name) != after.get(name)],
                   "dependencyCacheBeforeSha256": checksum(before.get("dependencyCache")),
                   "dependencyCacheAfterSha256": checksum(after.get("dependencyCache")),
                   "rootCount": len(names), "changedRootCount": len(changed),
                   "rootsTruncated": len(changed) > 8, "outputTruncated": False, "roots": []}
        for name in changed[:8]:
            left, right = old.get(name), new.get(name)
            left_files, right_files = (left or {}).get("files", {}), (right or {}).get("files", {})
            added, removed = sorted(set(right_files) - set(left_files)), sorted(set(left_files) - set(right_files))
            changes = module.input_changes(left_files, right_files, limit=16)
            changes["values"] = [{"path": path("/".join(value["path"])),
                                  "before": identity(value["before"]), "after": identity(value["after"])}
                                 for value in changes["values"]]
            details["roots"].append({"root": path(name), "before": summary(left), "after": summary(right),
                "addedPathCount": len(added), "removedPathCount": len(removed),
                "addedPaths": [path(item) for item in added[:16]],
                "removedPaths": [path(item) for item in removed[:16]],
                "membershipPathsTruncated": len(added) > 16 or len(removed) > 16, "fileChanges": changes})
        # Keep a hard byte limit as well as per-root/path limits, including Unicode paths.
        prefix = "[ordinary-entry] pre-submit capture drift "
        while len(prefix.encode("utf-8")) + len(encoded(details)) + 2 > 32768:
            details["outputTruncated"] = True
            root = details["roots"][-1]
            if root["fileChanges"]["values"]:
                root["fileChanges"]["values"].pop()
                root["fileChanges"]["truncated"] = True
            elif root["addedPaths"] or root["removedPaths"]:
                (root["addedPaths"] or root["removedPaths"]).pop()
                root["membershipPathsTruncated"] = True
            else:
                details["roots"].pop()
                details["rootsTruncated"] = True
        print(prefix + encoded(details).decode("utf-8"), file=sys.stderr)
    except Exception:
        with contextlib.suppress(Exception):
            print("[ordinary-entry] pre-submit capture drift diagnostics unavailable", file=sys.stderr)


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
        if _after_root != source_root or after != before:
            capture_drift_diagnostics(context["module"], source_root, before, _after_root, after)
        require(_after_root == source_root and after == before, "Source changed during pre-submit capture")
        actor.verify_declared_inputs(before, payload)
        return before["sourceDigest"], source_root, context["module"].source_components(before)
    finally:
        if previous is None: os.environ.pop("GIT_OPTIONAL_LOCKS", None)
        else: os.environ["GIT_OPTIONAL_LOCKS"] = previous


def _replay_route(context, payload, *, read_connection=None):
    """Read only an existing exact origin/routed mapping; never create a context."""
    repo = context["repo"]
    names = ("repositoryId", "sessionId", "requestId", "operationId")
    with (contextlib.nullcontext(read_connection) if read_connection is not None else connection(repo)) as current:
        class ReadRecords:
            def get(self, domain, key, *, connection=None):
                require(connection is None or connection is current, "Read-only query connection changed")
                row = current.execute("SELECT domain,key,payload,version FROM records WHERE domain=? AND key=?",
                                      (domain, key)).fetchone()
                return None if row is None else {**dict(row), "payload": json.loads(row["payload"])}
        state = ReadRecords()
        origin = {name: payload[name] for name in names}
        helper = context["module"].source_helper()
        replay = helper._managed_route_replay(state, payload, sha(encoded(origin)), connection=current)
        value, route = replay if replay is not None else (dict(payload), None)
        if route is not None:
            require(route.get("schemaVersion") == 1 and route.get("repositoryRoot") == str(repo)
                    and route.get("driverDigest") == value["driverDigest"]
                    and route.get("runtimeOperationId") == value["runtimeOperationId"]
                    and route.get("routeRef") == sha(encoded(route.get("originIdentity"))),
                    "Original managed route binding changed")
            for who, snapshot_name in (("originIdentity", "originSnapshot"),):
                bound = route.get(who, {})
                row = current.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                      (bound.get("repositoryId"), bound.get("sessionId"))).fetchone()
                require(row is not None and row["status"] == "active",
                        "Original managed route authorization changed")
                snapshot = {"repositoryId": bound.get("repositoryId"), "sessionId": bound.get("sessionId"),
                            "principal": row["principal"], "paths": json.loads(row["paths"]),
                            "actions": json.loads(row["actions"]), "evidence": json.loads(row["evidence"])}
                require(sha(encoded(snapshot)) == row["snapshot_digest"]
                        and row["snapshot_digest"] == route.get(snapshot_name),
                        "Original managed route authorization changed")
            routed_identity = {name: value[name] for name in names}
            require(routed_identity == route.get("routedIdentity")
                    and routed_identity["repositoryId"] == origin["repositoryId"],
                    "Original routed identity changed")
            if route["routedIdentity"] != route["originIdentity"]:
                alias = state.get("management_context_route_alias", sha(encoded(routed_identity)), connection=current)
                require(alias is not None and alias["payload"] == {"routeRef": route["routeRef"]},
                        "Original managed route alias changed")
            snapshot_name = ("writerAuthorizationSnapshot" if value["sessionId"] == route.get("writerSessionId")
                             else "readerAuthorizationSnapshot" if value["sessionId"] == route.get("readerSessionId")
                             else None)
            row = current.execute("SELECT status,snapshot_digest FROM authorizations WHERE repository_id=? AND session_id=?",
                                  (value["repositoryId"], value["sessionId"])).fetchone()
            require(snapshot_name is not None and row is not None and row["status"] == "active"
                    and row["snapshot_digest"] == route.get(snapshot_name),
                    "Original routed authorization changed")
        return value, route


def _delivery_marker(context, payload, route):
    def leaf(identity):
        key = sha("|".join(identity[name] for name in ("sessionId", "requestId", "operationId")).encode())
        return plain(context["repo"] / ".jenkins/state/management/submissions" / (key + ".json"))
    marker = leaf(payload)
    if context.get("publicTransport") is True and route is not None:
        candidates = {leaf(route["originIdentity"]), leaf(route["routedIdentity"])}
        existing = [path for path in candidates if path.exists()]
        require(len(existing) <= 1, "Origin and routed delivery markers are ambiguous")
        if existing:
            marker = existing[0]
    return marker


def _stored_public_delivery(context, old, route):
    """Bind the actual POST bytes before bridging to the server's routed input."""
    delivered = old.get("payload")
    require(isinstance(delivered, dict) and delivered.get("observationOnly") in (None, False),
            "Original delivery payload is unavailable")
    for name in ("repositoryId", "sessionId", "requestId", "operationId", "driverDigest", "runtimeOperationId"):
        require(old.get(name) == delivered.get(name), "Original delivery envelope changed: " + name)
    require(old.get("inputDigest") == sha(encoded(delivered))
            and old.get("phase") in {"dispatching", "submitted", "delivery_unknown"}
            and old.get("formalAcceptance") is False and old.get("m8Accepted") is False,
            "Original delivery input or phase changed")
    parameters = {"SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
                  "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
                  "INPUT_JSON": encoded(delivered).decode(), "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
                  "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"], "QUERY_ONLY": "false"}
    require(old.get("bodyDigest") == sha(urllib.parse.urlencode(parameters).encode("ascii")),
            "Original delivery body changed")
    routed, observed = _replay_route(context, delivered)
    require(observed == route, "Original delivery managed route changed")
    return routed


def input_payload(context, payload, *, identity=None, recipe=None, target_dir=None, build_root=None,
                  automatic_context=True, routing=None):
    value = dict(payload)
    if identity:
        for name, expected in identity.items():
            require(value.get(name) == expected, "Input differs from submit argument: " + name)
    for name in ("driverDigest", "runtimeOperationId"):
        require(value.get(name) == context["config"][name], "Original request differs from installed " + name)
    origin_session = value.get("sessionId")
    session = os.environ.get("ZIRCON_JENKINS_SESSION_ID")
    require(not session or session == origin_session, "Input belongs to another current session")
    if context.get("publicTransport") is True:
        value, route = _replay_route(context, value)
    else:
        value, route = context["module"].route_request_context(
            context["config"], context["repo"], context["manager"], value,
            query_only=not automatic_context)
    if route is not None and routing is not None:
        routing.update(route)
    authorize(context["repo"], value, context=context, automatic=automatic_context,
              origin_session=origin_session if route is not None else None)
    if value["action"] == PREVIEW_ACTION:
        require(recipe is None and target_dir is None and build_root is None,
                "Fixed preview accepts only its descriptor inputs")
        context["module"].fixed_editor_preview_recipe(value)
    if value["action"] in {"prepare-worktree", "inspect-worktree", "retire-worktree-reader"} or value.get("worktreePreparationRef"):
        require(context["config"].get("managedWorktree") == MANAGED_WORKTREE_CAPABILITY,
                "Managed worktree source preparation is pending installation", "managed_worktree_pending")
    if "checkoutProduct" in value:
        require(value["action"] == "validate" and recipe is None, "Fixed checkout product uses its validation-only two-command contract")
        product = value.get("checkoutProduct")
        require(isinstance(product, dict) and type(product.get("schemaVersion")) is int
                and product["schemaVersion"] in {1, 2},
                "A supported fixed editor product schema is required", "checkout_product_schema_unsupported")
        if product["schemaVersion"] == 2:
            require(context["config"].get("checkoutProduct", {}).get("schemaVersion") == 2,
                    "The installed management capsule does not support product schema 2",
                    "checkout_product_schema_unsupported")
        helper = context["module"].checkout_product_helper(context["config"])
        product_batch = helper.batch_payload(value, context["repo"])
        require("batch" not in value or value["batch"] == product_batch, "Fixed product batch argv changed")
        value["batch"] = product_batch
    if value["action"] == "validate":
        if "batch" in value:
            require(recipe is None, "A batch cannot be combined with a single Cargo adapter recipe")
        elif recipe:
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
        validator.payload, validator.repo, validator.config = value, context["repo"], context["config"]
        if "batch" in value:
            validator.validation_batch()
        else:
            validator.validation_recipe()
        require(value.get("sourceView") in {"checkout", "worktree"}, "An explicit sourceView is required")
        if value["sourceView"] == "checkout":
            require(context["config"].get("checkoutValidation") == {
                        "enabled": True, "policy": "actual-checkout", "guard": "checkout_guard.py"}
                    and "checkout_guard.py" in context["config"].get("files", {}),
                    "Actual checkout validation requires its installed pinned read guard",
                    "checkout_input_capability_pending")
            require(not value.get("worktreeId") and value.get("externalRoots") == [str(plain(context["repo"].parent / "zr_vm"))],
                    "Checkout validation needs its explicit canonical external dependency")
    require(value.get("observationOnly") in (None, False), "Use QueryOnly to observe the original input")
    value.pop("observationOnly", None)
    return value



def resume_delivery(context, payload, marker, original, operation, input_digest):
    """Prove the last exact Jenkins delivery terminal before one explicit retry."""
    require("batch" in payload and operation is not None,
            "Only a durably checkpointed batch can resume its original delivery")
    repo, manager = context["repo"], context["manager"]
    deliveries = sorted(marker.parent.glob(marker.stem + ".resume-*.json"))
    require(len(deliveries) < 64, "Original resume delivery bound reached")
    previous_path = deliveries[-1] if deliveries else marker
    previous, previous_raw = document(previous_path)
    delivered = payload
    if context.get("publicTransport") is True:
        _, route = _replay_route(context, payload)
        require(previous.get("phase") == "submitted"
                and previous.get("payload") == original.get("payload"),
                "Unknown original delivery must be reconciled before resume")
        require(_stored_public_delivery(context, previous, route) == payload,
                "Original delivery does not match the routed operation")
        delivered = previous["payload"]
    else:
        require(previous.get("payload") == payload and previous.get("inputDigest") == input_digest
                and previous.get("phase") == "submitted", "Unknown original delivery must be reconciled before resume")
    queue = previous.get("queueUrl")
    prefix = manager.base_url.rstrip("/") + "/queue/item/"
    require(isinstance(queue, str) and queue.startswith(prefix)
            and re.fullmatch(r"\d+/?", queue[len(prefix):]), "Original Jenkins queue identity is unavailable")
    queue_id = int(queue[len(prefix):].strip("/"))
    status, raw = manager._request("queue/item/" + str(queue_id) + "/api/json")
    if status == 404:
        # Jenkins expires left queue items. Reconcile the same queue ID against
        # a bounded build inventory; absence or ambiguity still forbids POST.
        status, raw = manager._request("job/zircon-management/api/json?tree=builds[number,url,queueId]{0,128}")
        require(status == 200, "Original queue needs explicit terminal reconciliation")
        candidates = [v for v in json.loads(raw).get("builds", []) if v.get("queueId") == queue_id]
        require(len(candidates) == 1, "Original expired queue lacks one exact build")
        queued = {"id": queue_id, "executable": candidates[0]}
    else:
        require(status == 200, "Original queue needs explicit terminal reconciliation")
        queued = json.loads(raw)
    require(queued.get("id") == queue_id and queued.get("cancelled") is not True,
            "Original executable delivery identity is unavailable")
    executable = queued.get("executable", {})
    number = executable.get("number")
    require(type(number) is int and number > 0
            and executable.get("url") == manager.base_url.rstrip("/") + "/job/zircon-management/" + str(number) + "/",
            "Original queue lacks its exact management build")
    status, raw = manager._request("job/zircon-management/" + str(number) + "/api/json")
    require(status == 200, "Original build cannot be observed")
    build = json.loads(raw)
    parameters = [action["parameters"] for action in build.get("actions", [])
                  if isinstance(action, dict) and isinstance(action.get("parameters"), list)]
    require(build.get("building") is False and build.get("queueId") == queue_id
            and build.get("result") in {"SUCCESS", "FAILURE", "ABORTED", "UNSTABLE", "NOT_BUILT"}
            and len(parameters) == 1, "Original management delivery is not proven terminal")
    observed = {value["name"]: value.get("value") for value in parameters[0]}
    expected = {"SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
                "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
                "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
                "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"]}
    require(all(observed.get(k) == v for k, v in expected.items())
            and observed.get("QUERY_ONLY") in (False, "false")
            and json.loads(observed.get("INPUT_JSON", "null")) == delivered,
            "Original terminal build used different input/source/runtime")
    from tools.jenkins.state import State
    actor = object.__new__(context["module"].Management)
    actor.config, actor.repo, actor.manager, actor.payload = context["config"], repo, manager, payload
    actor.repo_id, actor.session, actor.request, actor.operation = (
        payload["repositoryId"], payload["sessionId"], payload["requestId"], payload["operationId"])
    actor.key, actor.input_digest, actor.owned = operation["key"], input_digest, payload["ownedPaths"]
    actor.state = object.__new__(State)
    actor.state.path = repo / ".jenkins/state/coordination.sqlite3"
    actor.state.connect = lambda: connection(repo)
    require(actor.batch_recovery_ready(), "Original batch has an unproven Native/EOF or preparation claim")
    destination = marker.with_name(marker.stem + ".resume-%04d.json" % (len(deliveries) + 1))
    return destination, {"resumeOf": {"path": str(previous_path), "sha256": sha(previous_raw)},
                         "previousQueueUrl": queue, "previousBuild": number}


def submit_payload(context, payload, *, query_only=False, dry_run=False, resume_original=False):
    repo, manager = context["repo"], context["manager"]
    route = None
    if context.get("publicTransport") is True:
        payload, route = _replay_route(context, payload)
    captured_components = None
    operation_key = sha(encoded({name: payload[name] for name in ("repositoryId", "sessionId", "requestId", "operationId")}))
    marker = _delivery_marker(context, payload, route)
    old = document(marker)[0] if marker.exists() else None
    existing = record(repo, "management_operation", operation_key)
    original = (old or {}).get("payload") or (existing["payload"].get("payload") if existing else None)
    if old is not None and context.get("publicTransport") is True:
        original = _stored_public_delivery(context, old, route)
        require(existing is None or existing["payload"].get("payload") == original,
                "Original State and delivery payloads differ")
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
    elif payload["action"] == "validate" and "batch" not in payload:
        # Batch capture belongs to the one admitted Jenkins parent. Legacy
        # single recipes retain their original strict pre-submit semantics.
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
    resume_fact = None
    if existing:
        value = existing["payload"]
        require(value.get("inputDigest") == input_digest, "Original operation input differs")
        if not resume_original or query_only or value["phase"] == "complete":
            return {**envelope, "status": "observed" if value["phase"] == "complete" else "pending",
                    "phase": value["phase"], "result": value.get("result"), "resubmitted": False}
    if resume_original and not query_only:
        require(old is not None and existing is not None, "Original durable delivery/operation is required to resume")
        marker, resume_fact = resume_delivery(context, payload, marker, old, existing, input_digest)
    if old and resume_fact is None:
        require(context.get("publicTransport") is True or old.get("inputDigest", input_digest) == input_digest,
                "Original delivery input differs")
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
    # A retry preserves the original POST and receipt; State remains routed.
    delivered, delivery_envelope = payload, envelope
    if resume_fact is not None and context.get("publicTransport") is True:
        delivered = old["payload"]
        delivery_envelope = {**envelope, **{name: delivered[name] for name in (
            "repositoryId", "sessionId", "requestId", "operationId", "driverDigest", "runtimeOperationId")},
            "inputDigest": sha(encoded(delivered))}
    parameters = {"SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
                  "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
                  "INPUT_JSON": encoded(delivered).decode(), "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
                  "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"], "QUERY_ONLY": "false"}
    body = urllib.parse.urlencode(parameters).encode("ascii")
    saved = {**delivery_envelope, "payload": delivered, "bodyDigest": sha(body), "phase": "dispatching", "queueUrl": None}
    if resume_fact is not None:
        require(old.get("bodyDigest") == sha(body), "Original delivery body changed")
        saved.update(resume_fact)
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
        if context.get("publicTransport") is True:
            manager.verify_public_transport()
        else:
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
            "phase": saved["phase"], "queueUrl": saved["queueUrl"], "resubmitted": resume_fact is not None,
            "nextAction": "Query this same identity", "targetPolicy": "managed-compatibility-pool",
            "buildRoot": payload.get("buildRoot"), "requestedTargetDir": payload.get("requestedTargetDir"),
            "actualTargetDir": None}



def historical_query(context, payload, *, identity=None, recipe=None, target_dir=None, build_root=None):
    """Observe an original delivery in one read-only authority snapshot."""
    repo = context["repo"]
    require(isinstance(payload, dict), "An original request object is required")
    if identity:
        for name, expected in identity.items():
            require(payload.get(name) == expected, "Input differs from submit argument: " + name)
    require(isinstance(payload.get("driverDigest"), str) and SHA.fullmatch(payload["driverDigest"])
            and isinstance(payload.get("runtimeOperationId"), str) and ID.fullmatch(payload["runtimeOperationId"]),
            "The original driver and runtime identities are required")
    names = ("repositoryId", "sessionId", "requestId", "operationId")
    def normalized(value):
        require(isinstance(value, dict), "Original delivery payload is unavailable")
        require("observationOnly" not in value or value["observationOnly"] is False,
                "Only an original non-observation delivery can be queried")
        return {name: item for name, item in value.items() if name != "observationOnly"}
    normalized(payload)
    with connection(repo) as current:
        authorize(repo, payload, context=context, read_connection=current)
        class ReadRecords:
            def get(self, domain, key, *, connection=None):
                require(connection is None or connection is current, "Read-only query connection changed")
                row = current.execute("SELECT domain,key,payload,version FROM records WHERE domain=? AND key=?",
                                      (domain, key)).fetchone()
                return None if row is None else {**dict(row), "payload": json.loads(row["payload"])}
        state = ReadRecords()
        value, route = _replay_route(context, payload, read_connection=current)
        authorize(repo, value, context=context, origin_session=payload["sessionId"], read_connection=current)
        routed_identity = {name: value[name] for name in names}
        key = sha(encoded(routed_identity))
        existing = state.get("management_operation", key, connection=current)
        marker = _delivery_marker(context, value, route)
        old, marker_raw = document(marker) if marker.exists() else (None, None)
        original = None
        if existing is not None:
            operation = existing["payload"]
            original = normalized(operation.get("payload"))
            require(existing["key"] == sha(encoded({name: original.get(name) for name in names}))
                    and operation.get("action") == original.get("action")
                    and operation.get("phase") in {"running", "waiting", "complete", "unknown"}
                    and operation.get("inputDigest") == sha(encoded(original)),
                    "Original operation input or phase changed")
            for name in (*names, "action", "driverDigest", "runtimeOperationId"):
                require(name not in operation or operation[name] == original.get(name),
                        "Original operation envelope changed: " + name)
            if isinstance(operation.get("result"), dict):
                for name, expected in {**{name: original.get(name) for name in (*names, "driverDigest", "runtimeOperationId")},
                                       "operationKey": key}.items():
                    require(name not in operation["result"] or operation["result"][name] == expected,
                            "Original operation result identity changed: " + name)
        if old is not None:
            delivered = old.get("payload")
            delivery = normalized(delivered)
            if context.get("publicTransport") is True:
                delivery, delivered_route = _replay_route(context, delivery, read_connection=current)
                require(delivered_route == route, "Original delivery managed route changed")
            for name in (*names, "driverDigest", "runtimeOperationId"):
                require(old.get(name) == delivered.get(name), "Original delivery envelope changed: " + name)
            require(sha(encoded({name: delivery.get(name) for name in names})) == key
                    and old.get("inputDigest") == sha(encoded(delivered))
                    and old.get("phase") in {"dispatching", "submitted", "delivery_unknown"}
                    and old.get("formalAcceptance") is False and old.get("m8Accepted") is False,
                    "Original delivery input or phase changed")
            parameters = {"SESSION_ID": delivered["sessionId"], "REQUEST_ID": delivered["requestId"],
                          "OPERATION_ID": delivered["operationId"], "ACTION": delivered["action"],
                          "INPUT_JSON": encoded(delivered).decode(), "EXPECTED_DRIVER_DIGEST": delivered["driverDigest"],
                          "RUNTIME_OPERATION_ID": delivered["runtimeOperationId"], "QUERY_ONLY": "false"}
            require(old.get("bodyDigest") == sha(urllib.parse.urlencode(parameters).encode("ascii")),
                    "Original delivery body changed")
            require(original is None or original == delivery, "Original State and delivery payloads differ")
            original = delivery
        if original is None:
            result = {"status": "pending", "reasonCode": "original_operation_missing",
                      "requestId": value["requestId"], "operationId": value["operationId"],
                      "resubmitted": False, "formalAcceptance": False, "m8Accepted": False}
        else:
            candidate = normalized(value)
            if recipe is not None:
                require(value["action"] == "validate" and "batch" not in original and isinstance(recipe, dict)
                        and not set(recipe) - RECIPE_FIELDS, "Original Cargo adapter recipe is incompatible")
                for name, expected in recipe.items():
                    require(name not in candidate or candidate[name] == expected,
                            "Input differs from Cargo option: " + name)
                    candidate[name] = expected
            for name, expected in (("requestedTargetDir", target_dir), ("buildRoot", build_root)):
                if expected is not None:
                    require(name in original and original[name] == expected
                            and (name not in candidate or candidate[name] == expected),
                            "Input differs from explicit output option: " + name)
                    candidate[name] = expected
            if value["action"] == "validate":
                for name in RECIPE_FIELDS | {"expectedSourceDigest"}:
                    if name not in candidate and name in original:
                        candidate[name] = original[name]
            require(candidate == original, "Original identity already has different action/input/source/runtime")
            envelope = {**{name: original[name] for name in (*names, "driverDigest", "runtimeOperationId")},
                        "inputDigest": sha(encoded(original)), "formalAcceptance": False, "m8Accepted": False}
            if existing is not None:
                operation = existing["payload"]
                result = {**envelope, "status": "observed" if operation["phase"] == "complete" else "pending",
                          "phase": operation["phase"], "result": operation.get("result"), "resubmitted": False}
            else:
                result = {**envelope, "status": "pending", "phase": old["phase"], "queueUrl": old.get("queueUrl"),
                          "reasonCode": "original_delivery_requires_reconciliation", "resubmitted": False,
                          "nextAction": "Query the same identity; do not submit a replacement"}
        require(marker_raw is None or marker.read_bytes() == marker_raw, "Original delivery changed during query")
        return {**result, "managedContext": route} if route is not None else result


def dispatch(repo, payload, *, identity=None, recipe=None, target_dir=None, build_root=None,
             query_only=False, dry_run=False, resume_original=False):
    context = capability(repo, public_transport=True)
    require(not query_only or not (dry_run or resume_original),
            "QueryOnly cannot be combined with dry-run or resume")
    if query_only:
        return historical_query(context, payload, identity=identity, recipe=recipe,
                                target_dir=target_dir, build_root=build_root)
    routing = {}
    value = input_payload(context, payload, identity=identity, recipe=recipe,
                          target_dir=target_dir, build_root=build_root,
                          automatic_context=not query_only and not dry_run, routing=routing)
    result = submit_payload(context, value, query_only=query_only, dry_run=dry_run,
                            resume_original=resume_original)
    return {**result, "managedContext": routing} if routing else result


def cargo_entry(repo, arguments, *, identity_file=None, target_dir=None, build_root=None,
                query_only=False, dry_run=False, runtime_profile=None, unlocked=False):
    scoped_arguments = list(arguments)
    if runtime_profile and not query_only and "--no-default-features" not in scoped_arguments:
        # A named runtime preset explicitly selects its own feature set.
        require("--" not in scoped_arguments, "Runtime preset needs explicit --no-default-features before harness arguments")
        scoped_arguments.append("--no-default-features")
    recipe = parse_cargo(scoped_arguments, enforce_policy=not query_only, repo=repo)
    if unlocked:
        require("--locked" not in arguments, "Unlocked input conflicts with --locked")
        recipe["locked"] = False
    if runtime_profile:
        require(not query_only, "Historical Cargo queries need explicit original features, not a current runtime profile")
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
            sub.add_argument("cargo_args", nargs=argparse.REMAINDER,
                             help="Scoped Cargo argv; --profile test is supported only with cargo test")
        elif command == "submit":
            sub.add_argument("--input-file", required=True)
            sub.add_argument("--resume-original", action="store_true")
            for name in ("session-id", "request-id", "operation-id", "action"):
                sub.add_argument("--" + name, required=True)
            sub.add_argument("--query-only", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.command == "capability":
            context = capability(args.repo_root, public_transport=True)
            result = {"status": "ready", "verificationMode": "public-transport", "nativeLifecycleVerified": False,
                      "ordinaryEntry": CAPABILITY,
                      "managedWorktree": context["config"].get("managedWorktree"),
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
            result = dispatch(args.repo_root, payload, identity=identity, query_only=args.query_only,
                              resume_original=args.resume_original)
        print(encoded(result).decode())
        return 2 if result["status"] == "pending" else 0
    except Exception as error:
        print(encoded({"status": "pending", "reasonCode": getattr(error, "code", "ordinary_entry_unproven"),
                       "message": str(error), "formalAcceptance": False, "m8Accepted": False}).decode())
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
