---
title: Editor Sprite、Atlas、TileSet、TileMap、Canvas 2D、Animation、Collision 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor155
review_date: 2026-08-27
baseline_head: 60d6ef9e98acc76b6433b01a7c6dc7ad2c0eb439
verification_head: d6370a7a9759d25d603e23a3b0380a8dcb7fbcd9
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor34
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/108-editor-sprite-atlas-tileset-tilemap-canvas2d-animation-collision-preview-current-source-review.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/sprite_atlas
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/scene/components/render2d
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
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperSprite.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperSpriteAtlas.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperFlipbook.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileSet.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileMap.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileLayer.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2D/Classes/PaperTileMapComponent.h
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/Atlasing/PaperAtlasGenerator.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/TileMapEditing/EdModeTileMap.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/TileMapEditing/TileMapEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/TileSetEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/SpriteEditor/SpriteEditor.cpp
  - dev/godot/scene/2d/sprite_2d.h
  - dev/godot/scene/2d/sprite_2d.cpp
  - dev/godot/scene/2d/animated_sprite_2d.h
  - dev/godot/scene/2d/animated_sprite_2d.cpp
  - dev/godot/scene/2d/tile_map_layer.h
  - dev/godot/scene/2d/tile_map_layer.cpp
  - dev/godot/scene/resources/2d/tile_set.h
  - dev/godot/editor/scene/2d/sprite_2d_editor_plugin.cpp
  - dev/godot/editor/scene/2d/tiles/tile_map_layer_editor.cpp
  - dev/godot/editor/scene/2d/tiles/tile_set_atlas_source_editor.cpp
  - dev/Fyrox/fyrox-impl/src/scene/sprite.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/data.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/tileset.rs
  - dev/Fyrox/fyrox-impl/src/scene/tilemap/tile_collider.rs
  - dev/Fyrox/editor/src/plugins/tilemap/commands.rs
  - dev/Fyrox/editor/src/plugins/tilemap/interaction_mode.rs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Renderer2DData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/Renderer2DRendergraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/DrawRenderer2DPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Lights/Light2DCullResult.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/PixelPerfectCameraInternal.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Editor/Renderer2DEditorTests.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Editor/RenderSpriteTests.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Runtime/Renderer2DTests.cs
  - dev/Graphics/Tests/SRPTests/Projects/UniversalGraphicsTest_2D/Assets/Test/Runtime/TilemapRenderer2DTests.cs
finding_status:
  p0_open: 5
  p0_partial: 0
  p1_open: 44
  p1_partial: 16
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 26
  partial: 6
  pass: 0
---

# Editor155 · Sprite / Atlas / TileSet / TileMap / Canvas 2D / Animation / Collision / Preview 当前源码复核

## 1. 结论

Editor108之后出现了两类真实但局部的工程进展。第一，TileMap Editor paint helper从数组下标写入改为经过校验的layer name identity，单次stroke限制为4,096个唯一cell，先验证整笔请求再修改，失败时保持asset不变，并用cell state delta更新统计。第二，Sprite renderer补出了`PreparedSpriteQueueStats`及Runtime diagnostic投影，能报告sprite、slice expansion、batch、pass、vertex及Opaque/Mask/Transparent阶段数量。Atlas manifest discovery也将重复目录收集收束为线性收集后一次排序。这些底层应保留。

它们没有关闭2D产品链。Project Scene仍在load时固定`Sprite2dComponent = None`，save时固定`tilemap = None`，因而Sprite2D从未进入Scene source contract，TileMap一轮load/save仍会丢失。通用World clone/serde snapshot能携带Sprite/Mesh2D，只证明ECS transport有局部基础，不能替代Project Scene roundtrip。

Sprite renderer仍只有一个内嵌`textureSample * color` shader和一个始终alpha blend、depth write关闭的pipeline；material不进入shader、bind group、pipeline或batch key，AlphaMask没有discard threshold。batch只合并相邻同texture项，每个batch每帧`create_buffer_init`并打开独立render pass。Sprite有typed bounds和phase queue DTO，但visibility输入没有bounds，Graphics侧也没有Sprite visibility result消费，所以离屏Sprite仍进入vertex generation与draw。

Atlas仍是Editor cache工具。`SpriteAtlasAsset`校验严格，packer确定性且避免RGBA全量clone；但`pack_sprite_atlas_sources()`与`write_sprite_atlas_artifacts()`的production caller为零，后者固定写`.zircon/cache/editor-sprite-atlases`，先发布PNG再写TOML，不具备recipe/digest/generation/DDC/platform variant或多文件原子性。ResourceKind/ImportedAsset/typed marker中仍没有Sprite、SpriteAtlas或Flipbook，Scene Sprite也只能直接拼texture和inline UV。

TileMap runtime仍只有动态component descriptor、asset importer descriptor和`DiagnosticOnlyAssetImporter`，不存在typed TileMap component、chunk compiler、renderer、collision、navigation、occlusion、streaming或runtime mutation owner。Editor仍公开Import/Create/Open/Paint五个operation，但没有factory/controller；声明的`authoring.zui`和`tilemap_component.zui`仍不存在。first-party runtime/editor catalogs和`zircon_app`仍未装配TileMap。插件将runtime capability诚实标为`Partial`，但README把descriptor层称为runtime-backed authoring plugin，仍高于实际可执行能力。

因此Editor34继续是canonical owner。本轮只刷新currentness，不增加canonical finding总数：**5个P0全部Open；60个P1为44 Open / 16 Partial / 0 Closed；12个P2全部Open；32门为26 Fail / 6 Partial / 0 Pass**。目标链保持为：

`Texture Source + Sprite Import Recipe -> versioned Sprite Source -> Atlas/Flipbook compile -> immutable generation artifacts -> Scene component -> material-aware renderer`

`TileSet Source -> validated TileSet -> transactional TileMap document -> chunk/collision/navigation/occlusion cook -> generation-qualified Scene TileMap component -> streaming renderer`

## 2. 当前物理范围与证据等级

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | working-tree指纹与说明 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **87 / 12,017 / 11,018 / 418,945 / 98 / 8** | 资产schema、Scene persistence、Sprite extract/renderer、Atlas cache、TileMap plugin/catalog/App；`eca57be6964a9fae3b210f562a2f9cb948b38ceebd30b6c6c8a5191d09bb123a` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **41 / 32,872 / 28,501 / 1,277,943 / 48 / 0** | Paper2D、Godot TileMap quadrant/editor、Fyrox tile/brush/command、Bevy chunk renderer、Unity URP 2D与tests；`676f0c90287162c485bc3424ee1b06747ed959214f51ad4c4ad81d2397073afe` |
| 全部选择集 | **128 / 44,889 / 39,519 / 1,696,888 / 146 / 8** | 两组路径不重叠；`16b401509d7b84648f18f5fdf389f3944da6dfd4e84624282be2e12d399156d9` |

指纹算法为：按仓库相对路径ordinal排序，对每个文件计算SHA-256，形成`relative_path<TAB>file_sha256<LF>`清单后再计算SHA-256。统计以当前共享dirty working tree为准；实施前必须重算，因为本轮明确保留了其他会话的源码与文档修改。

本轮只做静态源码review，没有运行Cargo、Editor、WGPU、import/cook、图像golden、fault、scale、soak或跨平台动态lane。按用户要求排除Tooling优化，也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前存在且必须保留的底座

1. `Sprite2dComponent`已表达texture/material、inline atlas UV、rect、flip、anchor、custom size、Fit/Fill/Tiled/Sliced、tint、z-order和alpha mode；Sprite geometry tests覆盖多种image mode。
2. Sprite extract拥有2D/3D render phase、queue value和稳定entity tie-break；`PreparedSpriteQueueStats`把batch、slice expansion、vertex与阶段数量投影到Runtime diagnostics。
3. `SpriteAtlasAsset`对zero dimensions、blank/duplicate name、source dimensions、rect bounds、UV finite/range/order及pixel-derived UV一致性做严格校验。
4. Atlas packer对输入排序、RGBA长度和max size有确定性校验，使用rectangle pack并支持基础padding；artifact writer会在写manifest前再次执行Runtime validation。
5. `ResourceKind::TileSet/TileMap`、typed marker/load API和built-in TOML ingest已经存在；TileMap source支持四种projection和多layer dense cells。
6. 新paint kernel拥有bounded request、唯一cell、layer identity preflight、checked index、failure atomicity及统计delta，测试覆盖layer reorder后仍按identity命中。
7. TileMap插件明确声明`CapabilityStatus::Partial`，Tiled后端缺失时使用DiagnosticOnly importer，而不是静默伪造import artifact。
8. Editor asset registry、operation descriptor、toolkit/creation contribution、Background Job、transaction/save/recovery、Runtime resource streamer和render graph可作为正式产品的共享底座，不应另建2D专用简化框架。

## 4. 当前断路与错误authority

| 当前表面 | 当前真实行为 | 工程断路 | 目标authority |
|---|---|---|---|
| Scene Sprite2D | ECS component与generic snapshot存在 | Project Scene source/load/save没有Sprite字段 | versioned Scene Sprite component mapper |
| Scene TileMap | `SceneEntityAsset.tilemap`可反序列化 | World load忽略、save固定`None` | typed TileMap component + artifact reference |
| Sprite material/alpha | component和phase分类存在 | 固定shader/pipeline、material未消费、Mask无discard | material/pipeline resolver与PSO key |
| Sprite bounds/visibility | typed bounds和identity存在 | bounds未进入visibility，离屏仍tessellate/draw | pre-submit Sprite/Canvas culling |
| Sprite batching | 相邻同texture合并 | 忽略material/pipeline/page；per-batch buffer/pass | persistent instance/ring + state-aware batch |
| Atlas pack/build | tested pure pack与PNG/TOML writer | 无source recipe、job、caller、generation、atomic publish | shared compiler + artifact transaction |
| Atlas runtime | Editor retained-host cache消费manifest | gameplay asset/resource/install路径不存在 | Sprite/Atlas typed asset与streamer generation |
| TileSet/TileMap source | 单image、dense layer、numeric tile ID | 无stable ID/multi-source/typed layers/chunks/version | versioned source schema与migration |
| TileMap runtime plugin | descriptor + DiagnosticOnly importer | 无typed component/system/cook/renderer | Runtime TileMap product owner |
| TileMap Editor | 五个command/menu/contribution descriptor | operation factory、controller与两份ZUI缺失 | transactional toolkit/document session |
| Paint helper | bounded in-memory asset mutation | 无command/undo/dirty/save/conflict/job/cancel | Editor02 scoped command transaction |
| Product assembly | package manifest可单独描述plugin | first-party catalogs/App没有feature/provider branch | profile-qualified runtime/editor provider |

## 5. 参考引擎对照结论

| 参考 | 已核验的工程事实 | Zircon必须吸收的边界 |
|---|---|---|
| Unreal Paper2D | `PaperSprite`分离SourceTexture与BakedSourceTexture，保存PPU、collision/render rebuild；Atlas有GUID/padding/build state；TileSet有terrain/per-tile metadata；TileMapComponent区分owned asset并集中rebuild collision；paint/erase/fill使用`FScopedTransaction`与`Modify()` | Source/derived分层、stable identity、asset/component ownership、批量collision rebuild和正式可撤销toolkit |
| Godot | TileMap cell以source/atlas coordinate/alternative组成identity；TileSet拥有alternative、terrain、custom data、physics/nav/occluder；TileMapLayer维护render/physics quadrant、dirty flags、navigation与occluder，并支持y-sort和terrain solver；Editor使用UndoRedo action | typed tile identity、sparse quadrant dirty update、多域cook、projection-aware editor与统一undo |
| Fyrox | Sprite以material为batch边界；TileMap把data、tileset、tile source、collider、brush/autotile/property/update分模块；Editor有commands、palette、interaction mode、collider/tileset/preview | Rust内的typed resource/command设计、brush source与tile source分离、material-aware batching |
| Bevy | Sprite使用Image与TextureAtlasLayout handles；renderer进入Transparent2d phase并计算batch range；TilemapChunk缓存共享mesh，用tile-data image承载cells并只在`Changed<TilemapChunkTileData>`时更新 | typed handles、ECS change detection、chunk mesh/cache和GPU tile-data更新路径 |
| Unity Graphics URP 2D | Renderer2DData表达layer mask、blend styles、camera sorting layer texture、shadow memory budget；RenderGraph独立normal/light/shadow/sorting/pixel-perfect资源；2D test project覆盖Renderer和TilemapRenderer | 2D render graph、sorting/light/shadow/normal/mask、pixel-perfect contract、GPU预算与visual regression矩阵 |

这些参考并非都达到同一成熟度：Paper2D仍含未完成terrain TODO，Bevy不提供同级Editor产品，Unity Graphics仓主要覆盖renderer而非完整authoring。Zircon应吸收经过源码与测试证明的owner边界和数据流，不复制某一引擎的历史包袱。

## 6. P0当前状态

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | Open | Project Scene load固定`Sprite2D=None`，save固定`TileMap=None`；generic snapshot不构成项目roundtrip | 先建立Sprite/TileMap source mapper、missing-provider policy、unknown preservation与roundtrip tests |
| P0-2 | Open | TileMap只有asset kind、feature slot和dynamic descriptor，无typed runtime component/system/renderer/cook | 冻结Runtime owner，贯通artifact install、chunk render、collision/nav/occlusion与runtime policy |
| P0-3 | Open | 五个operation无factory/controller，两份声明ZUI不存在，贡献仍可见 | 资源/factory/capability任一缺失时整体禁用；正式接入document/transaction/job/receipt |
| P0-4 | Open | Sprite/Atlas/Flipbook不是正式ResourceKind；Atlas pack/write仅由tests调用且固定Editor cache | 建立typed Sprite/Atlas/Flipbook source、compiler、artifact、streamer与toolkit |
| P0-5 | Open | TileSet仍是单image + string collider，TileMap仍是单tileset + dense numeric cells | 设计schema v2、stable identities、typed data layers、sparse chunks、migration与budgets |

## 7. P1逐项状态

| ID | 状态 | 当前证据与剩余重构 |
|---|---|---|
| P1-01 | Open | 无正式`SpriteAsset`、stable local ID、source revision或default material authority。 |
| P1-02 | Open | `SpriteAtlasBuildConfig`只是进程内pack参数，未形成可持久化recipe与derived layout边界。 |
| P1-03 | Open | atlas entry仍以display name引用，无stable Sprite ID、redirect或generation lookup。 |
| P1-04 | Open | manifest缺source/dependency digest、compiler version、trim/rotation、platform与artifact generation。 |
| P1-05 | Open | anchor/custom size是component局部字段；PPU、pixel center、origin、trim compensation无共享数学authority。 |
| P1-06 | Open | socket、render/collision polygon、secondary textures、material slots与nine-slice source owner缺失。 |
| P1-07 | Open | TileSet仍单image、格子式numeric ID，无multi-source、stable TileId或proxy。 |
| P1-08 | Open | per-tile仅`name`和`Option<String> collider`，未类型化physics/nav/terrain/custom/animation/material。 |
| P1-09 | Open | cell仍是`Option<u32>`，无alternative、transform、tint、variant seed或instance data。 |
| P1-10 | Open | projection只有枚举，无cell/local/world、neighbor、bounds、picking和stagger/hex参数golden。 |
| P1-11 | Open | source以全矩形dense Vec保存；无sparse chunk/infinite map与cell/chunk/bytes admission。 |
| P1-12 | Open | Sprite/Atlas/TileSet/TileMap没有显式schema version、upgrader、downgrade拒绝与unknown preservation。 |
| P1-13 | Partial | generic World clone/serde transport能保留Sprite/Mesh2D，但Project Scene mapper仍确定性丢Sprite/TileMap。 |
| P1-14 | Open | plugin component只是动态descriptor，背后无typed component、artifact handle、generation或override policy。 |
| P1-15 | Open | 没有TileMap renderer、chunk mesh/instance/tile-data texture或dirty chunk更新路径。 |
| P1-16 | Partial | Sprite bounds和visibility identity DTO存在，但bounds未进入visibility，TileMap bounds不存在，culling未在tessellation前生效。 |
| P1-17 | Open | material handle存在于component/snapshot，却未resolve shader/bindings/secondary textures，也未进入batch key。 |
| P1-18 | Partial | Opaque/Mask/Blend phase标签存在；三个阶段仍共享alpha-blend/depth-off pipeline，Mask无discard。 |
| P1-19 | Partial | `z_order`与stable entity tie-break可用；sorting layer、y/custom axis、camera/canvas order仍固定或不存在。 |
| P1-20 | Partial | 已有相邻同texture batch和queue统计；仍为每batch每帧buffer + render pass，且错误忽略material/pipeline。 |
| P1-21 | Open | Atlas page没有resource streaming generation、eviction、repack fence或GPU lifetime合同。 |
| P1-22 | Open | Texture/Sprite/Atlas/Flipbook/TileSet/TileMap无stage/validate/frame-boundary atomic install receipt。 |
| P1-23 | Open | 全仓无Canvas2D/CanvasLayer runtime authority、clip/modulate/screen-space hierarchy。 |
| P1-24 | Open | 全仓无Camera2D/pixel-perfect product；现有普通camera没有2D limits/reference resolution/safe area。 |
| P1-25 | Open | Sprite/Atlas/TileSet/TileMap没有可执行document session/details/viewport/save/conflict/reimport toolkit。 |
| P1-26 | Open | texture import没有single/grid/automatic/manual slicing recipe、stable ID diff或预览。 |
| P1-27 | Open | Atlas pack/write没有production caller，也未进入Editor09 job admission/progress/cancel/publication。 |
| P1-28 | Partial | packer有padding、deterministic input和max size；trim/rotation/extrude/dilate/multipage/mip/compression/secondary alignment缺失。 |
| P1-29 | Open | 无stable/incremental layout、page/UV diff、waste统计或基于stable ID的引用修复。 |
| P1-30 | Open | 无Sprite source region、pivot/socket/render/collision/nine-slice编辑模式。 |
| P1-31 | Open | 无TileSet grid/source/alternative/data layer/animation/terrain/collision/nav/occlusion editor。 |
| P1-32 | Open | 无projection-aware canvas、layer tree、palette、selection、hover/pick与visible chunk culling。 |
| P1-33 | Partial | bounded atomic stroke kernel可作为commit算法底座；brush/stamp/line/rect/bucket/erase/picker/preview/job/cancel均不存在。 |
| P1-34 | Partial | layer name经过唯一性/长度校验且数组reorder不改变paint target；尚无serialized stable LayerId、selection/clipboard/lock/hide/remap。 |
| P1-35 | Open | paint仍直接修改asset Vec；无command merge、before/after chunk diff、undo/redo、dirty/save/recovery/conflict。 |
| P1-36 | Open | 无isolated PreviewWorld、真实runtime artifact preview或Editor/PIE共享renderer。 |
| P1-37 | Open | 无Sprite collision geometry source、validation、simplification或physics cook artifact。 |
| P1-38 | Open | string collider无backend consumer；无dirty chunk collider build与stroke-end批处理。 |
| P1-39 | Open | 无per-tile navigation/occluder/custom data与generation-qualified chunk cook。 |
| P1-40 | Open | 无terrain set、neighbor mask、weighted alternative与deterministic solver。 |
| P1-41 | Open | 无Pattern/Brush Macro/WFC source、compiler、budget、seed或cancelable job。 |
| P1-42 | Open | ResourceKind/ImportedAsset/marker中无Flipbook，frame/duration/event/dependency合同不存在。 |
| P1-43 | Open | 全仓无AnimatedSprite runtime component、deterministic update phase与Scene persistence。 |
| P1-44 | Open | 未复用Editor14 timeline/onion skin/frame strip/event marker owner。 |
| P1-45 | Open | 无stable socket identity、frame pose attachment、redirect/diagnostic。 |
| P1-46 | Open | 固定Sprite shader只采样主texture；Light2D/normal/mask/secondary textures无产品合同。 |
| P1-47 | Open | Scene NodeKind/create/picking没有Sprite2D/TileMap/Canvas2D；无actual geometry/cell hit与真实overlay。 |
| P1-48 | Partial | queue stats能报告batch/pass/slice/vertex；没有Runtime/Compiler generation-qualified grid/chunk/bounds/overdraw/collision/nav质量视图。 |
| P1-49 | Open | TMX/TSX/JSON importer仍是DiagnosticOnly，没有解析或fidelity report。 |
| P1-50 | Open | 无TMX->TSX->image/template依赖图、digest、stable subasset与incremental reimport。 |
| P1-51 | Partial | importer registry与DiagnosticOnly降级符合“backend缺失不得伪造”的方向；normalized 2D IR与Aseprite/TexturePacker/LDtk策略仍缺失。 |
| P1-52 | Partial | package/runtime/editor descriptor和native dist manifest存在；first-party catalogs、App feature/profile和可执行provider装配缺失。 |
| P1-53 | Open | Import/Create/Open/Paint无factory、payload validation、document scope、owner lease、cancel/shutdown receipt。 |
| P1-54 | Open | 两份声明ZUI不存在；resource hash/cook/admission没有把断裂贡献整体拒绝。 |
| P1-55 | Partial | Atlas validation、DiagnosticOnly importer和paint diagnostics提供局部错误；无统一typed journal、provenance、repair action与surface投影。 |
| P1-56 | Partial | Atlas尺寸/rect/UV校验与4,096-cell stroke cap是局部budget；map multiplication、layers/XML/JSON/property/geometry总体预算和fuzz lane缺失。 |
| P1-57 | Partial | in-memory pack对同序输入确定；没有source/dependency/settings/compiler/platform key、byte-identical cook或local/remote DDC。 |
| P1-58 | Partial | Sprite queue与paint stats已存在；无cull ratio、upload bytes、atlas/chunk residency、cook/reimport时间和generation provenance。 |
| P1-59 | Partial | 98个选取范围tests覆盖Atlas validation/pack、Sprite geometry/queue与paint failure atomicity；migration/roundtrip/render/collision/nav/device loss/fault矩阵缺失。 |
| P1-60 | Open | 无Windows/Linux/macOS与目标GPU的2D visual/performance/release qualification evidence。 |

## 8. P2当前状态

| ID | 状态 | 当前结论 |
|---|---|---|
| P2-01 | Open | 无2D skeletal deformation、deform bounds、GPU skinning与SpriteMask交互。 |
| P2-02 | Open | 无Sprite Shape/spline terrain source与render/collision派生链。 |
| P2-03 | Open | 无palette swap、lookup texture、per-instance parameter block与batch-compatible variant。 |
| P2-04 | Open | 无高级hex/isometric elevation、height layer、custom sort axis与projection-aware navigation。 |
| P2-05 | Open | 无World Partition集成的region/chunk async streaming、HLOD/minimap artifact。 |
| P2-06 | Open | 无runtime copy-on-write overlay、save delta、replication/prediction/rollback。 |
| P2-07 | Open | 无procedural tile rule graph、deterministic compile、局部求解与debug trace。 |
| P2-08 | Open | 无chunk/cell协作operation、stable layer merge/lock/conflict/provenance。 |
| P2-09 | Open | 无基于runtime telemetry的可复现Atlas热度布局建议与recipe approval。 |
| P2-10 | Open | 无duplicate/unused/missing/waste/overdraw/seam分析及transactional repair。 |
| P2-11 | Open | 无GPU cull、indirect/multi-draw、bindless page或chunk compaction资格。 |
| P2-12 | Open | 无跨Paper2D/Godot/Fyrox/Bevy/Unity Graphics统一任务与性能基准。 |

## 9. Gate当前状态

| Gate | 状态 | 当前证据缺口 |
|---|---|---|
| G01 Scene Sprite roundtrip | Fail | Project Scene没有Sprite source字段，load固定None。 |
| G02 Scene TileMap roundtrip | Fail | save固定TileMap None，provider三态未定义。 |
| G03 Schema migration | Fail | 无v2 schema、migration corpus与downgrade拒绝。 |
| G04 Stable identity | Fail | Sprite/Atlas entry/Tile/layer没有完整stable identity与redirect。 |
| G05 Atlas atomicity | Fail | PNG先写、TOML后写，失败/取消/崩溃可见半artifact。 |
| G06 Atlas deterministic | Partial | pure pack顺序可确定；clean/DDC/remote-DDC与artifact hash未建立。 |
| G07 Atlas quality | Fail | trim/rotation/extrude/dilate/mip/secondary/platform visual golden缺失。 |
| G08 TileSet validation | Fail | TileSet无validate，string collider与unknown tile未封闭。 |
| G09 Projection golden | Fail | 无转换、邻接、bounds、pick的golden/fuzz。 |
| G10 Sparse scale | Fail | source随完整矩形增长，无occupied-chunk证据。 |
| G11 Sprite material | Fail | 同纹理不同material仍会错误合批并使用同一shader。 |
| G12 Alpha semantics | Fail | 三阶段仍共享固定alpha blend/depth-off、无Mask discard。 |
| G13 Sorting | Partial | z-order/entity tie-break存在；layer/y-axis/camera/canvas/cross-2D-3D矩阵缺失。 |
| G14 Sprite culling | Fail | bounds不进入visibility，离屏Sprite仍生成vertices。 |
| G15 TileMap chunk renderer | Fail | renderer/chunk/update path不存在。 |
| G16 GPU lifetime | Fail | Atlas generation/eviction/device-loss/surface-recreate合同不存在。 |
| G17 Collision cook | Fail | typed geometry与dirty chunk physics build不存在。 |
| G18 Navigation/occlusion | Fail | generation-qualified nav/occlusion artifact不存在。 |
| G19 Flipbook determinism | Fail | Flipbook asset/runtime/event update不存在。 |
| G20 Editor transaction | Fail | bounded atomic mutation不是undoable Editor02 transaction。 |
| G21 Conflict handling | Fail | 无本地dirty与external reimport三方diff。 |
| G22 Tiled fidelity | Fail | backend未安装，corpus字段未解析。 |
| G23 Reimport dependency | Fail | 无TSX/image/template依赖触发和stable diff。 |
| G24 Plugin admission | Partial | capability Partial和DiagnosticOnly状态诚实；缺ZUI/factory时贡献仍可见。 |
| G25 Job cancellation | Fail | Atlas/Tiled/fill/autotile/cook均无正式job/cancel terminal。 |
| G26 Malformed/fuzz | Partial | Atlas和paint有checked局部校验；无全格式fuzz与完整budget。 |
| G27 Performance telemetry | Partial | Sprite queue统计存在；TileMap/cull/upload/residency/generation provenance缺失。 |
| G28 Large-scene baseline | Fail | 无十万Sprite/百万cell/千visible chunk可复现预算。 |
| G29 Visual matrix | Fail | 无lit/mask/normal/sort/dilate/secondary/preview图像矩阵。 |
| G30 Cross-platform | Fail | 无多平台texture/pixel-center/precision证据。 |
| G31 Headless cook/package | Fail | shipping仍没有正式Sprite/Atlas/TileMap cook/install链。 |
| G32 Truthful maturity | Partial | Runtime capability为Partial；README/Editor贡献仍把descriptor层描述为可执行authoring。 |

## 10. 目标Owner与数据流

1. **Runtime Interface**只拥有跨进程稳定identity、source/artifact reference、generation、diagnostic与receipt DTO，不拥有Editor document实现。
2. **Runtime Asset**拥有Sprite/Atlas/Flipbook/TileSet/TileMap source schema、migration、import IR、dependency graph、compiler key和immutable artifact schema。
3. **Runtime Scene**拥有Sprite2D/AnimatedSprite/TileMap/CanvasLayer typed components、source mapper、runtime overrides和generation-qualified install。
4. **Graphics**拥有material-aware Sprite pipelines、persistent instance buffers、Sprite/TileMap bounds/culling、chunk renderer、2D lighting/mask/normal与GPU lifetime。
5. **Physics/Navigation**消费同一Sprite/TileSet/TileMap artifact generation，产出可追踪的collision/navigation/occlusion chunk artifacts，不解析Editor source。
6. **Editor**拥有transactional Sprite/Atlas/TileSet/TileMap documents、toolkits、canvas、selection、commands、preview、job orchestration与conflict/recovery。
7. **Plugin**只在backend、resources、factories和capabilities全部可用时贡献产品入口；不能用descriptor数量充当功能完成度。

```text
Texture/Image Source
  -> Sprite Import Recipe
  -> Sprite Source (stable SpriteId, pivot, trim, geometry, collision, material inputs)
  -> Atlas/Flipbook Compiler
  -> Generation Artifact Set (pages, layout, animation, diagnostics)
  -> Scene Install -> Visibility/Material Pipeline -> Sprite Renderer

TileMap/Tiled Source + Dependencies
  -> Normalized Tile IR
  -> TileSet Source (stable source/tile/alternative ids, typed data layers)
  -> Transactional TileMap Document (stable layer ids, sparse chunks)
  -> Chunk Compiler
  -> Render + Collision + Navigation + Occlusion Artifact Set
  -> Scene Install -> Streaming/Dirty Update -> Runtime Systems
```

## 11. 分层重构里程碑

### M0：Truthfulness与数据保真

关闭Scene Sprite/TileMap load-save断路；在factory/ZUI/backend缺失时隐藏或disabled-with-reason；修正README与capability投影。G01、G02、G24是退出门。

### M1：Stable identity、Schema v2与Migration

建立SpriteId、AtlasEntryId、TileSourceId、TileId、AlternativeId、LayerId和source/artifact generation；完成Sprite/Atlas/Flipbook/TileSet/TileMap v2 schema、unknown preservation与migration corpus。G03、G04、G08、G09通过后继续。

### M2：Shared Compiler、Artifact与Atomic Publication

将Atlas pack、Tiled normalize、TileSet compile和TileMap chunk cook接入统一job/DDC/artifact transaction；支持cancel、failure injection、旧generation保留与dependency diff。G05、G06、G07、G22、G23、G25、G31通过。

### M3：Scene Component与Runtime Install

实现typed Sprite2D、AnimatedSprite、TileMap、CanvasLayer、Camera2D component及source mapper；所有依赖以frame-boundary install receipt提交并由ResourceStreamer管理GPU generation。G16、G19通过。

### M4：Renderer正确性与性能基线

material/pipeline/bindings进入batch key；分离Opaque/Mask/Blend PSO；引入persistent ring/instance buffers、共享pass、Sprite bounds culling和TileMap chunk renderer。G11-G15、G27、G28通过。

### M5：Collision、Navigation、Occlusion与Animation

统一typed per-sprite/per-tile geometry、dirty chunk cook、navigation/occlusion generation和Flipbook event update；所有域共享同一source/artifact revision。G17-G19通过。

### M6：Sprite、Atlas与TileSet Toolkits

基于Editor02/04/09建立document session、slicing、pivot/socket/geometry/collision、Atlas recipe/diff、TileSet data layers/animation/terrain与真实PreviewWorld。

### M7：TileMap Canvas与Tiled Reimport

完成projection-aware canvas、layer tree、palette、selection/clipboard、brush/stamp/line/rect/bucket/picker、terrain/autotile、undo/redo/dirty/save/conflict及dependency reimport。G20-G23通过。

### M8：Fault、规模、视觉与跨平台资格

冻结malformed/fuzz、cancel/crash/restart、device loss、十万Sprite、百万逻辑cell、图像golden、多平台和headless cook证据。G26-G31通过。

### M9：高级2D生态

在P0/P1和G01-G31全部关闭后，再推进skeletal 2D、Sprite Shape、runtime editable/replicated TileMap、procedural graph、collaboration、GPU-driven submission和跨引擎benchmark。

## 12. 禁止的临时修补

1. 禁止把Retained Host icon atlas cache改名为正式SpriteAtlas产品。
2. 禁止用ResourceKind、enum slot、dynamic descriptor、operation path、menu item或ZUI URI数量证明功能存在。
3. 禁止让Scene继续直接保存texture + inline UV，或以display name作为Sprite/Tile/layer持久identity。
4. 禁止用`apply_tilemap_paint_stroke()`直接修改asset Vec冒充transaction；原子内存写入不能替代undo/save/conflict/recovery。
5. 禁止用texture-only相邻batch、固定alpha blend、默认material或队列统计宣称2D renderer性能/正确性完成。
6. 禁止在Editor callback里同步decode/pack/import/fill/autotile/cook，或取消后留下半PNG/TOML与stuck dirty。
7. 禁止为Canvas2D/Camera2D/TileMap另建与现有Scene、Camera、Renderer、Physics、Navigation、Job、Transaction平行的简化服务。
8. 禁止在未通过G01-G31时将TileMap或2D authoring标记Complete/Stable。

## 13. 本轮边界

本报告完成的是Editor34/108 current-source刷新、参考源码差异复核、逐项状态和分层重构计划。它没有修改生产代码，也没有声明任何P0/P1/P2或Gate已经完成。后续实施必须从M0开始，以source/artifact/runtime/editor纵向闭环和可复现动态证据作为验收单位。
