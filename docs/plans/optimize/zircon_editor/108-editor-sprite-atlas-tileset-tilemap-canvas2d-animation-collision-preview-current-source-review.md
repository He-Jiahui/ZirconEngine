---
title: Editor Sprite、Atlas、TileSet、TileMap、Canvas 2D、Animation、Collision 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor108
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor34
refreshes:
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/sprite_atlas
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/scene/components/render2d
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/core/framework/render/sprite
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas
  - zircon_editor/src/ui/workbench/event/node_kind_from_id.rs
  - zircon_plugins/tilemap_2d
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/Cargo.toml
tests:
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperSprite.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperSpriteAtlas.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperFlipbook.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileSet.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileMap.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileLayer.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileMapComponent.h
  - dev/godot/scene/2d/sprite_2d.h
  - dev/godot/scene/2d/tile_map_layer.h
  - dev/godot/scene/resources/2d/tile_set.h
  - dev/godot/editor/scene/2d/sprite_2d_editor_plugin.h
  - dev/godot/editor/scene/2d/tiles/tile_map_layer_editor.h
  - dev/godot/editor/scene/2d/tiles/tile_set_atlas_source_editor.h
  - dev/Fyrox/fyrox-impl/src/scene/sprite.rs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 60 open
  p2: 12 open
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor34/108 · Sprite、Atlas、TileSet、TileMap、Canvas 2D、Animation、Collision 与 Preview 当前源码复核

## 1. 结论

Zircon 有真实 2D 底座：`Sprite2dComponent` 支持 texture/material、atlas UV、rect、flip、anchor、custom size、Fit/Fill/Tiled/Sliced、tint 与 alpha mode；Sprite extract/phase queue、2D/3D transparent ordering、WGPU sprite renderer、`SpriteAtlasAsset` 几何/UV 校验、确定性 rectangle pack、RGBA/PNG/TOML writer 和 Retained Host atlas image cache 都应保留。

但 Scene 持久化已形成确定性数据丢失。`SceneEntityAsset` 可以保存 `tilemap: Option<SceneTileMapAsset>`，`World::from_scene_asset()` 不读取它，`World::to_scene_asset()` 固定写 `None`；Sprite2D 更早就没有 SceneEntityAsset 字段，load 固定 `sprite_2d = None`，save 无处写回。TileMap load/save 一次即可丢失，Sprite2D 从未进入项目 Scene contract。

TileMap runtime 是数据/名称容器：ResourceKind、ImportedAsset、typed marker/load API 与 `BuiltinRenderFeature::Tilemap` slot 存在，但没有 TileMap component、chunk compiler、renderer、collision、navigation、occlusion、streaming 或 runtime mutation。`tilemap_2d` plugin 只有动态 component descriptor 与 DiagnosticOnly Tiled importer；Editor 五个 operation 没有 factory，两份 declared ZUI 不存在，`apply_tilemap_paint()` 不是 transaction。默认 catalogs/App 也没有 TileMap feature。

Atlas 只被 Editor UI cache 消费。它没有 Sprite/SpriteAtlas/Flipbook ResourceKind/ImportedAsset、stable entry ID 或 Sprite component atlas handle；packer 无 production caller，输出固定 cache path，先写 PNG 再写 TOML，缺 source digest/recipe/generation/DDC/platform variant/atomic multi-file publish。Sprite renderer 的 material 不进 batch key/shader，所有 phase 共用固定 alpha blend/depth-off pipeline；只合并相邻同纹理项，每 batch 每帧建 vertex buffer/render pass。

TileSet 只有单 image、tile size、id/name/collider string；TileMap 只有单 TileSet、projection、width/height 与 dense `Vec<Option<u32>>` layers。没有 multi-source/alternative/animated/terrain/custom data、cell transform/tint、sparse chunk/infinite map、physics/nav/occlusion/object layer、runtime edit 或 cell picking。目标必须分成 `Texture Source + Sprite Import Recipe -> Sprite/Atlas/Flipbook artifacts` 与 `TileSet Source -> validated TileSet -> transactional TileMap document -> chunk/collision/nav/occlusion cook -> Scene TileMap component -> streaming renderer`。

## 2. 当前物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与说明 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **84 / 11,284 / 10,278 / 389,980 / 82 / 0** | Sprite/atlas/scene/render/visibility、TileSet/TileMap schema、Editor atlas、tilemap plugin/catalog；fingerprint `6bf47abe138db32708ff5dc780f49e8867a425002068de86217495381849d2ff` |
| Unreal/Godot/Fyrox/Bevy reference | **16 / 6,711 / 5,554 / 248,066 / 11 / 0** | Paper2D Sprite/Atlas/Flipbook/TileSet/TileMap、Godot Sprite/TileMap/TileSet editor、Fyrox/Bevy sprite render；fingerprint `9e6e57bc753b44abb1d6cf3d2c89f50e0f7b0a77bb8b8391c594eabe3ee52def` |
| Zircon selected union | **100 / 17,995 / 15,832 / 638,046 / 93 / 0** | 两组路径不重叠；fingerprint `61df3b02fa317153999322c074c99885c0f61bde8c970ae2981d76b0a69209e0` |

逐层事实：

1. Sprite2D component 没有 `ZrReflect`；NodeKind/editor create/picking 只有 Empty/Camera/Cube/Mesh/Light，没有 Sprite2D/TileMap/Canvas2D/Camera2D。
2. SceneEntityAsset 没有 Sprite2D 字段，TileMap load/save 不对称；Scene reference test 只统计 reference，不证明 World roundtrip。
3. Sprite snapshot 的 material handle 不进入 shader/bind group/pipeline 或 batch key；Opaque/AlphaMask/Transparent phase 仍使用同一固定 alpha blend、depth write off、无 discard threshold 管线。
4. batch key 只有 Texture ID，连续同纹理才合并；不同 material 会被错误合批。每个 batch 每帧 `create_buffer_init` 并单独开 render pass，没有 persistent/ring/instance/multi-draw/indirect path。
5. `RenderSpriteBounds` 无 production consumer；visibility payload 没 bounds，graphics 不执行 Sprite bounds culling。z-order/camera/sorting/y-sort 也由 extract 固定或 component 缺失。
6. Atlas validation 很严格（zero size、duplicate/blank name、pixel rect、finite/range/order/derived UV），但没有 trim/rotation/extrude/dilate、multi-page、mip/compression/color-space、stable layout、platform profile 或 atomic publish。
7. Atlas writer 先写 PNG 再写 TOML，第二步失败会留下半发布 artifact；唯一实际 consumer 是 Editor host image cache，非 gameplay Sprite/Flipbook。
8. TileSet 无 validate；TileMap validation 未检查 zero dimensions、checked multiplication、duplicate layer/name、opacity range、unknown tile id、resource consistency、cell budget；dense vector不支持大世界 sparse/chunk。
9. `BuiltinRenderFeature::Tilemap` 只有 enum/slot/descriptor test，没有 renderer/install/extract/dirty chunk/collision/navigation/occlusion。
10. TileSet collider string 没有 PhysicsMesh/cook/backend consumer；Runtime 没有 TileMap collider/nav/replication/runtime cell edit。
11. tilemap_2d runtime provider 只注册 dynamic component descriptor；Editor provider 的 Import/Create/Open/Paint operations 没 factory，声明的 `authoring.zui` 与 `tilemap_component.zui` 不存在。
12. `apply_tilemap_paint()` 直接改 dense cell vector，无 document revision/command/transaction/dirty/save/undo；palette/layer/grid/picking/brush/terrain/animation/collision editor 均缺失。
13. first-party runtime/editor catalog 与 App feature 没有 TileMap branch/dependency；DiagnosticOnly Tiled importer 只报告 backend unavailable。

## 3. 参考引擎对照

- Unreal Paper2D 的 Sprite/Atlas/Flipbook/TileSet/TileMap/Layer/Component 与独立 Sprite/TileSet/TileMap/Atlas toolkits，提供 trim/rotation/pivot/PPU/material/collision/physics/atlas page/transaction/cook 分层。
- Godot TileSet/TileMapLayer 支持多 source、alternative/animation/terrain/physics/navigation/occlusion/custom data、quadrant/dirty/chunk runtime 与专用 editor；Sprite editor 也有真实 picking/preview。
- Fyrox 把 sprite/tilemap data、tileset、brush、autotile、collider、command/palette/preview 分开；Bevy Sprite/TextureAtlas/TilemapChunk 提供 typed handle 与 chunk mesh/change detection 的 Runtime 基线，但没有同级 Editor 产品。

## 4. P0：数据保真与运行时断路

| ID | 当前差异 | 必须重构 |
|---|---|---|
| P0-1 | Sprite2D 不在 Scene source/load/save | Scene SpriteComponent + stable texture/atlas entry + roundtrip receipt |
| P0-2 | TileMap source load/save 会丢失 | World install/save 对称、typed TileMapComponent、generation-qualified migration |
| P0-3 | TileMap/Sprite/Atlas 没有 Runtime product owner | 统一 source/artifact/install/renderer/collision/nav/chunk owner |
| P0-4 | Sprite material/pipeline/batch 语义错误且低效 | material-aware pipeline key、alpha-mask/depth semantics、persistent instance/batch path |
| P0-5 | Editor2D/TileMap/Atlas operations/ZUI/factory 缺失 | 正式 asset/toolkit/transaction/preview/job/receipt，不再使用固定 feedback |

## 5. P1：Sprite、Atlas、TileSet、TileMap、Canvas 与质量

| ID | 差异 | ID | 差异 |
|---|---|---|---|
| P1-01 | Sprite asset identity 缺失 | P1-02 | Sprite2D reflection/Inspector 缺失 |
| P1-03 | Sprite Scene stable reference 缺失 | P1-04 | atlas handle/entry generation 缺失 |
| P1-05 | atlas import recipe/digest 缺失 | P1-06 | atlas packer 无 production job |
| P1-07 | trim/rotate/extrude/dilate 缺失 | P1-08 | multi-page/mip/compression policy 缺失 |
| P1-09 | atomic PNG/TOML publication 缺失 | P1-10 | reimport/repack diff 缺失 |
| P1-11 | platform/color-space profile 缺失 | P1-12 | Sprite thumbnail/toolkit 缺失 |
| P1-13 | material 不是 batch/pipeline identity | P1-14 | alpha mask/discard/depth semantics 缺失 |
| P1-15 | texture-only adjacent batching | P1-16 | per-batch buffer/pass 分配 |
| P1-17 | persistent/instance/indirect path 缺失 | P1-18 | Sprite bounds culling 缺失 |
| P1-19 | camera/sorting/y-sort/canvas layers 缺失 | P1-20 | pixel snap/PPU/pivot contract 缺失 |
| P1-21 | Sprite animation/Flipbook 资产缺失 | P1-22 | frame event/timeline/socket 缺失 |
| P1-23 | TileSet validation 缺失 | P1-24 | tile stable source/alternative identity 缺失 |
| P1-25 | multi-source/terrain/custom-data 缺失 | P1-26 | animated tile/proxy 缺失 |
| P1-27 | TileSet collider typed schema 缺失 | P1-28 | TileMap layer schema/version 缺失 |
| P1-29 | TileMap cell checked budget 缺失 | P1-30 | sparse/chunk/infinite representation 缺失 |
| P1-31 | cell transform/tint/flip 缺失 | P1-32 | projection math/picking/bounds 缺失 |
| P1-33 | unknown tile/resource validation 缺失 | P1-34 | layer identity/merge/conflict 缺失 |
| P1-35 | TileMap component install 缺失 | P1-36 | chunk compiler/dirty update 缺失 |
| P1-37 | TileMap renderer/extract 缺失 | P1-38 | TileMap streaming/residency 缺失 |
| P1-39 | 2D collision cook 缺失 | P1-40 | navigation/occlusion cook 缺失 |
| P1-41 | runtime cell edit/replication 缺失 | P1-42 | collider backend qualification 缺失 |
| P1-43 | TileMap resource reference roundtrip 缺失 | P1-44 | import/reimport/source provenance 缺失 |
| P1-45 | Tiled importer 无 backend | P1-46 | tiled layer/object/image/group mapping 缺失 |
| P1-47 | TileMap create/import operation factory 缺失 | P1-48 | declared ZUI templates 缺失 |
| P1-49 | 2D Scene node create/picking 缺失 | P1-50 | canvas/orthographic editor camera 缺失 |
| P1-51 | palette/layer/grid editor 缺失 | P1-52 | brush/stamp/line/rect/bucket/picker 缺失 |
| P1-53 | terrain/autotile/pattern/WFC 缺失 | P1-54 | animation/onion-skin/collision edit 缺失 |
| P1-55 | command/undo/redo/dirty/save 缺失 | P1-56 | preview/thumbnail/stale receipt 缺失 |
| P1-57 | missing asset/error projection 缺失 | P1-58 | large map/render/cook budgets 缺失 |
| P1-59 | fault/cancel/restart/recovery matrix 缺失 | P1-60 | cross-platform 2D quality/performance gate 缺失 |

## 6. P2、Gate 与重构顺序

P2 全部 Open：nine-slice/vector sprite、skeletal 2D animation、deformation/mesh sprite、tile terrain graph、procedural/WFC authoring、HLOD/streaming world、remote atlas/cook、collaborative tilemap merge、runtime editor scripting、跨引擎 Paper2D/Godot import 与大型 2D benchmark。

32 个 Gate 当前为 **32 Fail / 0 Partial / 0 Pass**。必须证明 Sprite/TileMap Scene load-save 不丢数据；atlas entry generation/repack 可修复引用；material/alpha/depth/visibility batching 正确；TileSet/TileMap/Collision/Nav/Chunk artifact 同一 source/revision；Editor brush/undo/save/preview/cancel/crash/restart 有 receipt；large sparse/infinite maps、GPU/CPU memory、draw calls、streaming/collision rebuild、Tiled import 和跨平台 cook 有冻结基准。

1. **M0 保真/owner**：补 Sprite2D/TileMap Scene component、load/save roundtrip，冻结 TileMap/Sprite/Atlas Runtime owner，删除无 consumer 的虚假 feature slot。
2. **M1 Source/assets**：建立 Sprite/Atlas/Flipbook/TileSet/TileMap typed source、stable ids、recipe、entry/page/layer/cell schema、reimport diff。
3. **M2 Derived/runtime**：atomic atlas pages、material-aware Sprite artifact/batching、TileMap chunk compiler、renderer/extract、collision/nav/occlusion cook、streaming/runtime edit。
4. **M3 Editor**：正式 Sprite/Atlas/TileSet/TileMap toolkits、orthographic canvas、palette/layer/grid/picking/brush/terrain/animation/collision、Editor02 transaction/undo/save。
5. **M4 Qualification**：Tiled/atlas corpus、fault/cancel/restart、large map and sprite stress、GPU capture、draw/alloc/RSS、multi-platform artifact/reimport receipts。

禁止把 UI icon atlas cache 改名 SpriteAtlas；禁止用 `apply_tilemap_paint()` 直接写 Vec 或用 enum/descriptor/ZUI 名称冒充 TileMap；禁止把 texture-only adjacent batch、固定 alpha blend、默认 material、静态 test、placeholder template 或 coverage 数字当 2D 性能与产品完成度。本轮只做静态 review，没有修改生产代码或运行 2D 动态 lane。
