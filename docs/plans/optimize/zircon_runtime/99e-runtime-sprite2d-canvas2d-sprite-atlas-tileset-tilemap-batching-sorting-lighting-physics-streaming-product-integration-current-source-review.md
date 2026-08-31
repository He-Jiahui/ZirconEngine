---
title: Runtime Sprite2D、Canvas2D、Sprite Atlas、TileSet、TileMap、Batching、Sorting、Lighting、Physics、Streaming 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime104
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 337
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
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_plugins/tilemap_2d
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas
tests:
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue_summary.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
  - tests/acceptance/render-product-m6a-sprite-default-2d.md
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/68-runtime-sprite2d-canvas2d-sprite-atlas-tileset-tilemap-batching-sorting-lighting-physics-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
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

# Runtime Sprite2D、Canvas2D、Sprite Atlas、TileSet、TileMap、Batching、Sorting、Lighting、Physics、Streaming 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Sprite2D 不是纯占位。`Sprite2dComponent`、`RenderSpriteSnapshot`、Scene extract、2D phase queue、CPU quad/tiled/sliced geometry、WGPU texture draw 和 Sprite stats 已经形成一条真实提交链；TileSet/TileMap 也有 typed TOML importer、`ResourceKind`、marker、cache payload 和 load facade。Atlas validator、TileMap `Partial` capability、Tiled `DiagnosticOnlyAssetImporter` 都是应保留的诚实基础。

但这仍是一条“单类 Sprite 最小渲染路径”，不是工程级 2D 子系统。仓内没有 production `Canvas2d`、`CanvasLayer`、`Camera2d`、`Light2d`、`SpriteMask`、`AnimatedSprite` 或 `TileMapComponent`；`Mesh2dComponent`只进入 component storage/snapshot/test。Sprite 的 material handle 被 extract 保存，却没有被 streamer、batch key、pipeline 或 shader 消费。TileMap 停留在 authoring DTO 与 descriptor，Scene project I/O 又会丢弃 Sprite2D/Mesh2D，并把 TileMap 固定写回 `None`。

本轮纠正 Runtime68 的一项旧结论：Sprite 现在会被加入 `VisibilityInput.renderables`，不再是“完全未进入 visibility”。但该输入只有 entity、stable key、mobility 和 layer mask，没有 Sprite bounds、camera rectangle、spatial index 或 tile chunk bounds，因此它只是 membership/upload planning，不是实际的 2D camera/frustum culling。`RenderSpriteBounds`仍无 production consumer。

本轮不新增 P0。Scene round-trip 数据丢失继续由 Runtime61 拥有，Editor 可见命令/模板与实际 handler、文档缺失继续由 Editor34/Plugins08 拥有；`tilemap_2d`仍标 `Partial`且在产品 profile 中是 optional。若在资格门关闭前把 capability 升为 Complete/required、接入 shipping 产品或宣传“优于 Unreal”，必须重新升级为 P0。本篇以当前源码重新归并为 **0项新增 P0、48项 P1、12项 P2 和44项资格门**，取代 Runtime68 的 currentness，数量不与旧72/16相加。

目标不是复制某个参考引擎，而是形成唯一权威链：`Canvas2dSource/Scene -> versioned 2D assets -> compiled Sprite/Atlas/Animation/TileMap artifacts -> Canvas2dWorldService -> dirty/spatial/sort/batch compilers -> immutable Canvas2dRenderPacket -> Render Graph/GPU Scene -> physics/navigation/occlusion adapters -> product receipts`。

## 2. 审查边界、currentness 与证据

### 2.1 物理冻结

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| Zircon production / contract / product slice | 102 / 17,852 / 16,440 / 662,744 / 88 / 0 | E3 主链逐段读取；E2 owner/caller/zero search | `ee953df7b02e8cf17ee4ac2e8abc7c05622a75b50a91b190a730e1f65852fdab` |
| dedicated tests / acceptance | 27 / 7,425 / 6,940 / 283,193 / 126 / 0 | E2/E3 断言与证据类型分类；未执行 | `f5d8834097494f17459770d5041699d11aca42adaf59d4e781e28ab83d4ec666` |
| Unreal Paper2D | 16 / 7,343 / 5,980 / 247,246 | E3 | `66d4a43a30dc1f80a4c588cf8c3c385b1c3850c55ff54b0fb53fb007eaa75d41` |
| Godot Canvas2D | 16 / 18,113 / 15,426 / 707,843 | E3 | `07ce6fe532ec793a0bb70070d98067445336611f7c39700a64437c749741b0af` |
| Unity Graphics URP 2D | 16 / 4,510 / 3,719 / 186,720 | E3 | `703759613f9015886655f696f6eca09c1d7865e88a99eaeb137dc365d288e254` |
| Bevy Sprite/Material2d/TilemapChunk | 11 / 4,335 / 3,882 / 161,057 | E3 | `5d05b2cfd47efb1db38331beb36d38163cd9d624eefa37dbc78b42c9c6236177` |
| Fyrox Sprite/TileMap | 10 / 8,462 / 7,935 / 320,091 | E3 | `046add8c30c3aa76e802f56b56bb9abf70047fb7a113e756968c27b7a41699fa` |
| combined reference slice | 69 / 42,763 / 36,942 / 1,622,957 | E3 | `abe2cd1d1491f9b7ebf5de678d392fc51dbef11c7ec983550760c39246a1989a` |

fingerprint 算法为：路径排序后，对每个 working-tree 文件计算 SHA-256，再对 UTF-8 `path<TAB>hash<LF>` manifest 计算 SHA-256。冻结对象是2026-08-22共享 working tree，物理 HEAD 为`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator epoch 为337；会话从`08094b9b9e17f6c80372e15c17b01204038b305b`开始，后续两个提交仅涉及 UI input transaction 与 particle coverage，不改变本冻结集结论。

Bevy、Fyrox、Godot 与 Unity Graphics revision 分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal 目录不是独立 Git checkout，只使用上述16个文件与 manifest fingerprint，不伪造 revision。

### 2.2 数据链读取深度

本轮沿`profile/catalog -> Sprite/Mesh2d component -> Scene project I/O -> extract/visibility/phase sort -> CPU geometry -> batch/GPU submit -> stats -> Atlas layout/validation/Editor pack/cache -> TileSet/TileMap importer/cache/load -> tilemap runtime/editor/dist descriptor -> tests/acceptance/product`闭环读取。对 `Canvas2d/Camera2d/Light2d/AnimatedSprite/SpriteMask/TileMapComponent/TileMapChunk`做 production 精确零搜索，对 material、bounds、Mesh2d、operation path、ZUI、resource kind 和 Scene round-trip 做 owner/caller 搜索。

### 2.3 明确未做

本轮是 review-only，没有修改 Rust、Cargo、WGSL、asset、Editor 或插件实现；没有运行 Cargo、WGPU、Editor、RenderDoc、参考引擎、产品场景、pixel golden、physics/navigation 集成、soak 或 benchmark。tooling 按用户要求排除，未来 Rust 工具只能消费 canonical source/artifact，不能成为第二权威。

## 3. 当前可保留基础与旧结论校正

| 项目 | 当前源码裁决 | 后续处理 |
|---|---|---|
| Sprite component/extract | texture、optional material、atlas UV、rect、flip、anchor、custom size、四种 image mode、color、z、alpha mode 真实存在 | 保留字段语义，迁入 versioned Canvas2d component/artifact |
| phase queue | 有 Opaque2d/AlphaMask2d/Transparent2d、render/material queue、稳定 entity tie-break | 保留确定性排序底座，接入真实 camera/sorting/Y-sort 来源 |
| geometry | Stretch/Scale/Tiled/Sliced 数学与 tests 存在 | 保留 oracle；frame path 改为 instance/analytic/prepared artifact |
| WGPU submit | 能建立 pipeline、bind texture、写真实 color target | 保留 RHI 接口；替换固定 pipeline 与 per-batch allocation/pass |
| visibility | Sprite 已进入 renderable membership | 关闭旧“完全未接入”结论；继续补 bounds/camera rect/spatial culling |
| SpriteAtlas validation | dimensions、name、pixel/UV bounds、一致性检查较严格 | 保留 validator；升级为正式 source/derived resource |
| TileSet/TileMap asset | typed TOML、kind/marker/import/cache/load 与 direct dependency 存在 | 保留 pipeline 接线；重做 schema validator 与 compiled artifact |
| plugin truth | runtime capability 为 Partial，Tiled backend DiagnosticOnly | 保留 fail-close；不要用 descriptor 伪装 runtime/editor 完成 |
| Scene internal snapshot | `NodeRecord`/clone/snapshot 能保留 Sprite2D/Mesh2D | 仅内部能力；canonical project I/O 仍必须补齐 |

## 4. 当前实现的关键断裂

### 4.1 Sprite 是 Scene renderable，不是 Canvas2D world

production 精确搜索没有 `Canvas2d`、`Camera2d`、`Light2d`、`AnimatedSprite`、`SpriteMask`或 TileMap component/chunk。通用 Scene camera 与 render layer 可以筛掉部分 Sprite，但没有 Canvas hierarchy、relative Z、Y-sort、clip/mask group、light mask、texture filter/repeat、pixel snap 或 canvas-to-UI composition contract。

`Mesh2dComponent`只有12类 component/storage/record/test 使用，没有 extract、phase、resource prepare 或 renderer consumer。继续向 `Sprite2dComponent`堆字段无法建立 Sprite、Mesh2d、TileMap、2D light 与 UI 共同遵守的 world/view 语义。

### 4.2 Extract 有确定性底座，但生产输入被拍平

`World::collect_render_sprites`每帧遍历整个 Sprite component table，复制 snapshot，再按`(z_order, entity)`排序。`SpriteExtract::from_sprites_and_phase_inputs`虽然接受 render/material queue、depth bias 和 UI Z，构建 `SpritePhaseInput`时却把`camera_order=0`、`sorting_layer=0`、`y_sort=None`固定写死。

packed sort key 会静默 clamp camera、sorting layer、order 与量化 Y lane；entity ordering key 已提供确定性 tie-break，因此本轮不再误报 key collision 会导致随机顺序。真正问题是 authored 值没有权威来源、超界没有 structured diagnostic，而且不同 camera/canvas 的空间语义未进入 key compiler。

### 4.3 Visibility 已接入 membership，但没有 2D culling

`build_visibility_input`现在把 Sprite 的 entity、stable key、mobility 和 layer mask 加入 renderables，这是相对 Runtime68 的实质进展。但 `VisibilityRenderableInput`没有 bounds；`collect_render_sprites`在 camera layer 过滤后仍复制所有候选 Sprite；`RenderSpriteBounds`只有定义/导出，没有 production consumer。

因此 current visibility 只能用于成员集合与 upload planning，不能证明 Sprite camera rectangle/frustum culling，更不能支撑 TileMap chunk culling、static canvas cache 或 large-world 2D streaming。

### 4.4 空 phase fallback 会破坏未来剔除语义

`build_sprite_vertices`先读取目标 phase 的 SpriteIndex；若结果为空，则回退扫描所有 Sprite 并按 alpha mode 重新选择。它无法区分“phase queue 尚未建立”和“queue 已建立但 culling/policy 有意得到空集合”。一旦真实 visibility 返回零对象，fallback 可能重新绘制本应不可见的 Sprite，必须用 explicit queue readiness/generation 替换 empty heuristic。

非有限 color/size 被静默跳过，非有限 atlas/UV 回退 full region；没有 entity、asset generation、reason code 或 remediation。frame path 同时承担 validation、修复和 geometry prepare，错误不能追踪到 authoring source。

### 4.5 Sprite material 字段到 renderer 后断线

Scene snapshot 保留 `sprite.material`，但 `ensure_scene_resources`对 Sprite 只确保 texture；Sprite readiness 也只统计 texture。batch key 只有 texture `ResourceId`，embedded WGSL 只有 texture sample 乘 vertex color，renderer 不读取 prepared material、sampler、shader、queue、mask/light state 或 generation。

这不是“自定义材质暂未丰富”，而是公开字段没有执行语义。任何保存、preview 或测试只验证 handle 被复制，都不能证明 material 生效。

### 4.6 三个 2D phase 共用错误的固定 pipeline

`SpriteRenderer`只有一条 pipeline：所有 stage 都启用 SrcAlpha blending、depth write off、LessEqual；shader 没有 alpha cutoff/discard。结果是 Opaque2d 仍走 blend，AlphaMask2d 没有 mask semantics，Transparent2d 也没有 material/premultiply/sort policy specialization。

默认 Core2d graph 声明 Sprite/PostProcess/UI/Debug，Sprite feature 虽有 Opaque/Mask/Transparent pass，但没有 2D normal/light/shadow/occluder pass；stage selection test 明确排除 Lighting。通用 3D lighting 存在不能替代 2D light layer、normal map、shadow caster 和 blend style contract。

### 4.7 CPU expansion 与提交热路径不可扩展

普通 Sprite 每帧在 CPU 应用 transform 后展开6 vertex；Tiled/Sliced展开重复 quad。单 Sprite 的 slice 上限为1,000，超过时静默截断，现有 test 还把1,000当预期成功结果。没有 compile-time budget、quality fallback 或 receipt。

Core2d Sprite graph path 只合并相邻同 texture 项；每个 batch 每帧 `create_buffer_init`，并为每个 batch `begin_render_pass`一次。Transparent3d mixed legacy path 能在一个 mixed pass 中穿插 mesh/Sprite，但仍为每个 Sprite 创建独立 vertex buffer。不能笼统声称所有路径都 per-batch pass，但两条路径都存在严重 transient allocation/upload churn，且没有 instance/ring/indirect/GPU Scene owner。

### 4.8 Atlas 有严格 DTO，却没有 runtime identity

`SpriteAtlasAsset`有 atlas texture、width/height、padding、entry name/source、pixel rect、UV rect和 source size，validator 也较严格；Editor packer 使用 rectangle-pack，检查 RGBA 长度并生成 PNG/TOML。这些基础应保留。

但仓内没有`ResourceKind::SpriteAtlas`、`ImportedAsset::SpriteAtlas`、marker、facade、loader 或 runtime cache variant。Sprite 保存 texture + inline UV，不保存 atlas asset ID、stable entry ID、page/generation/lease。schema 没有 version、rotation、trim/pivot/border、secondary texture semantic、platform cook identity 或 relocation map。

Editor packer只生成单页并通过同时放大宽高寻找容量；padding 区没有 extrusion/dilation。retained host 又直接 filesystem `read_to_string`、TOML parse 和 process-global `OnceLock<Mutex<...>>`缓存 manifest/resolution；manifest cache key 只有 path，失效依赖显式清理，没有 ResourceAuthority/artifact generation/last-good receipt。

### 4.9 TileSet/TileMap 是 authoring DTO，不是 compiled runtime

`TileSetAsset`只有 tile size、单 image 与`id/name/collider: Option<String>`，importer只 parse，不运行 validator。`TileMapAsset`只有尺寸、四种 projection enum、单 TileSet reference 与 dense layers；`validate_layers`直接计算`width as usize * height as usize`并只检查 cell count，没有 dimension/byte/chunk budget、checked multiplication、opacity finite/range、layer identity、tile ID resolution 或 tileset compatibility。

projection enum 没有 map/local/world math consumer。schema 也缺 stable source/alternative identity、orientation、per-tile transform/material/animation/property、terrain/autotile、typed collider/navigation/occlusion layer、migration/version 与 compiled artifact。cache 保存的是 authoring DTO，不是 target-qualified runtime data。

### 4.10 TileMap plugin 只注册声明

runtime plugin 注册一个含`tilemap/material` asset reference 的 `ComponentTypeDescriptor`和一个 DiagnosticOnly Tiled importer，systems/events为空；仓内没有 typed `TileMapComponent`。native dist 是 stateless、schema version 0，没有 command/event/invoke/save/restore/unload/host-ready。descriptor 能被 catalog/export 测试识别，但不会生成 chunk、渲染、碰撞或 streaming state。

Editor 注册 import/create/open/paint command、menu、toolkit、template 和 inspector descriptor，但 operation path 没有 production handler；descriptor 引用的`authoring.zui`与`tilemap_component.zui`在插件目录不存在。独立 `apply_tilemap_paint` helper 有 bounds/checked index 校验，却只在 unit tests 调用，不验证 tile ID 对 TileSet 的存在性，也没有 transaction/undo/save/reopen/reimport。

### 4.11 Scene project I/O 仍丢失 2D 数据

canonical `World::from_scene_asset`明确写入`sprite_2d: None`与`mesh_2d: None`，忽略 `SceneEntityAsset.tilemap`；`to_scene_asset`又固定写`tilemap: None`。Scene authoring schema本身没有 Sprite2D/Mesh2D字段。内部 `NodeRecord`、snapshot 和 clone 能保留 typed component，不等于项目文件 round-trip 安全。

这是 shipping blocker，但 P0 owner 已在 Runtime61/Editor34，本文不重复计数。Runtime104 必须消费其无损 Scene schema/artifact，不能建立第二套 Sprite/TileMap 文件格式。

## 5. 五套参考实现的工程语义

| 参考 | 本轮验证的结构 | Zircon 应吸收 | 明确不照搬 |
|---|---|---|---|
| Unreal Paper2D | Sprite resource保存 source/baked UV、origin before trim、additional texture、socket、body setup、atlas group和 baked render data；scene proxy按 material/base/additional texture组织 section；TileMap component有 asset/owned copy、runtime edit、resize、bounds、render dirty与 collision rebuild | resource/component/proxy generation、material key、owned-edit transaction、bounds/collision lifecycle | legacy 全量 CPU scene-proxy rebuild 不是性能目标 |
| Godot | CanvasItem有 relative Z、Y-sort、top-level、clip、material、filter/repeat和 light/visibility mask；CanvasLayer有 layer/transform/viewport；TileMapLayer按 dirty cell 分别更新 rendering/physics/navigation/occlusion；TileSet有 source/coord/alternative stable cell与代理迁移 | Canvas domain、dirty frontier、stable tile identity、独立 derived subsystem adapters | 不复制 server RID 布局或所有 quadrant 重建策略 |
| Unity Graphics | Renderer2D按 SortingLayer/light 状态形成 LayerBatch；RenderGraph显式创建 normals/light/shadow/camera-sorting-layer/upscale资源和 pass；Light2D有 layer、mask、cookie、normal/shadow/volume与 culling；PixelPerfect有 offscreen/upscale/crop policy | pass/resource依赖、sorting range batch、2D light culling、pixel-perfect qualification | 当前 `LightBatch.isBatchingSupported=false`，不能把参考本身当性能完成证明 |
| Bevy | Sprite 使用 instance buffer与共享 index buffer；pipeline按 target/MSAA/tonemap key；Material2d有 Opaque/Mask/Blend phase与 specialized pipeline cache；TilemapChunk把 tile data编码GPU image并复用 mesh/material | instance/ring buffer、material specialization、GPU tile data与 change-driven upload | experimental单 chunk、单 tileset image路线不是完整 TileMap schema |
| Fyrox | TileMapData以16x16稀疏 chunk存储，删除空 chunk并支持 bounded iterator；TileMap把 frustum转 cell bounds；TileSet有 typed properties/collider/transform/animation，另有 autotile、brush、effect与 undo-friendly update | sparse chunk、bounded traversal、typed properties/collider、animation/autotile和 effect/transaction boundary | `TileMapData::can_be_saved=false`及已知 collider TODO 不应继承 |

这些参考证明的不是“功能越多越好”，而是 identity、authority、compiled data、dirty propagation、derived subsystem、resource lifetime 与 product evidence 必须闭环。Zircon可以采用更强的 GPU-driven、streaming 和增量编译实现，但不能以“未来会优化”为理由跳过这些边界。

## 6. P1 工程差距

### 6.1 Authority、Scene、asset 与 identity

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime104-P1-01 | 无 per-World `Canvas2d` authority | 建 `Canvas2dWorldService`，持有 canvas/view/component generation、dirty frontier 与 shutdown |
| Runtime104-P1-02 | 无 CanvasLayer/Camera2D/light-mask/clip/pixel policy | 定义 versioned Canvas2d component family及与通用 camera/UI composition合同 |
| Runtime104-P1-03 | Scene project I/O丢弃 Sprite2D/Mesh2D | 消费 Runtime61 的无损 typed component schema与 migration，save/reopen fail-close |
| Runtime104-P1-04 | Scene TileMap reference load被忽略、save固定`None` | 建 Scene TileMap component reference、load/attach/detach/save round-trip |
| Runtime104-P1-05 | SpriteAtlas不是正式 runtime resource | 增加 source kind/marker/import/cache/facade与 derived `SpriteAtlasArtifact` |
| Runtime104-P1-06 | Sprite只存 inline UV，无 atlas entry identity | 使用 atlas asset + stable entry ID + artifact generation + page lease |
| Runtime104-P1-07 | Atlas schema无 version/trim/pivot/border/secondary texture/cook identity | 定义可迁移 source schema和 target-qualified multi-page artifact |
| Runtime104-P1-08 | Editor atlas cache绕过 ResourceAuthority/generation | 通过 canonical resolver加载 manifest/image，提供 last-good/stale/invalidation receipt |
| Runtime104-P1-09 | 无 Sprite animation/flipbook runtime program | 编译 clip/frame timing/event为 `SpriteAnimationProgram`，由 scheduler驱动 |
| Runtime104-P1-10 | `Mesh2dComponent`无 extract/render consumer | 接入 shared 2D material/phase/GPU Scene，或在迁移期 fail-close拒绝 attach |
| Runtime104-P1-11 | TileSet importer无 semantic validation | 校验尺寸、stable tile ID、duplicate、image bounds、typed derived layers与 dependency generation |
| Runtime104-P1-12 | TileMap dense authoring DTO直接作为 runtime cache payload | 编译为 versioned chunk artifact；source DTO不进入 frame/runtime hot path |

### 6.2 Extract、visibility、sort 与 diagnostics

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime104-P1-13 | 每帧遍历整个 Sprite table、复制并预排序 | change-driven extract + immutable snapshot pages；按 view只投影 visible candidate |
| Runtime104-P1-14 | visibility只有 membership/layer，无2D bounds/camera rect | 接入 `Canvas2dSpatialIndex`、world/canvas bounds、camera rectangle与 view mask |
| Runtime104-P1-15 | `RenderSpriteBounds`无 consumer | 由 compiled geometry/animation generation产 bounds，visibility与debug共用 |
| Runtime104-P1-16 | production camera order/sorting layer/Y-sort固定零/None | 从 Canvas/Camera/component编译权威 `Canvas2dSortInput` |
| Runtime104-P1-17 | packed sort lane静默 clamp | compile阶段范围校验，超界产生 source-linked diagnostic，不在 frame path修复 |
| Runtime104-P1-18 | empty phase回退重扫全部 Sprite | 增加 phase queue state/generation；ready-empty必须保持零 draw |
| Runtime104-P1-19 | 非有限 color/size静默跳过 | source/import/attach validation拒绝；runtime mutation返回 typed receipt |
| Runtime104-P1-20 | 非有限 atlas/UV静默回退 full UV | last-good artifact或显式 fallback code，绑定 entity/asset/entry generation |
| Runtime104-P1-21 | 无 component/asset/material/atlas dirty frontier | 统一 dirty reason bitset，增量更新 extract/bounds/sort/batch/GPU data |
| Runtime104-P1-22 | 无 multi-view/canvas cache与 pixel policy | view-qualified packet、camera generation、pixel grid与 history invalidation进入编译键 |

### 6.3 Renderer、material 与 GPU scalability

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime104-P1-23 | Sprite material未被 ensure/load/readiness消费 | `Canvas2dResourceAdapter`解析 prepared material、shader、textures、sampler与 generation |
| Runtime104-P1-24 | batch key只含相邻 texture ID | key包含 pipeline/material/sampler/atlas page/mask/light/canvas/view generation |
| Runtime104-P1-25 | 三个2D phase共用固定 blend/depth pipeline | 建 `Canvas2dPipelineKey`与 cached specialization，phase语义不可伪装 |
| Runtime104-P1-26 | AlphaMask无 cutoff/discard | material artifact提供 cutoff与 shader specialization，加入 reference pixel oracle |
| Runtime104-P1-27 | Opaque仍 blend且 depth write off | 定义真正 opaque、premultiplied/straight blend、depth/write/sort policy |
| Runtime104-P1-28 | Core2d每 batch建 buffer并开 render pass | persistent ring/instance buffer；一个 graph pass内多 batch，记录 upload/pass budget |
| Runtime104-P1-29 | mixed Transparent3d每 Sprite建 vertex buffer | hard cut legacy 2D-in-3D旁路到统一 packet/buffer allocator与 phase submission |
| Runtime104-P1-30 | 每帧 CPU transform后展开所有 quad | quad instance + shared geometry；sliced/tiled使用 analytic shader或 cached mesh artifact |
| Runtime104-P1-31 | slice上限1,000后静默截断 | compile/budget planner给出 reject/degrade/LOD receipt，不允许无声丢图元 |
| Runtime104-P1-32 | 无 Canvas2d GPU Scene/indirect/residency/time统计 | 建 persistent instance/chunk pages、indirect draw、residency与 CPU/GPU ms/upload bytes/alloc统计 |

### 6.4 TileMap execution、derived systems、Editor 与 product

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime104-P1-33 | 无 typed `TileMapComponent` | Scene/ECS持有 artifact handle、layer state、transform、generation与 runtime owner |
| Runtime104-P1-34 | plugin component descriptor没有 storage/system/execution | descriptor绑定真实 component factory、system schedule与 capability receipt |
| Runtime104-P1-35 | 四种 projection只有 enum | 实现 map/local/world transform、cell bounds、pick与负坐标 oracle |
| Runtime104-P1-36 | layer是全图 dense Vec，无 sparse chunk store | 建 stable chunk coordinate/ID、空 chunk reclaim、bounded iterator与 byte budget |
| Runtime104-P1-37 | 无 compiled GPU tile chunk | 生成 material/page grouped instance或 GPU tile data page，按 dirty region更新 |
| Runtime104-P1-38 | 无 dirty cell/chunk/derived frontier | cell mutation传播 rendering/physics/nav/occlusion/streaming独立 dirty reason |
| Runtime104-P1-39 | 无 runtime mutation transaction/receipt | typed set/erase/fill/stamp批事务，支持 undo/redo、replication与 failure rollback |
| Runtime104-P1-40 | string collider无 physics cook/lifecycle | TileSet typed collider layers -> chunk collision artifact -> runtime bridge/rebuild |
| Runtime104-P1-41 | 无 navigation derived data | tile navigation layers增量 cook，绑定 nav generation与 agent/profile |
| Runtime104-P1-42 | 无 occlusion/2D lighting/mask derived data | occluder、normal、light mask编译进 Canvas2dLightingGraph与 chunk packet |
| Runtime104-P1-43 | 无 chunk streaming/residency | camera/prefetch范围、IO/decompress/upload预算、evict/last-good与 cancellation |
| Runtime104-P1-44 | 无 alternative/orientation/property/animation/terrain/autotile | stable tile cell + typed property layers + animation/terrain/autotile compiler |
| Runtime104-P1-45 | native dist stateless且无 command/event/state/unload | 暴露 generation-qualified runtime service、mutation receipt、save/restore/unload |
| Runtime104-P1-46 | Editor命令无 handler且两个 ZUI document缺失 | 在 Editor34/Plugins08 owner下完成真实 operation、document、capability-gated UI闭环 |
| Runtime104-P1-47 | `apply_tilemap_paint`仅测试调用且不解析 TileSet ID | operation handler使用 canonical transaction、TileSet resolution、undo/save/reopen |
| Runtime104-P1-48 | 无首方 Sprite/Atlas/TileMap产品场景与规模证据 | 建 load/render/edit/reload/save-reopen/cull/light/collision/nav/stream首方 qualification |

## 7. P2 完整性与运维差距

| ID | 差距 | 后续处理 |
|---|---|---|
| Runtime104-P2-01 | runtime UI 的 `UiCanvasLayerGroup`易与 Scene Canvas2d混淆 | 文档和命名明确两套 authority，只通过 composition contract交互 |
| Runtime104-P2-02 | sort clamp与 fallback无聚合诊断视图 | 增加按 reason/source/generation分页的 Canvas2d diagnostics |
| Runtime104-P2-03 | Atlas packer无 multi-page/rotation/extrusion/dilation | 正确性门后加入可配置 packing policy与 deterministic artifact |
| Runtime104-P2-04 | Atlas process-global cache无 hit/miss/stale/evict可观测性 | 接入资源 cache stats、budget与 invalidation receipt |
| Runtime104-P2-05 | `Sprite2d`/`Tilemap`大小写和产品命名不统一 | schema/API/display name在 hard cut时统一，避免兼容别名 |
| Runtime104-P2-06 | TileMap缺 editor/runtime debug overlay | 增加 chunk bounds、dirty reason、collision/nav/occlusion与 residency可视化 |
| Runtime104-P2-07 | Tile selection/brush preview没有与 runtime projection共用 oracle | Editor只调用 canonical transform/compiled preview，不复制数学 |
| Runtime104-P2-08 | Sprite stats无 timing/upload/allocation/cull原因 | 扩展 stats并绑定 frame/view/source fingerprint |
| Runtime104-P2-09 | visual artifact没有 source/settings/GPU/driver metadata | pixel/temporal receipt必须可重放和比较 |
| Runtime104-P2-10 | acceptance Markdown是历史命令记录，不绑定本轮源码 | 由 CI/qualification生成 source-bound machine-readable receipt |
| Runtime104-P2-11 | `tilemap_2d` beta展示与 Partial执行成熟度容易混淆 | package/catalog/UI同时显示 capability status与缺失 backend原因 |
| Runtime104-P2-12 | platform compression/filter/cook策略尚未进入2D artifact | 正确性与 identity门关闭后增加 target profile和质量降级矩阵 |

## 8. 目标架构与 owner 边界

| Owner | 唯一职责 | 禁止事项 |
|---|---|---|
| Scene / Runtime61 | 2D component identity、hierarchy、save/load/snapshot/clone | 不保存 renderer transient buffer或 Editor cache path |
| `core::framework::render` | neutral Canvas2d packet、sort/bounds/material/lighting contract | 不持有 WGPU对象或资源加载器 |
| Asset / Runtime64 | source/artifact handle、dependency、generation、lease、reload | 不让 Editor filesystem cache成为 runtime authority |
| Graphics / Runtime09* | prepared material、GPU Scene、Render Graph、buffer/pass/resource lifetime | 不解释 authoring DTO或修复非法 source |
| `Canvas2dWorldService` | per-World dirty/spatial/sort/batch/animation/chunk execution | 不形成第二 Scene或全局 process singleton |
| Physics/Navigation | 消费 generation-qualified TileMap derived artifact | 不直接解析 TileMap TOML或 string collider |
| Editor34/Plugins08 | authoring command/document/undo/import/preview/save | preview不得自建弱化 runtime或假成功 handler |
| Product/App | 选择 profile、加载 scene、展示 qualification receipt | 不通过 dynamic JSON/descriptor旁路写最终 render data |

核心数据流：

`Scene/Asset source -> semantic validation/migration -> SpriteAtlasArtifact + SpriteAnimationProgram + TileMapChunkArtifact -> Canvas2dWorldService dirty/spatial update -> Canvas2dSortCompiler -> Canvas2dBatchCompiler -> immutable Canvas2dRenderPacket -> Graphics prepared resources/GPU Scene -> Core2d Render Graph -> frame/product receipts`。

## 9. 依赖顺序

### M0：冻结 truth 与 fail-close

保持 TileMap Partial/optional；禁止新增可见假 handler、Complete/required或 shipping 声明。建立 source fingerprint与 owner表。

### M1：Scene 2D component round-trip

在 Runtime61 owner下完成 Canvas/Sprite/Mesh2d/TileMap typed schema、migration、load/save/snapshot/clone；删除固定 `None`。

### M2：2D asset identity 与 compiler

建立 Atlas/Animation/TileSet/TileMap versioned source、semantic validator、dependency与 target-qualified artifact。

### M3：Canvas2d world 与 projection

建立 per-World service、Canvas/Camera/Layer、projection、pixel policy、dirty frontier与 lifecycle。

### M4：Bounds、spatial 与 visibility

编译 Sprite/animation/chunk bounds，建立 spatial index、camera rect、multi-view candidate与 ready-empty语义。

### M5：Sort 与 phase correctness

接入 camera/sorting/Y-sort权威输入、范围诊断、稳定 tie-break和 Opaque/Mask/Blend oracle。

### M6：Material 与 Sprite GPU packet

接入 prepared material/sampler/pipeline key；改为 instance/ring buffer与单 graph pass多 batch。

### M7：Atlas/animation runtime

完成 stable entry/generation/lease、multi-page artifact、animation scheduler与 bounds/history invalidation。

### M8：TileMap chunk runtime

完成 sparse chunk store、projection、dirty mutation transaction、GPU chunk artifact和 bounded rendering。

### M9：Lighting、physics、navigation、occlusion

按独立 derived generation接入 2D normals/light/shadow/mask、collision、nav和 occlusion。

### M10：Streaming、reload 与故障恢复

加入 chunk/page residency、budget、cancellation、device loss、last-good与 stale receipt。

### M11：Editor 与产品闭环

完成真实 document/operation/undo/import/preview/save/reopen；首方 2D scene消费同一 artifact/runtime。

### M12：资格与竞争性优化

通过正确性、规模、视觉、故障、平台和产品门后，再与 Unreal/Unity/Godot/Fyrox/Bevy做同画质同语义 benchmark；只有证据支持时才能声称性能领先。

## 10. 验收资格门

| Gate | 必须证明 |
|---|---|
| G01 | Canvas/Sprite/Mesh2d/TileMap Scene save/reopen保持 identity、字段与 asset generation |
| G02 | World load/unload attach/detach无跨 world service、chunk或GPU资源泄漏 |
| G03 | SpriteAtlas是正式 resource/artifact，entry handle含 stable ID与 generation |
| G04 | Atlas repack/reload能 relocation或 fail-close，旧 entry不会静默指向错误 UV |
| G05 | Sprite animation clip/frame/event在30/60/144Hz与暂停恢复下符合定义 |
| G06 | Mesh2d attach要么真实渲染，要么显式拒绝，不再只存储无执行 |
| G07 | TileSet duplicate/尺寸/bounds/property/collider语义验证 fail-close |
| G08 | TileMap尺寸乘法、byte/chunk budget、opacity、layer和 tile ID均验证 |
| G09 | 四种 projection的 map/local/world、负坐标、pick与 bounds有 oracle |
| G10 | Sprite extract不随全 Sprite storage在每个 view无条件复制排序 |
| G11 | Sprite bounds进入 visibility，camera rect外对象不进入 draw packet |
| G12 | TileMap只遍历相交 chunk/cell，不扫描全图 dense layer |
| G13 | ready-empty phase产生零 draw，绝不回退全量 Sprite |
| G14 | camera order/sorting layer/Y-sort来自 authored authority并跨帧确定 |
| G15 | sort超界返回 source-linked diagnostic，不静默 clamp |
| G16 | 非有限 color/size/UV在 import/attach/mutation阶段显式失败 |
| G17 | Sprite material被 load/readiness/batch/pipeline/shader实际消费 |
| G18 | Opaque2d reference pixel证明无透明 blend且 depth policy正确 |
| G19 | AlphaMask2d reference pixel证明 cutoff/discard与 material参数正确 |
| G20 | Transparent2d证明 straight/premultiplied policy、排序与 blending正确 |
| G21 | batch key覆盖 material/sampler/pipeline/atlas/light/mask/generation |
| G22 | Core2d Sprite graph pass内复用 persistent/ring buffer，不 per-batch建 pass |
| G23 | mixed 2D/3D路径不再 per Sprite创建临时 vertex buffer |
| G24 | 普通 Sprite使用 shared quad/instance；CPU不逐帧展开全部 transformed vertex |
| G25 | tiled/sliced超过预算有 reject/degrade receipt，不静默截断 |
| G26 | 10k/100k Sprite有 CPU/GPU ms、upload bytes、VRAM、draw/pass曲线 |
| G27 | Canvas2d normals/light/shadow/mask资源由 Render Graph显式拥有 |
| G28 | Light layer/mask/cookie/normal/shadow/occluder有 reference scene |
| G29 | pixel-perfect offscreen/upscale/crop在不同 resolution/aspect有 oracle |
| G30 | TileMap cell mutation只更新受影响 chunk和 derived generations |
| G31 | TileMap batch transaction失败可 rollback，undo/redo/save/reopen一致 |
| G32 | Tile animation/orientation/alternative/property/autotile有 deterministic oracle |
| G33 | collision chunk cook/rebuild与 runtime query在编辑后更新 |
| G34 | navigation chunk generation与 agent/profile绑定并增量更新 |
| G35 | occlusion/lighting derived artifact与 tile generation一致，无 stale混用 |
| G36 | streaming按 camera/prefetch/budget加载与驱逐，取消后不提交 stale page |
| G37 | asset hot reload保留 last-good，失败返回 generation-qualified receipt |
| G38 | device loss/OOM/shader failure能 retire旧 GPU page并恢复或显式降级 |
| G39 | tilemap native runtime有真实 service/command/state/unload，不只 descriptor |
| G40 | Editor import/create/open/paint命令有 handler与存在的 ZUI document |
| G41 | Editor preview、runtime与 shipping使用同一 compiled artifact/projection |
| G42 | 首方产品场景完成 load/render/edit/reload/save-reopen/light/collision/nav |
| G43 | visual/perf artifact绑定 source fingerprint、settings、GPU/driver和 metric |
| G44 | 只有同硬件同画质同语义 benchmark胜出后才允许“优于 Unreal”声明 |

## 11. 测试与 artifact 判定

冻结 production 切片内有88个 inline test attribute，专用 test/acceptance切片有126个。它们覆盖 phase queue与稳定排序、Sprite geometry和 slice、atlas validation/packer/cache、typed component storage、asset parse/load、plugin descriptor/capability、profile source contract以及真实 WGPU framework submit。这些测试证明多个局部机制是真实代码，不应被误报为纯 placeholder。

但 `render_product_sprite`主要断言 ready/fallback/stats与提交成功，不检查 Opaque/Mask/Blend/custom material/atlas的 reference pixel。TileMap测试主要断言 manifest/descriptor与 standalone paint helper，没有 component/runtime render/chunk/cull/physics/nav/streaming/product。`render-product-m6a-sprite-default-2d.md`是历史 baseline，且明确把 atlas、material、Mesh2d和 alpha-mask specialization排除在范围外；它不绑定本轮 source fingerprint。

本轮没有发现当前 source-bound 的 2D product scene、pixel/temporal golden、large-map edit/cull receipt、GPU capture、reload/device-loss恢复、CPU/GPU/VRAM曲线或参考引擎同场景 benchmark。单元测试数量不能替代这些资格证据。

## 12. 完成定义与退出条件

只有当 M0-M12 按依赖顺序实施、44项 gate都有当前 source-bound evidence、Scene round-trip数据丢失被关闭、Sprite material/phase/visibility/GPU packet真实执行、Atlas与TileMap进入唯一 resource/artifact/runtime链、Editor和首方产品使用同一 compiled artifact后，本报告才能把`implementation_status`改为 complete。

在此之前，`tilemap_2d`必须保持 Partial/optional，缺失 handler/document必须 fail-close；不允许用 component field、descriptor、enum、cache variant、test fixture、统计计数、optional WGPU submit或旧 acceptance Markdown替代工程化完成证据。
