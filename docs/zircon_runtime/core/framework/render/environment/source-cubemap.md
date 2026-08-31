---
related_code:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem_layout.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/rgba16f.rs
  - zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/half_float.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/fixture_assets.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/hdri_metrics.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix_quantitative.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/sphere_reflection.rs
  - zircon_runtime/tests/runtime_environment_ibl_artifact_source_identity_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/cmft/src/cmft/cubemaputils.h
  - dev/cmftStudio/src/backgroundjobs.cpp
  - dev/cmftStudio/src/shaders/fs_mesh.shdr
  - dev/cmftStudio/src/shaders/fs_skybox.sc
  - dev/cmftStudio/src/shaders/vs_skybox.sc
  - dev/cmftStudio/src/shaders/utils.shdr
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShaders.usf
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShared.ush
  - dev/UnrealEngine/Engine/Source/Developer/TextureCompressor/Private/TextureCompressorModule.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Math/UnrealMath.cpp
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem_layout.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/rgba16f.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/half_float.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/fixture_assets.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/hdri_metrics.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix_quantitative.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/sphere_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs
  - zircon_runtime/tests/runtime_environment_brdf_lut_contract.rs
plan_sources:
  - user: 2026-07-05 real HDRI cubemap skybox/reflection mosaic correction and cmft mip filtering request
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - tools/tests/test_runtime_job_system_audit.py
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/tests/runtime_environment_source_cubemap_contract.rs
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - zircon_runtime/tests/runtime_environment_source_irradiance_cubemap_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_artifact_source_identity_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_source_cubemap_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_seam_contract.rs
  - zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/fixture_assets.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/hdri_metrics.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix_quantitative.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/sphere_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs
  - cargo test -p zircon_runtime --lib ibl_bake_request --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-request-owner-coremin-0706 --color never -- --nocapture --test-threads=1
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
doc_type: module-detail
---

# Source Cubemap Environment

## Purpose

`source_cubemap.rs` is the real HDRI source-cubemap path for skybox and standard PBR reflection validation. It supersedes the retired sampled-equirect table path with a six-face source cubemap plus a separate specular PMREM mip chain, then uploads both as `texture_cube<f32>` bindings for skybox and environment sampling.

The EL-M2 capture extension accepts six face-major HDR mip-zero images through `build_source_cubemap_from_captured_faces_with_quality(...)`. It builds the same angular source mip pyramid, GGX filtered-importance-sampling PMREM and SH9 as imported equirectangular sources, so reflection probes and skybox imports cannot drift into separate filtering implementations. `source_cubemap_capture_hash(...)` hashes the exact HDR capture bits and face size for stable staged-artifact identity. Fast/Normal/High select 32/64/128 GGX samples, with the low-roughness tier halved. The filtered GGX/cosine branches select source LOD only from the Unreal sample PDF and clamp it to the source mip range; destination texel footprint is reserved for mip-zero direct downsampling and ordinary source-mip construction, so it cannot force glossy PMREM levels to blur early.

The current chain includes an EC-M2a CPU GGX filtered-importance-sampling PMREM bridge for higher mips, an EC-M2b CPU SH9 diffuse irradiance bridge, the EC-M2c runtime upload bridge that keeps source and specular PMREM textures separate in RGBA16F while standard PBR uses an RG16F environment BRDF LUT, an EC-M2d CPU IEM reference bridge in `source_irradiance_cubemap.rs`, an EC-M2i artifact application bridge in `source_cubemap_artifact.rs` that preserves source cubemap display mips while replacing PMREM/SH9 from a decoded reusable artifact payload, an EC-M2j runtime IEM carrier/binding slice that attaches optional 32x32x6 irradiance cubes to `SourceCubemapEnvironment` and uploads them as a stable scene bind-group slot, and an EC-M2l upload-key slice that includes artifact payload identity in WGPU cubemap upload invalidation without changing the source-only IBL bake key. EC-M2u moves the PMREM generator into `source_cubemap/pmrem.rs` and changes the full-roughness tail to Unreal-style cosine hemisphere convolution from the source mip pyramid; it explicitly supersedes the earlier EC-M2r previous-PMREM-downsample experiment because ordinary downsampling can preserve blocky low-mip structure instead of integrating the real environment lobe. EC-M2ai makes the source cubemap environment the explicit IBL bake request shape owner: `SourceCubemapEnvironment::ibl_bake_artifact_request(...)` and `EnvironmentExtract::source_cubemap_ibl_bake_request(...)` provide bake key, face size, and mip count, while renderer graph resources only select which artifact contents are requested. EC-M3a adds a whole-matrix quantitative guard for the real-HDRI screenshot export, EC-M3b locks progressive high-frequency luma-variance reduction plus final-mip face averaging, EC-M3c quantizes cube-edge seam energy on rough PMREM mips, EC-M3d fixes CPU mip sampling across cube-face edges while validating the Poly Haven lakes 2K HDRI export, and EC-M3e adds a non-ignored saved-PNG regression that replays the Plan 06 matrix metrics against the accepted 2K screenshot. EC-M3h/EC-M3i split source mip generation into `source_cubemap/mipmap.rs` and replace the old single-sample previous-mip lookup with a UE-inspired angular source mip pyramid: an average input chain plus per-output-texel angular-footprint sub-sampling weighted by cubemap solid angle, with a final 1x1 six-face average. EC-M3ab adds the `.zcube` source container helper in `asset/assets/texture/zcube.rs`; it serializes only `source_texels()` through the shared RGBA16F layout and explicitly stays source-only rather than becoming a PMREM artifact. This closes the user-visible 1K/source-table cubemap mosaic issue for the current CPU bridge, gives roughness-driven PBR reflections a real blurred environment source, preserves HDR values above 1.0 on the runtime texture path, and removes the previous low-mip diffuse approximation from source-cubemap lighting. EC-M3j upgrades the manual PBR visual gate from an 8x8 matrix to a 10x10 metallic/smoothness matrix and adds single-sphere plus texture-map material exports using real ambientCG Metal009 Color/NormalGL/Roughness/Metalness inputs. EC-M3k fixes the reflection-view vector used by standard PBR, fallback mesh, and deferred lighting: `SceneUniform` now carries camera world position plus a projection-mode view-direction flag, so mirror-like reflections use fragment-to-camera for perspective cameras and a fixed camera forward vector for orthographic cameras instead of the old constant `+Z` direction. EC-M3k also adds perfect-mirror orthographic/perspective validation plus ambientCG Metal008, Metal025, and Metal029 texture-map exports. It is still not the final production IBL stack: GPU/offline compute baking, importer/staged derived artifact production, runtime readback scheduling, GPU/offline IEM bake production, engine quality selection for `ZR_ENV_DIFFUSE_IEM`, GPU/offline artifact seam validation, strict SSIM against source-cubemap references, RenderDoc/product capture, and higher-resolution offline bake acceptance remain pending.

EC-M3l/EC-M3m update the mirror validation path on top of that chain. EC-M3l lowers the standard material roughness floor so authored mirrors can sample PMREM mip0, while EC-M3m fixes the validation sphere winding plus skybox `-Z` screen direction so the accepted mirror screenshots no longer show vertically or front/back inverted environment content.

EC-M3n superseded the mirror-image hashes after re-aligning face orientation, the then-current linear roughness-to-LOD consumption and `fixCubeLookup` with cmft/cmftStudio. The 2026-07-11 EC-M3 quantitative audit then replaced the remaining cosine-power CPU filter with the planned UE-style GGX FIS implementation. The 2026-07-13 correction closes two audit defects in that first port: for `V=N`, the light-direction PDF is `D / 4` because `NoH` cancels `VoH`, and filtered source LOD no longer applies a destination-footprint floor. CPU and WGSL now share Hammersley GGX/cosine sampling, PDF-selected source mip, and UE's `CubemapMaxMip - 1 - LevelFrom1x1` roughness mapping. The subsequent artifact-identity audit increments `IBL_BAKE_ALGORITHM_VERSION` to `2026_07_13_0002`, so older `.zribl` products are rejected and rebuilt. The full Unreal angular source-mip cutover increments it again to `2026_07_13_0003`, because changing the FIS source pyramid changes derived PMREM bytes even when the HDR source key is unchanged.

EC-M3ar separates display-source and reflection-filter storage all the way through the runtime. `SourceCubemapMipChain` now owns `source_face_size/source_mip_count/source_texels` independently from fixed `pmrem_face_size = 128`, `pmrem_mip_count = 8`, and `pmrem_texels`; the WGPU renderer uploads two textures with their own layouts and scene-uniform metadata. `.zcube` remains a self-describing full-resolution source product, while `.zribl` descriptors and payloads describe the fixed PMREM product. CPU and WGSL roughness conversion now share the Shader 06 exponential Unreal mapping, and filtered-importance source LOD uses the actual source texture dimensions rather than the fixed PMREM dimensions. Environment importer source hashes include the selected source face/mip layout, and staged `.zcube` reads reject request/layout mismatches, so changing import resolution cannot silently reuse another bake.

`SourceCubemapPmremLayout` owns the reflection-result dimensions separately from the source layout. Its default remains the product artifact contract (`128x128`, eight mips), while `SourceCubemapMipChain::with_pmrem_face_size(...)` rebuilds a full mip chain at a requested validation resolution from the retained source pyramid. The interactive viewer uses that path after staging, defaults the active PMREM face size to the resolved source face size, and clears the artifact hash because the rebuilt result is not the staged `.zribl`. This does not change the reusable artifact descriptor or claim that product caches are 512-wide; it gives visual and quantitative validation an explicit way to compare PMREM result resolution without conflating it with source resolution.

EC-M3aj closes the staged consumption proof. The asset staging store reconstructs `SourceCubemapEnvironment` from the source `.zcube` plus derived `.zribl` without rebuilding the staged artifact; the viewer may then replace only its active validation PMREM through `with_pmrem_face_size(...)`. The DX12 interactive viewer consumes that source/artifact pair and produces default plus yaw/pitch +/-120-degree screenshots from the same process; all views retain matched skybox/reflection orientation and filtered reflection detail.

EC-M3o supersedes the EC-M3n mirror-image hashes for the standard-material validation path only. The cubemap orientation and cmft PMREM path remain unchanged; the new root cause was that generated StandardPBR material WGSL sampled the renderer-owned neutral normal fallback even when the material had no authored normal texture. The standard-material template now receives a `ZR_FEATURE_HAS_NORMAL_TEXTURE` define, returns the geometric normal when the bit is false, and only samples `standard_material_normal_tex` for authored normal-map materials. The mirror export assertion now also checks left/right grazing balance so the previous one-sided edge highlight cannot silently return.

EC-M2p adds the backend section acquisition helper for runtime cache writeback: `IblBakeArtifactWgpuReadbackResources` declares which PMREM texture, SH9 buffer, and optional IEM texture are required by the artifact descriptor, then `read_ibl_bake_artifact_wgpu_sections(...)` produces `IblBakeArtifactReadbackSections` from already-built WGPU resources. This closes the synchronous resource-to-section helper only; compute production, render-graph scheduling, and writeback dispatch integration remain separate pending slices.

EC-M3g adds a manual 1K source-vs-PMREM mip diagnostic export for investigating visible blockiness. EC-M2u refreshes that diagnostic after the cosine full-roughness change. The diagnostic paints source mips and PMREM mips separately, so enlarged raw low mip tiles are inspected as bake data while the runtime skybox still samples source cube mip 0 and PBR reflections still use trilinear roughness sampling from the specular PMREM cube. After EC-M3h, source diagnostic mips are expected to show angular-footprint blur instead of nearest-neighbor block structure.

EC-M2x aligns the renderer-local PMREM WGSL kernel plan with the CPU bridge for the algorithm pieces that live inside a single per-mip shader: centered Hammersley samples, Unreal `E.y *= 0.995`, filtered-importance source-lod selection with the `* 2.0` texel solid-angle scale, 128 samples for high roughness, and the `roughness >= 0.99` cosine hemisphere convolution branch. This is still a shader-plan parity slice, not the final GPU bake: WGPU command encoding, async scheduling/readback, cache writeback from GPU outputs, and the final 1x1 six-face GPU average pass remain open.

EC-M3w adds an artifact seam roundtrip guard. `runtime_environment_ibl_bake_artifact_seam_contract.rs` builds the same synthetic high-frequency seam-stress environment as the source-cubemap seam test, encodes the PMREM/SH9 payload as a `IblBakeArtifactBlob`, decodes it through `decode_current_for_request(...)`, applies it through `source_cubemap_mip_chain_with_bake_artifact(...)`, and compares cube-edge luma statistics before and after the RGBA16F artifact roundtrip. This closes the serialization/decode/application seam-preservation gap for CPU/offline-style artifact payloads; GPU-produced texture readback and importer/staged-build artifact parity still need their own product-level comparison.

EC-M3x extends that seam guard down to the backend WGPU texture readback boundary. `readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip` writes a seam-stress PMREM artifact payload into an actual `Rgba16Float` cube texture using the same face-major/mip-major layout as `SourceCubemapMipChain`, reads it back through `read_ibl_bake_artifact_wgpu_sections(...)`, requires exact PMREM/SH9 payload byte equality, then applies the readback payload and repeats the mid/rough cube-edge seam checks. This closes the backend texture-copy/readback layout risk that could have transposed faces, mips, or rows before the artifact payload reached source-cubemap application.

EC-M3y extends the same seam guard to live compute-produced PMREM graph output. `runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams` builds a synthetic seam-stress source cubemap, records the actual PMREM WGPU graph passes, reads the produced graph texture through runtime cache writeback, resolves the second dispatch from `.zircon/cache` with zero runtime compute, decodes the PMREM payload into a `SourceCubemapMipChain`, and compares base/mid/rough cube-edge luma statistics. This closes the live PMREM graph-output readback seam gap.

EC-M3z adds the matching live compute-produced IEM graph-output guard. `runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance` builds a low-frequency directional source cubemap, records the actual WGPU irradiance-cube graph pass, reads the produced 32x32x6 `Rgba16Float` IEM texture through runtime cache writeback, resolves the second dispatch from `.zircon/cache` with zero runtime compute, decodes the payload into `SourceCubemapIrradianceCube`, and compares normalized luma response against the CPU cosine-convolution reference. This closes the live IEM graph-output readback parity gap; importer/staged-build artifact parity, product scheduler proof, and product-level comparison remain pending.

## Related Files

The CPU source/SH9 chain is implemented in [source_cubemap.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/source_cubemap.rs). Source mip generation is owned by [mipmap.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs), while PMREM prefiltering is owned by [pmrem.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs). Projection math comes from [cubemap_projection.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs). Runtime upload and bind-group ownership live in [environment_cubemap.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs), while the shared 6-binding scene layout entries are defined in [scene_bind_group_layout.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs).

The skybox pass samples mip 0 from the source cube in [skybox_procedural.wgsl](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl). Standard PBR indirect reflection samples the specular PMREM cube by roughness-derived mip in [zr_environment.wgsl](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl), while source-cubemap diffuse ambient evaluates SH9 coefficients uploaded through [scene_uniform.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs). Camera position and projection-mode view direction are declared for WGSL in [zr_scene_runtime.wgsl](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl) and consumed by [zr_shading_standard_pbr.wgsl](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl), [deferred_lighting.wgsl](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl), and [fallback_mesh.wgsl](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl) so environment reflection vectors are camera-correct. Split-sum specular energy comes from the CPU-built LUT in [environment_brdf_lut.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs) uploaded by [environment_brdf_lut.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs). The manual HDRI screenshot export is [runtime_shader_pbr_hdri_export.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs), and the shared saved-PNG/matrix assertions live in [hdri_metrics.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/runtime_shader_pbr_hdri_export/hdri_metrics.rs).

Runtime cache writeback is split by boundary. [ibl_bake_artifact_readback.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs) validates descriptor-shaped PMREM/SH9/IEM byte sections, [ibl_bake_artifact_runtime_writeback.rs](/E:/Git/ZirconEngine/zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs) writes current sections into the runtime cache, and [read_ibl_bake_artifact_sections.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs) is the WGPU acquisition helper that reads already-produced PMREM textures, SH9 buffers, and optional IEM textures through the lower-level RGBA16F texture and SH9 buffer readback helpers.

`SourceCubemapEnvironment` also owns the source-shaped IBL bake request. `ibl_bake_artifact_request(...)` packages the source-only `IblBakeKey`, the source mip-chain face size, and the full mip count with caller-selected artifact contents. `EnvironmentExtract::source_cubemap_ibl_bake_request(...)` exposes the same request to renderer executors. This keeps SH9-only and IEM-only bake graphs from depending on a PMREM output texture just to recover source dimensions.

The artifact format-v2 descriptor/header persists both source face/mip layout and derived PMREM face/mip layout. `is_current_for(...)`, runtime-cache identity, asset-derived paths, WGPU command planning, and runtime writeback all derive from the same request and reject a matching bake key paired with a different source layout. `IblBakeArtifactPayload::from_source_cubemap(...)` always validates source layout; PMREM layout is validated only when PMREM bytes are present, so SH9-only payloads remain independent of the PMREM texture while still retaining correct source identity. Format v1 headers are rejected rather than upgraded through a compatibility path.

## Behavior Model

`SourceCubemapMipChain` stores texels face-major, with all mips for one face laid out before the next face. The face order is `+X, -X, +Y, -Y, +Z, -Z`, matching cmft and the cube texture resource contract. `source_texels()` is the display/source mip chain. `texels()` is the specular PMREM chain. Mip 0 is identical between both chains; higher source mips are angular-footprint filtered from the source environment, while higher specular mips are GGX filtered.

`build_source_cubemap_from_source_mips(...)` is the external-container entry point. It validates and preserves a complete face-major source pyramid, projects SH9 from source radiance, and regenerates PMREM plus the final six-face average rather than interpreting authored DDS/KTX mips as specular prefiltering. The asset decoder handles cmft DDS face-major bytes and reorders KTX1/KTX2 mip-major bytes before calling this builder.

The input face size is derived from the equirectangular height:

```text
face_size = clamp(next_power_of_two((height + 1) / 2), 64, 1024)
```

Mip count is the full power-of-two chain down to 1x1. Mip 0 is generated by sampling the equirectangular source through the shared cubemap projection helpers, so skybox rendering uses the full source cube rather than a small storage-buffer table.

The builder also creates a source mip chain from mip 0 through `source_cubemap/mipmap.rs`. The source mip owner first builds an interval-covered average chain for input acceleration; power-of-two faces reduce as 2x2 blocks, while non-power-of-two faces include every edge texel instead of dropping the final row or column. For every higher output mip it then follows the structural contract of Unreal `GenerateAngularFilteredMips`: `cone_angle = clamp((pi/2) / output_face_size, 0.002, pi/2)`, converts the normalized spherical-cap area into a quality-biased input mip (`QualityBias = 3`), and integrates source texels from all six faces inside that cone. A conservative face-region hierarchy uses Unreal's sphere/cone intersection to reject irrelevant regions without changing the accepted texel set. Input texel solid angles are cached once per distinct selected input mip for the complete source-chain build. Source inputs at 128 texels or larger retain Unreal's one-worker-per-face policy only when the caller uses an explicit parallel-executor builder; the supplied runtime pool performs that work through `ParallelSliceExecutor`. Synchronous builders intentionally use the serial face executor because they were not given a runtime execution owner. This replaces the earlier direct-Rayon bypass and the earlier fixed 4x4 previous-mip footprint bridge; source mips are now an angular cubemap minification/FIS structure, not the specular PMREM result. Mip zero remains the exact source image, and the final source 1x1 mip averages all six faces following cmft's final-mip discipline.

### Execution Ownership

`SourceCubemapMipChain::from_equirect_with_parallel_executor(...)`, `from_captured_faces_with_parallel_executor(...)`, and the quality-selecting captured-face variant accept the neutral `ParallelSliceExecutor` contract. `TaskPool` implements that contract in `core::runtime::tasks`, keeping Rayon and thread-budget ownership below the framework boundary. The original synchronous constructors remain deterministic serial entry points; they do not allocate a private pool and do not use the process-global Rayon pool.

Zircon intentionally diverges from the current Unreal TextureCompressor at the final sample weight. Unreal computes an area-compensation table but disables it, accumulates the filter weight in Alpha, normalizes RGB, and writes Alpha as zero. Plan 06 and the cmft projection discipline require exact cubemap solid-angle weighting, so Zircon multiplies the same Unreal smoothstep kernel by `cubemap_texel_solid_angle(...)` and normalizes all four source channels. Preserving filtered source Alpha is required by the RGBA16F `.zcube` and upload contracts; this is a documented engine contract rather than an accidental claim of byte-for-byte Unreal output.

The same regular source chain provides the SH9 projection source. `source_cubemap_irradiance_mip_level(...)` chooses the mip closest to a 32x32 face, matching the plan 06/UE diffuse-irradiance scale. Projection uses the shared exact cubemap texel solid angle, normalizes the accumulated weight back to `4*pi`, and stores cosine-lobe-premultiplied L2 coefficients in `SourceCubemapMipChain::irradiance_sh9()`.

The EC-M2d IEM bridge is intentionally split into [source_irradiance_cubemap.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs). It builds a 32x32x6 face-major diffuse irradiance cube by direct cosine convolution over the same selected source mip and exact solid-angle weights, then exposes cross-face bilinear sampling for future GPU/offline IEM comparisons. The runtime upload path now accepts this optional cube through `SourceCubemapEnvironment::with_irradiance_cube(...)` and binding 5. `zr_environment.wgsl` exposes `ZR_ENV_DIFFUSE_IEM` and `zr_environment_irradiance_cube_color(...)`; the default remains SH9 until the engine quality/specialization path owns that global switch.

## Artifact Application

`source_cubemap_artifact.rs` owns the bridge from decoded reusable bake payloads back into `SourceCubemapMipChain` and `SourceCubemapEnvironment`. `source_cubemap_mip_chain_with_bake_artifact(...)` accepts an existing source cubemap chain and an `IblBakeArtifactPayload`, verifies matching face size and mip count, requires PMREM plus SH9 content, preserves `source_texels()` for skybox/source sampling, and replaces only the specular PMREM texels plus SH9 coefficients. `source_cubemap_environment_with_bake_artifact(...)` applies the same PMREM/SH9 payload to an environment object and attaches an optional decoded IEM cube when the artifact contents include `IEM`.

This is the CPU contract that lets asset-derived artifacts or `.zircon/cache` runtime cache blobs feed the current upload path without losing the full-resolution source cubemap used by the skybox. Applying an artifact leaves `ibl_bake_key()` unchanged, because the bake key describes the source input, but updates the `SourceCubemapUploadKey` artifact hash so the WGPU upload cache refreshes source/specular/IEM textures when the reusable artifact content changes. It deliberately does not create files, schedule GPU readback, or rebuild source mips.

Mip 1 and below are generated as a UE-style GGX PMREM bridge:

- roughness and mip level use `SOURCE_CUBEMAP_ROUGHEST_MIP = 1.0` and `SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE = 1.2`,
- `alpha_squared` is `roughness^4`, matching the shader-side roughness convention,
- Hammersley samples are distributed through the GGX importance sampling equation,
- `E.y` is scaled by `0.995` to avoid singular edge cases,
- sample pdf uses `D_GGX(alpha_squared, NoH) * 0.25`,
- input source mip uses `0.5 * log2(solid_angle_sample / solid_angle_texel)`,
- samples are trilinearly fetched from the source mip chain and weighted by `NoL`,
- low roughness uses 32 samples, mid roughness uses 64 samples, and high roughness uses 128 samples,
- `roughness >= 0.99` uses the Unreal `FilterPS` full-roughness branch: cosine hemisphere samples, `pdf = NoL / pi`, filtered-importance source-mip selection, and source-pyramid trilinear fetches rather than previous-PMREM box downsampling,
- the final 1x1 mip averages all six faces to remove low-mip directional noise.

This deliberately avoids using the source mip chain directly as the reflection mip chain. A plain or angular source mip pyramid is still an environment texture-minification structure; specular reflection roughness needs the GGX/cosine radiance convolution above. Reusing source mip downsampling as PMREM was the visible failure mode in the earlier HDRI screenshot.

## GPU Data Flow

`SourceCubemapEnvironment` is carried by `EnvironmentExtract`. `SkyboxSettings::source_cubemap(...)` guarantees a current immutable upload artifact before render submission. The artifact is mip-major, face-packed and WGPU-row-aligned for source, PMREM and irradiance; environments without an IEM carry a pre-encoded 1x1 black irradiance cube. During scene-uniform preparation, the renderer detects upload-key/layout changes, validates the complete artifact before replacing textures, packs one shared staging payload, appends one staging-buffer range to the frame's existing `FrameBufferUpload`, and records one buffer-to-texture copy per changed mip in the scene encoder. Stable keys produce no payload or copy. The upload key is committed only after scene submission succeeds, so graph/admission/submit failure retries without publishing stale residency.

All runtime environment cube textures are `Rgba16Float`. The screenshot/test helper now exposes the HDR source into linear float space without Reinhard tone mapping before cubemap construction, then the runtime upload encodes each channel as FP16. The CPU bridge still builds the PMREM and optional IEM chains in memory; `ibl_bake_artifact.rs` owns the reusable artifact descriptor/payload contract, `ibl_bake_artifact_blob.rs` owns the raw byte container, `asset/artifact/ibl_bake_artifact_cache.rs` owns `.zribl` runtime cache placement, and `source_cubemap_artifact.rs` owns applying decoded PMREM/SH9/IEM payloads to a source cubemap environment. `SceneEnvironmentCubemap::ensure_uploaded(...)` compares `SourceCubemapUploadKey` rather than only source revision/hash, so source-identical artifact updates cannot leave stale WGPU PMREM/IEM textures. GPU/offline compute output, importer/staged derived production, and runtime readback scheduling remain later EC-M2 work.

`RenderSceneExtract::fallback_skybox_kind()` treats `SkyboxMode::SourceCubemap` as `FallbackSkyboxKind::None`. The source cubemap is an authored environment input with its own cube texture binding, so it must not re-enter the procedural fallback path while the environment bind group is already carrying the cube.

The scene bind group now contains:

- binding 0: scene uniform buffer,
- binding 1: source environment cube texture view for skybox and source lookup,
- binding 2: filtering sampler,
- binding 3: RG16F environment BRDF LUT,
- binding 4: specular PMREM cube texture view for roughness-driven PBR reflections,
- binding 5: optional diffuse irradiance cube texture view for the `ZR_ENV_DIFFUSE_IEM` path, with a 1x1 black fallback when no IEM is attached.

`scene_bind_group_layout_entries()` is visible within the `scene_renderer` subtree so renderer construction, shader prewarm validation, and focused pipeline tests all consume this single 6-binding layout owner instead of copying layout definitions.

The sampler enables linear minification and trilinear mip sampling. The skybox shader always samples mip 0 from the source cube for source-cubemap mode. `zr_environment.wgsl` uses the UE roughness-to-mip constants already documented in plan 06, then samples binding 4 at that lod for specular reflections and binding 3 for split-sum specular scale/bias.

Diffuse environment lighting for source cubemaps is no longer sampled from a rough cube mip. `SceneUniform.environment_sh9` carries nine `vec4<f32>` coefficients, and `zr_environment_sh9_eval(...)` evaluates the y-up SH basis used by the CPU bridge before applying environment intensity. When `ZR_ENV_DIFFUSE_IEM` is specialized on, `zr_environment_irradiance_cube_color(...)` samples binding 5 instead. Procedural environments still use the procedural sky color path until the later ambient-mode SH convergence work lands.

`SceneUniform` also carries `camera_world_position.xyz` and `camera_view_direction`. The `w` component of `camera_view_direction` is a projection-mode selector: `0.0` means perspective and the shader derives the view vector from fragment world position to camera world position; `1.0` means orthographic and the shader uses the normalized camera forward vector directly. Standard PBR, fallback mesh, and deferred lighting all go through this helper behavior before calling `zr_environment_pbr_indirect(...)`. This prevents mirror-like source-cubemap reflections from appearing vertically flipped or camera-invariant when the camera projection changes.

## Design And Rationale

The user-facing defect was not only a filtering issue: the older data path had no real cubemap and no high-resolution texture source for the skybox. This module fixes that first by switching visible sky and material reflections to native cube texture sampling.

The module keeps cmft/cmftStudio's face order, lat-long projection, source cubemap layout, cross-face neighbor sampling discipline, final-mip face averaging, and sampling-side `fixCubeLookup`. PMREM generation follows the Shader 06 UE FIS contract on CPU and GPU, while source and PMREM textures now have independent physical layouts. The strict eight-step sphere-frequency product gate remains open until the dual-layout implementation is compiled and rerun against the real HDRI frame captures.

The result is intentionally staged. It provides a source-cubemap/PMREM screenshot path, SH9 diffuse CPU reference, RGBA16F runtime upload, and BRDF LUT shader bridge. The mirror source-reference SSIM gate currently measures `0.999848`, but the all-eight-level Laplacian gate is not accepted; no new quantitative screenshot is promoted while that assertion fails.

## 2026-07-06 Reference Design Audit

The 2026-07-04 artifact [runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png) is now explicitly treated as a rejected baseline. It was produced by the retired sampled-equirect path and therefore proves the old 16x8 table failure; it must not be cited as cubemap or PMREM acceptance evidence.

The accepted design separates display and reflection data. Skybox rendering samples source cubemap mip 0, while standard PBR reflections sample the separate specular PMREM cube through the Unreal roughness-to-mip mapping. For the Poly Haven lakes assets, the current face-size rule means the 1K HDRI produces 256x256 faces and the 2K HDRI produces 512x512 faces. Any future documentation or tests that say "1K produces 512 face" are wrong unless the source file itself is 2048x1024.

The PMREM mip chain is physically separate from the source mip pyramid. cmft/cmftStudio remain authoritative for face order, equirect projection, cross-face edge discipline, final 1x1 face averaging and the PMREM/IEM split. UE `TextureCompressorModule.cpp` remains the source-mip reference, while UE reflection-environment GGX FIS and exponential roughness mapping drive both the CPU bridge and GPU prefilter/consumer shaders. Source faces retain imported resolution; the reusable PMREM product is fixed at 128x128x6 with eight mips.

The reference audit now has a production follow-up: source mips and PMREM filtering have both moved into child owners, `source_cubemap/mipmap.rs` and `source_cubemap/pmrem.rs`. The parent source-cubemap owner remains focused on layout, SH9, and public data contracts. Future IEM, GPU/offline bake, derived-cache, or stricter screenshot metric work should continue to use child owners rather than growing `source_cubemap.rs` again.

## Edge Cases And Constraints

Small HDRI sources clamp to 64x64 faces so manual screenshots cannot silently fall back to tiny cubemaps. Large sources clamp to 1024x1024 faces until the asset pipeline owns offline cubemap baking.

Sampling across face edges in CPU mip generation now projects out-of-face taps back through cubemap directions and resolves them on the neighboring face. Source angular mip generation also samples sub-texel directions that can leave the current face, so source mips blur across cube boundaries instead of clamping. GPU consumption still uses native cube textures. The current tests cover source PMREM seam-energy reduction and one direct cross-face bilinear tap; production GPU/offline artifacts still need the same seam comparison once derived baking exists.

The staged sampled-equirect compatibility path has been removed from the runtime render contract. `SkyboxSettings` now exposes only disabled, procedural-gradient, and source-cubemap modes, and `EnvironmentExtract` routes real HDRI validation through `SourceCubemapEnvironment`.

## Test Coverage

The focused CPU tests in `source_cubemap.rs` and `runtime_environment_source_cubemap_contract.rs` cover:

- equirectangular height to face-size clamping,
- face-major mip layout offsets,
- constant-environment preservation through every mip,
- Unreal cone-angle and quality-biased input-mip selection for the 512-face source chain,
- exact small-cubemap agreement with an independent six-face Unreal cone-filter reference,
- non-power-of-two average-input coverage of the final source row and column,
- high-output-mip agreement with an independent reference reading the selected higher-resolution average mip,
- cubemap face direction to equirectangular UV sampling,
- cross-face bilinear sampling for CPU source/PMREM input taps,
- roughness to PMREM mip mapping,
- high-frequency source blur in the GGX PMREM mip chain,
- preservation of HDR values above 1.0 for the float upload path,
- separation between angular source mips and GGX PMREM mips,
- preservation of externally supplied source mip identity while regenerating distinct PMREM data,
- source mip high-frequency variance reduction while staying sharper than same-level PMREM,
- source final 1x1 six-face averaging,
- luma-variance reduction in rough mips,
- progressive high-frequency luma-variance reduction across rougher PMREM mips,
- roughness=1.0 selected PMREM mip cosine convolution that differs from ordinary previous-mip downsampling while reducing high-frequency variance,
- final 1x1 mip averaging across all faces,
- cube-edge seam-energy reduction on rough PMREM mips,
- SH9 irradiance mip selection,
- constant diffuse environment preservation through SH9,
- vertical-gradient diffuse response through SH9,
- constant diffuse environment preservation through the CPU IEM bridge,
- low-frequency CPU IEM samples matching the SH9 bridge,
- BRDF LUT split-sum corner values, finite nonnegative RG texels, and clamped public texel indexing,
- descriptor-driven WGPU PMREM/SH9/IEM readback resource requirements for artifact section acquisition,
- manual real-HDRI source-vs-PMREM mip-grid export for raw mip blur inspection,
- 10x10 real-HDRI PBR metallic/smoothness matrix metrics,
- single-metal-sphere HDRI reflection assertion,
- perfect-mirror HDRI reflection orientation and left/right grazing balance for orthographic and perspective cameras,
- real PBR texture-map material variation assertion using Color/NormalGL/Roughness/Metalness inputs.

The focused validation commands used for this slice include:

```powershell
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never
cargo test -p zircon_runtime --lib source_cubemap --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_brdf_lut_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_cubemap_projection_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --ignored --exact export_runtime_shader_pbr_real_hdri_reflection_png --nocapture --test-threads=1
cargo test -p zircon_runtime --lib source_cubemap_linear_sampling_bleeds_across_face_edges --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --nocapture
cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --ignored --exact export_runtime_shader_pbr_real_hdri_2k_reflection_png --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-png-metrics-0706 --message-format short --color never runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics -- --exact --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_source_irradiance_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-iem-contract-0706 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-iem-contract-0706 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_ibl_artifact_source_identity_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\ZirconBuilds\shader-pbr-artifact-source-layout-20260713 --color never -- --nocapture
cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-iem-contract-0706 --message-format short --color never runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics -- --exact --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-artifact-apply-0706 --message-format short --color never -- --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-artifact-resolve-0706 --message-format short --color never -- --nocapture --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-text-default-fix-0706 --message-format short --color never -- --ignored --exact export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png --nocapture --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-text-default-fix-0706 --message-format short --color never runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics -- --exact --nocapture --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-text-default-fix-0706 --message-format short --color never -- --nocapture --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib read_ibl_bake_artifact_sections --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-section-readback-0706 --message-format short --color never -- --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-source-cubemap-pmrem-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib source_cubemap --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-source-cubemap-pmrem-contract-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-pmrem-saturated-tail-0706 --message-format short --color never -- --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-hdri-export-pmrem-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png --locked --jobs 1 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-pmrem-saturated-tail-0706 --message-format short --color never -- --exact --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-hdri-export-pmrem-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export export_runtime_shader_pbr_real_hdri_reflection_png --locked --jobs 1 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-hdri-export-pmrem-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export export_runtime_shader_pbr_real_hdri_2k_reflection_png --locked --jobs 1 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-source-cubemap-mipmap-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --no-default-features --features core-min --locked --no-run --jobs 1 --color never
E:\cargo-targets\zircon-source-cubemap-mipmap-0706\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe --ignored --exact export_runtime_shader_pbr_real_hdri_2k_reflection_png --nocapture --test-threads=1
E:\cargo-targets\zircon-source-cubemap-mipmap-0706\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe --ignored --exact export_runtime_shader_pbr_real_hdri_single_reflection_png --nocapture --test-threads=1
E:\cargo-targets\zircon-source-cubemap-mipmap-0706\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe --ignored --exact export_runtime_shader_pbr_real_hdri_textured_material_png --nocapture --test-threads=1
E:\cargo-targets\zircon-source-cubemap-mipmap-0706\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe --exact runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-reflection-orientation-0707'; $env:CARGO_INCREMENTAL='0'; cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --color never
E:\cargo-targets\zircon-reflection-orientation-0707\debug\deps\runtime_shader_pbr_hdri_export-3eb8718632dd95b6.exe export_runtime_shader_pbr_real_hdri_mirror_reflection_png --ignored --nocapture --test-threads=1
E:\cargo-targets\zircon-reflection-orientation-0707\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe export_runtime_shader_pbr_real_hdri_ambientcg_metal008_025_029_png --ignored --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-cmft-skybox-0707'; cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --color never
E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\runtime_shader_pbr_hdri_export-ddba9a07f2a7f0d8.exe export_runtime_shader_pbr_real_hdri_mirror_reflection_png --ignored --nocapture --test-threads=1
E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\runtime_shader_pbr_hdri_export-ddba9a07f2a7f0d8.exe runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_orientation_and_grazing_metrics --exact --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cmft-skybox-0707 --color never -- --exact --nocapture --test-threads=1
cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png_matches_blur_metrics --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cmft-skybox-0707 --color never -- --exact --nocapture --test-threads=1
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-ibl-artifact-seam-0708'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_seam_contract --no-default-features --features core-min --locked --jobs 1 --color never -- --nocapture --test-threads=1
E:\cargo-targets\zircon-ibl-wgpu-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::backend::render_backend::read_ibl_bake_artifact_sections::tests::readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip --exact --nocapture --test-threads=1
```

The focused source-cubemap integration test passed 9/9 after EC-M3c landed, covering the prior PMREM and SH9 assertions, HDR values above 1.0, PMREM blur relative to the same-level regular source mip, progressive variance reduction across rougher PMREM mips, final 1x1 mip face averaging, and rough-mip cube-edge seam-energy reduction. The EC-M3b first focused Cargo run timed out at the tool boundary while the Windows cold compile continued in background; after cargo/rustc naturally exited, the same command passed 8/8 with a 3.56s test body and a 7m53s warmed total. EC-M3c then reran the same focused contract command and passed 9/9 with a 4.74s test body and a 7m18s total.

EC-M3d added the direct cross-face unit guard `source_cubemap_linear_sampling_bleeds_across_face_edges`, which passed 1/1 after verifying that a bilinear sample near a cube edge receives a neighbor-face contribution instead of clamp-to-edge color. After the cross-face source mip, 128-sample high-roughness PMREM, and earlier saturated-tail work landed, `runtime_environment_source_cubemap_contract` passed 9/9 again under `E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705` with a 5.28s test body. `runtime_environment_brdf_lut_contract` passed 2/2 earlier, covering split-sum corner values and public texel indexing. `cargo check -p zircon_runtime --lib --no-default-features --features core-min` also passed for the ABI update before EC-M3d.

The ignored HDRI screenshot export passed 1/1 after the source/specular cube split, RGBA16F upload, BRDF LUT binding, shadow-map fallback bind group ABI, and shared scene layout visibility were aligned. It wrote [runtime_shader_pbr_real_hdri_lakes_hdr_pmrem_reflection_20260705.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_hdr_pmrem_reflection_20260705.png). The PNG is 1280x960, 845,069 bytes, SHA256 `64D5873A09DDC348C15A2444221DF07961A47A34D5FB281B73F7122FEAB0782E`, with 74,903 unique colors.

The EC-M3d/EC-M2r 2K HDRI export used Poly Haven `lakes` 2K from [polyhaven_lakes_2k.hdr](/E:/Git/ZirconEngine/docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr). The HDR file is 5,918,432 bytes, SHA256 `B2506E0EE912C4C599FF013566FBD3ECAAC2F4B176319D450CCE0DE5758FED98`. That historical export wrote [runtime_shader_pbr_real_hdri_lakes_2k_hdr_pmrem_reflection_20260705.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_hdr_pmrem_reflection_20260705.png), 1280x960, SHA256 `7878B8A2CFF3DE85E1528D88DC86909F7AFC431206BB12FD7EFA3854F2E90606`. It remains as prior evidence only; EC-M2u supersedes it with the cosine PMREM screenshots listed below.

EC-M3e splits the 8x8 HDRI matrix assertions into `runtime_shader_pbr_hdri_export/hdri_metrics.rs` and adds the non-ignored `runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics` regression. The test reads the accepted 2K PNG from `docs/tests/runtime/shader`, checks the 1280x960 dimensions, and reruns the sky variation, metallic/smoothness response, and legacy 16x8 grid-seam assertions without regenerating the screenshot. The focused command passed 1/1 with 0 ignored and 2 filtered tests under `E:\cargo-targets\zircon-hdri-png-metrics-0706`; the test body was 0.23s after the 8m43s build, stderr contained only existing workspace warnings, and same-name scans under `target` and `E:\cargo-targets` returned zero hits.

EC-M3g/EC-M2r added the earlier ignored manual diagnostic `export_runtime_shader_pbr_real_hdri_1k_pmrem_mip_diagnostic_png`. It wrote [runtime_shader_pbr_real_hdri_lakes_1k_pmrem_mip_diagnostic_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_1k_pmrem_mip_diagnostic_20260706.png), an 864x1152 PNG of 954,870 bytes with SHA256 `12BF0D51060CCB0A53C1BCC4A97D6118D0B55004FB8619D46C27B67869DF26A2`. The top half shows `source_texels()` regular mips; the bottom half shows `texels()` GGX PMREM mips. This diagnostic is historical now; the current full-roughness cosine diagnostic is the EC-M2u file listed next.

EC-M2u supersedes the EC-M2r saturated-tail downsample experiment. The module-local guard is now `source_cubemap_saturated_roughness_mip_uses_cosine_convolution`: it builds a high-frequency synthetic HDRI, computes the roughness=1.0 selected PMREM mip, rejects equality with ordinary trilinear downsampling from the previous PMREM mip, and requires lower luma variance than the previous mip. The counted focused lib run used `E:\cargo-targets\zircon-source-cubemap-pmrem-0706` and passed 15/15 source-cubemap tests with 6897 filtered after the PMREM owner split.

EC-M2w aligns the public integration contract with the EC-M2u PMREM behavior. `runtime_environment_source_cubemap_saturated_roughness_mip_uses_cosine_convolution` now performs the same public-surface checks as the module-local guard instead of asserting equality with previous-PMREM downsampling. The first focused integration command exceeded the 1204s tool window during Windows cold compilation while cargo/rustc continued in the background; after that process finished, the same command passed 10/10 with a 6.63s test body under `E:\cargo-targets\zircon-source-cubemap-pmrem-contract-0706`.

EC-M2x adds a focused renderer-local shader-plan check for GPU PMREM parity. `ibl_bake_shader_plan` passed 5/5 under `E:\cargo-targets\zircon-ibl-shader-plan-check-0706`, including Naga parsing for all three IBL bake WGSL kernels and static assertions that the PMREM kernel keeps the cosine full-roughness branch, PDF-driven source-lod selection, Unreal texel solid-angle scale, centered Hammersley samples, and high-roughness 128 sample dispatch parameter.

EC-M2u refreshed the manual HDRI outputs after the cosine full-roughness PMREM change. The 1K diagnostic is [runtime_shader_pbr_real_hdri_lakes_1k_cosine_pmrem_mip_diagnostic_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_1k_cosine_pmrem_mip_diagnostic_20260706.png), 864x1152, SHA256 `6F7B37E061A04C3701CD5384F13F5A21159992527C65F21FA3C2D4210B1DAAB1`. The 1K PBR matrix is [runtime_shader_pbr_real_hdri_lakes_cosine_pmrem_reflection_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_cosine_pmrem_reflection_20260706.png), 1280x960, SHA256 `B7BEBB66E806667B35A1B5792D5698607C683931DA7EDB78339DEEED86CD15C5`. The 2K PBR matrix is [runtime_shader_pbr_real_hdri_lakes_2k_cosine_pmrem_reflection_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_cosine_pmrem_reflection_20260706.png), 1280x960, SHA256 `4C974C6D1A43309032B0B1718762F9B9566B19C5D02DF3B954D45E5988831DCD`. The three ignored export tests each passed 1/1, `target` contained no `*cosine_pmrem*.png`, and the external export target was cleaned after free space fell below the repository threshold.

EC-M3i refreshes the manual mip diagnostic after the UE-style angular source mip implementation. The diagnostic is [runtime_shader_pbr_real_hdri_lakes_1k_angular_source_pmrem_mip_diagnostic_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_1k_angular_source_pmrem_mip_diagnostic_20260706.png), 936,165 bytes, SHA256 `FD448CF295A6FCEE139994F2516C0C729086C279615D47C7B114C820FEF8C692`. The focused source-cubemap contract direct binary under `E:\cargo-targets\zircon-source-cubemap-mipmap-0706` passed 12/12, including angular source-mip blur and roughest 1x1 six-face averaging. Same-name scans under repository `target` and `E:\cargo-targets` found zero PNG copies.

EC-M3j upgrades the visual PBR acceptance from 8x8 to 10x10 and adds two focused material views. The 10x10 output is [runtime_shader_pbr_real_hdri_lakes_2k_10x10_cosine_pmrem_reflection_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_10x10_cosine_pmrem_reflection_20260706.png), 1,439,659 bytes, SHA256 `EC7AEE773AE2CE5E548E1621D2BBD19BFFA06C8B10EAB9C82BEA85080364B07A`. The single metal sphere output is [runtime_shader_pbr_real_hdri_lakes_single_metal_sphere_reflection_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_single_metal_sphere_reflection_20260706.png), 808,790 bytes, SHA256 `C36E78FE7812E5DBA95F98CF5B185C700E38D243BF5AD4AD51F276B8486181A4`. The texture-driven material output is [runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260706.png), 965,226 bytes, SHA256 `35873B136AE64A2127F1864A65AF3E45D2CB99AC19EBAE4E737B4166731F4BA8`. The ignored direct binary exports passed 1/1 each, the saved-PNG metrics regression passed 1/1, and same-name scans under repository `target` plus `E:\cargo-targets` found zero PNG copies.

EC-M3k adds the camera-correct reflection direction regression and the requested ambientCG Metal008/025/029 validation set. The perfect mirror orthographic output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png), 935,722 bytes, SHA256 `9667EEBA655665AFF78DD80B1895C11651771DEF87ACC31F1FC9915E6D71460F`. The perfect mirror perspective output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png), 823,873 bytes, SHA256 `1FA18935B01B6C8199BBFFB18F482899BC6578D124C2EBA4ABC1270204724278`. The additional texture-map material outputs are [runtime_shader_pbr_real_hdri_lakes_ambientcg_metal008_texture_maps_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal008_texture_maps_20260707.png), 1,019,285 bytes, SHA256 `FE62236AE3A3401C1C77893CA97086D4437C58F22B2B7F4C2AE7AE19DA72530A`; [runtime_shader_pbr_real_hdri_lakes_ambientcg_metal025_texture_maps_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal025_texture_maps_20260707.png), 1,100,587 bytes, SHA256 `2C518C372C42D586157A5654833B69B2CE4CA91C0527C58B916468A16536E01A`; and [runtime_shader_pbr_real_hdri_lakes_ambientcg_metal029_texture_maps_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal029_texture_maps_20260707.png), 1,151,790 bytes, SHA256 `14DC53B73A92D3D4404F8906769C8084A1B22DC43FA9B341193A6F62E76C1E74`. Direct execution of the already-built integration-test binary passed the mirror export 1/1 and ambientCG batch 1/1. The Cargo wrapper attempts for the mirror export exceeded the tool window during this session, so they are not counted as pass evidence; the direct binary rerun is the counted exit-0 proof.

EC-M3l supersedes the EC-M3k mirror-image hashes for the two perfect-mirror screenshots after fixing the standard material roughness floor. The old `0.04` floor made authored `roughness=0` sample a blurred PMREM mip instead of mip0, which showed up as a white/gray mirror ball. `STANDARD_MATERIAL_MIN_ROUGHNESS` is now `0.001` across the material uniform, generated StandardPBR surface, GBuffer encode, deferred lighting, fallback mesh, SSR, and fixture paths. The refreshed orthographic mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png), 1,105,989 bytes, SHA256 `8109D779CCCED49354C2B5FAACF4D6561F12287C7537E7ED621CF80D5C61AB4B`, with mirror stats `mean_luma=143.45,luma_std=49.43,mean_sat=0.1031,clip=0.1114`. The refreshed perspective mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png), 1,202,996 bytes, SHA256 `49414938370F7ECD1AD7B182E3815CE442F7141CE8DE1515BE025DC01FA1651C`, with mirror stats `mean_luma=130.58,luma_std=39.95,mean_sat=0.0624,clip=0.0820`. The direct mirror export binary passed 1/1 in 254.40s, and same-name scans under repository `target` plus `E:\cargo-targets\zircon-reflection-orientation-0707` found zero PNG copies.

EC-M3m supersedes the EC-M3l mirror-image hashes after fixing the remaining orientation issue. The validation UV sphere now uses outward triangle winding so single-sided rendering shows the front shell instead of the back shell, and `skybox_procedural.wgsl` samples screen-space source cubemap directions with `-Z` to match Zircon's default camera convention. The updated orthographic mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png), 1,010,236 bytes, SHA256 `E6E6E09DEC7EA4014D0A1302641DFD558111B665D8255D0282A78C3289C58992`, with mirror stats `mean_luma=164.73,luma_std=45.35,mean_sat=0.2234,clip=0.1151` and upper/lower blue-excess `69.46/9.65`. The updated perspective mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png), 1,055,118 bytes, SHA256 `4DE32672A09851B2A304461FA1E5938ECDA5531D20B9901B04CCDD6BE424BADB`, with mirror stats `mean_luma=192.52,luma_std=41.85,mean_sat=0.2419,clip=0.2790` and upper/lower blue-excess `79.14/-6.98`. The counted direct mirror export binary passed 1/1 in 227.66s after the strengthened orientation assertions, and same-name scans under repository `target`, `E:\cargo-targets\zircon-reflection-orientation-0707`, and `E:\cargo-targets\zircon-reflection-orientation-0707-m3m` found zero PNG copies.

EC-M3n supersedes the EC-M3m mirror-image hashes after the cmft/cmftStudio alignment pass. PMREM roughness now maps linearly to mip level, PMREM generation uses the cmft Blinn/cosine-power radiance lobe, shader-side source/specular/IEM/skybox cube sampling applies `fixCubeLookup`, and skybox source-cubemap directions are reconstructed from camera matrices for both perspective and orthographic cameras. The refreshed orthographic mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png), 1,011,195 bytes, SHA256 `B3D9EC6B22671BF6E663BDBD130CDFE72BAB8F3AA059B3C25BC44E258721819E`. The refreshed perspective mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png), 827,376 bytes, SHA256 `8515068DAF9716AB8AA2DDD2B6A391C9A3F3035C4CC83663663141C4D8A9A142`. Counted validation under `E:\cargo-targets\zircon-cmft-skybox-0707` passed: `skybox_shader_is_valid_wgsl` via Cargo, direct lib-test runs for `source_cubemap_cmft_pmrem` 2/2, `skybox_shader_is_valid_wgsl` 1/1, `skybox_shader_reconstructs_camera_ray_before_source_cubemap_sampling` 1/1, and `deferred_lighting_shader_applies_environment_reflections_to_standard_pbr` 1/1. The integration test no-run build passed, the direct mirror export passed 1/1 in 288.44s, and same-name scans under repository `target` plus `E:\cargo-targets` found zero PNG copies.

EC-M3o supersedes the EC-M3n mirror-image hashes after adding the standard-material normal-map feature guard and grazing-balance assertion. The refreshed orthographic mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png), 1,033,454 bytes, SHA256 `9B294F637F66CAB25DBD1B0C711C557C55749ABFB967837D0D41CC7EDEF13825`, with left/right grazing luma `141.15/129.07` and stddev ratio `0.972`. The refreshed perspective mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png), 897,232 bytes, SHA256 `072DE7EDD7A3D9522B8350FFF4F9401F1DD3F7A14285C22923302F5DAAFE3727`, with left/right grazing luma `152.59/162.11` and stddev ratio `1.535`. The direct mirror export passed 1/1 in 281.59s after the strengthened assertion, and same-name scans under repository `target` plus `E:\cargo-targets` found zero PNG copies.

EC-M3p supersedes the EC-M3o mirror-image hashes after fixing the frame/render-region projection aspect used by scene uniforms. The root cause of the remaining right-side grazing exaggeration was that `SceneUniform::from_frame(...)` built `ViewProjectionMatrixPair` from `RenderFrameExtract.view.effective_render_size()`, which can be stale 1x1 in snapshot/export paths while the render pass writes 1280x960. It now uses `frame.render_region().local_size()`, and the new unit guard `scene_uniform_uses_frame_render_region_size_for_projection_aspect` locks the 1280x960 orthographic scale to `0.75/1.0` even when the extract still reports 1x1. The refreshed perspective mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png), 909,796 bytes, SHA256 `D22CF31B3BF9742B4077E4A52608FC68EE9F47DD492241E26FB8D4847D4E62A8`, with left/right grazing luma `138.24/133.67`, mean ratio `0.967`, and stddev ratio `0.985`. The refreshed orthographic mirror output is [runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png), 1,016,494 bytes, SHA256 `C5D273AFD58880BD03295BF2077975ED1844A0B78C0F4D81A8B5159358107572`, with left/right grazing luma `137.40/122.42`, mean ratio `0.891`, and stddev ratio `0.936`. The counted checks are the cargo check listed above, the direct mirror export 1/1 in 225.23s, and the saved-PNG orientation/grazing regression 1/1 in 0.64s. The exact lib-test Cargo wrapper for the new unit guard did not produce counted pass evidence in this workspace because one run exited rustc without diagnostics and the rerun timed out at 364s; same-name scans under repository `target` plus `E:\cargo-targets` found zero PNG copies.

EC-M3q strengthens the saved 2K PBR matrix metrics with a screenshot-level high-frequency roughness response gate. `hdri_metrics.rs` now samples the center disk of high-metal cells in [runtime_shader_pbr_real_hdri_lakes_2k_10x10_cosine_pmrem_reflection_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_10x10_cosine_pmrem_reflection_20260706.png), excludes the tiny center and avoids the silhouette edge, then compares rough, mid-smooth, and smooth row groups. The current rough/mid/smooth high-frequency energies are approximately `15.52/17.96/18.85`; the assertion requires mid > rough x 1.05 and smooth > rough x 1.10. The counted command listed above passed 1/1 with a 0.46s test body after the 2m26s focused integration-test build. This slice does not create a new screenshot; it makes the archived 2K 10x10 screenshot a stronger regression artifact.

EC-M3r adds the saved PMREM mip diagnostic blur gate for [runtime_shader_pbr_real_hdri_lakes_1k_angular_source_pmrem_mip_diagnostic_20260706.png](/E:/Git/ZirconEngine/docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_1k_angular_source_pmrem_mip_diagnostic_20260706.png). The diagnostic is a 96px tile grid: source cubemap mips occupy the first six face rows and PMREM mips occupy the next six face rows. The saved-PNG assertion verifies mip0 equality, then requires non-flat PMREM rough mips to have lower high-frequency energy than the same-level source mips. Current measured values are mip0 source/PMREM `9.073/9.073`, mip1 `7.782/1.950`, and mip1..7 average PMREM/source ratio about `0.666`. The Cargo exact test passed 1/1 with a 0.29s test body after the 9m35s focused build; direct rerun from the latest integration-test binary passed the 2K matrix metrics in 0.46s and the PMREM diagnostic metrics in 0.35s.

EC-M3w adds `runtime_environment_ibl_bake_artifact_pmrem_roundtrip_preserves_seam_metrics`. The test preserves the PMREM seam gate across the artifact boundary: it encodes a seam-stress PMREM into the fixed RGBA16F artifact payload, decodes the current blob for a matching request, applies it to the source chain, verifies mid/rough seam statistics remain within `0.003` of the pre-encoded PMREM, and reasserts that the applied rough mips reduce mean and worst seam deltas versus mip0. The focused command under `E:\cargo-targets\zircon-ibl-artifact-seam-0708` passed 1/1; the test body was 1.83s after an 18m57s cold build, with only existing workspace warnings.

EC-M3x adds `readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip` under the backend readback module. The test uses an offscreen WGPU backend, uploads the synthetic seam-stress PMREM artifact bytes into a real `Rgba16Float` cube texture with padded write rows, reads PMREM plus SH9 back into `IblBakeArtifactReadbackSections`, asserts exact payload byte equality, applies the payload to the source chain, and verifies mid/rough seam statistics stay within `0.003` while rough mips still reduce worst seam delta versus mip0. The final generated lib-test binary under `E:\cargo-targets\zircon-ibl-wgpu-readback-seam-0708` passed 1/1 with a 1.81s test body; Cargo-wrapper attempts built/linked in the same target but exceeded the tool window and are not counted as passing wrapper evidence.

EC-M3z adds `runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance` under `ibl_bake_runtime_writeback.rs`. The test uses an offscreen WGPU backend, uploads a 32x32 face-size low-frequency directional source cubemap, dispatches the real IEM graph pass, writes the produced graph output into runtime cache, confirms the next dispatch has zero runtime compute, and checks the decoded IEM is non-black, directionally varied, and correlated with the CPU direct-convolution IEM after scale normalization. The final generated lib-test binary under `E:\cargo-targets\zircon-ibl-live-iem-readback-0708` passed 1/1 with an 8.14s test body after the Cargo wrapper timed out during cold compile. This slice generated no screenshot.

EC-M2p adds the WGPU artifact section acquisition contract in `read_ibl_bake_artifact_sections.rs`. The focused lib test passed 3/3 under `E:\cargo-targets\zircon-ibl-wgpu-section-readback-0706`, covering descriptor preservation, descriptor-driven PMREM/SH9/IEM resource requirements, and missing-resource error labels. This slice generated no screenshot and does not claim render-graph scheduling, compute bake production, or cache writeback dispatch integration.

EC-M2d adds the optional CPU IEM reference bridge in `source_irradiance_cubemap.rs`. The first focused Cargo run failed because the root `core::framework::render` facade did not yet re-export the new IEM symbols; after adding those facade exports, `runtime_environment_source_irradiance_cubemap_contract` passed 2/2 under `E:\cargo-targets\zircon-hdri-iem-contract-0706`. A later Cargo wrapper recheck timed out at the tool boundary after 904s and is not counted as pass evidence; direct execution of the already-built test binary then passed 2/2 in 15.85s. The accepted test run covered constant diffuse environment preservation and low-frequency agreement with the SH9 bridge. This slice generated no screenshot.

EC-M2j adds the runtime IEM carrier/binding bridge. `runtime_environment_ibl_bake_artifact_contract` passed 12/12 under `E:\cargo-targets\zircon-hdri-iem-contract-0706`, including `runtime_environment_ibl_bake_artifact_payload_applies_optional_iem_to_environment`, which verifies PMREM/SH9 replacement, source mip preservation, IEM attachment, and runtime intensity/rotation preservation. The non-ignored saved-PNG metrics regression `runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics` passed 1/1 with 2 filtered tests under the same target, confirming the accepted 2K screenshot still satisfies the Plan 06 matrix metrics after recompiling the new 6-binding render code. Two earlier exact lib-test attempts for the new layout test were blocked before running target assertions by the then-current `MeasureKey: Default` compile error in the then-current `zircon_runtime/src/graphics/text/cache/tests.rs` owner; that historical blocker should not be treated as layout-specific pass evidence, and the exact layout assertion still needs a correctly listed rerun before it can be counted.

EC-M2i adds `source_cubemap_artifact.rs` as the child owner for applying decoded IBL artifact payloads back to the source-cubemap model. `source_cubemap_mip_chain_with_bake_artifact(...)` preserves regular source/display texels while replacing the specular PMREM chain and SH9 coefficients. `source_cubemap_environment_with_bake_artifact(...)` preserves source revision/hash, intensity, and rotation, and attaches optional IEM data as `SourceCubemapIrradianceCube`. The focused artifact contract passed 12/12 under `E:\cargo-targets\zircon-ibl-artifact-apply-0706`; logs are `docs/tests/runtime/render/plan11_ibl_artifact_payload_apply_current_source_cargo_20260706.{out,err}.log` and `.exit.txt`. This is a CPU payload-application contract only; GPU/offline bake, readback writeback, derived importer/staged production, and product second-launch dispatch proof remain open.

The EC-M3a screenshot guard samples every cell of the 8x8 metallic/smoothness matrix. The current run measured luma range `124.501..233.256`, smooth-metal versus smooth-dielectric endpoint delta `85.106`, smooth-row metallic group delta `32.130`, high-metal rough/smooth group delta `59.064`, responsive row/column counts `8/8` and `6/8`, and legacy 16x8 grid boundary means `0.02491` vertical / `0.02711` horizontal. Search confirmed this screenshot exists under `docs/tests/runtime/shader` and not under `target` or `E:\cargo-targets`.

## Open Issues

The complete EC-M2/EC-M3 chain is still pending: broader product WGPU command scheduling/readback/cache writeback from live compute outputs, offline BRDF LUT bake production, compressed/Basis external cubemap transcoding, six-file/cross authoring inputs, runtime quality/specialization ownership for enabling `ZR_ENV_DIFFUSE_IEM`, importer-produced artifact seam comparison, stricter automated SSIM against direct source-cubemap references, additional RenderDoc product captures, probe capture/blending, and higher-resolution 4K/16K offline bake acceptance beyond the current angular source-mip contract, artifact payload seam roundtrip, backend WGPU texture readback seam guard, live PMREM/IEM graph-output writeback guards, 10x10 matrix guard, mirror orientation export, single-sphere export, and texture-map material exports.
## 2026-07-11 dual-layout verification

- `source_texel` and `pmrem_texel` now expose symmetric, layout-specific reads; source projection tests no longer sample fixed-PMREM coordinates by accident.
- WGPU bake dispatch uses the fixed 128 face size and eight PMREM mips. The focused GPU group passes 39/39 and the backend RGBA16F readback seam roundtrip passes 1/1.
- Product EC-M3 remains open until the plan's complete GPU/offline per-texel parity gate is recorded.

## 2026-07-13 CPU/GPU PMREM algorithm convergence

The CPU PMREM owner in `source_cubemap/pmrem.rs` now implements the same UE-style GGX filtered-importance contract as `ibl_prefilter.wgsl`. Both paths use the same Hammersley sequence, `alpha = roughness^2`, `D_GGX / 4` light-direction PDF for `V=N`, PDF-derived source mip without a destination-footprint lower bound, the full-roughness cosine branch, and the Normal-quality 32/64/128 sample tiers. Mip-zero direct copies still use the destination footprint, and the final 1x1 face average remains shared across all six faces. This is a hard convergence of the common PMREM algorithm: no cmft cosine-power production branch or test-only reference algorithm remains in the CPU source-cubemap path.

The same audit clamps externally requested source and PMREM mip counts to physically available chains before validating storage or indexing it. SH9-only artifact encoding no longer requires a PMREM layout match; PMREM layout validation runs only when PMREM bytes are requested. The focused `core-min` regressions for filtered LOD, external source-mip construction, and SH9-only payloads each pass 1/1, while the graphics-only WGSL contract passes 1/1. The default-feature lib wrapper was not counted because an unrelated `sdf_font_bake/tests/offline.rs` compile error lacked the `AssetManager` trait import.

The source-identity review regression first failed 0/1 because a descriptor with the same bake key and fixed PMREM 128x8 still accepted source 256x9 after being authored for source 128x8. After the format-v2 hard cut, `runtime_environment_ibl_artifact_source_identity_contract` passes 1/1 and verifies `current_for_request(...)` retains source 256x9. A broader artifact-contract rerun was blocked before its test body by unrelated concurrent bridge re-export errors (`BridgeInterfaceStatus`, `InterfaceSlot`, and `BridgeOwnerTransitionMode`); it is not counted as an artifact failure.

A follow-up review also found one runtime-writeback test fixture still authored its SH9 blob with the legacy PMREM-only `current(...)` helper even though the request source layout was 16x16/mip5. The fixture now uses `current_for_request(request)`, matching the production writeback path. Its exact graphics lib-test wrapper did not enter the test body because the shared workspace currently has three unrelated test compilation errors: a missing `approx` test dependency in text decoration metrics and two UI decoration fixtures relying on interface types that no longer implement `Default`. This external blocker is recorded without claiming the exact runtime-writeback test passed.

The implementation is grounded in Unreal Engine `ReflectionEnvironmentShaders.usf` / `MonteCarlo.ush` and cross-checked against cmft/cmftStudio for cubemap projection, face/edge handling, and final-face averaging. The ignored 8x8 product test passes after the integration harness split into 598/231/572/750-line folder-backed owners. Its accepted matrix has mirror SSIM `0.998698`, dielectric center F0 response `0.041225`, dielectric grazing response `0.266590`, and rough-metal luma `0.500247`; the PNG SHA256 is `68738E5E792AD428B48E8F89AF234F33FF3A6FA0EAE778EC3AF01465471C4D38`.

## 2026-07-16 WGPU seamless cubemap lookup

cmft and cmftStudio retain `fixCubeLookup` for older APIs that do not guarantee seamless filtering across cubemap faces. Zircon's WGPU environment and procedural-skybox shaders now preserve their normalized lookup directions instead: WGPU performs native cross-face filtering, so the historical LOD/face-size warp would over-compress high-roughness and grazing reflections. `runtime_environment_wgpu_cubemap_sampling_contract.rs` locks both shader boundaries to an identity direction and rejects the old `adjusted` state and `exp2` edge warp. The GGX PMREM bake remains unchanged and continues to be the only roughness blur source.
