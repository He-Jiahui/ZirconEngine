"""Affected-owner CI planning. Reading a plan never executes Cargo.

Recipes are reviewed contracts, not a workspace enumeration. Unknown impact is
an error even when --select is provided; selection cannot waive coverage.
"""
from __future__ import annotations

import argparse
import glob
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]
HOST = "x86_64-pc-windows-msvc"
RENDER_ROWS = [
    {
        "feature": "baked_lighting",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_baked_lighting_runtime",
        "args": "--lib --no-default-features",
        "filter": "baked_lighting_feature_executes_in_nonempty_wgpu_product_frame"
    },
    {
        "feature": "decals",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_decals_runtime",
        "args": "--lib --no-default-features",
        "filter": "decals_feature_executes_in_nonempty_wgpu_product_frame"
    },
    {
        "feature": "contact_shadow",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_contact_shadow_runtime",
        "args": "--lib --no-default-features",
        "filter": "contact_shadow_wgpu_product_capture_darkens_"
    },
    {
        "feature": "irradiance_volumes",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_af_m2_cookie_and_volume_execute_and_change_wgpu_frame"
    },
    {
        "feature": "light_cookies",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_af_m2_cookie_and_volume_execute_and_change_wgpu_frame"
    },
    {
        "feature": "oit",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_advanced_lighting_oit_three_crossing_planes_differs_from_sorted"
    },
    {
        "feature": "planar_reflections",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_planar_reflection_changes_mirror_floor_through_camera_loop"
    },
    {
        "feature": "post_process",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_post_uber_light_effects_change_final_frame"
    },
    {
        "feature": "reflection_probes",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_probe_feature_off_falls_back_to_skybox"
    },
    {
        "feature": "ssao",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "ssao_product_default_is_fail_closed_and_preserves_scene_output"
    },
    {
        "feature": "subsurface_scattering",
        "manifest": "Cargo.toml",
        "package": "zircon_runtime",
        "args": "--lib --no-default-features --features native-host,graphics",
        "filter": "render_product_sss_skin_sphere_executes_burley_indirect_pipeline"
    },
    {
        "feature": "ray_tracing_policy",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_ray_tracing_policy_runtime",
        "args": "--lib --no-default-features",
        "filter": "ray_tracing_policy_reports_backend_gates_and_keeps_a_real_product_frame"
    },
    {
        "feature": "shader_graph",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_shader_graph_runtime",
        "args": "--lib --no-default-features",
        "filter": "shader_graph_feature_executes_a_real_product_frame"
    },
    {
        "feature": "vfx_graph",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_vfx_graph_runtime",
        "args": "--lib --no-default-features",
        "filter": "vfx_graph_feature_executes_a_real_product_frame"
    },
    {
        "feature": "volumetric_fog",
        "manifest": "zircon_plugins/Cargo.toml",
        "package": "zircon_plugin_rendering_volumetric_fog_runtime",
        "args": "--lib --no-default-features",
        "filter": "smoke_volumetric_fog_executes_a_product_frame"
    }
]
RENDER_WRAPPERS = {
    "irradiance_volumes": ("zircon_plugin_rendering_irradiance_volumes_runtime", "irradiance_volume_feature_is_optional_and_registers_binding_executor"),
    "light_cookies": ("zircon_plugin_rendering_light_cookies_runtime", "light_cookie_feature_is_optional_and_registers_atlas_executor"),
    "subsurface_scattering": ("zircon_plugin_rendering_subsurface_scattering_runtime", "sss_feature_registers_three_passes_executors_and_shading_model"),
}
DOMAIN_FEATURES = ["ai-contracts","animation","diagnostic-log","dynamic-api","graphics","navigation","net-contracts","physics-contracts","script","sound-contracts","text","ui"]
MIRI_FILTERS = (
    "scene::ecs::storage::component_storage::store::tests::sparse_rows_rekey_at_the_target_tick",
    "scene::ecs::archetype::table::tests",
    "scene::tests::ecs_query_many::unsafe_custom_mutable_query_preserves_real_borrows_and_change_ticks",
)
# Each rule binds an actual subtree to the minimum feature and mounted test module.
RUNTIME_RULES = {
    "scene/": ("native-host", "scene::"),
    "core/framework/ai/": ("ai-contracts", None),
    "core/framework/net/": ("net-contracts", "core::framework::net::"),
    "core/framework/physics/": ("physics-contracts", "core::framework::physics::"),
    "core/framework/sound/": ("sound-contracts", "core::framework::sound::"),
    "core/framework/audio/": ("core-min", "core::framework::audio::"),
    "core/framework/animation/": ("core-min", "core::framework::animation::"),
    "core/framework/time/": ("core-min", "core::framework::time::"),
    "core/runtime/": ("native-host", "core::runtime::"),
    "core/math/": ("core-min", None),
    "engine_module/": ("core-min", "engine_module::"),
    "core/": ("native-host", "core::"),
    "animation/": ("animation", "animation::"),
    "foundation/": ("native-host", "foundation::"),
    "asset/": ("native-host", "asset::"),
    "input/": ("native-host", "input::"),
    "graphics/": ("graphics", "graphics::"),
    "render_graph/": ("graphics", "render_graph::"),
    "operation/": ("native-host", "operation::"),
    "platform/": ("native-host", "platform::"),
    "ui/": ("ui", "ui::"),
    "text/": ("text", "text::"),
    "script/": ("script", "script::"),
    "dynamic_api/": ("dynamic-api", "dynamic_api::"),
    "navigation/": ("navigation", "navigation::"),
    "plugin/": ("native-host", "plugin::"),
    "builtin/": ("native-host", "builtin::"),
    "tests/contracts/structure/": ("native-host", "structure_convention"),
}
# Unit-test binaries also compile the native project-backed graphics test helpers.
# Keep production domain checks graphics-only; this augmentation belongs to tests.
RUNTIME_TEST_EXTRA_FEATURES = {
    "graphics/": ("native-host",),
    "render_graph/": ("native-host",),
}
# Source ownership is independent of the package hosting the real frame test.
RENDER_SOURCE_OWNERS = {
    "baked_lighting": "zircon_plugin_rendering_baked_lighting_runtime",
    "contact_shadow": "zircon_plugin_rendering_contact_shadow_runtime",
    "decals": "zircon_plugin_rendering_decals_runtime",
    "irradiance_volumes": "zircon_plugin_rendering_irradiance_volumes_runtime",
    "light_cookies": "zircon_plugin_rendering_light_cookies_runtime",
    "oit": "zircon_plugin_rendering_oit_runtime",
    "planar_reflections": "zircon_plugin_rendering_planar_reflections_runtime",
    "post_process": "zircon_plugin_rendering_post_process_runtime",
    "reflection_probes": "zircon_plugin_rendering_reflection_probes_runtime",
    "ssao": "zircon_plugin_rendering_ssao_runtime",
    "subsurface_scattering": "zircon_plugin_rendering_subsurface_scattering_runtime",
    "ray_tracing_policy": "zircon_plugin_rendering_ray_tracing_policy_runtime",
    "shader_graph": "zircon_plugin_rendering_shader_graph_runtime",
    "vfx_graph": "zircon_plugin_rendering_vfx_graph_runtime",
    "volumetric_fog": "zircon_plugin_rendering_volumetric_fog_runtime"
}
# These roots are mounted without optional features by lib.rs -> core/mod.rs.
# Test-only files do not become Wasm production inputs merely by sharing a directory.
PORTABLE_SOURCE_PREFIXES = (
    "core/framework/audio/", "core/framework/animation/", "core/framework/time/",
    "core/math/", "core/runtime/", "engine_module/",
)
# No native-host fallback may hide code behind these independent feature/backend cfgs.
RUNTIME_CFG_BOUNDARIES = (
    "core/mod.rs", "core/framework/mod.rs", "core/framework/tests/",
    "core/manager/", "core/resource/", "core/runtime/mod.rs",
    "core/runtime/tests/", "core/runtime/diagnostics/", "core/runtime/render_stats_store/",
)


def portable_source(local):
    return (local.endswith(".rs") and "/tests/" not in "/" + local
            and not local.endswith("/tests.rs")
            and local.startswith(PORTABLE_SOURCE_PREFIXES)
            and not any(local == boundary or local.startswith(boundary)
                        for boundary in RUNTIME_CFG_BOUNDARIES))

# Public leaf changes need a reviewed caller contract, not every reverse dependent.
LEAF_CONSUMERS = {
    "zr_math": {"zircon_runtime": "runtime-scene", "zircon_runtime_interface": "leaf-zircon_runtime_interface"},
    "zr_resource": {"zircon_runtime": "runtime-asset"},
    "zr_rhi": {"zircon_runtime": "runtime-graphics", "zr_rhi_wgpu": "leaf-zr_rhi_wgpu"},
    "zr_rhi_wgpu": {"zircon_runtime": "runtime-graphics"},
    "zircon_reflect_derive": {"zircon_runtime": "runtime-scene"},
    "zircon_runtime_interface": {"zircon_runtime": "runtime-dynamic_api"},
}
LEAVES = frozenset(LEAF_CONSUMERS)
EXPORT_PLATFORMS = ("windows", "linux", "macos", "android", "ios", "web_gpu", "wasm", "headless")
EXPORT_POLICY_SOURCES = frozenset({
    "zircon_runtime/src/core/framework/project/export_profile.rs",
    "zircon_runtime/src/tests/plugins/export_build_plan_platform.rs",
})
PLATFORM_FILTER = ("tests::plugin_extensions::export_build_plan_platform::"
                   "platform_target_policy_matches_host_resource_and_plugin_strategy")


class ScopeError(ValueError):
    pass


def need(condition, message):
    if not condition:
        raise ScopeError(message)


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root)


def changed_paths(root, base, head):
    for revision in (base, head):
        need(re.fullmatch(r"[0-9a-fA-F]{40}", revision or "") and set(revision) != {"0"},
             "A real base and head SHA are required; fetch the base or supply --base explicitly.")
        git(root, "cat-file", "-e", revision + "^{commit}")
    fields = git(root, "diff", "--name-status", "-z", "--find-renames", base, head, "--").split(b"\0")
    paths, index = set(), 0
    while index < len(fields) and fields[index]:
        status = fields[index].decode("ascii")
        index += 1
        count = 2 if status.startswith(("R", "C")) else 1
        for _ in range(count):
            need(index < len(fields) and bool(fields[index]), "Truncated changed-path record")
            value = fields[index].decode("utf-8", errors="strict")
            index += 1
            name = PurePosixPath(value)
            need(not name.is_absolute() and ".." not in name.parts and "\\" not in value,
                 "Noncanonical changed path: " + value)
            paths.add(value)
    return sorted(paths)


def catalog(root):
    """Read declared members only; never follow an arbitrary repository scan."""
    root = root.resolve()
    packages = {}
    for workspace in ("Cargo.toml", "zircon_plugins/Cargo.toml"):
        manifest = root / workspace
        doc = tomllib.loads(manifest.read_text(encoding="utf-8"))
        workspace_deps = doc.get("workspace", {}).get("dependencies", {})
        excluded = set(doc.get("workspace", {}).get("exclude", []))
        for member in doc.get("workspace", {}).get("members", []):
            if member in excluded:
                continue
            directories = sorted(glob.glob(str(manifest.parent / member)))
            need(bool(directories), "Declared member disappeared: " + member)
            for directory in directories:
                file = Path(directory) / "Cargo.toml"
                need(file.is_file(), "Declared member has no manifest: " + str(file))
                need(not file.is_symlink() and file.absolute() == file.resolve()
                     and file.resolve().is_relative_to(root),
                     "Aliased/outside member requires explicit source contract: " + str(file))
                data = tomllib.loads(file.read_text(encoding="utf-8"))
                package = data.get("package", {})
                name = package.get("name")
                need(name and name not in packages, "Missing/duplicate package: " + str(file))
                deps = []
                for table in (data, *data.get("target", {}).values()):
                    for kind in ("dependencies", "build-dependencies", "dev-dependencies"):
                        for depname, value in table.get(kind, {}).items():
                            if isinstance(value, dict):
                                if value.get("workspace"):
                                    inherited = workspace_deps.get(depname, {})
                                    value = {**(inherited if isinstance(inherited, dict) else {}), **value}
                                if "path" in value:
                                    # Workspace paths are relative to their declaring workspace.
                                    origin = manifest.parent if value.get("workspace") else file.parent
                                    dependency = (origin / value["path"] / "Cargo.toml").resolve()
                                    need(dependency.is_relative_to(root) and dependency.is_file(),
                                         "Path dependency has no canonical owner: " + str(dependency))
                                    depdoc = tomllib.loads(dependency.read_text(encoding="utf-8"))
                                    deps.append(depdoc["package"]["name"])
                                else:
                                    deps.append(value.get("package", depname))
                            else:
                                deps.append(depname)
                relative = file.parent.relative_to(root).as_posix()
                packages[name] = dict(name=name, directory=relative, manifest=workspace,
                                      data=data, dependencies=deps)
    return packages


def recipe(identifier, package, group="rust", features=(), selector=None, verb="test", target="lib", reason="owning module"):
    need(verb in {"check", "build", "test", "clippy"}, "Unsupported recipe action")
    return dict(id=identifier, package=package, group=group, features=list(features),
                selector=selector, verb=verb, target=target, profile="dev",
                kind="owner-regression" if verb == "test" else "compile-check",
                platform=HOST, reason=reason)


def registry(packages, root=ROOT):
    recipes = {}
    def add(value):
        name = value["package"]
        need(name in packages, "Recipe package disappeared: " + name)
        value["manifest"] = packages[name]["manifest"]
        package = packages[name]
        need("lib" in package["data"] or (root / package["directory"] / "src/lib.rs").is_file(),
             "Recipe requires an actual library target: " + value["id"])
        features = package["data"].get("features", {})
        need(all(feature in features for feature in value["features"]),
             "Recipe feature disappeared: " + value["id"])
        recipes[value["id"]] = value

    for prefix, (feature, selector) in RUNTIME_RULES.items():
        name = prefix.rstrip("/").replace("/", "-")
        value = recipe("runtime-" + name, "zircon_runtime",
                       features=(feature, *RUNTIME_TEST_EXTRA_FEATURES.get(prefix, ())), selector=selector,
                       verb="test" if selector is not None else "check")
        if prefix == "core/framework/ai/":
            value["reason"] = "AI DTO owner has no mounted unit tests; ai-contracts compile check only"
        elif selector is None:
            value["reason"] = "curated reexport owner has no mounted unit tests; compile check only"
        add(value)
    for name in sorted(LEAVES & packages.keys()):
        add(recipe("leaf-" + name, name, selector="tests::", reason="changed leaf and reviewed callers"))
    # Explicit library recipes for real plugin runtime owners; no dist/all-package loop.
    for name, package in packages.items():
        directory = package["directory"]
        data = package["data"]
        library = "lib" in data or (root / directory / "src/lib.rs").is_file()
        if library and directory.startswith("zircon_plugins/") and "/runtime" in directory:
            add(recipe("plugin-" + name, name, selector="tests::", reason="changed plugin runtime library"))
        # First-party editor plugins follow the same library recipe as runtime plugins.
        if library and directory.startswith("zircon_plugins/") and directory.endswith("/editor"):
            add(recipe("plugin-" + name, name, selector="tests::", reason="changed first-party editor library"))
        if directory.endswith("/dist") and "dist" in data.get("features", {}):
            add(recipe("dist-" + name, name, group="dist", features=("dist",), verb="build"))
    if "zircon_editor" in packages:
        add(recipe("editor-zircon_editor", "zircon_editor", selector="tests::",
                   reason="changed editor library"))
    if "zircon_first_party_runtime_catalog" in packages:
        add(recipe("plugin-zircon_first_party_runtime_catalog", "zircon_first_party_runtime_catalog",
                   selector="tests::", reason="first-party runtime catalog consumer"))
    need(set(RENDER_SOURCE_OWNERS) == {row["feature"] for row in RENDER_ROWS},
         "Every real frame contract needs exactly one declared source owner")
    for feature, name in RENDER_SOURCE_OWNERS.items():
        directory = "subsurface" if feature == "subsurface_scattering" else feature
        need(name in packages and packages[name]["directory"] ==
             "zircon_plugins/rendering/features/" + directory + "/runtime",
             "Render source owner moved; review the frame contract mapping: " + feature)
    for row in RENDER_ROWS:
        features = ("native-host", "graphics") if row["package"] == "zircon_runtime" else ()
        add(recipe("render-" + row["feature"], row["package"], group="render",
                   features=features, selector=row["filter"], reason="real feature graph and product frame"))
        if row["feature"] in RENDER_WRAPPERS:
            package, selector = RENDER_WRAPPERS[row["feature"]]
            identifier = "render-wrapper-" + row["feature"]
            add(recipe(identifier, package, group="render-wrapper", selector=selector,
                       reason="render feature wrapper registration contract"))
            recipes["render-" + row["feature"]]["companions"] = [identifier]
    for index, selector in enumerate(MIRI_FILTERS):
        value = recipe("miri-" + str(index), "zircon_runtime", group="miri",
                       features=("native-host",), selector=selector, reason="ECS storage and borrow interpretation")
        value["platform"] = "x86_64-unknown-linux-gnu"
        add(value)
    add(recipe("portable-core", "zircon_runtime", group="portable", features=("core-min",), verb="check"))
    recipes["portable-core"]["platform"] = "wasm32-unknown-unknown"
    profiles = tomllib.loads((root / "zircon_runtime/runtime-feature-presets.toml").read_text(encoding="utf-8"))
    for profile in profiles["profiles"]:
        add(recipe("profile-" + profile["id"], "zircon_app", group="profile",
                   features=(profile["cargo_feature"],), verb="check"))
    for prefix, (feature, _) in RUNTIME_RULES.items():
        identifier = "domain-" + feature
        if identifier not in recipes:
            add(recipe(identifier, "zircon_runtime", group="domain", features=(feature,), verb="check"))
    for feature in DOMAIN_FEATURES:
        identifier = "domain-" + feature
        if identifier not in recipes:
            add(recipe(identifier, "zircon_runtime", group="domain", features=(feature,), verb="check"))
    add(recipe("export-policy", "zircon_runtime", group="export", features=("native-host",),
               selector=PLATFORM_FILTER, reason="shared export policy: original eight cases in one test"))
    recipes["export-policy"]["export_platform"] = None
    for platform in EXPORT_PLATFORMS:
        identifier = "export-policy-" + platform
        add(recipe(identifier, "zircon_runtime", group="export", features=("native-host",),
                   selector=PLATFORM_FILTER, reason="selected export policy input: " + platform))
        recipes[identifier]["export_platform"] = platform
    return recipes


def non_code(path):
    suffix = Path(path).suffix.lower()
    # Docs, agent notes, and trees outside both Cargo workspaces have no compile owner.
    # .codex/config.toml, Jenkins overlap, and other tools stay out of the recipe plan.
    return (path.startswith("docs/") and suffix in {".md", ".txt", ".svg", ".png", ".jpg", ".json"}
            or path in {"LICENSE", "README.md", "AGENTS.md"} or path.endswith("/README.md")
            or path.startswith(".github/") and suffix in {".py", ".ps1", ".cmd", ".yml", ".yaml", ".gitignore"}
            or path.startswith(("tools/", "third_party/", ".codex/")))


def plan_paths(paths, packages, recipes, selected=()):
    explicit = set(selected)
    need(explicit <= recipes.keys(), "Unknown recipe IDs; use catalog to list valid --select IDs.")
    picked, unresolved = set(), []
    public_controls = {"Cargo.toml", "Cargo.lock", "zircon_plugins/Cargo.toml",
                       "zircon_plugins/Cargo.lock", "rust-toolchain.toml", "rust-toolchain"}
    for path in paths:
        if non_code(path):
            continue
        if path in EXPORT_POLICY_SOURCES:
            picked.add("export-policy")
            continue
        owners = [p for p in packages.values() if path.startswith(p["directory"] + "/")]
        if path in public_controls or path.endswith("/build.rs"):
            # Plugin lock edges are covered by the library recipes of the manifests
            # in the same change. A lock-only edit, including Windows target churn,
            # still needs its own reviewed rule.
            if path == "zircon_plugins/Cargo.lock" and any(
                    item.startswith("zircon_plugins/") and item.endswith("/Cargo.toml") for item in paths):
                continue
            unresolved.append(path + ": shared build control; add a reviewed impact rule before compilation")
            continue
        if not owners:
            unresolved.append(path + ": no declared Cargo owner; add a reviewed owner contract")
            continue
        owner = max(owners, key=lambda p: len(p["directory"]))
        name = owner["name"]
        relative = path[len(owner["directory"]) + 1:]
        candidates = set()
        if name == "zircon_runtime":
            if relative.startswith("src/"):
                local = relative[4:]
                if any(local == boundary or local.startswith(boundary) for boundary in RUNTIME_CFG_BOUNDARIES):
                    unresolved.append(path + ": cfg/backend boundary needs reviewed finite feature contracts; "
                                      "native-host and unrelated core tests cannot cover this source")
                    continue
                matches = [prefix for prefix in RUNTIME_RULES if local.startswith(prefix)]
                if matches:
                    prefix = max(matches, key=len)
                    if prefix == "core/":
                        unresolved.append(path + ": shared core owner needs a reviewed cfg-specific contract")
                        continue
                    candidates.add("runtime-" + prefix.rstrip("/").replace("/", "-"))
                    if prefix.startswith("scene/ecs") or local.startswith(("scene/ecs/", "scene/tests/ecs")):
                        candidates.update("miri-" + str(i) for i in range(3))
                    candidates.add("domain-" + RUNTIME_RULES[prefix][0])
                    if portable_source(local):
                        candidates.add("portable-core")
                    if local.startswith(("graphics/", "render_graph/")):
                        candidates.update("render-" + row["feature"] for row in RENDER_ROWS)
                elif local == "rhi.rs":
                    # Facade reexports the reviewed zr_rhi leaf and its existing callers.
                    candidates.update(("leaf-zr_rhi", "leaf-zr_rhi_wgpu", "runtime-graphics"))
            if not candidates:
                unresolved.append(path + ": shared Runtime boundary; register explicit module/consumer contracts")
        elif name in LEAVES:
            candidates.add("leaf-" + name)
            if relative.endswith(".rs") or relative == "Cargo.toml":
                callers = LEAF_CONSUMERS.get(name, {})
                for dependent in sorted(p["name"] for p in packages.values() if name in p["dependencies"]):
                    rid = callers.get(dependent)
                    if rid in recipes:
                        candidates.add(rid)
                    else:
                        unresolved.append(path + ": uncovered reverse consumer " + dependent + "; add a reviewed caller recipe")
        elif "plugin-" + name in recipes:
            candidates.add("plugin-" + name)
            candidates.update("render-" + feature for feature, source_owner in RENDER_SOURCE_OWNERS.items()
                              if source_owner == name)
            if relative in {"Cargo.toml", "src/lib.rs"}:
                dependents = sorted(p["name"] for p in packages.values() if name in p["dependencies"])
                # Dist packages and first-party plugin/editor libraries are the reviewed callers.
                for dependent in dependents:
                    covered = False
                    for rid in ("dist-" + dependent, "plugin-" + dependent):
                        if rid in recipes:
                            candidates.add(rid)
                            covered = True
                    if not covered:
                        unresolved.append(path + ": uncovered reverse consumer " + dependent)
        elif "dist-" + name in recipes:
            candidates.add("dist-" + name)
        elif name == "zircon_editor" and "editor-zircon_editor" in recipes:
            candidates.add("editor-zircon_editor")
        elif name == "zircon_app" and relative.startswith("src/entry/"):
            # Profile is explicitly chosen because this common entry serves different targets.
            candidates.update(rid for rid in explicit if rid.startswith("profile-"))
            if not candidates:
                unresolved.append(path + ": choose applicable --select profile-ID; catalog lists valid IDs")
        else:
            unresolved.append(path + ": owner has no reviewed scoped recipe: " + name)
        picked.update(candidates)
    need(not unresolved, "Uncovered impact (explicit selections cannot waive this):\n" + "\n".join(unresolved))
    # An explicit recipe supplements coverage, never erases or overrides mandatory recipes.
    picked.update(explicit)
    if any(identifier.startswith("miri-") for identifier in picked):
        picked.update("miri-" + str(index) for index in range(3))
    for identifier in tuple(picked):
        picked.update(recipes[identifier].get("companions", ()))
    need(len(picked) <= 32, "Affected contract set exceeds bounded CI plan; review and split the change.")
    compiled_packages = {recipes[r]["package"] for r in picked}
    need(not compiled_packages or compiled_packages != set(packages),
         "Refusing a full package enumeration disguised as recipes")
    return [recipes[r] for r in sorted(picked)]


def make_plan(root, base, head, selected=()):
    need(git(root, "rev-parse", "HEAD").decode().strip() == head, "Checked out HEAD differs from plan head")
    packages = catalog(root)
    recipes = registry(packages, root)
    paths = changed_paths(root, base, head)
    chosen = plan_paths(paths, packages, recipes, selected)
    result = dict(schema=1, base=base, head=head, paths=paths, recipes=chosen,
                  noCompilation=not chosen, reason="known source-only change" if not chosen else "affected owner contracts")
    for group in ("profile", "domain", "dist", "render", "export"):
        result[group] = {"include": [{"recipe": r["id"], **r} for r in chosen if r["group"] == group]}
        result["has_" + group] = bool(result[group]["include"])
    result["has_miri"] = any(r["group"] == "miri" for r in chosen)
    result["has_portable"] = any(r["group"] == "portable" for r in chosen)
    return result


def admission(value):
    need(set(value) <= {"id", "package", "group", "features", "selector", "verb", "target",
                        "profile", "platform", "reason", "manifest", "companions", "export_platform", "kind"},
         "Unrepresented recipe options; raw argv/default-feature overrides are forbidden")
    need(value["verb"] in {"check", "build", "test", "clippy"}, "Unsupported recipe action")
    need(value["kind"] == ("owner-regression" if value["verb"] == "test" else "compile-check"),
         "Recipe kind must distinguish an executed owner regression from compilation")
    need(value["manifest"] in {"Cargo.toml", "zircon_plugins/Cargo.toml"}, "Unsupported manifest")
    need(value["platform"] in {HOST, "wasm32-unknown-unknown"}, "Unsupported target outside pinned Miri lane")
    need(value["target"] == "lib" and value["profile"] == "dev", "Unrepresented target/profile")
    need(re.fullmatch(r"[A-Za-z0-9_-]{1,80}", value["package"]), "Invalid package")
    need(all(re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_-]*", f) for f in value["features"]), "Invalid feature")
    if value["group"] == "export":
        need("export_platform" in value and (value["export_platform"] is None
             or value["export_platform"] in EXPORT_PLATFORMS), "Unknown export policy input")
        need(value["package"] == "zircon_runtime" and value["features"] == ["native-host"]
             and value["selector"] == PLATFORM_FILTER and value["platform"] == HOST
             and value["verb"] == "test", "Unrepresented export policy contract")
    else:
        need("export_platform" not in value, "Export input belongs only to an export policy recipe")
    args = [value["verb"] if value["verb"] != "clippy" else "check",
            "-p", value["package"], "--no-default-features", "--locked", "--lib"]
    if value["features"]:
        args += ["--features", ",".join(value["features"])]
    if value["verb"] == "test":
        need(value["selector"] and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_:]{2,255}", value["selector"]), "Missing bounded test filter")
        args += [value["selector"]]
    spec = importlib.util.spec_from_file_location("ci_management_admission", ROOT / "tools/jenkins-management/entry.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.parse_cargo(args, enforce_policy=True)
    args[0] = value["verb"]
    args += ["--target", value["platform"]]
    if value["verb"] == "clippy":
        args += ["--", "-D", "warnings"]
    return args


_FIXED_HISTORY_CONSUMER_TESTS = (
    'tests::runtime_absorption::code_review_findings::f12_dead_code::review_f12_runtime_production_dead_code_suppression_is_globally_gated',
    'tests::runtime_absorption::code_review_findings::f8_api_convergence::descriptor_builder::first_party_descriptors::review_f8_first_party_runtime_plugin_descriptors_use_builder',
    'tests::runtime_absorption::code_review_findings::f8_api_convergence::descriptor_builder::scaffold::review_f8_runtime_plugin_descriptor_exposes_builder_scaffold',
    'tests::runtime_absorption::code_review_findings::f8_api_convergence::descriptor_builder::test_fixtures::review_f8_runtime_plugin_descriptor_test_fixtures_use_builder',
    'tests::runtime_absorption::code_review_findings::f8_api_convergence::descriptor_privacy::status_mirrors::review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending',
    'tests::runtime_absorption::code_review_findings::f8_api_convergence::texture_import_settings::review_f8_texture_import_settings_use_fallible_apply_not_with',
    'tests::runtime_absorption::code_review_findings::late_api_cleanup::f11_shading_model_registry::review_f11_shading_model_registry_has_no_dead_plugin_registration_surface',
    'tests::runtime_absorption::code_review_findings::late_api_cleanup::f15_editor_pane_data_conversion::review_f15_editor_pane_data_conversion_top_row_uses_projection_owners',
    'tests::runtime_absorption::code_review_findings::late_api_cleanup::f18_asset_manager_resolution::review_f18_asset_manager_resolution_returns_registered_handle',
    'tests::runtime_absorption::code_review_findings::late_api_cleanup::f19_scene_renderer_construction::review_f19_scene_renderer_construction_modules_use_construct_names',
    'tests::runtime_absorption::code_review_findings::p0_robustness::lock_poison::review_f2_scene_eventbus_locks_recover_after_poison',
    'tests::runtime_absorption::code_review_findings::p0_robustness::native_fixture::sdk_macro_manifest::review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest',
    'tests::runtime_absorption::code_review_findings::p0_robustness::native_host_callbacks::review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi',
    'tests::runtime_absorption::code_review_findings::p0_robustness::priority_recommendation::review_priority_recommendation_tracks_current_remaining_work',
    'tests::runtime_absorption::code_review_findings::p0_robustness::render_submit::review_f4_render_submit_capability_gaps_return_typed_errors',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d10_bridge_call::review_d10_animation_physics_tests_use_sdk_bridge_call',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d12_runtime_exports::review_d12_runtime_helper_exports_use_sdk_macro',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d13_importer_sdk::manifest_parity::review_d13_importer_manifest_parity_guard_lives_in_sdk_builder',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d13_importer_sdk::runtime_exports::review_d13_importer_runtime_exports_use_sdk_macro',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d13_importer_sdk::runtime_manifests::review_d13_importer_runtime_manifests_use_sdk_builder',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d5_editor_authoring::review_d5_editor_authoring_plugins_use_sdk_macro',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d6_runtime_plugin_id::review_d6_runtime_plugin_id_accepts_external_string_keys',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d8_registration_builder::review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder',
    'tests::runtime_absorption::code_review_findings::plugin_importer_dx::d9_editor_runtime_mirror::review_d9_editor_runtime_mirror_consumers_use_sdk_declaration',
    'tests::runtime_absorption::code_review_findings::render_structure::review_f16_compiled_scene_render_path_uses_split_owners',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::asset_loaders::artifact_importer::review_f7_asset_artifact_errors_use_asset_import_error_sources',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::asset_loaders::mesh_obj::review_f5_mesh_loader_and_obj_decoder_use_typed_errors',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::asset_records::font::review_f5_font_asset_uses_typed_error_source',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::asset_records::zshader::review_f5_zshader_v2_replaces_user_shader_definitions',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::scene_world::dynamic_components::review_f5_dynamic_component_errors_preserve_scene_error_sources',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::scene_world::fixed_mutation::review_f5_fixed_world_mutation_uses_scene_error_variants',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::scene_world::property_access::review_f5_scene_property_access_uses_scene_error',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::scene_world::typed_mutation_surface::review_f5_world_spawn_bundle_surface_uses_scene_error',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::shader_prewarm_cli::args_boundary::review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary',
    'tests::runtime_absorption::code_review_findings::typed_error_convergence::ui_asset_documents::review_f5_ui_asset_documents_use_typed_errors_before_import_boundary',
    'tests::runtime_absorption::dynamic_scene::split_layout::runtime_15_dynamic_scene_route_owner_is_folder_backed',
    'tests::runtime_absorption::input_stack::inventory::mirror_docs::runtime_12_input_stack_mirror_docs_match_structure_audit_counts',
    'tests::runtime_absorption::job_system::split_layout::runtime_15_job_system_route_owner_is_folder_backed',
    'tests::runtime_absorption::performance_hotspots::artifact_render_diagnostics_splits::artifact_cache_payload::runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed',
    'tests::runtime_absorption::performance_hotspots::owner_budget::large_file_gate::runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit',
    'tests::runtime_absorption::performance_hotspots::scene_project_splits::project_io::runtime_07_project_io_folder_split_keeps_entry_and_converter_owners',
    'tests::runtime_absorption::plugin_surface_lifecycle::mirror_docs::runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts',
    'tests::runtime_absorption::plugin_surface_lifecycle::split_layout::runtime_15_plugin_surface_lifecycle_guard_is_folder_backed',
    'tests::runtime_absorption::rayon_boundary::cutover_status::rayon_render_exception_cutover_is_recorded_in_runtime_11_m2_1_status',
    'tests::runtime_absorption::rayon_boundary::split_layout::runtime_15_rayon_boundary_route_owner_is_folder_backed',
    'tests::runtime_absorption::root_entries::module_families::mirror_docs::runtime_14_module_family_mirror_docs_match_structure_audit_counts',
    'tests::runtime_absorption::script_absorption::split_layout::runtime_15_script_absorption_route_owner_is_folder_backed',
    'tests::runtime_absorption::script_host_ledger::ledger::host_function_registry_matches_documented_ledger',
    'tests::runtime_absorption::tech_stack::mirror_docs::runtime_01_tech_stack_mirror_docs_match_structure_audit_counts',
    'tests::runtime_absorption::tech_stack::split_layout::runtime_15_tech_stack_route_owner_is_folder_backed',
    'tests::runtime_absorption::ui_architecture::architecture_boundaries::runtime_09_taffy_layout_pass_order_uses_bridge_authority',
    'tests::runtime_absorption::ui_architecture::architecture_boundaries::runtime_09_v2_verdict_matches_runtime_and_interface_modules',
    'tests::runtime_absorption::ui_architecture::architecture_boundaries::runtime_09_virtualization_scroll_boundary_records_invalidation_authority',
    'tests::runtime_absorption::ui_architecture::mirror_docs::runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts',
    'tests::runtime_absorption::ui_architecture::split_layout::runtime_15_ui_architecture_route_owner_is_folder_backed',
)


def recipe_consumes_history(value):
    # libtest filters are substrings of full mounted test names.
    # Compile-only/lint and dependency-library recipes do not execute readers.
    if value["package"] != "zircon_runtime" or value["verb"] != "test" or value.get("target") != "lib":
        return False
    selector = value.get("selector")
    if not selector:
        return True
    structure = "tests::runtime_absorption::structure_convention"
    if selector in {"structure_convention", structure, structure + "::"} or selector.startswith(structure + "::"):
        return True
    return any(selector in name for name in _FIXED_HISTORY_CONSUMER_TESTS)


def history_archive_environment(root_value, inherited=None, *, required=False):
    """Pass one explicit read-only historical input; never use ambient fallback."""
    environment = dict(os.environ if inherited is None else inherited)
    environment = {key: value for key, value in environment.items()
                   if key.casefold() not in {"zircon_docs_history_archive_root", "ci_docs_history_archive_root"}}
    if not required:
        return environment
    need(root_value is not None, "Historical guard recipe requires explicit archive root")
    need(isinstance(root_value, str) and root_value and len(root_value.encode("utf-8")) <= 4096
         and not any(c in root_value for c in "\0\r\n"), "Invalid fixed historical archive root")
    root = Path(root_value)
    need(root.is_absolute(), "Fixed historical archive root must be absolute")
    root = root.resolve(strict=True)
    need(root.is_dir(), "Fixed historical archive root must be a readable directory")
    with os.scandir(root):
        pass
    environment["ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"] = str(root)
    return environment


def execute(root, result, recipe_id=None, group=None, history_archive_root=None):
    wanted = [r for r in result["recipes"] if r["id"] == recipe_id or group and r["group"] == group]
    need(wanted or group == "rust", "Recipe/group is absent from the recomputed affected plan")
    if not wanted:
        print(json.dumps({"group": group, "status": "no affected Rust library recipe; formatting/source gates only"}))
    for value in wanted:
        need(value["group"] != "miri", "Miri uses the fixed pinned workflow/sysroot lane")
        need(value["group"] != "dist",
             "Native artifact acceptance pending: cargo-zircon plugin check has no per-plugin selection; "
             "implement a reviewed scoped artifact validator before admitting this dist recipe. "
             "The source-only standalone manifest gate is not artifact acceptance.")
        args = admission(value)
        print(json.dumps({"declaration": value, "cargo": args}, sort_keys=True), flush=True)
        command = ["pwsh", "-NoProfile", "-File", str(root / ".github/scripts/invoke-cargo-msvc.ps1")]
        if value["verb"] == "test":
            command += ["-RequireMatchingTests"]
        command += ["cargo", args[0], "--manifest-path", value["manifest"], *args[1:]]
        environment = history_archive_environment(history_archive_root, required=recipe_consumes_history(value))
        if value["group"] == "export":
            # No policy input means the original Rust all_cases loop checks every platform.
            environment.pop("ZR_EXPORT_CONTRACT_PLATFORM", None)
            if value["export_platform"] is not None:
                environment["ZR_EXPORT_CONTRACT_PLATFORM"] = value["export_platform"]
        subprocess.run(command, cwd=root, check=True, env=environment)
        for identifier in value.get("companions", ()):
            execute(root, result, recipe_id=identifier, history_archive_root=history_archive_root)
        if group == "rust" and value["verb"] in {"check", "build", "test"}:
            lint = {**value, "verb": "clippy", "selector": None, "kind": "compile-check"}
            execute_single = admission(lint)
            print(json.dumps({"declaration": lint, "cargo": execute_single}), flush=True)
            subprocess.run(["pwsh", "-NoProfile", "-File", str(root / ".github/scripts/invoke-cargo-msvc.ps1"),
                            "cargo", execute_single[0], "--manifest-path", lint["manifest"], *execute_single[1:]],
                           cwd=root, check=True, env=history_archive_environment(None, environment))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("plan", "run", "catalog"))
    parser.add_argument("--base")
    parser.add_argument("--head")
    parser.add_argument("--select", action="append", default=[])
    parser.add_argument("--recipe")
    parser.add_argument("--history-archive-root", default=os.environ.get("CI_DOCS_HISTORY_ARCHIVE_ROOT") or None,
                        help="Explicit read-only historical archive directory; no checkout fallback")
    parser.add_argument("--group", choices=("rust",))
    parser.add_argument("--github-output", action="store_true")
    args = parser.parse_args(argv)
    if args.action == "catalog":
        print(json.dumps(registry(catalog(ROOT), ROOT), sort_keys=True))
        return 0
    result = make_plan(ROOT, args.base, args.head, args.select)
    if args.action == "run":
        need(bool(args.recipe) != bool(args.group), "Select exactly one --recipe or --group")
        execute(ROOT, result, args.recipe, args.group, history_archive_root=args.history_archive_root)
    elif args.github_output:
        destination = os.environ.get("GITHUB_OUTPUT")
        need(destination, "GITHUB_OUTPUT is required")
        with open(destination, "a", encoding="utf-8") as output:
            for name in ("profile", "domain", "dist", "render", "has_profile", "has_domain", "has_dist",
                         "has_render", "has_miri", "has_portable", "export", "has_export", "base", "head"):
                encoded = result[name] if isinstance(result[name], str) else json.dumps(result[name], separators=(",", ":"))
                output.write(name + "=" + encoded + "\n")
        print(json.dumps(result, sort_keys=True))
    else:
        print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ScopeError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print("CI scope refused: " + str(error), file=sys.stderr)
        raise SystemExit(2)
