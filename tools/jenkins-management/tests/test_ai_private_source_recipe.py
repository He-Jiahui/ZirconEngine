"""Private AI r03 filters and read-only fixtures; no product commands here."""
from copy import deepcopy

import pytest

from test_managed_node import candidate, node


@pytest.fixture(params=["penpot-ai-private-source-tests-r03", "penpot-ai-source-export-tests-r03"])
def ai_candidate(node, candidate, request):
    rule = node.RECIPES[request.param]
    candidate.config["test"]["include"] = list(rule["configSpecs"])
    candidate.config["test"].pop("testNamePattern")
    candidate.config["cacheDir"] = str(candidate.output / "zircon-local/cache/penpot-unreal-01a0f04d/vitest-focused")
    for name in rule["configSpecs"]:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic fixture, no product source copy\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["dependencyHashes"] = {}
    for name in rule["readOnlyInputs"]:
        path = candidate.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic read-only fixture\n", encoding="utf-8")
        candidate.payload["dependencyHashes"][name] = node.file_hash(path)
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": request.param,
                                      "config": candidate.write_config()}
    return rule


def test_ai_r03_uses_original_exact_cli_filters_and_one_worker(node, candidate, ai_candidate):
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path),
                               *ai_candidate["specs"], "-t", ai_candidate["pattern"]]
    assert plan["timeoutSeconds"] == ai_candidate["timeout"]
    assert plan["rule"]["expectedPassed"] in (21, 4)
    assert plan["configuration"]["test"]["include"] == list(node.AI_PRIVATE_SPECS + node.AI_EXPORT_SPECS)
    assert plan["cwd"] == str(candidate.app) and plan["compileKind"] is None and not plan["worker"]


@pytest.mark.parametrize("change", ["extra-spec", "missing-spec", "old-public8", "pattern", "watch",
    "parallel", "coverage", "workers", "command", "cache", "output", "missing-source", "unknown-selector"])
def test_ai_r03_rejects_test_or_output_scope_expansion(node, candidate, ai_candidate, change):
    test = candidate.config["test"]
    if change == "extra-spec": test["include"].append("src/unreviewed.spec.ts")
    elif change == "missing-spec": test["include"].pop()
    elif change == "old-public8": test["include"] = list(node.PUBLIC_SPECS)
    elif change == "pattern": test["testNamePattern"] = ".*"
    elif change == "watch": test["watch"] = True
    elif change == "parallel": test["fileParallelism"] = True
    elif change == "coverage": test["coverage"]["enabled"] = True
    elif change == "workers": test["maxWorkers"] = 2
    elif change == "command": candidate.payload["nodeRecipe"]["command"] = ["node", "arbitrary.mjs"]
    elif change == "cache": candidate.config["cacheDir"] = str(candidate.output / "foreign-cache")
    elif change == "output": test["outputFile"]["json"] = str(candidate.repo / "result.json")
    elif change == "missing-source": candidate.payload["sourceHashes"].pop(
        (candidate.app / node.AI_EXPORT_SPECS[0]).relative_to(candidate.repo).as_posix())
    else: candidate.payload["nodeRecipe"]["id"] = "ai-unbounded"
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("index", range(4))
@pytest.mark.parametrize("change", ["missing", "source-not-dependency", "owned-file", "owned-parent"])
def test_ai_r03_requires_all_four_read_only_inputs_without_write_scope(node, candidate, ai_candidate, index, change):
    name = node.AI_PRIVATE_READ_INPUTS[index]
    if change in ("missing", "source-not-dependency"):
        value = candidate.payload["dependencyHashes"].pop(name)
        if change == "source-not-dependency": candidate.payload["sourceHashes"][name] = value
    else:
        candidate.payload["ownedPaths"].append(name if change == "owned-file" else name.rsplit("/", 1)[0])
    with pytest.raises(ValueError, match="read-only dependency"):
        node.recipe(candidate.payload, candidate.repo)


def actual_result(node, candidate, rule):
    count = rule["expectedPassed"]
    rows = [{"name": str(candidate.app / name), "assertionResults": []} for name in rule["specs"]]
    for index in range(count):
        rows[index % len(rows)]["assertionResults"].append({"status": "passed"})
    return {"success": True, "testResults": rows}


def test_ai_r03_requires_actual_21_or_4_discovery(node, candidate, ai_candidate):
    plan = node.recipe(candidate.payload, candidate.repo)
    details = node.verify_test_result(plan, actual_result(node, candidate, ai_candidate))
    assert details["passed"] == ai_candidate["expectedPassed"] and details["failed"] == 0


@pytest.mark.parametrize("change", ["old13", "one-less", "extra", "duplicate-spec", "missing-spec",
                                   "empty-spec", "failed", "skipped", "no-success"])
def test_ai_r03_rejects_missing_duplicate_or_unexecuted_results(node, candidate, ai_candidate, change):
    plan = node.recipe(candidate.payload, candidate.repo)
    result = actual_result(node, candidate, ai_candidate)
    if change == "old13":
        result = actual_result(node, candidate, {**ai_candidate, "expectedPassed": 13})
    elif change == "one-less": result["testResults"][0]["assertionResults"].pop()
    elif change == "extra": result["testResults"][0]["assertionResults"].append({"status": "passed"})
    elif change == "duplicate-spec": result["testResults"].append(deepcopy(result["testResults"][0]))
    elif change == "missing-spec": result["testResults"].pop()
    elif change == "empty-spec": result["testResults"][0]["assertionResults"] = []
    elif change == "no-success": result["success"] = False
    else: result["testResults"][0]["assertionResults"][0]["status"] = change
    with pytest.raises(ValueError):
        node.verify_test_result(plan, result)
