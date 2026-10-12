"""Deterministic readable names for the authorized engine path migration."""
from __future__ import annotations

from collections import Counter, defaultdict
from pathlib import PurePosixPath
import re

from .path_names import RELATIVE_LIMIT, path_units

# Physical names only; Rust API/module identities are preserved by their mounts.
DIRECTORY_NAMES = {
 "package_receipt":"receipt", "shader_invocation":"shaders",
 "runtime_text_multilingual_product_framebuffer":"text_pixels",
 "retained_host":"retained", "host_contract":"contract", "paint_template_nodes":"paint",
 "scene_renderer":"renderer", "runtime_absorption":"contracts",
 "structure_convention":"structure", "optional_feature_manifest":"features",
 "test_file_budget":"test_size", "production_file_budget":"source_size",
 "code_review_findings":"review", "pane_data_conversion":"data",
 "pane_component_projection":"projection", "material_primitives":"primitives",
 "runtime_plugin":"runtime", "package_validation":"validation",
 "native_plugin_loader":"loader", "runtime_plugin_catalog":"catalog",
 "paint_workbench_renderer":"workbench", "callback_dispatch":"dispatch",
 "render_framework":"framework", "plugin_extensions":"plugins",
 "static_manifest_contracts":"manifests", "editor_manager_plugins_export":"exports",
 "submit_frame_extract":"extract", "retained_callback_dispatch":"callbacks",
 "typed_error_convergence":"errors", "typed_error_owners":"errors",
 "folder_backed_summary":"folders", "direct_review_assertions":"assertions",
 "structure_guard_children":"children", "root_output_sources":"outputs",
 "root_state_readbacks":"readbacks", "virtual_geometry":"geometry",
 "hybrid_gi":"gi", "root_render_passes":"passes", "gpu_resources":"gpu",
 "virtual_geometry_executed_cluster_selection_pass":"cluster_select",
 "virtual_geometry_node_and_cluster_cull_pass":"cluster_cull",
 "seed_backed_execution_selection":"seeds",
 "virtual_geometry_gpu_resources":"resources",
 "virtual_geometry_runtime_state":"state", "virtual_geometry_indirect_stats":"stats",
 "virtual_geometry_debug_snapshot":"debug", "virtual_geometry_gpu_readback":"readback",
 "virtual_geometry_prepare":"prepare", "render_pass_executors":"executors",
 "hybrid_gi_runtime_state":"state", "hybrid_gi_resolve_runtime":"resolve",
 "hybrid_gi_gpu_readback":"readback", "hybrid_gi_render_framework_stats":"stats",
 "dynamic_scene":"dynamic", "dynamic_scene_session":"session",
 "scene_renderer_core_render_compiled_scene":"render",
 "scene_renderer_core_construct":"construct", "scene_renderer_core":"core",
 "scene_renderer_render_with_pipeline":"pipeline", "scene_renderer_runtime_outputs":"outputs",
 "post_process":"post", "render_backend":"backend", "system_texture_generation_owner":"textures",
 "performance_hotspots":"performance", "resource_owner_boundary":"ownership",
 "resource_foundation":"resources", "native_pointer":"pointer",
 "button_dispatch":"buttons", "scroll_dispatch":"scroll", "move_dispatch":"motion",
 "template_bridge":"templates", "extension_module_navigation":"navigation",
 "extension_module_feedback":"feedback", "template_material_feedback":"feedback",
 "style_selector":"selector", "graph_execution":"execution",
 "project_asset_manager":"assets", "default_editor_asset_manager":"assets",
 "editor_asset_manager":"assets", "workbench_host_window":"window",
 "retained_menu_pointer":"menus", "ui_asset_editor_detail_events":"asset_events",
 "render_pass_execution_context":"context", "native_plugin_live_host":"live",
 "builtin_render_feature_descriptor":"builtin", "plugin_importer_dx_owners":"importers",
 "render_pipeline_asset":"pipeline", "render_command_conversion":"commands",
 "viewport_overlay_renderer":"overlay", "zui_visual_acceptance":"visual",
 "render_asset_residency":"residency", "module_plugin_actions":"plugins",
 "build_frame_submission_context":"submission", "render_product_mesh_cache":"meshes",
 "runtime_input_reply_routes":"input", "capability_dependencies":"dependencies",
 "build_virtual_geometry_plan":"geometry_plan", "ui_asset_detail_fields":"asset_fields",
 "effect_stack_settings":"effects", "pane_payload_builders":"payloads",
 "chrome_template_projection":"projection", "render_shader_template_assembly":"shaders",
 "runtime_builtin_graphics":"graphics", "from_extract_with_history":"history",
 "scene_frame_history_textures":"history", "render_graph_execution_resources":"resources",
 "render_framework_state":"state", "project_manifest_validation":"manifest",
 "project_plugin_registry_material_passes_staged_cache":"material_cache",
 "workbench_extension_module_template_bindings":"bindings",
 "workbench_window_projection":"windows", "workbench_toolbar_breakpoints":"breakpoints",
 "pending_command_cache_extract":"command_cache", "capability_status_references":"references",
 "capability_status_targets":"targets", "capability_statuses":"statuses",
 "feature_registration_report":"registration", "native_plugin_load_report":"report",
 "compiled_render_pipeline":"compiled", "render_pass_executor_registry":"registry",
 "deferred_scene_resources":"resources", "mesh_draw_command_list":"commands",
 "resource_streamer_ensure_material":"ensure_material", "global_sdf_scene_state":"sdf_state",
 "world_space_submission":"world", "artifact_render_diagnostics_splits":"diagnostics",
 "p0_native_fixture_leaf_owners":"fixtures", "native_material_painter_mui_primitives":"mui",
 "runtime_feature_definitions":"definitions", "build_export_wizard_session":"wizard",
 "contribution_duplicates":"duplicates", "advanced_plugin_resources":"resources",
 "advanced_plugin_readbacks":"readbacks", "gpu_texture_resource_from_asset":"asset_texture",
 "source_cubemap_environment":"cubemap", "workbench_window_menus":"menus",
 "build_virtual_geometry_debug_snapshot":"debug", "render_frame_with_pipeline":"pipeline",
 "graphics_debugger_capture":"capture", "compile_options_for_profile":"profile",
 "virtual_geometry_hardware_rasterization_pass":"raster",
 "virtual_geometry_visbuffer64_pass":"visibility",
}
PREFIXES = (
 "execute_screen_space_reflection_", "execute_screen_space_reflection",
 "template_viewport_scene_", "template_", "workbench_", "scene_renderer_",
)
SHORT_WORDS = {
 "performance":"perf", "optimization":"perf", "allocation":"alloc",
 "allocations":"allocs", "construction":"construct", "deserialization":"decode",
 "serialization":"encode", "configuration":"config", "validation":"validate",
 "diagnostics":"diagnostics", "diagnostic":"diagnostic", "projection":"project",
 "generation":"generate", "registration":"register", "composition":"compose",
 "initialization":"init", "implementation":"impl", "collection":"collect",
 "component":"component", "management":"manage", "publication":"publish",
 "completion":"complete", "descriptor":"desc", "descriptors":"descs",
}


PATH_NAMES = {'zircon_editor/src/ui/layouts/views/assets_activity/tests/responsive_layout_tests.rs': 'zircon_editor/src/ui/layouts/views/assets_activity/tests/narrow_controls.rs', 'zircon_runtime/src/core/framework/animation/compiler/tests/graph_optimization_batch_hc_runtime584_tests.rs': 'zircon_runtime/src/core/framework/animation/compiler/tests/graph_refs_perf.rs', 'zircon_runtime/src/graphics/scene/scene_renderer/core/tests/scene_renderer_render_scene.rs': 'zircon_runtime/src/graphics/scene/renderer/core/tests/scene_admission.rs', 'zircon_plugins/net/features/content_download/runtime/src/manager/tests/bitmap_completion_index_tests.rs': 'zircon_plugins/net/features/content_download/runtime/src/tests/bitmap.rs', 'zircon_plugins/neural/features/post_process/runtime/src/tests/lib_registration_tests.rs': 'zircon_plugins/neural/features/post_process/runtime/src/tests/register.rs', 'zircon_plugins/rendering/features/contact_shadow/runtime/src/tests/wgpu_product_tests.rs': 'zircon_plugins/rendering/features/contact_shadow/runtime/src/tests/gpu.rs', 'zircon_plugins/rendering/features/reflection_probes/editor/src/capture/tests/publication.rs': 'zircon_plugins/rendering/features/reflection_probes/editor/src/tests/publish.rs', 'zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/tests/face_view.rs': 'zircon_plugins/rendering/features/reflection_probes/runtime/src/tests/faces.rs', 'zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests/wgpu_product_tests.rs': 'zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests/gpu.rs', 'zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests/tests/renderdoc_capture.rs': 'zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests/capture.rs'}


def short_name(name: str, *, leaf: bool = False) -> str:
    suffix = PurePosixPath(name).suffix if leaf else ""
    stem = name[:-len(suffix)] if suffix else name
    if stem in DIRECTORY_NAMES:
        stem = DIRECTORY_NAMES[stem]
    elif stem.startswith("execute_screen_space_reflection"):
        stem = re.sub(r"^execute_screen_space_reflection_?", "ssr_", stem)
        stem = stem.replace("reflection_", "")
    elif stem.startswith("template_viewport_scene_"):
        stem = "scene_" + stem[len("template_viewport_scene_"):]
    elif stem.startswith(("template_","workbench_")):
        stem = stem.split("_", 1)[1]
    if leaf:
        stem = re.sub(r"_?optimization_batch_[a-z0-9]+(?:_(?:runtime|editor)(?:[a-z0-9]+)?)?(?:_[a-z]+)?_tests$", "_perf", stem)
        stem = re.sub(r"_?optimization_batch_\d+[a-z]+.*$", "_perf", stem)
        stem = re.sub(r"_?(?:performance|optimization)_tests$", "_perf", stem)
        stem = re.sub(r"_tests$", "", stem)
        if len(stem)>26:
            stem = "_".join(SHORT_WORDS.get(word,word) for word in stem.split("_"))
    return stem.strip("_") + suffix


def candidate(value: str) -> str:
    parts = value.split("/")
    # Crate names and configured source/resource roots stay fixed.
    anchor = 3 if parts[0]=="zircon_plugins" else 2
    if parts[0]=="zircon_plugins":
        for n,part in enumerate(parts[2:],2):
            if part in {"src","assets","tests","build","fixtures"}:
                anchor=n+1;break
    fixed=parts[:anchor]
    rest=parts[anchor:]
    mapped=[short_name(part,leaf=n==len(rest)-1) for n,part in enumerate(rest)]
    return "/".join(fixed+mapped)


def compact(value: str) -> str:
    """Remove redundant physical layers; keep owner, test scope and leaf role."""
    parts=value.split("/")
    anchor=3 if parts[0]=="zircon_plugins" else 2
    if parts[0]=="zircon_plugins":
        for n,part in enumerate(parts[2:],2):
            if part in {"src","assets","tests","build","fixtures"}:
                anchor=n+1;break
    # Short words and repeated domain prefixes do not change a file's role.
    for n in range(anchor,len(parts)):
        leaf=n==len(parts)-1
        suffix=PurePosixPath(parts[n]).suffix if leaf else ""
        stem=parts[n][:-len(suffix)] if suffix else parts[n]
        if len(stem)>24:
            words=[SHORT_WORDS.get(w,w) for w in stem.split("_")]
            context=set(w for parent in parts[anchor:n] for w in parent.split("_"))
            while len(words)>1 and words[0] in context:words.pop(0)
            stem="_".join(words)
        parts[n]=stem+suffix
    # Keep tests as a physical distinction from production siblings.
    while path_units("/".join(parts))>RELATIVE_LIMIT and len(parts)>anchor+1:
        choices=[n for n in range(anchor,len(parts)-1) if parts[n]!="tests"]
        if not choices:break
        # Intermediate containers are cheaper to remove than a direct owner.
        last=len(parts)-2
        choices.sort(key=lambda n:(n==anchor,n==last,-len(parts[n]),n))
        parts.pop(choices[0])
    return "/".join(parts)


def alternatives(original: str, current: str) -> list[str]:
    """Use semantic parent words when collapsed branches share a filename."""
    source=PurePosixPath(candidate(original))
    target=PurePosixPath(current)
    leaf=target.stem
    parents=[p for p in reversed(source.parts[:-1]) if p not in target.parts[:-1]]
    results=[]
    for parent in parents:
        words=parent.split("_")
        contexts=[parent]
        if len(words)>1:
            contexts.extend(["_".join(words[-2:]),words[-1],words[0]])
        for context in contexts:
            name=context+"_"+leaf+target.suffix
            value=compact((target.parent/name).as_posix())
            if path_units(value)<=RELATIVE_LIMIT:results.append(value)
    # Shorten only complete words when a long leaf has no room for its owner.
    words=leaf.split("_")
    if len(words)>2:
        for count in range(len(words)-1,1,-1):
            for brief in ("_".join(words[:count]),"_".join(words[-count:])):
                value=compact((target.parent/(brief+target.suffix)).as_posix())
                if path_units(value)<=RELATIVE_LIMIT:results.append(value)
                for parent in parents:
                    for context in (parent.split("_")[-1],parent.split("_")[0]):
                        value=compact((target.parent/(context+"_"+brief+target.suffix)).as_posix())
                        if path_units(value)<=RELATIVE_LIMIT:results.append(value)
    # Combined parent qualifiers distinguish same-named leaves in deep branches.
    context_words=list(dict.fromkeys(
        SHORT_WORDS.get(w,w) for part in reversed(source.parts[2:-1])
        for w in part.split("_") if w not in {"src","tests","runtime","core","structure","support"}
    ))
    for a in context_words:
        for b in context_words:
            if a==b:continue
            brief=leaf if len(leaf)<24 else "_".join(leaf.split("_")[-2:])
            value=compact((target.parent/(a+"_"+b+"_"+brief+target.suffix)).as_posix())
            if path_units(value)<=RELATIVE_LIMIT:results.append(value)
    return list(dict.fromkeys(results))


def make_moves(paths: list[str]) -> tuple[dict[str,str],list[dict]]:
    long_dirs=set()
    for value in paths:
        if path_units(value)>RELATIVE_LIMIT:
            long_dirs.update(parent.as_posix() for parent in PurePosixPath(value).parents)
    mapping={}
    for value in paths:
        if path_units(value)>RELATIVE_LIMIT or any(
            parent.as_posix() in long_dirs and short_name(parent.name)!=parent.name
            for parent in PurePosixPath(value).parents
        ):
            new=compact(candidate(value))
            if new!=value:mapping[value]=new
    for old,new in PATH_NAMES.items():
        if old in mapping:mapping[old]=new
    # Shader manifests and their code are a package: retain sibling filenames.
    for role in ("compute","fullscreen"):
        for extension in ("wgsl","zshader"):
            old=f"zircon_runtime/tests/fixtures/shader_invocation/{role}_binding_probe/{role}_binding_probe.{extension}"
            if old in mapping:
                mapping[old]=f"zircon_runtime/tests/fixtures/shaders/{role}/{role}_binding_probe.{extension}"
    # A preexisting short destination always wins; moved files choose a distinct,
    # descriptive parent qualifier rather than overwriting or using a hash.
    occupied={p.casefold() for p in paths if p not in mapping}
    issues=[]
    for old in sorted(mapping,key=lambda p:(path_units(mapping[p])>RELATIVE_LIMIT,p)):
        new=mapping[old]
        if new.casefold() in occupied or path_units(new)>RELATIVE_LIMIT:
            options=alternatives(old,new)
            new=next((v for v in options if v.casefold() not in occupied),new)
        if new.casefold() in occupied or path_units(new)>RELATIVE_LIMIT:
            issues.append({"path":old,"candidate":new,"reason":"readable-name-collision-or-budget"})
        occupied.add(new.casefold());mapping[old]=new
    return mapping,issues
