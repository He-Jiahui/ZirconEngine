"""Pinned development editor product on the guarded actual checkout."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import sys
from types import SimpleNamespace

KIND = "zircon-development-editor-preview"
TASK = "mvp-editor-native-input-contract-01a0dbf0-20261013-r1"
RESPONSE = ".jenkins/state/records-queue/followups/responses/5b9604e4189a17622df94c78401988561c6ad137552ad1ddefddc931b8560a37.json"
RESPONSE_SHA = "0292d6e32c962de6bfd379760eaafe346f3b2eb277d4692b82073e4e5bed0f46"
PARENT_QUERY = ".jenkins/state/requests/operator-editor/2a6c73ad723be3b034b83299b52c4390a0598b7da00a71d10863bd2ccddcb622/current-source-query.json"
PARENT_QUERY_SHA = "89c75198a8c3dd0e80c8835d083a04801910a43d038a366b72a989be05698bc8"
BUILDER = "tools/build/build_editor.py"
BUILDER_SHA = "f9275213de306405ebaa705dac24e84b421a7f7d9a37328076118423b4b01143"
COMMANDS = (
    ("cargo", "build", "-p", "zircon_app", "--bin", "zircon_editor", "--locked", "--offline",
     "--no-default-features", "--features", "target-editor-host", "--jobs", "2",
     "--message-format=json-render-diagnostics"),
    ("cargo", "rustc", "-p", "zircon_runtime", "--lib", "--crate-type", "cdylib", "--locked",
     "--offline", "--no-default-features", "--features", "target-editor-host", "--jobs", "2"),
)
PHASES = ["stage_editor_bundle", "publish_editor_bundle"]
PIN_FIELDS = {"baseHead", "ownedPaths", "untrackedPaths", "sourceHashes",
              "externalGitlinks", "expectedEngineSourceDigest"}
IDENTITY_FIELDS = ("repositoryId", "sessionId", "requestId", "operationId",
                   "driverDigest", "runtimeOperationId")
PRODUCT_INPUT_FIELDS = {
    "action", *IDENTITY_FIELDS, "baseHead", "ownedPaths", "untrackedPaths", "sourceHashes",
    "expectedEngineSourceDigest", "expectedSourceDigest", "externalCapturePolicy", "externalGitlinks",
    "externalRoots", "sourceView", "worktreeId", "buildRoot", "checkoutProduct", "batch",
    "observationOnly", "commit", "push", "notify", "publishSource", "reuseValidation",
}
GITLINKS = {
    "tests/third_party/zr_unity/Unity", "zr_vm_core/third_party/zr_utf8proc/utf8proc",
    "zr_vm_core/third_party/zr_xx_hash/xxHash", "zr_vm_lib_ffi/third_party/zr_libffi/libffi",
    "zr_vm_lib_network/third_party/zr_libuv/libuv", "zr_vm_library/third_party/zr_c_json/cJSON",
    "zr_vm_library/third_party/zr_tiny_dir/tinydir",
}
HEX64 = re.compile(r"[0-9a-f]{64}")
HEX40 = re.compile(r"[0-9a-f]{40}")


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def ordinary(path):
    path = Path(path)
    if not path.is_absolute():
        raise ValueError("An absolute physical path is required")
    for part in (path, *path.parents):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 1024:
            raise ValueError("Product paths must not contain reparse points")
    return path


def exact_ref(repo, path, checksum):
    return {"path": str(ordinary(Path(repo) / path)), "sha256": checksum}


def pinned_json(repo, path, checksum):
    origin = ordinary(Path(repo) / path)
    raw = origin.read_bytes()
    if sha(raw) != checksum:
        raise ValueError("Pinned product contract bytes changed")
    value = json.loads(raw)
    if origin.read_bytes() != raw or not isinstance(value, dict):
        raise ValueError("Pinned product contract changed during observation")
    return value


def contract(repo):
    response = pinned_json(repo, RESPONSE, RESPONSE_SHA)
    amendment = response.get("contractAmendment", {})
    query = pinned_json(repo, PARENT_QUERY, PARENT_QUERY_SHA)
    if (response.get("taskId") != TASK or amendment.get("exactCargoCommands") != [list(x) for x in COMMANDS]
            or amendment.get("fixedBundlePhases") != PHASES
            or response.get("originalIdentity", {}).get("plannedRequestId") != "mvp-preview-20261012-r13"
            or query.get("identity", {}).get("requestId") != "mvp-preview-20261012-r13"):
        raise ValueError("The immutable parent and exact editor product contract differ")
    return response, query


def relative(value):
    if not isinstance(value, str) or not value or any(c in value for c in "\\\0\r\n:"):
        raise ValueError("A normal relative source path is required")
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in value.split("/")):
        raise ValueError("A normal relative source path is required")
    return value


def in_scope(name, scopes):
    folded = name.casefold()
    return any(folded == scope.casefold() or folded.startswith(scope.casefold().rstrip("/") + "/")
               for scope in scopes)


def validate_product(payload, repo):
    product = payload.get("checkoutProduct")
    fields = {"schemaVersion", "kind", "contractRef", "parentQueryRef", "logicalIdentity", "sourcePins"}
    if (not isinstance(product, dict) or set(product) != fields
            or type(product.get("schemaVersion")) is not int or product["schemaVersion"] not in {1, 2}
            or product.get("kind") != KIND
            or product.get("contractRef") != exact_ref(repo, RESPONSE, RESPONSE_SHA)
            or product.get("parentQueryRef") != exact_ref(repo, PARENT_QUERY, PARENT_QUERY_SHA)):
        raise ValueError("Typed checkout product references differ")
    schema_version = product["schemaVersion"]
    if schema_version == 2 and set(payload) - PRODUCT_INPUT_FIELDS:
        raise ValueError("Schema 2 accepts only the fixed product and source admission fields")
    response, query = contract(repo)
    amendment = response["contractAmendment"]
    identity = {name: payload.get(name) for name in IDENTITY_FIELDS}
    if (any(not isinstance(v, str) or not v for v in identity.values())
            or product["logicalIdentity"] != identity
            or identity["driverDigest"] != amendment["selectedDriverDigest"]
            or identity["runtimeOperationId"] != amendment["runtimeOperationId"]
            or identity["repositoryId"] != query["identity"]["repositoryId"]):
        raise ValueError("Checkout product identity, driver or runtime differs")
    auth = amendment["currentSuccessorAuthorization"]
    owned, untracked, hashes = (payload.get(name) for name in ("ownedPaths", "untrackedPaths", "sourceHashes"))
    if (auth.get("status") != "active" or auth.get("actions") != ["validation"]
            or identity["sessionId"] != auth.get("sessionId")
            or not isinstance(owned, list) or not owned or not isinstance(untracked, list)
            or not isinstance(hashes, dict) or not hashes):
        raise ValueError("The existing validation grant and explicit source pins are required")
    scopes = [relative(name) for name in owned]
    if (len(set(scopes)) != len(scopes) or len(set(untracked)) != len(untracked)
            or any(not in_scope(name, auth["paths"]) for name in scopes)
            or any(not in_scope(relative(name), scopes) for name in untracked)
            or any(not in_scope(relative(name), scopes) or
                   (value is not None and (not isinstance(value, str) or not HEX64.fullmatch(value)))
                   for name, value in hashes.items())
            or not set(untracked).issubset(hashes)):
        raise ValueError("Owned files, directory scopes and hashes exceed or omit their grant")
    if any(not isinstance(hashes[name], str) or not HEX64.fullmatch(hashes[name]) for name in untracked):
        raise ValueError("Every untracked source file requires its exact SHA-256")
    pins = product.get("sourcePins")
    if (not isinstance(pins, dict) or set(pins) != PIN_FIELDS
            or any(payload.get(name) != pins[name] for name in PIN_FIELDS)
            or not isinstance(pins["baseHead"], str) or not HEX40.fullmatch(pins["baseHead"])
            or schema_version == 1 and (not isinstance(pins["expectedEngineSourceDigest"], str)
                                        or not HEX64.fullmatch(pins["expectedEngineSourceDigest"]))
            or schema_version == 2 and pins["expectedEngineSourceDigest"] is not None
                and (not isinstance(pins["expectedEngineSourceDigest"], str)
                     or not HEX64.fullmatch(pins["expectedEngineSourceDigest"]))):
        raise ValueError("Checkout product must bind every current source pin")
    if schema_version == 2:
        if ("expectedEngineSourceDigest" not in payload
                or payload.get("externalCapturePolicy") != "jenkins-admission"
                or pins["expectedEngineSourceDigest"] is None and "expectedSourceDigest" in payload
                or payload.get("externalRoots") != [str(ordinary(Path(repo).parent / "zr_vm"))]):
            raise ValueError("Schema 2 requires explicit Jenkins source admission without a stale full-source digest")
    links = pins["externalGitlinks"]
    if (not isinstance(links, dict) or set(links) != GITLINKS
            or any(not isinstance(value, dict) or set(value) != {"head", "indexCommit"}
                   or not isinstance(value["head"], str) or not HEX40.fullmatch(value["head"])
                   or value["head"] != value["indexCommit"] for value in links.values())):
        raise ValueError("All seven external Gitlinks require matching index and HEAD pins")
    if (payload.get("action") != "validate" or payload.get("sourceView") != "checkout"
            or payload.get("worktreeId") or ordinary(payload.get("buildRoot", "")) != Path("E:/cargo-targets")
            or any(payload.get(name) for name in ("commit", "push", "notify", "publishSource", "reuseValidation"))
            or schema_version == 2 and set(payload) & {"commit", "push", "notify", "publishSource", "reuseValidation"}):
        raise ValueError("This editor contract permits actual checkout validation on its explicit E root")
    return response, query


def validate_admission_capture(payload, before, repo):
    product = payload.get("checkoutProduct") if isinstance(payload, dict) else None
    if not isinstance(product, dict) or product.get("schemaVersion") != 2:
        return
    validate_product(payload, repo)
    if not isinstance(before, dict) or before.get("sourceView") != "checkout":
        raise ValueError("Schema 2 requires a complete actual-checkout admission capture")
    if before.get("sourceRoot") != str(ordinary(repo)):
        raise ValueError("Schema 2 source capture is not the canonical physical checkout")
    engine = before.get("source")
    if not isinstance(engine, dict) or not isinstance(engine.get("files"), dict):
        raise ValueError("Schema 2 source capture is incomplete")
    engine_body = {name: value for name, value in engine.items() if name != "sourceDigest"}
    if (not isinstance(engine.get("sourceDigest"), str)
            or engine.get("sourceDigest") != sha(encoded(engine_body))):
        raise ValueError("Schema 2 engine capture digest is incomplete")
    if engine.get("head") != payload["baseHead"]:
        raise ValueError("Jenkins-captured checkout HEAD differs from the original baseHead")
    full_body = {name: value for name, value in before.items() if name != "sourceDigest"}
    if (not isinstance(before.get("sourceDigest"), str)
            or before.get("sourceDigest") != sha(encoded(full_body))):
        raise ValueError("Schema 2 complete source/cache admission digest is invalid")


def batch_payload(payload, repo):
    validate_product(payload, repo)
    return {"schemaVersion": 1, "sharedSingleBatchRequired": True,
            "contractRef": payload["checkoutProduct"]["contractRef"],
            "recipes": [{"ordinal": index, "name": name, "cargoArguments": list(argv[1:]),
                         "ownedNativeTimeoutSeconds": 7200}
                        for index, (name, argv) in enumerate(zip(("editor-exe", "runtime-cdylib"), COMMANDS), 1)],
            "stepEnvironments": [{}, {}]}


def validation_batch(payload, repo):
    expected = batch_payload(payload, repo)
    if payload.get("batch") != expected:
        raise ValueError("The fixed editor command sequence or environment changed")
    single = {"verb", "packages", "features", "profile", "offline", "noDefaultFeatures", "lib", "tests",
              "testFilter", "noRun", "allFeatures", "locked", "jobs", "testThreads", "bins",
              "allTests", "harnessFilters", "exact", "ignored", "nocapture"}
    if set(payload) & single:
        raise ValueError("The fixed editor product cannot include a second single recipe")
    recipes = [{"command": list(argv), "verb": argv[1], "packages": [package],
                "features": ["target-editor-host"], "profile": "dev", "offline": True,
                "noDefaultFeatures": True, "allFeatures": False, "locked": True, "jobs": 2,
                "testThreads": 1, "lib": index == 1, "bins": ["zircon_editor"] if index == 0 else [],
                "tests": [], "noRun": False}
               for index, (package, argv) in enumerate(zip(("zircon_app", "zircon_runtime"), COMMANDS))]
    coverage = {**expected, "kind": KIND, "fixedBundlePhases": PHASES,
                "parentQueryRef": payload["checkoutProduct"]["parentQueryRef"]}
    if payload["checkoutProduct"]["schemaVersion"] == 2:
        coverage.update({"productSchemaVersion": 2, "sourceAdmissionPolicy": "jenkins-admission"})
    return {"recipes": recipes, "coverage": coverage,
            "stepEnvironments": [{}, {}], "reuseValidation": None}


def source_inventory_from_capture(before):
    """Flatten the existing complete capture; do not walk or copy source before packaging."""
    result = {}
    def flatten(source, prefix, root_control=False):
        if not isinstance(source, dict) or not isinstance(source.get("files"), dict):
            raise ValueError("A complete captured source inventory is required")
        for name, value in source["files"].items():
            name = relative(name)
            parts = name.split("/")
            if (any(part in {".git", "target", "__pycache__", ".pytest_cache"} for part in parts)
                    or root_control and parts[0] in {".codex", ".jenkins", ".opencode", "dev", "node_modules"}
                    or parts[-1].split(".")[0].upper() in {"NUL", "CON", "PRN", "AUX",
                        *("COM" + str(n) for n in range(1, 10)), *("LPT" + str(n) for n in range(1, 10))}
                    or value is None):
                continue
            label = prefix + name
            if isinstance(value, dict):
                if set(value) != {"gitlinkCommit", "source"} or not HEX40.fullmatch(value["gitlinkCommit"]):
                    raise ValueError("A captured Gitlink is incomplete")
                flatten(value["source"], label + "/")
            elif isinstance(value, str) and HEX64.fullmatch(value):
                if label in result:
                    raise ValueError("Captured source inventories overlap")
                result[label] = value
            else:
                raise ValueError("Captured source file lacks its actual SHA-256")
    flatten(before.get("source"), "source/", root_control=True)
    for item in before.get("externalRoots", []):
        if not isinstance(item, dict) or set(item) != {"path", "source"}:
            raise ValueError("External capture is incomplete")
        root = ordinary(item["path"])
        label = root.name
        if label not in {"zr_vm", "external"}:
            raise ValueError("The pinned packager does not cover this external root")
        flatten(item["source"], label + "/")
    return result


def validate_native_steps(steps, repo):
    if not isinstance(steps, list) or len(steps) != 2:
        raise ValueError("Both native steps must complete before packaging")
    for index, (step, argv) in enumerate(zip(steps, COMMANDS), 1):
        if not isinstance(step, dict):
            raise ValueError("A typed native proof is required")
        command, identity, ref = step.get("command"), step.get("identity", {}), step.get("nativeEvidence")
        if (not isinstance(command, list) or command[1:] != list(argv[1:])
                or not command or not Path(command[0]).is_absolute() or step.get("cwd") != str(repo)
                or step.get("complete") is not True or step.get("childrenGone") is not True
                or step.get("stdoutEof") is not True or step.get("stderrEof") is not True
                or type(step.get("activeProcesses")) is not int or step["activeProcesses"] != 0
                or type(step.get("processExitCode")) is not int or step["processExitCode"] != 0
                or type(identity.get("pid")) is not int or identity["pid"] <= 0
                or not isinstance(identity.get("creationTime"), str)
                or not identity["creationTime"].isascii() or not identity["creationTime"].isdigit()
                or int(identity["creationTime"]) <= 0 or identity.get("executable") != command[0]
                or identity.get("commandDigest") != sha("\0".join(command).encode())
                or not isinstance(ref, dict) or set(ref) != {"path", "sha256"}
                or not isinstance(ref["sha256"], str) or not HEX64.fullmatch(ref["sha256"])
                or not Path(ref["path"]).is_absolute()):
            raise ValueError("Native step %d lacks exact argv or complete terminal/EOF proof" % index)


def stage_and_publish(repo, target, output, key, source_files, native_steps, guard, environment, native_runner):
    repo = ordinary(repo)
    validate_native_steps(native_steps, repo)
    if (not callable(getattr(guard, "assert_stable", None)) or not callable(native_runner)
            or not isinstance(environment, dict) or not HEX64.fullmatch(key)):
        raise ValueError("Continuous source guard, owned native launcher and operation identity are required")
    guard.assert_stable()
    from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
    root = canonical_build_root("E:/cargo-targets")
    out, target = physical_path_under(root, output), physical_path_under(root, target)
    folder = physical_path_under(root, out / "products" / key)
    staging, final = physical_path_under(root, folder / "staging"), physical_path_under(root, folder / "bundle")
    if staging.exists() or final.exists():
        raise ValueError("Product outputs require their original fresh operation destination")
    builder_path = ordinary(repo / BUILDER)
    if sha(builder_path.read_bytes()) != BUILDER_SHA:
        raise ValueError("Pinned build_editor implementation changed")
    spec = importlib.util.spec_from_file_location("pinned_checkout_editor_" + key, builder_path)
    builder = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = builder
    try:
        spec.loader.exec_module(builder)
        builder.subprocess = SimpleNamespace(run=native_runner,
                                            CREATE_NO_WINDOW=getattr(builder.subprocess, "CREATE_NO_WINDOW", 0))
        artifacts = {}
        for name in ("zircon_editor.exe", "zircon_runtime.dll"):
            origin = ordinary(target / "debug" / name)
            if not origin.is_file() or not stat.S_ISREG(origin.lstat().st_mode):
                raise ValueError("The fixed development artifact is missing: " + name)
            artifacts[name] = (origin, sha(origin.read_bytes()))
        staging.mkdir(parents=True, exist_ok=False)
        shell = Path(environment.get("SYSTEMROOT") or environment.get("SystemRoot") or r"C:\Windows") / "System32/WindowsPowerShell/v1.0/powershell.exe"
        package = builder.stage_editor_bundle(repo, staging, artifacts, cargo_profile="development",
                  source_files=source_files, crt_directory=None, shell=shell, environment=environment)
        guard.assert_stable()
        closure = builder.publish_editor_bundle(staging, final, package["output_closure"])
        if builder.verify_editor_bundle(final, package["output_closure"]) != closure:
            raise ValueError("Published product closure differs")
        guard.assert_stable()
        return {"kind": KIND, "buildSet": package["build_set"], "bundle": str(final),
                "outputClosure": closure, "officialCrt": package["official_crt"],
                "formalAcceptance": False, "m8Accepted": False}
    finally:
        sys.modules.pop(spec.name, None)


def verify_published_product(repo, product, expected_bundle):
    if (not isinstance(product, dict)
            or set(product) != {"kind", "buildSet", "bundle", "outputClosure", "officialCrt", "formalAcceptance", "m8Accepted"}
            or product["kind"] != KIND or product["formalAcceptance"] is not False or product["m8Accepted"] is not False
            or ordinary(product["bundle"]) != ordinary(expected_bundle)):
        raise ValueError("Published editor product identity differs from its original destination")
    origin = ordinary(Path(repo) / BUILDER)
    if sha(origin.read_bytes()) != BUILDER_SHA:
        raise ValueError("Pinned output verifier implementation changed")
    spec = importlib.util.spec_from_file_location("pinned_checkout_editor_verifier", origin)
    builder = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = builder
    try:
        spec.loader.exec_module(builder)
        builder.verify_editor_bundle(Path(product["bundle"]), product["outputClosure"])
    finally:
        sys.modules.pop(spec.name, None)
