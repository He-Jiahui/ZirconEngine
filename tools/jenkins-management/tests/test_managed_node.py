"""Focused Node management contracts; no product Node/Cargo execution here."""
from __future__ import annotations

from contextlib import nullcontext
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
from unittest.mock import patch

import pytest


ROOT = Path(__file__).resolve().parents[1]


def load(name):
    spec = importlib.util.spec_from_file_location("managed_node_test_" + name, ROOT / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@pytest.fixture
def node():
    return load("managed_node")


@pytest.fixture
def candidate(tmp_path, node):
    # tmp_path is provisioned by the Jenkins harness under its registered E root.
    assert tmp_path.is_relative_to(Path("E:/cargo-targets"))
    repo, output = tmp_path / "repo", tmp_path / "output"
    app = repo / "tools/penpot/apps/zircon-zui-plugin"
    public = output / "public"
    public.mkdir(parents=True)
    spec_path = app / "src/penpot-control-content.spec.ts"
    spec_path.parent.mkdir(parents=True)
    spec_path.write_text("// synthetic fixture; no engine source copy\n", encoding="utf-8")
    config = {"root": str(app), "cacheDir": str(public / "cache"),
              "resolve": {"alias": {"vitest": str(node.VITEST.parent / "dist/index.js"),
                "smol-toml": str(node.DEPS / ".pnpm/smol-toml@1.8.0/node_modules/smol-toml/dist/index.js"),
                "playwright": str(node.PLAYWRIGHT), "feather-icons/dist/icons.json": str(node.DEPS / "feather-icons/dist/icons.json")}},
              "test": {"watch": False, "globals": True, "environment": "node", "maxWorkers": 1,
                       "fileParallelism": False, "include": ["src/penpot-control-content.spec.ts"],
                       "reporters": ["default", "json"], "coverage": {"enabled": False},
                       "testTimeout": 15000, "pool": "forks", "outputFile": {"json": str(public / "result.json")},
                       "testNamePattern": node.TREE_PATTERN}}
    config_path = public / "focused.config.mjs"
    def write_config():
        config_path.write_text("export default " + json.dumps(config) + ";\n", encoding="utf-8")
        return {"path": str(config_path), "sha256": node.file_hash(config_path)}
    payload = {"action": "prepare-node", "repositoryId": "a" * 32, "sessionId": "public-owner",
               "requestId": "public8-original", "operationId": "prepare-review-r1", "driverDigest": "b" * 64,
               "runtimeOperationId": "start-existing", "ownedPaths": [spec_path.relative_to(repo).as_posix()],
               "sourceHashes": {spec_path.relative_to(repo).as_posix(): node.file_hash(spec_path)},
               "nodeRecipe": {"schemaVersion": 1, "id": "penpot-character-tree-row-r1", "config": write_config()}}
    with patch.multiple(node, REPOSITORY=repo, APP=app, OUTPUT_ROOT=output, PUBLIC=public):
        yield SimpleNamespace(repo=repo, app=app, output=output, public=public, config=config,
                              config_path=config_path, payload=payload, write_config=write_config)


def test_exact31_recipe_has_distinct_action_and_exact_argv(node, candidate):
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["cwd"] == str(candidate.app) and plan["timeoutSeconds"] == 120
    assert plan["rule"]["expectedPassed"] == 31 and plan["worker"] is False


def character_table_png_candidate(node, candidate):
    candidate.config["test"].update(include=list(node.CHARACTER_TABLE_PNG_SPECS),
                                    testNamePattern=node.CHARACTER_TABLE_PNG_PATTERN)
    for name in node.CHARACTER_TABLE_PNG_SPECS:
        path = candidate.app / name
        path.write_text("// synthetic management fixture; not a product source copy\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-character-table-png-tests-r02",
                                      "config": candidate.write_config()}
    return node.recipe(candidate.payload, candidate.repo)


def character_table_png_result(node, candidate):
    return {"success": True, "testResults": [
        {"name": str(candidate.app / name), "assertionResults": [{"status": "passed"} for _ in range(count)]}
        for name, count in zip(node.CHARACTER_TABLE_PNG_SPECS, (31, 8, 7))]}


def test_character_table_png_r02_keeps_exact_three_specs_filter_46_and_no_compile(node, candidate):
    plan = character_table_png_candidate(node, candidate)
    assert plan["rule"]["specs"] == list(node.CHARACTER_TABLE_PNG_SPECS)
    assert plan["configuration"]["test"]["testNamePattern"] == node.CHARACTER_TABLE_PNG_PATTERN
    assert plan["timeoutSeconds"] == 120 and plan["rule"]["expectedPassed"] == 46
    assert plan["compileKind"] is None and plan["worker"] is False and plan["builtin"] is False
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["cwd"] == str(candidate.app)
    counts = node.verify_test_result(plan, character_table_png_result(node, candidate))
    assert counts["passed"] == 46 and counts["failed"] == counts["skipped"] == 0
    assert {row["spec"]: row["passed"] for row in counts["perSpec"]} == dict(zip(node.CHARACTER_TABLE_PNG_SPECS, (31, 8, 7)))
    old = node.RECIPES["penpot-character-tree-row-r1"]
    assert old["specs"] == ["src/penpot-control-content.spec.ts"] and old["expectedPassed"] == 31
    assert old["pattern"] == node.TREE_PATTERN


@pytest.mark.parametrize("change", ["old-one-spec", "extra-spec", "missing-spec", "old-pattern", "wide-pattern",
    "missing-owner-hash", "watch", "workers", "coverage", "file-parallelism", "other-root", "foreign-output",
    "cargo-field", "shell-field"])
def test_character_table_png_r02_rejects_unreviewed_source_and_expanded_command_scope(node, candidate, change):
    character_table_png_candidate(node, candidate)
    test = candidate.config["test"]
    if change == "old-one-spec": test["include"] = [node.CHARACTER_TABLE_PNG_SPECS[0]]
    elif change == "extra-spec": test["include"].append("src/unrelated.spec.ts")
    elif change == "missing-spec": test["include"].pop()
    elif change == "old-pattern": test["testNamePattern"] = node.TREE_PATTERN
    elif change == "wide-pattern": test["testNamePattern"] = ".*"
    elif change == "missing-owner-hash":
        candidate.payload["sourceHashes"].pop((candidate.app / node.CHARACTER_TABLE_PNG_SPECS[1]).relative_to(candidate.repo).as_posix())
    elif change == "watch": test["watch"] = True
    elif change == "workers": test["maxWorkers"] = 2
    elif change == "coverage": test["coverage"]["enabled"] = True
    elif change == "file-parallelism": test["fileParallelism"] = True
    elif change == "other-root": candidate.config["root"] = str(candidate.repo)
    elif change == "foreign-output": test["outputFile"]["json"] = str(candidate.repo / "foreign.json")
    elif change == "cargo-field": candidate.payload["packages"] = ["zircon_editor"]
    else: candidate.payload["command"] = ["cmd.exe", "/c", "anything"]
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("change", ["missing-spec", "extra-spec", "duplicate-spec", "swapped-spec-counts",
    "missing-case", "extra-case", "balanced-counts", "failed-case", "skipped-case", "empty-table", "not-success"])
def test_character_table_png_r02_requires_actual_31_8_7_discovery(node, candidate, change):
    plan = character_table_png_candidate(node, candidate)
    result = character_table_png_result(node, candidate)
    rows = result["testResults"]
    if change == "missing-spec": rows.pop()
    elif change == "extra-spec": rows.append({"name": str(candidate.app / "src/unrelated.spec.ts"), "assertionResults": [{"status": "passed"}]})
    elif change == "duplicate-spec": rows.append(dict(rows[1]))
    elif change == "swapped-spec-counts": rows[1]["assertionResults"], rows[2]["assertionResults"] = rows[2]["assertionResults"], rows[1]["assertionResults"]
    elif change == "missing-case": rows[1]["assertionResults"].pop()
    elif change == "extra-case": rows[1]["assertionResults"].append({"status": "passed"})
    elif change == "balanced-counts": rows[2]["assertionResults"].append(rows[1]["assertionResults"].pop())
    elif change == "failed-case": rows[1]["assertionResults"][0]["status"] = "failed"
    elif change == "skipped-case": rows[1]["assertionResults"][0]["status"] = "pending"
    elif change == "empty-table": rows[1]["assertionResults"] = []
    else: result["success"] = False
    with pytest.raises(ValueError):
        node.verify_test_result(plan, result)


def test_character_table_png_r02_retains_actual_filtered_out_case_counts(node, candidate):
    plan = character_table_png_candidate(node, candidate)
    result = character_table_png_result(node, candidate)
    result["testResults"][1]["assertionResults"].append({"status": "pending"})
    counts = node.verify_test_result(plan, result)
    assert counts["passed"] == 46 and counts["skipped"] == 1
    assert next(row for row in counts["perSpec"] if row["spec"].endswith("penpot-table-controls.spec.ts"))["skipped"] == 1


def actor_layout_candidate(node, candidate):
    candidate.config["test"]["include"] = list(node.ACTOR_LAYOUT_SPECS)
    candidate.config["test"].pop("testNamePattern")
    for name in node.ACTOR_LAYOUT_SPECS:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic spec; no product Node execution\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-actor-content-allocation-tests-r02",
                                      "config": candidate.write_config()}


@pytest.mark.parametrize("compile_candidate", ["layout-worker"], indirect=True)
@pytest.mark.parametrize("strict", [None, False, "true"])
def test_layout_worker_cannot_accept_noemit_without_actual_strict_typecheck(node, compile_candidate, strict):
    plan = compile_candidate.plan()
    receipt = Path(plan["outputs"]["compileReceipt"])
    receipt.parent.mkdir(parents=True, exist_ok=True)
    receipt.write_text(json.dumps({"status": "compiled-current", "typescriptNoEmitPassed": True,
        "sourceUnchanged": True, "sourceCopiesCreated": False, "typescriptStrictPassed": strict}), encoding="utf-8")
    with pytest.raises(ValueError, match="strict TypeScript"):
        node.verify_compile_result(plan, {"files": {}})


def test_actor_allocation_r02_recipe_keeps_nine_specs_and_existing_tool_identity(node, candidate):
    actor_layout_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["rule"]["specs"] == list(node.ACTOR_LAYOUT_SPECS)
    assert len(plan["rule"]["specs"]) == 9 and plan["rule"]["requireEachSpec"] is True
    assert plan["rule"]["specs"][-1] == "src/penpot-export-guard.spec.ts"
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["compileKind"] is None and plan["worker"] is False


def test_actor_allocation_superseded_r01_recipe_cannot_run(node, candidate):
    actor_layout_candidate(node, candidate)
    candidate.payload["nodeRecipe"]["id"] = "penpot-actor-content-allocation-tests-r01"
    with pytest.raises(ValueError, match="Unknown or expanded Node recipe"):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("change", ["extra", "missing", "other", "pattern", "missing-owner-hash", "watch", "workers"])
def test_actor_allocation_recipe_rejects_scope_or_source_contract_changes(node, candidate, change):
    actor_layout_candidate(node, candidate)
    test = candidate.config["test"]
    if change == "extra": test["include"].append("src/unrelated.spec.ts")
    elif change == "missing": test["include"].pop()
    elif change == "other": test["include"][0] = "src/unrelated.spec.ts"
    elif change == "pattern": test["testNamePattern"] = "another domain"
    elif change == "missing-owner-hash":
        candidate.payload["sourceHashes"].pop((candidate.app / node.ACTOR_LAYOUT_SPECS[0]).relative_to(candidate.repo).as_posix())
    elif change == "watch": test["watch"] = True
    else: test["maxWorkers"] = 2
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("change", [None, "missing", "extra", "failed", "empty", "all-skipped"])
def test_actor_allocation_requires_actual_passing_discovery_from_every_selected_spec(node, candidate, change):
    actor_layout_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    result = {"success": True, "testResults": [
        {"name": str(candidate.app / spec), "assertionResults": [{"status": "passed"}]}
        for spec in node.ACTOR_LAYOUT_SPECS]}
    if change == "missing": result["testResults"].pop()
    elif change == "extra": result["testResults"].append({"name": str(candidate.app / "src/other.spec.ts"), "assertionResults": [{"status": "passed"}]})
    elif change == "failed": result["testResults"][0]["assertionResults"][0]["status"] = "failed"
    elif change == "empty": result["testResults"][0]["assertionResults"] = []
    elif change == "all-skipped": result["testResults"][0]["assertionResults"][0]["status"] = "pending"
    if change is None:
        checked = node.verify_test_result(plan, result)
        assert checked["passed"] == 9 and checked["testsExecuted"] is True
    else:
        with pytest.raises(ValueError):
            node.verify_test_result(plan, result)


def tree_property_candidate(node, candidate):
    candidate.config["test"]["include"] = list(node.TREE_PROPERTY_SPECS)
    candidate.config["test"].pop("testNamePattern")
    for name in node.TREE_PROPERTY_SPECS:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic spec; no product Node execution\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-tree-property-consumers-r02",
                                      "config": candidate.write_config()}


def tree_property_result(node, candidate):
    return {"success": True, "testResults": [
        {"name": str(candidate.app / spec), "assertionResults": [{"status": "passed"}]}
        for spec in node.TREE_PROPERTY_SPECS]}


def test_tree_property_recipe_runs_only_the_three_owner_specs(node, candidate):
    tree_property_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["rule"]["specs"] == list(node.TREE_PROPERTY_SPECS)
    assert plan["rule"]["requireEachSpec"] is True and plan["timeoutSeconds"] == 120
    assert plan["cwd"] == str(candidate.app) and set(plan["outputs"]) == {"cache", "result"}
    assert plan["worker"] is False and plan["builtin"] is False and plan["compileKind"] is None
    result = node.verify_test_result(plan, tree_property_result(node, candidate))
    assert result["passed"] == 3 and result["failed"] == result["skipped"] == 0
    assert len(result["specs"]) == 3 and result["testsExecuted"] is True


@pytest.mark.parametrize("change", ["old-two-specs", "extra-spec", "watch", "coverage", "workers",
                                    "filter", "missing-source", "foreign-output"])
def test_tree_property_recipe_rejects_old_scope_and_execution_expansion(node, candidate, change):
    tree_property_candidate(node, candidate)
    test = candidate.config["test"]
    if change == "old-two-specs": test["include"] = list(node.TREE_PROPERTY_SPECS[1:])
    elif change == "extra-spec": test["include"].append("src/unrelated.spec.ts")
    elif change == "watch": test["watch"] = True
    elif change == "coverage": test["coverage"]["enabled"] = True
    elif change == "workers": test["maxWorkers"] = 2
    elif change == "filter": test["testNamePattern"] = "only-one-case"
    elif change == "foreign-output": test["outputFile"]["json"] = str(candidate.repo / "target/result.json")
    else:
        candidate.payload["sourceHashes"].pop(
            (candidate.app / node.TREE_PROPERTY_SPECS[0]).relative_to(candidate.repo).as_posix())
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("change", ["missing-spec", "empty-spec", "skipped-spec", "failed-spec"])
def test_tree_property_discovery_requires_actual_passes_from_every_spec(node, candidate, change):
    tree_property_candidate(node, candidate)
    result = tree_property_result(node, candidate)
    if change == "missing-spec": result["testResults"].pop()
    elif change == "empty-spec": result["testResults"][0]["assertionResults"] = []
    else:
        result["testResults"][0]["assertionResults"][0]["status"] = (
            "pending" if change == "skipped-spec" else "failed")
    with pytest.raises(ValueError):
        node.verify_test_result(node.recipe(candidate.payload, candidate.repo), result)


def settings_property_candidate(node, candidate):
    name = "tools/zui-layout-dependencies.spec.ts"
    spec = candidate.app / name
    spec.parent.mkdir(parents=True, exist_ok=True)
    spec.write_text("// synthetic spec; no product Node execution\n", encoding="utf-8")
    candidate.payload["sourceHashes"][spec.relative_to(candidate.repo).as_posix()] = node.file_hash(spec)
    candidate.config["test"]["include"] = [name]
    candidate.config["test"]["testNamePattern"] = "original PropertyEditorRow height variants and typed value slot"
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-settings-property-consumers-r01",
                                      "config": candidate.write_config()}


def settings_property_result(candidate):
    return {"success": True, "testResults": [
        {"name": str(candidate.app / "tools/zui-layout-dependencies.spec.ts"),
         "assertionResults": [{"status": "passed"} for _ in range(4)]}]}


def test_settings_property_recipe_uses_exact_tools_spec_and_name_filter(node, candidate):
    settings_property_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["rule"]["specs"] == ["tools/zui-layout-dependencies.spec.ts"]
    assert plan["configuration"]["test"]["testNamePattern"] == (
        "original PropertyEditorRow height variants and typed value slot")
    assert plan["cwd"] == str(candidate.app) and plan["timeoutSeconds"] == 120
    assert plan["worker"] is False and plan["builtin"] is False and plan["compileKind"] is None
    assert set(plan["outputs"]) == {"cache", "result"}
    result = node.verify_test_result(plan, settings_property_result(candidate))
    assert result["passed"] == 4 and result["failed"] == result["skipped"] == 0


@pytest.mark.parametrize("change", ["old-src-spec", "extra-spec", "watch", "coverage", "workers",
                                    "filter", "missing-source", "foreign-output"])
def test_settings_property_recipe_rejects_different_or_expanded_scope(node, candidate, change):
    settings_property_candidate(node, candidate)
    test = candidate.config["test"]
    if change == "old-src-spec": test["include"] = ["src/penpot-control-content.spec.ts"]
    elif change == "extra-spec": test["include"].append("tools/unrelated.spec.ts")
    elif change == "watch": test["watch"] = True
    elif change == "coverage": test["coverage"]["enabled"] = True
    elif change == "workers": test["maxWorkers"] = 2
    elif change == "filter": test["testNamePattern"] = "PropertyEditorRow"
    elif change == "foreign-output": test["outputFile"]["json"] = str(candidate.repo / "target/result.json")
    else:
        candidate.payload["sourceHashes"].pop(
            (candidate.app / "tools/zui-layout-dependencies.spec.ts").relative_to(candidate.repo).as_posix())
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("change", ["zero", "too-few", "too-many", "wrong-spec", "failed", "skipped"])
def test_settings_property_discovery_requires_four_actual_passes_from_the_tools_spec(node, candidate, change):
    settings_property_candidate(node, candidate)
    result = settings_property_result(candidate)
    row = result["testResults"][0]
    if change == "zero": row["assertionResults"] = []
    elif change == "too-few": row["assertionResults"].pop()
    elif change == "too-many": row["assertionResults"].append({"status": "passed"})
    elif change == "wrong-spec": row["name"] = str(candidate.app / "src/penpot-control-content.spec.ts")
    else: row["assertionResults"][0]["status"] = "failed" if change == "failed" else "pending"
    with pytest.raises(ValueError):
        node.verify_test_result(node.recipe(candidate.payload, candidate.repo), result)


def builtin_candidate(node, candidate):
    for name in node.REAL_UI_NODE_SOURCES:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic source; no product Node execution\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    config = {"repo": str(candidate.repo), "spec": node.REAL_UI_NODE_SOURCES[0],
              "outputFile": str(candidate.public / "real-ui-node-tests.json")}
    candidate.config_path.write_text(json.dumps(config), encoding="utf-8")
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-real-ui-node-tests-r02",
        "config": {"path": str(candidate.config_path), "sha256": node.file_hash(candidate.config_path)}}
    return config


def test_real_ui_builtin_has_one_fixed_spec_without_worker_or_browser(node, candidate):
    builtin_candidate(node, candidate)
    plan = node.recipe(candidate.payload, candidate.repo)
    assert plan["command"] == [str(node.NODE), "--test", "--test-reporter=tap",
                               str(candidate.app / node.REAL_UI_NODE_SOURCES[0])]
    assert plan["builtin"] is True and not plan["worker"] and plan["compileKind"] is None
    assert plan["cwd"] == str(candidate.repo) and plan["timeoutSeconds"] == 120
    assert plan["rule"]["expectedPassed"] == 14 and set(plan["outputs"]) == {"result"}


@pytest.mark.parametrize("change", ["spec", "cwd", "command", "profile", "output", "missing-source", "config-change"])
def test_real_ui_builtin_rejects_scope_expansion_and_stale_configuration(node, candidate, change):
    config = builtin_candidate(node, candidate)
    if change == "spec": config["spec"] = "tools/unrelated.test.mjs"
    elif change == "cwd": config["repo"] = str(candidate.app)
    elif change == "command": config["command"] = ["node", "unreviewed.js"]
    elif change == "profile": config["profile"] = "full"
    elif change == "output": config["outputFile"] = str(candidate.repo / "target/result.json")
    elif change == "missing-source":
        candidate.payload["sourceHashes"].pop((candidate.app / node.REAL_UI_NODE_SOURCES[-1]).relative_to(candidate.repo).as_posix())
    candidate.config_path.write_text(json.dumps(config), encoding="utf-8")
    if change != "config-change":
        candidate.payload["nodeRecipe"]["config"]["sha256"] = node.file_hash(candidate.config_path)
    else:
        candidate.config_path.write_text(json.dumps({**config, "extra": True}), encoding="utf-8")
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


def builtin_tap():
    return ("TAP version 13\n" + "\n".join("ok " + str(i) + " - actual-case-" + str(i) for i in range(1, 15))
            + "\n1..14\n# tests 14\n# pass 14\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n")


def test_real_ui_builtin_requires_actual_complete_discovery(node, candidate):
    builtin_candidate(node, candidate)
    details = node.verify_builtin_result(node.recipe(candidate.payload, candidate.repo), builtin_tap().encode())
    assert details["passed"] == 14 and details["testsExecuted"] is True
    assert len(details["testNames"]) == 14 and details["headlessExecuted"] is False


@pytest.mark.parametrize("change", ["zero", "failure", "skip", "cancel", "todo", "missing-total",
                                    "duplicate-total", "missing-case", "wrong-plan", "duplicate-case",
                                    "duplicate-name", "missing-header"])
def test_real_ui_builtin_never_accepts_empty_skipped_or_ambiguous_tap(node, candidate, change):
    builtin_candidate(node, candidate)
    text = builtin_tap()
    if change == "zero": text = text.replace("# tests 14", "# tests 0").replace("# pass 14", "# pass 0")
    elif change == "failure": text = text.replace("# fail 0", "# fail 1")
    elif change == "skip": text = text.replace("# skipped 0", "# skipped 1")
    elif change == "cancel": text = text.replace("# cancelled 0", "# cancelled 1")
    elif change == "todo": text = text.replace("# todo 0", "# todo 1")
    elif change == "missing-total": text = text.replace("# fail 0\n", "")
    elif change == "duplicate-total": text += "# tests 14\n"
    elif change == "missing-case": text = text.replace("ok 14 - actual-case-14\n", "")
    elif change == "wrong-plan": text = text.replace("1..14", "1..15")
    elif change == "duplicate-case": text = text.replace("ok 14 - actual-case-14", "ok 13 - actual-case-14")
    elif change == "duplicate-name": text = text.replace("actual-case-14", "actual-case-13")
    else: text = text.replace("TAP version 13\n", "")
    with pytest.raises(ValueError):
        node.verify_builtin_result(node.recipe(candidate.payload, candidate.repo), text.encode())


def search_candidate(node, candidate):
    candidate.config["test"]["include"] = list(node.SEARCH_SPECS)
    candidate.config["test"].pop("testNamePattern")
    for name in node.SEARCH_SPECS:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic selected spec\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-search-native-fill-tests-r1",
                                        "config": candidate.write_config()}
    return node.recipe(candidate.payload, candidate.repo)


def test_search_recipe_runs_exact_nine_specs_and_requires_real_discovery(node, candidate):
    plan = search_candidate(node, candidate)
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["timeoutSeconds"] == 180 and len(plan["rule"]["specs"]) == 9
    result = {"success": True, "testResults": [{"name": str(candidate.app / name),
              "assertionResults": [{"status": "passed"}]} for name in node.SEARCH_SPECS]}
    assert node.verify_test_result(plan, result)["passed"] == 9
    result["testResults"][0]["assertionResults"] = []
    with pytest.raises(ValueError, match="Every selected spec"):
        node.verify_test_result(plan, result)


@pytest.mark.parametrize("change", ["extra-spec", "omitted-spec", "filter", "unreviewed-spec"])
def test_search_recipe_does_not_expand_or_silently_drop_owner_scope(node, candidate, change):
    search_candidate(node, candidate)
    if change == "extra-spec": candidate.config["test"]["include"].append("src/unrelated.spec.ts")
    elif change == "omitted-spec": candidate.config["test"]["include"].pop()
    elif change == "filter": candidate.config["test"]["testNamePattern"] = ".*"
    else: candidate.payload["sourceHashes"].pop((candidate.app / node.SEARCH_SPECS[-1]).relative_to(candidate.repo).as_posix())
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


def public_canonical_candidate(node, candidate):
    candidate.config["test"]["include"] = list(node.PUBLIC_CANONICAL_SPECS)
    candidate.config["test"].pop("testNamePattern")
    for name in node.PUBLIC_CANONICAL_SPECS:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic reviewed r03 spec; never executed\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-public-canonical-tests-r03",
                                        "config": candidate.write_config()}
    return node.recipe(candidate.payload, candidate.repo)


def test_public_canonical_r03_selects_exact_six_specs_without_changing_public8_or_tree31(node, candidate):
    plan = public_canonical_candidate(node, candidate)
    assert plan["rule"]["specs"] == [
        "tools/zui-layout-real-plugin-registration.spec.ts", "tools/penpot-canonical-library-publication.spec.ts",
        "src/penpot-canonical-library.spec.ts", "src/penpot-native-components.spec.ts",
        "src/penpot-control-content.spec.ts", "src/penpot-capture-validation.spec.ts"]
    assert plan["command"] == [str(node.NODE), str(node.VITEST), "run", "--config", str(candidate.config_path)]
    assert plan["cwd"] == str(candidate.app) and plan["timeoutSeconds"] == 180
    assert plan["configuration"]["test"]["maxWorkers"] == 1
    assert plan["configuration"]["test"]["watch"] is False
    assert plan["configuration"]["test"]["fileParallelism"] is False
    assert plan["configuration"]["test"]["coverage"] == {"enabled": False}
    assert node.RECIPES["penpot-public8-tests-r1"]["specs"] == [
        "src/penpot-canonical-library.spec.ts", "src/penpot-native-component-copy.spec.ts",
        "src/penpot-native-component-ownership.spec.ts", "src/bridge/zui-component-review-host.spec.ts"]
    assert node.RECIPES["penpot-character-tree-row-r1"]["expectedPassed"] == 31


@pytest.mark.parametrize("change", ["extra-spec", "omitted-spec", "old-public4", "search-nine", "filter",
    "unreviewed-spec", "watch", "coverage", "workers", "parallel", "hook", "alias", "output", "command"])
def test_public_canonical_r03_rejects_scope_controls_and_output_expansion(node, candidate, change):
    public_canonical_candidate(node, candidate)
    test = candidate.config["test"]
    if change == "extra-spec": test["include"].append("src/other.spec.ts")
    elif change == "omitted-spec": test["include"].pop()
    elif change == "old-public4": test["include"] = list(node.PUBLIC_SPECS)
    elif change == "search-nine": test["include"] = list(node.SEARCH_SPECS)
    elif change == "filter": test["testNamePattern"] = ".*"
    elif change == "unreviewed-spec": candidate.payload["sourceHashes"].pop(
        (candidate.app / node.PUBLIC_CANONICAL_SPECS[-1]).relative_to(candidate.repo).as_posix())
    elif change == "watch": test["watch"] = True
    elif change == "coverage": test["coverage"]["enabled"] = True
    elif change == "workers": test["maxWorkers"] = 2
    elif change == "parallel": test["fileParallelism"] = True
    elif change == "hook": test["globalSetup"] = "arbitrary.ts"
    elif change == "alias": candidate.config["resolve"]["alias"]["vitest"] = "C:/foreign/vitest.mjs"
    elif change == "output": candidate.config["cacheDir"] = str(candidate.repo / "target")
    else: candidate.payload["command"] = ["node", "arbitrary.js"]
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("change", [None, "empty-spec", "missing-spec", "duplicate-spec", "failed", "all-skipped"])
def test_public_canonical_r03_reports_actual_discovered_tests_and_unique_spec_results(node, candidate, change):
    plan = public_canonical_candidate(node, candidate)
    result = {"success": True, "testResults": [{"name": str(candidate.app / name),
        "assertionResults": [{"status": "passed"}, {"status": "passed"}]} for name in node.PUBLIC_CANONICAL_SPECS]}
    if change == "empty-spec": result["testResults"][0]["assertionResults"] = []
    elif change == "missing-spec": result["testResults"].pop()
    elif change == "duplicate-spec": result["testResults"].append(result["testResults"][0])
    elif change == "failed": result["testResults"][0]["assertionResults"][0]["status"] = "failed"
    elif change == "all-skipped":
        for row in result["testResults"]: row["assertionResults"] = [{"status": "pending"}]
    if change is None:
        actual = node.verify_test_result(plan, result)
        assert actual["passed"] == 12 and actual["failed"] == actual["skipped"] == 0
        assert actual["testsExecuted"] is True and len(actual["specs"]) == 6
    else:
        with pytest.raises(ValueError):
            node.verify_test_result(plan, result)


@pytest.fixture
def style_codec_candidate(node, candidate):
    entry = candidate.app / "src/plugin.ts"
    entry.write_text("// synthetic fixed worker; no product source copy\n", encoding="utf-8")
    for name in node.STYLE_CODEC_SPECS:
        path = candidate.app / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// synthetic style-codec spec; never executed\n", encoding="utf-8")
        candidate.payload["sourceHashes"][path.relative_to(candidate.repo).as_posix()] = node.file_hash(path)
    candidate.payload["sourceHashes"][entry.relative_to(candidate.repo).as_posix()] = node.file_hash(entry)
    config = {"repo": str(candidate.repo), "mainApp": str(candidate.app),
              "dependencyRoot": str(node.DEPS.parent), "out": str(candidate.output / "build"),
              "playwright": str(node.PLAYWRIGHT)}
    def write_config():
        candidate.config_path.write_text(json.dumps(config), encoding="utf-8")
        return {"path": str(candidate.config_path), "sha256": node.file_hash(candidate.config_path)}
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-source-style-codec-r01",
                                        "config": write_config()}
    with patch.multiple(node, COMPILE_OUTPUT_ROOT=candidate.output / "build",
                        COMPILE_REPORT_ROOT=candidate.public, COMPILE_HELPER_ROOT=candidate.public / "tmp"):
        yield SimpleNamespace(config=config, write_config=write_config,
                              plan=lambda: node.recipe(candidate.payload, candidate.repo))


def test_style_codec_selects_only_five_unfiltered_specs_and_one_worker(node, candidate, style_codec_candidate):
    plan = style_codec_candidate.plan()
    assert plan["rule"]["specs"] == ["src/penpot-capture-validation.spec.ts", "src/bridge/zui-source-export.spec.ts",
        "src/penpot-source-colors.spec.ts", "src/penpot-asset-renderer.spec.ts", "src/penpot-tree-row-controls.spec.ts"]
    assert plan["rule"]["pattern"] is None and plan["compileKind"] == "style-codec"
    assert plan["timeoutSeconds"] == 180 and plan["worker"] is False
    assert plan["command"] == [str(node.NODE), plan["compileHelper"], str(candidate.config_path)]
    assert set(plan["outputFiles"]) == {"bundle", "compileReceipt", "metafile", "testResult", "junit"}
    assert plan["outputs"]["bundle"].endswith("plugin-style-codec-r01.js")
    assert not Path(plan["compileHelper"]).exists()
    assert node.RECIPES["penpot-ai-source-export-tests-r03"]["expectedPassed"] == 4
    assert node.RECIPES["penpot-search-native-fill-tests-r1"]["specs"] == list(node.SEARCH_SPECS)


@pytest.mark.parametrize("change", ["missing-spec-pin", "missing-worker-pin", "extra-command", "other-app",
    "other-dependencies", "other-output", "extra-stage", "unprepared-validation"])
def test_style_codec_rejects_unreviewed_source_and_arbitrary_stages(node, candidate, style_codec_candidate, change):
    c = style_codec_candidate
    if change == "missing-spec-pin":
        candidate.payload["sourceHashes"].pop((candidate.app / node.STYLE_CODEC_SPECS[-1]).relative_to(candidate.repo).as_posix())
    elif change == "missing-worker-pin":
        candidate.payload["sourceHashes"].pop((candidate.app / "src/plugin.ts").relative_to(candidate.repo).as_posix())
    elif change == "extra-command": candidate.payload["command"] = ["node", "arbitrary.js"]
    elif change == "other-app": c.config["mainApp"] = str(candidate.repo / "other")
    elif change == "other-dependencies": c.config["dependencyRoot"] = "C:/foreign"
    elif change == "other-output": c.config["out"] = str(candidate.repo / "target")
    elif change == "extra-stage": c.config["stages"] = ["publisher", "runner"]
    else: candidate.payload["action"] = "validate-node"
    candidate.payload["nodeRecipe"]["config"] = c.write_config()
    with pytest.raises(ValueError): c.plan()


@pytest.mark.parametrize("changed", [False, True])
def test_style_codec_generation_is_fixed_registered_and_never_overwrites(node, candidate, style_codec_candidate, changed):
    plan = style_codec_candidate.plan()
    report = candidate.public / "management"
    report.mkdir()
    records = []
    def write_record(path, value):
        records.append((path, value))
        path.write_text(json.dumps(value), encoding="utf-8")
    m = SimpleNamespace(operation="original-style-op", write_record=write_record)
    if changed:
        target = node.style_codec_generated_inputs()[0]
        target.parent.mkdir(parents=True)
        target.write_text("foreign control", encoding="utf-8")
        with pytest.raises(ValueError, match="Retained style-codec"):
            node.materialize_style_codec_helper(m, plan, report)
        assert target.read_text() == "foreign control" and not Path(plan["compileHelper"]).exists()
        return
    node.materialize_style_codec_helper(m, plan, report)
    assert records[0][1]["operationId"] == "original-style-op"
    assert len(records[0][1]["inputs"]) == 3
    raw = node.style_codec_generated_inputs()[0].read_text()
    config = json.loads(raw[len("export default "):-2])
    assert config["test"]["include"] == list(node.STYLE_CODEC_SPECS)
    assert config["test"]["maxWorkers"] == 1 and config["test"]["fileParallelism"] is False
    assert config["test"]["watch"] is False and config["test"]["coverage"] == {"enabled": False}
    assert "testNamePattern" not in config["test"] and "globalSetup" not in config["test"]
    assert config["test"]["reporters"] == ["default", "json", "junit"]
    strict = json.loads(node.style_codec_generated_inputs()[1].read_bytes())
    assert strict["files"] == [str(candidate.app / name) for name in node.STYLE_CODEC_SPECS[:2]]
    assert strict["include"] == [] and strict["compilerOptions"] == {"strict": True, "noEmit": True, "incremental": False}
    assert node.file_hash(Path(plan["compileHelper"])) == node.COMPILE_RECIPES["style-codec"]["sha256"]
    before = {p: p.read_bytes() for p in node.style_codec_generated_inputs()}
    node.materialize_style_codec_helper(m, plan, report)
    assert all(p.read_bytes() == value for p, value in before.items())


@pytest.fixture
def style_codec_result(node, candidate, style_codec_candidate):
    import xml.etree.ElementTree as ET
    plan = style_codec_candidate.plan()
    outputs = {k: Path(v) for k, v in plan["outputs"].items()}
    for path in outputs.values(): path.parent.mkdir(parents=True, exist_ok=True)
    outputs["bundle"].write_bytes(b"// synthetic output; no Node executed\n")
    paths = [candidate.app / "src/plugin.ts", *[candidate.app / name for name in node.STYLE_CODEC_SPECS]]
    files = {str(p): node.file_hash(p) for p in paths}
    meta = {"inputs": {str(paths[0]): {}}, "outputs": {str(outputs["bundle"]): {"entryPoint": str(paths[0])}}}
    outputs["metafile"].write_text(json.dumps(meta), encoding="utf-8")
    tests = {"success": True, "testResults": [{"name": str(candidate.app / name),
        "assertionResults": [{"status": "passed"}]} for name in node.STYLE_CODEC_SPECS]}
    outputs["testResult"].write_text(json.dumps(tests), encoding="utf-8")
    xml = ET.Element("testsuites")
    for name in node.STYLE_CODEC_SPECS:
        ET.SubElement(ET.SubElement(xml, "testsuite", name=name), "testcase", name="synthetic case")
    outputs["junit"].write_bytes(ET.tostring(xml))
    roots = [[str(candidate.app / name) for name in node.STYLE_CODEC_SPECS[:2]], [str(paths[0])]]
    result = {"schema": "managed-source-style-codec/v1", "status": "compiled-current",
        "typescriptNoEmitPassed": True, "typescriptStrictPassed": True, "sourceUnchanged": True,
        "sourceCopiesCreated": False, "specs": list(node.STYLE_CODEC_SPECS), "entries": [str(paths[0])],
        "workerBuilt": True, "publisherBuilt": False, "runnerBuilt": False, "angularBuilt": False,
        "dependencyInstallation": False, "deployed": False, "inputBeforeHashes": files, "inputAfterHashes": files,
        "inputs": [{"path": p, "sha256": value} for p, value in files.items()],
        "programInputPaths": list(files), "metafileInputPaths": [str(paths[0])],
        "programs": [{"roots": value, "strict": True, "noEmit": True, "incremental": False, "diagnostics": 0}
                     for value in roots],
        "testResult": {"path": str(outputs["testResult"]), "sha256": node.file_hash(outputs["testResult"])},
        "output": {"path": str(outputs["bundle"]), "sha256": node.file_hash(outputs["bundle"])}}
    def write_result(): outputs["compileReceipt"].write_text(json.dumps(result), encoding="utf-8")
    write_result()
    return SimpleNamespace(plan=plan, outputs=outputs, result=result, tests=tests, xml=xml,
                           captured={"files": files}, write_result=write_result)


@pytest.mark.parametrize("change", [None, "failed-test", "skipped-test", "empty-spec", "duplicate-spec",
    "other-spec", "strict-false", "wrong-roots", "publisher", "stale-before", "missing-program-input",
    "junit-skipped", "junit-wrong-spec", "unguarded-input"])
def test_style_codec_requires_actual_tests_two_strict_programs_one_bundle_and_all_input_pins(
        node, candidate, style_codec_result, change):
    import xml.etree.ElementTree as ET
    c = style_codec_result
    if change == "failed-test": c.tests["testResults"][0]["assertionResults"][0]["status"] = "failed"
    elif change == "skipped-test": c.tests["testResults"][0]["assertionResults"].append({"status": "pending"})
    elif change == "empty-spec": c.tests["testResults"][0]["assertionResults"] = []
    elif change == "duplicate-spec": c.tests["testResults"].append(c.tests["testResults"][0])
    elif change == "other-spec": c.tests["testResults"][0]["name"] = str(candidate.app / "other.spec.ts")
    elif change == "strict-false": c.result["typescriptStrictPassed"] = False
    elif change == "wrong-roots": c.result["programs"][0]["roots"].append(str(candidate.app / "other.spec.ts"))
    elif change == "publisher": c.result["publisherBuilt"] = True
    elif change == "stale-before": c.result["inputBeforeHashes"] = {}
    elif change == "missing-program-input": c.result["programInputPaths"].remove(c.result["entries"][0])
    elif change == "junit-skipped": ET.SubElement(c.xml.find("testsuite/testcase"), "skipped")
    elif change == "junit-wrong-spec": c.xml.find("testsuite").set("name", "other.spec.ts")
    elif change == "unguarded-input": c.captured["files"] = {}
    c.outputs["testResult"].write_text(json.dumps(c.tests), encoding="utf-8")
    c.result["testResult"]["sha256"] = node.file_hash(c.outputs["testResult"])
    c.outputs["junit"].write_bytes(ET.tostring(c.xml))
    c.write_result()
    if change is None:
        actual = node.verify_compile_result(c.plan, c.captured)
        assert actual["testDetails"]["passed"] == 5 and actual["testDetails"]["skipped"] == 0
        assert actual["deployed"] is False and actual["engineRustExecuted"] is False
    else:
        with pytest.raises(ValueError): node.verify_compile_result(c.plan, c.captured)


@pytest.fixture(params=["publisher", "worker", "layout-worker"])
def compile_candidate(request, node, candidate):
    kind = request.param
    rules = {name: dict(rule) for name, rule in node.COMPILE_RECIPES.items()}
    helper_root, report_root, output_root = candidate.public / "helpers", candidate.public / "reports", candidate.output / "build"
    helper_root.mkdir()
    helper = helper_root / rules[kind]["helper"]
    helper.write_text("// synthetic reviewed helper; never executed\n", encoding="utf-8")
    rules[kind]["sha256"] = node.file_hash(helper)
    entry = candidate.app / rules[kind]["entry"]
    entry.parent.mkdir(parents=True, exist_ok=True)
    entry.write_text("// synthetic canonical entry\n", encoding="utf-8")
    candidate.payload["sourceHashes"][entry.relative_to(candidate.repo).as_posix()] = node.file_hash(entry)
    config = {"repo": str(candidate.repo), "mainApp": str(candidate.app), "dependencyRoot": str(node.DEPS.parent),
              "out": str(output_root), "playwright": str(node.PLAYWRIGHT)}
    def write_config():
        candidate.config_path.write_text(json.dumps(config), encoding="utf-8")
        candidate.payload["nodeRecipe"]["config"] = {"path": str(candidate.config_path), "sha256": node.file_hash(candidate.config_path)}
    selection = "penpot-canonical-layout-worker-r4" if kind == "layout-worker" else "penpot-canonical-" + kind + "-r1"
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": selection}
    write_config()
    with patch.multiple(node, COMPILE_RECIPES=rules, COMPILE_HELPER_ROOT=helper_root,
                        COMPILE_REPORT_ROOT=report_root, COMPILE_OUTPUT_ROOT=output_root):
        yield SimpleNamespace(kind=kind, helper=helper, entry=entry, config=config, write_config=write_config,
                              plan=lambda: node.recipe(candidate.payload, candidate.repo))


def test_canonical_compiles_use_reviewed_helpers_separate_from_legacy_worker(node, candidate, compile_candidate):
    c = compile_candidate
    plan = c.plan()
    assert plan["command"] == [str(node.NODE), str(c.helper), str(candidate.config_path)]
    assert plan["compileKind"] == c.kind and plan["worker"] is False
    assert plan["cwd"] == str(candidate.repo) and plan["timeoutSeconds"] == 180
    assert set(plan["outputFiles"]) == {"bundle", "compileReceipt", "metafile"}
    assert plan["configuration"] == c.config
    expected_entries = {"publisher": "tools/penpot-canonical-library-publication.ts",
                        "worker": "src/plugin.ts", "layout-worker": "src/plugin.ts"}
    assert c.entry == candidate.app / expected_entries[c.kind]
    assert c.entry != candidate.app / "tools/zui-layout-real-session.ts"


@pytest.mark.parametrize("change", ["root", "dependencies", "output", "playwright", "generic-control", "helper", "entry"])
def test_canonical_compile_contract_rejects_other_sources_tools_or_destinations(node, candidate, compile_candidate, change):
    c = compile_candidate
    if change == "root": c.config["mainApp"] = str(candidate.repo)
    elif change == "dependencies": c.config["dependencyRoot"] = "C:/unreviewed/node_modules"
    elif change == "output": c.config["out"] = str(candidate.repo / "target")
    elif change == "playwright": c.config["playwright"] = "C:/unreviewed/playwright.js"
    elif change == "generic-control": c.config["command"] = ["node", "arbitrary.js"]
    elif change == "helper": c.helper.write_text("// changed unreviewed helper\n", encoding="utf-8")
    else: candidate.payload["sourceHashes"].pop(c.entry.relative_to(candidate.repo).as_posix())
    c.write_config()
    with pytest.raises(ValueError):
        c.plan()


@pytest.fixture
def ui_compile_candidate(node, candidate):
    helper_root, report_root, output_root = candidate.public / "helpers", candidate.public / "reports", candidate.output / "ui-build"
    helper_root.mkdir()
    entry = candidate.app / "src/main.ts"
    entry.write_bytes(b"// synthetic single UI entry\n")
    candidate.payload["sourceHashes"][entry.relative_to(candidate.repo).as_posix()] = node.file_hash(entry)
    config = {"repo": str(candidate.repo), "mainApp": str(candidate.app), "dependencyRoot": str(node.DEPS.parent),
              "out": str(output_root), "playwright": str(node.PLAYWRIGHT)}
    candidate.payload["nodeRecipe"] = {"schemaVersion": 1, "id": "penpot-field-number-single-ui-aot-r01"}
    def write_config():
        candidate.config_path.write_text(json.dumps(config), encoding="utf-8")
        candidate.payload["nodeRecipe"]["config"] = {"path": str(candidate.config_path), "sha256": node.file_hash(candidate.config_path)}
    write_config()
    with patch.multiple(node, COMPILE_HELPER_ROOT=helper_root, COMPILE_REPORT_ROOT=report_root,
                        COMPILE_OUTPUT_ROOT=output_root):
        yield SimpleNamespace(entry=entry, config=config, write_config=write_config,
                              helper=helper_root / node.COMPILE_RECIPES["ui-aot"]["helper"],
                              plan=lambda: node.recipe(candidate.payload, candidate.repo))


def test_single_ui_recipe_is_pure_and_helper_generation_is_registered(node, candidate, ui_compile_candidate):
    c = ui_compile_candidate
    plan = c.plan()
    assert plan["compileKind"] == "ui-aot" and not c.helper.exists()
    assert plan["command"] == [str(node.NODE), str(c.helper), str(candidate.config_path)]
    assert plan["outputs"]["build"] == str(node.COMPILE_OUTPUT_ROOT / "ui-field-number-r01")
    records = []
    def write_record(path, value):
        records.append((path, value, c.helper.exists()))
    m = SimpleNamespace(operation="reviewed-single-ui", write_record=write_record)
    node.materialize_ui_helper(m, plan, candidate.public / "execution-report")
    assert len(records) == 1 and records[0][1]["reused"] is False and records[0][2] is False
    assert node.file_hash(c.helper) == node.COMPILE_RECIPES["ui-aot"]["sha256"]
    node.materialize_ui_helper(m, plan, candidate.public / "execution-report")
    assert records[-1][1]["reused"] is True
    c.helper.write_bytes(b"foreign helper must be preserved")
    with pytest.raises(ValueError):
        node.materialize_ui_helper(m, plan, candidate.public / "execution-report")
    assert c.helper.read_bytes() == b"foreign helper must be preserved"


@pytest.mark.parametrize("change", ["workspace", "output", "extra-command", "extra-target", "missing-entry", "foreign-helper"])
def test_single_ui_recipe_rejects_expanded_scope(node, candidate, ui_compile_candidate, change):
    c = ui_compile_candidate
    if change == "workspace": c.config["mainApp"] = str(candidate.repo)
    elif change == "output": c.config["out"] = "C:/unapproved-ui"
    elif change == "extra-command": c.config["command"] = ["node", "arbitrary.cjs"]
    elif change == "extra-target": c.config["project"] = "another-angular-app"
    elif change == "missing-entry": candidate.payload["sourceHashes"].pop(c.entry.relative_to(candidate.repo).as_posix())
    else: c.helper.write_bytes(b"foreign helper")
    c.write_config()
    with pytest.raises(ValueError):
        c.plan()


@pytest.mark.parametrize("change", [None, "no-aot", "watch", "worker", "publisher", "install", "no-strict",
                                    "no-typecheck", "source-drift", "unguarded-input", "extra-output"])
def test_single_ui_result_requires_strict_sources_and_complete_outputs(node, candidate, ui_compile_candidate, change):
    c = ui_compile_candidate
    plan = c.plan()
    directory = Path(plan["outputs"]["build"])
    directory.mkdir(parents=True)
    (directory / "index.html").write_bytes(b'<script src="main-fixture.js"></script>')
    (directory / "main-fixture.js").write_bytes(b"// synthetic emitted UI")
    (directory / "stats.json").write_text(json.dumps({"inputs": {str(c.entry): {}}}), encoding="utf-8")
    seal = {str(c.entry): node.file_hash(c.entry)}
    result = {"schema": "managed-single-angular-ui/v1", "status": "compiled-current",
              "project": "zircon-zui-plugin", "entry": str(c.entry), "aot": True, "watch": False,
              "typescriptNoEmitPassed": True, "typescriptStrictPassed": True, "sourceUnchanged": True,
              "workerBuilt": False, "publisherBuilt": False, "dependencyInstallation": False,
              "sourceCopiesCreated": False, "sourceBeforeHashes": seal, "sourceAfterHashes": dict(seal),
              "inputs": [{"path": str(c.entry), "sha256": node.file_hash(c.entry)}],
              "programInputPaths": [str(c.entry)], "statsInputPaths": [str(c.entry)],
              "outputs": [{"path": str(path), "sha256": node.file_hash(path), "bytes": path.stat().st_size}
                          for path in directory.iterdir()],
              "indexSha256": node.file_hash(directory / "index.html")}
    captured = {"files": dict(seal)}
    if change == "no-aot": result["aot"] = False
    elif change == "watch": result["watch"] = True
    elif change == "worker": result["workerBuilt"] = True
    elif change == "publisher": result["publisherBuilt"] = True
    elif change == "install": result["dependencyInstallation"] = True
    elif change == "no-strict": result["typescriptStrictPassed"] = False
    elif change == "no-typecheck": result["typescriptNoEmitPassed"] = False
    elif change == "source-drift": result["sourceAfterHashes"][str(c.entry)] = "0" * 64
    elif change == "unguarded-input": captured["files"] = {}
    elif change == "extra-output": (directory / "foreign.js").write_bytes(b"must not be omitted")
    if change is None:
        observed = node.verify_ui_compile_result(plan, captured, result)
        assert observed["passed"] is True and observed["profile"] == "production"
        assert observed["testsExecuted"] is observed["engineRustExecuted"] is False
    else:
        with pytest.raises(ValueError):
            node.verify_ui_compile_result(plan, captured, result)


def test_managed_generated_ui_helper_parses_with_pinned_node24(tmp_path, node):
    # Syntax only, inside the retained Jenkins pytest Job; no product imports/build.
    assert node.file_hash(node.NODE) == node.NODE_SHA
    helper = tmp_path / "managed-ui-aot.cjs"
    helper.write_bytes(node.UI_AOT_HELPER.encode())
    environment = {key: os.environ[key] for key in ("SYSTEMROOT", "WINDIR", "COMSPEC") if key in os.environ}
    environment.update({key: str(tmp_path) for key in ("TEMP", "TMP", "HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA")})
    result = subprocess.run([str(node.NODE), "--check", str(helper)], capture_output=True,
                            env=environment, timeout=30, creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode == 0, result.stderr.decode(errors="replace")


@pytest.mark.parametrize("change", [None, "no-typecheck", "unguarded-import", "missing-entry", "wrong-bundle", "wrong-metafile-entry", "extra-output"])
def test_compile_receipt_proves_typecheck_imports_entry_and_exact_bundle(node, candidate, compile_candidate, change):
    c = compile_candidate
    plan = c.plan()
    for path in plan["outputs"].values(): Path(path).parent.mkdir(parents=True, exist_ok=True)
    output = Path(plan["outputs"]["bundle"])
    output.write_bytes(b"synthetic qualified bundle; no source copy")
    extra = candidate.app / "dependency.ts"
    extra.write_bytes(b"// synthetic dependency")
    inputs = [{"path": str(path), "sha256": node.file_hash(path)} for path in (c.entry, extra)]
    captured = {"files": {row["path"]: row["sha256"] for row in inputs}}
    result = {"status": "compiled-current", "typescriptNoEmitPassed": True, "sourceUnchanged": True,
              "sourceCopiesCreated": False, "inputs": inputs,
              "output": {"path": str(output), "sha256": node.file_hash(output)}}
    if c.kind == "layout-worker":
        result["typescriptStrictPassed"] = True
    metafile = {"inputs": {str(c.entry): {}}, "outputs": {str(output): {"entryPoint": str(c.entry)}}}
    if change == "no-typecheck": result["typescriptNoEmitPassed"] = False
    elif change == "unguarded-import": captured["files"].pop(str(extra))
    elif change == "missing-entry": result["inputs"] = inputs[1:]
    elif change == "wrong-bundle": result["output"]["path"] = str(output.with_suffix(".other"))
    elif change == "wrong-metafile-entry": metafile["outputs"][str(output)]["entryPoint"] = str(extra)
    elif change == "extra-output": metafile["outputs"][str(output.with_suffix(".other"))] = {"entryPoint": str(c.entry)}
    Path(plan["outputs"]["compileReceipt"]).write_text(json.dumps(result), encoding="utf-8")
    Path(plan["outputs"]["metafile"]).write_text(json.dumps(metafile), encoding="utf-8")
    if change is None:
        actual = node.verify_compile_result(plan, captured)
        assert actual["deployed"] is False and actual["engineRustExecuted"] is False
        assert actual["typescriptNoEmitPassed"] is True
    else:
        with pytest.raises(ValueError):
            node.verify_compile_result(plan, captured)


@pytest.mark.parametrize("change", ["all-specs", "watch", "coverage", "workers", "filter", "code", "alias", "output", "hook"])
def test_rejects_scope_or_execution_expansion(node, candidate, change):
    cfg = candidate.config
    if change == "all-specs": cfg["test"]["include"] = ["**/*.spec.ts"]
    elif change == "watch": cfg["test"]["watch"] = True
    elif change == "coverage": cfg["test"]["coverage"]["enabled"] = True
    elif change == "workers": cfg["test"]["maxWorkers"] = True  # bool != integer budget
    elif change == "filter": cfg["test"]["testNamePattern"] = ".*"
    elif change == "alias": cfg["resolve"]["alias"]["vitest"] = "C:/foreign/vitest.js"
    elif change == "output": cfg["cacheDir"] = str(candidate.repo / "target")
    elif change == "hook": cfg["test"]["globalSetup"] = "arbitrary.ts"
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    if change == "code":
        candidate.config_path.write_text("import 'arbitrary'; export default {};", encoding="utf-8")
        candidate.payload["nodeRecipe"]["config"]["sha256"] = node.file_hash(candidate.config_path)
    with pytest.raises((ValueError, TypeError)):
        node.recipe(candidate.payload, candidate.repo)


@pytest.mark.parametrize("field,value", [("command", ["cmd.exe", "/c", "anything"]), ("features", ["all"]),
                                          ("recipe", {"verb": "test"}), ("sourceView", "snapshot")])
def test_identity_cannot_smuggle_generic_or_cargo_controls(node, candidate, field, value):
    candidate.payload[field] = value
    with pytest.raises(ValueError):
        node.recipe(candidate.payload, candidate.repo)


def test_stale_config_and_unreviewed_spec_are_rejected(node, candidate):
    candidate.config_path.write_text("export default {};", encoding="utf-8")
    with pytest.raises(ValueError, match="bytes changed"):
        node.recipe(candidate.payload, candidate.repo)
    candidate.payload["nodeRecipe"]["config"] = candidate.write_config()
    candidate.payload["sourceHashes"] = {"other-source.ts": "0" * 64}
    with pytest.raises(ValueError, match="current source-owner hash"):
        node.recipe(candidate.payload, candidate.repo)


def test_node_never_falls_through_cargo_and_preserves_managed_worktree_contract(node):
    entry, runner = load("entry"), load("runner")
    assert entry.MANAGED_WORKTREE_CAPABILITY == runner.MANAGED_WORKTREE_CAPABILITY
    assert not node.ACTIONS & set(entry.MANAGED_WORKTREE_CAPABILITY["actions"])
    for action in node.ACTIONS:
        assert entry.AUTH[action] == runner.AUTH[action] == "validation"
    with pytest.raises(entry.EntryError):
        entry.parse_cargo(["node", "vitest.mjs", "run"])


def test_discovery_proves31_passes_and_rejects_zero_wrong_spec_failed_or_other_count(node, candidate):
    plan = node.recipe(candidate.payload, candidate.repo)
    result = {"success": True, "testResults": [{"name": str(candidate.app / "src/penpot-control-content.spec.ts"),
               "assertionResults": [{"status": "passed"} for _ in range(31)] + [{"status": "pending"}]}]}
    assert node.verify_test_result(plan, result)["passed"] == 31
    for invalid in (dict(result, success=False), {"success": True, "testResults": []},
                    {"success": True, "testResults": [{"name": "foreign.spec.ts", "assertionResults": []}]},
                    {"success": True, "testResults": [{"name": result["testResults"][0]["name"],
                       "assertionResults": [{"status": "passed"} for _ in range(30)]}]}):
        with pytest.raises(ValueError):
            node.verify_test_result(plan, invalid)


def test_duplicate_keys_or_executable_json_are_not_configuration(node):
    for raw in ('{"root":1,"root":2}', '{"budget":NaN}', '{};process.exit(0)'):
        with pytest.raises(ValueError):
            node.unique_json(raw)


@pytest.mark.parametrize("profile", ["tree31", "public-canonical-r03"])
def test_completed_server_preparation_required_and_revision_bound(node, candidate, profile):
    if profile == "public-canonical-r03":
        public_canonical_candidate(node, candidate)
    key = "c" * 64
    path = candidate.output / "zircon-local/management-node/reports" / key / "receipt.json"
    path.parent.mkdir(parents=True)
    receipt = {"kind": "managed-node-preparation-v1", "status": "prepared", "operationKey": key,
               "sessionId": candidate.payload["sessionId"], "requestId": candidate.payload["requestId"],
               "repositoryId": candidate.payload["repositoryId"], "driverDigest": candidate.payload["driverDigest"],
               "runtimeOperationId": candidate.payload["runtimeOperationId"], "nodeRecipe": candidate.payload["nodeRecipe"],
               "inputsDigest": "same-inputs", "reservationReleased": True, "sourcePinsReleased": True}
    path.write_text(json.dumps(receipt), encoding="utf-8")
    ref = {"path": str(path), "sha256": node.file_hash(path)}
    payload = {**candidate.payload, "action": "validate-node", "preparationRef": ref}
    row = {"payload": {"phase": "complete", "action": "prepare-node", "result": {"receipt": ref}}}
    m = SimpleNamespace(payload=payload, session=payload["sessionId"], request=payload["requestId"],
                        repo_id=payload["repositoryId"], config=payload,
                        state=SimpleNamespace(get=lambda *_args: row))
    node.verify_preparation(m, "same-inputs")
    with pytest.raises(ValueError, match="stale"):
        node.verify_preparation(m, "new-source")
    m.state.get = lambda *_args: None
    with pytest.raises(ValueError, match="actual completed server"):
        node.verify_preparation(m, "same-inputs")


@pytest.mark.parametrize("mode", ["prepare", "execute", "native-unknown", "capacity-wait"])
def test_preparation_execution_and_unknown_release_are_distinct(node, candidate, mode):
    class Records:
        def __init__(self):
            self.rows = {("resource_policy", "default"): {"version": 1, "payload": {
                "cpuBudget": 6, "memoryBudget": 12 * 1024**3, "diskBudget": 40 * 1024**3}}}
        def get(self, domain, key):
            return self.rows.get((domain, key))
        def put(self, domain, key, payload, expected_version=0):
            old = self.get(domain, key)
            assert (old["version"] if old else 0) == expected_version
            self.rows[(domain, key)] = {"version": expected_version + 1, "payload": payload}
    state, resources, guards = Records(), [], []
    class Budget:
        def __init__(self, *_args):
            self.released = False
            resources.append(self)
        def queue(self, *_args, **_kwargs): pass
        def scan_inventory(self, *_args): return {}, "fresh-inventory"
        def admit(self, owner, amount, **kwargs):
            assert amount.cpu == 1 and amount.disk_bytes == 1024**3 and kwargs["heavy_writer"] is True
            return None if mode == "capacity-wait" else {"key": owner + ":" + owner, "payload": {"owner": owner}}
        def release_unlaunched(self, _key, *, writer_key, execution_id):
            assert state.get("execution", execution_id)["payload"]["status"] in {"failed", "cancelled"}
            assert state.get("execution_launch", execution_id) is None
            self.released = True
        def release(self, _key, *, writer_key, native_proof_ref):
            assert state.get("native_job", native_proof_ref)["payload"]["completeProof"]["complete"] is True
            self.released = True
    class Guard:
        def __init__(self, *_args, **kwargs):
            assert kwargs["soft_pins"] is False
            self.closed = False
            guards.append(self)
        def acquire(self): return self
        def facts(self): return {"testDouble": True}
        def assert_stable(self): pass
        def finish(self): return {"complete": True, "stable": True, "waitersJoined": True}
        def close(self): self.closed = True
    def write(path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(node.encoded(value) + b"\n")
        return {"path": str(path), "sha256": node.file_hash(path)}
    def native(command, cwd, _environment, _report, _timeout):
        result = {"success": True, "testResults": [{"name": str(candidate.app / "src/penpot-control-content.spec.ts"),
                   "assertionResults": [{"status": "passed"} for _ in range(31)]}]}
        Path(candidate.config["test"]["outputFile"]["json"]).write_text(json.dumps(result), encoding="utf-8")
        return {"command": command, "cwd": str(cwd), "rootExitCode": 0, "cleanup": {"attempted": False}}
    m = SimpleNamespace(repo=candidate.repo, payload=candidate.payload, config={**candidate.payload, "managedNode": node.CAPABILITY},
                        state=state, key="a" * 64, session="public-owner", request="public8-original",
                        operation="prepare-review-r1", action="prepare-node", repo_id=candidate.payload["repositoryId"],
                        input_digest="original-input", checkout_git_read_conflict=lambda **_kwargs: None,
                        _node_guard_helper=lambda: SimpleNamespace(CheckoutReadGuard=Guard),
                        _node_write_json=write, _node_client_path=lambda: "native-test-seam",
                        _node_native_proof=lambda *_args: {"complete": mode != "native-unknown"})
    captured = {"files": {str(candidate.app / "src/penpot-control-content.spec.ts"): "0" * 64}, "bindings": {}}
    with patch("tools.jenkins.resources.capacity.ResourceManager", Budget), \
         patch("tools.jenkins.state.locks.process_lock", lambda *_a, **_k: nullcontext()), \
         patch.object(node, "capture", lambda *_a, **_k: captured), \
         patch.object(node, "_load", lambda *_a: SimpleNamespace(_run_owned_command=native)):
        prepared = node.run(m)
        if mode == "capacity-wait":
            assert prepared["status"] == "waiting" and not guards and not resources[0].released
            assert state.get("execution", "management-node-" + m.key) is None
            return
        assert prepared["status"] == "prepared" and prepared["captureReady"] is False
        assert prepared["nodeStarted"] is False and resources[0].released and guards[0].closed
        if mode == "prepare": return
        state.put("management_operation", m.key, {"phase": "complete", "action": "prepare-node", "result": prepared})
        m.key, m.action, m.operation = "d" * 64, "validate-node", "execute-reviewed-r1"
        m.payload = {**candidate.payload, "action": m.action, "operationId": m.operation,
                     "preparationRef": prepared["receipt"]}
        result = node.run(m)
        if mode == "execute":
            assert result["status"] == "passed" and result["details"]["passed"] == 31
            assert result["reservationReleased"] and resources[-1].released and guards[-1].closed
        else:
            assert result["status"] == "unknown" and result["originalReservationRetained"] is True
            assert not resources[-1].released and not guards[-1].closed
            assert state.get("management_checkout_read", "management-node-" + m.key)["payload"]["status"] == "unknown"


def test_capture_timeout_is_failure_not_empty_success(node):
    with pytest.raises(ValueError, match="timed out"):
        node.capture({"config": str(node.NODE), "worker": False}, {"sourceHashes": {}}, deadline=0)


@pytest.fixture
def exporter(node, candidate):
    for tree in node.EXPORTER_SOURCE_TREES:
        (candidate.repo / tree).mkdir(parents=True, exist_ok=True)
    http = candidate.repo / "third_party/penpot/exporter/src/app/http.cljs"
    http.parent.mkdir(parents=True, exist_ok=True)
    http.write_bytes(b"synthetic old listener\r\n")
    generated = candidate.output / "existing-enum.js"
    generated.write_bytes(b"// synthetic provenance fixture, never reused\n")
    value = {"requestId": node.EXPORTER_REQUEST, "ownerThreadId": "public-owner", "submitted": False,
             "officialSources": [{"path": str(http), "sha256": node.file_hash(http)}], "configs": [], "tools": [],
             "sourcePatchAdmission": {"path": http.relative_to(candidate.repo).as_posix(),
                                      "afterSha256": hashlib.sha256(b"synthetic authorized listener\r\n").hexdigest()},
             "generatedEnumReuse": {"candidate": {"path": str(generated), "sha256": node.file_hash(generated)}}}
    path = candidate.public / "exporter-candidate.json"
    path.write_bytes(node.encoded(value))
    with patch.multiple(node, EXPORTER_CANDIDATE=path, EXPORTER_CANDIDATE_SHA=node.file_hash(path)):
        payload = {**candidate.payload, "requestId": node.EXPORTER_REQUEST,
                   "nodeRecipe": {"schemaVersion": 1, "id": node.EXPORTER_RECIPE,
                                  "config": {"path": str(path), "sha256": node.file_hash(path)}}}
        yield SimpleNamespace(payload=payload, path=path, value=value, http=http)


def test_exporter_original_candidate_admits_only_metadata(node, candidate, exporter):
    plan = node.recipe(exporter.payload, candidate.repo)
    assert plan["exporterAdmission"] and plan["package"] == "penpot/exporter"
    assert plan["productCompilationSupported"] is False and "command" not in plan


@pytest.mark.parametrize("change", ["request", "validate", "config", "hash", "shell", "cargo"])
def test_exporter_rejects_other_identity_execution_and_configuration(node, candidate, exporter, change):
    payload = json.loads(json.dumps(exporter.payload))
    if change == "request": payload["requestId"] = "other-exporter"
    if change == "validate":
        payload.update(action="validate-node", preparationRef={})
    if change == "config":
        other = candidate.public / "other.json"
        other.write_bytes(exporter.path.read_bytes())
        payload["nodeRecipe"]["config"]["path"] = str(other)
    if change == "hash": payload["nodeRecipe"]["config"]["sha256"] = "0" * 64
    if change == "shell": payload["nodeRecipe"]["command"] = ["cmd", "/c", "anything"]
    if change == "cargo": payload["cargo"] = {"package": "zircon_editor"}
    with pytest.raises(ValueError):
        node.recipe(payload, candidate.repo)


def test_exporter_equal_enum_never_proves_reuse_or_compile_closure(node, candidate, exporter):
    unknown = candidate.repo / "third_party/penpot/common/target/classes/unknown.class"
    unknown.parent.mkdir(parents=True)
    unknown.write_bytes(b"foreign compiled class must not be read or removed")
    result = node.inspect_exporter_admission(exporter.path)
    codes = {row["code"] for row in result["blockedBy"]}
    assert {"exporter_http_source_revision_pending", "exporter_generated_enum_provenance_unproven",
            "exporter_compiler_closure_unprepared"} <= codes
    enum = next(row for row in result["blockedBy"] if row["code"] == "exporter_generated_enum_provenance_unproven")
    assert enum["bytesMatch"] is True and result["generatedEnumReused"] is False
    assert str(unknown) not in result["sourceCensus"]["files"] and unknown.is_file()
    assert result["productCommandsExecuted"] == [] and result["captureReady"] is False


def test_exporter_current_http_after_is_not_rejected_against_old_proposal(node, exporter):
    exporter.http.write_bytes(b"synthetic authorized listener\r\n")
    result = node.inspect_exporter_admission(exporter.path)
    assert result["httpAfterMatches"] is True
    assert result["sourceInputs"][0]["matches"] is True
    assert {row["code"] for row in result["blockedBy"]} == {
        "exporter_generated_enum_provenance_unproven", "exporter_compiler_closure_unprepared"}


def test_exporter_absent_configured_search_root_is_evidence_not_empty_compiler_acceptance(node, candidate, exporter):
    missing = candidate.repo / "third_party/penpot/common/vendor"
    missing.rmdir()
    result = node.inspect_exporter_admission(exporter.path)
    assert result["sourceCensus"]["rootPresence"][str(missing)] is False
    assert result["sourceCensus"]["missingConfiguredRoots"] == [str(missing)]
    assert result["sourceCensus"]["observedStable"] is True
    assert result["sourceCensus"]["compilerClosureComplete"] is False
    assert len(result["sourceInputs"]) == 1 and result["productCompilationSupported"] is False


def test_exporter_membership_drift_remains_a_real_blocker(node, exporter):
    original, calls = node._tree_files, 0
    def changing(root, *, deadline):
        nonlocal calls
        calls += 1
        if calls == len(node.EXPORTER_SOURCE_TREES) + 1:
            exporter.http.write_bytes(b"concurrent synthetic source revision\n")
        yield from original(root, deadline=deadline)
    with patch.object(node, "_tree_files", changing):
        result = node.inspect_exporter_admission(exporter.path)
    drift = next(row for row in result["blockedBy"] if row["code"] == "exporter_source_membership_changed_during_inspection")
    assert str(exporter.http) in drift["changed"] and result["sourceCensus"]["observedStable"] is False


def test_exporter_changed_candidate_is_not_silently_normalized(node, exporter):
    exporter.path.write_bytes(exporter.path.read_bytes() + b"\n")
    with pytest.raises(ValueError, match="candidate changed"):
        node.inspect_exporter_admission(exporter.path)


@pytest.mark.parametrize("mode", ["blocked", "native-unknown", "capacity-blocked", "invalid-result", "helper-failure"])
def test_exporter_inspection_lifecycle_keeps_product_and_unknown_reservations_distinct(node, candidate, exporter, mode):
    class Records:
        def __init__(self):
            self.rows = {("resource_policy", "default"): {"version": 11, "payload": {
                "cpuBudget": 6, "memoryBudget": 12 * 1024**3, "diskBudget": 40 * 1024**3}}}
        def get(self, domain, key): return self.rows.get((domain, key))
        def put(self, domain, key, payload, expected_version=0):
            old = self.get(domain, key)
            assert (old["version"] if old else 0) == expected_version
            row = {"key": key, "version": expected_version + 1, "payload": payload}
            self.rows[(domain, key)] = row
            return row
    state, budgets, launched = Records(), [], []
    class Budget:
        def __init__(self, *_args):
            self.released = False
            budgets.append(self)
        def queue(self, owner, **kwargs):
            return state.put("resource_queue", owner + ":" + owner, {"status": "waiting"})
        def scan_inventory(self, *_args): return {"freeBytes": 72 * 1024**3}, "fresh-exporter-inventory"
        def admit(self, owner, amount, **kwargs):
            assert (amount.cpu, amount.memory_bytes, amount.disk_bytes) == (1, 512 * 1024**2, 16 * 1024**2)
            assert kwargs["build_root"] == str(candidate.output)
            return None if mode == "capacity-blocked" else {"key": owner + ":" + owner}
        def release(self, _key, *, writer_key, native_proof_ref):
            assert state.get("native_job", native_proof_ref)["payload"]["completeProof"]["complete"] is True
            self.released = True
    def write(path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(node.encoded(value) + b"\n")
        return {"path": str(path), "sha256": node.file_hash(path)}
    def native(command, cwd, environment, report, timeout):
        assert command[-2:] == ["exporter-admission", str(exporter.path)] and timeout == 180
        assert all(environment[k] == str(report) for k in ("TEMP", "TMP", "TMPDIR"))
        launched.append(command)
        details = node.inspect_exporter_admission(exporter.path)
        if mode == "invalid-result": details["productCompilationSupported"] = True
        (report / "stdout.bin").write_bytes(node.encoded(details))
        return {"command": command, "cwd": str(cwd), "rootExitCode": 0, "cleanup": {"attempted": False}}
    def helper(*_args):
        if mode == "helper-failure": raise ValueError("pinned native helper changed")
        return SimpleNamespace(_run_owned_command=native)
    m = SimpleNamespace(repo=candidate.repo, payload=exporter.payload,
        config={"managedNode": node.CAPABILITY}, state=state, key="e" * 64, session="public-owner",
        request=node.EXPORTER_REQUEST, operation="prepare-exporter-r01", action="prepare-node",
        input_digest="original-exporter-input", checkout_git_read_conflict=lambda **_kwargs: None,
        _node_write_json=write, _node_client_path=lambda: "pinned-native-test-seam",
        _node_native_proof=lambda *_args: {"complete": mode != "native-unknown"})
    with patch("tools.jenkins.resources.capacity.ResourceManager", Budget), \
         patch("tools.jenkins.state.locks.process_lock", lambda *_a, **_k: nullcontext()), \
         patch.object(node, "_load", helper):
        result = node.run(m)
    owner = "management-node-" + m.key
    assert result["productStarted"] is False and result["captureReady"] is False
    if mode == "helper-failure":
        assert result["status"] == "failed" and state.get("execution", owner) is None and not launched
    elif mode == "capacity-blocked":
        assert result["status"] == "complete" and result["reasonCode"] == "resource_capacity" and not launched
        assert state.get("resource_queue", owner + ":" + owner)["payload"]["status"] == "cancelled"
    elif mode == "native-unknown":
        assert result["status"] == "unknown" and result["originalReservationRetained"] is True
        assert not budgets[0].released
        assert state.get("management_checkout_read", owner)["payload"]["status"] == "unknown"
    else:
        assert result["status"] == ("failed" if mode == "invalid-result" else "complete")
        assert result["metadataReservationReleased"] is True and budgets[0].released
        assert state.get("management_checkout_read", owner)["payload"]["pinsReleased"] is True
    if mode != "helper-failure":
        with pytest.raises(ValueError, match="requires reconciliation"):
            node.run(m)


@pytest.mark.skipif(os.name != "nt", reason="requires Windows native retained Job")
def test_actual_native_job_eof_and_owned_timeout(tmp_path):
    assert tmp_path.is_relative_to(Path("E:/cargo-targets"))
    helper = load("runner")
    local_path = ROOT.parents[1] / "tools/dev/local_cargo.py"
    spec = importlib.util.spec_from_file_location("node_checks_retained_job", local_path)
    native_helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native_helper)
    for ordinal, program, timeout in [(1, "import sys;print('native-node-route-check');print('stderr-eof',file=sys.stderr)", 10),
                                      (2, "import time;time.sleep(60)", 0.25)]:
        report = tmp_path / ("native-" + str(ordinal))
        report.mkdir()
        command = [sys.executable, "-B", "-c", program]
        native = native_helper._run_owned_command(command, tmp_path, dict(os.environ), report, timeout)
        raw = json.dumps(native, sort_keys=True).encode()
        (report / "native.json").write_bytes(raw)
        node = load("managed_node")
        proof = node.settled_native_proof(native,
            {"path": str(report / "native.json"), "sha256": hashlib.sha256(raw).hexdigest()},
            helper.native_terminal_proof)
        assert proof["complete"] is True and proof["childrenGone"] is True
        assert proof["stdoutEof"] is True and proof["stderrEof"] is True
        if ordinal == 1:
            assert native["rootExitCode"] == 0 and native["cleanup"]["attempted"] is False
        else:
            assert native["cleanup"]["complete"] is True and proof["naturalCompletion"] is False
            assert proof["ownedCleanup"] is True and native["terminalErrors"]
            native["processTree"]["pipes"]["stderr"]["eof"] = False
            assert node.settled_native_proof(native, {}, helper.native_terminal_proof)["complete"] is False


@pytest.mark.skipif(os.name != "nt", reason="requires native Windows file pins")
def test_pnpm_hardlink_read_identity_is_exact_and_does_not_relax_source(tmp_path, node):
    guard_module = load("checkout_guard")
    package_file = node.VITEST.parent / "package.json"
    checksum = node.file_hash(package_file)
    fact = guard_module.capture_readonly_node_input(str(package_file), checksum)
    assert fact["kind"] == "readonly-node-dependency" and fact["linkCount"] >= 1
    own = tmp_path / "input.json"
    own.write_text('{"synthetic":true}', encoding="utf-8")
    source_hash = node.file_hash(own)
    guard = guard_module.CheckoutReadGuard(tmp_path, files={str(own): source_hash, str(package_file): checksum},
        watch_roots=[str(tmp_path), str(package_file.parent)], trusted_native_inputs={str(package_file): fact})
    with guard:
        guard.assert_stable()
    assert guard.finish()["complete"] is True and guard.finish()["stable"] is True
    for path in (str(own), "C:/foreign/node_modules/pkg/package.json"):
        with pytest.raises(ValueError):
            guard_module.capture_readonly_node_input(path, source_hash)
    wrong = {**fact, "sha256": "0" * 64}
    with pytest.raises(ValueError):
        guard_module.CheckoutReadGuard(tmp_path, files={str(package_file): checksum},
            watch_roots=[str(tmp_path), str(package_file.parent)], trusted_native_inputs={str(package_file): wrong})
    alias = tmp_path / "hardlink.json"
    os.link(own, alias)
    with pytest.raises(ValueError):
        guard_module.CheckoutReadGuard(tmp_path, files={str(own): source_hash}, watch_roots=[str(tmp_path)]).acquire()


def test_installer_powershell_parses_without_execution():
    # Parse only: no install, source generation, service, or Node invocation.
    script = ROOT.parents[1] / "tools/dev/install-jenkins-management.ps1"
    raw = script.read_text(encoding="utf-8-sig")
    assert "[switch]$EnableManagedNode" in raw
    bootstrap = raw.split("$taskCandidateBootstrap = @'", 1)[1].split("\n'@", 1)[0]
    compile(bootstrap, "installer-candidate-bootstrap", "exec")


@pytest.mark.parametrize("package,pattern", [
    ("zircon_plugin_material_editor_editor", "material_graph_"),
    ("zircon_plugin_prefab_tools_editor", "prefab_"),
    ("zircon_plugin_particles_editor", "source_tests::"),
    ("zircon_plugin_sound_editor", "mixer_authoring_"),
    ("zircon_plugin_ai_editor", "authoring_tests::"),
    ("zircon_plugin_ai_editor", "gameplay_debugger_tests::"),
    ("zircon_plugin_ai_editor", "tests::pie_mirror_"),
    ("zircon_plugin_ai_editor", "tests::perception_overlay_"),
])
def test_deployed_runner_plugin_contracts_survive_node_install(package, pattern):
    entry, runner = load("entry"), load("runner")
    arguments = ["test", "--manifest-path", "zircon_plugins/Cargo.toml", "-p", package,
        "--no-default-features", "--locked", "--lib", "--profile", "test",
        "--jobs", "1", pattern, "--", "--test-threads", "1"]
    recipe = entry.parse_cargo(arguments)
    holder = SimpleNamespace(payload=recipe)
    result = runner.Management.validation_recipe(holder)
    assert result["packages"] == [package] and result["manifestPath"] == "zircon_plugins/Cargo.toml"
    assert result["testFilter"] == pattern and result["features"] == []
    assert result["profile"] == "test" and result["locked"] is True
    assert runner.plugin_lock_version("cargo 1.94.1 (old)") is False
    assert runner.plugin_lock_version("cargo 1.97.0 (required)") is True


@pytest.mark.parametrize("package,pattern", [
    ("zircon_plugin_material_editor_editor", "material_graph_"),
    ("zircon_plugin_sound_editor", "mixer_authoring_"),
])
def test_nested_manifest_cli_survives_client_to_runner_without_injected_recipe(package, pattern):
    entry, runner = load("entry"), load("runner")
    repo = ROOT.parents[1]
    manifest = repo / "zircon_plugins/Cargo.toml"
    arguments = ["test", "--manifest-path", manifest.as_posix(), "--package",
        package, "--no-default-features", "--locked", "--lib",
        "--profile", "test", "--jobs", "1", pattern, "--", "--test-threads", "1"]
    parsed = entry.parse_cargo(arguments, repo=repo)
    assert "manifestPath" in entry.RECIPE_FIELDS
    recipe = runner.Management.validation_recipe(SimpleNamespace(payload=parsed))
    assert recipe["command"][1:3] == ["--manifest-path", "zircon_plugins/Cargo.toml"]
    assert recipe["command"][-4:] == [pattern, "--", "--test-threads", "1"]
    assert recipe["packages"] == [package]
    assert recipe["features"] == [] and recipe["noDefaultFeatures"] and recipe["locked"]
    assert runner.cargo_manifest_environment(repo, recipe) == {
        "cargoWorkspaceManifest": "zircon_plugins/Cargo.toml"}
    assert runner.cargo_manifest_environment(repo, {"manifestPath": "Cargo.toml"}) != \
        runner.cargo_manifest_environment(repo, recipe)


@pytest.mark.parametrize("manifest", ["../Cargo.toml", "./zircon_plugins/Cargo.toml",
    "zircon_plugins/../Cargo.toml", "zircon_plugins/ai/editor/Cargo.toml",
    "F:/Git/ZirconEngine/zircon_plugins/Cargo.toml", "//other/share/Cargo.toml"])
@pytest.mark.parametrize("package,pattern", [
    ("zircon_plugin_material_editor_editor", "material_graph_"),
    ("zircon_plugin_sound_editor", "mixer_authoring_"),
])
def test_plugin_cli_rejects_non_catalog_or_noncanonical_manifest(manifest, package, pattern):
    entry = load("entry")
    with pytest.raises(entry.EntryError) as error:
        entry.parse_cargo(["test", "--manifest-path", manifest, "-p", package,
            "--no-default-features", "--locked", "--lib", "--profile", "test", pattern],
            repo=ROOT.parents[1])
    assert error.value.code == "cargo_manifest_unsupported"


@pytest.mark.parametrize("extra", [["--jobs", "2"], ["--features", "extra"], ["--ignored"],
    ["--test", "unrelated"], ["--package", "zircon_editor"], ["--no-run"], ["--release"]])
@pytest.mark.parametrize("package,pattern", [
    ("zircon_plugin_material_editor_editor", "material_graph_"),
    ("zircon_plugin_sound_editor", "mixer_authoring_"),
])
def test_plugin_cli_preserves_bounded_package_recipe(extra, package, pattern):
    entry = load("entry")
    arguments = ["test", "--manifest-path", "zircon_plugins/Cargo.toml", "-p",
        package, "--no-default-features", "--locked", "--lib",
        "--profile", "test", pattern]
    arguments += ["--", "--ignored"] if extra == ["--ignored"] else extra
    with pytest.raises(entry.EntryError):
        entry.parse_cargo(arguments)


@pytest.mark.parametrize("package,pattern", [
    ("zircon_plugin_material_editor_editor", "material_graph_"),
    ("zircon_plugin_sound_editor", "mixer_authoring_"),
])
def test_cargo_entry_forwards_normalized_manifest_and_original_identity(monkeypatch, package, pattern):
    entry = load("entry")
    payload = {"action": "validate", "sessionId": "fixture-owner", "requestId": "fixture-request",
               "operationId": "fixture-operation"}
    expected = dict(payload)
    monkeypatch.setattr(entry, "document", lambda path: (payload, entry.encoded(payload)))
    calls = []
    def dispatch(repo, value, **kwargs):
        calls.append((repo, value, kwargs))
        return {"fixture": True}
    monkeypatch.setattr(entry, "dispatch", dispatch)
    repo = ROOT.parents[1]
    assert entry.cargo_entry(repo, ["test", "--manifest-path", str(repo / "zircon_plugins/Cargo.toml"),
        "-p", package, "--no-default-features", "--locked", "--lib",
        "--profile", "test", "--jobs", "1", pattern, "--", "--test-threads", "1"],
        identity_file="fixture-identity.json") == {"fixture": True}
    assert len(calls) == 1 and calls[0][1] == expected == payload
    assert calls[0][2]["recipe"]["manifestPath"] == "zircon_plugins/Cargo.toml"
    assert calls[0][2]["recipe"]["packages"] == [package]
    assert calls[0][2]["recipe"]["testFilter"] == pattern


@pytest.mark.parametrize("integration", [True, False])
def test_manifest_cli_preserves_the_two_existing_gltf_recipes(integration):
    entry, runner = load("entry"), load("runner")
    args = ["test", "--manifest-path", "zircon_plugins/Cargo.toml", "-p",
        "zircon_plugin_gltf_importer_runtime", "--no-default-features", "--locked", "--profile", "test",
        "--jobs", "1", "--features", "runtime,zircon_runtime/native-host,zircon_runtime/animation"]
    args += ["--test", "character_skinning_sources"] if integration else ["--lib",
        "source_snapshot_tests::importer_rejects_required_extensions_outside_its_stable_policy"]
    args += ["--", "--test-threads", "1"]
    result = runner.Management.validation_recipe(SimpleNamespace(payload=entry.parse_cargo(args)))
    assert result["manifestPath"] == "zircon_plugins/Cargo.toml"
    assert result["tests"] == (["character_skinning_sources"] if integration else [])
    assert result["lib"] is (not integration)


@pytest.mark.parametrize("verb", ["check", "build", "test"])
def test_manifest_adapter_preserves_existing_explicit_root_package_recipes(verb):
    entry, runner = load("entry"), load("runner")
    args = [verb, "-p", "zircon_runtime", "--no-default-features", "--features", "animation", "--lib"]
    if verb == "test":
        args += ["public_bool_channel_", "--", "--test-threads", "1"]
    parsed = entry.parse_cargo(args)
    assert "manifestPath" not in parsed
    result = runner.Management.validation_recipe(SimpleNamespace(payload=parsed))
    assert result["verb"] == verb and result["packages"] == ["zircon_runtime"]
    assert result["features"] == ["animation"] and result["noDefaultFeatures"]
    assert "--manifest-path" not in result["command"]


def test_material_manifest_closure_keeps_nested_lock_and_cross_workspace_edges(tmp_path, monkeypatch):
    runner = load("runner")
    files = {
        "Cargo.toml": '[workspace]\nmembers=["zircon_editor","zircon_runtime"]\n',
        "zircon_plugins/Cargo.toml": '[workspace]\nmembers=["material_editor/editor"]\n'
            '[workspace.dependencies]\nzircon_editor={path="../zircon_editor"}\n'
            'zircon_runtime={path="../zircon_runtime",features=["native-host"]}\n',
        "zircon_plugins/material_editor/editor/Cargo.toml": '[package]\n'
            'name="zircon_plugin_material_editor_editor"\nversion="0.1.0"\n'
            '[dependencies]\nzircon_editor.workspace=true\nzircon_runtime.workspace=true\n',
        "zircon_editor/Cargo.toml": '[package]\nname="zircon_editor"\nversion="0.1.0"\n'
            '[dependencies]\nzircon_runtime={path="../zircon_runtime",features=["target-editor-host"]}\n',
        "zircon_runtime/Cargo.toml": '[package]\nname="zircon_runtime"\nversion="0.1.0"\n',
        "zircon_plugins/Cargo.lock": 'version=4\n[[package]]\n'
            'name="zircon_plugin_material_editor_editor"\nversion="0.1.0"\n',
    }
    for name, text in files.items():
        path = tmp_path / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    source = {"files": {name: runner.sha((tmp_path / name).read_bytes()) for name in files}}
    monkeypatch.setattr(runner, "source_helper", lambda: SimpleNamespace(source_files=lambda value: value["files"]))
    captured = {"sourceView": "checkout", "sourceRoot": str(tmp_path), "source": source, "externalRoots": []}
    recipe = {"manifestPath": "zircon_plugins/Cargo.toml", "packages": ["zircon_plugin_material_editor_editor"]}
    contract = runner.plugin_workspace_inputs(tmp_path, recipe, captured)
    assert contract["selectedLockfile"] == str(tmp_path / "zircon_plugins/Cargo.lock")
    assert str(tmp_path / "Cargo.toml") in contract["workspaceManifests"]
    assert any(edge["package"] == "zircon_runtime" and "native-host" in edge["declaredFeatures"]
               for edge in contract["pathDependencyEdges"])
    assert any(edge["fromPackage"] == "zircon_editor" and "target-editor-host" in edge["declaredFeatures"]
               for edge in contract["pathDependencyEdges"])
    assert contract["cargoResolutionPerformed"] is False
    (tmp_path / "zircon_plugins/Cargo.lock").write_text("version=3\n", encoding="utf-8")
    with pytest.raises(ValueError, match="captured source"):
        runner.plugin_workspace_inputs(tmp_path, recipe, captured)


def test_installer_accepts_existing_node_inventory_and_rejects_unknown_members():
    # Evaluate the actual production statements without invoking the installer.
    installer = ROOT.parent / "dev/install-jenkins-management.ps1"
    script = r"""
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile('__INSTALLER__', [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Installer syntax error' }
$assignments = @($ast.FindAll({ param($n)
    $n -is [Management.Automation.Language.AssignmentStatementAst] -and
    $n.Left -is [Management.Automation.Language.VariableExpressionAst] -and
    $n.Left.VariablePath.UserPath -ceq 'taskOldInventory'
}, $true))
$guards = @($ast.FindAll({ param($n)
    $n -is [Management.Automation.Language.IfStatementAst] -and
    $n.Extent.Text -match '^if \(\$taskOldInventory -cnotin '
}, $true))
if ($assignments.Count -ne 1 -or $guards.Count -ne 1) { throw 'Original inventory gate is ambiguous' }
$gate = [scriptblock]::Create($assignments[0].Extent.Text + "`n" + $guards[0].Extent.Text)
$legacy = @('runner.py','source_preparation.py','zircon-management.groovy')
$product = @('checkout_guard.py','checkout_product.py') + $legacy
$node = @('managed_node.py') + $product
$cases = @(
    @{id='legacy'; names=$legacy; accepted=$true},
    @{id='product'; names=$product; accepted=$true},
    @{id='node'; names=$node; accepted=$true},
    @{id='unknown'; names=(@('unknown.py') + $node); accepted=$false},
    @{id='missing-runner'; names=@($node | Where-Object { $_ -cne 'runner.py' }); accepted=$false},
    @{id='node-without-product'; names=(@('managed_node.py') + $legacy); accepted=$false}
)
$rows = @()
foreach ($case in $cases) {
    $files = [ordered]@{}
    foreach ($name in $case.names) { $files[$name] = 'a' * 64 }
    $taskOldMetadata = [pscustomobject]@{files=[pscustomobject]$files}
    try { & $gate; $accepted=$true; $message=$null }
    catch { $accepted=$false; $message=$_.Exception.Message }
    $rows += [pscustomobject]@{id=$case.id; accepted=$accepted; expected=$case.accepted; message=$message}
}
[Console]::Out.WriteLine((ConvertTo-Json -InputObject $rows -Depth 4 -Compress))
""".replace("__INSTALLER__", str(installer).replace("'", "''"))
    result = subprocess.run(["C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe",
        "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=30, check=False)
    assert result.returncode == 0, result.stderr
    rows = json.loads(result.stdout)
    assert len(rows) == 6
    for row in rows:
        assert row["accepted"] is row["expected"], row
        if not row["expected"]:
            assert row["message"] == "Original capsule inventory differs"
