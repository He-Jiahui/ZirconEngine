"""Existing public Node lineage: fixed contracts and a runner-only compiler."""
import hashlib
import json
from pathlib import Path
import subprocess
from types import SimpleNamespace
from unittest.mock import patch

import pytest

from test_managed_node import candidate, node


def contract_candidate(node, candidate):
    for name in node.SESSION_CONTRACT_NODE_SOURCES:
        p = candidate.app / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text("// synthetic source; no product execution\n", encoding="utf-8")
        candidate.payload["sourceHashes"][p.relative_to(candidate.repo).as_posix()] = node.file_hash(p)
    config = {"repo": str(candidate.repo), "spec": node.SESSION_CONTRACT_NODE_SOURCES[0],
              "outputFile": str(candidate.public / "session-contract-tests.json")}
    candidate.config_path.write_text(json.dumps(config), encoding="utf-8")
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-existing-session-contract-tests-r01",
        "config": {"path": str(candidate.config_path), "sha256": node.file_hash(candidate.config_path)}}
    return config


def test_session_contract_recipe_has_one_exact_file_and_preserves_real_ui(node, candidate):
    contract_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["command"] == [str(node.NODE), "--test", "--test-reporter=tap",
                               str(candidate.app / "tools/zui-layout-real-session-contract.test.mjs")]
    assert plan["builtin"] and plan["compileKind"] is None and not plan["worker"]
    assert plan["timeoutSeconds"] == 90 and plan["rule"]["expectedPassed"] is None
    assert node.RECIPES["penpot-real-ui-node-tests-r02"]["specs"] == ["tools/zui-layout-real-ui.test.mjs"]
    assert node.RECIPES["penpot-real-ui-node-tests-r02"]["expectedPassed"] == 14


@pytest.mark.parametrize("change", ["two-files", "wrong-file", "cwd", "command", "output", "missing-source", "old-hash"])
def test_session_contract_scope_and_source_controls_cannot_expand(node, candidate, change):
    cfg = contract_candidate(node, candidate)
    if change == "two-files": cfg["spec"] = node.SESSION_CONTRACT_NODE_SOURCES
    elif change == "wrong-file": cfg["spec"] = node.REAL_UI_NODE_SOURCES[0]
    elif change == "cwd": cfg["repo"] = str(candidate.app)
    elif change == "command": cfg["command"] = ["node", "arbitrary.mjs"]
    elif change == "output": cfg["outputFile"] = str(candidate.repo / "target/out.json")
    elif change == "missing-source":
        candidate.payload["sourceHashes"].pop((candidate.app / node.SESSION_CONTRACT_NODE_SOURCES[-1]).relative_to(candidate.repo).as_posix())
    candidate.config_path.write_text(json.dumps(cfg), encoding="utf-8")
    if change != "old-hash": candidate.payload["nodeRecipe"]["config"]["sha256"] = node.file_hash(candidate.config_path)
    else: candidate.config_path.write_text("{}", encoding="utf-8")
    with pytest.raises(ValueError): node.recipe(candidate.payload, candidate.repo)


def session_tap(count):
    return ("TAP version 13\n" + "\n".join(f"ok {i} - session-case-{i}" for i in range(1, count + 1))
            + f"\n1..{count}\n# tests {count}\n# pass {count}\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n")


@pytest.mark.parametrize("count", [1, 10, 27])
def test_contract_reports_actual_nonzero_discovery_without_claiming_static_counts(node, candidate, count):
    contract_candidate(node, candidate)
    details = node.verify_builtin_result(node.recipe(candidate.payload, candidate.repo), session_tap(count).encode())
    assert details["passed"] == count and len(details["testNames"]) == count
    assert details["specs"] == [node.SESSION_CONTRACT_NODE_SOURCES[0]]
    assert not details["headlessExecuted"]


@pytest.mark.parametrize("change", ["zero", "failed", "skipped", "cancelled", "todo", "duplicate-total",
                                    "missing-total", "missing-row", "duplicate-name", "wrong-plan"])
def test_contract_does_not_accept_empty_partial_failed_or_skipped_discovery(node, candidate, change):
    contract_candidate(node, candidate)
    text = session_tap(10)
    if change == "zero": text = session_tap(0)
    elif change == "failed": text = text.replace("# fail 0", "# fail 1")
    elif change == "skipped": text = text.replace("# skipped 0", "# skipped 1")
    elif change == "cancelled": text = text.replace("# cancelled 0", "# cancelled 1")
    elif change == "todo": text = text.replace("# todo 0", "# todo 1")
    elif change == "duplicate-total": text += "# tests 10\n"
    elif change == "missing-total": text = text.replace("# pass 10\n", "")
    elif change == "missing-row": text = text.replace("ok 10 - session-case-10\n", "")
    elif change == "duplicate-name": text = text.replace("session-case-10", "session-case-9")
    else: text = text.replace("1..10", "1..11")
    with pytest.raises(ValueError): node.verify_builtin_result(node.recipe(candidate.payload, candidate.repo), text.encode())


def registration_candidate(node, candidate):
    spec = "tools/zui-layout-real-plugin-registration.spec.ts"
    p = candidate.app / spec
    p.parent.mkdir(exist_ok=True)
    p.write_text("// synthetic registration test\n", encoding="utf-8")
    candidate.payload["sourceHashes"][p.relative_to(candidate.repo).as_posix()] = node.file_hash(p)
    candidate.config["test"]["include"] = [spec]
    candidate.config["test"].pop("testNamePattern")
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-existing-session-registration-r01",
                                       "config": candidate.write_config()}
    return p


def test_registration_recipe_is_exact_single_spec_without_browser_or_worker(node, candidate):
    registration_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["rule"]["specs"] == ["tools/zui-layout-real-plugin-registration.spec.ts"]
    assert plan["timeoutSeconds"] == 90 and not plan["worker"] and plan["compileKind"] is None
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]


@pytest.mark.parametrize("change", ["old-six", "extra-spec", "watch", "workers", "parallel", "coverage", "filter", "hook"])
def test_registration_controls_reject_wide_or_executable_config(node, candidate, change):
    registration_candidate(node, candidate)
    test = candidate.config["test"]
    if change == "old-six": test["include"] = list(node.PUBLIC_CANONICAL_SPECS)
    elif change == "extra-spec": test["include"].append(node.REAL_UI_NODE_SOURCES[0])
    elif change == "watch": test["watch"] = True
    elif change == "workers": test["maxWorkers"] = 2
    elif change == "parallel": test["fileParallelism"] = True
    elif change == "coverage": test["coverage"]["enabled"] = True
    elif change == "filter": test["testNamePattern"] = "anything"
    else: test["setupFiles"] = ["arbitrary.js"]
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError): node.recipe(candidate.payload, candidate.repo)


@pytest.fixture
def session_compile(node, candidate):
    rules = {name: dict(rule) for name, rule in node.COMPILE_RECIPES.items()}
    helper_root = candidate.public / "helpers"
    helper_root.mkdir()
    helper = helper_root / rules["session-runner"]["helper"]
    helper.write_text("// synthetic reviewed helper; never executed\n", encoding="utf-8")
    rules["session-runner"]["sha256"] = node.file_hash(helper)
    entry = candidate.app / rules["session-runner"]["entry"]
    entry.parent.mkdir(exist_ok=True)
    entry.write_text("// synthetic existing runner\n", encoding="utf-8")
    candidate.payload["sourceHashes"][entry.relative_to(candidate.repo).as_posix()] = node.file_hash(entry)
    cfg = {"repo": str(candidate.repo), "mainApp": str(candidate.app), "dependencyRoot": str(node.DEPS.parent),
           "out": str(candidate.output / "build"), "playwright": str(node.PLAYWRIGHT)}
    candidate.config_path.write_text(json.dumps(cfg), encoding="utf-8")
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-existing-session-runner-r01",
        "config": {"path": str(candidate.config_path), "sha256": node.file_hash(candidate.config_path)}}
    with patch.multiple(node, COMPILE_RECIPES=rules, COMPILE_HELPER_ROOT=helper_root,
                        COMPILE_REPORT_ROOT=candidate.public / "reports", COMPILE_OUTPUT_ROOT=candidate.output / "build"):
        yield SimpleNamespace(entry=entry, helper=helper, cfg=cfg,
                              plan=lambda: node.recipe(candidate.payload, candidate.repo))


def test_session_compiler_is_one_original_entry_and_distinct_from_worker_and_publisher(node, candidate, session_compile):
    plan = session_compile.plan()
    assert plan["compileKind"] == "session-runner" and not plan["worker"]
    assert plan["command"] == [str(node.NODE), str(session_compile.helper), str(candidate.config_path)]
    assert str(Path(plan["outputs"]["bundle"]).relative_to(candidate.output / "build")).replace("\\", "/") == (
        "public-component-closure-current/existing-plugin-session-r01/tools/zui-layout-real-session.mjs")
    assert set(plan["outputs"]) == {"bundle", "compileReceipt", "metafile"}
    assert node.COMPILE_RECIPES["publisher"]["entry"] != node.COMPILE_RECIPES["session-runner"]["entry"]
    assert node.COMPILE_RECIPES["worker"]["entry"] != node.COMPILE_RECIPES["session-runner"]["entry"]


@pytest.mark.parametrize("change", ["extra-command", "worker-entry", "out", "deps", "playwright", "helper"])
def test_session_compiler_rejects_generic_worker_and_alternate_config(node, candidate, session_compile, change):
    c = session_compile
    if change == "extra-command": c.cfg["command"] = ["node", "arbitrary.js"]
    elif change == "worker-entry": candidate.payload["sourceHashes"].pop(c.entry.relative_to(candidate.repo).as_posix())
    elif change == "out": c.cfg["out"] = str(candidate.repo / "target")
    elif change == "deps": c.cfg["dependencyRoot"] = "C:/unreviewed"
    elif change == "playwright": c.cfg["playwright"] = "C:/unreviewed.mjs"
    else: c.helper.write_text("changed helper", encoding="utf-8")
    candidate.config_path.write_text(json.dumps(c.cfg), encoding="utf-8")
    candidate.payload["nodeRecipe"]["config"]["sha256"] = node.file_hash(candidate.config_path)
    with pytest.raises(ValueError): c.plan()


@pytest.mark.parametrize("change", [None, "strict", "entries", "worker", "angular", "install", "before", "after",
                                    "program", "program-import", "metafile", "meta-import"])
def test_session_compilation_receipt_needs_full_pre_post_program_and_metafile_census(node, candidate, session_compile, change):
    c = session_compile
    plan = c.plan()
    for p in plan["outputs"].values(): Path(p).parent.mkdir(parents=True, exist_ok=True)
    output = Path(plan["outputs"]["bundle"])
    output.write_bytes(b"synthetic runner output")
    imported = candidate.app / "dependency.ts"
    imported.write_bytes(b"// synthetic import")
    inputs = {str(p): node.file_hash(p) for p in [c.entry, imported]}
    paths = list(inputs)
    data = {"status": "compiled-current", "typescriptNoEmitPassed": True, "typescriptStrictPassed": True,
            "sourceUnchanged": True, "sourceCopiesCreated": False, "entries": [str(c.entry)],
            "workerBuilt": False, "angularBuilt": False, "dependencyInstallation": False,
            "inputBeforeHashes": inputs, "inputAfterHashes": inputs,
            "programInputPaths": paths, "metafileInputPaths": paths,
            "inputs": [{"path": p, "sha256": v} for p, v in inputs.items()],
            "output": {"path": str(output), "sha256": node.file_hash(output)}}
    if change == "strict": data["typescriptStrictPassed"] = False
    elif change == "entries": data["entries"].append(str(imported))
    elif change == "worker": data["workerBuilt"] = True
    elif change == "angular": data["angularBuilt"] = True
    elif change == "install": data["dependencyInstallation"] = True
    elif change == "before": data["inputBeforeHashes"] = {}
    elif change == "after": data["inputAfterHashes"] = {**inputs, str(c.entry): "0" * 64}
    elif change == "program": data["programInputPaths"] = []
    elif change == "program-import": data["programInputPaths"] = [*paths, str(candidate.repo / "foreign.ts")]
    elif change == "metafile": data["metafileInputPaths"] = []
    elif change == "meta-import": data["metafileInputPaths"] = [str(c.entry)]
    meta = {"inputs": {p: {} for p in paths}, "outputs": {str(output): {"entryPoint": str(c.entry)}}}
    Path(plan["outputs"]["compileReceipt"]).write_text(json.dumps(data), encoding="utf-8")
    Path(plan["outputs"]["metafile"]).write_text(json.dumps(meta), encoding="utf-8")
    if change is None:
        result = node.verify_compile_result(plan, {"files": inputs})
        assert result["typescriptStrictPassed"] and not result["deployed"]
    else:
        with pytest.raises(ValueError): node.verify_compile_result(plan, {"files": inputs})


def test_generated_runner_helper_is_hash_pinned_and_node_syntax_valid_without_product_compilation(node):
    helper = node.COMPILE_HELPER_ROOT / node.COMPILE_RECIPES["session-runner"]["helper"]
    assert hashlib.sha256(helper.read_bytes()).hexdigest() == node.COMPILE_RECIPES["session-runner"]["sha256"]
    assert hashlib.sha256(node.NODE.read_bytes()).hexdigest() == node.NODE_SHA
    result = subprocess.run([str(node.NODE), "--check", str(helper)], capture_output=True, timeout=30)
    assert result.returncode == 0, result.stderr.decode("utf-8", errors="replace")
    assert b"write: false" in helper.read_bytes() and b"entryPoints: [entry]" in helper.read_bytes()
    assert b"program.getSourceFiles()" in helper.read_bytes() and b"inputBeforeHashes" in helper.read_bytes()
    assert b"workerBuilt: false" in helper.read_bytes()


def test_runner_resolves_existing_pnpm_bindings_but_rejects_source_and_external_aliases(node):
    helper = node.COMPILE_HELPER_ROOT / node.COMPILE_RECIPES["session-runner"]["helper"]
    source = helper.read_text(encoding="utf-8")
    # Execute only the two input-identity functions with a synthetic filesystem;
    # no package load, TypeScript, bundle or source read occurs.
    functions = "\n".join(source[source.index("function " + name + "("):
        source.index("\n}", source.index("function " + name + "(")) + 2]
        for name in ("physical", "inputPath"))
    program = """
const path = require('node:path');
const deps = path.resolve('D:/cargo-targets/fixed-deps');
const packages = path.join(deps, 'node_modules');
const same = (a,b) => path.resolve(a).toLowerCase() === path.resolve(b).toLowerCase();
const inside = (p,r) => same(p,r) || path.resolve(p).toLowerCase().startsWith(path.resolve(r).toLowerCase()+path.sep);
const logical = path.join(packages,'typescript/lib/typescript.js');
const actual = path.join(packages,'.pnpm/typescript@6.0.3/node_modules/typescript/lib/typescript.js');
const source = path.resolve('E:/Git/ZirconEngine/source.ts');
const external = path.join(packages,'foreign/index.js');
const mappings = new Map([[logical,actual],[source,path.resolve('E:/cargo-targets/copied-source.ts')],
                         [external,path.resolve('C:/unreviewed.js')]]);
const fs = {existsSync: () => true, realpathSync:{native:p=>mappings.get(path.resolve(p)) || path.resolve(p)}};
""" + functions + """
if (inputPath(logical) !== actual) throw Error('pnpm read binding was not resolved');
for (const p of [source, external]) {
 let rejected = false; try { inputPath(p); } catch { rejected = true; }
 if (!rejected) throw Error('Foreign/source alias accepted');
}
console.log('dependency-binding-and-source-alias-contract-passed');
"""
    result = subprocess.run([str(node.NODE), "--input-type=commonjs", "-e", program],
                            capture_output=True, timeout=30)
    assert result.returncode == 0, result.stderr.decode("utf-8", errors="replace")
    assert b"dependency-binding-and-source-alias-contract-passed" in result.stdout


@pytest.mark.parametrize("identity", ["matching", "wrong-name", "wrong-version", "missing"])
def test_node_capture_reuses_only_the_fixed_existing_smol_alias_without_installation(node, candidate, identity):
    deps = candidate.output / "fixed-deps/node_modules"
    package = deps / ".pnpm/smol-toml@1.8.0/node_modules/smol-toml"
    if identity != "missing":
        package.mkdir(parents=True)
        manifest = {"name": "foreign" if identity == "wrong-name" else "smol-toml",
                    "version": "9.0.0" if identity == "wrong-version" else "1.8.0"}
        (package / "package.json").write_text(json.dumps(manifest), encoding="utf-8")
    with patch.object(node, "DEPS", deps), patch.object(node, "_resolve_package", return_value=None):
        if identity == "matching":
            assert node._seed_package("smol-toml") == (package, package)
            assert node._seed_package("typescript") is None
        else:
            with pytest.raises(ValueError): node._seed_package("smol-toml")
