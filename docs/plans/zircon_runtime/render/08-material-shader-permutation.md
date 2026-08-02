---
related_code:
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_blinn_phong.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_unlit.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_geometry_source_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/ownership.rs
  - zircon_runtime/src/plugin/extension_registry/register/metadata.rs
  - zircon_runtime/src/plugin/extension_registry/access/metadata.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/package_contributions/manifest_metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/graphics/material/shading_models/registry.rs
  - zircon_runtime/src/graphics/material/shading_models/builtins.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_construction.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_shading_models.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/material_custom_shading_model_runtime.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_revision.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_shading_model_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_geometry_source_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_runtime_staged_cache_hit.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mesh_pipeline_variant_cache_owner.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build.py
  - tools/tests/test_zircon_build_shader_prewarm.py
  - tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/management.rs
  - zircon_runtime/src/core/framework/render/material/management/tests.rs
  - zircon_runtime/src/core/framework/render/material/management/tests/record_views.rs
  - zircon_runtime/src/core/framework/render/material/management/tests/query_execution.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests/uniform_debug_counts.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_material_product_debug_counts_tests.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/src/graphics/tests/project_render/render_quality.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_project_scene_products_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime/pbr_projection.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/material_runtime_pbr_projection_tests.rs
  - zircon_runtime/src/graphics/material/mod.rs
  - zircon_runtime/src/graphics/material/shading_models/mod.rs
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/mod.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/fixture.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/product_png.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_second_launch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_material_management_tests.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/VertexFactory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MaterialShader.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshMaterialShader.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/GlobalShader.h
  - dev/Graphics/Packages/com.unity.shadergraph
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_material/src/specialize.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_pbr/src/render/skin.rs
  - dev/bevy/crates/bevy_pbr/src/render/morph.rs
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
plan_sources:
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
---

# 计划 08:材质、光照模型与 shader permutation 管理

## 目标

1. 建立 VertexFactory 等价物 `GeometrySource`:static / skinned / morph / instanced / VG 等几何来源与材质着色正交组合,GPU skinning/morph 对任意材质生效(不再只有 fallback shader)。
2. 建立光照模型(shading model)注册体系:`Unlit` / `BlinnPhong` / `StandardPbr` / 自定义模型,材质声明模型,deferred G-buffer 编码与 forward 着色按模型分发。
3. permutation 管理:(材质 shader, GeometrySource, pass 类型, feature 开关) → 变体键 → 编译缓存(内存 + 磁盘),离线预热工具,缺失变体诊断。
4. `mesh_pipeline_cache` 重构在统一变体键之上,与 shader graph 插件编译路径收敛。

## 现状与差距

- skinning 由 `fallback_mesh_shader_source` 拼接进 fallback WGSL,自定义材质(zshader/shader graph)无法获得 GPU 形变;morph 仅 CPU。
- shader 变体键(`definition_value.rs`)存在但维度不全(无几何源、pass 维度不显式);无磁盘缓存与预热,首次遇到变体即时编译造成卡顿。
- 光照模型单一:standard material 走固定 PBR 路径,无 unlit/blinn-phong/自定义模型位;deferred G-buffer 编码未按模型抽象。
- SRP bind group ABI(group 0 model / 1 material / 2 view)中 skinning 绑定硬编码在 mesh shader,几何源差异没有抽象层。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../RenderCore/Public/VertexFactory.h` | 顶点工厂抽象:几何源声明顶点流/位置获取函数,与材质 shader 正交;permutation 维度如何进 shader map 键 |
| `dev/UnrealEngine/.../Renderer/Public/MaterialShader.h` + `MeshMaterialShader.h` | 材质 shader map:(material, vertex factory, shader type) 三维组合、permutation domain、lazy compile + DDC 缓存的层次 |
| `dev/UnrealEngine/.../RenderCore/Public/GlobalShader.h` | 非材质全局 shader(后处理/compute)的独立管理 —— 对应本引擎 post/compute WGSL 不进材质变体空间 |
| `dev/Graphics/Packages/com.unity.shadergraph` | shader graph → 模板代码生成:graph 输出函数 + 管线模板拼接(本引擎 zshader/shader_graph 插件的生成式参考);Unity 的 multi_compile/shader_feature 即变体开关先例 |

次参考:`dev/bevy/crates/bevy_pbr`(`MeshPipelineKey` 位打包变体键的 Rust 表达;`StandardMaterial` 的 shader defs 注入)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_pbr/src/render/mesh.rs` | `ShaderVariantKey` 位打包与 pipeline specialization(重点) | `MeshPipelineKey: u64` bitflags 全集与 `MeshPipeline::specialize`(键 → shader defs → pipeline descriptor 的完整链);`ViewKeyCache` 的 per-view 键缓存 |
| `dev/bevy/crates/bevy_mesh/src/lib.rs` | 变体键维度分段 | `BaseMeshPipelineKey`:几何属性位段独立定义、高位保留给上层键合成(与 `packed_dims` 分段思路同构) |
| `dev/bevy/crates/bevy_material/src/specialize.rs` | 材质维度进变体键 | `ErasedMaterialPipelineKey` + specializer 函数指针表:材质类型擦除后统一走 `SpecializedMeshPipelines::specialize` |
| `dev/bevy/crates/bevy_shader/src/shader.rs` | defines header 注入 | `Shader.shader_defs`/`ShaderDefVal`(Bool/Int/UInt 值型 define,等价本计划 `RenderShaderDefinitionValue`) |
| `dev/bevy/crates/bevy_shader/src/shader_cache.rs` | 模板拼接器 + 变体缓存 | `naga_oil::compose::Composer` 的 include 组合与去重、`global_shader_defs`(平台 capability → define,即 `platform_token` 语义)、编译产物缓存键 |
| `dev/bevy/crates/bevy_pbr/src/render/skin.rs` | `SkinnedMesh` 几何源(storage palette) | `SkinUniforms::current_buffer/prev_buffer`:palette 升 storage buffer 与 prev palette 双缓冲(配 `skinning.wgsl` 的 uniform/storage 双形态) |
| `dev/bevy/crates/bevy_pbr/src/render/morph.rs` | `MorphedMesh` 几何源 | morph 权重双帧映射(`prev` 表)与 GPU 权重 buffer 布局(配 `morph.wgsl` 的 `prev_weight_at`) |
| `dev/Fyrox/fyrox-material/src/shader/mod.rs` | zshader 资产形态与 pass 集合 | `ShaderDefinition`:RON 数据驱动 shader(passes/properties/draw_parameters/disabled_passes),封闭 pass 枚举先例 |

`GeometrySource` × 材质 surface 函数的模板拼接(改名注入 `zr_material_surface`、pass 特化裁剪)无完整 Rust 同类参照(bevy 用 naga_oil import 组合而非 surface 函数模板),实现时以 UE `VertexFactory.h`/`MeshMaterialShader.h` 为唯一样板,按 index §8 第 8 条配对拍测试先行;磁盘变体缓存 + 离线预热同样无 Rust 同类参照(bevy 仅内存缓存),以 UE DDC 思路为样板。

## 目标架构

归属:`GeometrySource` 与变体键契约进 `core/framework/render/shader/`;WGSL 组装与缓存在 `graphics/shader/` 与 `graphics/scene/scene_renderer/mesh/`;shading model 注册在 `graphics/material/`。

核心设计:

- `GeometrySourceDescriptor`:声明顶点输入布局、形变阶段代码片段(WGSL include:`fetch_position/fetch_normal/...`)、所需绑定(skinning palette storage、morph 权重、instance index)。内建实现:`StaticMesh`、`SkinnedMesh`(palette 升 storage buffer,解除 256 uniform 限制)、`MorphedMesh`(GPU 权重混合)、组合型 `SkinnedMorphed`;VG 几何源由 virtual_geometry 插件注册。
- 材质 shader 模板:材质(手写 zshader 或 shader graph 产物)只提供 surface 函数(输入插值器,输出 `SurfaceOutput`);管线模板按 (GeometrySource, pass, shading model) 拼接最终 WGSL —— 自定义材质自动获得 skinning/morph/instancing。
- `ShadingModelDescriptor`:注册 forward 着色函数与 deferred G-buffer 编/解码函数;G-buffer 写 shading model id;内建 `Unlit`/`BlinnPhong`/`StandardPbr`,插件可注册自定义模型(描述符含所需 G-buffer 通道,超出当前布局时报诊断而非静默)。
- `ShaderVariantKey` 定稿:位打包 (material id+revision, geometry source, pass type, shading model, feature flags, quality);`mesh_pipeline_cache` 改为以此为键。
- `ShaderVariantCache`:内存 LRU + 磁盘缓存(键哈希 → 编译产物/naga 验证结果);`zircon_build.py` 增加预热步骤(枚举资产引用的变体离线编译);运行时 miss 记录进诊断(缺失变体报告)。

## 里程碑

### MS-M1 GeometrySource 与 WGSL 组装管线

实施切片:
1. `GeometrySourceDescriptor` 契约与四个内建实现;模板拼接器(surface 函数 + 几何源 include + pass 模板)。
2. 内建 standard material 切到模板路径;删除 fallback 拼接中被取代的 skinning 段。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime shader --locked`(组装产物 naga 验证)+ `render_product` 回归
- 验收证据:同一材质在 static/skinned 两种几何源下产出不同变体且渲染正确。

### MS-M2 GPU skinning/morph 全材质化

实施切片:
1. palette 升 storage buffer;prev palette(供计划 06)同步;morph 权重 GPU 混合。
2. 自定义材质(zshader)走模板获得形变;CPU skinning 收缩为能力回落档。

测试阶段:
- `cargo test -p zircon_runtime skinning --locked` 与动画场景 `render_product`
- 验收证据:自定义材质角色 GPU 蒙皮正确(与 CPU 路径产物对拍);palette 数量超 256 骨骼用例通过。

### MS-M3 shading model 注册体系

实施切片:
1. `ShadingModelDescriptor` 与内建三模型;G-buffer 编码加 model id;deferred lighting 按 id 分发。
2. 材质资产增加 shading model 字段;blinn-phong/unlit 内建材质模板。

测试阶段:
- `cargo test -p zircon_runtime material --locked`(三模型 forward/deferred 产物对拍一致性)
- 验收证据:同场景三模型混用,deferred 与 forward+ 产物一致。

### MS-M4 变体缓存与预热

实施切片:
1. `ShaderVariantKey` 定稿与 `mesh_pipeline_cache` 重构;磁盘缓存。
2. `tools/zircon_build.py` 预热步骤;缺失变体诊断报告。

测试阶段:
- `cargo test -p zircon_runtime shader --locked`(键稳定性:同输入跨进程同键;缓存命中)
- 验收证据:二次启动同场景零运行时编译(诊断计数);预热产物被运行时命中。

## 工程落地细化

本章是本计划的实施权威(见 index.md §8 第 7 条)。bind group 槽位、GPU 数据布局、`zr_` include、测试命名等全局约定直接引用 index.md §8,不在此重定义;facade 固定 `zircon_runtime::core::framework::render`,契约层不出现 `wgpu` 类型,全部硬切换,渲染侧只消费 `RenderFrameExtract`。

### 模块与文件落点

新增(契约归 `core/framework/render`,实现归 `graphics`,不新增 crate):

| 落点 | 内容 |
|------|------|
| `zircon_runtime/src/core/framework/render/shader/geometry_source.rs` | `GeometrySourceId`、`GeometrySourceDescriptor` 契约(无 wgpu) |
| `zircon_runtime/src/core/framework/render/material/shading_model.rs` | `ShadingModelId`、`ShadingModelDescriptor`、`GBufferChannelMask` 契约 |
| `zircon_runtime/src/graphics/shader/geometry_sources/{mod,static_mesh,skinned_mesh,morphed_mesh,skinned_morphed}.rs` | 四个内建几何源描述符构造 + 注册表(插件注册入口,VG 经 virtual_geometry 插件走此口) |
| `zircon_runtime/src/graphics/shader/template/{mod,assemble,module_registry,pass_specialization}.rs` | 模板拼接器:include 注入去重、surface 函数改名、pass 特化裁剪、`ZrVertexInput` 生成 |
| `zircon_runtime/src/graphics/shader/variant_cache/{mod,resolve,disk,prewarm}.rs` | `ShaderVariantCache`:内存 interning + LRU、磁盘缓存、预热清单、缺失变体诊断 |
| `zircon_runtime/src/graphics/material/shading_models/{mod,registry,unlit,blinn_phong,standard_pbr}.rs` | shading model 注册表与三内建模型(含 G-buffer model id 编码) |
| `zircon_runtime/src/graphics/shader/wgsl/zr_geometry_{static,skinned,morphed,skinned_morphed}.wgsl` | 几何源 include(`fetch_*` 族),经 `include_str!` 内嵌 |
| `zircon_runtime/src/graphics/shader/wgsl/{zr_surface_types,zr_gbuffer_encode}.wgsl` | `ZrSurfaceOutput`/`ZrShadingContext` struct 与 G-buffer 编码骨架 |
| `zircon_runtime/src/graphics/shader/wgsl/zr_shading_{unlit,blinn_phong,standard_pbr}.wgsl` | shading include(`shade_forward`/`encode_gbuffer`/`shade_deferred`) |
| `zircon_runtime/src/graphics/shader/wgsl/zr_template_{forward,gbuffer,depth,shadow,velocity}.wgsl` | entry point 模板保留 `zr_vs_main`/`zr_fs_main` 调试入口,并通过 `zr_vs_main_impl`/`zr_fs_main_impl` 暴露 runtime `vs_main`/`fs_main` 别名,含拼接占位符 |

修改:

| 落点 | 改动 |
|------|------|
| `zircon_runtime/src/graphics/shader/mod.rs` | 删除旧 `ShaderVariantKey { shader_id, domain, keywords }`;改为模块声明 + 再导出(保持瘦) |
| `zircon_runtime/src/core/framework/render/shader/{mod.rs,variant_key.rs}` | `ShaderVariantKey` 定稿(下节);既有 `RenderShaderVariantKey { entry_point, stage, defines }` 降级为单模块编译请求载体,由新键展开生成 |
| `zircon_runtime/src/core/framework/render/material/{mod.rs,lighting_model.rs,standard_material.rs}` | `RenderMaterialLightingModel::as_token()` → `ShadingModelId` 解析;`StandardMaterialDescriptor.lighting_model` 接入变体维度 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/{mesh_pipeline_cache,ensure_pipeline,ensure_motion_vector_pipeline,new}.rs` | 以 `pipeline_variant_id` 为键重构;删除 `HashMap<String, ShaderModule>` 与 `PipelineKey` 双轨;motion vector 管线并入 `ShaderPassType::Velocity` 变体 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs` | CPU `skin_model_primitive` 默认路径删除,收缩为能力回落档;palette 直传 storage 上传路径 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs` | zshader 校验从"整段 WGSL"改为"surface 函数 + 声明清单";fallback 材质走同一模板路径 |
| `tools/zircon_build.py` | 新增 `--prewarm-shaders` 步骤(见下) |
| `zircon_plugins`(shader_graph 生成器、virtual_geometry 注册点) | MS-M1 同步改产 surface 函数;VG 几何源注册(切片内硬切换) |

### 核心类型与接口

契约层(`core/framework/render`,可序列化、无 wgpu):

```rust
// shader/geometry_source.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometrySourceId(pub u8); // 0=StaticMesh 1=SkinnedMesh 2=MorphedMesh 3=SkinnedMorphed; 4..=15 插件注册

pub struct GeometrySourceDescriptor {
    pub id: GeometrySourceId,
    pub token: String,                      // 稳定名,进磁盘键;如 "static_mesh"
    pub wgsl_include: String,               // 提供 fetch_* 函数族的 include 全文
    pub vertex_attributes: Vec<RenderShaderVertexAttribute>, // 生成 ZrVertexInput
    pub object_bindings: Vec<RenderShaderBindingDescriptor>, // 仅允许 group3 槽内(binding 1..=4)
    pub supports_prev_position: bool,       // false 时 velocity pass 回退 instance prev transform
    pub defines: Vec<RenderShaderDefinitionValue>, // 注入拼接 header,等价 UE ModifyCompilationEnvironment
}

// material/shading_model.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadingModelId(pub u8);          // G-buffer 8bit 编码,上限 256;0=Unlit 1=BlinnPhong 2=StandardPbr;插件自 16 起
pub struct ShadingModelDescriptor {
    pub id: ShadingModelId,
    pub token: String,                      // 对齐 RenderMaterialLightingModel::as_token()
    pub forward_include: String,            // 提供 shade_forward
    pub gbuffer_encode_include: String,     // 提供 encode_gbuffer
    pub deferred_include: String,           // 提供 shade_deferred(解码后着色)
    pub required_channels: GBufferChannelMask, // 超出当前 G-buffer 布局 → 注册时诊断拒绝,非静默
}

// shader/variant_key.rs(定稿)
pub enum ShaderPassType { Forward, GBuffer, DepthPrepass, Shadow, Velocity } // 4bit,余位保留
pub struct ShaderFeatureBits(pub u32);
// bit0 ALPHA_TEST | bit1 RECEIVE_SHADOWS | bit2 DOUBLE_SIDED | bit3 LOD_DITHER_CROSSFADE(计划 10 占用)
// bit4 INSTANCED_PREV_TRANSFORM(计划 06) | bit5..=23 保留 | bit24..=31 插件段
pub struct ShaderVariantKey {
    pub material_shader: ResourceId,        // zshader/graph 产物资源
    pub material_revision: u64,             // resource_revision,热改材质即新键
    pub geometry_source: GeometrySourceId,
    pub shading_model: ShadingModelId,
    pub pass_type: ShaderPassType,
    pub features: ShaderFeatureBits,
    pub quality: ShaderQualityTier,         // Low/Medium/High/Ultra(2bit)
    pub platform_token: String,             // backend + downlevel caps 摘要,只进磁盘哈希
}
impl ShaderVariantKey {
    pub fn packed_dims(&self) -> u64;       // geometry 0..3 | shading 4..11 | pass 12..15 | features 16..47 | quality 48..49;材质维度不打包
    pub fn canonical_string(&self) -> String; // 全字段 + 模板修订号,磁盘哈希输入
}
```

实现层(`graphics/shader/variant_cache`,持 wgpu 对象):

```rust
pub struct ResolvedVariant {
    pub pipeline_variant_id: u32,           // cache interning 分配的稠密索引,写入 MeshDrawCommand(计划 02)
    pub module: wgpu::ShaderModule,
    pub layout: RenderShaderPipelineLayoutDescriptor, // 语义槽声明,重排后 replayer 映射收敛恒等(计划 02)
}
impl ShaderVariantCache {
    pub fn resolve(&mut self, key: &ShaderVariantKey) -> Result<ResolvedVariant, GraphicsError>;
    pub fn miss_report(&self) -> &VariantMissReport; // 帧诊断:本次会话即时编译清单
}
```

归属:`ShaderVariantKey`/两 descriptor 在 framework 契约层;拼接器、cache、注册表实例在 graphics 层并挂在 scene_renderer resources 上,生命周期与 `MeshPipelineCache` 现位置一致。

### GPU 数据布局与 WGSL 约定

bind group 槽位直接引用 index.md §8 第 1 条(group2=material、group3=object/instance),并落到具体 binding:

| group | binding | 资源 | 来源 |
|-------|---------|------|------|
| 0 | 0 | `SceneUniform` | MS-M1 `zr_scene_runtime.wgsl` 引用当前 runtime view/clip uniform |
| 1 | — | light grid / shadow map+sampler / HZB | 计划 05 的 `zr_light_grid.wgsl`/`zr_shadow.wgsl` 槽位;shadow 采样按计划 03 GS-M2 重排落 group1 |
| 2 | 0 | material uniform | 材质 property uniform(既有 `property_uniform.rs` 路径) |
| 2 | 1..N | 材质纹理/采样器对 | 按材质声明顺序 |
| 3 | 0 | `zr_primitive_data`(storage) | 当前 runtime `zr_gpu_scene.wgsl` primitive SOA/AoS 数据 |
| 3 | 1 | `zr_instance_data`(storage) | 当前 runtime `zr_gpu_scene.wgsl` instance transform 数据,供 `zr_world_from_local(instance_index)` |
| 3 | 2 | `zr_light_data`(storage) | 当前 runtime light 数据 |
| 3 | 3 | `zr_skinned_joint_palette`(uniform) | MS-M1 template 复用当前 runtime skinned palette helper;storage palette 升级归 MS-M2 |
| 3 | 4 | `zr_previous_skinned_joint_palette`(uniform) | 供 velocity/previous pose;storage palette 升级归 MS-M2 |
| 3 | 5 | `zr_visible_instance_remap`(storage) | 当前 runtime indirect/visible-instance remap |
| 3 | 6 | `zr_visible_instance_remap_params`(uniform) | 当前 runtime remap/light-count 参数 |
| 3 | 7 | `zr_morph_deltas`(storage, template reserved) | Morphed 源 target 增量;真实 runtime layout/上传归 MS-M2 |
| 3 | 8 | `zr_morph_weights`(storage, template reserved) | Morphed 源权重;真实 runtime layout/上传归 MS-M2 |
| 3 | 11 | `zr_morph_payloads`(storage) | MS-M2 payload header slot;`GpuInstanceData.morph_payload_slot` 指向每 draw 的 delta/weight base、vertex count 和 target count |

几何源 include 契约 —— 每个 include 必须以固定签名提供下列函数(`ZrVertexInput` 由拼接器按 `vertex_attributes` 生成;沿用本计划"目标架构"既定的 `fetch_*` 命名,计划 15 等下游按此消费):

```wgsl
fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>;      // object space,含形变
fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>; // 上一帧形变位置(计划 06 velocity)
fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32>;
fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32>;
fn fetch_uv0(v: ZrVertexInput) -> vec2<f32>;
fn fetch_uv1(v: ZrVertexInput) -> vec2<f32>;
fn fetch_color(v: ZrVertexInput, instance_index: u32) -> vec4<f32>;
```

- Static:直读顶点属性;`fetch_prev_position` 返回当前位置(prev 差异由 instance prev transform 承担)。
- Skinned:MS-M1 先复用当前 runtime `zr_skinned_joint_matrix(...)` / `zr_previous_skinned_joint_matrix(...)` uniform palette helper,避免与 group3 binding1/2 的 GPUScene instance/light 数据冲突;palette 升 storage buffer 和 per-instance palette offset 归 MS-M2。
- 精确锚点：当前 skinned 模板路径保留 `zr_skinned_joint_matrix(v.joints.x)`，Base mesh source/cache handoff 的消费 owner 为 `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs`。
- Morphed:`fetch_position` 内做 GPU 权重混合;SkinnedMorphed 先 morph 后 skin,与 CPU 路径 `to_morphed_model_primitive` → `skin_model_primitive` 顺序一致以便对拍。MS-M2 已落地 bindings 7/8 的 delta/weight storage、binding 11 的 payload header 和 `GpuInstanceData.morph_payload_slot`；production draw selection now routes payload-backed unmorphed sources to `GEOMETRY_SOURCE_ID_MORPHED_MESH` / `GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH`, while CPU-baked morph fallbacks remain Static/Skinned to avoid double morph. 2026-06-30 product source-selection observability proves direct active morph weights reach the WGPU GPU-morphed source path through `RenderStats`/DiagnosticStore counters, and the follow-up product readbacks compare direct GPU Morphed and skinned GPU SkinnedMorphed output against CPU-baked reference frames. Previous morph weights for velocity are now code-wired through GPUScene successful-submit rolling plus a previous-weight block in the shared morph weight buffer; focused direct and skinned 0.0 -> 1.0 WGPU product readbacks prove nonzero `scene-velocity` output. RenderDoc capture, broader miss=0 acceptance, and full CI still gate final closure.
- 变体位:几何源本身即 `geometry_source` 维度,不再额外占 feature bit。

shading include 接口(`zr_surface_types.wgsl` 定义 struct,材质 surface 函数产出 `ZrSurfaceOutput`):

```wgsl
struct ZrSurfaceOutput {
    base_color: vec4<f32>, normal_ws: vec3<f32>,
    metallic: f32, roughness: f32, occlusion: f32,
    emissive: vec3<f32>, custom0: vec4<f32>,   // 自定义模型扩展通道
}
struct ZrShadingContext { position_ws: vec3<f32>, view_dir_ws: vec3<f32>, ndc: vec4<f32>, instance_index: u32 }
fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32>;   // 消费 zr_light_grid/zr_shadow 查询函数(计划 05)
fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrGBufferEncoded; // 写 8bit shading model id
fn shade_deferred(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32>;
```

拼接顺序(固定):defines header → `zr_scene_runtime.wgsl` → `zr_gpu_scene.wgsl` → `zr_surface_types.wgsl` → 几何源 include → pass 专属 include(Forward:`zr_light_grid`+`zr_shadow`;GBuffer:`zr_gbuffer_encode`)→ shading include → 材质 surface 函数 → entry point 模板。include 按 id 注入一次去重;材质 surface 函数拼接时改名为 `zr_material_surface`,材质源内全局符号若撞 `zr_`/`fetch_`/`shade_` 前缀,naga 解析后诊断拒绝并回落 fallback 材质(报材质 uri)。pass 特化:DepthPrepass/Shadow 只保留 `fetch_position`(ALPHA_TEST 位再保留 `fetch_uv0`+base_color 采样),Velocity 保留 `fetch_position`+`fetch_prev_position` 双投影,均不拼 shading include。

### 帧时序与集成点

入口时序不变(Extract → Prepare → Queue/Sort → Execute):

1. Prepare 早段:`resource_streamer.ensure_shader_source` 产出 surface 函数源 + revision(沿用现 fallback 报告通道)。
2. Prepare 中段(计划 02 的 command build 处,即现 `build_mesh_draws`):每 draw 组装 `ShaderVariantKey`(几何源由 mesh 形变能力判定,shading model 由 `StandardMaterialDescriptor.lighting_model` 解析,pass 由所属 mesh pass 决定,features 由材质/组件旗标合成)→ `ShaderVariantCache::resolve` → `pipeline_variant_id` 写入 `MeshDrawCommand`。命中内存 interning 为 O(1);磁盘命中只做 naga 反序列化校验;全 miss 即时编译并计入 `VariantMissReport`。
3. Queue/Sort:不触碰 —— sort_key 位段归计划 09。
4. Execute:replayer 按语义槽映射绑定;本计划完成 WGSL 槽位重排后映射表收敛为恒等(计划 02 约定)。

硬切换删除项(随对应切片同变更删除,不留双路径):

- `fallback_mesh_shader_source` 中 skinning 拼接段(GS-M2 重排后的 fallback_mesh.wgsl 以模板路径产出)。
- `SkinnedMeshJointPaletteUniform` uniform palette 路径与 256 骨骼上限。
- `build/skinning.rs` 的 CPU 顶点蒙皮默认路径(保留为 storage buffer 能力缺失时的显式回落档,经能力检测进入)。
- `MeshPipelineCache` 的 `shader_modules: HashMap<String, _>` 与 `PipelineKey` 旧键、`motion_vector_mesh_pipelines` 独立表。
- `graphics/shader/mod.rs` 旧 `ShaderVariantKey { keywords }`。
- shader_graph 插件旧"整段 shader 产物"生成路径。

磁盘缓存:根目录默认 `<project>/.zircon-cache/shader_variants/`(`ZR_SHADER_CACHE_DIR` 覆写);布局 `<root>/v<schema_version>/<hash[0..2]>/<hash>.wgsl.zst` + 同名 `.meta`(键 canonical_string、模板修订号、naga/wgpu 版本、创建时间)。`hash = blake3(canonical_string + 全部参与 include 的内容哈希)`,宁可多失效不可错命中(与"风险与回退"口径一致)。并发写:临时文件 + 原子 rename,先到为准;读损坏视为 miss 并删除条目。

prewarm 钩子:`tools/zircon_build.py --prewarm-shaders` 在 stage 完成后,扫描 staged 资产清单中材质引用,生成变体枚举清单(材质 × 适用几何源 × 启用 pass × 默认 quality),调用 runtime 的 headless 入口 `zircon_runtime::dynamic_api::prewarm_shader_variants(manifest, cache_dir)` 离线编译,产物放入 staged payload 的 `ZirconEngine/cache/shader_variants/`,运行时缓存查找链为"运行期目录 → staged 预热目录"。

### 实施切片细化

MS-M1(GeometrySource 与拼接管线;切片期只 `cargo check -p zircon_runtime --lib --locked`):

- 切片 1:新增 `geometry_source.rs` 契约、`graphics/shader/geometry_sources/*` 四内建、`template/*` 拼接器、`zr_geometry_*.wgsl`/`zr_surface_types.wgsl`/entry 模板;`ShaderVariantKey` 结构与内存 interning(供计划 02 的 `pipeline_variant_id`)。完成判据:check 过;拼接产物 static/skinned 双变体均通过 naga 验证(模块内单测)。
- 切片 2:standard material 与 fallback 材质切到模板路径;改 `ensure_pipeline.rs` 消费 `ResolvedVariant`;删除 fallback skinning 拼接段与旧 `ShaderVariantKey`;shader_graph 插件生成器改产 surface 函数。完成判据:check 过;`render_product` 现有场景产物不回归(里程碑测试阶段验证)。

MS-M2(GPU 形变全材质化):

- 切片 1:group3 binding1/2 palette 升 storage(含 prev palette 槽对接 GpuScene)、morph deltas/weights/payload header 上 GPU;`build/skinning.rs` 与 mesh-build morph payload 上传路径改写。完成判据:check 过;>256 骨骼用例与 morph payload slot/indexing 单测就位。
- 切片 2:zshader 自定义材质经模板获得形变;CPU skinning 改能力回落档并加进入日志。完成判据:`render_product` 蒙皮对拍(GPU vs CPU 参考)通过 —— 本切片是"GPU skinning 全材质可用"验收主线。

MS-M3(shading model 注册体系):

- 切片 1:`shading_model.rs` 契约、`shading_models/*` 三内建与注册表、`zr_shading_*.wgsl`、`zr_gbuffer_encode.wgsl` 写 8bit model id、deferred lighting 按 id 分发。完成判据:check 过;id 编解码 roundtrip 单测。
- 切片 2:`lighting_model.rs` token → `ShadingModelId` 解析接入变体键;blinn-phong/unlit 内建材质模板;自定义模型通道溢出诊断。完成判据:三模型 forward/deferred 对拍一致(里程碑测试)。

MS-M4(变体缓存与预热):

- 切片 1:`variant_cache/disk.rs` 磁盘缓存(布局/哈希/并发如上);`mesh_pipeline_cache` 以 `pipeline_variant_id` 重构,motion vector 管线并入 Velocity 变体;`VariantMissReport` 进帧诊断。完成判据:check 过;键稳定性单测(跨进程同键)。
- 切片 2:`variant_cache/prewarm.rs` + `dynamic_api::prewarm_shader_variants` + `zircon_build.py --prewarm-shaders`。完成判据:预热后二次启动同场景 miss 计数为 0。

### 测试与验收清单

单测放各 owner 模块 `#[cfg(test)]`(命名遵循 index.md §8 第 6 条),产物对拍走既有 `render_product_*` 设施:

| 测试函数 | 断言要点 | 位置 |
|---------|---------|------|
| `render_shader_variant_key_packs_dimensions_stably` | `packed_dims`/`canonical_string` 对 golden 值稳定;字段变更必须显式改 golden | `framework/render/shader/variant_key.rs` |
| `render_shader_template_assembles_static_and_skinned_variants` | 同一 surface 函数 × 两几何源产物不同且均过 naga | `graphics/shader/template/assemble.rs` |
| `render_shader_template_dedupes_includes_and_renames_surface` | include 注入恰一次;`zr_material_surface` 改名生效;前缀冲突材质被诊断拒绝 | 同上 |
| `render_shader_geometry_include_provides_fetch_contract` | 四内建源均解析出全部 `fetch_*` 签名(含 `fetch_prev_position`) | `graphics/shader/geometry_sources/mod.rs` |
| `render_shader_variant_cache_hits_disk_after_restart` | 写盘→新建 cache 实例→resolve 零编译;损坏条目按 miss 处理 | `graphics/shader/variant_cache/disk.rs` |
| `render_shader_variant_miss_report_counts_runtime_compiles` | miss 计数与即时编译次数一致 | `graphics/shader/variant_cache/resolve.rs` |
| `render_skinning_storage_palette_exceeds_256_bones` | 300 骨骼 palette 上传与寻址正确 | `scene_renderer/mesh/skinning` 测试模块 |
| `render_material_shading_model_id_roundtrips_gbuffer_encoding` | 0/1/2/自定义 id 编解码 roundtrip;255 上限 | `graphics/material/shading_models/registry.rs` |
| `render_material_custom_model_channel_overflow_reports_diagnostic` | `required_channels` 超布局 → 注册返回诊断而非静默 | 同上 |
| `render_material_lighting_model_token_resolves_shading_id` | `as_token()` 全枚举映射 + 未注册 custom 回落 Pbr 并诊断 | `framework/render/material/lighting_model.rs` |
| `render_product_skinned_custom_material_matches_cpu_reference` | zshader 材质 GPU 蒙皮 vs CPU 参考对拍(验收主线) | render_product 套件 |
| `render_product_three_shading_models_forward_deferred_parity` | 同场景三模型混用,deferred 与 forward+ 产物一致 | render_product 套件 |
| `render_product_morph_gpu_blend_matches_cpu_reference` | GPU morph 混合 vs CPU `to_morphed_model_primitive` 对拍 | render_product 套件 |

里程碑过滤词:`cargo test -p zircon_runtime shader --locked` / `material` / `skinning`;插件接缝按 index.md §7 跑 `zircon_plugins` 受影响包。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`../../_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md`](../../_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md)

## 性能审阅交接

- 2026-07-18 shader 性能交接：mesh每batch/pass重复构造owned variant key/诊断维度，首遇variant在base-scene pass同步disk/zstd/driver compile，prewarm按variant复制WGSL并串行验证写盘，include解析由asset/template/IDE重复全文扫描。Render08联动Render03/17、Runtime04与Editor09交付generation-owned dense variant id、Queued→Ready异步pipeline、content-addressed source-table prewarm及单遍include artifact；见PERF-MVP-355..358与`docs/plans/performance/01/2026-07-18-runtime-core-framework-render-shader-static-review.md`。
- 2026-07-18 material override性能交接：稳定property override当前每mesh frame重建payload，同entity多primitive重复clone/hash，每draw再clone/pad并创建GPU uniform buffer，同时永久绕开static command cache。Render08联动Render03/17按entity+material revision+override generation+layout hash发布唯一prepared payload/signature/GPU handle，primitives/cameras共享且dirty-range更新；见PERF-MVP-359及material root静态证据。
- 2026-07-18 material management性能交接：resource streamer getter为全部material深clonereadiness details，随后多轮构建summary/status/issue、clone/sort全量rows后才分页。Render08联动Render17/Editor09按material generation发布唯一compact rows+indices，changed rows增量更新，full details按selected id懒取；当前未确认产品consumer，接线前必须满足stable poll零重建；见PERF-MVP-360。
- 2026-07-18 built-in post pipeline补充交接：同源9条post entry已直接共享单一shader module/layout；剩余约27条内建post pipelines仍在renderer构造期同步创建。Render08把PERF-MVP-356的typed Queued→Creating→Ready/Error pipeline cache扩展到内建post descriptors，显式source/depth/ABI dedup，F2集合预热、optional按需single-flight并定义ready前fallback；见PERF-MVP-371。
- 2026-07-18 material bind-group性能交接：mesh draw当前为每draw创建custom/standard两套13-entry WGPU bind group，即使1k instance共享同一material generation也创建2k组。Render08须发布material/textures/uniform/layout generation keyed prepared binding pair/allocator，只prepare dirty material；Render02/03消费dense handle。stable create=0、changed≤2/unique generation。见PERF-MVP-384及build-mesh-draws root静态证据。
- 2026-07-18 mesh pipeline stable-hit补充交接：Base/OIT/GBuffer/depth/shadow/velocity/TAA ensure已改为pipeline map命中后在variant/key/source投影前返回，stable WGSL assembly/hash/module-key work=0。Render08后续Queued→Ready实现必须保留该O(1) ready path，并把首次disk/driver工作移出frame线程；见PERF-MVP-355/356及mesh-pipeline-cache静态证据。
- 2026-07-18 driver pipeline cache补充交接：`mesh_pipeline/**` 7类descriptor当前全部`cache: None`；WGSL disk hit仍不代表跨启动driver compile hit。PERF-MVP-356实现须按adapter/device/driver/layout/source generation管理兼容pipeline cache artifact或后台driver lane，warm frame-thread compile=0、失配有界重建。见mesh-pipeline静态证据。
- 2026-07-18 Deferred pipeline补充交接：普通/SSS MRT构造现共享一次include导出、WGSL assembly、shader module与layout，前端2→1；但两条descriptor仍`cache: None`且SSS无需求也同步创建。Render08须把Deferred variants纳入PERF-MVP-356 typed async/driver cache，并按compiled shading-model generation懒建SSS；见PERF-MVP-390及Deferred静态证据。
- 2026-07-18 overlay pipeline补充交接：`ViewportOverlayRenderer::new`当前无条件同步创建line/sky/icon pipelines与相关fallback，即使minimal/headless或sky/icons关闭。Render08联动Render10把三类descriptor纳入typed async/driver cache并按compiled overlay feature generation按需single-flight；minimal未请求pipeline create=0，warm frame-thread compile=0。见PERF-MVP-356/390及overlay静态证据。
- 2026-07-18 particle pipeline hard-cut交接：legacy `ParticleRenderer::new`无条件同步创建depth/overlay/velocity三条pipeline且descriptor cache均为None。FX-M2删除旧目录时，Render08须把billboard color/velocity variants纳入typed queued/driver cache并按compiled particle feature generation懒建；particle-off create=0、warm frame compile=0。见PERF-MVP-396。
- 2026-07-18 base screen-space UI pipeline补充交接：UI renderer构造期同步创建基础pipeline且descriptor cache为None；它是编辑器MVP必需而非可裁剪热点，但仍须进入PERF-MVP-356 typed queued/driver cache，F0/F2 ready前有明确fallback，warm process frame-thread compile=0。见UI root小文件静态证据。
- 2026-07-18 material/shader resident DAG交接：Render08须消费Runtime04的material parent、shader import、texture dependency revision DAG和一次解析的ABI/property artifact，hot reload只失效affected generation；render线程不得重复load asset、解析parent/import或构建诊断集合。本轮stable material texture load、stable valid shader load及ABI成功路径临时Vec已清零，最终changed工作近dirty dependency closure。见PERF-MVP-404。
- 2026-07-18 pipeline validation artifact交接：Render08与Render01让pipeline register/reload对每handle+revision+executor/capability generation只compile/validate一次，发布immutable requirements/executor binding/profile variants；set viewport/profile不得再次clone asset或compile。reload single-flight、失败保留last-good，见PERF-MVP-412/365。
- 2026-07-18 frame material summary交接：context builder已用frame-local unique-material cache把visible mesh的parent lineage解析从每mesh降到每root，但每camera仍重建且subsurface另扫mesh/material表。Render08联动Runtime04按material parent/shader/property revision发布prepared effective material与scene material-usage/subsurface summary，实例只持stable material id；稳定generation lineage/load/mesh scan=0，多camera共享。参考Bevy `RenderMaterialInstances`+`PreparedMaterial` owner原则，见PERF-MVP-414。
- 2026-07-18 renderer feature descriptor交接：`RendererFeatureAsset::feature_name()`/`descriptor()`当前返回owned String/完整descriptor，validation、filter、plugin apply与report重复物化。Render08联动Render01/Plugins01把feature identity、capability、pass/resource ranges编译为PERF-MVP-422 generation artifact；稳定pipeline/reload descriptor clone bytes=0，affected feature变更只重建相关range。
- 2026-07-18 graphics shader全目录补充：preview URI index已从shader×variant降为每批一次，三类assembly的token/hash clone已移动清零。Render08继续按PERF-MVP-356/357/358交付content-addressed module/variant generation、bounded queued compile/prewarm与indexed include DAG；稳定builtin registry/hash/assembly/Naga/disk/driver工作=0，frame线程只读ready/last-good ticket。
- 2026-07-18 graphics material registry补充：builtin shading-model lookup已从两次String分配降为静态token借用，include duplicate判定不再收集全匹配Vec；但plugin descriptor×3 pass仍全扫ready shader并同步load/copy WGSL。Render08按PERF-MVP-358/404消费Runtime04 normalized token→parsed module Arc index，stable lookup/scan/load/clone=0，reload只失效affected model。
- 2026-07-22 shader prewarm CLI补充交接：`src/bin` 40/40审查确认每asset root被resource export、shader recursion、AssetRegistryIndex与material recursion至少多轮遍历；include dependency又按每source重走DAG，material按label/id线性找并深clone source。Render08联动Runtime04/11把PERF-MVP-357/358落到一次bounded inventory、indexed include DAG与content-addressed source table，新增PERF-MVP-448；见`08/failure-2026-07-22-shader-prewarm-multi-scan-dag.md`。
- 2026-07-22 material asset contract补充：`assets/material` 14/14审查确认readiness、standard/shader-aware descriptor与contract validation反复扫描并复制properties/slots/dependencies/errors，parent inheritance按层深clone。Render08按PERF-MVP-516消费Runtime04唯一effective-material generation，以Bevy `PreparedMaterial`、change tick和specialization cache为本地参考；stable frame只读`Arc` payload/binding/key，prepare≤1/changed generation，并继续满足PERF-MVP-359/360/384/404。
- 2026-07-22 shader asset generation补充：`assets/shader` 8/8审查确认property packing首适配最坏O(P²)、variant keys按entry深clone全部defines、readiness/management复制宽report；PERF-MVP-517已止损summary 14→1遍历与stage parse临时String。Render08按PERF-MVP-518把355..358落到唯一compiled generation：deterministic indexed packing、shared define/entry/layout/WGSL、compact counters与lazy detail，stable specialize/management不得重建资产DTO。
