use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src};

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
