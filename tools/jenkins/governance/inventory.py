"""Read-only whole-volume discovery; candidates never imply deletion authority."""
from __future__ import annotations

from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import time

from .paths import digest, file_fact, linked, physical


SOURCE_MARKERS = {"Cargo.toml", "CMakeLists.txt", "package.json", "pyproject.toml", "pom.xml", "build.gradle"}
OUTPUT_NAMES = {"target", "build", "dist", "obj", "bin", "__pycache__", ".pytest_cache", ".cache", "node_modules", "cmake-build-debug", "cmake-build-release"}
ARTIFACT_SUFFIXES = {".rlib", ".rmeta", ".obj", ".o", ".pdb", ".pch", ".pyc", ".exe", ".dll",
                     ".lib", ".a", ".so", ".dylib", ".class", ".jar", ".war", ".wasm", ".gch", ".ilk"}
SOURCE_SUFFIXES = {".rs", ".py", ".c", ".cc", ".cpp", ".cxx", ".h", ".hpp", ".cs", ".java", ".kt",
                   ".js", ".jsx", ".ts", ".tsx", ".go", ".swift", ".m", ".mm", ".vue", ".sln", ".vcxproj", ".csproj"}
PROTECTED_NAMES = {"credentials.json", "auth.json", "agent.secret", ".env", ".npmrc", ".pypirc",
                   ".git-credentials", ".netrc", "_netrc", "id_rsa", "id_ed25519", "id_ecdsa", "id_dsa",
                   "token", "secret", "known_hosts", "authorized_keys", "hudson.util.secret"}
AUTHENTICATION_DIRECTORIES = {"secrets", "credentials", ".ssh", ".aws", ".azure", ".kube", ".gnupg", ".docker"}


def authentication_path(path: Path) -> bool:
    name = path.name.casefold()
    return (name in PROTECTED_NAMES or name.startswith((".env.", "credentials.", "access-token", "api-token"))
            or any(part.casefold() in AUTHENTICATION_DIRECTORIES for part in path.parts[:-1])
            or name in AUTHENTICATION_DIRECTORIES
            or path.suffix.casefold() in {".pem", ".key", ".pfx", ".p12", ".jks"})


def _git_command(root: Path, *args: str) -> list[str]:
    return ["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=" + os.devnull, "-c", "protocol.allow=never", "-c", "core.fsmonitor=false", "-c", "diff.external=",
            "-c", "core.longpaths=true", "-C", str(root), *args]


def git(root: Path, *args: str, timeout: int = 90) -> bytes:
    # No locks, fsmonitor, hooks, external diff drivers or lazy object fetching.
    environment = {**os.environ, "GIT_OPTIONAL_LOCKS": "0", "GIT_TERMINAL_PROMPT": "0", "GIT_NO_LAZY_FETCH": "1"}
    result = subprocess.run(_git_command(root, *args), env=environment, capture_output=True, timeout=timeout)
    if result.returncode:
        failure = ValueError("git_observation_failed:" + str(result.returncode))
        if result.stderr:
            failure.add_note("Git stderr (last 2048 bytes): " + result.stderr[-2048:].decode("utf-8", "replace").strip())
        raise failure
    return result.stdout


def text_git(root: Path, *args: str) -> str:
    return git(root, *args).decode("utf-8", "surrogateescape").strip()


def resolve_primary(root: Path, registered: str | None = None) -> str | None:
    branches = text_git(root, "for-each-ref", "--format=%(refname:short)", "refs/heads").splitlines()
    if registered:
        return registered if registered in branches else None
    try:
        remote = text_git(root, "symbolic-ref", "refs/remotes/origin/HEAD").removeprefix("refs/remotes/origin/")
        if remote in branches:
            return remote
    except ValueError:
        pass
    candidates = [name for name in ("main", "master") if name in branches]
    return candidates[0] if len(candidates) == 1 else None


STATUS_COMMAND_UNITS = 20000


def _status_observation(root, authentication_exclusions, excluded_paths, *, head_exists):
    """Partition positive scopes before a Windows command can exceed its limit."""
    links = sorted(set(excluded_paths))
    base = ["status", "--porcelain=v1", "-z", "--untracked-files=all", "--ignore-submodules=all", "--no-renames", "--"]
    def arguments(scopes, excluded):
        return (base + [":(top,literal)" + scope if scope else "." for scope in scopes]
                + authentication_exclusions + [":(top,literal,icase,exclude)" + value for value in excluded])
    def fits(scopes, excluded):
        text = subprocess.list2cmdline(_git_command(root, *arguments(scopes, excluded)))
        return len(text.encode("utf-16-le", "surrogatepass")) // 2 <= STATUS_COMMAND_UNITS
    if fits([""], links):
        commands = [arguments([""], links)]
    else:
        # The index contributes additions and missing working files. Only staged
        # deletions need HEAD evidence; do not recursively enumerate its tree.
        tracked = set(git(root, "ls-files", "-z").split(b"\0"))
        if head_exists:
            tracked.update(git(root, "diff", "--cached", "--name-only", "--diff-filter=D", "--no-renames", "-z", "HEAD").split(b"\0"))
        tracked_paths = [value.decode("utf-8", "surrogateescape") for value in tracked if value]
        pending, units = [""], []
        from .walk import walk
        while pending:
            scope = pending.pop()
            descendants = [value for value in links if not scope or value.casefold().startswith(scope.casefold() + "/")]
            if any(scope.casefold() == value.casefold() for value in links):
                continue
            if fits([scope], descendants):
                units.append((scope, descendants))
                continue
            prefix = scope + "/" if scope else ""
            children = {value[len(prefix):].split("/", 1)[0] for value in tracked_paths if value.startswith(prefix)}
            directory = physical(root / scope, exists=False)
            if directory.exists():
                if not directory.is_dir():
                    raise ValueError("bounded_git_status_scope_is_not_directory")
                def failed(path, error):
                    raise error
                iterator = walk([directory], error=failed, link=lambda path: None)
                try:
                    _, entries = next(iterator)
                    for entry in entries:
                        relative = prefix + entry.name
                        information = entry.stat(follow_symlinks=False)
                        if linked(information):
                            if relative not in links:
                                links.append(relative)
                            continue
                        children.add(entry.name)
                finally:
                    iterator.close()
            if not children:
                raise ValueError("bounded_git_status_scope_cannot_be_partitioned")
            for name in sorted(children, reverse=True):
                relative = prefix + name
                if name.casefold() == ".git" or name.casefold() == ".jenkins" or authentication_path(root / relative):
                    continue
                pending.append(relative)
        commands, scopes, exclusions = [], [], []
        for scope, descendants in units:
            if scopes and not fits(scopes + [scope], exclusions + descendants):
                commands.append(arguments(scopes, exclusions))
                scopes, exclusions = [], []
            if not fits([scope], descendants):
                raise ValueError("bounded_git_status_single_scope_exceeds_windows_limit")
            scopes.append(scope)
            exclusions.extend(descendants)
        if scopes:
            commands.append(arguments(scopes, exclusions))
    rows = set()
    for command in commands:
        rows.update(value for value in git(root, *command).split(b"\0") if value)
    # Rename discovery varies with positive scopes. D/A preserves both versions;
    # normalize every observation so a bounded refresh has the same digest.
    return b"".join(value + b"\0" for value in sorted(rows, key=lambda value: (value[3:], value[:2])))


def repository_fact(root: Path, registration: dict | None = None, *, content_hashes=False, excluded_paths=()) -> dict:
    if authentication_path(root):
        return {"path": str(root), "disposition": "needs-review",
                "errors": ["authentication-directory-root-not-inspected"]}
    root = physical(root)
    result = {"path": str(root), "disposition": "needs-review", "errors": []}
    try:
        if (root / ".git").exists():
            physical(root / ".git")
        common = Path(text_git(root, "rev-parse", "--path-format=absolute", "--git-common-dir"))
        common = physical(common)
        is_bare = text_git(root, "rev-parse", "--is-bare-repository") == "true"
        if not is_bare and physical(text_git(root, "rev-parse", "--show-toplevel")) != root:
            raise ValueError("repository_root_is_not_checkout_toplevel")
        refs = text_git(root, "for-each-ref", "--format=%(objectname) %(refname)", "refs/heads")
        primary = resolve_primary(root, (registration or {}).get("primaryBranch"))
        result.update(commonDir=str(common), repositoryId=digest({"commonDir": str(common).casefold()}),
                      bare=is_bare, primaryBranch=primary,
                      branches=[{"head": line.split(" ", 1)[0], "ref": line.split(" ", 1)[1]} for line in refs.splitlines()],
                      worktrees=text_git(root, "worktree", "list", "--porcelain"),
                      primaryHead=text_git(root, "rev-parse", "refs/heads/" + primary) if primary else None)
        result["rootCommits"] = text_git(root, "rev-list", "--max-parents=0", "--all").splitlines()
        try:
            result["head"] = text_git(root, "rev-parse", "HEAD")
        except ValueError:
            result["head"] = None
        if not is_bare:
            # NUL output hashes include index and working changes without exposing contents.
            # Git refresh can itself read tracked files. Exclude authentication
            # paths before invoking it, rather than merely skipping our hashes.
            excluded = ["**/" + name for name in sorted(PROTECTED_NAMES)]
            excluded += ["**/.env.*", "**/credentials.*", "**/access-token*", "**/api-token*"]
            excluded += ["**/*" + suffix for suffix in (".pem", ".key", ".pfx", ".p12", ".jks")]
            # Controller trees contain credential stores under many filenames.
            # Exclude their contents before Git refresh, not after hashing them.
            # A worktree rooted inside .jenkins still observes its own source:
            # these pathspecs are relative to that worktree's Git toplevel.
            excluded += ["**/" + name + "/**" for name in sorted(AUTHENTICATION_DIRECTORIES | {".jenkins"})]
            pathspecs = [":(glob,icase,exclude)" + value for value in excluded]
            status = _status_observation(root, pathspecs, excluded_paths, head_exists=result.get("head") is not None)
            result["authenticationContentInspection"] = "excluded-before-Git-refresh"
            result["excludedLinkedPaths"] = list(excluded_paths)
            result["statusDigest"] = hashlib.sha256(status).hexdigest()
            result["indexDigest"] = hashlib.sha256(git(root, "ls-files", "--stage", "-z")).hexdigest()
            entries, index = status.split(b"\0"), 0
            changes = []
            while index < len(entries) and entries[index]:
                row = entries[index].decode("utf-8", "surrogateescape")
                item = {"status": row[:2], "path": row[3:]}
                index += 1
                if "R" in row[:2] or "C" in row[:2]:
                    item["originalPath"] = entries[index].decode("utf-8", "surrogateescape")
                    index += 1
                changes.append(item)
            result["changes"] = changes
            if content_hashes:
                facts = {}
                for item in changes:
                    for relative in (item["path"], item.get("originalPath")):
                        if not relative or relative in facts:
                            continue
                        path = root / relative
                        if authentication_path(path):
                            facts[relative] = {"status": "needs-review", "reason": "authentication-content-not-read"}
                            continue
                        try:
                            facts[relative] = file_fact(physical(path)) if path.exists() else None
                        except (OSError, ValueError) as error:
                            facts[relative] = {"status": "needs-review", "reason": type(error).__name__}
                result["sourceFacts"] = facts
        result["disposition"] = "discovered" if primary else "needs-review"
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result["errors"].append(type(error).__name__ + ":" + str(error)[:120])
    return result


def scan(roots: list[Path], *, progress=None, registrations: dict | None = None, checkpoint=None) -> dict:
    started = time.time()
    report = {"schemaVersion": 4, "roots": [str(root) for root in roots], "startedAt": started,
              "sourceContentHashes": "deferred-to-fresh-claim-takeover-integration-or-cleanup-admission",
              "repositories": [], "sourceCandidates": [], "outputCandidates": [], "links": [], "errors": [],
              "directories": 0, "files": 0, "logicalBytes": 0}
    coverage = {str(root): {"directories": 0, "files": 0, "logicalBytes": 0, "errors": 0} for root in roots}
    def root_key(path):
        return next((str(root) for root in roots if Path(path).is_relative_to(root)), None)
    def failed(path, error):
        report["errors"].append({"path": str(path), "reason": type(error).__name__,
                                 "winerror": getattr(error, "winerror", None), "errno": getattr(error, "errno", None)})
        key = root_key(path)
        if key in coverage:
            coverage[key]["errors"] += 1
    def skipped(path):
        try:
            target = os.readlink(path)
        except OSError:
            target = None
        report["links"].append({"path": str(path), "target": target, "followed": False})
    repositories, last_progress = set(), started
    from .walk import walk
    for directory, entries in walk(roots, error=failed, link=skipped):
        try:
            report["directories"] += 1
            root_stats = coverage[root_key(directory)]
            root_stats["directories"] += 1
            names = {entry.name for entry in entries}
            git_internal = any(part.casefold() == ".git" for part in directory.parts)
            if not git_internal and (".git" in names or {"HEAD", "objects", "refs"} <= names):
                repositories.add(directory)
            markers = sorted(SOURCE_MARKERS & names)
            source_names = [entry.name for entry in entries if Path(entry.name).suffix.casefold() in SOURCE_SUFFIXES]
            if markers or source_names:
                report["sourceCandidates"].append({"path": str(directory), "markers": markers,
                    "sourceFileCount": len(source_names), "sourceSamples": source_names[:8], "disposition": "needs-review"})
            artifact_names = [entry.name for entry in entries if Path(entry.name).suffix.casefold() in ARTIFACT_SUFFIXES]
            strong = sorted(names & {"CMakeCache.txt", "CMakeFiles", ".fingerprint", ".rustc_info.json", "CACHEDIR.TAG", "project.assets.json"})
            if strong or artifact_names or directory.name.casefold() in OUTPUT_NAMES:
                report["outputCandidates"].append({"path": str(directory), "markers": strong,
                    "artifactSamples": artifact_names[:8], "artifactFileCount": len(artifact_names),
                    "kind": "build-cache-or-compiled-or-installed-files", "disposition": "needs-review"})
            for entry in entries:
                try:
                    info = entry.stat(follow_symlinks=False)
                    if not linked(info) and stat.S_ISREG(info.st_mode):
                        report["files"] += 1
                        report["logicalBytes"] += info.st_size
                        root_stats["files"] += 1
                        root_stats["logicalBytes"] += info.st_size
                except OSError as error:
                    failed(Path(entry.path), error)
        except (OSError, ValueError) as error:
            failed(directory, error)
        if progress and time.time() - last_progress >= 20:
            progress({"directories": report["directories"], "files": report["files"],
                      "currentDirectory": str(directory), "rootCoverage": coverage})
            last_progress = time.time()
    report["rootCoverage"] = coverage
    report["metadataFinishedAt"] = time.time()
    if checkpoint:
        checkpoint("metadata", {**report, "repositoryRoots": sorted(str(root) for root in repositories)})
    if progress:
        progress({"phase": "repository-observations", "roots": report["roots"], "repositoryCount": len(repositories),
                  "directories": report["directories"], "files": report["files"], "rootCoverage": coverage})
    linked_paths = {root: [] for root in repositories}
    for row in report["links"]:
        path = Path(row["path"])
        for parent in path.parents:
            if parent in repositories:
                linked_paths[parent].append(path.relative_to(parent).as_posix())
    for root in sorted(repositories, key=lambda value: str(value).casefold()):
        try:
            fact = repository_fact(root, (registrations or {}).get(str(root).casefold()), content_hashes=False,
                                   excluded_paths=sorted(linked_paths[root]))
        except (OSError, ValueError) as error:
            # A repository can disappear after its directory handle is released.
            # Preserve the completed traversal and report the omitted identity.
            failed(root, error)
            fact = {"path": str(root), "disposition": "needs-review", "errors": [type(error).__name__]}
        report["repositories"].append(fact)
        if checkpoint:
            checkpoint("repository", fact)
        if progress:
            progress({"repository": str(root), "changes": len(fact.get("changes", []))})
    report.update(finishedAt=time.time(), durationSeconds=round(time.time() - started, 2),
                  rootCoverage=coverage, status="complete" if not report["errors"] else "partial",
                  limitations=["Live observation; mutation requires a fresh bounded inventory.",
                               "Source bytes are hashed at bounded claim, takeover, integration or cleanup admission.",
                               "Unrecognised file formats remain outside candidate classification; file metadata is counted.",
                               "Installed binaries, dependency sources and compiler outputs require provenance before classification.",
                               "Authentication paths and linked paths are excluded from source-content inspection.",
                               "Logical bytes include hardlinks and do not measure reclaimable space.",
                               "A name or artifact marker does not authorize deletion."])
    report["inventoryDigest"] = digest(report)
    return report


MANAGED_POLICY = "managed-roots-v1"
MANAGED_ROOTS = [drive + ":/cargo-targets" for drive in "DEF"]
MANAGED_LIMITS = {"entries": 100000, "directories": 10000, "rootSeconds": 120,
                  "nativeSeconds": 600, "scope": "per-root"}


def inventory_roots(policy):
    if policy == "whole-volume-v1":
        return ["C:/", "D:/", "E:/"]
    if policy == MANAGED_POLICY:
        return list(MANAGED_ROOTS)
    raise ValueError("unsupported_inventory_policy")


def scan_managed(roots, index, *, progress=None, budget=None, admissions=None):
    """Index-directed metadata only; protected or unknown roots are never opened."""
    from .walk import WalkBudget, WalkBudgetExceeded, walk
    budget = budget or WalkBudget()
    started = time.time()
    report = {"schemaVersion": 4, "inventoryPolicy": MANAGED_POLICY, "roots": [str(root) for root in roots],
        "startedAt": started, "stateIndex": index, "globalInventoryComplete": False,
        "coverageScope": "index-directed-managed-root-metadata", "limits": MANAGED_LIMITS,
        "sourceContentHashes": "not-read", "repositories": [], "sourceCandidates": [], "outputCandidates": [],
        "protected": [], "links": [], "errors": list(index.get("omissions", [])), "rootCoverage": {},
        "directories": 0, "files": 0, "logicalBytes": 0}
    if index.get("status") != "complete" and not report["errors"]:
        report["errors"].append({"reason": "partial-state-index"})
    for raw_root in roots:
        root = Path(raw_root)
        coverage = {"status": "complete", "directories": 0, "files": 0, "logicalBytes": 0,
                    "pruned": 0, "errors": 0, "physicalRoot": (admissions or {}).get(str(root))}
        report["rootCoverage"][str(root)] = coverage
        budget.begin_root()
        trie, source_roots = {}, set()
        for row in index["records"]:
            for scope in row["paths"]:
                path = Path(scope["path"])
                if not path.is_relative_to(root):
                    continue
                node = trie
                for part in path.relative_to(root).parts:
                    node = node.setdefault(part.casefold(), {})
                node.setdefault("", []).append({**scope, "domain": row["domain"], "key": row["key"],
                                                "version": row["version"]})
        def protected(path, reason, proofs=()):
            coverage["pruned"] += 1
            report["protected"].append({"path": str(path), "reason": reason, "proofs": list(proofs),
                                        "followed": False, "cleanupAuthorized": False})
            return False
        def descend(path):
            if (any(part.casefold() in {".git", ".jenkins", "jenkins_home", "jenkins-home"}
                    for part in path.relative_to(root).parts) or authentication_path(path)):
                return protected(path, "protected-control")
            if any(path.is_relative_to(source) for source in source_roots):
                return protected(path, "protected-source")
            node = trie
            for part in path.relative_to(root).parts:
                node = node.get(part.casefold())
                if node is None:
                    return protected(path, "protected-unknown")
            guarded = [item for item in node.get("", []) if item["role"] != "namespace"]
            if guarded:
                return protected(path, sorted(item["protection"] for item in guarded)[0], guarded)
            if not node:
                return protected(path, "protected-unknown")
            return True
        def failed(path, error):
            coverage["errors"] += 1
            report["errors"].append({"path": str(path), "reason": type(error).__name__})
        try:
            if admissions is not None and not coverage["physicalRoot"].get("verified"):
                raise ValueError("managed-root-physical-identity-unavailable")
            if not root.exists():
                coverage["status"] = "not-present"
                continue
            physical(root)
            for directory, entries in walk([root], error=failed,
                    link=lambda path: report["links"].append({"path": str(path), "followed": False}),
                    budget=budget, descend=descend):
                coverage["directories"] += 1
                names = {entry.name for entry in entries}
                if ".git" in names or names & SOURCE_MARKERS:
                    source_roots.add(directory)
                    report["sourceCandidates"].append({"path": str(directory), "disposition": "protected-source"})
                for entry in entries:
                    if authentication_path(Path(entry.path)):
                        continue
                    info = entry.stat(follow_symlinks=False)
                    if not linked(info) and stat.S_ISREG(info.st_mode):
                        coverage["files"] += 1
                        coverage["logicalBytes"] += info.st_size
                if progress:
                    progress({"phase": "managed-metadata", "root": str(root),
                              "directories": coverage["directories"], "files": coverage["files"]})
        except (OSError, ValueError, WalkBudgetExceeded) as error:
            failed(root, error)
        coverage.update(budget.counts)
        if coverage["errors"]:
            coverage["status"] = "partial"
        for field in ("directories", "files", "logicalBytes"):
            report[field] += coverage[field]
    report.update(finishedAt=time.time(), durationSeconds=round(time.time() - started, 2),
        status="partial" if report["errors"] else "complete",
        limitations=["Bounded metadata coverage; not whole-volume or global inventory.",
                     "Active, historical, source, control and unknown scopes are protected and pruned.",
                     "Logical bytes are not reclaimable bytes; cleanup requires existing independent proofs."])
    report["inventoryDigest"] = digest(report)
    return report
