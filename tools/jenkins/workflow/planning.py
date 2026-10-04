"""Trusted recipe planning and registration.

The planner is the only place where a Cargo recipe is assembled.  Callers
provide a sealed input reference and a bounded change selection; commands,
toolchain, wrapper and environment identity are derived here and persisted in
the State authority.  A recipe reference is therefore an auditable State key,
not a caller supplied command blob.
"""
from __future__ import annotations

import os
import re
import shutil
import subprocess
import ntpath
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Iterable

from ..contracts import JenkinsError, digest, file_digest, require_digest, identifier
from ..resources.paths import canonical_build_root, build_root_containing, physical_path_under
from .cargo_adapter import parse_metadata, build_adapter_payload
from .coverage import discover_selected_test_count


_TOOLCHAIN_RE = re.compile(r"^[A-Za-z0-9_.:+\-/ ]{1,160}$")
_CARGO_PHASES = ("compile", "list", "test")


def _plain(value: Any) -> Any:
    if isinstance(value, Mapping):
        return {str(k): _plain(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [_plain(v) for v in value]
    return value


def _digest_file(path: Path) -> str | None:
    return file_digest(path) if path.is_file() else None


def _toolchain(repo: Path) -> tuple[str, str | None]:
    for name in ("rust-toolchain.toml", "rust-toolchain"):
        path = repo / name
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="strict").strip()
        if name.endswith(".toml"):
            match = re.search(r"(?:^|\n)\s*channel\s*=\s*[\"']([^\"']+)", text)
            value = match.group(1) if match else "stable"
        else:
            value = text.splitlines()[0].strip() if text else "stable"
        if not _TOOLCHAIN_RE.fullmatch(value):
            raise JenkinsError("toolchain_invalid", "rust toolchain declaration is invalid")
        return value, file_digest(path)
    return "stable", None


def _cargo_path(value: str | Path | None) -> Path:
    # The caller may select a tool binary only by an absolute path.  A missing
    # value is resolved by the controlled host environment, never persisted as
    # a shell command supplied by the request.
    if value is None:
        found = shutil.which("cargo")
        if not found:
            raise JenkinsError("cargo_unavailable", "Cargo executable is not available")
        path = Path(found)
    else:
        path = Path(value)
        if not path.is_absolute():
            raise JenkinsError("cargo_path_invalid", "cargoPath must be absolute")
    if not path.is_file() or path.name.casefold() not in {"cargo", "cargo.exe"}:
        raise JenkinsError("cargo_path_invalid", "cargoPath must identify a Cargo executable")
    return path.resolve()


def _environment_key(value: str) -> str:
    # Windows treats PATH case-insensitively.  The policy file historically
    # used both ``Path`` and ``PATH``; canonicalising it here keeps the recipe
    # identity stable without allowing the caller's process environment in.
    return "PATH" if value.casefold() == "path" else value


def _environment_policy(state, repo_root: Path) -> tuple[dict[str, str], dict[str, Any], Path]:
    record = state.get("environment_policy", "default")
    if not record or not isinstance(record.get("payload"), Mapping):
        raise JenkinsError("environment_policy_missing", "trusted build environment policy is unavailable")
    policy = _plain(record["payload"])
    if policy.get("status") != "verified" or policy.get("schemaVersion") != 1:
        raise JenkinsError("environment_policy_unverified", "trusted build environment policy is not verified")
    file_value = policy.get("environmentFile") or policy.get("file")
    if not isinstance(file_value, str) or not file_value:
        raise JenkinsError("environment_policy_file_missing", "trusted environment file is missing")
    env_file = Path(file_value)
    if not env_file.is_absolute():
        env_file = repo_root / env_file
    env_file = env_file.absolute()
    if not env_file.is_file():
        raise JenkinsError("environment_policy_file_missing", "trusted environment file is unavailable")
    expected_file = policy.get("fileDigest") or policy.get("sha256")
    if not isinstance(expected_file, str) or file_digest(env_file) != expected_file:
        raise JenkinsError("environment_policy_file_changed", "trusted environment file digest differs")
    try:
        import json
        file_values = json.loads(env_file.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError) as exc:
        raise JenkinsError("environment_policy_file_invalid", "trusted environment file is not valid JSON") from exc
    if not isinstance(file_values, Mapping) or not file_values:
        raise JenkinsError("environment_policy_file_invalid", "trusted environment file has no variables")
    declared = policy.get("environment")
    if not isinstance(declared, Mapping) or not declared:
        raise JenkinsError("environment_policy_mismatch", "State environment binding is missing")
    canonical_declared = {_environment_key(str(k)): str(v) for k, v in declared.items()}
    canonical_file = {_environment_key(str(k)): str(v) for k, v in file_values.items()}
    if any(canonical_declared.get(key) != value for key, value in canonical_file.items()):
        raise JenkinsError("environment_policy_mismatch", "State environment binding differs from its sealed file")
    raw_digest = digest(dict(sorted(canonical_declared.items())))
    if policy.get("environmentDigest") and policy["environmentDigest"] != raw_digest:
        raise JenkinsError("environment_policy_digest_mismatch", "trusted environment digest differs")
    values = canonical_declared
    # These are execution controls, rather than ambient caller settings.  A
    # missing wrapper is represented explicitly so inherited wrappers cannot
    # silently alter the compiler identity.
    values.setdefault("RUSTC_WRAPPER", "")
    values.setdefault("RUSTC_WORKSPACE_WRAPPER", "")
    values.setdefault("CARGO_NET_OFFLINE", "true")
    values.setdefault("CARGO_INCREMENTAL", "1")
    for required in ("INCLUDE", "LIB", "LIBPATH", "PATH"):
        if not values.get(required):
            raise JenkinsError("environment_policy_incomplete", f"trusted environment is missing {required}")
    return dict(sorted(values.items())), policy, env_file


def _cargo_evidence(state, source_ref: str) -> dict[str, Any]:
    record = state.get("sealed_cargo_evidence", source_ref)
    if not record or not isinstance(record.get("payload"), Mapping):
        raise JenkinsError("cargo_evidence_missing", "authoritative sealed Cargo evidence is unavailable")
    evidence = _plain(record["payload"])
    if evidence.get("sourceDigest") != source_ref or evidence.get("manifestDigest", source_ref) != source_ref:
        raise JenkinsError("cargo_evidence_source_mismatch", "Cargo evidence is not bound to the sealed source")
    metadata = evidence.get("cargoMetadata") or evidence.get("metadata")
    if not isinstance(metadata, Mapping):
        raise JenkinsError("cargo_metadata_missing", "authoritative Cargo evidence has no metadata")
    if not isinstance(evidence.get("toolchainDigest"), str) or not isinstance(evidence.get("lockDigest"), str):
        raise JenkinsError("cargo_toolchain_evidence_missing", "authoritative Cargo toolchain or lock evidence is missing")
    workspace = evidence.get("materializedRoot") or evidence.get("workspaceRoot") or evidence.get("workspace_root") or evidence.get("runRoot")
    if not isinstance(workspace, str) or not workspace:
        raise JenkinsError("cargo_workspace_missing", "Cargo evidence has no sealed workspace root")
    workspace_canonical = workspace.replace("/", "\\")
    build_root_containing(workspace_canonical)
    evidence["cargoMetadata"] = _normalize_metadata(metadata, workspace)
    evidence["workspaceRoot"] = workspace_canonical
    return evidence


def _normalize_metadata(metadata: Mapping[str, Any], workspace_root: str) -> dict[str, Any]:
    """Make sealed-run absolute Cargo paths repository/materialization relative.

    Cargo metadata is collected from a materialized workspace under an approved
    drive root.  Persisting those absolute paths in a recipe would bind it to a
    particular run directory and could accidentally point execution at the live
    checkout.  Only known Cargo path fields are rewritten; all other metadata
    remains byte-for-byte representable for diagnostics.
    """
    root = str(workspace_root).replace("\\", "/").rstrip("/").casefold()

    def relative(value: Any) -> Any:
        if not isinstance(value, str):
            return value
        normalized = value.replace("\\", "/")
        lowered = normalized.casefold()
        prefix = root + "/"
        if lowered == root:
            return "."
        if lowered.startswith(prefix):
            return normalized[len(prefix):]
        if ntpath.isabs(normalized) or (len(normalized) >= 2 and normalized[1] == ":"):
            raise JenkinsError("cargo_metadata_path_outside_workspace",
                               "Cargo metadata path is outside the sealed workspace")
        if any(part in {"", ".", ".."} for part in normalized.split("/")):
            raise JenkinsError("cargo_metadata_path_invalid", "Cargo metadata path is not canonical")
        return normalized

    copied = dict(metadata)
    packages = []
    for package in metadata.get("packages", ()):
        if not isinstance(package, Mapping):
            packages.append(package)
            continue
        item = dict(package)
        for field in ("manifest_path", "root"):
            if field in item:
                item[field] = relative(item[field])
        targets = []
        for target in item.get("targets", ()):
            if isinstance(target, Mapping):
                target_copy = dict(target)
                if "src_path" in target_copy:
                    target_copy["src_path"] = relative(target_copy["src_path"])
                targets.append(target_copy)
            else:
                targets.append(target)
        if "targets" in item:
            item["targets"] = targets
        packages.append(item)
    if "packages" in copied:
        copied["packages"] = packages
    if "workspace_root" in copied:
        copied["workspace_root"] = relative(copied["workspace_root"])
    if "target_directory" in copied:
        # The collector explicitly places compiler output outside its sealed
        # source directory. Validate that separate physical path rather than
        # interpreting it as a source path.
        from ..resources import physical_path_under
        drive_root = build_root_containing(workspace_root)
        physical_path_under(drive_root, copied["target_directory"])
        copied["target_directory"] = "<managed-target>"
    return copied


def _trusted_resource_estimate(state, source_ref: str) -> tuple[str, dict[str, int]]:
    policy_record = state.get("resource_policy", "default")
    if not policy_record or not isinstance(policy_record.get("payload"), Mapping):
        raise JenkinsError("resource_policy_missing", "trusted resource policy is unavailable")
    policy = policy_record["payload"]
    estimates = policy.get("recipeEstimates")
    if not isinstance(estimates, Mapping):
        raise JenkinsError("resource_estimate_missing", "trusted recipe estimates are unavailable")
    selected = state.get("validation_profile", source_ref)
    profile = "default"
    if selected and isinstance(selected.get("payload"), Mapping):
        profile_payload = selected["payload"]
        if profile_payload.get("sourceDigest", source_ref) != source_ref:
            raise JenkinsError("validation_profile_mismatch", "trusted validation profile is bound to another source")
        if profile_payload.get("status") not in (None, "verified", "accepted"):
            raise JenkinsError("validation_profile_untrusted", "validation profile is not trusted")
        value = profile_payload.get("profile") or profile_payload.get("name")
        if isinstance(value, str) and value:
            profile = value
    estimate = estimates.get(profile)
    if not isinstance(estimate, Mapping):
        raise JenkinsError("resource_estimate_missing", "trusted validation profile has no resource estimate")
    result: dict[str, int] = {}
    for field in ("cpu", "memoryBytes", "diskBytes"):
        value = estimate.get(field)
        if type(value) is not int or value <= 0:
            raise JenkinsError("resource_estimate_invalid", f"trusted resource estimate {field} is invalid")
        result[field] = value
    return profile, result


@dataclass(frozen=True)
class RecipePlan:
    recipe_ref: str
    recipe: dict[str, Any]
    source_ref: str
    coverage_ref: str
    generation: str
    driver: dict[str, Any]


class RecipePlanner:
    """Create recipes from State sealed inputs and controlled host evidence."""

    def __init__(self, state, *, repo_root: str | Path,
                 build_root: str | Path | None = None,
                 cargo_path: str | Path | None = None):
        self.state = state
        self.repo_root = Path(repo_root).absolute()
        self.build_root = canonical_build_root(build_root)
        # ``cargo_path`` is retained as a compatibility-shaped constructor
        # argument for callers that already construct a planner.  The actual
        # executable is selected only from the verified State environment
        # policy in ``plan``; a caller cannot smuggle a different binary into
        # a durable recipe.
        self.cargo = Path(cargo_path).absolute() if cargo_path is not None else None

    def _sealed(self, ref: str) -> dict[str, Any]:
        require_digest(ref, "sealedInputRef")
        record = self.state.get("sealed_input", ref)
        if not record or record["payload"].get("status") != "sealed":
            raise JenkinsError("sealed_input_missing", "sealed input reference is not available")
        payload = _plain(record["payload"])
        if payload.get("sourceDigest") != ref:
            raise JenkinsError("sealed_input_identity_mismatch", "sealed input key differs from source digest")
        coverage = payload.get("coverage")
        if not isinstance(coverage, Mapping) or coverage.get("sourceDigest") not in (None, ref):
            raise JenkinsError("coverage_source_mismatch", "coverage is not bound to sealed source")
        return payload

    def plan(self, *, sealed_input_ref: str, driver: Mapping[str, Any],
             changed_paths: Iterable[str] = (), features: Iterable[str] = (),
             profile: str | None = None, template: str = "module_unit",
             cargo_path: str | Path | None = None) -> RecipePlan:
        sealed = self._sealed(sealed_input_ref)
        if not isinstance(driver, Mapping) or not isinstance(driver.get("digest"), str):
            raise JenkinsError("driver_binding_missing", "trusted driver digest is required")
        driver_digest = require_digest(driver["digest"], "driverDigest")
        generation = driver.get("generation")
        if not isinstance(generation, (str, int)) or not str(generation):
            raise JenkinsError("driver_generation_missing", "trusted active driver generation is required")
        source_manifest = sealed.get("sourceManifest") or sealed.get("manifest") or {}
        entries = source_manifest.get("entries", ()) if isinstance(source_manifest, Mapping) else ()
        all_paths = [str(item.get("path")) for item in entries if isinstance(item, Mapping) and item.get("path")]
        selected_paths = list(changed_paths) if changed_paths else all_paths
        evidence = _cargo_evidence(self.state, sealed_input_ref)
        metadata = parse_metadata(evidence["cargoMetadata"])
        package_ids = set()
        for path in selected_paths:
            normalized = str(path).replace("\\", "/").casefold()
            for package in metadata.get("packages", ()):
                if not isinstance(package, Mapping) or not package.get("id"):
                    continue
                candidates = [package.get("manifest_path", "")]
                candidates.extend(t.get("src_path", "") for t in package.get("targets", ()) if isinstance(t, Mapping))
                for candidate in candidates:
                    candidate = str(candidate).replace("\\", "/").casefold()
                    # The evidence metadata was normalized from its sealed
                    # D:/E:/F: workspace root.  Accept an already relative
                    # path as-is; an absolute path outside that root is never
                    # interpreted relative to the live checkout.
                    # A manifest path names the package root; a target source
                    # path names a file below it.  Compare both against the
                    # sealed relative paths, never against the live checkout.
                    package_root = candidate.rsplit("/", 1)[0] if "/" in candidate else ""
                    if candidate and (normalized == candidate or
                                      (package_root and (normalized == package_root or normalized.startswith(package_root + "/"))) or
                                      False):
                        package_ids.add(str(package["id"])); break
        if selected_paths and not package_ids and template not in {"comments_only", "cache_maintenance"}:
            raise JenkinsError("unknown_impact", "unknown impact: changed paths do not map to Cargo packages")
        adapter = build_adapter_payload(metadata, package_ids, profile=profile, features=features)
        coverage = _plain(sealed.get("coverage") or {})
        # A sealed coverage receipt may carry a positive discovery count from
        # an earlier managed ``cargo test -- --list`` step.  Target metadata
        # alone is never interpreted as a test count.
        discovery = coverage.get("testDiscovery")
        selected_tests = 0
        if isinstance(discovery, Mapping) and discovery.get("status") == "accepted":
            count = discovery.get("count")
            if isinstance(count, int) and count > 0 and isinstance(discovery.get("receiptRef"), str):
                selected_tests = count
        package_names = adapter["packages"]
        if template not in {"comments_only", "cache_maintenance"}:
            if not package_names:
                raise JenkinsError("cargo_packages_missing", "Cargo validation requires affected package selection")
        env, environment_policy, environment_file = _environment_policy(self.state, self.repo_root)
        cargo_value = environment_policy.get("cargoPath")
        cargo = _cargo_path(cargo_value)
        expected_cargo = environment_policy.get("cargoSha256")
        if not isinstance(expected_cargo, str) or file_digest(cargo) != expected_cargo:
            raise JenkinsError("cargo_policy_mismatch", "Cargo executable differs from trusted environment policy")
        toolchain = evidence.get("toolchain")
        toolchain_digest = evidence.get("toolchainDigest")
        compiler_identity = evidence.get("compilerIdentity")
        lock_digest = evidence.get("lockDigest")
        if not isinstance(toolchain, (str, Mapping)) or not isinstance(toolchain_digest, str) or not isinstance(lock_digest, str):
            raise JenkinsError("toolchain_evidence_missing", "sealed Cargo evidence lacks toolchain and lock identity")
        wrapper = env.get("CARGO_BUILD_RUSTC_WRAPPER") or env.get("RUSTC_WRAPPER", "")
        profile_name, resource_estimate = _trusted_resource_estimate(self.state, sealed_input_ref)
        coverage_semantics = {str(k): _plain(v) for k, v in coverage.items()
                              if str(k).casefold() not in {"sourcedigest", "sourcemanifestref", "manifestdigest", "coveragedigest"}}
        recipe: dict[str, Any] = {
            "kind": "cargo" if template not in {"comments_only", "cache_maintenance"} else "light",
            "template": template, "readOnly": True, "sourceDigest": sealed_input_ref,
            "coverageDigest": str(sealed.get("coverageDigest") or digest(coverage)),
            "sourceManifestRef": sealed_input_ref, "packages": adapter["packages"],
            "packageIds": adapter["packageIds"], "targets": adapter["targets"],
            "features": adapter["features"], "profile": adapter["profile"],
            "testCount": selected_tests, "toolchain": _plain(toolchain),
            "toolchainDigest": toolchain_digest, "lockDigest": lock_digest,
            "compilerIdentity": compiler_identity,
            "cargoPath": str(cargo), "wrapper": wrapper, "environment": env,
            "environmentFile": str(environment_file),
            "environmentFileDigest": str(environment_policy.get("fileDigest")),
            "environmentPolicyDigest": str(environment_policy.get("environmentDigest") or digest(env)),
            "environmentDigest": digest(env), "buildRoot": str(self.build_root.path),
            "cargoSha256": str(environment_policy.get("cargoSha256")),
            "rustcPath": environment_policy.get("rustcPath"),
            "rustcSha256": environment_policy.get("rustcSha256"),
            "host": environment_policy.get("host"),
            "msvcVersion": environment_policy.get("msvcVersion"),
            "driverGeneration": str(generation), "driverDigest": driver_digest,
             "resourceProfile": profile_name, "resource": resource_estimate,
             "resourceEstimate": resource_estimate,
             "resourceEstimates": resource_estimate,
            "workspaceRoot": str(evidence["workspaceRoot"]),
            "cargoMetadata": metadata,
            "metadataDigest": str(evidence.get("metadataDigest") or digest(metadata)),
            "normalizedMetadataDigest": digest(metadata),
            "coverageSemanticsDigest": digest(coverage_semantics),
            "commands": {}, "requiredPhases": list(_CARGO_PHASES),
        }
        if recipe["kind"] == "cargo":
            test_stages = {"module_unit": ["unit_test"],
                           "cross_module": ["unit_test", "integration_test"],
                           "failure_repair": ["unit_test", "regression_test"]}.get(template)
            if not test_stages:
                raise JenkinsError("cargo_template_invalid", "Cargo requires a registered validation template")
            recipe["requiredTestStages"] = test_stages
            recipe["phaseStages"] = {"compile": "build", "list": "discovery", "test": "unit_test"}
            recipe["requiredOutputs"] = ["stdout.log", "stderr.log"]
            recipe["requiredOutputGlobs"] = ["binaries/*.exe"]
            selector = [part for name in package_names for part in ("--package", name)]
            feature_args = (["--no-default-features"] if bool(coverage.get("noDefaultFeatures", False)) else [])
            if adapter["features"]:
                feature_args += ["--features", ",".join(adapter["features"])]
            base = [str(cargo), "test", "--locked", "--offline", "--jobs", str(resource_estimate["cpu"]), *selector, *feature_args]
            if recipe["profile"] != "dev":
                base += ["--profile", recipe["profile"]]
            compile_command = [*base, "--no-run"]
            discovery_command = [*base, "--", "--list"]
            run_command = [*base, "--", "--test-threads", str(resource_estimate["cpu"])]
            recipe["commands"] = {"compile": compile_command, "list": discovery_command, "test": run_command}
            recipe["command"] = compile_command
            command_semantics = {phase: ["<cargo>" if index == 0 else value for index, value in enumerate(command)]
                                 for phase, command in recipe["commands"].items()}
            recipe["preparationKey"] = digest({"schemaVersion": 1, "toolchain": toolchain,
                                                "toolchainDigest": toolchain_digest, "lock": lock_digest,
                                                "compiler": compiler_identity, "wrapper": wrapper,
                                                "coverage": coverage_semantics,
                                                "command": command_semantics,
                                                "profile": adapter["profile"], "features": adapter["features"],
                                                "environment": recipe["environmentPolicyDigest"]})
            namespace = self.build_root.namespace() / "preparations" / recipe["preparationKey"]
            materialized = evidence.get("materializedRoot") or evidence.get("runRoot") or evidence.get("workspaceRoot")
            if not isinstance(materialized, str):
                raise JenkinsError("cargo_workspace_invalid", "sealed materialized workspace is not under an approved root")
            physical_path_under(self.build_root, materialized)
            recipe["preparationRoot"] = str(namespace / "inputs" / "source")
            recipe["materializedRoot"] = materialized
            # The execution owner materializes this stable source directory
            # after acquiring its PreparationKey writer reservation. Planning
            # must not modify a pool that another execution is using.
            execution = dict(env)
            execution.update({"CARGO_TARGET_DIR": str(namespace / "target"),
                              "CARGO_HOME": str(namespace / "cargo-home"),
                              "CARGO_BUILD_BUILD_DIR": str(namespace / "build"),
                              "SCCACHE_DIR": str(namespace / "sccache"),
                              "TEMP": str(namespace / "tmp"), "TMP": str(namespace / "tmp"),
                              "TMPDIR": str(namespace / "tmp"), "RUSTC_WRAPPER": "",
                              "RUSTC_WORKSPACE_WRAPPER": "",
                              "RUSTC": str(environment_policy.get("rustcPath") or ""),
                              "CARGO_ENCODED_RUSTFLAGS": "", "RUSTFLAGS": "", "RUSTDOCFLAGS": ""})
            recipe["executionEnvironment"] = dict(sorted(execution.items()))
            recipe["outputRoot"] = str(namespace / "target")
            recipe["targetDir"] = str(namespace / "target")
            recipe["tempRoot"] = str(namespace / "tmp")
            recipe["cargoHome"] = str(namespace / "cargo-home")
            recipe["testAllocation"] = {"source": "cargo-test-list", "runPerDiscoveredTest": True,
                                        "zeroTests": "reject"}
            recipe["requiredTests"] = list(coverage.get("requiredTests") or ())
            recipe["originalFailure"] = coverage.get("originalFailure")
            # ExecutionHost consumes deterministic phase names and resolves
            # each sealed command from ``commands``.  The materialized source
            # root is carried separately so host cwd validation remains strict.
            recipe["phases"] = list(_CARGO_PHASES)
        recipe_digest = digest(recipe)
        coverage_ref = str(sealed.get("coverageDigest") or digest(coverage))
        payload = {"recipe": recipe, "recipeDigest": recipe_digest, "sourceRef": sealed_input_ref,
                   "coverageRef": coverage_ref, "driver": _plain(driver), "generation": generation,
                   "status": "planned"}
        self._register(recipe_digest, payload)
        return RecipePlan(recipe_digest, recipe, sealed_input_ref, coverage_ref, str(generation), _plain(driver))

    def plan_native_fixture(self, *, source_ref: str, driver: Mapping[str, Any],
                            command: Iterable[str], fixture_kind: str = "native-validation") -> RecipePlan:
        """Internal test-only subprocess plan; it is never labelled as Cargo."""
        if fixture_kind != "native-validation":
            raise JenkinsError("fixture_kind_invalid", "unsupported native fixture kind")
        values = list(command)
        if not values or any(not isinstance(v, str) or not v for v in values):
            raise JenkinsError("fixture_command_invalid", "native fixture command is invalid")
        require_digest(source_ref, "sourceRef")
        if not isinstance(driver, Mapping) or not isinstance(driver.get("digest"), str):
            raise JenkinsError("driver_binding_missing", "trusted driver digest is required")
        generation = driver.get("generation")
        if not isinstance(generation, (str, int)) or not str(generation):
            raise JenkinsError("driver_generation_missing", "trusted active driver generation is required")
        recipe = {"kind": fixture_kind, "readOnly": True, "sourceDigest": source_ref,
                  "command": values, "buildRoot": str(self.build_root.path),
                  "driverGeneration": str(generation), "driverDigest": require_digest(driver["digest"], "driverDigest")}
        ref = digest(recipe)
        self._register(ref, {"recipe": recipe, "recipeDigest": ref, "sourceRef": source_ref,
                             "coverageRef": "", "driver": _plain(driver), "generation": str(generation),
                             "status": "planned", "fixtureOnly": True})
        return RecipePlan(ref, recipe, source_ref, "", str(generation), _plain(driver))

    def _register(self, ref: str, payload: dict[str, Any]) -> None:
        existing = self.state.get("recipe_plan", ref)
        if existing is not None:
            if digest(existing["payload"]) != digest(payload):
                raise JenkinsError("recipe_plan_conflict", "recipe reference is already registered with different payload")
            return
        self.state.put("recipe_plan", ref, payload)


def get_recipe_plan(state, recipe_ref: str) -> dict[str, Any]:
    require_digest(recipe_ref, "recipeRef")
    record = state.get("recipe_plan", recipe_ref)
    if not record or record["payload"].get("recipeDigest") != recipe_ref:
        raise JenkinsError("recipe_plan_missing", "recipe reference is not registered")
    payload = _plain(record["payload"])
    if digest(payload["recipe"]) != recipe_ref:
        raise JenkinsError("recipe_plan_corrupt", "recipe digest does not match registered plan")
    return payload


__all__ = ["RecipePlan", "RecipePlanner", "get_recipe_plan"]
