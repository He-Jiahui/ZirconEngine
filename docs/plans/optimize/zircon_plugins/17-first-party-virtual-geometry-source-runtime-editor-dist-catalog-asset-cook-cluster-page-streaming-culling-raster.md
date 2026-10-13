---
title: First-Party Virtual Geometry Source、Runtime、Editor、Dist、Catalog、Asset Cook、Cluster、Page Streaming、Culling、Raster 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins17
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/render_pass_executors.rs
  - zircon_plugins/virtual_geometry/runtime/src/provider.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/page_payload.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/shaders
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources
  - zircon_plugins/virtual_geometry/editor/Cargo.toml
  - zircon_plugins/virtual_geometry/editor/src
  - zircon_plugins/virtual_geometry/dist/Cargo.toml
  - zircon_plugins/virtual_geometry/dist/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_plugins/asset_importers/model/runtime/src/mesh_importer.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry
  - zircon_app/src/bin/zircon_shader_pbr_viewer
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/virtual_geometry_cook
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
tests:
  - zircon_plugins/virtual_geometry/runtime/src/tests.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_renderer_test_promotion_guard.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/tests
  - zircon_plugins/virtual_geometry/editor/src/tests.rs
  - zircon_runtime/src/asset/tests/virtual_geometry_cook.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/virtual_geometry.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
  - docs/plans/zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md
  - docs/plans/zircon_plugins/13/failure-2026-08-13-runtime-package-asset-root-projection.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-15-virtual-geometry-debug-snapshot-project-toml-consumer-drift.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-22-virtual-geometry-cook-generation-policy.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/NaniteBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/Cluster.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/ClusterDAG.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/GraphPartitioner.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/Encode/NaniteEncode.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/Encode/NaniteEncodeGeometryData.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/Encode/NaniteEncodePageAssignment.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/Encode/NaniteEncodeFixup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Rendering/NaniteStreamingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Nanite/NaniteStreamingPageUploader.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteCullRaster.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteShading.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteRayTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteVisualize.cpp
  - dev/bevy/crates/bevy_pbr/src/meshlet
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/GPUResidentDrawer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/GPUResidentContext.cs
  - dev/godot/servers/rendering/renderer_rd/storage_rd/mesh_storage.cpp
  - dev/Fyrox/fyrox-impl/src/scene/base.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/bundle.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 17 · First-Party Virtual Geometry Source、Runtime、Editor、Dist、Catalog、Asset Cook、Cluster、Page Streaming、Culling、Raster 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/virtual_geometry`不是空目录，也不是只有几份接口声明。全包已有237个tracked文本文件、37,539行、1,390,191 bytes和214个`#[test]`；runtime内部定义了page request、resident/pending状态、slot allocator、CPU hierarchy traversal、GPU buffer/readback、indirect segment、所谓hardware raster与visbuffer输出等较大结构。这些类型化状态和拆分后的模块边界可以作为后续重构底座。

但是，从普通Client或Editor的一帧真实画面反向追踪，当前实现仍不是可交付的虚拟几何产品。插件向Render Framework登记的prepare、node/cluster cull、page feedback、visbuffer和debug overlay五个executor都只调用`validate_context(...)`并返回`Ok(())`，没有编码GPU命令。私有`VirtualGeometryGpuResources::new`在production没有构造点，唯一调用位于测试support；大部分所谓render pass因而处于“内部代码存在、产品链不可达”的状态。

即使绕过公共executor直接读内部renderer，语义仍与Nanite/成熟meshlet pipeline有本质差距。两份WGSL中，一份只是把7个seed word和budget复制为10个work item，另一份由单个invocation循环分配slot并更新page table；没有GPU BVH/cluster culling、geometry decode、software raster、visibility resolve或material shading。名为hardware rasterization和visbuffer64的pass只把CPU record打包进storage buffer，不创建render pipeline、不绘制像素。

当前真正进入普通mesh产品路径的能力，是“保留完整原始indexed mesh，再用实验性storage payload替换部分position/normal/tangent”。`assign_virtual_geometry_vertex_ordinals`占用非skinned mesh的`joints.x/y`传递vertex ordinal；shader越界或payload缺失时静默回退原顶点；indirect draw仍使用原mesh的index count/first index。它不具备独立cluster index、page-local material range、压缩属性或无需原网格的virtualized draw，因此不能宣称已经实现虚拟几何栅格链。

Cook同样只完成了inspection-friendly原型。`ZVG0` payload记录node/cluster与triangle start/count，不包含可直接上传和解码的positions、indices、normals、tangents、UV、color、material、codec、fixup或dependency。runtime必须重新访问原`ModelPrimitiveAsset.vertices/indices`，按triangle展开重复顶点并以四个`vec4<f32>`存储每个顶点。页面不是自包含streaming artifact，也没有异步I/O、page store、DDC generation、带宽/VRAM预算或上传几何字节的GPU transcoder。

Editor、Dist与产品装配没有补上这些断点。runtime catalog仅在可选`advanced-render-runtime-plugins`下提供插件；默认Runtime profile不启用它，默认Editor profile只启用Hybrid GI，viewport默认关闭，PBR viewer显式为`None`。editor catalog没有Virtual Geometry依赖，插件editor又指向不存在的`plugins://virtual_geometry/editor/authoring.zui`。NativeDynamic dist只有descriptor/registration metadata，无invoke/state/bridge/lifecycle，不能提供与linked source相同的渲染能力。

Runtime09B/09C/09D、Runtime04、Editor22/32与Plugins01/04/06已经持有renderer、material、streaming、asset、authoring、carrier和product composition的最高优先级问题。本篇不重复累计P0，登记 **0项新增P0、48项P1、12项P2**；本篇唯一拥有Virtual Geometry单包从source、cook artifact、runtime renderer、editor、dist、catalog、App到可见产品证据的纵向闭环。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 冻结事实 |
|---|---:|---|
| manifest | 1 / 68 / 2,445 / 0 | experimental；Client/Editor；desktop三平台；source/library/native carrier声明 |
| dist | 2 / 116 / 4,224 / 2 | descriptor与registration metadata；无可执行渲染协议 |
| editor | 6 / 154 / 5,303 / 1 | view/drawer/template registration；模板物理文件缺失 |
| runtime production | 198 / 14,810 / 539,741 / 41 | state、CPU traversal、GPU resources/readback、pass DTO与两份WGSL |
| runtime `test_sources` | 14 / 17,290 / 655,868 / 115 | 5份接线；9份被promotion guard明确要求保持未接线 |
| runtime `tests` | 16 / 5,101 / 182,610 / 55 | contract与fixture覆盖，不等于产品像素链 |
| 插件包合计 | 237 / 37,539 / 1,390,191 / 214 | 全部tracked文件均为文本；包内working tree clean |
| package fingerprint | `9cecec2bfccd27e76d3d0a66f1314532cb9326e1e3f5294c95db60fba89c9760` | tracked path排序，以小写path、空格与file SHA-256组成LF串，无末尾LF后重算SHA-256 |

源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`，coordinator baseline epoch为333。Virtual Geometry包最近一次提交为`7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`（2026-08-16）。邻接Runtime、Editor和App路径存在其他会话活动，因此`source_recheck_required: true`；本文没有修改production或tests。

### 2.2 静态风险信号

runtime production中显式`TODO/FIXME/unimplemented!/todo!`为0，但“没有TODO”不代表语义完成。扫描得到12个`panic`、166个`expect`、796个`unwrap`，另有27处`create_buffer_init`、612处`Vec::new`、100处`collect::<Vec<_>>`、79处`HashMap`、82处`BTreeMap`、415次`readback`和189次`fallback`命中。大量unwrap来自测试，但per-frame buffer/vector/readback与fallback命名集中在产品关键路径，必须逐项用生命周期和基准证明，而不能靠静态零TODO过关。

### 2.3 测试库存不等于动态证据

5份已接线`test_sources`只有2,001行、27个test；9份刻意未接线文件占15,289行、88个test，名称覆盖args authority、execution stats、GPU、node/cluster execution、prepare render、submission order和unified indirect。`virtual_geometry_renderer_test_promotion_guard.rs`甚至把“这九份保持未接线”写成结构合同。

本轮是E3静态review，没有运行Cargo、GPU、Editor、NativeDynamic、跨平台或像素测试。214个test attribute只是库存；未接线测试不能作为通过数，历史failure记录也不能替代当前BuildSet动态复验。

### 2.4 开放Failure

协调器把本Session置于`resolving_failure`，因为Plugins13仍拥有两份open handoff：compute workload drift与runtime package asset root projection。Runtime04另有debug snapshot project TOML consumer drift和cook generation policy两份开放记录。本文只映射当前证据，不生成`fixed-*`，也不把源码中的局部变化冒充动态验收。

## 3. 当前真实产品链与断点

~~~text
model import
  -> optional typed [virtual_geometry] request in builtin path
  -> glTF / OBJ / model plugin paths call default cook unconditionally
  -> ZVG0 metadata: hierarchy + triangle ranges
  -X no self-contained compressed geometry page artifact
  -> runtime re-reads original vertices/indices and expands triangle vertices

default Client / Editor
  -X default runtime profile does not enable Virtual Geometry
  -X default editor profile enables Hybrid GI only
  -X viewport defaults virtual_geometry=false
  -X PBR viewer publishes virtual_geometry=None

explicit linked runtime plugin
  -> registers five Render Framework passes
  -> every public executor validates context and returns Ok(())
  -X private GPU resources have no production constructor/caller

ordinary mesh render product
  -> full original indexed mesh remains resident
  -> joints.x/y carry experimental vertex ordinal for non-skinned mesh
  -> storage payload may replace position/normal/tangent
  -> missing/out-of-range payload silently falls back to original vertex
  -X no cluster raster / visbuffer pixel / material resolve product chain

NativeDynamic dist / Editor
  -> metadata-only native descriptor
  -X no executable render bridge or state lifecycle
  -> editor registers missing authoring.zui path
  -X no authoring, validation, preview, stats, save or cook workflow
~~~

“拥有237个文件”“能生成buffer”“能登记pass”“普通mesh shader能读取实验payload”和“能在大型产品中虚拟化几何并稳定输出像素”是不同资格层。当前前四项有局部源码事实，第五项没有闭环证据。

## 4. 应保留的底座

| 基础 | 保留理由 | 收敛条件 |
|---|---|---|
| typed page request/residency/slot状态 | pending、resident、eviction和completion已有显式类型 | 改为byte/heap budget、generation、device-loss与async I/O状态机 |
| hierarchy/cluster DTO | node、cluster、draw、page和selection已有独立结构 | identity绑定artifact schema，数据由自包含page codec驱动 |
| prepare/commit式局部阶段 | prepare frame与completion分离有利于事务化 | GPU submission/completion成为唯一authority，失败有rollback/last-good |
| Render Framework feature descriptor | pass ID、dependency和capability入口已存在 | executor必须真正编码工作且由renderer product chain消费输出 |
| importer typed request | builtin import已有Enabled/Disabled与config入口 | 所有importer共享同一policy、generation key和deterministic cook receipt |
| shader module registry integration | 普通mesh pipeline已有VG shader family与permutation | 不再依赖joints偷渡ordinal，改用正式cluster/page binding schema |
| test promotion guard意图 | 作者已区分prototype test source与产品测试 | 逐里程碑接线、删除“保持未接线”合同并进入required matrix |
| debug/readback DTO | 可作为Bring-up与Editor诊断输入 | reader-gated、异步、有预算；不得反向成为执行source truth |

## 5. 参考实现约束

### 5.1 Unreal Nanite

NaniteBuilder把cluster build、graph partition、DAG、page assignment、fixup与geometry encode拆为不同阶段；geometry payload包含量化属性、dense index bitstream、material/decode信息，page header与dependency可用于独立streaming。Runtime streaming manager拥有异步worker、pool、root/non-root page策略、带宽与pending queue，page uploader用compute把页面转码到GPU。Renderer侧CullRaster、Shading、Feedback、RayTracing和Visualize构成消费同一artifact的产品链。

Zircon不必复制Nanite格式或算法，但必须达到同类工程语义：自包含可版本化artifact、独立page store、持久GPU pool、GPU-driven cull、真实raster/visibility/material输出、feedback闭环以及shadow/RT/GI等消费者一致性。只存triangle range再回读原mesh，不属于同一能力类别。

### 5.2 Bevy Meshlet

Bevy本地meshlet模块同时具有`from_mesh` cook、asset/resource manager、persistent buffer、instance/mesh manager、pipeline prepare，以及instance/BVH/cluster cull、hardware/software visibility raster、resolve和material shade nodes。其价值不是证明Bevy等价于Nanite，而是证明较小Rust引擎也会把asset、persistent GPU resource、render graph node和shader消费链闭合。

因此Zircon不能以“Rust/WGPU暂时只能CPU模拟”为理由保留名称超前的pass；功能未完成时应fail-close或显式ExperimentalUnavailable，完成时必须让同一Render Framework executor真正消费资源并产生像素。

### 5.3 Unity Graphics GPUDriven

Unity Graphics镜像中的`GPUResidentDrawer`与`GPUResidentContext`用于GPU-resident instance、BatchRendererGroup、culling和occlusion组织。它不提供Nanite式虚拟几何page codec，不能用来证明Zircon cluster/page设计正确；但它约束instance identity、resident resource、culler和render pipeline owner不能由per-frame临时buffer与debug snapshot拼接替代。

### 5.4 Godot与Fyrox适用性

Godot `mesh_storage.cpp`管理常规surface与LOD index buffers；Fyrox scene/base与renderer bundle管理距离LOD group、mesh resource和render filtering。本地两者都没有Nanite等价的虚拟几何artifact/streamer/raster链。它们用于负向基线：普通LOD与mesh资源应继续作为明确fallback产品，不能被Virtual Geometry capability冒名；也不能把“比Godot/Fyrox多了cluster类型”解释为性能或产品领先。

## 6. P0路由与去重

本篇没有新增P0。以下硬阻断继续由canonical owner持有：

1. Runtime09B拥有GPU Scene、visibility、culling、indirect submission与多view renderer产品链。
2. Runtime09C拥有material/shader/pipeline/PSO与visibility-to-material resolve。
3. Runtime09D拥有render asset streaming、residency、budget、upload与retirement。
4. Runtime04拥有source/import/cook/artifact schema、generation和last-good事务。
5. Editor22/32拥有render诊断与model/mesh/LOD/geometry authoring产品。
6. Plugins01/04/06拥有NativeDynamic ABI、render umbrella、catalog/profile/capability truth。

Plugins17的48项P1描述这些owner在Virtual Geometry纵向链上的具体落点，不得重复提升为新的P0。

## 7. P1：纵向工程差距

### 7.1 Product Composition、Catalog、Carrier 与 Editor

| ID | 差距 | 所需重构 |
|---|---|---|
| NVG-P1-001 | 普通Runtime profile不启用Virtual Geometry | 由project/render profile生成`VirtualGeometryActivationPlan`，required asset缺provider时启动前fail-close |
| NVG-P1-002 | Editor Host feature虽然可链接advanced插件，默认Editor profile只启用Hybrid GI | profile按project asset需求与device capability选择，生成effective feature receipt |
| NVG-P1-003 | viewport默认关闭且PBR viewer显式`None` | 提供一个默认可运行的真实VG场景、camera和render product，不以测试fixture代替 |
| NVG-P1-004 | first-party editor catalog没有VG editor dependency/feature | catalog把runtime artifact reader与editor authoring provider原子装配 |
| NVG-P1-005 | editor指向不存在的`authoring.zui` | 补真实retained UI asset并经package asset-root registration加载；路径缺失必须报typed error |
| NVG-P1-006 | editor仅154行，只有descriptor注册测试 | 建立import/cook config、mesh/cluster/page stats、validation、preview、reimport、undo/save工具 |
| NVG-P1-007 | dist只有metadata，无invoke/state/bridge/lifecycle | 实现等价NativeDynamic render service，或从manifest删除native carrier与不可交付能力 |
| NVG-P1-008 | manifest的Client/Editor、platform和capability未由实际shader/device/carrier计算 | capability必须绑定target、adapter limits、shader set、artifact schema与carrier receipt |

### 7.2 Cook、Asset、Cluster 与 Page Format

| ID | 差距 | 所需重构 |
|---|---|---|
| NVG-P1-009 | builtin importer尊重typed request，glTF/OBJ/model插件却无条件default cook | 所有importer只调用一个policy owner，Disabled绝不生成副产物 |
| NVG-P1-010 | Cook同步执行且无generation cache/single-flight | 建立content+config+toolchain+platform key、异步job、dedup、cancel与last-good |
| NVG-P1-011 | `ZVG0`只保存hierarchy与triangle range | 定义自包含page payload：compressed vertex/index/attribute/material/decode/fixup |
| NVG-P1-012 | runtime必须保留并读取原`ModelPrimitiveAsset` | VG artifact独立可解码；ordinary mesh作为显式fallback artifact而非隐藏依赖 |
| NVG-P1-013 | payload version是私有常量，缺schema/platform/codec compatibility | 发布qualified artifact schema ID、codec version、endianness、target与migration/reject合同 |
| NVG-P1-014 | malformed range被跳过，重复/越界/溢出没有统一admission | cook与load执行有界验证并返回typed diagnostic，任何partial geometry不得进入resident state |
| NVG-P1-015 | `u32` offset溢出饱和为`u32::MAX` | 使用checked arithmetic与显式artifact-size limit，错误绑定mesh/submesh/cluster identity |
| NVG-P1-016 | provider fixture只有空mesh与单page，无法证明可绘制artifact | 建立多材质、多page、多LOD、边界退化与corrupt corpus，并保存deterministic golden digest |

### 7.3 Residency、Streaming、Upload 与 Memory

| ID | 差距 | 所需重构 |
|---|---|---|
| NVG-P1-017 | 没有非resident page字节的store/source/async I/O | 建立page locator、DDC/package source、priority async reader、cancel和retry/backoff |
| NVG-P1-018 | page budget按数量而非CPU/GPU bytes | 分离disk cache、staging、GPU pool、page table与readback byte budget，受memory pressure驱动 |
| NVG-P1-019 | `execute_prepare`每次创建多个buffer和零填充Vec | per-device持久pool/ring/arena，按generation扩容并在GPU completion后退休 |
| NVG-P1-020 | uploader只更新slot/page table，不上传几何page bytes | submission必须包含page bytes staging、decode/transcode、copy和completion fence |
| NVG-P1-021 | WGSL uploader单workgroup单invocation循环 | 根据page/byte workload分块dispatch，定义并验证workgroup、overflow与large-batch策略 |
| NVG-P1-022 | feedback由已准备request ID构造external buffer，未从真实像素/visibility缺页产生 | raster/cull写feedback，异步压缩读取并合并优先级、去重和滞后控制 |
| NVG-P1-023 | eviction与slot reuse缺GPU generation/fence权威 | slot/page table/physical bytes以submission generation原子commit，旧帧引用完成后再复用 |
| NVG-P1-024 | device loss、OOM、resize和multi-adapter没有恢复合同 | per-device owner支持quiesce/recreate/replay root pages与typed degraded/unavailable状态 |

### 7.4 Culling、Raster、Visibility 与 Material

| ID | 差距 | 所需重构 |
|---|---|---|
| NVG-P1-025 | node/cluster traversal和selection主要在CPU完成 | GPU instance/BVH/cluster cull成为产品authority，CPU reference仅作oracle/debug |
| NVG-P1-026 | descriptor固定`[64,1,1]`和dispatch`[1,1,1]` | dispatch由instance/node/cluster workload和adapter limit计算，并记录overflow/indirect args |
| NVG-P1-027 | seed shader只复制word，未做frustum/cone/LOD/occlusion cull | 实现分层traversal、projected error、frustum/cone与HZB occlusion，输出bounded work queues |
| NVG-P1-028 | “hardware rasterization”只写CPU record storage buffer | 创建真实render/mesh pipeline、cluster-local index fetch与depth/visibility pixel output |
| NVG-P1-029 | “visbuffer64”只打包CPU entries | 定义像素级primitive/instance/material identity、clear/resolve与format capability fallback |
| NVG-P1-030 | 没有software raster路径 | 为small triangle/平台限制提供compute raster，并与hardware path共享coverage/depth规则 |
| NVG-P1-031 | 没有visibility-to-material shading/texture derivative路径 | 接入Runtime09C material resolve、attribute decode、gradient与opaque/masked material contract |
| NVG-P1-032 | depth、velocity、shadow、selection、RT/GI消费者没有同代VG输出 | 定义single frame generation与feature-specific views，禁止回读原mesh伪造一致性 |

### 7.5 Runtime State、Correctness、Lifecycle 与 Platform

| ID | 差距 | 所需重构 |
|---|---|---|
| NVG-P1-033 | 五个公共executor仅validate并返回成功 | 未接真实实现前返回Unavailable；接线后executor拥有encoding、submission与typed output receipt |
| NVG-P1-034 | 私有GPU resources只有测试构造点 | renderer product owner按device/session创建、缓存、销毁，插件pass只借用明确lifetime资源 |
| NVG-P1-035 | `runtime_prepare_renderer_outputs`返回default outputs | 输出必须包含同帧page table、visibility、indirect、feedback和diagnostic generation |
| NVG-P1-036 | snapshot把`resident_page_count`同时写入page-table entry count | 从真实page table结构计算独立字段，并加非相等回归与schema invariant |
| NVG-P1-037 | normalized page table反向遍历并静默last-wins去重 | duplicate page/slot返回typed conflict；commit前验证双射和generation |
| NVG-P1-038 | debug snapshot/readback可重建执行事实，authority倒置 | product state由GPU submission/completion维护；debug只读采样，不参与prepare/commit |
| NVG-P1-039 | vertex ordinal占用`joints.x/y`且排除skinned mesh | 建立正式instance/cluster/page binding；明确static、skinned、morph支持矩阵与拒绝路径 |
| NVG-P1-040 | shader payload缺失时静默回退原顶点 | capability启用后artifact/binding不完整必须可观测fail-close；fallback需显式profile policy |

### 7.6 Testing、Evidence、Performance 与 Documentation

| ID | 差距 | 所需重构 |
|---|---|---|
| NVG-P1-041 | 88个重要test被明确保持未接线 | 按里程碑接入required targets，promotion guard改为防止再次降级或遗漏 |
| NVG-P1-042 | tests集中CPU DTO、buffer word与fixture | 增加真实device command、pixel/depth/visibility/material golden与GPU validation |
| NVG-P1-043 | 没有source→cook→package→stream→raster端到端测试 | 使用非空多page产品asset验证冷启动、camera移动、缺页、evict和最终像素 |
| NVG-P1-044 | 没有NativeDynamic与linked carrier parity | 同一artifact/workload比较capability、输出、state、failure与unload receipt |
| NVG-P1-045 | 没有Windows/Linux/macOS adapter qualification | 建立backend/format/limit matrix，unsupported组合不得继续宣称platform支持 |
| NVG-P1-046 | 没有CPU/GPU/frame/VRAM/I/O基线 | 绑定BuildSet、scene、camera path、quality、hardware与统计分位生成benchmark receipt |
| NVG-P1-047 | 没有streaming churn、OOM、device loss、long-run soak | fault injection覆盖I/O错误、corrupt page、pool exhaustion、late completion和恢复 |
| NVG-P1-048 | 名称与文档把buffer prototype表述为Nanite/raster/visbuffer | 在产品门完成前统一标为Experimental，并记录Unavailable/Degraded的精确原因 |

## 8. 开放Failure的实施映射

| Failure | 当前证据 | 本篇映射 | 处理规则 |
|---|---|---|---|
| compute workload drift | source fixture已有局部变化，但未获当前动态验收 | NVG-P1-021、025、026、027、041、042 | Plugins13复验并返回`fixed-*`；本文不代签 |
| runtime package asset root projection | registration report缺resolved physical package root | NVG-P1-004、005、007 | package/distribution owner先修根投影，Editor只消费注册结果 |
| debug snapshot project TOML consumer drift | 历史实现记录不等于当前BuildSet receipt | NVG-P1-035、036、038、043 | Runtime04 owner保持open直到聚焦验证完成 |
| cook generation policy | 外部importer仍无条件default cook | NVG-P1-009、010、013、016 | Runtime04/Plugins importer共同收敛单一policy owner |

## 9. P2：竞争性能力

| ID | 能力 | 工程前置 |
|---|---|---|
| NVG-P2-001 | 面向Zircon workload的压缩cluster/page codec | P1 artifact schema、corpus、decode correctness与随机访问先完成 |
| NVG-P2-002 | GPU two-stage cull与HZB reproject/late occlusion | GPU Scene、多view history、bounded queue与camera-cut correctness先完成 |
| NVG-P2-003 | hardware/software hybrid raster自动分流 | 两条路径像素/depth一致，adapter capability和成本模型可测 |
| NVG-P2-004 | GPU page transcoding与并行decompression | signed artifact、staging budget、bounds validation和device fault isolation先完成 |
| NVG-P2-005 | camera/visibility预测式streaming与prefetch | feedback闭环、I/O telemetry、priority fairness和误预测上限先完成 |
| NVG-P2-006 | multi-view、stereo、shadow共享hierarchy traversal | frame/view identity、view-specific cull和共享work去重正确 |
| NVG-P2-007 | skinned/deformable virtual geometry | stable topology、deformation cache、bounds update和velocity语义先完成 |
| NVG-P2-008 | ray tracing、virtual shadow、GI统一geometry residency | page lifetime与各consumer acceleration/history generation一致 |
| NVG-P2-009 | programmable material与masked/displacement virtual surface | visibility payload、derivative、material binning和fallback政策先完成 |
| NVG-P2-010 | out-of-core与远端page store | signed manifest、content address、cancel/retry、offline cache和trust先完成 |
| NVG-P2-011 | Editor cluster/page/overdraw/residency可视化实验室 | 真实runtime telemetry、capture/replay与undo/save authoring先完成 |
| NVG-P2-012 | 对Unreal/Bevy/普通mesh的竞争性benchmark实验室 | 同场景同画质同硬件、公开workload和统计协议先完成 |

## 10. 目标架构与硬切边界

~~~text
Mesh Source + Import Policy + Cook Config + Toolchain BuildSet
  -> hermetic VirtualGeometryCookJob
  -> VirtualGeometryArtifact
       AssetId / SchemaId / CodecId / BuildSetId / content digest
       cluster DAG + bounds/error + material ranges
       independently decodable root/streaming pages
       fixups/dependencies + ordinary-mesh fallback artifact reference
  -> DDC / package page store / signed manifest
  -> target + adapter capability preflight
  -> VirtualGeometryActivationReceipt
  -> per-device VirtualGeometryScene
       persistent instance/node/cluster/page-table buffers
       byte-budgeted physical page pool + async streamer
       submission/completion generation + device-loss recovery
  -> Render Graph
       GPU instance/BVH/cluster cull + HZB
       page feedback
       hardware/software raster
       pixel visibility buffer
       material/depth/velocity resolve
       shadow / RT / GI / selection consumers
  -> diagnostics/readback/capture as bounded observers

Editor
  -> consumes the same source, cook policy, artifact, runtime scene and telemetry
  -> import/reimport/config/validation/preview/stats/visualization/undo/save
~~~

硬切要求：

1. 删除用`joints.x/y`传vertex ordinal的隐式ABI，所有consumer迁移到版本化binding schema。
2. `ZVG0` triangle-range prototype不得继续冒充streamable artifact；新格式上线后旧格式显式拒绝或离线迁移。
3. 公共Render Framework executor接通前必须报告Unavailable，不能继续用`Ok(())`表示执行成功。
4. Debug snapshot/readback退出source-of-truth路径，只保留有预算的observer。
5. Ordinary mesh fallback是独立artifact和profile policy，不允许shader静默掩盖VG缺失。
6. NativeDynamic无法提供等价能力时删除carrier声明，不保留metadata兼容外壳。

## 11. 分层重构里程碑

### M0 · Truth Freeze与Failure复验

- 冻结本报告finding、fingerprint、开放failure与current source；
- executor无真实work时改为typed Unavailable，capability UI同步显示；
- 接线9份被隔离test source中的当前有效测试，删除反向promotion合同；
- 建立一个非空、多page、多material、可见像素的最小产品fixture。

### M1 · Cook Policy、Artifact Schema 与 Corpus

- 所有importer收敛同一`VirtualGeometryCookRequest` owner；
- 定义versioned/self-contained artifact、checked admission和deterministic digest；
- 实现async/single-flight/generation/last-good cook；
- 建立valid/corrupt/large/degenerate/multi-material corpus。

### M2 · Page Store、Residency 与 GPU Pool

- 建立page locator、DDC/package store与异步I/O；
- 建立CPU staging/GPU page pool/page table/readback分域byte budgets；
- 用持久buffer/ring替代per-frame create/zero-fill；
- completion fence后原子publish/evict/reuse，覆盖device loss和OOM。

### M3 · GPU Cull 与 Feedback

- instance/BVH/cluster traversal迁到GPU，CPU reference作为oracle；
- 实现projected error、frustum/cone、HZB与bounded queue；
- workload-driven direct/indirect dispatch替代固定1组；
- 缺页从真实cull/raster路径产生并异步反馈streamer。

### M4 · Raster、Visibility 与 Material

- 实现真实hardware cluster raster和software raster；
- 定义像素visibility identity、clear/resolve与format fallback；
- 接入material attribute decode/shade、depth和velocity；
- 用GPU pixel golden证明路径不再依赖普通mesh顶点回退。

### M5 · Renderer Product Integration

- 五个Render Framework executor消费同一per-device scene并输出同代receipt；
- 接入multi-view、shadow、selection、RT/GI所需geometry view；
- 删除default renderer outputs、snapshot reconstruction和ordinary mesh隐式authority；
- target/profile/capability只发布实际可执行组合。

### M6 · Editor 与 Distribution

- 接通first-party editor catalog和真实`authoring.zui` package asset；
- 完成import/reimport/config/stats/validation/preview/visualization/undo/save；
- NativeDynamic实现等价bridge/state/lifecycle或从产品声明移除；
- package asset-root projection failure完成owner返回与消费复验。

### M7 · Platform、Fault 与 Scale Qualification

- Windows/Linux/macOS按adapter/backend/limit执行资格矩阵；
- 覆盖corrupt page、I/O timeout、pool exhaustion、late completion、device loss；
- cold/warm camera path、streaming churn、多view和长时soak全部绑定BuildSet；
- 未通过组合保持Unavailable，不以fallback画面报告通过。

### M8 · Performance 与竞争性验收

- 同场景、同画质、同硬件比较ordinary mesh、Zircon VG与参考实现；
- 同时报告CPU build/submit、GPU frame/pass、RSS、VRAM、I/O、page miss与stutter分位；
- correctness/failure/soak/memory gate先于性能gate；
- 只有统计证据支持时才允许“优于当前Unreal”结论。

## 12. 产品资格门

| Gate | 验收内容 |
|---|---|
| VG01 | 所有enabled VG asset都能回溯source、cook config、toolchain、artifact digest和schema |
| VG02 | Disabled request在所有importer均不生成VG artifact |
| VG03 | 相同输入与BuildSet生成byte-identical artifact；配置变化生成新generation |
| VG04 | corrupt/truncated/overflow/duplicate page数据在分配与GPU提交前被typed reject |
| VG05 | streaming page无需读取原mesh即可独立decode并绘制 |
| VG06 | ordinary mesh fallback是显式artifact/policy且能单独禁用 |
| VG07 | page store支持async cancel/retry/priority并受bytes/time/in-flight预算控制 |
| VG08 | CPU staging、GPU pool、page table与readback分别记录容量和pressure |
| VG09 | slot复用只发生在相关GPU submission完成之后 |
| VG10 | device loss/OOM能进入typed degraded/unavailable并恢复root pages |
| VG11 | GPU cull执行真实frustum/cone/error/HZB逻辑，CPU reference差分通过 |
| VG12 | dispatch由workload计算，大场景无固定1-group截断或无界队列 |
| VG13 | page feedback来自真实cull/raster缺页并闭合streaming priority |
| VG14 | hardware raster产生可验证depth/visibility像素 |
| VG15 | software raster与hardware raster在coverage/depth tolerance内一致 |
| VG16 | visibility resolve正确恢复instance/cluster/primitive/material identity |
| VG17 | material、UV、normal/tangent、masked行为来自VG artifact而非原mesh回退 |
| VG18 | depth、velocity、shadow、selection、RT/GI读取同一frame generation |
| VG19 | 五个公共executor均编码或消费实际GPU work，不存在validation-only success |
| VG20 | production具有唯一per-device GPU resource构造、销毁和generation owner |
| VG21 | debug/readback禁用时产品输出与调度不变 |
| VG22 | duplicate page/slot、stale completion与out-of-order反馈有确定拒绝/合并语义 |
| VG23 | static/skinned/morph支持矩阵由capability明确表达，unsupported输入fail-close |
| VG24 | default产品场景在普通Client和Editor viewport均能看到真实VG结果 |
| VG25 | Editor支持配置、reimport、validation、preview、undo、save、reopen闭环 |
| VG26 | editor package asset通过注册root加载，不依赖当前目录或复制到project |
| VG27 | linked与NativeDynamic carrier能力/输出/failure/lifecycle parity，或后者不宣称支持 |
| VG28 | 9份隔离test source完成有效迁移，required test inventory无隐藏测试 |
| VG29 | GPU validation、像素golden、端到端、fault、soak和跨平台结果绑定同一BuildSet |
| VG30 | benchmark绑定公开scene/camera/quality/hardware并报告CPU/GPU/RSS/VRAM/I/O分位 |
| VG31 | 任一fallback/degraded路径在receipt和diagnostic中可见，不能计入full-capability通过 |
| VG32 | 只有VG01-VG31全部通过后才允许产品完成或性能领先声明 |

## 13. 禁止的临时修补

1. 不得继续增加名称为raster/visbuffer/streaming但只写CPU record或metadata的pass。
2. 不得把测试support中的GPU resource构造复制到另一个fixture来伪造production caller。
3. 不得用更大的page count、更多`Vec`或每帧更多buffer替代byte-budgeted persistent pool。
4. 不得把原mesh vertices/indices复制进“page”后仍宣称out-of-core virtual geometry。
5. 不得再占用joint、color、UV等业务vertex channel偷渡内部identity。
6. 不得以静默ordinary mesh fallback掩盖artifact、binding、shader或device capability缺失。
7. 不得用debug snapshot/readback反向驱动产品执行。
8. 不得把未接线test文件、历史通过记录或compile-only结果当成当前GPU产品证据。
9. 不得为满足manifest继续保留无执行能力的NativeDynamic carrier。
10. 不得在同场景、同画质、同硬件和统计协议完成前声称性能优于Unreal。

## 14. 本轮产出边界

本轮只新增review与索引，不修改production、tests、manifest、feature或workflow；不运行Cargo、GPU、Editor、NativeDynamic和跨平台验证。开放Failure保持open，后续实现必须由其canonical owner按M0-M8依赖顺序推进。

| 项目 | 状态 | 证据 |
|---|---|---|
| 插件237文件逐路径库存与关键调用链复核 | review_complete | 37,539行、1,390,191 bytes、214 tests；fingerprint见2.1 |
| Unreal/Bevy/Unity/Godot/Fyrox对照 | review_complete | 只使用本地源码；适用性边界见第5节 |
| 新增severity | review_complete | 0 P0 / 48 P1 / 12 P2 |
| 目标架构、里程碑与资格门 | design_complete | M0-M8、VG01-VG32 |
| Production重构 | pending | 本篇未实施代码修正 |
