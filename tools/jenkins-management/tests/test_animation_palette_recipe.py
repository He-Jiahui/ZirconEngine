"""Bounded palette management contracts; these tests never invoke Cargo."""
from __future__ import annotations

import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
from types import SimpleNamespace

import pytest


TOOLS = Path(__file__).resolve().parents[1]
REPO = TOOLS.parents[1]
PACKAGE = "zircon_plugin_animation_graph_editor"
FILTER = "tests::graph_palette_matches_creatable_editor_capabilities"
MANIFEST = "zircon_plugins/Cargo.toml"


def load(name):
    spec = importlib.util.spec_from_file_location("palette_recipe_" + name, TOOLS / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def argv(manifest=MANIFEST):
    return ["test", "--manifest-path", manifest, "-p", PACKAGE,
            "--no-default-features", "--locked", "--lib", "--profile", "test",
            "--jobs", "1", FILTER, "--", "--test-threads", "1"]


def payload():
    return {"verb": "test", "manifestPath": MANIFEST, "packages": [PACKAGE],
            "features": [], "profile": "test", "offline": False, "noDefaultFeatures": True,
            "allFeatures": False, "locked": True, "jobs": 1, "lib": True,
            "tests": [], "noRun": False, "testFilter": FILTER, "testThreads": 1}


def server(runner, value):
    return runner.Management.validation_recipe(SimpleNamespace(payload=value))


@pytest.mark.parametrize("manifest", [MANIFEST, str(REPO / MANIFEST), (REPO / MANIFEST).as_posix()])
def test_palette_client_server_and_manifest_pool_identity(manifest):
    entry, runner = load("entry"), load("runner")
    parsed = entry.parse_cargo(argv(manifest), repo=REPO)
    assert parsed == payload()
    recipe = server(runner, parsed)
    command = recipe["command"]
    assert command == ["test", "--manifest-path", MANIFEST, "--jobs", "1", "--message-format=json",
                       "--locked", "--no-default-features", "--lib", "--package", PACKAGE,
                       "--profile", "test", FILTER, "--", "--test-threads", "1"]
    assert runner.cargo_manifest_environment(REPO, recipe) == {"cargoWorkspaceManifest": MANIFEST}
    assert runner.cargo_manifest_environment(REPO, recipe) != runner.cargo_manifest_environment(
        REPO, {**recipe, "manifestPath": "Cargo.toml"})


def wrapper(extra, cwd):
    return subprocess.run([
        os.environ.get("SYSTEMROOT", "C:/Windows") + "/System32/WindowsPowerShell/v1.0/powershell.exe",
        "-NoLogo", "-NoProfile", "-NonInteractive", "-File", str(REPO / "tools/dev/dev-module-build.ps1"),
        "-Package", PACKAGE, "-Action", "test", "-LibTests", "-TestFilter", FILTER,
        "-Jobs", "1", "-TestThreads", "1", "-DryRun", *extra,
    ], cwd=cwd, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=30)


def test_actual_wrapper_dry_run_round_trips_without_capture_dispatch_or_cargo(tmp_path):
    assert tmp_path.is_relative_to(Path("E:/cargo-targets"))
    result = wrapper([], tmp_path)
    assert result.returncode == 0, result.stderr
    plan = json.loads(result.stdout)
    assert plan["commandExecuted"] is False and plan["executionOwner"] == "Jenkins"
    parsed = load("entry").parse_cargo(plan["cargoArguments"], repo=REPO)
    assert parsed == payload()
    assert server(load("runner"), parsed)["manifestPath"] == MANIFEST


@pytest.mark.parametrize("extra", [
    ["-Features", "animation"], ["-Release"], ["-NoRun"], ["-Offline"],
    ["-TestTarget", "other"], ["-RepoRoot", "E:/other"], ["-ManifestPath", "Cargo.toml"],
])
def test_wrapper_rejects_scope_extensions_before_any_command(extra, tmp_path):
    result = wrapper(extra, tmp_path)
    assert result.returncode != 0


@pytest.mark.parametrize("field,value", [
    ("manifestPath", "Cargo.toml"), ("manifestPath", "other/Cargo.toml"),
    ("manifestPath", str(REPO / MANIFEST)), ("manifestPath", "zircon_plugins/../Cargo.toml"),
    ("packages", [PACKAGE, "zircon_editor"]), ("packages", ["zircon_plugin_material_editor_editor"]),
    ("features", ["animation"]), ("profile", "dev"), ("profile", "release"),
    ("verb", "check"), ("verb", "build"), ("noDefaultFeatures", False),
    ("locked", False), ("jobs", 2), ("testThreads", 2), ("lib", False),
    ("tests", ["cases"]), ("noRun", True), ("testFilter", "tests::"), ("testFilter", None),
    ("allFeatures", True), ("allTests", True), ("bins", True), ("offline", True),
    ("harnessFilters", ["extra"]), ("ignored", True), ("exact", True), ("nocapture", True),
    ("cwd", str(REPO)), ("target", "x86_64-pc-windows-msvc"),
])
def test_server_rejects_untrusted_recipe_extensions(field, value):
    recipe = payload()
    recipe[field] = value
    with pytest.raises(ValueError):
        server(load("runner"), recipe)


def test_server_requires_the_plugin_manifest_even_when_field_is_absent():
    recipe = payload()
    del recipe["manifestPath"]
    with pytest.raises(ValueError, match="canonical plugin manifest"):
        server(load("runner"), recipe)


@pytest.mark.parametrize("change", [
    ["--target", "x86_64-pc-windows-msvc"], ["--cwd", str(REPO)], ["--workspace"],
    ["--all"], ["--all-targets"], ["--all-features"], ["--features", "animation"],
    ["--tests"], ["--bins"], ["--no-run"], ["--offline"],
])
@pytest.mark.parametrize("enforce_policy", [True, False])
def test_client_rejects_scope_extensions_including_query_only(change, enforce_policy):
    args = argv()
    args[args.index("--"):args.index("--")] = change
    entry = load("entry")
    with pytest.raises(entry.EntryError):
        entry.parse_cargo(args, repo=REPO, enforce_policy=enforce_policy)


@pytest.mark.parametrize("old,new", [("test", "check"), ("test", "build"),
    ("test", "release"), ("--lib", "--bins"), (FILTER, "tests::"),
    (PACKAGE, "zircon_plugin_material_editor_editor"), ("--jobs", "--target")])
def test_client_rejects_changed_selectors(old, new):
    entry = load("entry")
    args = argv()
    # The profile uses the second occurrence of 'test'; verbs are the first.
    index = args.index("--profile") + 1 if new == "release" else args.index(old)
    args[index] = new
    with pytest.raises(entry.EntryError):
        entry.parse_cargo(args, repo=REPO)


@pytest.mark.parametrize("option", ["--jobs", "--test-threads"])
def test_client_rejects_parallel_palette_execution(option):
    entry = load("entry")
    args = argv()
    args[args.index(option) + 1] = "2"
    with pytest.raises(entry.EntryError):
        entry.parse_cargo(args, repo=REPO)


@pytest.mark.parametrize("manifest", ["Cargo.toml", "other/Cargo.toml", "zircon_plugins/../Cargo.toml",
                                      "E:/other/zircon_plugins/Cargo.toml"])
def test_client_rejects_manifest_aliases_and_root_fallback(manifest):
    entry = load("entry")
    with pytest.raises(entry.EntryError):
        entry.parse_cargo(argv(manifest), repo=REPO)


@pytest.fixture
def captured_palette(tmp_path, monkeypatch):
    assert tmp_path.is_relative_to(Path("E:/cargo-targets"))
    runner = load("runner")
    files = {
        "Cargo.toml": '[workspace]\nmembers=["zircon_editor","zircon_runtime"]\n',
        MANIFEST: '[workspace]\nmembers=["animation_graph/editor"]\n[workspace.dependencies]\n'
                  'zircon_editor={path="../zircon_editor"}\n'
                  'zircon_runtime={path="../zircon_runtime",default-features=false,features=["native-host"]}\n',
        "zircon_plugins/animation_graph/editor/Cargo.toml": '[package]\nname="' + PACKAGE + '"\n'
                  'version="0.1.0"\n[dependencies]\nzircon_editor.workspace=true\nzircon_runtime.workspace=true\n',
        "zircon_editor/Cargo.toml": '[package]\nname="zircon_editor"\nversion="0.1.0"\n'
                  '[dependencies]\nzircon_runtime={path="../zircon_runtime",features=["target-editor-host"]}\n',
        "zircon_runtime/Cargo.toml": '[package]\nname="zircon_runtime"\nversion="0.1.0"\n',
        "zircon_plugins/Cargo.lock": 'version=4\n[[package]]\nname="' + PACKAGE + '"\nversion="0.1.0"\n',
        ".cargo/config.toml": '[build]\njobs=1\n',
        "zircon_plugins/.cargo/config.toml": '[build]\njobs=1\n',
    }

    def write(name, content):
        path = tmp_path / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode("utf-8"))

    for name, content in files.items():
        write(name, content)
    monkeypatch.setattr(runner, "source_helper", lambda: SimpleNamespace(source_files=lambda value: value["files"]))

    def capture():
        return {"sourceView": "checkout", "sourceRoot": str(tmp_path), "externalRoots": [],
                "source": {"files": {name: runner.sha((tmp_path / name).read_bytes()) for name in files}}}

    return runner, tmp_path, files, write, capture


def test_palette_closure_binds_nested_lock_config_and_both_runtime_edges(captured_palette):
    runner, root, files, write, capture = captured_palette
    contract = runner.plugin_workspace_inputs(root, payload(), capture())
    assert contract["selectedPackage"] == PACKAGE
    assert contract["cwd"] == str(root) and contract["selectedLockfile"] == str(root / "zircon_plugins/Cargo.lock")
    assert str(root / "Cargo.toml") in contract["workspaceManifests"]
    assert all(str(root / name) in contract["files"] for name in files)
    edges = contract["pathDependencyEdges"]
    assert any(e["package"] == "zircon_runtime" and "native-host" in e["declaredFeatures"] for e in edges)
    assert any(e["fromPackage"] == "zircon_editor" and "target-editor-host" in e["declaredFeatures"] for e in edges)
    assert contract["cargoResolutionPerformed"] is False
    write("zircon_plugins/.cargo/config.toml", "[build]\njobs=2\n")
    changed = runner.plugin_workspace_inputs(root, payload(), capture())
    assert changed["inputDigest"] != contract["inputDigest"]
    assert next(v for v in changed["cargoConfiguration"] if v["path"] == str(root / ".cargo/config.toml"))[
        "effectiveFromCanonicalCwd"] is True
    assert next(v for v in changed["cargoConfiguration"] if v["path"] == str(root / "zircon_plugins/.cargo/config.toml"))[
        "effectiveFromCanonicalCwd"] is False


@pytest.mark.parametrize("name,replacement,recapture", [
    ("zircon_plugins/Cargo.lock", "version=4\n", False),
    ("zircon_plugins/Cargo.lock", "version=4\n", True),
    (MANIFEST, '[workspace]\nmembers=[]\n', True),
    (MANIFEST, '[workspace]\nmembers=["animation_graph/editor"]\n[workspace.dependencies]\n'
               'zircon_editor={path="../zircon_editor"}\nzircon_runtime={path="../zircon_runtime"}\n', True),
    ("zircon_editor/Cargo.toml", '[package]\nname="zircon_editor"\nversion="0.1.0"\n'
                                '[dependencies]\nzircon_runtime={path="../zircon_runtime"}\n', True),
    ("zircon_runtime/Cargo.toml", '[package]\nname="wrong_package"\nversion="0.1.0"\n', True),
    ("zircon_plugins/animation_graph/editor/Cargo.toml", '[package]\nname="' + PACKAGE + '"\n'
        'version="0.1.0"\n[dependencies]\nforeign={path="../../../../outside"}\n', True),
])
def test_palette_closure_rejects_stale_or_incomplete_source_and_feature_contracts(
        captured_palette, name, replacement, recapture):
    runner, root, files, write, capture = captured_palette
    before = capture()
    write(name, replacement)
    with pytest.raises(ValueError):
        runner.plugin_workspace_inputs(root, payload(), capture() if recapture else before)


@pytest.mark.parametrize("field,value", [("sourceView", "snapshot"), ("sourceRoot", "E:/cargo-targets")])
def test_palette_closure_requires_actual_canonical_checkout(captured_palette, field, value):
    runner, root, files, write, capture = captured_palette
    captured = copy.deepcopy(capture())
    captured[field] = value
    with pytest.raises(ValueError, match="canonical checkout"):
        runner.plugin_workspace_inputs(root, payload(), captured)
