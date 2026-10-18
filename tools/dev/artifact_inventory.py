"""Read-only Codex directory inventory; Jenkins owns report files and cleanup."""
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time
import tomllib


REPARSE = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
OUTPUT_NAMES = {"target", "node_modules", "__pycache__", ".pytest_cache", "dist"}
SOURCE_EXTENSIONS = {".rs", ".py", ".ts", ".tsx", ".js", ".cpp", ".h", ".wgsl"}


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def linked(info: os.stat_result) -> bool:
    return stat.S_ISLNK(info.st_mode) or bool(getattr(info, "st_file_attributes", 0) & REPARSE)


def physical_literal(path: Path) -> Path:
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError(f"Expected an absolute literal path: {path}")
    for component in (path, *path.parents):
        try:
            if linked(component.lstat()):
                raise ValueError(f"Reparse point or symlink: {component}")
        except FileNotFoundError:
            continue
    if os.path.normcase(str(path.resolve())) != os.path.normcase(str(path)):
        raise ValueError(f"Path resolves through an alias: {path}")
    return path


def load_grants(repo: Path) -> tuple[dict, dict[str, str], str]:
    raw = (repo / ".codex/config.toml").read_bytes()
    config = tomllib.loads(raw.decode("utf-8-sig"))
    name = config.get("default_permissions", "zircon-strict")
    visited = set()
    def resolve(profile):
        if profile.startswith(":"):
            return {}
        if profile in visited:
            raise ValueError("Permission profile inheritance cycle")
        visited.add(profile)
        value = config["permissions"][profile]
        inherited = resolve(value["extends"]) if value.get("extends") else {}
        return {**inherited, **value.get("filesystem", {})}
    # Full Access has no filesystem table. Retain the project's explicit audit
    # scope without changing permissions or pretending this is sandbox enforcement.
    scope = "zircon-strict" if name == ":danger-full-access" else name
    grants = resolve(scope)
    return config, grants, hashlib.sha256(raw).hexdigest()


def compiler_unit(path: Path) -> Path | None:
    parts = path.parts
    if len(parts) < 4 or parts[0].rstrip("\\/").casefold() not in {"d:", "e:", "f:"} or parts[1].casefold() != "cargo-targets":
        return None
    folded = [part.casefold() for part in parts]
    if "pools" in folded:
        index = folded.index("pools")
        if len(parts) > index + 3 and folded[index + 2] in {"target", "build"}:
            return Path(*parts[:index + 2])
    for index in range(2, len(parts) - 1):
        if folded[index] in {"target", "build", "sccache"}:
            return Path(*parts[:index + 1])
    return None


def writable(path: Path, grants: dict[str, str]) -> bool:
    matches = []
    for value, mode in grants.items():
        if value.startswith(":"):
            continue
        root = Path(value)
        if path == root or path.is_relative_to(root):
            matches.append((len(root.parts), mode))
    return bool(matches) and max(matches, key=lambda item: item[0])[1] == "write"


def classify(path: Path, repo: Path) -> str:
    if path.is_relative_to(repo):
        relative = path.relative_to(repo).parts
        if relative[0] in {".git", ".jenkins"}:
            return "protected-control"
        if relative[:2] == (".codex", "outbox"):
            return "handoff-evidence"
        if relative[0] == ".codex" and "state" in relative:
            return "state-evidence"
        if relative[:2] == (".codex", "sessions"):
            return "session-evidence"
        if "skills" in relative:
            return "skill-source"
        return "canonical-source-or-docs"
    lower = set(part.casefold() for part in path.parts)
    if "zircon-jenkins" in lower:
        return "protected-jenkins"
    if lower & {"cache", "codex-cache", "win32", "windows", "dependency-fix"}:
        return "build-or-shared-cache"
    if "codex-permissions" in lower or "evidence" in lower:
        return "retained-evidence"
    if lower & {"codex-data", "codex-state", "codex-temporary"}:
        return "shared-tool-state-or-temp"
    return "legacy-task-output-unclaimed"


def tree_fact(path: Path, *, inspect_source: bool = False, deadline: float | None = None) -> dict:
    """Count logical bytes without following links; report incomplete observations."""
    fact = {"path": str(path), "files": 0, "directories": 0, "logicalBytes": 0,
            "hardlinkedFiles": 0, "hardlinkCountComplete": True, "reparsePoints": [], "errors": [],
            "sourceFiles": 0, "sourceSamples": [], "nestedOutputs": [],
            "sourceTreeMarkers": [], "manifestPaths": [], "buildUnits": {}, "latestWriteUtc": None}
    latest = 0.0
    stack = [(path, None)]
    while stack:
        if deadline is not None and time.monotonic() >= deadline:
            fact["errors"].append({"path": str(path), "type": "ScanBudgetExceeded"})
            fact["scanTimedOut"] = True
            fact["unvisitedEntriesLowerBound"] = len(stack)
            break
        current, cached_info = stack.pop()
        try:
            # Windows directory enumeration already carries file size and time.
            # Refresh directories before descending so a replaced junction is
            # never traversed using stale cached directory attributes.
            info = current.lstat() if cached_info is None or stat.S_ISDIR(cached_info.st_mode) else cached_info
            if linked(info):
                fact["reparsePoints"].append(str(current))
                continue
            latest = max(latest, info.st_mtime)
            if stat.S_ISDIR(info.st_mode):
                fact["directories"] += 1
                if current != path and current.name in OUTPUT_NAMES:
                    fact["nestedOutputs"].append(str(current))
                if inspect_source and current.name.startswith("zircon_"):
                    source_directory = current / "src"
                    try:
                        source_info = source_directory.lstat()
                        if not linked(source_info) and stat.S_ISDIR(source_info.st_mode):
                            fact["sourceTreeMarkers"].append(str(current))
                    except (FileNotFoundError, NotADirectoryError):
                        pass
                    except OSError as error:
                        fact["errors"].append({"path": str(source_directory), "type": type(error).__name__})
                with os.scandir(current) as entries:
                    for entry in entries:
                        try:
                            stack.append((Path(entry.path), entry.stat(follow_symlinks=False)))
                        except OSError as error:
                            fact["errors"].append({"path": entry.path, "type": type(error).__name__})
            elif stat.S_ISREG(info.st_mode):
                fact["files"] += 1
                fact["logicalBytes"] += info.st_size
                fact["hardlinkedFiles"] += info.st_nlink > 1
                # DirEntry.stat on Windows does not supply link counts. Keep
                # this evidence gap explicit; logical bytes never authorize GC.
                if os.name == "nt" and cached_info is not None:
                    fact["hardlinkCountComplete"] = False
                unit = compiler_unit(current)
                if unit is not None:
                    usage = fact["buildUnits"].setdefault(str(unit), {"logicalBytes": 0, "files": 0,
                        "incrementalBytes": 0, "symbolBytes": 0})
                    usage["logicalBytes"] += info.st_size
                    usage["files"] += 1
                    if "incremental" in [part.casefold() for part in current.relative_to(unit).parts]:
                        usage["incrementalBytes"] += info.st_size
                    elif current.suffix.casefold() in {".pdb", ".dwp", ".dwo", ".dsym"}:
                        usage["symbolBytes"] += info.st_size
                if inspect_source and current.suffix.lower() in SOURCE_EXTENSIONS:
                    fact["sourceFiles"] += 1
                    if len(fact["sourceSamples"]) < 8:
                        fact["sourceSamples"].append(str(current))
                if current.name in {"artifact-manifest.json", "artifacts.json"}:
                    fact["manifestPaths"].append(str(current))
                if inspect_source and current.name == "Cargo.toml":
                    fact["sourceTreeMarkers"].append(str(current))
        except OSError as error:
            fact["errors"].append({"path": str(current), "type": type(error).__name__})
    if latest:
        fact["latestWriteUtc"] = datetime.fromtimestamp(latest, timezone.utc).isoformat()
    fact["complete"] = not fact["errors"] and not fact["reparsePoints"]
    return fact


def process_observation() -> dict:
    """Use command lines only for matching; never emit their potentially secret bytes."""
    command = ("$ErrorActionPreference='Stop'; @(Get-CimInstance Win32_Process | "
               "Select-Object ProcessId,ParentProcessId,Name,CreationDate,CommandLine,ExecutablePath) "
               "| ConvertTo-Json -Compress")
    try:
        result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
                                capture_output=True, encoding="utf-8", errors="replace", timeout=45, check=True)
        rows = json.loads(result.stdout.strip().lstrip("\ufeff"))
        if not isinstance(rows, list):
            rows = [rows]
        return {"complete": True, "rows": rows, "observedAtUtc": utc_now()}
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        return {"complete": False, "rows": [], "error": type(error).__name__, "observedAtUtc": utc_now()}


def process_matches(path: Path, rows: list[dict]) -> list[dict]:
    needle = str(path).replace("\\", "/").casefold().rstrip("/")
    matched = []
    for row in rows:
        arguments = " ".join(str(row.get(name) or "") for name in ("CommandLine", "ExecutablePath"))
        if needle in arguments.replace("\\", "/").casefold():
            matched.append({key: row.get(key) for key in ("ProcessId", "Name", "CreationDate")})
    return matched


def scan(repo: Path, scopes: list[Path] | None = None, *, max_seconds: float = 900) -> dict:
    start = time.monotonic()
    if not 0 < max_seconds <= 3600:
        raise ValueError("Inventory budget must be between zero and 3600 seconds")
    deadline = start + max_seconds
    config, grants, config_hash = load_grants(repo)
    observation = process_observation()
    rows = []
    declared_roots = sorted((Path(value) for value, mode in grants.items() if mode == "write" and not value.startswith(":")), key=str)
    if scopes:
        for scope in scopes:
            if not any(scope == root or scope.is_relative_to(root) for root in declared_roots):
                raise ValueError(f"Scope is not a declared write directory or descendant: {scope}")
        roots = sorted(set(scopes), key=str)
    else:
        roots = declared_roots
    # Compiler growth is the daily decision input. A large Jenkins worktree
    # archive must not consume the entire budget before those roots are observed.
    roots = sorted(roots, key=lambda root: ("cargo-targets" not in [p.casefold() for p in root.parts], str(root)))
    # Avoid double counting narrower grants already covered by a directory grant.
    for root in roots:
        if any(root != other and root.is_relative_to(other) and other.is_dir() for other in roots):
            continue
        row = {"path": str(root), "category": classify(root, repo), "exists": root.exists(),
               "cleanupDisposition": "preserve"}
        if root.exists():
            try:
                physical_literal(root)
                print(f"Inventorying {root}", file=sys.stderr, flush=True)
                row.update(tree_fact(root, inspect_source=row["category"] in {"handoff-evidence", "session-evidence", "state-evidence"}, deadline=deadline))
                row["kind"] = "directory" if root.is_dir() else "file"
            except (OSError, ValueError) as error:
                row["scanError"] = str(error)
        rows.append(row)
        print(f"Inventoried {root}", file=sys.stderr, flush=True)
    output_items = []
    namespaces = [Path(f"{drive}:/cargo-targets/zircon-local") for drive in "DEF"]
    namespaces.extend(repo / f".codex/{name}" for name in ("outbox", "sessions"))
    for namespace in namespaces:
        if scopes and not any(namespace == scope or namespace.is_relative_to(scope) or scope.is_relative_to(namespace) for scope in scopes):
            continue
        if not namespace.exists():
            continue
        try:
            physical_literal(namespace)
            children = sorted(namespace.iterdir(), key=str)
        except (OSError, ValueError) as error:
            output_items.append({"path": str(namespace), "scanError": str(error)})
            continue
        for child in children:
            if scopes and not any(child == scope or child.is_relative_to(scope) or scope.is_relative_to(child) for scope in scopes):
                continue
            print(f"Classifying {child}", file=sys.stderr, flush=True)
            row = tree_fact(child, inspect_source=True, deadline=deadline)
            row["category"] = classify(child, repo)
            row["processReferences"] = process_matches(child, observation["rows"])
            row["cleanupDisposition"] = "preserve-owner-review-required"
            output_items.append(row)
            print(f"Classified {child}", file=sys.stderr, flush=True)
    disks = []
    for drive in "DEF":
        try:
            usage = shutil.disk_usage(f"{drive}:/")
            disks.append({"drive": drive, "freeBytes": usage.free, "totalBytes": usage.total,
                          "belowBuildReserve": usage.free < 35 * 1024**3})
        except OSError as error:
            disks.append({"drive": drive, "error": type(error).__name__})
    facts = [row for row in rows if row.get("exists")]
    return {"schemaVersion": 1, "observedAtUtc": utc_now(), "repository": str(repo),
            "complete": all(row.get("complete") is True for row in facts + output_items),
            "scanBudgetSeconds": max_seconds, "timedOut": any(row.get("scanTimedOut") for row in facts + output_items),
            "configSha256": config_hash, "profile": config.get("default_permissions", "zircon-strict"),
            "inventoryScopeSource": ("permissions.zircon-strict.filesystem (audit scope only)"
                if config.get("default_permissions") == ":danger-full-access" else "selected permission profile"),
            "declaredWriteGrants": [value for value, mode in grants.items() if mode == "write"],
            "declaredWriteGrantCount": len(declared_roots), "scopes": [str(scope) for scope in scopes or []],
            "inventory": rows, "outputItems": output_items,
            "diskObservation": disks, "processObservationComplete": observation["complete"],
            "processObservedAtUtc": observation["observedAtUtc"],
            "summary": {"existingInventoryRoots": len(facts), "files": sum(row.get("files", 0) for row in facts),
                        "logicalBytes": sum(row.get("logicalBytes", 0) for row in facts),
                        "outputItemCount": len(output_items),
                        "categories": dict(Counter(row["category"] for row in rows)),
                        "durationSeconds": round(time.monotonic() - start, 2)},
            "configuredControlWriteGrants": [value for value, mode in grants.items()
                                              if mode == "write" and Path(value) in {repo / ".git", repo / ".jenkins"}],
            "configuredNetworkEnabled": config.get("permissions", {}).get(config.get("default_permissions", "zircon-strict"), {}).get("network", {}).get("enabled"),
            "limitations": ["Live filesystem observation, not an atomic snapshot.",
                            "Logical bytes are not uniquely allocated or reclaimable bytes.",
                            "Windows enumeration does not inspect per-file hardlink counts; see hardlinkCountComplete.",
                            "Process command-line references cannot prove absence of users or open handles.",
                            "Missing manifests do not prove that a directory is abandoned.",
                            "Source markers identify review candidates, not ownership or deletion authorization."]}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--scope", type=Path, action="append", help="Absolute declared directory or descendant; repeat to narrow the audit")
    parser.add_argument("--host-jenkins", action="store_true", help="Submit whole C/D/E discovery to the existing bounded Jenkins governance job")
    parser.add_argument("--operation-id", help="Stable original operation identity for the Jenkins volume scan")
    args = parser.parse_args(argv)
    try:
        repo = physical_literal(args.repo)
        if args.host_jenkins:
            if args.scope or not args.operation_id:
                raise ValueError("Whole-volume Jenkins inventory requires --operation-id and no local --scope")
            return subprocess.run([sys.executable, "-B", "-m", "tools.dev.jenkins_governance", "--repo-root", str(repo),
                "submit", "--action", "scan", "--operation-id", args.operation_id], cwd=repo, check=False).returncode
        scopes = [physical_literal(scope) for scope in args.scope or []]
        report = scan(repo, scopes=scopes or None)
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0
    except (OSError, ValueError, KeyError) as error:
        parser.exit(2, f"Inventory failed: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
