---
title: Runtime Sprite2D、Canvas2D、Sprite Atlas、TileSet、TileMap、Batching、Sorting、Lighting、Physics、Streaming 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime68
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/asset/assets/sprite_atlas
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/core/framework/render/sprite
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/sprite.rs
  - zircon_runtime/src/scene/components/render2d
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_plugins/tilemap_2d
  - zircon_plugins/first_party_runtime_catalog/src/tests/generated_manifest.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas
  - zircon_editor/src/core/asset/type_registry/builtin.rs
tests:
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/graphics/tests/render_perf_baseline.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperSprite.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperSprite.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperSpriteAtlas.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/Atlasing/PaperAtlasGenerator.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/SpriteDrawCall.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperRenderSceneProxy.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperRenderSceneProxy.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileSet.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileMap.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperTileMap.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileMapComponent.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperTileMapComponent.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Private/PaperTileMapRenderSceneProxy.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileLayer.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperFlipbook.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperGroupedSpriteComponent.h
  - dev/godot/scene/main/canvas_item.h
  - dev/godot/scene/main/canvas_item.cpp
  - dev/godot/scene/main/canvas_layer.h
  - dev/godot/scene/2d/camera_2d.h
  - dev/godot/scene/2d/sprite_2d.h
  - dev/godot/scene/2d/sprite_2d.cpp
  - dev/godot/scene/2d/animated_sprite_2d.h
  - dev/godot/scene/2d/animated_sprite_2d.cpp
  - dev/godot/scene/2d/tile_map_layer.h
  - dev/godot/scene/2d/tile_map_layer.cpp
  - dev/godot/scene/resources/2d/tile_set.h
  - dev/godot/scene/resources/2d/tile_set.cpp
  - dev/godot/scene/2d/light_2d.h
  - dev/godot/scene/2d/light_occluder_2d.h
  - dev/godot/tests/scene/test_sprite_2d.cpp
  - dev/godot/tests/scene/test_sprite_frames.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Renderer2DData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/Renderer2DRendergraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Passes/Utility/LayerUtility.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Passes/Utility/LightBatch.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Lights/Light2D.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Lights/Light2DCullResult.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Lights/Light2DManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/DrawNormal2DPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/DrawLight2DPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/DrawShadow2DPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/DrawRenderer2DPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/PixelPerfectCameraInternal.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Runtime/Renderer2DTests.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Runtime/Light2DTests.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Runtime/TilemapRenderer2DTests.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Editor/RenderSpriteTests.cs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite/src/texture_slice/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/sprite.wgsl
  - dev/bevy/crates/bevy_sprite_render/src/mesh2d/material.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/tile_orientation.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/tilemap_chunk_material.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/tilemap_chunk_material.wgsl
  - dev/bevy/examples/2d/tilemap_chunk.rs
  - dev/bevy/examples/stress_tests/many_sprites.rs
  - dev/Fyrox/fyrox-impl/src/scene/sprite.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/data.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/tileset.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/update.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/transform.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/tile_collider.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/property.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/effect.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/autotile.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 68 · Runtime Sprite2D、Canvas2D、Sprite Atlas、TileSet、TileMap、Batching、Sorting、Lighting、Physics、Streaming 与 Product Integration 工程化差距

## 1. 结论

Zircon当前已有一条能实际提交像素的Sprite2D路径，不能将它误报为纯占位：`Sprite2dComponent`拥有texture/material、atlas UV、rect、flip、anchor、custom size、Stretch/Scale/Tiled/Sliced、color、z order和alpha mode；World能够提取snapshot并建立2D phase；CPU tessellator覆盖普通、缩放、平铺与九宫格；WGPU renderer能按texture连续段绘制；Sprite统计也能输出ready/fallback、pass、batch、slice、vertex与stage数量。TileSet/TileMap同样已经进入`AssetKind`、typed marker、imported asset、cache payload和typed load facade，Tilemap插件也诚实报告`Partial`并使用`DiagnosticOnlyAssetImporter`拒绝伪装Tiled后端。

但这些局部底座尚未形成工程级2D runtime。场景中没有Canvas2D/CanvasLayer/Camera2D/Light2D/SpriteMask/AnimatedSprite authority；Sprite bounds没有进入可见性，排序输入把camera order、sorting layer和Y-sort固定为零/None；Sprite material handle到renderer后被丢弃，Opaque2d/AlphaMask2d/Transparent2d最终共用一条SrcAlpha、depth-write-off管线；每个batch每帧创建vertex buffer并开启独立render pass，批次键只看texture。SpriteAtlas只是可校验的TOML结构和Editor缓存，不是runtime asset/artifact/generation；`Mesh2dComponent`只进入typed storage，完全没有extract/render消费者。TileMap运行时则停在typed asset和插件descriptor：没有typed component、系统、chunk compiler、renderer、projection math、dirty region、collision/navigation/occlusion、streaming或runtime mutation receipt。

本轮登记 **0项新增P0、72项P1、16项P2和48项验收门禁**。0项新增P0并不表示2D产品可发布，而是因为所有会造成当前项目数据丢失或虚假产品承诺的硬阻断已经由Runtime61和Editor34登记；`client2d`也只把`tilemap_2d`列为optional，插件明确是Partial，没有新的Available/Required假声明需要重复升级。Runtime68拥有运行时执行纵切面，目标架构为`Canvas2dWorldService + Canvas2dResourceAdapter + SpriteAtlasArtifact + SpriteAnimationProgram + Canvas2dSceneExtract + Canvas2dSpatialIndex + Canvas2dSortCompiler + Canvas2dBatchCompiler + Canvas2dGpuScene + Canvas2dRenderPipeline + Canvas2dLightingGraph + TileMapChunkStore + TileMapDerivedArtifact + Canvas2dPhysicsNavigationBridge + Canvas2dStreamingCoordinator + Canvas2dDiagnosticsReceipt`。

本轮只做静态review与计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、产品进程、GPU capture、pixel golden、physics/navigation集成、fault、soak或benchmark，因此不能宣称2D功能已经完成，更不能宣称性能或表现超过Unreal。用户已暂停tooling优化；本篇不新增脚本、生成器或工具迁移任务，未来Rust工具只能消费canonical 2D source/artifact schema，不能形成第二权威。

## 2. 审查边界、规模与currentness

### 2.1 Zircon物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Asset、Scene Component、RenderExtract与2D core contract | 57 | 14,011 | 533,042 |
| Graphics renderer、pipeline、diagnostics与focused tests | 25 | 6,751 | 248,591 |
| Tilemap plugin、catalog/profile、App/Editor product boundary与manifest | 50 | 7,446 | 266,012 |
| 去重合计 | **132** | **28,208** | **1,047,645** |

Zircon冻结集fingerprint为SHA-256 `db7b26bc2c83c386fd3ad4115379275faf9559830747cfc111a261a73a2fe0b3`。算法沿用Runtime67：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。三个分组指纹依次为`e4d6b4cc52836513b9af8c39c0adb52339d42f217aa3365457f9781a3e23a7f6`、`aec4268dde89d3ae2480a0cfda40e20056fbe3ef905a5dbdbc49b8fa1acdad82`与`187e01876619d9cf9f7222f81457880127118f49dbad08e38447b9fc24a5c850`。聚焦Sprite/Atlas/Tilemap目录共有43个Rust test attribute，但多数验证DTO、几何数学、phase/stats和source shape，不等于产品视觉或性能资格。

冻结时8个入选working-tree路径带修改标记：Editor plugin SDK测试、`packed_sort_key.rs`、`phase_queue_summary.rs`、`phase_sort.rs`、framework render root、Scene derived state、World和plugin workspace shape测试。当前结论按共享working copy冻结；实施前必须重验这些路径、全部精确零搜索、profile/catalog和指纹。本报告证明成文时的物理事实，不是绕过current-source复核的永久许可。

### 2.2 参考物理冻结

| 参考 | 文件 | 行 | bytes | 指纹 |
|---|---:|---:|---:|---|
| Unreal Paper2D resource/component/proxy/atlas/tilemap | 16 | 7,343 | 247,246 | `4eac5b4a...9925e` |
| Godot CanvasItem/Sprite/TileMapLayer/TileSet/Light2D与tests | 16 | 18,113 | 707,843 | `82cfe4fe...53b47` |
| Unity Graphics URP 2D RenderGraph/light/sort/pixel/test | 16 | 4,510 | 186,720 | `0303c50e...40dd` |
| Bevy Sprite instance/Material2d/TilemapChunk/examples | 11 | 4,335 | 161,057 | `fc377b53...75bf` |
| Fyrox Sprite/TileMap chunk/resource/property/collider/effect | 10 | 8,462 | 320,091 | `b1d645ab...2410` |
| 合计 | **69** | **42,763** | **1,622,957** | `63f41fa3474bf3fa64c5765d394dbc9a95322f078ced3a32d03509d9745b763c` |

Unreal提供resource/component/scene-proxy、material/additional texture、atlas bake、runtime editable tilemap与collision lifecycle对照；Godot提供CanvasItem语义、dirty-cell/quadrant拆分、runtime tile override、physics/navigation/occlusion对照；Unity Graphics提供SortingLayer batch、Normal/Shadow/Light/Sprite RenderGraph、2D light culling和pixel-perfect对照；Bevy提供instance-rate Sprite buffer、specialized Material2d phase与GPU tile chunk对照；Fyrox提供稀疏chunk数据、bounded iteration、tile property/collider/animation/transform/effect对照。

这些参考不是可整套复制的答案。Paper2D仍有CPU重建和legacy路径；Godot会在某些dirty flag下重建quadrant；Unity当前`LightBatch.isBatchingSupported`明确为false；Bevy TilemapChunk是实验性单chunk material路线；Fyrox `TileMapData::can_be_saved`当前返回false且collider存在TODO。Zircon应吸收职责边界、identity、lifecycle和可验证语义，再以自己的RHI、resource、scene和plugin owner做更强实现，不能把参考引擎的历史限制当性能上限。

### 2.3 本轮拥有与明确不拥有

- Runtime68拥有每World的2D runtime authority、Sprite/Atlas/Animation运行时投影、Canvas2D extract/visibility/sort/batch/render、TileMap chunk execution、2D light/mask、physics/navigation/streaming adapter及产品资格。
- Runtime61继续拥有SceneAsset roundtrip、World clone/snapshot/save/play数据损失；Editor34拥有Sprite/Atlas/TileSet/TileMap source schema、authoring toolkit、Tiled reimport和Editor preview。Runtime68消费其无损artifact，不能复制authoring source或重复登记P0。
- Runtime09A/09B/09C/09D/09E分别拥有RHI/RenderGraph、visibility/GPU Scene、material/shader/pipeline、resource residency和通用lighting/shadow父合同；Runtime68只拥有2D specialization、adapter和qualification。
- Runtime08A/08D拥有通用physics/navigation；Runtime37拥有camera endpoint/director；Runtime62拥有hierarchy/transform/activation/bounds；Runtime64拥有resource handle/load/reload/version lease；Runtime65拥有quality/profile/frame budget。Runtime68不得建立弱化私有版本。
- Runtime42拥有catalog/profile/capability truth；Plugins01拥有package/native ABI/lease；Plugins08拥有首方authoring extension装配；App01拥有产品host。Runtime68定义2D capability与receipt，父owner负责承载。
- `zircon_runtime::ui`里的`UiCanvasLayerGroup`是runtime UI layout术语，不是scene Canvas2D authority；两者必须通过明确composition contract协作，不能因名称相近而合并状态。

## 3. 当前实现的真实能力与断裂

### 3.1 Sprite组件字段较完整，但没有Canvas2D domain

`Sprite2dComponent`和`RenderSpriteSnapshot`已携带texture、optional material、atlas region、rect、flip、anchor、custom size、image mode、color、z order和material alpha mode。这个schema足以证明当前不是“只画固定quad”。但Scene没有Canvas2D/CanvasLayer/Camera2D/Light2D/LightOccluder2D/SpriteMask/AnimatedSprite类型；测试创建Sprite时仍借用`NodeKind::Mesh`并移除MeshRenderer。`Mesh2dComponent`只存在于typed storage、record/snapshot和测试，production没有extract、phase或renderer消费。

因此当前Sprite是挂在通用Scene上的一类renderable record，不是具备canvas hierarchy、camera/view、relative Z、Y-sort、mask/light layer、clip/group和pixel policy的2D world。继续往`Sprite2dComponent`堆字段只会扩大record，不会建立跨Sprite、TileMap、UI与camera的共同执行语义。

### 3.2 Extract与phase具备扩展点，但实际输入被拍平

`World::collect_render_sprites`遍历Sprite storage，检查active和render layer后按`(z_order, entity)`排序并复制snapshot；没有camera frustum、bounds或spatial index。`RenderSpriteBounds`类型存在但无production consumer，visibility input只携带entity/mobility/layer。通用`SpritePhaseInput`已经设计了camera order、sorting layer、Y-sort、depth bias和UI Z，但`FrameExtract::from_sprites_and_phase_inputs`把前三者固定为`0/0/None`。

这意味着当前复杂sort key大部分是“可表达但没有来源”。它不能证明多camera、sorting layer或Y-sort已经工作。更危险的是vertex builder在phase item为空时回退到扫描全部Sprite，无法区分“没有phase queue”和“剔除/策略有意得到空phase”，未来加入真实culling后可能重新绘制本应不可见对象。

### 3.3 CPU几何路径有功能覆盖，但热路径和失败语义不成立

几何构建支持Stretch、Scale Fit/Fill、Tiled与Sliced，并有相应单元测试；这是应保留的数学底座。但它每帧为每个Sprite重新展开顶点并在CPU逐顶点应用transform。Tiled/Sliced以重复quad表达，单Sprite最多`MAX_SPRITE_IMAGE_SLICES = 1000`，达到上限时truncate/停止生成而没有结构化降级receipt；单对象可产生6,000个vertex，仍没有instance/analytic shader、geometry cache或预算策略。

非有限color/size会被静默跳过，非法atlas region会回退full UV，empty phase又回退全量扫描。几何函数因此同时承担输入修复、策略降级和渲染准备，却没有错误分类、source identity或diagnostics。工程化实现需要validate/compile阶段生成可复用artifact，frame path只消费generation-qualified prepared data。

### 3.4 Batch和WGPU提交是可运行原型，不是工程级renderer

`prepared_batches`只按相邻Sprite的`texture_id`合并，保持原顺序；key不含material、sampler、alpha/depth pipeline、atlas page generation、mask/light state。`SpriteRenderer`内嵌的shader仅执行texture乘vertex color；`sprite.material`从component一路复制到snapshot，却从不进入batch key、pipeline specialization、bind group或shader。三个2D phase最终使用同一条SrcAlpha blend、LessEqual、depth-write-off pipeline，AlphaMask没有discard/cutoff，Opaque也没有真实opaque semantics。

更直接的性能阻断是`record`对每个batch调用`device.create_buffer_init`并开启一次`begin_render_pass`，之后只draw一次。不存在persistent/ring/instance/indirect buffer、frame arena、multi-draw或单pass多batch。当前统计能看到batch/sprite/slice/vertex/stage数量，但没有CPU/GPU time、upload bytes、buffer/pass churn、culled count、resident page、tile chunk或light/shadow成本，无法建立“优于参考引擎”的证据。

### 3.5 SpriteAtlas是严格文档结构，但不是runtime资源

`SpriteAtlasAsset`对atlas尺寸、padding、entry name、pixel rect、UV有限性/范围/顺序和pixel-to-UV一致性做了较严格校验，Editor packer/cache也会生成和解析它，应保留这些校验。问题在于它没有`AssetKind::SpriteAtlas`、`ImportedAsset` variant、runtime marker/facade/cache payload、load API或artifact resolver；production runtime ingest不调用该validator。精确usage只落在定义、validation tests和Editor packer/cache。

Sprite组件保存的是texture加inline UV region，没有atlas asset ID、stable entry ID、artifact generation或page lease。Atlas schema也没有rotation、trim、pivot、extrusion/dilate、secondary texture semantic、platform compression/cook或relocation map。Editor重新pack后，runtime既没有明确rebind对象，也没有last-known-good generation与stale handle规则。

### 3.6 TileSet/TileMap进入asset pipeline，但schema和execution均未闭合

`TileSetAsset`只有tile尺寸、单image和`id/name/collider: Option<String>`；没有独立validator。`TileMapAsset`只有width/height、四种projection enum、单tileset和dense layer；validation只比较`tiles.len()`与`width as usize * height as usize`，没有checked multiplication、零尺寸、opacity有限/范围、重复layer、tile ID resolution或projection参数。与Godot/Fyrox相比，缺少stable source/alternative identity、transform flags、material、animation、terrain/autotile、typed property、multi collider/nav/occlusion layer和proxy/migration。

尽管TileSet/TileMap已有AssetKind、ImportedAsset、cache、marker、facade和builtin TOML importer，graphics、scene和Tilemap runtime插件没有production consumer。插件只注册一个含tilemap/material asset reference的`ComponentTypeDescriptor`和一个DiagnosticOnly Tiled importer；declaration的systems/events均为空，dist无state/command/event/unload/ready callback。它没有`TileMapComponent`、chunk store、projection math、renderer、culling、physics/nav/occlusion cook、runtime mutation、replication或streaming。

### 3.7 Product/profile和测试仍只证明局部结构

`client2d`把`tilemap_2d`列入optional plugin，required capabilities不含它；普通Editor/Dev profile也没有形成2D authoring/runtime共同activation plan。runtime catalog metadata row和App source assertion只能证明crate被识别，不证明产品安装provider、加载TileMap、生成chunk、render、碰撞、保存、reload或卸载。插件`Partial`与DiagnosticOnly是正确的truthful fallback，不能在功能未闭环时改成Available来“显示进度”。

43个focused test attribute覆盖atlas validator、asset parse、sprite geometry、phase/stats、typed ECS storage和plugin source contracts。仓内也有真实WGPU framework测试，但没有Opaque/Mask/Blend reference pixel、custom material、atlas relocation、1000-slice degradation、tile chunk edit/cull、2D light/shadow、collision/nav、resource reload、profile activation、multi-camera、GPU capture或规模benchmark。`sprite_stage_selection`还明确把Lighting排除在Sprite stage外。

## 4. 五套参考实现的语义差异

| 参考 | 已验证的工程语义 | Zircon当前差异 | 应吸收/明确拒绝 |
|---|---|---|---|
| Unreal Paper2D | Sprite resource保存source/baked UV、additional textures、pivot、socket、collision、baked render data和atlas group；render section按material/base/additional texture组织并可持有预构建vertex buffer；TileMap component有asset/owned copy、bounds、runtime edit、render state dirty、physics rebuild和version migration | inline UV无atlas identity；material丢失；无persistent section/buffer；TileMap无component/runtime edit/collision lifecycle | 吸收resource/component/proxy generation、material key、owned edit与collision/bounds生命周期；拒绝照搬legacy全量CPU scene-proxy重建 |
| Godot | CanvasItem提供relative Z、Y-sort、light/visibility mask、material、filter/repeat、clip/top-level；TileMapLayer按dirty cell分别更新rendering/physics/navigation/occlusion，按material/Z建CanvasItem，支持runtime TileData override和map/local转换 | 无Canvas2D domain，phase输入被拍平；无dirty chunk与四类derived consumer | 吸收统一Canvas语义、dirty frontier与独立derived adapters；不复制所有quadrant重建策略或server RID布局 |
| Unity Graphics URP 2D | Renderer2D按SortingLayer计算LayerBatch，建立Normal/Shadow/Light/Sprite RenderGraph，Light2D带blend style、sorting layers、cookie、normal distance、shadow/volume和bounds culling；PixelPerfect计算offscreen/upscale/crop | Core2D无normal/light/shadow，Sprite pipeline固定；无sorting-layer batch和pixel policy | 吸收pass/resource依赖、light culling、sorting range和pixel-perfect资格；明确Unity当前LightBatch关闭，不能拿其代码作为性能完成证明 |
| Bevy | Sprite提取后写instance-rate buffer，连续image batch共享bind group，pipeline按target/MSAA/tonemap specialization；Material2d分Opaque/AlphaMask/Transparent phase；TilemapChunk把tile data编码为GPU image并复用mesh | Zircon CPU展开、per-batch buffer/pass；material与Mesh2d断线；TileMap无GPU chunk | 吸收instance buffer、pipeline cache和GPU tile data思路；不把Bevy experimental chunk的单material/单mesh限制当完整TileMap schema |
| Fyrox | Sprite按material batch；TileMapData用稀疏固定chunk、bounded iterator和frustum bounds；TileSet有atlas/freeform/transform/animation page、typed properties/collider layer与effect hook | dense whole-map layer、string collider、无bounded traversal、animation/property/effect | 吸收稀疏chunk、stable handle、typed property/collider/transform/animation；拒绝`can_be_saved=false`与未完成mesh collider等已知限制 |

## 5. 已有P0路由与本轮0新增P0

| 已有硬阻断 | Canonical owner | Runtime68只承担的下游责任 |
|---|---|---|
| Canonical SceneAsset忽略terrain/tilemap/prefab，save固定写None | Runtime61 `RWL-P0-004`，Editor34 P0-1 | 定义可安装的TileMap runtime artifact/component generation；不得在renderer私建save格式 |
| Sprite2D/Mesh2D不能进入canonical project document | Runtime61 `RWL-P0-005`，Editor34 P0-1 | 提供runtime component/artifact contract并验证无损load后可extract；不得扩写clone whitelist掩盖问题 |
| TileMap只有asset/feature slot，没有runtime product | Editor34 P0-2 | 实现chunk store、extract/render、derived adapters和产品receipt |
| Tilemap插件公开不可工作的命令/导入能力、缺真实UI/host装配 | Editor34 P0-3，Plugins08 | Runtime provider保持Partial/Unavailable直到真实backend和activation通过 |
| Sprite/SpriteAtlas非正式asset、packer为孤立Editor cache；TileSet/TileMap schema不具长期稳定性 | Editor34 P0-4/P0-5 | 消费stable source identity与compiled artifacts，提供generation/reload/runtime qualification |

任何实施都必须先满足这些owner的schema和无损持久化门禁。Runtime68不得通过另建`.canvas2d`文件、renderer私有JSON、plugin dynamic payload或Editor cache直读来绕过P0；否则是第二权威，不是解锁。

## 6. 目标架构与责任分解

```text
AuthoringSceneDocument / Sprite+Atlas+TileSet+TileMap source   (Editor34, Runtime61)
                              |
                              v
ResourceAuthority + compiler/artifact generation              (Runtime64, Runtime09D)
  SpriteAtlasArtifact / SpriteAnimationProgram / TileMapDerivedArtifact
                              |
                              v
Canvas2dWorldService                                           (Runtime68, per World/Level)
  component install -> dirty frontier -> spatial index -> per-view scene extract
                              |
             +----------------+----------------+
             |                                 |
             v                                 v
Canvas2dSortCompiler                    TileMapChunkStore
  canvas/layer/z/y/material key           sparse chunks + dirty revisions
             |                                 |
             +----------------+----------------+
                              v
Canvas2dBatchCompiler -> Canvas2dGpuScene -> Canvas2dRenderPipeline
  persistent instance/ring buffers          opaque/mask/blend/normal/light/shadow
                              |
          +-------------------+--------------------+
          v                   v                    v
 Physics/Nav adapter     Streaming/residency   DiagnosticsReceipt
```

关键约束如下：

1. `Canvas2dWorldService`只能是`zircon_runtime::scene`下的per-world authority，不能成为新的root package、全局singleton或与World平行的实体容器。
2. `Canvas2dResourceAdapter`只消费Runtime64的handle/version lease和Runtime09D的residency，不自己加载文件、持有Editor cache或定义另一套URI。
3. source document、compiled artifact、runtime instance必须有不同identity；Atlas entry、Tile definition、TileMap chunk和Canvas item均带generation，reload以prepare/validate/swap/retire完成。
4. sort key必须明确区分稳定绘制语义与可重排区间。Opaque可在同等视觉约束内优化；Transparent必须保持确定顺序；mask/clip/light dependency进入batch key。
5. Sprite、Mesh2D和TileMap共享2D material/pipeline contract，但不强制共享同一几何表示。普通Sprite走instance quad；复杂Mesh2D走prepared mesh；TileMap走chunk GPU data或chunk mesh。
6. collision/navigation/occlusion由typed derived artifact和adapter生成，失败不能阻止last-known-good render，但必须使对应capability/receipt降级；renderer不直接创建physics body。
7. performance资格必须以相同内容、相同分辨率、相同quality、相同backend和捕获工具对比；“draw call更少”不能代替frame time、upload、memory、latency与visual parity。

## 7. P1重构清单

### 7.1 Authority、Sprite资源、Atlas与Animation

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| R2D-P1-001 | 没有per-world Canvas2D authority | 建立`Canvas2dWorldService`，由World lifecycle创建/清空/替换/销毁并输出generation-qualified snapshot |
| R2D-P1-002 | Sprite仍借用`NodeKind::Mesh`语义 | 定义不破坏Scene通用hierarchy的2D participation/category，禁止用删除MeshRenderer来区分Sprite |
| R2D-P1-003 | `Mesh2dComponent`只有storage | 增加prepared mesh、material、bounds、extract、phase和renderer垂直链；无consumer前capability不得宣称可用 |
| R2D-P1-004 | Component缺canvas/layer/view/light/mask语义 | 用typed canvas membership与render policy组件组合，不把所有字段继续塞入Sprite record |
| R2D-P1-005 | Component/snapshot没有artifact generation | texture/material/atlas/animation引用必须带version lease，stale generation在extract fail-closed并有receipt |
| R2D-P1-006 | Sprite material复制后被renderer丢弃 | material identity进入prepared draw、pipeline key、bind group与diagnostics；missing material使用可追踪fallback |
| R2D-P1-007 | SpriteAtlas不是正式runtime asset | 增加AssetKind、ImportedAsset、marker、facade、cache/artifact payload和typed loader，统一唯一owner |
| R2D-P1-008 | Sprite只保存inline UV | 引入stable `SpriteRegionId/AtlasEntryId`及page/generation；inline rect只作为明确的raw texture模式 |
| R2D-P1-009 | Atlas validator不在production ingest | 所有source/import/compile路径共享validator；错误包含asset/entry/field和source revision |
| R2D-P1-010 | Atlas schema缺rotation/trim/pivot/border | artifact明确original size、trim rect、pivot、rotation、nine-slice、extrusion/dilate和pixel/UV convention |
| R2D-P1-011 | Atlas没有secondary texture语义 | 支持normal/mask/emission等命名semantic且每页维度/entry布局一致，material按semantic解析 |
| R2D-P1-012 | Atlas没有platform artifact/cook | recipe包含page size、format、mip、padding、compression、quality与toolchain version，输出deterministic hash |
| R2D-P1-013 | 没有Sprite animation runtime | 建立frame duration/loop/ping-pong/event的`SpriteAnimationProgram`和deterministic evaluator，禁止每帧字符串查找 |
| R2D-P1-014 | Sprite collision/picking无stable source | collision/picking shape引用Sprite/Atlas entry generation，trim/pivot/flip后仍保持相同空间定义 |
| R2D-P1-015 | 非法尺寸/UV静默skip或full-UV回退 | validation与frame fallback分层，所有skip/substitute都有reason、count、entity和asset identity |
| R2D-P1-016 | Atlas重排没有runtime relocation | compile输出entry relocation map；reload按generation重绑并保留last-known-good直到GPU lease退役 |

### 7.2 Canvas、Extract、Visibility与Sorting

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| R2D-P1-017 | Sprite extract全表扫描 | 由component change frontier维护2D render records，frame extract只读取current generation和per-view候选集 |
| R2D-P1-018 | `RenderSpriteBounds`无production consumer | bounds由validated size/anchor/transform生成并接入Runtime62 canonical bounds与Runtime09B visibility |
| R2D-P1-019 | visibility input不含2D bounds | 建立2D spatial index/cull input，支持camera rect、layer mask、large world origin generation和culled receipt |
| R2D-P1-020 | 实际排序只有`z_order/entity` | 定义完整`Canvas2dSortKey`及稳定tie-break，source字段、范围和overflow策略均可诊断 |
| R2D-P1-021 | camera order固定0 | 从Runtime37 view/camera stack投影真实camera order与viewport identity，禁止global implicit camera |
| R2D-P1-022 | sorting layer固定0 | 建立stable sorting-layer ID/catalog/value，rename/reorder不改变serialized identity |
| R2D-P1-023 | Y-sort固定None | 建立parent/group Y-sort origin、axis、relative transform和deterministic float quantization规则 |
| R2D-P1-024 | 没有CanvasLayer hierarchy | 支持layer order、transform、visibility、follow-view、top-level/relative-Z和跨layer composition规则 |
| R2D-P1-025 | 没有per-view visibility/light mask | component、camera与Light2D使用typed mask，filter发生在extract/cull而非shader后丢弃 |
| R2D-P1-026 | 没有2D camera与pixel contract | 在Runtime37下定义orthographic 2D projection、viewport scaling、snap/upscale/crop和history key |
| R2D-P1-027 | 没有clip/mask/group modulation | 建立SpriteMask/clip stack/CanvasGroup及modulate继承，batch key包含clip/mask generation |
| R2D-P1-028 | empty phase会回退全Sprite | 显式区分`QueueUnavailable`与`QueueReadyEmpty`，后者必须绘制0对象并保留cull reason |
| R2D-P1-029 | extract输入错误无结构化结果 | 每帧记录invalid transform/color/size/resource/atlas/material分类，不允许无界逐实体日志 |
| R2D-P1-030 | 三个2D phase只在descriptor层成立 | Opaque/Mask/Blend各自完成queue、pipeline、depth/blend和test，stage summary与实际submit一致 |

### 7.3 Geometry、Batching、Material与GPU提交

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| R2D-P1-031 | 普通Sprite每帧CPU tessellate | 普通quad改为instance data；仅Sliced/复杂mesh生成prepared geometry并按dirty generation缓存 |
| R2D-P1-032 | transform逐CPU vertex展开 | world/view transform进入instance或GPU scene buffer，支持current/previous generation和camera-relative转换 |
| R2D-P1-033 | 1000 slice静默截断 | 用checked budget和typed degradation：reject、lower-detail或analytic tiling，任何截断必须进入receipt |
| R2D-P1-034 | Tiled/Sliced顶点爆炸 | 为repeat/mirror/partial edge提供shader或bounded instance表达，规模复杂度和极限值有测试 |
| R2D-P1-035 | 没有geometry cache/invalidation | cache key覆盖mode、rect、border、size、atlas generation和quality；reload/resize只失效相关记录 |
| R2D-P1-036 | batch key只含texture | key至少覆盖pipeline、material/bind group、sampler、texture page、clip/mask、light mode和target format |
| R2D-P1-037 | 只合并相邻texture | 在视觉等价区间内执行stable sort/partition；transparent顺序不可被texture优化破坏 |
| R2D-P1-038 | material不参与batch/pipeline | 统一Sprite/Mesh2D material specialization，支持unlit/lit/custom shader和property override |
| R2D-P1-039 | sampler与atlas generation不在key | texture view/sampler/residency/page generation均参与binding compatibility，stale binding fail-closed |
| R2D-P1-040 | alpha/depth/mask/light状态不在key | 编译明确render state；不同语义绝不因共享texture误合批 |
| R2D-P1-041 | 每batch每帧创建vertex buffer | 建立persistent/ring/frame arena与容量增长/回收/fence规则，steady state buffer allocation为0 |
| R2D-P1-042 | 每batch开启独立render pass | RenderGraph每phase/target开启受控pass，在pass内提交多个batch并记录pass/batch原因 |
| R2D-P1-043 | 所有phase固定SrcAlpha | Opaque关闭blend，Mask使用cutoff/discard或coverage，Blend按material blend mode；visual golden覆盖 |
| R2D-P1-044 | 所有phasedepth-write关闭 | Opaque/Mask/Blend的depth read/write由pipeline policy决定并验证2D/3D混合场景遮挡 |
| R2D-P1-045 | AlphaMask只有名字 | cutoff进入material schema/pipeline key/shader，NaN/out-of-range在compile拒绝而非运行时猜测 |
| R2D-P1-046 | shader固定texture*color | 接入Material2D shader variant、secondary textures、color space、premultiplied alpha和fallback policy |
| R2D-P1-047 | Mesh2D与Sprite renderer断裂 | 共享material/view/phase合同但各自保留geometry path，统一diagnostics和submission receipt |
| R2D-P1-048 | 无instance/indirect/GPU scene | 先实现instance buffer与retained draw，再以实测阈值决定indirect/MDI；不得为“先进”空建slot |
| R2D-P1-049 | texture/atlas paging无绑定策略 | 与Runtime09D协作建立array/bindless/page table capability fallback，低端backend有可验证路径 |
| R2D-P1-050 | 无GPU/上传/资源成本统计 | 记录CPU extract/build、upload bytes、allocation、pass/draw/batch、GPU timestamp、resident/evict和fallback |

### 7.4 TileSet、TileMap、Chunk、Derived Systems与产品

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| R2D-P1-051 | TileSet/TileMap schema过浅 | 消费Editor34 stable schema/artifact，runtime只持compiled definition/chunk，不持authoring string bag |
| R2D-P1-052 | dimensions使用unchecked乘法 | 使用checked area、file/entry/byte/chunk预算，zero/overflow/oversize均typed error |
| R2D-P1-053 | TileSet没有validator | 验证tile size、source、ID唯一性、atlas bounds、alternatives、materials、properties和derived layer引用 |
| R2D-P1-054 | collider是`Option<String>` | 替换为stable collider layer/shape artifact ID，shape type、one-way、material和transform均typed |
| R2D-P1-055 | TileMap只有dense全图layer | 支持稀疏/无限map、chunk目录、per-layer visibility/opacity/order/material和bounded decode |
| R2D-P1-056 | cell只有`Option<u32>` | cell引用stable source/tile/alternative并编码rotate/flip/transpose、color与runtime override identity |
| R2D-P1-057 | projection enum无执行数学 | 为orthogonal/isometric/staggered/hex定义map/local双向转换、neighbor/polygon和property tests |
| R2D-P1-058 | 没有typed `TileMapComponent` | 安装asset/chunk set、layer policy、generation、bounds与runtime edit authority，纳入World lifecycle |
| R2D-P1-059 | Tiled importer永远DiagnosticOnly | backend未安装时保持拒绝；安装后必须走SourceBroker、stable external dependency与canonical compiler |
| R2D-P1-060 | plugin component只有descriptor | plugin贡献typed component/system/factory lease，reload/unload时quiesce、revoke并清理per-world state |
| R2D-P1-061 | plugin systems/events为空且dist无状态 | activation建立runtime provider实例、ready/failure receipt、event/operation和terminal unload |
| R2D-P1-062 | 没有chunk compile与dirty frontier | `TileMapChunkStore`维护chunk revision、dirty rect、derived artifact generation和bounded rebuild队列 |
| R2D-P1-063 | 没有TileMap renderer/culling/batch | 每view只选择相交chunk，GPU tile data/mesh与material batch可复用并报告empty/culled/resident状态 |
| R2D-P1-064 | 无tile animation/terrain/autotile/runtime override | compiler生成deterministic animation/terrain program；runtime override采用copy-on-write/overlay generation |
| R2D-P1-065 | 无collision compile/physics update | typed collider artifact按dirty chunk增量生成body/shape，适配Runtime08A并保留body-to-cell查询 |
| R2D-P1-066 | 无navigation/occlusion derived data | nav region与2D occluder独立generation/dirty queue，失败单独降级且不污染render generation |
| R2D-P1-067 | 无partition/streaming/residency | chunk与atlas page按camera/World partition预取、pin/evict，missing resident page使用bounded fallback |
| R2D-P1-068 | 无runtime mutation transaction | set/erase/fill/replace以precondition、revision、batch commit、rollback和effect receipt修改chunk |
| R2D-P1-069 | `client2d`只有optional metadata | Runtime42产出2D activation plan，required/optional/provider/quality/backend closure fail-closed |
| R2D-P1-070 | App/Editor无真实2D产品fixture | 建立可打开、save/reload/play的Sprite+Atlas+TileMap场景，覆盖plugin source/native carrier |
| R2D-P1-071 | 测试主要是DTO/source shape | 增加reference pixel、resource reload、chunk edit/cull、physics/nav、profile、unload、fault和cross-backend tests |
| R2D-P1-072 | diagnostics不足以资格化性能 | `Canvas2dDiagnosticsReceipt`连接entity/asset/chunk/view/pipeline generation、预算、降级和capture evidence |

## 8. P2延后项

| ID | 延后项 | 提升条件 |
|---|---|---|
| R2D-P2-001 | 用户自定义sorting policy/plugin comparator | canonical sort fields、确定性和性能门先完成 |
| R2D-P2-002 | Sprite animation blend/cross-fade/event graph | 基础frame program和fixed/update clock语义稳定后 |
| R2D-P2-003 | Deformable/Skinning Sprite与complex polygon sprite mesh | Mesh2D material/geometry contract和GPU buffer lifetime稳定后 |
| R2D-P2-004 | 2D normal/specular/height texture高级材质 | unlit/lit material与secondary semantic先通过跨backend资格 |
| R2D-P2-005 | 多blend-style Light2D与global canvas modulation | 基础light culling、normal、mask和shadow闭环后 |
| R2D-P2-006 | 2D soft shadow、SDF occlusion与volumetric light | hard shadow/occluder generation、预算和降级先完成 |
| R2D-P2-007 | Camera sorting-layer texture与2D screen sampling | RenderGraph resource lifetime和multi-camera排序先稳定 |
| R2D-P2-008 | 高级pixel-perfect/grid snapping/Cinemachine式协作 | 2D camera projection、upscale/crop基础闭环后 |
| R2D-P2-009 | 多viewport/canvas render target与nested canvas | 单viewport per-view identity、history和resource lease先完成 |
| R2D-P2-010 | GPU-driven TileMap compaction/indirect dispatch | chunk CPU/GPU基线证明CPU或draw submission确为瓶颈后 |
| R2D-P2-011 | Terrain/autotile constraint solver与weighted variants | stable TileSet terrain schema、determinism和editor ownership稳定后 |
| R2D-P2-012 | Parallax2D、repeat/mirroring与infinite background | CanvasLayer transform/camera contract先完成 |
| R2D-P2-013 | 2D reflection/refraction、distortion与custom compositor | 2D RenderGraph/material domain和screen texture规则稳定后 |
| R2D-P2-014 | Network authoritative tile mutation/replication | local transaction、stable cell identity和Runtime08E authority先完成 |
| R2D-P2-015 | SaveGame增量TileMap delta/checkpoint | Runtime61/SaveGame participant与chunk revision闭环后 |
| R2D-P2-016 | 平台特化的超大atlas/virtual texture/streaming | 普通page residency、低端fallback和真实内存证据先完成 |

## 9. 分层实施顺序

| 里程碑 | 内容 | 进入条件 | 退出证据 |
|---|---|---|---|
| M0 Owner与schema解锁 | 关闭Runtime61/Editor34五项P0依赖；冻结2D source/artifact/runtime identity | canonical owner确认，无第二格式 | 无损scene roundtrip、artifact manifest、capability truth |
| M1 Sprite资源链 | SpriteAtlas asset/compiler/artifact、stable entry、animation program、generation reload | Runtime64/09D adapter可用 | atlas relocation/reload、last-known-good、typed errors tests |
| M2 Canvas执行链 | Canvas2dWorldService、bounds/spatial、per-view extract、sorting/layer/camera/mask | Runtime62 bounds与Runtime37 view identity可消费 | multi-camera/layer/Y-sort/empty-phase确定性 tests |
| M3 GPU renderer | instance/prepared geometry、material2d、phase pipeline、persistent buffer、single-pass batches | M1/M2 snapshot稳定 | visual golden、steady allocation 0、GPU/CPU capture |
| M4 TileMap runtime | typed component、sparse chunk、projection、dirty rebuild、render/runtime mutation | Editor34 compiled TileSet/TileMap artifact可用 | edit/cull/reload/rollback/generation tests |
| M5 Derived systems | Light2D/mask/normal/shadow、collision/nav/occlusion、streaming/residency | M3/M4 generation和resource lease稳定 | subsystem独立降级、fault/recovery、large map soak |
| M6 Product资格 | client2d/profile/App/Editor/plugin carrier、cook/save/play、cross-backend/perf | 所有前序门通过 | activation/load receipt、reference scene、benchmark/capture matrix |

M3不得抢在M0-M2之前以“先优化性能”为名重写renderer；没有stable resource generation、可见性和sort contract时，任何batch结果都可能在后续语义修正中作废。M5也不得把Light2D、physics或nav逻辑塞进TileMap renderer；它们必须是消费同一chunk generation的独立adapter。

## 10. 验收门禁

### 10.1 Owner、identity与资源门 G01-G08

| Gate | 验收标准 |
|---|---|
| R2D-G01 | Runtime61/Editor34五项P0均有通过证据，Sprite/TileMap Scene load-save-play无字段丢失 |
| R2D-G02 | 2D source、artifact、runtime instance和GPU allocation四类identity不可混用，均有generation |
| R2D-G03 | 全仓只有一个SpriteAtlas、TileSet、TileMap canonical source/compiler owner，无renderer/plugin私有格式 |
| R2D-G04 | Canvas2dWorldService随World create/replace/destroy，无全局跨World状态泄漏 |
| R2D-G05 | Resource handle/version lease由Runtime64提供，2D模块无直接文件I/O或Editor cache读取 |
| R2D-G06 | plugin register/unload/reload具owner lease、quiesce、revoke和terminal receipt |
| R2D-G07 | capability在provider/artifact/backend不满足时为Unavailable/Partial且fail-closed |
| R2D-G08 | 所有schema有version、validation、migration/support window和deterministic artifact hash |

### 10.2 Sprite、Atlas与Animation门 G09-G16

| Gate | 验收标准 |
|---|---|
| R2D-G09 | SpriteAtlas通过typed import/load/cache/cook/runtime resolve，production ingest调用同一validator |
| R2D-G10 | Atlas entry stable ID在repack后通过relocation map解析，stale generation不采样错误区域 |
| R2D-G11 | rotation/trim/pivot/border/extrusion/secondary texture的pixel与UV golden全部通过 |
| R2D-G12 | platform format/mip/padding/quality进入artifact key，clean rebuild hash一致 |
| R2D-G13 | Sprite animation在fixed/update clock下确定，loop/ping-pong/event边界测试完整 |
| R2D-G14 | missing texture/material/atlas entry使用可识别fallback并输出一次性聚合receipt |
| R2D-G15 | atlas hot reload在GPU in-flight frame期间保持旧lease，swap后安全retire |
| R2D-G16 | Sprite collision/picking在trim/pivot/flip/scale后与visual geometry空间一致 |

### 10.3 Canvas、Visibility与Sorting门 G17-G24

| Gate | 验收标准 |
|---|---|
| R2D-G17 | bounds进入canonical scene derived state，camera rect外Sprite不会进入prepared draw |
| R2D-G18 | `QueueReadyEmpty`提交0 draw，不会触发全Sprite fallback |
| R2D-G19 | camera order、sorting layer、relative Z、Y-sort和stable tie-break有组合golden |
| R2D-G20 | transparent排序在run-to-run、thread count和entity storage顺序变化下确定 |
| R2D-G21 | per-view visibility/light mask在多camera、多viewport下无跨view泄漏 |
| R2D-G22 | CanvasLayer transform/follow-view/top-level/visibility与hierarchy生命周期一致 |
| R2D-G23 | clip/mask/group modulation嵌套有明确最大深度/预算/overflow结果 |
| R2D-G24 | pixel-perfect/offscreen/upscale/crop输出在奇偶分辨率和resize下无抖动/NaN |

### 10.4 Geometry、Material与GPU门 G25-G32

| Gate | 验收标准 |
|---|---|
| R2D-G25 | 普通Sprite steady state只更新instance data，不生成重复CPU quad vertices |
| R2D-G26 | steady state每帧Sprite buffer allocation为0；增长、wrap、fence、retire有stress证据 |
| R2D-G27 | 每2D phase/target的render pass数量受图控制，不随batch线性增加 |
| R2D-G28 | batch compatibility key覆盖pipeline/material/sampler/page/mask/light/target并有负向测试 |
| R2D-G29 | Opaque/Mask/Blend visual golden、depth交互和alpha cutoff跨backend通过 |
| R2D-G30 | custom Material2D与secondary textures实际影响像素，missing variant有fallback receipt |
| R2D-G31 | Tiled/Sliced极限内容不会静默截断，预算策略和画面降级可观测 |
| R2D-G32 | CPU/GPU capture能关联view/phase/batch/material/asset generation并报告upload与GPU time |

### 10.5 TileMap、Derived与Streaming门 G33-G40

| Gate | 验收标准 |
|---|---|
| R2D-G33 | sparse/infinite map按chunk bounded decode；尺寸乘法、entry/byte/chunk预算全部checked |
| R2D-G34 | 四类projection的map/local roundtrip、neighbor与cell polygon property tests通过 |
| R2D-G35 | 单cell edit只重建相关chunk/derived records，不全图重建 |
| R2D-G36 | chunk renderer只提交可见且resident chunk，material/animation/orientation像素正确 |
| R2D-G37 | runtime batch mutation具precondition/revision/rollback/effect receipt并可故障注入 |
| R2D-G38 | physics collider按dirty chunk更新，body-to-cell查询与one-way/material语义正确 |
| R2D-G39 | navigation/occlusion失败独立降级，last-known-good render与其他derived generation不受污染 |
| R2D-G40 | camera移动时chunk/atlas page预取、pin/evict有budget和thrash/teleport stress证据 |

### 10.6 Product、测试与性能门 G41-G48

| Gate | 验收标准 |
|---|---|
| R2D-G41 | `client2d` resolved activation plan明确provider、artifact、quality与backend closure |
| R2D-G42 | 第一方2D reference project可从clean clone导入、保存、reload、play、cook并退出 |
| R2D-G43 | source plugin与NativeDynamic carrier在capability、component、render、reload、unload语义parity |
| R2D-G44 | App/Editor只在Runtime readiness receipt后显示功能可用，不以manifest row代替ready |
| R2D-G45 | reference pixel覆盖Sprite mode、atlas、material、phase、mask/light/shadow和TileMap projection |
| R2D-G46 | fault matrix覆盖missing/corrupt asset、reload race、GPU pressure、plugin unload和derived failure |
| R2D-G47 | 大Sprite/Atlas/TileMap场景完成长时soak，无无界内存、buffer/pass churn或generation泄漏 |
| R2D-G48 | 相同内容/分辨率/quality/backend下记录Zircon与参考基线的CPU/GPU time、memory、upload、draw与visual parity；未胜出不得宣称性能超过Unreal |

## 11. 禁止的临时实现

1. 禁止把`SpriteAtlasAsset`继续留作Editor TOML cache，再让runtime按文件路径/entry name临时查UV。
2. 禁止只给`Sprite2dComponent`增加`sorting_layer`、`light_mask`等字段却不建立Canvas authority、catalog、per-view extract和batch semantics。
3. 禁止通过扩大`MAX_SPRITE_IMAGE_SLICES`掩盖Tiled/Sliced顶点爆炸与静默截断。
4. 禁止把`material`加入batch key后仍使用固定shader，并据此宣称Material2D完成。
5. 禁止用每Sprite或每Tile draw、每batch buffer/pass创建完成TileMap“首版”。
6. 禁止让Tilemap plugin直接拥有World、文件加载、physics body、navigation region或GPU resource全生命周期。
7. 禁止用`DiagnosticOnlyAssetImporter`成功字符串、catalog metadata、source assertion或disabled命令证明provider ready。
8. 禁止只保存dense `Vec<Option<u32>>`并用更大上限冒充无限/streaming TileMap。
9. 禁止用raw JSON/string collider/property代替stable typed TileSet definition和derived artifact。
10. 禁止重复登记Runtime61/Editor34 P0、另建renderer私有scene格式或通过clone whitelist局部补字段。
11. 禁止把Unity/Unreal/Godot/Bevy/Fyrox任一实现的局部机制直接当架构结论；必须通过Zircon owner和资格门。
12. 禁止在没有同内容、同quality、同backend和visual parity时用draw call数量宣称性能超过参考引擎。

## 12. 当前状态

本篇静态review完成，implementation仍为pending。已确认的可保留底座是Sprite component/snapshot与四类image mode、phase/sort DTO、CPU几何数学和单元测试、WGPU真实像素提交、Sprite诊断、SpriteAtlas严格validator、TileSet/TileMap typed asset/import/cache/facade，以及Tilemap插件当前诚实的Partial/DiagnosticOnly状态。必须重构的是它们之间缺失的authority、identity、generation、visibility、material、batch、GPU lifetime、chunk、derived system、streaming和产品资格链。

下一步不是立刻扩写Sprite shader或Tilemap plugin，而是先关闭Runtime61/Editor34硬阻断并冻结M0 source/artifact/runtime schema；随后按M1-M6逐层实现。任何层未通过对应gate时，Runtime42和产品UI必须继续显示Partial/Unavailable。本篇不改变MVP baseline优先级，也不授权在MVP完成前实施advanced 2D lighting、SDF shadow、GPU-driven tilemap或virtual texture。
