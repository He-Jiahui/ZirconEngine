"""Fixed editor product phases run only by a registered native execution.

The planner chooses the tools, profiles and commands. This child consumes that
immutable plan and the coordinator materialization; it never runs the independent
preview entry or accepts an arbitrary shell command.
"""
from __future__ import annotations

import ast
import importlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import posixpath
import tomllib
from typing import Mapping

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from tools.jenkins.contracts import JenkinsError, digest, file_digest, require_digest
from tools.jenkins.resources import canonical_build_root, physical_path_under
from tools.jenkins.source import compute_source_digest
from tools.jenkins.validation.receipts import validate_producer_receipt

TEMPLATES = frozenset({"editor_product_development", "editor_product_shipping"})
PHASES = ("check", "compile", "list", "test", "editor_exe", "runtime_dll", "package")
PHASE_STAGES = {"check": "package_check", "compile": "build", "list": "discovery",
                "test": "unit_test", "editor_exe": "editor_exe", "runtime_dll": "runtime_dll", "package": "package"}
NATIVE_STAGES = ("package_check", "build", "unit_test", "editor_exe", "runtime_dll", "package")
PACKAGES = ("zircon_runtime_interface", "zircon_runtime", "zircon_editor", "zircon_app")
# Production include_bytes! inputs from the Interface template pack.
# Hidden .gitignore files are compiler inputs too; a directory inventory alone
# must not silently omit them from a product preparation.
PROJECT_TEMPLATE_INPUTS = tuple("templates/projects/renderable-empty/" + path for path in (
    ".gitignore", ".zircon/autosave/.gitignore", ".zircon/cache/.gitignore",
    ".zircon/play/.gitignore", ".zircon/registry/.gitignore", ".zircon/settings.toml",
    ".zircon/thumbnails/.gitignore", "assets/materials/default.zmaterial",
    "assets/materials/default.zmaterial.zmeta", "assets/models/cube.obj",
    "assets/models/cube.obj.zmeta", "assets/scenes/main.scene.toml",
    "assets/shaders/pbr_shader.zmeta", "assets/shaders/pbr_shader/pbr.wgsl",
    "assets/shaders/pbr_shader/pbr.zshader", "export/desktop_windows.zpreset",
    "zircon-project.toml",
))
HELPER_MODULES = ("tools.build.build_editor", "tools.dev.local_cargo",
                  "tools.build.zircon_build_product_profile", "tools.build.zircon_build_runtime_manifest",
                  "tools.build.zircon_build_size_report")


def _entries(manifest: Mapping) -> dict[str, dict]:
    return {str(entry["path"]).casefold(): entry for entry in manifest.get("entries", [])
            if entry.get("status") == "present"}


def helper_closure(manifest: Mapping, objects: Path) -> dict[str, str]:
    """Prove the transitive local imports without importing any source module."""
    entries = _entries(manifest)
    required = set()
    pending = list(HELPER_MODULES)
    visited = set()
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        visited.add(module)
        path = module.replace(".", "/") + ".py"
        entry = entries.get(path.casefold())
        if not entry:
            raise JenkinsError("editor_helper_missing", "Product helper import is absent from the sealed closure", details={"path": path})
        source = objects / "inputs" / "objects" / str(entry.get("objectDigest", ""))
        if not source.is_file() or file_digest(source) != entry.get("sha256"):
            raise JenkinsError("editor_helper_corrupt", "Sealed product helper object differs from its manifest")
        required.add(path)
        try:
            tree = ast.parse(source.read_bytes(), filename=path)
        except (SyntaxError, UnicodeError) as error:
            raise JenkinsError("editor_helper_invalid", "Sealed product helper is not valid Python") from error
        for node in ast.walk(tree):
            names = []
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                if node.level:
                    raise JenkinsError("editor_helper_import_invalid", "Product helper closure requires explicit local module imports")
                base = node.module or ""
                if base in {"tools.build", "tools.dev"}:
                    names = [base + "." + alias.name for alias in node.names]
                else:
                    names = [base]
            for name in names:
                if name.startswith(("tools.build.", "tools.dev.")):
                    pending.append(name)
                elif name == "tools" or name.startswith("tools.jenkins"):
                    # Jenkins imports belong to the verified immutable driver.
                    continue
                elif name.startswith("tools."):
                    raise JenkinsError("editor_helper_import_invalid", "Product helper imports an unsupported local namespace")
    # These directories are namespace packages in ZirconEngine. An initializer
    # supplied by another closure would execute too, and must be sealed/parsed.
    for package in ("tools", "tools/build", "tools/dev"):
        path = package + "/__init__.py"
        if path in entries:
            entry = entries[path]
            obj = objects / "inputs" / "objects" / str(entry["objectDigest"])
            if not obj.is_file() or file_digest(obj) != entry["sha256"]:
                raise JenkinsError("editor_helper_corrupt", "Sealed helper package initializer differs")
            tree = ast.parse(obj.read_bytes(), filename=path)
            if any(isinstance(node, (ast.Import, ast.ImportFrom)) for node in ast.walk(tree)):
                raise JenkinsError("editor_helper_import_invalid", "Product namespace initializers cannot import unverified modules")
            required.add(path)
    missing = sorted(path for path in required if path.casefold() not in entries)
    if missing:
        raise JenkinsError("editor_helper_missing", "Product helper package files are not sealed", details={"paths": missing})
    return {path: entries[path.casefold()]["sha256"] for path in sorted(required)}


def template_closure(manifest: Mapping, objects: Path) -> dict[str, str]:
    """Bind every production project template to its verified sealed bytes."""
    present = _entries(manifest)
    missing = sorted(path for path in PROJECT_TEMPLATE_INPUTS if path.casefold() not in present)
    if missing:
        raise JenkinsError("editor_inputs_missing", "Mandatory project template inputs are absent", details={"paths": missing})
    result = {}
    for path in sorted(PROJECT_TEMPLATE_INPUTS):
        entry = present[path.casefold()]
        checksum = require_digest(entry.get("sha256"), "projectTemplateHash")
        if entry.get("objectDigest") != checksum:
            raise JenkinsError("editor_template_corrupt", "Sealed project template object differs from its content hash", details={"path": path})
        obj = objects / "inputs" / "objects" / checksum
        if not obj.is_file() or file_digest(obj) != checksum:
            raise JenkinsError("editor_template_corrupt", "Sealed project template object is missing or corrupt", details={"path": path})
        result[path] = checksum
    return result


def mandatory_inputs(manifest: Mapping, *, shipping: bool, objects: Path) -> None:
    present = _entries(manifest)
    if manifest.get("worktreePreparationRef"):
        from .product_inputs import require_product_inputs
        require_product_inputs({path: entry.get("sha256") for path, entry in present.items()},
                               manifest.get("compilerControlInputs"))
    required = {"cargo.toml", "cargo.lock", *[name + "/cargo.toml" for name in PACKAGES],
                "zircon_runtime/assets/fonts/default.font.toml", "zircon_runtime/assets/fonts/editor-ui.font.toml",
                "zircon_runtime_interface/src/runtime_build_set/interface_spec_v1.json",
                "zircon_runtime_interface/src/runtime_build_set/payload_schema_set_v1.json",
                *[path.casefold() for path in PROJECT_TEMPLATE_INPUTS]}
    # Asset membership is exact in the current-source intent. The package helper
    # validates every mandatory font payload and the BuildSet schema at staging.
    if not any(path.startswith("zircon_editor/assets/") for path in present):
        raise JenkinsError("editor_assets_missing", "The sealed product closure contains no editor assets")
    if shipping:
        required.add("tools/export/product_build_profiles.toml")
    missing = sorted(required - set(present))
    if missing:
        raise JenkinsError("editor_inputs_missing", "Mandatory editor product inputs are absent", details={"paths": missing})
    for name in ("default.font.toml", "editor-ui.font.toml"):
        relative = "zircon_runtime/assets/fonts/" + name
        entry = present[relative]
        obj = objects / "inputs" / "objects" / str(entry["objectDigest"])
        if not obj.is_file() or file_digest(obj) != entry["sha256"]:
            raise JenkinsError("editor_font_invalid", "Sealed mandatory font descriptor is corrupt")
        try:
            descriptor = tomllib.loads(obj.read_text(encoding="utf-8"))
        except (ValueError, UnicodeError) as error:
            raise JenkinsError("editor_font_invalid", "Sealed mandatory font descriptor is invalid") from error
        payload = descriptor.get("source")
        if (not isinstance(payload, str) or not payload.strip() or ":" in payload
                or payload.startswith(("/", "\\"))):
            raise JenkinsError("editor_font_invalid", "Mandatory font source must be an asset-relative path")
        payload = posixpath.normpath("zircon_runtime/assets/fonts/" + payload.strip().replace("\\", "/")).casefold()
        font = present.get(payload)
        if not payload.startswith("zircon_runtime/assets/") or not font or int(font.get("size", 0)) <= 0:
            raise JenkinsError("editor_font_missing", "Mandatory font payload is absent or empty in the sealed closure")


def product_recipe(recipe: dict, *, template: str, manifest: Mapping, objects: Path,
                   python: Path, runner: Path) -> dict:
    shipping = template == "editor_product_shipping"
    mandatory_inputs(manifest, shipping=shipping, objects=objects)
    helpers = helper_closure(manifest, objects)
    templates = template_closure(manifest, objects)
    feature = "shipping-editor" if shipping else "target-editor-host"
    jobs = str(recipe["resourceEstimates"]["cpu"])
    cargo = recipe["cargoPath"]
    common = ["--locked", "--offline", "--no-default-features", "--jobs", jobs]
    if shipping:
        common += ["--profile", "shipping", "--target", "x86_64-pc-windows-msvc"]
    selector = [part for name in PACKAGES for part in ("--package", name)]
    features = ["--features", f"zircon_app/{feature},zircon_runtime/{feature}"]
    tests = [cargo, "test", *common, *selector, *features]
    command = [str(python), "-B", str(runner)]
    commands = {"check": [*command, "check"], "compile": [*tests, "--no-run"],
                "list": [*tests, "--", "--list"],
                "test": [*tests, "--", "--test-threads", jobs],
                "editor_exe": [*command, "editor_exe"], "runtime_dll": [*command, "runtime_dll"],
                "package": [*command, "package"]}
    required = {phase: ["stdout.log", "stderr.log"] for phase in PHASES if phase != "list"}
    required.update(check=["stdout.log", "stderr.log", "package-checks.json"],
                    editor_exe=["stdout.log", "stderr.log", "zircon_editor.exe", "artifact.json"],
                    runtime_dll=["stdout.log", "stderr.log", "zircon_runtime.dll", "artifact.json"],
                    package=["stdout.log", "stderr.log", "product-package.json", "bundle/zircon_editor.exe", "bundle/zircon_runtime.dll",
                             "bundle/zircon_runtime.dll.manifest.json", "bundle/assets/fonts/default.font.toml",
                             "bundle/assets/fonts/editor-ui.font.toml"])
    required["package"] += ["bundle/msvcp140.dll", "bundle/vcruntime140.dll", "bundle/vcruntime140_1.dll"]
    if shipping:
        required["package"].append("bundle.zip")
    result = {**recipe, "kind": "cargo-editor-product", "template": template,
              "packages": list(PACKAGES), "features": [feature], "profile": "shipping" if shipping else "dev",
              "productProfile": "shipping" if shipping else "development", "commands": commands,
              "command": commands["check"], "phases": list(PHASES), "requiredPhases": list(PHASES),
              "phaseStages": dict(PHASE_STAGES), "requiredNativeStages": list(NATIVE_STAGES),
              "requiredTestStages": ["unit_test"], "requiredOutputsByPhase": required,
              "requiredOutputsByStage": {PHASE_STAGES[phase]: value for phase, value in required.items()},
              "requiredOutputGlobs": [], "requiredOutputGlobsByPhase": {"compile": ["binaries/*.exe"], "test": ["binaries/*.exe"]},
              "productHelperClosure": helpers, "productHelperClosureDigest": digest(helpers),
              "productTemplateClosure": templates, "productTemplateClosureDigest": digest(templates),
              "productPython": str(python), "productPythonSha256": file_digest(python),
              "productRunner": str(runner), "productRunnerSha256": file_digest(runner),
              "nativeUiAcceptance": False, "mvpAcceptance": False, "shippingAcceptance": False}
    return result


def _verify_source(recipe: dict, sealed: dict, source: Path) -> None:
    manifest = sealed["sourceManifest"]
    if compute_source_digest(manifest) != recipe["sourceDigest"]:
        raise JenkinsError("editor_source_mismatch", "Product source manifest differs from its recipe")
    expected = {str(entry["path"]): entry["sha256"] for entry in manifest["entries"] if entry["status"] == "present"}
    actual = {}
    for path in source.rglob("*"):
        if path.is_file():
            physical_path_under(canonical_build_root(recipe["buildRoot"]), path, allow_missing=False)
            actual[path.relative_to(source).as_posix()] = file_digest(path)
    if actual != expected:
        raise JenkinsError("editor_source_mismatch", "Prepared product source differs from the complete sealed closure")
    from tools.jenkins.source.external_inputs import verify_prepared
    verify_prepared(manifest, source, canonical_build_root(recipe["buildRoot"]))


def _fresh_output(output: Path) -> None:
    if output.exists():
        if any(output.iterdir()):
            raise JenkinsError("editor_output_existing", "Product phase cannot overwrite previous output evidence")
    output.mkdir(parents=True, exist_ok=True)


def _context(phase: str):
    from tools.jenkins.state import State
    from tools.jenkins.workflow.planning import get_recipe_plan
    state_path = os.environ.get("ZIRCON_PRODUCT_STATE", "")
    execution_id = os.environ.get("ZIRCON_PRODUCT_EXECUTION", "")
    if not state_path or not execution_id or phase not in {"check", "editor_exe", "runtime_dll", "package"}:
        raise JenkinsError("editor_execution_missing", "Product phase requires the native execution owner context")
    state = State(state_path)
    execution = state.get("execution", execution_id)
    if not execution or execution["payload"].get("status") != "running":
        raise JenkinsError("editor_execution_missing", "Product phase has no running registered execution")
    payload = execution["payload"]
    plan = get_recipe_plan(state, payload["recipeRef"])
    recipe = plan["recipe"]
    if recipe.get("kind") != "cargo-editor-product" or recipe.get("template") not in TEMPLATES:
        raise JenkinsError("editor_recipe_invalid", "Native product phase requires a fixed trusted editor recipe")
    if (str(Path(sys.executable).resolve()) != recipe["productPython"]
            or file_digest(sys.executable) != recipe["productPythonSha256"]
            or str(Path(__file__).resolve()) != recipe["productRunner"]
            or file_digest(__file__) != recipe["productRunnerSha256"]):
        raise JenkinsError("editor_runner_changed", "Product Python or immutable driver runner differs from its plan")
    from tools.jenkins.workflow.pools import _hold
    _hold(state, execution_id, payload["preparationKey"])
    approved = canonical_build_root(recipe["buildRoot"])
    output = physical_path_under(approved, Path(payload["phaseOutputRoot"]) / phase)
    if str(output) != os.environ.get("ZIRCON_PRODUCT_OUTPUT_ROOT"):
        raise JenkinsError("editor_output_mismatch", "Native product phase differs from its registered phase output")
    _fresh_output(output)
    source = physical_path_under(approved, recipe["preparationRoot"], allow_missing=False)
    sealed = state.get("sealed_input", payload["sourceDigest"])["payload"]
    _verify_source(recipe, sealed, source)
    return state, execution_id, payload, recipe, sealed, source, output


def _helper(recipe: dict, source: Path):
    for relative, expected in recipe["productHelperClosure"].items():
        if file_digest(source / relative) != expected:
            raise JenkinsError("editor_helper_changed", "Prepared packaging helper differs from its registered digest")
    import tools
    prepared_tools = str(source / "tools")
    driver_tools = str(Path(__file__).resolve().parent.parent)
    # Remove ambient checkout namespace entries before any helper import.
    tools.__path__ = [driver_tools, prepared_tools]
    modules = [path[:-3].replace("/", ".") for path in recipe["productHelperClosure"]
               if not path.endswith("/__init__.py")]
    for module in ("tools.build", "tools.dev", *modules):
        loaded = sys.modules.get(module)
        if loaded and getattr(loaded, "__file__", None) and not Path(loaded.__file__).resolve().is_relative_to(source):
            raise JenkinsError("editor_helper_unsealed", "Packaging helper was imported outside prepared inputs")
        if loaded and getattr(loaded, "__path__", None) and any(not Path(path).resolve().is_relative_to(source) for path in loaded.__path__):
            raise JenkinsError("editor_helper_unsealed", "Packaging namespace was loaded outside prepared inputs")
    helper = importlib.import_module("tools.build.build_editor")
    if Path(helper.__file__).resolve() != (source / "tools/build/build_editor.py").resolve():
        raise JenkinsError("editor_helper_unsealed", "Packaging helper must be loaded from exact prepared inputs")
    return helper


def _json(path: Path, value: dict):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def _prior_artifact(state, execution_id: str, payload: dict, recipe: dict, phase: str, name: str) -> tuple[Path, str]:
    attempt_id = execution_id + ":" + phase
    attempt = state.get("validation_attempt", attempt_id)
    receipt = state.get("validation_receipt", attempt_id)
    if not attempt or attempt["payload"].get("status") != "passed" or not receipt:
        raise JenkinsError("editor_artifact_receipt_missing", "Product package requires both preceding native artifact receipts")
    root = physical_path_under(canonical_build_root(recipe["buildRoot"]), Path(payload["phaseOutputRoot"]) / phase, allow_missing=False)
    validate_producer_receipt(receipt["payload"], output_root=root, required_outputs=[name, "artifact.json"],
                             expected={"executionId": attempt_id, "stage": phase, "sourceDigest": recipe["sourceDigest"],
                                       "coverageDigest": recipe["coverageDigest"], "recipeDigest": payload["recipeRef"],
                                       "driverDigest": recipe["driverDigest"]})
    artifact = json.loads((root / "artifact.json").read_text(encoding="utf-8"))
    if artifact.get("file_name") != name or artifact.get("sha256") != file_digest(root / name):
        raise JenkinsError("editor_artifact_changed", "Product artifact differs from its native phase receipt")
    return root / name, artifact["sha256"]


def run_phase(phase: str) -> int:
    state, eid, payload, recipe, sealed, source, output = _context(phase)
    helper = _helper(recipe, source)
    jobs = recipe["resourceEstimates"]["cpu"]
    if phase == "check":
        results = []
        feature = "shipping-editor" if recipe["productProfile"] == "shipping" else "target-editor-host"
        for package in PACKAGES:
            command = [recipe["cargoPath"], "check", "--package", package, "--all-targets", "--locked", "--offline",
                       "--no-default-features", "--jobs", str(jobs)]
            if package in {"zircon_app", "zircon_runtime"}:
                command += ["--features", feature]
            if recipe["productProfile"] == "shipping":
                command += ["--profile", "shipping", "--target", "x86_64-pc-windows-msvc"]
            completed = subprocess.run(command, cwd=source, check=False, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            results.append({"package": package, "command": command, "exitCode": completed.returncode})
            if completed.returncode:
                return completed.returncode
        _json(output / "package-checks.json", {"sourceDigest": recipe["sourceDigest"], "checks": results})
    elif phase in {"editor_exe", "runtime_dll"}:
        index = 0 if phase == "editor_exe" else 1
        command = [recipe["cargoPath"], *helper.build_commands(recipe["productProfile"], jobs)[index]]
        command.insert(2, "--offline")
        log = output / "cargo.log"
        with log.open("xb") as stream:
            completed = subprocess.run(command, cwd=source, stdout=stream, stderr=subprocess.STDOUT,
                                       check=False, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        if completed.returncode:
            return completed.returncode
        name = "zircon_editor.exe" if index == 0 else "zircon_runtime.dll"
        artifact = helper.reported_artifact(log, name, Path(recipe["targetDir"]))
        checksum = file_digest(artifact)
        helper.verify_x64_pe(artifact, dll=index == 1)
        shutil.copyfile(artifact, output / name)
        if file_digest(artifact) != checksum or file_digest(output / name) != checksum:
            raise JenkinsError("editor_artifact_changed", "Compiled product changed while binding its phase artifact")
        _json(output / "artifact.json", {"file_name": name, "sha256": checksum, "command": command,
                                         "sourceDigest": recipe["sourceDigest"], "profile": recipe["productProfile"]})
    else:
        artifacts = {"zircon_editor.exe": _prior_artifact(state, eid, payload, recipe, "editor_exe", "zircon_editor.exe"),
                     "zircon_runtime.dll": _prior_artifact(state, eid, payload, recipe, "runtime_dll", "zircon_runtime.dll")}
        staged = output / "bundle.staging"
        staged.mkdir()
        source_files = helper.source_inventory(source)
        shell = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        package = helper.stage_editor_bundle(source, staged, artifacts, cargo_profile=recipe["productProfile"],
                                             source_files=source_files, crt_directory=None, shell=shell, environment=dict(os.environ))
        shipping = recipe["productProfile"] == "shipping"
        if shipping:
            publication = helper.publish_editor_shipping_bundle(source, staged, output / "bundle", package,
                                                                 source_files=source_files)
        else:
            publication = {"output_closure": helper.publish_editor_bundle(staged, output / "bundle", package["output_closure"])}
        _json(output / "product-package.json", {"sourceDigest": recipe["sourceDigest"], "buildSet": package["build_set"],
                                                "officialCrt": package["official_crt"], "publication": publication,
                                                "nativeUiAcceptance": False, "mvpAcceptance": False, "shippingAcceptance": False})
    _verify_source(recipe, sealed, source)
    return 0


def verify_test_run(text: str, required_tests: list[str]) -> dict:
    passed = set(re.findall(r"^test (.+?) \.\.\. ok\s*$", text, flags=re.MULTILINE))
    counts = [int(count) for count in re.findall(r"test result: ok\. (\d+) passed;", text)]
    missing = sorted(set(required_tests) - passed)
    if missing or not counts or sum(counts) <= 0:
        raise JenkinsError("editor_tests_not_run", "Product validation requires actual passing focused tests", details={"missing": missing})
    return {"passedCount": sum(counts), "requiredTests": sorted(required_tests)}


if __name__ == "__main__":
    try:
        raise SystemExit(run_phase(sys.argv[1] if len(sys.argv) == 2 else ""))
    except (JenkinsError, OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(2)
