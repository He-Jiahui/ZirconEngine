"""Read-only build growth and policy evidence for the existing Codex daily audit.

Manifests and Jenkins requests establish candidate writers, not ownership of every
byte in a shared pool. Neither growth nor a missing owner authorizes deletion.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sqlite3
import tomllib

if __package__:
    from . import artifact_inventory as inventory
else:
    import artifact_inventory as inventory

MAX_METADATA_BYTES = 1024 * 1024
MAX_RECORDS = 20000
GROWTH_REVIEW_BYTES = 1024**3


def key(path):
    return str(path).replace("\\", "/").rstrip("/").casefold()


def read_metadata(path):
    inventory.physical_literal(path)
    first = path.stat()
    if first.st_size > MAX_METADATA_BYTES:
        raise ValueError("metadata exceeds read budget")
    with path.open("rb") as stream:
        raw = stream.read(MAX_METADATA_BYTES + 1)
    if len(raw) > MAX_METADATA_BYTES:
        raise ValueError("metadata grew beyond read budget")
    last = path.stat()
    if (first.st_mtime_ns, first.st_size) != (last.st_mtime_ns, last.st_size):
        raise ValueError("metadata changed during read")
    value = json.loads(raw.decode("utf-8-sig"))
    if not isinstance(value, dict):
        raise ValueError("metadata object required")
    return value, hashlib.sha256(raw).hexdigest()


def recipe_violations(recipe):
    """Same bounded-build contract for actual commands and source evidence."""
    violations = []
    if not recipe.get("packages"):
        violations.append("implicit-workspace")
    if recipe.get("allFeatures"):
        violations.append("all-features")
    if not recipe.get("noDefaultFeatures"):
        violations.append("default-features")
    if recipe.get("allTests") or recipe.get("allTargets"):
        violations.append("all-test-targets")
    if recipe.get("verb") == "test" and not recipe.get("lib") and len(recipe.get("tests", [])) != 1:
        violations.append("test-target-unspecified")
    return violations


def observed_cargo(command):
    # Do not split a PowerShell/string command or log arbitrary command bytes.
    if not isinstance(command, list) or not all(isinstance(x, str) for x in command):
        return None
    positions = [i for i, word in enumerate(command) if word.replace("\\", "/").rsplit("/", 1)[-1].casefold() in {"cargo", "cargo.exe"}]
    if len(positions) != 1:
        return None
    args = command[positions[0] + 1:]
    if not args or args[0] not in {"check", "build", "test"}:
        return None
    args = args[:args.index("--")] if "--" in args else args
    packages, tests = [], []
    for i, token in enumerate(args):
        if token in {"-p", "--package", "--test"} and i + 1 < len(args):
            (tests if token == "--test" else packages).append(args[i + 1])
        elif token.startswith("--package=") or token.startswith("-p") and not token.startswith("--") and len(token) > 2:
            packages.append(token.split("=", 1)[1] if "=" in token else token[2:])
        elif token.startswith("--test="):
            tests.append(token.split("=", 1)[1])
    return {"verb": args[0], "packages": packages, "tests": tests, "lib": "--lib" in args,
            "noDefaultFeatures": "--no-default-features" in args,
            "allFeatures": "--all-features" in args, "allTests": "--tests" in args,
            "allTargets": "--all-targets" in args or "--workspace" in args or "--all" in args}


def manifest_roots(value):
    roots = value.get("output_roots", value.get("outputRoots", []))
    result = list(roots.values()) if isinstance(roots, dict) else list(roots) if isinstance(roots, list) else []
    result.extend(x.get("path") for x in value.get("artifacts", []) if isinstance(x, dict))
    return [key(x) for x in result if isinstance(x, str) and Path(x).is_absolute()]


def source_roots(repo, errors):
    roots = {key(repo): {"path": str(repo), "sessionId": None, "ownership": "shared-checkout"}}
    database = repo / ".jenkins/state/coordination.sqlite3"
    try:
        inventory.physical_literal(database)
        with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=5) as connection:
            connection.execute("PRAGMA query_only=ON")
            connection.execute("PRAGMA temp_store=MEMORY")
            rows = connection.execute("SELECT key,payload FROM records WHERE domain='management_worktree' LIMIT ?", (MAX_RECORDS + 1,)).fetchall()
        if len(rows) > MAX_RECORDS:
            raise ValueError("worktree metadata exceeds read budget")
        for identifier, raw in rows:
            value = json.loads(raw)
            if value.get("status") != "ready":
                continue
            path = inventory.physical_literal(Path(value["path"]))
            roots[key(path)] = {"path": str(path), "sessionId": value.get("sessionId"),
                                "worktreeId": identifier, "branch": value.get("branch"),
                                "creationOperationKey": value.get("creationOperationKey")}
    except (OSError, ValueError, TypeError, KeyError, sqlite3.Error) as error:
        errors.append({"subject": str(database), "error": type(error).__name__})
    return roots


def profile_observations(roots, errors):
    observations = []
    for owner in roots.values():
        root = Path(owner["path"])
        for relative in ("Cargo.toml", "zircon_plugins/Cargo.toml", "examples/woc/native/Cargo.toml"):
            path = root / relative
            try:
                inventory.physical_literal(path)
                first = path.stat()
                if first.st_size > MAX_METADATA_BYTES:
                    raise ValueError("manifest exceeds read budget")
                raw = path.read_bytes()
                last = path.stat()
                if (first.st_mtime_ns, first.st_size) != (last.st_mtime_ns, last.st_size):
                    raise ValueError("source changed during read")
                profiles = tomllib.loads(raw.decode("utf-8-sig")).get("profile", {})
                if not isinstance(profiles, dict):
                    raise ValueError("invalid Cargo profile table")
                for name in ("dev", "test"):
                    profile = profiles.get(name, {})
                    if not isinstance(profile, dict) or not isinstance(profile.get("package", {}), dict):
                        raise ValueError("invalid Cargo profile override table")
                    problems = []
                    if profile.get("incremental", True) is not False:
                        problems.append("incremental-enabled")
                    if profile.get("debug", 2) not in (0, False, "none"):
                        problems.append("debug-symbols-enabled")
                    for package, override in {**profile.get("package", {}), "build-override": profile.get("build-override", {})}.items():
                        if not isinstance(override, dict):
                            raise ValueError("invalid Cargo package profile override")
                        if override.get("incremental") is True or override.get("debug", 0) not in (0, False, "none"):
                            problems.append("package-override:" + package)
                    observations.append({"path": str(path), "profile": name, "violations": problems,
                        "sourceSha256": hashlib.sha256(raw).hexdigest(), "owner": owner,
                        "scope": "source configuration; not proof this source created any measured bytes"})
            except FileNotFoundError:
                errors.append({"subject": str(path), "error": "SourceUnavailable"})
            except (OSError, ValueError, tomllib.TOMLDecodeError) as error:
                errors.append({"subject": str(path), "error": type(error).__name__})
    return observations


def audit(repo, scan, previous=None):
    """Consume the existing disjoint deep inventory, without a second file walk."""
    previous = previous or {}
    errors, writers, units, requests = [], [], {}, []
    rows = scan.get("outputItems", []) if scan else []
    complete = bool(scan) and scan.get("complete") is not False and not scan.get("scopes") and all(row.get("complete") is True for row in rows)
    if not complete:
        errors.append({"subject": "deep-inventory", "error": "InventoryIncompleteOrScoped"})
    for row in rows:
        for path, counts in row.get("buildUnits", {}).items():
            unit = units.setdefault(key(path), {"path": path, "logicalBytes": 0, "files": 0,
                "incrementalBytes": 0, "symbolBytes": 0, "complete": True})
            for name in ("logicalBytes", "files", "incrementalBytes", "symbolBytes"):
                unit[name] += counts[name]
            unit["complete"] &= row.get("complete") is True
        if row.get("complete") is not True:
            errors.append({"subject": row["path"], "error": "InventoryIncomplete"})
    manifest_paths = sorted({p for row in rows for p in row.get("manifestPaths", [])})
    if len(manifest_paths) > MAX_RECORDS:
        errors.append({"subject": "artifact-manifests", "error": "MetadataBudgetExceeded"})
    for name in manifest_paths[:MAX_RECORDS]:
        try:
            value, checksum = read_metadata(Path(name))
            roots = manifest_roots(value)
            if roots:
                commands = [observed_cargo(p.get("command")) for p in value.get("processes", []) if isinstance(p, dict)]
                writers.append({"manifest": name, "manifestSha256": checksum, "roots": roots,
                    "sessionId": value.get("owner", value.get("sessionId")),
                    "requestId": value.get("request_id", value.get("requestId")),
                    "operationId": value.get("operation_id", value.get("operationId")),
                    "sourceRoot": value.get("source_root", value.get("sourceRoot")),
                    "observedRecipes": [r for r in commands if r is not None]})
        except (OSError, ValueError, TypeError) as error:
            errors.append({"subject": name, "error": type(error).__name__})
    directory = repo / ".jenkins/state/management/branch-integration/requests"
    try:
        inventory.physical_literal(directory)
        paths = sorted(directory.glob("*.json"))
        if len(paths) > MAX_RECORDS:
            raise ValueError("request metadata exceeds read budget")
        for path in paths:
            value, checksum = read_metadata(path)
            request = value.get("request", {})
            if not isinstance(request, dict):
                raise ValueError("invalid original request metadata")
            requests.append({"sessionId": request.get("sessionId"), "ownerThreadId": request.get("ownerThreadId"),
                "sourceRoot": request.get("sourceRoot"), "operationId": request.get("operationId"),
                "sourceCommit": value.get("sourceCommit"), "status": value.get("status"),
                "evidence": str(path), "sha256": checksum})
    except (OSError, ValueError, TypeError) as error:
        errors.append({"subject": str(directory), "error": type(error).__name__})
    baseline = dict(previous.get("baseline", {}))
    observations = []
    for identity, unit in sorted(units.items()):
        before = baseline.get(identity)
        candidates = []
        for writer in writers:
            # A parent namespace or cache root is not evidence of a pool writer.
            if not any(root == identity or root.startswith(identity + "/") for root in writer["roots"]):
                continue
            links = [r for r in requests if r["sessionId"] and r["sessionId"] == writer["sessionId"]
                     and (not writer["sourceRoot"] or key(r.get("sourceRoot", "")) == key(writer["sourceRoot"]))]
            candidates.append({**writer, "deliveryReferences": links})
        valid_delta = unit["complete"] and before is not None
        growth = unit["logicalBytes"] - before["logicalBytes"] if valid_delta else None
        observations.append({**unit, "previousValid": before, "logicalDeltaBytes": growth,
            "writerCandidates": candidates, "attribution": "shared-or-uncertain" if len(candidates) != 1 else "manifest-linked-candidate",
            "wasteProvenByGrowth": False, "cleanupDisposition": "preserve; Jenkins ownership and native/reference checks required"})
        if unit["complete"]:
            baseline[identity] = {name: unit[name] for name in ("path", "logicalBytes", "incrementalBytes", "symbolBytes")}
    if scan is None:
        errors.append({"subject": "build-storage", "error": "DeepInventoryRequired"})
    # Do not erase prior units after a partial/fast scan or pretend disappearance
    # proves cleanup. Keep retained baselines until a later complete observation.
    roots = source_roots(repo, errors)
    profiles = profile_observations(roots, errors)
    findings = []
    for unit in observations:
        if unit["logicalDeltaBytes"] is not None and unit["logicalDeltaBytes"] >= GROWTH_REVIEW_BYTES and not unit["writerCandidates"]:
            findings.append({"id": "build_growth_owner_unknown:" + key(unit["path"]), "code": "build_growth_owner_unknown",
                "severity": "review", "subject": unit["path"],
                "message": "At least 1 GiB of observed growth lacks a matching manifest; ownership review required, waste unproven"})
        for writer in unit["writerCandidates"]:
            for recipe in writer["observedRecipes"]:
                violations = recipe_violations(recipe)
                if violations:
                    subject = str(writer.get("operationId") or writer["manifest"])
                    findings.append({"id": "build_scope_violation:" + subject, "code": "build_scope_violation",
                        "severity": "review", "subject": subject,
                        "message": "Recorded Cargo command violates compact-build policy: " + ", ".join(violations)})
    for profile in profiles:
        if profile["violations"]:
            subject = profile["path"] + ":" + profile["profile"]
            findings.append({"id": "build_profile_violation:" + key(subject), "code": "build_profile_violation",
                "severity": "review", "subject": subject,
                "message": "Source configuration needs an owner-scoped Jenkins correction: " + ", ".join(profile["violations"])})
    if errors:
        findings.append({"id": "build_storage_incomplete:inventory", "code": "build_storage_incomplete",
            "severity": "review", "subject": "build-storage", "message": "Storage or attribution observation incomplete; retain prior baselines and unresolved evidence"})
    current = {x["id"]: x for x in findings}
    # Growth notifications stay pending until owner evidence appears; lack of new
    # growth the next day is not a resolution. Partial observations close nothing.
    attributed = {key(unit["path"]) for unit in observations if unit["complete"] and unit["writerCandidates"]}
    observed_profiles = {key(p["path"] + ":" + p["profile"]) for p in profiles}
    for old in previous.get("findings", []):
        if old["code"] == "build_profile_violation" and key(old["subject"]) in observed_profiles:
            # A stable, exact source observation may resolve its own finding even
            # when an unrelated old output directory remains unreadable.
            continue
        if errors or old["code"] == "build_growth_owner_unknown" and key(old["subject"]) not in attributed:
            current.setdefault(old["id"], old)
    return {"schemaVersion": 1, "complete": complete and not errors, "units": observations,
        "baseline": baseline, "profiles": profiles, "errors": errors, "findings": sorted(current.values(), key=lambda x: x["id"]),
        "remediationOwner": "Jenkins; exact original request/worktree/session and reviewed patch required",
        "duplicatePools": "Not inferred from names, timestamps or crate names; full compatibility evidence required",
        "logicalBytesAreReclaimable": False, "automaticCleanup": False, "reclaimedBytes": 0}
