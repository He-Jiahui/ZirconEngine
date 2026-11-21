"""Pure, single-owner Cargo admission for Agent development (no process or service I/O)."""
from __future__ import annotations

import fnmatch
from pathlib import Path, PurePosixPath
import re
import tomllib


VERBS = {"check", "build", "test", "run", "clippy", "doc", "rustc"}
NAME = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_-]{0,119}\Z")
FEATURE = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_.:/?+\-]{0,119}\Z")
RETIRED_OPTIONS = {"-jenkinsidentityfile", "--jenkins-identity-file", "--identity-file"}


def reject_retired_options(arguments: list[str]) -> None:
    for argument in arguments:
        name = argument.split("=", 1)[0].split(":", 1)[0].casefold()
        if name in RETIRED_OPTIONS:
            raise ValueError("Jenkins identity options are retired; Agent-local execution never reads an identity file")


def plain_path(value: str | Path, *, base: Path | None = None) -> Path:
    path = Path(value)
    if ".." in path.parts or not str(value) or str(value).strip() != str(value):
        raise ValueError("Source/manifest paths must be nonempty canonical paths without parent traversal")
    if not path.is_absolute():
        path = (base or Path.cwd()) / path
    absolute = path.absolute()
    for component in (absolute, *absolute.parents):
        if component.is_symlink() or (component.exists() and getattr(component.lstat(), "st_file_attributes", 0) & 0x400):
            raise ValueError("Source/manifest path aliases are not accepted")
    if absolute.resolve() != absolute:
        raise ValueError("Source/manifest paths must physically resolve to themselves")
    return absolute


def package_owners(repo: Path) -> dict[str, dict[str, str]]:
    """Read only members declared by the two supported Cargo workspaces."""
    result = {}
    for name in ("Cargo.toml", "zircon_plugins/Cargo.toml"):
        workspace = plain_path(repo / name)
        if not workspace.is_file():
            if name == "Cargo.toml":
                raise ValueError("Repository has no Cargo.toml")
            continue
        document = tomllib.loads(workspace.read_text(encoding="utf-8"))
        declared = document.get("workspace", {})
        members = list(declared.get("members", []))
        if "package" in document:
            members.append(".")
        excluded = set(declared.get("exclude", []))
        for member in members:
            portable = PurePosixPath(member)
            if not member or portable.is_absolute() or ".." in portable.parts or "\\" in member:
                raise ValueError("Workspace has a noncanonical declared member")
            if member in excluded:
                continue
            directories = [workspace.parent] if member == "." else sorted(workspace.parent.glob(member))
            if not directories:
                raise ValueError("A declared workspace member is missing: " + member)
            for directory in directories:
                relative = directory.relative_to(workspace.parent).as_posix()
                if any(fnmatch.fnmatchcase(relative, pattern) for pattern in excluded):
                    continue
                manifest = plain_path(directory / "Cargo.toml")
                if not manifest.is_relative_to(repo) or not manifest.is_file():
                    raise ValueError("Workspace member has no canonical repository-owned manifest")
                data = tomllib.loads(manifest.read_text(encoding="utf-8"))
                package = data.get("package", {}).get("name")
                if not isinstance(package, str) or not NAME.fullmatch(package):
                    raise ValueError("Workspace member has no exact Cargo package name")
                owner = {"package": package, "packageManifest": str(manifest), "workspaceManifest": str(workspace)}
                if package in result and result[package] != owner:
                    raise ValueError("Ambiguous Cargo package owner: " + package)
                result[package] = owner
    return result


def admit_cargo(arguments: list[str], repo: Path) -> dict:
    """Validate every compiler option; return argv bound to one real manifest owner.

    Workspace/package-set runs need a separate explicitly authorized entry. This
    ordinary entry deliberately cannot grant them through raw Cargo flags.
    """
    reject_retired_options(arguments)
    args = list(arguments)
    if not args or args[0] not in VERBS:
        raise ValueError("Select an explicit Cargo verb without a rustup toolchain override")
    verb = args[0]
    separator = args.index("--") if "--" in args else len(args)
    before, harness = args[1:separator], args[separator:]
    normalized, values, flags, filters = [verb], {}, set(), []
    switches = {"--no-default-features", "--locked", "--offline", "--release", "--lib", "--no-run", "--no-deps"}
    valued = {"-p": "package", "--package": "package", "--manifest-path": "manifest", "--features": "features", "-F": "features",
              "--profile": "profile", "-j": "jobs", "--jobs": "jobs", "--test": "test", "--bin": "bin",
              "--example": "example", "--bench": "bench", "--target": "target", "--message-format": "messageFormat", "--crate-type": "crateType"}
    index = 0
    while index < len(before):
        token = before[index]
        index += 1
        if not isinstance(token, str) or not token or any(c in token for c in "\0\r\n"):
            raise ValueError("Invalid literal Cargo argument")
        if len(token) > 2 and token[:2] in {"-p", "-j", "-F"} and not token.startswith("--"):
            token = token[:2] + "=" + token[2:]
        key, equal, inline = token.partition("=")
        if token in switches:
            if token in flags:
                raise ValueError("Repeated Cargo flag: " + token)
            flags.add(token)
            normalized.append(token)
        elif key in valued:
            field = valued[key]
            if not equal:
                if index >= len(before):
                    raise ValueError("Cargo option requires a value: " + key)
                inline = before[index]
                index += 1
            if not inline or inline.startswith("-") or any(c in inline for c in "\0\r\n"):
                raise ValueError("Cargo option requires a nonempty literal value: " + key)
            if field in values:
                raise ValueError("Select one exact package/target; repeated option: " + key)
            values[field] = inline
            if field == "messageFormat":
                normalized.append("--message-format=" + inline)
            elif field != "manifest":
                normalized += [key, inline]
        elif not token.startswith("-") and verb in {"test", "bench"} and not filters:
            if len(token.encode("utf-8")) > 256:
                raise ValueError("Test filter is too long")
            filters.append(token)
            normalized.append(token)
        else:
            raise ValueError("Unsupported Cargo option; workspace/all/output/config overrides require a separately authorized entry: " + key)
    package = values.get("package", "")
    if not NAME.fullmatch(package):
        raise ValueError("Select one exact Cargo package with -p/--package; patterns and implicit root selection are forbidden")
    if "--no-default-features" not in flags:
        raise ValueError("Ordinary Agent builds require --no-default-features and explicit needed features")
    features = re.split(r"[,\s]+", values["features"]) if "features" in values else []
    if any(not FEATURE.fullmatch(feature) for feature in features) or len(set(features)) != len(features):
        raise ValueError("Select explicit unique Cargo feature names")
    profile = values.get("profile", "release" if "--release" in flags else "test" if verb == "test" else "dev")
    if profile not in {"dev", "test", "release", "profiling", "shipping", "shipping-symbols"}:
        raise ValueError("Unsupported explicit Cargo profile")
    if "profile" in values and "--release" in flags:
        raise ValueError("--profile conflicts with --release")
    if values.get("jobs", "1") not in {str(i) for i in range(1, 9)}:
        raise ValueError("Select one to eight Cargo jobs")
    if "jobs" not in values:
        normalized += ["--jobs", "1"]
    targets = [name for name in ("test", "bin", "example", "bench") if name in values]
    if len(targets) + ("--lib" in flags) > 1:
        raise ValueError("Select exactly one compilation target")
    for field in targets:
        if not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]{0,119}", values[field]):
            raise ValueError("Select an exact Cargo target name")
    if verb == "test" and not ("--lib" in flags or targets == ["test"]):
        raise ValueError("Cargo tests require --lib or one exact --test target")
    if "--no-run" in flags and (verb not in {"test", "bench"} or harness):
        raise ValueError("--no-run requires test/bench without harness options")
    if "test" in values and verb != "test":
        raise ValueError("--test is only supported with Cargo test")
    if "crateType" in values and (verb != "rustc" or values["crateType"] not in {"cdylib", "rlib", "staticlib"}):
        raise ValueError("Only explicit rustc cdylib/rlib/staticlib crate types are supported")
    if "target" in values and not re.fullmatch(r"[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+){1,5}", values["target"]):
        raise ValueError("Cargo --target must be an explicit target triple, not a path")
    if values.get("messageFormat", "human") not in {"human", "short", "json", "json-render-diagnostics", "json-diagnostic-rendered-ansi"}:
        raise ValueError("Unsupported Cargo message format")
    if harness:
        if verb == "rustc":
            raise ValueError("Raw rustc output flags are not admitted")
        if any(not value or any(c in value for c in "\0\r\n") for value in harness[1:]):
            raise ValueError("Invalid literal harness argument")
        if verb in {"test", "bench"}:
            allowed = {"--exact", "--ignored", "--nocapture", "--show-output"}
            position = 1
            while position < len(harness):
                option = harness[position]
                position += 1
                key, equal, value = option.partition("=")
                if option in allowed:
                    continue
                if key != "--test-threads":
                    raise ValueError("Unsupported bounded test harness option: " + key)
                if not equal:
                    if position >= len(harness):
                        raise ValueError("--test-threads requires a value")
                    value = harness[position]
                    position += 1
                if value not in {str(i) for i in range(1, 9)}:
                    raise ValueError("Select one to eight test threads")
        elif verb == "clippy":
            if harness != ["--", "-D", "warnings"]:
                raise ValueError("Only -- -D warnings is supported for Clippy")
        elif verb != "run":
            raise ValueError("This Cargo action does not accept trailing arguments")
    owners = package_owners(repo)
    if package not in owners:
        raise ValueError("No declared exact package owner: " + package)
    owner = owners[package]
    manifest = plain_path(values["manifest"], base=repo) if "manifest" in values else Path(owner["workspaceManifest"])
    if str(manifest) not in {owner["workspaceManifest"], owner["packageManifest"]}:
        raise ValueError("Explicit manifest does not own the selected exact package")
    normalized += ["--manifest-path", str(manifest)]
    normalized += harness
    return {**owner, "manifestPath": str(manifest), "verb": verb, "features": features, "profile": profile,
            "noDefaultFeatures": True, "jobs": int(values.get("jobs", 1)), "cargoArguments": normalized,
            "testFilter": filters[0] if filters else None, "selectionScope": "single-exact-package"}
