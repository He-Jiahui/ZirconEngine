mod module_layout;
mod renderer_output_accessors;

use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_offscreen_target_texture_owner_cleanup() {
    let offscreen_target = read_runtime_src("graphics/backend/render_backend/offscreen_target.rs");
    let offscreen_construct =
        read_runtime_src("graphics/backend/render_backend/offscreen_target_new/construct.rs");
    let frame_graph_binder = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !offscreen_target.contains("#[allow(dead_code)]"),
        "OffscreenTarget texture owners should be live ownership contracts, not dead-code suppressions"
    );
    assert_contains_all(
        "offscreen retained WGPU texture owners",
        &offscreen_target,
        &[
            "pub(crate) const RETAINED_FRAME_TEXTURE_COUNT: usize = 9;",
            "pub(crate) fn retained_frame_texture_count(&self) -> usize",
            "&self.final_color",
            "&self.global_illumination",
            "&self.scene_color",
            "&self.bloom",
            "&self.gbuffer_albedo",
            "&self.gbuffer_material",
            "&self.normal",
            "&self.ambient_occlusion",
            "&self.depth",
        ],
    );
    assert_contains_all(
        "offscreen construction still materializes every retained owner",
        &offscreen_construct,
        &[
            "final_color: final_color.texture",
            "global_illumination: global_illumination.texture",
            "scene_color: scene_color.texture",
            "bloom: bloom.texture",
            "gbuffer_albedo: gbuffer_albedo.texture",
            "gbuffer_material: gbuffer_material.texture",
            "normal: normal.texture",
            "ambient_occlusion: ambient_occlusion.texture",
            "depth,",
        ],
    );
    assert_contains_all(
        "compiled-scene frame graph binder consumes retained owner contract",
        &frame_graph_binder,
        &[
            "target.retained_frame_texture_count()",
            "OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT",
            "fixed offscreen frame target must retain every WGPU texture owner backing imported views",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 offscreen target texture owner cleanup",
                "runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result",
                "runtime_15_offscreen_target_texture_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_render_backend_state_owner_cleanup() {
    let render_backend = read_runtime_src("graphics/backend/render_backend/render_backend.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !render_backend.contains("#[allow(dead_code)]"),
        "RenderBackend state owners should be live ownership contracts, not dead-code suppressions"
    );
    assert_contains_all(
        "render backend retained state owner contract",
        &render_backend,
        &[
            "pub(crate) const RETAINED_STATE_OWNER_COUNT: usize = 3;",
            "pub(crate) fn retained_state_owner_count(&self) -> usize",
            "&self.instance",
            "&self.adapter",
            "&self.config",
            "self.retained_state_owner_count()",
            "RenderBackend must retain instance, adapter, and config owners while reporting caps",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 render backend state owner cleanup",
                "runtime_15_render_backend_state_owner_cleanup_coremin_check_passed",
                "runtime_15_render_backend_state_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_gpu_texture_resource_owner_cleanup() {
    let gpu_texture =
        read_runtime_src("graphics/scene/resources/gpu_texture/gpu_texture_resource.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !gpu_texture.contains("#[allow(dead_code)]"),
        "GpuTextureResource identity and WGPU owners should be live binding contracts, not dead-code suppressions"
    );
    assert_contains_all(
        "gpu texture retained owner contract",
        &gpu_texture,
        &[
            "pub(crate) const RETAINED_TEXTURE_BINDING_OWNER_COUNT: usize = 4;",
            "pub(crate) fn retained_texture_binding_owner_count(&self) -> usize",
            "&self.id",
            "&self.texture",
            "&self.view",
            "&self.sampler",
            "self.retained_texture_binding_owner_count()",
            "GpuTextureResource must retain identity, texture, view, and sampler while exposing bindings",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 gpu texture resource owner cleanup",
                "runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed",
                "runtime_15_gpu_texture_resource_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_gpu_material_uniform_owner_cleanup() {
    let gpu_material_uniform = read_runtime_src(
        "graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs",
    );
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !gpu_material_uniform.contains("#[allow(dead_code)]"),
        "GpuMaterialUniformResource buffer and byte-length diagnostics should be live binding contracts, not dead-code suppressions"
    );
    assert_contains_all(
        "gpu material uniform retained owner contract",
        &gpu_material_uniform,
        &[
            "pub(crate) const RETAINED_MATERIAL_UNIFORM_OWNER_COUNT: usize = 3;",
            "pub(crate) fn retained_material_uniform_owner_count(&self) -> usize",
            "&self.buffer",
            "&self.payload_byte_len",
            "&self.buffer_byte_len",
            "self.retained_material_uniform_owner_count()",
            "GpuMaterialUniformResource must retain buffer and byte-length diagnostics while exposing uniform bindings",
            "pub(crate) fn payload_byte_len(&self) -> u64",
            "pub(crate) fn buffer_byte_len(&self) -> u64",
        ],
    );
    assert_contains_all(
        "resource streamer consumes material uniform diagnostics through owner accessors",
        &resource_streamer_accessors,
        &[
            "prepared.uniform.payload_byte_len()",
            "prepared.uniform.buffer_byte_len()",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 gpu material uniform owner cleanup",
                "runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed",
                "runtime_15_gpu_material_uniform_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_gpu_mesh_order_signature_cleanup() {
    let gpu_mesh = read_runtime_src("graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs");
    let gpu_mesh_from_asset =
        read_runtime_src("graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs");
    let mesh_draw_builder = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !gpu_mesh.contains("#[allow(dead_code)]"),
        "GpuMeshResource indirect order signature should be live draw ordering input, not a dead-code suppression"
    );
    assert_contains_all(
        "gpu mesh indirect order signature resource contract",
        &gpu_mesh,
        &[
            "indirect_order_signature: u64",
            "pub(crate) const fn indirect_order_signature(&self) -> u64",
            "self.indirect_order_signature",
        ],
    );
    assert_contains_all(
        "gpu mesh upload still derives the complete indirect order signature",
        &gpu_mesh_from_asset,
        &[
            "let indirect_order_signature = indirect_order_signature(&payload);",
            "hash = fnv1a_f32_slice(hash, &vertex.position);",
            "hash = fnv1a_f32_slice(hash, &vertex.normal);",
            "hash = fnv1a_f32_slice(hash, &vertex.uv);",
            "hash = fnv1a_u16_slice(hash, &vertex.joint_indices);",
            "hash = fnv1a_f32_slice(hash, &vertex.joint_weights);",
            "hash = fnv1a_f32_slice(hash, &vertex.tangent);",
            "hash = fnv1a_f32_slice(hash, &vertex.color);",
            "hash = fnv1a_u32(hash, *index);",
        ],
    );
    assert_contains_all(
        "prepared mesh draw sorting consumes the mesh order signature",
        &mesh_draw_builder,
        &[
            "mesh.indirect_order_signature()",
            "mesh_order_command_sort_tie_breaker(",
            "fn mesh_order_command_sort_tie_breaker(",
            "let stable_instance_key =",
            "stable_instance_key.hash(&mut hasher);",
            "mesh_order_signature.hash(&mut hasher);",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 gpu mesh order signature cleanup",
                "runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed",
                "runtime_15_gpu_mesh_order_signature_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_gpu_model_identity_cleanup() {
    let gpu_model = read_runtime_src("graphics/scene/resources/gpu_model/gpu_model_resource.rs");
    let gpu_model_from_asset =
        read_runtime_src("graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs");
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !gpu_model.contains("#[allow(dead_code)]"),
        "GpuModelResource identity should be a live streamer cache contract, not a dead-code suppression"
    );
    assert_contains_all(
        "gpu model identity resource contract",
        &gpu_model,
        &[
            "pub(super) id: ResourceId",
            "pub(crate) const fn id(&self) -> ResourceId",
            "self.id",
        ],
    );
    assert_contains_all(
        "gpu model upload still records the resource identity",
        &gpu_model_from_asset,
        &[
            "id: ResourceId",
            "Self {",
            "id,",
            "meshes: model_primitives_preferring_mesh_assets(asset, load_mesh_asset)",
        ],
    );
    assert_contains_all(
        "resource streamer validates model identity on cache lookup",
        &resource_streamer_accessors,
        &[
            "pub(crate) fn model(&self, id: &ResourceId) -> Option<&Arc<GpuModelResource>>",
            "debug_assert_eq!(",
            "prepared.resource.id()",
            "*id",
            "GpuModelResource identity must match the ResourceStreamer model key",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 gpu model identity cleanup",
                "runtime_15_gpu_model_identity_cleanup_coremin_check_passed",
                "runtime_15_gpu_model_identity_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_post_process_lut_texture_owner_cleanup() {
    let lut_texture = read_runtime_src(
        "graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs",
    );
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !lut_texture.contains("#[allow(dead_code)]"),
        "PostProcessLutTextureResource texture owner should be a live LUT binding contract, not a dead-code suppression"
    );
    assert_contains_all(
        "post-process LUT retained texture owner contract",
        &lut_texture,
        &[
            "pub(in crate::graphics::scene::resources) const RETAINED_LUT_TEXTURE_OWNER_COUNT: usize = 2;",
            "pub(in crate::graphics::scene::resources) fn retained_lut_texture_owner_count(&self) -> usize",
            "let _retained_lut_texture_owners = (&self.texture, &self.view);",
            "pub(in crate::graphics::scene::resources) fn view(&self) -> &wgpu::TextureView",
            "PostProcessLutTextureResource must retain texture and view while exposing 3D LUT bindings",
        ],
    );
    assert_contains_all(
        "resource streamer consumes LUT owner accessor for 3D bindings",
        &resource_streamer_accessors,
        &[
            "pub(crate) fn prepared_post_process_lut_3d_view(",
            ".matches_texture_3d(&prepared.resource.descriptor)",
            ".then_some(prepared.resource.view())",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 post-process LUT texture owner cleanup",
                "runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed",
                "runtime_15_post_process_lut_texture_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_output_target_texture_owner_cleanup() {
    let output_target = read_runtime_src(
        "graphics/scene/resources/output_target_texture/output_target_texture_resource.rs",
    );
    let prepared_output_target =
        read_runtime_src("graphics/scene/resources/prepared/prepared_output_target_texture.rs");
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let resource_streamer_ensure = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs",
    );
    let resource_streamer_writeback = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs",
    );
    let frame_graph_binder = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    for (label, source) in [
        ("OutputTargetTextureResource", output_target.as_str()),
        (
            "PreparedOutputTargetTexture",
            prepared_output_target.as_str(),
        ),
    ] {
        assert!(
            !source.contains("#[allow(dead_code)]"),
            "{label} owner fields should be live output target contracts, not dead-code suppressions"
        );
    }
    assert_contains_all(
        "output target retained WGPU owner contract",
        &output_target,
        &[
            "pub(in crate::graphics::scene) const RETAINED_OUTPUT_TARGET_TEXTURE_OWNER_COUNT: usize = 4;",
            "pub(in crate::graphics::scene) fn retained_output_target_texture_owner_count(&self) -> usize",
            "let _retained_output_target_texture_owners =",
            "&self.descriptor",
            "&self.texture",
            "&self.view",
            "&self.sampler",
            "OutputTargetTextureResource must retain descriptor, texture, view, and sampler while exposing output target writeback and graph-import bindings",
        ],
    );
    assert_contains_all(
        "prepared output target cache owner contract",
        &prepared_output_target,
        &[
            "pub(in crate::graphics::scene::resources) const RETAINED_OUTPUT_TARGET_CACHE_OWNER_COUNT:",
            "usize = 1;",
            "pub(in crate::graphics::scene::resources) fn retained_output_target_cache_owner_count(",
            ") -> usize",
            "let _retained_output_target_cache_owner = &self.resource;",
            "pub(in crate::graphics::scene::resources) fn resource(",
            ") -> &Arc<OutputTargetTextureResource>",
            "PreparedOutputTargetTexture must retain the output target resource while streamer exposes writeback and graph-import access",
        ],
    );
    assert_contains_all(
        "resource streamer accesses prepared output target through owner accessor",
        &resource_streamer_accessors,
        &[
            "pub(in crate::graphics::scene) fn output_target_texture_resource(",
            ".map(|prepared| Arc::clone(prepared.resource()))",
        ],
    );
    assert_contains_all(
        "output target readiness uses prepared owner accessor",
        &resource_streamer_ensure,
        &[
            ".and_then(|texture| self.output_target_textures.get(&texture.id()))",
            ".map(|prepared| prepared.resource().descriptor().format.as_str())",
        ],
    );
    assert_contains_all(
        "output target writeback clones through prepared owner accessor",
        &resource_streamer_writeback,
        &[
            ".map(|prepared| Arc::clone(prepared.resource()))",
            "prepared_resource.texture().as_image_copy()",
            "prepared_resource.view()",
        ],
    );
    assert_contains_all(
        "compiled-scene graph import consumes output target owner accessors",
        &frame_graph_binder,
        &["view: resource.view()"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 output target texture owner cleanup",
                "runtime_15_output_target_texture_owner_cleanup_coremin_check_passed",
                "runtime_15_output_target_texture_owner_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_material_runtime_capture_seed_cleanup() {
    let material_runtime = read_runtime_src("graphics/scene/resources/runtime/material_runtime.rs");
    let runtime_mod = read_runtime_src("graphics/scene/resources/runtime/mod.rs");
    let resources_mod = read_runtime_src("graphics/scene/resources/mod.rs");
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !material_runtime.contains("#[allow(dead_code)]"),
        "MaterialRuntime and MaterialCaptureSeed should not hide production dead-code surfaces behind suppressions"
    );
    assert_contains_all(
        "material runtime capture seed is test-only",
        &material_runtime,
        &[
            "#[cfg(test)]",
            "pub(crate) struct MaterialCaptureSeed",
            "impl MaterialRuntime",
            "pub(crate) fn capture_seed(&self) -> MaterialCaptureSeed",
        ],
    );
    assert_contains_all(
        "material capture seed re-export stays behind test cfg",
        &runtime_mod,
        &[
            "pub(crate) use material_runtime::MaterialRuntime;",
            "#[cfg(test)]",
            "pub(crate) use material_runtime::MaterialCaptureSeed;",
        ],
    );
    assert_contains_all(
        "resources facade keeps production material runtime separate from test capture seed",
        &resources_mod,
        &[
            "pub(crate) use runtime::MaterialRuntime;",
            "#[cfg(test)]",
            "pub(crate) use runtime::MaterialCaptureSeed;",
        ],
    );
    assert_contains_all(
        "resource streamer capture seed accessor is test-only",
        &resource_streamer_accessors,
        &[
            "use super::super::MaterialCaptureSeed;",
            "pub(crate) fn material_capture_seed(",
            "pub(crate) fn sample_texture_rgba(",
            "fn sample_texture_asset_rgba(",
            "fn wrap01(",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 material runtime capture seed cleanup",
                "runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed",
                "runtime_15_material_runtime_capture_seed_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_resource_streamer_diagnostics_accessor_cleanup() {
    let resource_streamer_accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );
    let resource_streamer_ensure = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !resource_streamer_accessors.contains("#[allow(dead_code)]"),
        "ResourceStreamer diagnostics accessors should be test-only or production-live, not dead-code suppressions"
    );
    assert_contains_all(
        "test-only asset and material diagnostics accessors",
        &resource_streamer_accessors,
        &[
            "#[cfg(test)]",
            "pub(crate) fn model_asset_overview(",
            "pub(crate) fn asset_management_record_sets(",
            "pub(crate) fn material_uniform_payload_byte_len(",
            "pub(crate) fn material_management_record_set(",
            "pub(crate) fn material_prepared_state(",
        ],
    );
    assert_contains_all(
        "production material readiness accessor remains live",
        &resource_streamer_accessors,
        &[
            "pub(crate) fn material_readiness_report(",
            "pub(crate) fn material_readiness_summary(",
            "self.material_readiness_report(id)",
        ],
    );
    assert_contains_all(
        "scene resource ensure path consumes production readiness summary",
        &resource_streamer_ensure,
        &[
            "if let Some(summary) = self.material_readiness_summary(&material_id)",
            "summary.is_ready",
            "summary.uses_fallback",
            "summary.validation_error_count",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 resource streamer diagnostics accessor cleanup",
                "runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked",
                "runtime_15_resource_streamer_diagnostics_accessor_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_resource_streamer_resolve_texture_id_cleanup() {
    let resolve_texture = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert!(
        !resolve_texture.contains("#[allow(dead_code)]"),
        "ResourceStreamer texture-reference resolution should not hide unused helpers behind dead-code suppression"
    );
    assert!(
        !resolve_texture.contains("fn resolve_texture_id("),
        "the unused ResourceStreamer::resolve_texture_id helper should stay retired"
    );
    assert_contains_all(
        "production texture-reference resolution entry points remain live",
        &resolve_texture,
        &[
            "pub(in crate::graphics::scene::resources) fn resolve_texture_reference(",
            "pub(in crate::graphics::scene::resources) fn resolve_texture_reference_with_support(",
            "pub(in crate::graphics::scene::resources) fn id(&self) -> Option<ResourceId>",
            "RenderMaterialValidationError::TextureNotUploadReady",
            "RenderMaterialTextureSlotFallback::not_upload_ready",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 resource streamer resolve texture id cleanup",
                "runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked",
                "runtime_15_resource_streamer_resolve_texture_id_cleanup",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    let path = runtime_src_path(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("runtime source should exist at {}: {error}", path.display())
    })
}

fn read_repo(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("repo source should exist at {}: {error}", path.display()))
}
