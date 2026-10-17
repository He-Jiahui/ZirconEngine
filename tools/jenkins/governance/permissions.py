"""Generate per-session project profiles; activation needs real sandbox probes."""
from __future__ import annotations

from pathlib import Path
import re
import tomllib

from .paths import build_namespace, digest, file_fact, physical


def plan(authority, payload, report):
    registration = authority.get("repository", payload["repositoryId"])
    session = payload["ownerSessionId"]
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}", session):
        raise ValueError("permission_session_identity_invalid")
    root = physical(registration["canonicalCheckout"])
    config = root / ".codex/config.toml"
    previous = file_fact(config) if config.exists() else None
    if payload.get("expectedConfigHash") != (previous["sha256"] if previous else None):
        raise ValueError("project_permission_config_changed")
    # The profile has no source, Git, Jenkins State or build-pool write grant.
    namespace = build_namespace(registration["buildNamespace"])
    handoff = root / ".codex/outbox/jenkins-inbox" / session
    temporary = namespace / "session-tmp" / session
    profile = "jenkins-session-" + session
    source_probes = [{"path": str(root), "kind": "directory", "scope": "source-root"}]
    original_config = tomllib.loads(config.read_text(encoding="utf-8-sig")) if config.exists() else {}
    old_profile = original_config.get("default_permissions", "")
    def grants(name, seen):
        if not name or name.startswith(":"):
            return {}
        if name in seen:
            raise ValueError("permission_profile_inheritance_cycle")
        settings = original_config.get("permissions", {}).get(name, {})
        return {**grants(settings.get("extends", ""), seen | {name}), **settings.get("filesystem", {})}
    old_grants = grants(old_profile, set())
    for value, access in old_grants.items():
        candidate = Path(value)
        if access != "write" or not candidate.is_absolute() or not candidate.is_relative_to(root):
            continue
        if candidate == root or candidate == root / ".git" or candidate.is_relative_to(root / ".jenkins"):
            continue
        if candidate.exists():
            physical(candidate)
            source_probes.append({"path": str(candidate), "kind": "directory" if candidate.is_dir() else "file",
                                  "scope": "source:" + candidate.relative_to(root).as_posix()})
    text = (f"[permissions.{profile}]\n"
            'description = "Session patches only; Jenkins owns source, Git and compiler outputs."\n'
            'extends = ":read-only"\n\n'
            f"[permissions.{profile}.filesystem]\n"
            '":root" = "read"\n":minimal" = "read"\n":tmpdir" = "read"\n":slash_tmp" = "read"\n'
            f"'{handoff.as_posix()}' = \"write\"\n'{temporary.as_posix()}' = \"write\"\n\n"
            f"'{(authority.root / '.jenkins/state/credentials.json').as_posix()}' = \"deny\"\n"
            f"[permissions.{profile}.network]\nenabled = false\n")
    tomllib.loads(text)
    default_profile = "jenkins-managed-read-only"
    default_text = (f"[permissions.{default_profile}]\n"
            'description = "Read-only default; select the authorized session profile for patch handoffs."\n'
            'extends = ":read-only"\n\n'
            f"[permissions.{default_profile}.filesystem]\n"
            '":root" = "read"\n":minimal" = "read"\n":tmpdir" = "read"\n":slash_tmp" = "read"\n'
            f"'{(authority.root / '.jenkins/state/credentials.json').as_posix()}' = \"deny\"\n\n"
            f"[permissions.{default_profile}.network]\nenabled = false\n")
    expected_default = tomllib.loads(default_text)["permissions"][default_profile]
    existing_default = original_config.get("permissions", {}).get(default_profile)
    if existing_default is not None and existing_default != expected_default:
        raise ValueError("existing_managed_default_profile_requires_review")
    from .worker import _json
    value = {"status": "configuration-prepared", "repositoryId": payload["repositoryId"], "sessionId": session,
             "profile": profile, "config": str(config), "expectedConfig": previous,
             "profileToml": text, "handoffRoot": str(handoff), "temporaryRoot": str(temporary),
             "defaultProfile": default_profile, "defaultProfileToml": default_text,
             "sourceProbeTargets": source_probes,
             "activation": "pending-real-session-and-child-process-probes", "machineEnforcement": False,
             "independentWriters": "MCP, browsers and Apps require separate confinement", **authority.runtime}
    value["planDigest"] = digest(value)
    _json(report / "permission-plan.json", value)
    authority.put("permission_plan", payload["operationId"], value, expected_version=0)
    return value


def apply(authority, payload, report):
    prepared = authority.get("permission_plan", payload["planId"])
    if prepared["planDigest"] != payload["planDigest"]:
        raise ValueError("permission_original_plan_changed")
    config = physical(prepared["config"], exists=False)
    before = file_fact(config) if config.exists() else None
    if before != prepared["expectedConfig"]:
        raise ValueError("permission_config_preimage_changed")
    previous_bytes = config.read_bytes() if config.exists() else b""
    previous = previous_bytes.decode("utf-8-sig")
    parsed = tomllib.loads(previous)
    if "sandbox_mode" in parsed or "sandbox_workspace_write" in parsed:
        raise ValueError("legacy_sandbox_setting_requires_explicit_project_migration")
    if prepared["profile"] in parsed.get("permissions", {}):
        raise ValueError("permission_profile_already_exists")
    default_profile = prepared.get("defaultProfile")
    if not default_profile or default_profile == prepared["profile"]:
        raise ValueError("session_inbox_must_not_be_a_shared_project_default")
    expected_default = tomllib.loads(prepared["defaultProfileToml"])["permissions"][default_profile]
    existing_default = parsed.get("permissions", {}).get(default_profile)
    if existing_default is not None and existing_default != expected_default:
        raise ValueError("managed_default_profile_preimage_changed")
    replacement = re.sub(r'(?m)^default_permissions\s*=\s*[^\n]+',
                         'default_permissions = "' + default_profile + '"', previous, count=1)
    if "default_permissions" not in parsed:
        replacement = 'default_permissions = "' + default_profile + '"\n' + replacement
    replacement = replacement.rstrip() + "\n\n" + prepared["profileToml"]
    if existing_default is None:
        replacement += "\n" + prepared["defaultProfileToml"]
    checked = tomllib.loads(replacement)
    if checked["default_permissions"] != default_profile:
        raise ValueError("permission_default_selection_failed")
    from .source_write import write_verified
    from .versioning import write_git
    root = physical(authority.get("repository", prepared["repositoryId"])["canonicalCheckout"])
    # Retain the exact prior configuration in recoverable Git objects before an authorized config transition.
    preimage = write_git(root, ["hash-object", "-w", "--stdin"], data=previous_bytes).decode().strip()
    tree = write_git(root, ["mktree"], data=("100644 blob " + preimage + "\tconfig.toml\n").encode()).decode().strip()
    commit = write_git(root, ["commit-tree", tree], data=b"Retain exact project permission configuration preimage\n").decode().strip()
    retained_ref = "refs/jenkins/governance/permission/" + payload["operationId"]
    write_git(root, ["update-ref", retained_ref, commit, "0" * len(commit)])
    authority.put("permission_transition", payload["operationId"], {"status": "prepared", "planId": payload["planId"],
        "configPreimageBlob": preimage, "retainedRef": retained_ref, "retainedCommit": commit,
        "expectedConfig": before, **authority.runtime}, expected_version=0)
    from .worker import register_artifacts
    register_artifacts(authority, [{"path": prepared[key], "role": role, "retainUntil": "session handoffs and requests reconciled"}
        for key, role in (("handoffRoot", "session-patch-handoff"), ("temporaryRoot", "session-owned-temporary"))])
    for path in (Path(prepared["handoffRoot"]), Path(prepared["temporaryRoot"])):
        physical(path, exists=False).mkdir(parents=True, exist_ok=True)
    receipt = write_verified(config, before, replacement.encode())
    value = {"status": "configured-awaiting-probes", "planId": payload["planId"], "profile": prepared["profile"],
        "defaultProfile": default_profile,
        "repositoryId": prepared["repositoryId"], "config": receipt, "preimageBlob": preimage,
        "retainedRef": retained_ref, "retainedCommit": commit,
        "activatedSessions": [], "machineEnforcement": False, **authority.runtime}
    authority.put("permission_transition", payload["operationId"], value)
    return value


def probe(authority, payload, report):
    from .worker import _terminal, native
    prepared = authority.get("permission_plan", payload["planId"])
    config = physical(prepared["config"])
    current = file_fact(config)
    if current["sha256"] != payload["expectedConfigHash"]:
        raise ValueError("permission_probe_config_changed")
    expected_profile = tomllib.loads(prepared["profileToml"])["permissions"][prepared["profile"]]
    observed_profile = tomllib.loads(config.read_text(encoding="utf-8-sig")).get("permissions", {}).get(prepared["profile"])
    if observed_profile != expected_profile:
        raise ValueError("permission_probe_profile_differs_from_authorized_plan")
    parsed_config = tomllib.loads(config.read_text(encoding="utf-8-sig"))
    default_profile = prepared.get("defaultProfile")
    if (not default_profile or parsed_config.get("default_permissions") != default_profile
            or parsed_config.get("permissions", {}).get(default_profile)
               != tomllib.loads(prepared["defaultProfileToml"])["permissions"][default_profile]):
        raise ValueError("permission_probe_shared_default_changed")
    codex = physical(payload["codexExecutable"])
    if file_fact(codex)["sha256"] != payload["codexSha256"]:
        raise ValueError("permission_probe_cli_changed")
    registration = authority.get("repository", prepared["repositoryId"])
    root = physical(registration["canonicalCheckout"])
    probe_path = build_namespace(prepared["temporaryRoot"]) / ("probe-" + payload["operationId"])
    from .worker import register_artifacts
    register_artifacts(authority, [{"path": str(probe_path), "role": "permission-probe-evidence", "retainUntil": "probe result reconciled"}])
    probe_path.mkdir()
    from .paths import encoded
    specification = {"profile": prepared["profile"], "canonicalCheckout": str(root), "commonDir": registration["commonDir"],
        "controlRoot": str(authority.root), "buildNamespace": registration["buildNamespace"],
        "sourceTargets": prepared["sourceProbeTargets"],
        "allowedRoot": str(probe_path), "operationId": payload["operationId"]}
    specification["handoffRoot"] = prepared["handoffRoot"]
    canary_directories = {item["path"] for item in prepared["sourceProbeTargets"] if item["kind"] == "directory"}
    canary_directories.update((registration["commonDir"], str(authority.root / ".jenkins/state"), registration["buildNamespace"]))
    canaries = [Path(base) / (".governance-canary-" + payload["operationId"] + "-" + role + "-")
        for base in sorted(canary_directories)
        for role in ("parent", "child")]
    register_artifacts(authority, [{"path": str(path), "role": "expected-denied-permission-canary", "retainUntil": "probe terminal and exact owned residual removed"} for path in canaries])
    positive_paths = []
    for base in (str(probe_path), prepared["handoffRoot"]):
        for role in ("parent", "child"):
            path = Path(base) / (".governance-positive-" + payload["operationId"] + "-" + role + "-")
            positive_paths.extend((path, path.with_suffix(".renamed")))
    register_artifacts(authority, [{"path": str(path), "role": "permission-positive-control", "retainUntil": "probe terminal and exact owned residual removed"} for path in positive_paths])
    source = physical(authority.root / "tools/jenkins/governance/probe_child.py")
    # A fixed probe module, and a base64 data argument; no user-provided program or shell text.
    import base64, sys
    command = [str(codex), "sandbox", "--permission-profile", prepared["profile"], "--",
               sys.executable, "-B", "-X", "utf8", str(source),
               base64.b64encode(encoded(specification)).decode()]
    result = native(authority, command, root, report / "native-permission-probe", timeout=180,
                    environment={"TEMP": str(probe_path), "TMP": str(probe_path), "TMPDIR": str(probe_path)})
    probe_file = probe_path / "probe.json"
    observed = __import__("json").loads(probe_file.read_text(encoding="utf-8")) if probe_file.exists() else {}
    passed = (_terminal(result) and result.get("rootExitCode") == 0 and observed.get("passed") is True
              and file_fact(config) == current)
    # A passing CLI sandbox child is not evidence that this desktop chat selected that profile.
    cleanup = []
    if _terminal(result):
        import hashlib
        from .guarded_delete import remove_verified
        for row in observed.get("parent", []) + observed.get("child", []):
            if row.get("reason") == "write_succeeded" and row.get("residual") in [str(path) for path in canaries]:
                fact = file_fact(Path(row["residual"]))
                if fact["sha256"] == hashlib.sha256(b"permission-probe-canary").hexdigest():
                    cleanup.append(remove_verified(fact))
        for path in positive_paths:
            if path.exists():
                fact = file_fact(physical(path))
                if fact["sha256"] in {hashlib.sha256(value).hexdigest() for value in (b"probe", b"probe-updated")}:
                    cleanup.append(remove_verified(fact))
    value = {"status": "cli-probes-passed" if passed else "needs-review", "profile": prepared["profile"],
        "repositoryId": prepared["repositoryId"], "config": current, "codex": file_fact(codex),
        "probe": observed, "ownedResidualCleanup": cleanup, "desktopSessionActivation": "unverified", "machineEnforcement": False, **authority.runtime}
    authority.put("permission_probe", payload["operationId"], value, expected_version=0)
    return value
