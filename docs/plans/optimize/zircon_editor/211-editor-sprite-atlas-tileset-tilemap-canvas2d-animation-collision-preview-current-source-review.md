---
title: Editor Sprite、Atlas、TileSet、TileMap、Canvas 2D、Animation、Collision 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor211
review_date: 2026-08-28
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor34
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/108-editor-sprite-atlas-tileset-tilemap-canvas2d-animation-collision-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/155-editor-sprite-atlas-tileset-tilemap-canvas2d-animation-collision-preview-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99e-runtime-sprite2d-canvas2d-sprite-atlas-tileset-tilemap-batching-sorting-lighting-physics-streaming-product-integration-current-source-review.md
related_failures:
  - docs/plans/zircon_editor/editor/10/failure-2026-07-17-editor-sprite-atlas-paint-time-io.md
  - docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md
  - docs/plans/zircon_runtime/render/13/failure-2026-07-17-editor-ui-command-payload-duplication.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
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
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/image_resources.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_plugins/tilemap_2d
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/Cargo.toml
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
  - dev/bevy/crates/bevy_sprite/src/lib.rs
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

# Editor211 · Sprite / Atlas / TileSet / TileMap / Canvas 2D / Animation / Collision / Preview 当前源码复核

## 1. 结论

当前工作树已经有若干值得保留的局部能力，但仍没有工程级2D产品链。Sprite component与CPU几何生成支持rect、atlas UV、flip、anchor、custom size、Fit/Fill、Tiled和Sliced；phase queue和统计可以报告阶段、batch、slice与vertex。Editor UI atlas路径也开始传递`resource_key + generation + UV`，稳定visual-asset cache命中时会跳过候选路径构造与文件探测，command stream只在目标generation未驻留时提取一次整页RGBA。TileMap paint kernel按唯一layer name解析，限制4,096个唯一cell，先验证整笔stroke再提交，并以cell delta更新统计。

这些改进没有关闭核心断路。Project Scene source仍没有Sprite/Mesh2D字段，load明确构造`Sprite2dComponent = None`和`Mesh2dComponent = None`，save明确写`tilemap: None`。因此Sprite2D不能通过项目Scene roundtrip，TileMap会在load/save链丢失。generic World clone/serde能携带Sprite/Mesh2D只是内存transport能力，不能替代versioned项目持久化。

Sprite renderer仍只有一个内嵌`textureSample * color` shader和固定alpha blend pipeline。component/snapshot中的material没有进入batch key、bind group、shader或pipeline；Opaque、AlphaMask、Transparent三个stage只是筛选不同phase，实际仍共用alpha blend、depth write disabled且没有alpha discard的管线。batch只合并相邻同texture项，每个batch每帧CPU展开vertices、`create_buffer_init`并单独`begin_render_pass`。`RenderSpriteBounds`没有production consumer，visibility input也没有bounds，离屏Sprite仍会进入CPU几何生成。

Atlas仍是Editor cache工具而非正式游戏资产。`SpriteAtlasAsset`校验和packer有价值，但pack/write函数没有production caller，artifact writer固定输出Editor cache且先写PNG再写TOML。`ResourceKind`、`ImportedAsset`和marker都没有Sprite、SpriteAtlas或Flipbook。retained-host resolver还有确定性错误：`resolve_atlas_uncached()`在manifest循环内使用`?`，首个排序manifest无法加载或不含entry时会直接返回`None`，不会继续检查后续manifest。冷路径还会先解码单图并把其RGBA留在visual asset cache，再附加atlas metadata；这减少了重复I/O和command payload，却不是atlas-only residency。

TileMap plugin已增加可测试的内存paint kernel，但runtime仍注册`DiagnosticOnlyAssetImporter`，不存在typed TileMap component、compiler、chunk renderer、collision/navigation/occlusion cook或streaming owner。Editor公开Import/Create/Open/Paint五个operation和toolkit贡献，却没有任何operation factory/controller，声明的`authoring.zui`与`tilemap_component.zui`物理不存在。内置runtime catalog列出`tilemap_2d`，first-party runtime/editor provider catalog却都不返回它，默认产品无法安装该插件。

Editor34继续是canonical owner，本报告只刷新currentness，不重复增加finding总数。当前状态保持：**5个P0全部Open；60个P1为44 Open / 16 Partial / 0 Closed；12个P2全部Open；32门为26 Fail / 6 Partial / 0 Pass**。目标链保持为：

`Texture Source + Sprite Import Recipe -> versioned Sprite Source -> Atlas/Flipbook compiler -> immutable generation artifacts -> Scene component -> material-aware renderer`

`TileSet Source -> validated TileSet -> transactional TileMap document -> chunk/collision/navigation/occlusion cook -> generation-qualified Scene TileMap component -> streaming renderer`

## 2. 物理范围与证据等级

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | working-tree指纹与说明 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **95 / 13,913 / 12,762 / 486,340 / 123 / 10** | schema、Scene I/O、Sprite extract/renderer、Atlas build/cache/stream、TileMap plugin/catalog/App；`d8885a70396fa4c5bb4a256921fcaee9b9cd409a149901c10624d2934cdecb5c` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **42 / 33,301 / 28,886 / 1,291,753 / 51 / 0** | Paper2D、Godot quadrant/editor、Fyrox tile/command、Bevy bounds/chunk、Unity URP 2D/tests；`307a951e503321d49da1ffdf8f98db5cea0714616cbd086f912cb2de91650855` |
| 全部选择集 | **137 / 47,214 / 41,648 / 1,778,093 / 174 / 10** | 两组按ordinal相对路径去重；`98cdaf195a8181056741016e00f331b558999d39c25ff8d10b486f63ee135694` |

指纹算法为：相对路径ordinal排序，对每个文件计算SHA-256，形成`relative_path<TAB>file_sha256<LF>`后再计算SHA-256。统计基于当前共享dirty working tree，不表示这些源码变化已提交或已通过动态验收。

本轮只做静态源码review，没有运行Cargo、Editor、WGPU、import/cook、图像golden、fault、scale、soak或跨平台动态lane。Tooling按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前存在且必须保留的底座

1. `Sprite2dComponent`已表达texture/material、inline atlas region、rect、flip、anchor、custom size、image mode、tint、z-order和alpha mode。
2. Sprite geometry对非有限值、零尺寸和slice上限有局部防护，支持Fit/Fill/Tiled/Sliced并有focused tests。
3. Sprite phase queue拥有2D/3D阶段、queue value与stable entity tie-break，queue stats能报告batch、slice expansion、vertex和阶段分布。
4. Sprite visibility membership已存在，不能再写成“完全未进入visibility”；当前缺的是bounds与实际camera/spatial culling。
5. `SpriteAtlasAsset`对尺寸、duplicate/blank name、source size、rect bounds、UV finite/range/order和pixel-derived UV一致性做严格校验。
6. Atlas packer对输入、RGBA长度、size与padding有确定性校验，artifact writer在写manifest前再次运行Runtime validation。
7. UI atlas command使用`resource_key + generation + UV`，稳定缓存命中可跳过候选构建和文件探测，resident probe按同generation去重。
8. `ResourceKind::TileSet/TileMap`、typed marker、TOML ingest和direct reference收集已存在，不能另建平行asset registry。
9. TileMap paint kernel具备bounded stroke、唯一cell、layer identity preflight、checked cell index、failure atomicity及增量统计。
10. TileMap plugin把runtime capability标为`Partial`，backend缺失时使用DiagnosticOnly importer，没有静默伪造artifact。
11. Editor已有共享asset type registry、operation descriptor、document/history、background job、extension store、scene mode和render graph底座，2D实现应接入这些owner。

## 4. 当前源码差异

### 4.1 Sprite source、Scene与资源身份

1. Scene source没有Sprite2D或Mesh2D字段，运行时component无法被项目文档表达。
2. Scene load将Sprite2D/Mesh2D固定为`None`，Scene save将TileMap固定为`None`，这是确定性数据丢失而非缺少UI。
3. Sprite直接引用Texture并内联UV/rect，没有Sprite source asset、stable local ID、revision、redirect或dependency digest。
4. `ResourceKind`和`ImportedAsset`只有TileSet/TileMap，没有Sprite、SpriteAtlas、Flipbook，无法建立统一import/cook/install lifecycle。
5. Sprite没有PPU/pixel center、trim compensation、socket、render polygon、collision polygon、secondary texture或default material authority。
6. 全仓没有Flipbook/SpriteFrames/AnimatedSprite2D产品类型，也没有deterministic frame/event update或Scene persistence。

### 4.2 Sprite renderer、sorting、culling与GPU lifetime

1. material只随snapshot传递，graphics consumer没有读取它。
2. batch key只有texture ID，同texture不同material会错误合批。
3. Opaque/AlphaMask/Transparent共用固定alpha blend、depth write disabled pipeline，AlphaMask没有discard。
4. sorting只可靠表达`z_order + entity`；camera order、sorting layer、Y/custom axis、Canvas order没有authority。
5. `RenderSpriteBounds`只有定义和导出，没有生产consumer；visibility payload不含bounds。
6. `collect_render_sprites`在layer过滤后遍历和复制全部候选Sprite，culling没有在CPU tessellation前发生。
7. image slicing在CPU逐Sprite生成6的倍数vertices，单Sprite最多1,000 slices；这是防爆上限，不是GPU-friendly instance/chunk方案。
8. batch只合并相邻同texture项，每batch每帧创建新vertex buffer并开启独立render pass。
9. 游戏Sprite atlas没有generation、page residency、eviction/repack fence、device-loss恢复或frame-boundary atomic install。
10. 全仓没有Canvas2D、CanvasLayer、Camera2D、Light2D、SpriteMask或2D pixel-perfect产品authority。

### 4.3 Atlas source、compiler与Editor缓存

1. Atlas manifest只保存texture、page size、padding和name/source/rect/UV，缺schema version、stable SpriteId、source digest、compiler version、generation、platform variant、trim、rotation和border。
2. pack config只是进程内参数，不是可持久化recipe；没有依赖图、DDC key、stable/incremental layout或multipage。
3. pack/write API除自身tests外没有production caller，未进入Editor background job、progress、cancel、publication receipt或reimport。
4. artifact writer先保存PNG再写TOML，没有staging目录、双文件atomic publish、rollback或crash recovery。
5. padding只留空，不做extrude/dilate；没有trim、rotation、mip bleed、compression或secondary texture alignment。
6. resolver按目录扫描所有TOML后排序，首次miss仍有`read_dir + parse + decode`。
7. resolver在manifest循环内用`?`，首个候选不匹配就终止，多个atlas manifest时解析结果依赖字典序。
8. AtlasResolution cache按`source_key + source_path`缓存正负结果，任何atlas产品变化采用全局clear，没有generation-qualified增量替换。
9. RGBA cache使用单generation resource index，旧generation源像素不可并存寻址；eviction按最小路径而非LRU/成本/热度。
10. `copy_atlas_rgba`返回整页`Vec` clone，command compaction只保证同一frame同generation最多提取一次，不是零拷贝跨线程资源handoff。
11. 冷加载先解码原始单图并将`Arc<[u8]>`保存在visual asset cache，再解析atlas；recording虽丢弃单图payload，内存resident仍重复。
12. Editor10 failure仍应保持Open：warm cache静态路径已有明显进展，但没有instrumented gate证明steady-state 0 read_dir/stat/decode/clone，也没有immutable source-key index publication。

### 4.4 TileSet与TileMap schema

1. TileSet仍是单image、tile width/height和numeric tile ID列表。
2. per-tile metadata只有optional name和`Option<String> collider`，没有typed physics/nav/terrain/occlusion/custom/animation/material。
3. TileSet没有validate入口，unknown/duplicate tile ID、image grid和collider内容没有封闭校验。
4. TileMap仍是单TileSet、固定width/height、多层dense `Vec<Option<u32>>`。
5. cell没有source/atlas coordinate/alternative、transform/tint/variant/instance data。
6. layer name被paint helper临时当identity，但source schema没有serialized stable LayerId、redirect或rename policy。
7. projection只有枚举，没有cell/local/world、neighbor、bounds、picking及stagger/hex参数golden。
8. `validate_layers()`使用`width as usize * height as usize`，没有checked multiplication，也没有map/layer/cell/bytes总预算。
9. schema没有version、migration、unknown preservation、downgrade拒绝、sparse chunk或infinite map。

### 4.5 TileMap runtime与插件装配

1. runtime component只是动态`ComponentTypeDescriptor`，背后没有typed component storage或Scene mapper。
2. Tiled importer明确为`DiagnosticOnlyAssetImporter`，TMX/TSX/JSON没有parser、normalized IR、fidelity report或dependency graph。
3. 没有chunk compiler、shared mesh/instance/tile-data texture、dirty chunk update、bounds/culling或streaming。
4. 没有generation-qualified collision、navigation、occlusion artifact，也没有stroke-end批量rebuild。
5. builtin catalog列出`tilemap_2d`，first-party runtime catalog没有Tilemap2d provider branch。
6. first-party editor catalog只有Navigation和Neural分支，没有TileMap editor provider。
7. App只依赖catalog boundary，不直接依赖TileMap crate；catalog缺provider即意味着默认产品无法安装实现。
8. plugin manifest的`Partial`是真实状态，README所称runtime-backed authoring和已注册asset editor高于默认产品可执行事实。

### 4.6 TileMap Editor、preview与transaction

1. Import/Create TileMap/Create TileSet/Open/Paint五个operation只有descriptor/menu/payload schema，没有factory或controller。
2. `authoring.zui`与`tilemap_component.zui`不存在，view和inspector customization引用断链资源。
3. paint kernel直接修改`TileMapAsset`内存Vec，没有Editor command、before/after chunk diff、undo/redo、dirty/save、external conflict或recovery。
4. 4,096 cell上限和failure atomicity只封闭单次函数调用，不封闭document session、owner lease、job cancel或shutdown。
5. 没有palette、layer tree、selection、hover/pick、brush/stamp/line/rect/bucket/erase/eyedropper、preview overlay或projection-aware canvas。
6. 没有Sprite/Atlas/TileSet/TileMap专用toolkit、details panel、runtime artifact preview或isolated PreviewWorld。
7. built-in asset registry只给TileSet/TileMap名称、badge、icon和placeholder thumbnail，不提供toolkit；Texture才使用SourceImage thumbnail。
8. 全仓没有Sprite region/pivot/socket/render geometry/collision geometry编辑模式，也没有TileSet single-tile collision/nav/terrain editor。

## 5. 参考引擎对照

| 参考 | 当前源码事实 | Zircon应吸收的边界 |
|---|---|---|
| Unreal Paper2D | `PaperSprite`分离SourceTexture/BakedSourceTexture，保存PPU、trim/rotation、socket、render/collision geometry并集中rebuild；Atlas有GUID、padding策略、build state和incremental slots；TileMapComponent区分owned asset并支持集中collision rebuild；paint/erase/fill/terrain brush使用`FScopedTransaction + Modify()` | source/derived分层、stable identity、asset/component ownership、批量派生物重建、正式asset toolkit与transaction |
| Godot | cell identity由source/atlas coords/alternative组成；TileSet拥有multi-source、proxy、physics/navigation/occlusion/terrain/custom data；TileMapLayer用render/physics quadrant和dirty flags更新，并维护navigation/occluder/y-sort；Editor有pattern、line/rect/bucket/picker、preview与UndoRedo | typed cell identity、sparse quadrant、多域cook、projection-aware preview、统一undo与resource preview |
| Fyrox | Sprite以material为batch边界并在collect时做frustum判断；TileMap按data/tileset/tile source/collider/brush/autotile/property/update分模块；data为固定chunk sparse map；Editor command实现execute/revert，interaction mode拥有cursor/selection/update effects | Rust内typed resource、material-aware batching、chunk source、brush/macro与command边界 |
| Bevy | Sprite使用Image和TextureAtlasLayout handles，PostUpdate计算AABB并接visibility；renderer在Transparent2d phase建立batch range；TilemapChunk共享mesh，以tile-data image承载cells且只在`Changed<TilemapChunkTileData>`时更新 | typed handles、change detection、pre-render bounds、persistent batch range、共享chunk mesh与GPU cell update |
| Unity Graphics URP 2D | Renderer2DData表达layer mask、blend styles、sorting-layer texture与light/shadow texture budget；RenderGraph独立normal/light/shadow/sorting/pixel-perfect资源；tests覆盖Sprite instancing、Renderer2D与TilemapRenderer batching/culling | 2D RenderGraph、light/normal/shadow/mask/sorting、pixel-perfect contract、GPU预算和visual/runtime regression |

这些参考并非都完整。Paper2D自身标为beta且terrain留有TODO，Bevy没有同级Editor，Unity Graphics仓重点是renderer。应吸收其被源码和tests证明的owner、identity与dataflow，不复制历史包袱，也不能用“参考实现也不完整”降低Zircon门槛。

## 6. P0当前状态

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | Open | Project Scene load固定Sprite/Mesh2D None，save不表达Sprite且固定TileMap None | 先建立versioned Sprite/TileMap source mapper、unknown preservation、missing-provider policy与roundtrip corpus |
| P0-2 | Open | TileMap只有asset kind、dynamic descriptor与DiagnosticOnly importer | 冻结typed Runtime owner，贯通artifact install、chunk render、collision/nav/occlusion和streaming |
| P0-3 | Open | 五个operation无factory/controller，两份ZUI不存在，贡献仍可被声明 | backend/resource/factory/capability任一缺失时整体拒绝贡献，接入document/transaction/job/receipt |
| P0-4 | Open | Sprite/Atlas/Flipbook不是ResourceKind，Atlas pack/write无production caller | 建立typed source、recipe、compiler、artifact、streamer与toolkit，不再以Editor cache充当资产系统 |
| P0-5 | Open | TileSet单image/string collider，TileMap单tileset/dense numeric cells | 设计schema v2、stable identities、typed data layers、sparse chunks、migration与预算 |

## 7. P1逐项状态

| ID | 状态 | 当前证据与剩余重构 |
|---|---|---|
| P1-01 | Open | 无正式`SpriteAsset`、stable local ID、source revision或default material authority。 |
| P1-02 | Open | pack config不是可持久化recipe，source/settings/compiler/platform key不存在。 |
| P1-03 | Open | atlas entry仍以display name引用；多manifest resolver首项不匹配即终止。 |
| P1-04 | Open | manifest缺dependency digest、compiler version、generation、trim/rotation和platform variant。 |
| P1-05 | Open | PPU、pixel center、origin与trim compensation没有共享数学authority。 |
| P1-06 | Open | socket、render/collision polygon、secondary textures、material slots与nine-slice source缺失。 |
| P1-07 | Open | TileSet仍单image/numeric ID，无multi-source、stable TileId或proxy。 |
| P1-08 | Open | per-tile只有name/string collider，未类型化physics/nav/terrain/custom/animation/material。 |
| P1-09 | Open | cell是`Option<u32>`，无alternative、transform、tint、variant seed或instance data。 |
| P1-10 | Open | projection只有枚举，无转换、邻接、bounds、pick和stagger/hex参数golden。 |
| P1-11 | Open | dense Vec随完整矩形增长，无sparse chunk/infinite map与cell/chunk/bytes admission。 |
| P1-12 | Open | Sprite/Atlas/TileSet/TileMap无显式schema version、upgrader和unknown preservation。 |
| P1-13 | Partial | generic World transport可保留Sprite/Mesh2D，Project Scene仍确定性丢Sprite/TileMap。 |
| P1-14 | Open | TileMap plugin component只是descriptor，无typed component、artifact handle或override policy。 |
| P1-15 | Open | 无TileMap renderer、chunk mesh/instance/tile-data texture与dirty update。 |
| P1-16 | Partial | Sprite visibility membership和bounds DTO存在，bounds未进入visibility且TileMap bounds不存在。 |
| P1-17 | Open | material handle不进入shader/binding/pipeline/batch key，secondary texture也无owner。 |
| P1-18 | Partial | 三个phase标签存在，仍共享alpha blend/depth-off pipeline，Mask无discard。 |
| P1-19 | Partial | `z_order + entity`稳定，sorting layer/Y/custom axis/camera/canvas order缺失。 |
| P1-20 | Partial | 相邻同texture batch和stats存在，仍为每batch每帧buffer/pass且batch key错误。 |
| P1-21 | Open | 游戏Atlas page无streaming generation、eviction、repack fence或GPU lifetime；Editor generation不能替代它。 |
| P1-22 | Open | Sprite/Atlas/Flipbook/TileSet/TileMap无stage/validate/frame-boundary atomic install receipt。 |
| P1-23 | Open | 无Canvas2D/CanvasLayer hierarchy、clip/modulate/screen-space authority。 |
| P1-24 | Open | 无Camera2D/pixel-perfect product、limits/reference resolution/safe area。 |
| P1-25 | Open | 无可执行Sprite/Atlas/TileSet/TileMap document session/details/viewport/save/reimport toolkit。 |
| P1-26 | Open | texture import无single/grid/automatic/manual slicing recipe、stable diff和preview。 |
| P1-27 | Open | Atlas pack/write无production caller，也未进入job admission/progress/cancel/publication。 |
| P1-28 | Partial | packer有padding、deterministic input和max size；trim/rotation/extrude/dilate/multipage等缺失。 |
| P1-29 | Open | 无stable/incremental layout、page/UV diff、waste统计和基于stable ID的引用修复。 |
| P1-30 | Open | 无Sprite source region、pivot/socket/render/collision/nine-slice编辑模式。 |
| P1-31 | Open | 无TileSet source/alternative/data layer/animation/terrain/collision/nav/occlusion editor。 |
| P1-32 | Open | 无projection-aware canvas、layer tree、palette、selection、pick和visible chunk culling。 |
| P1-33 | Partial | bounded atomic paint kernel可作commit底座，brush/line/rect/fill/erase/picker/preview均不存在。 |
| P1-34 | Partial | 唯一layer name可抗数组reorder，仍无serialized stable LayerId、rename/clipboard/lock/remap。 |
| P1-35 | Open | paint直接修改Vec，无command merge、chunk diff、undo/redo、dirty/save/conflict。 |
| P1-36 | Open | 无isolated PreviewWorld、真实runtime artifact preview或Editor/PIE共享renderer。 |
| P1-37 | Open | 无Sprite collision geometry source、validation、simplification或physics cook artifact。 |
| P1-38 | Open | string collider无backend consumer，无dirty chunk collider build与stroke-end批处理。 |
| P1-39 | Open | 无per-tile navigation/occluder/custom data和generation-qualified chunk cook。 |
| P1-40 | Open | 无terrain set、neighbor mask、weighted alternative与deterministic solver。 |
| P1-41 | Open | 无Pattern/Brush Macro/WFC source、compiler、budget、seed或cancelable job。 |
| P1-42 | Open | ResourceKind/ImportedAsset/marker无Flipbook，frame/duration/event/dependency合同不存在。 |
| P1-43 | Open | 无AnimatedSprite runtime component、deterministic update phase与Scene persistence。 |
| P1-44 | Open | 未复用Editor14 timeline/onion skin/frame strip/event marker owner。 |
| P1-45 | Open | 无stable socket identity、frame pose attachment、redirect和diagnostic。 |
| P1-46 | Open | 固定Sprite shader只采主texture；Light2D/normal/mask/secondary texture无产品合同。 |
| P1-47 | Open | Scene NodeKind/create/picking无Sprite2D/TileMap/Canvas2D，缺真实geometry/cell hit。 |
| P1-48 | Partial | queue stats可见batch/pass/slice/vertex；缺grid/chunk/bounds/overdraw/cook generation质量视图。 |
| P1-49 | Open | TMX/TSX/JSON importer仍是DiagnosticOnly，无解析和fidelity report。 |
| P1-50 | Open | 无TMX到TSX/image/template依赖图、digest、stable subasset和incremental reimport。 |
| P1-51 | Partial | registry与DiagnosticOnly降级诚实；normalized 2D IR和Aseprite/TexturePacker/LDtk策略缺失。 |
| P1-52 | Partial | package/native dist和builtin descriptor存在；first-party catalogs/App可执行provider装配缺失。 |
| P1-53 | Open | 五个operation无factory、payload validation、document scope、owner lease或terminal receipt。 |
| P1-54 | Open | 两份ZUI不存在，admission没有把断裂贡献整体拒绝。 |
| P1-55 | Partial | Atlas校验、generation UI payload、paint diagnostics存在；缺typed journal、provenance和repair action。 |
| P1-56 | Partial | Atlas局部checked校验和4,096-cell cap存在；map multiplication、layers/parser/geometry总预算缺失。 |
| P1-57 | Partial | pack对同序输入确定；无dependency/settings/compiler/platform key与byte-identical DDC。 |
| P1-58 | Partial | Sprite queue和paint/stream stats存在；缺cull ratio、residency、cook/reimport时间及generation provenance。 |
| P1-59 | Partial | 123个选取范围test declarations覆盖局部几何、cache、pack、queue和paint；核心roundtrip/render/cook矩阵缺失。 |
| P1-60 | Open | 无Windows/Linux/macOS与目标GPU的2D visual/performance/release qualification。 |

## 8. P2当前状态

| ID | 状态 | 当前结论 |
|---|---|---|
| P2-01 | Open | 无2D skeletal deformation、deform bounds、GPU skinning与SpriteMask交互。 |
| P2-02 | Open | 无Sprite Shape/spline terrain source和render/collision派生链。 |
| P2-03 | Open | 无palette swap、lookup texture、per-instance parameter block与batch-compatible variant。 |
| P2-04 | Open | 无高级hex/isometric elevation、height layer、custom sort axis和projection-aware navigation。 |
| P2-05 | Open | 无World Partition region/chunk async streaming、HLOD和minimap artifact。 |
| P2-06 | Open | 无runtime copy-on-write overlay、save delta、replication/prediction/rollback。 |
| P2-07 | Open | 无procedural tile rule graph、deterministic compile、局部求解与debug trace。 |
| P2-08 | Open | 无chunk/cell协作operation、stable layer merge/lock/conflict/provenance。 |
| P2-09 | Open | 无基于runtime telemetry的可复现Atlas热度布局建议与recipe approval。 |
| P2-10 | Open | 无duplicate/unused/missing/waste/overdraw/seam分析和transactional repair。 |
| P2-11 | Open | 无GPU cull、indirect/multi-draw、bindless page或chunk compaction资格。 |
| P2-12 | Open | 无跨Paper2D/Godot/Fyrox/Bevy/Unity Graphics统一任务与性能基准。 |

## 9. Gate当前状态

| Gate | 状态 | 当前证据缺口 |
|---|---|---|
| G01 Scene Sprite roundtrip | Fail | Project Scene没有Sprite source字段，load固定None。 |
| G02 Scene TileMap roundtrip | Fail | load不安装TileMap，save固定None。 |
| G03 Schema migration | Fail | 无v2 schema、migration corpus与downgrade拒绝。 |
| G04 Stable identity | Fail | Sprite/Atlas entry/Tile/layer没有完整stable identity和redirect。 |
| G05 Atlas atomicity | Fail | PNG先写、TOML后写，失败/取消/崩溃可见半artifact。 |
| G06 Atlas deterministic | Partial | pure pack输入顺序可确定；clean/DDC/remote DDC和artifact hash未建立。 |
| G07 Atlas quality | Fail | trim/rotation/extrude/dilate/mip/secondary/platform golden缺失。 |
| G08 TileSet validation | Fail | TileSet无validate，string collider和unknown tile未封闭。 |
| G09 Projection golden | Fail | 无转换、邻接、bounds、pick的golden/fuzz。 |
| G10 Sparse scale | Fail | source随完整矩形增长，无occupied-chunk证据。 |
| G11 Sprite material | Fail | 同纹理不同material仍错误合批并使用同一shader。 |
| G12 Alpha semantics | Fail | 三阶段共享固定alpha blend/depth-off，Mask无discard。 |
| G13 Sorting | Partial | z-order/entity tie-break存在；layer/y-axis/camera/canvas/cross-2D-3D矩阵缺失。 |
| G14 Sprite culling | Fail | bounds不进入visibility，离屏Sprite仍生成vertices。 |
| G15 TileMap chunk renderer | Fail | renderer/chunk/update path不存在。 |
| G16 GPU lifetime | Fail | 游戏Atlas generation/eviction/device-loss/surface-recreate合同不存在。 |
| G17 Collision cook | Fail | typed geometry与dirty chunk physics build不存在。 |
| G18 Navigation/occlusion | Fail | generation-qualified nav/occlusion artifact不存在。 |
| G19 Flipbook determinism | Fail | Flipbook asset/runtime/event update不存在。 |
| G20 Editor transaction | Fail | bounded atomic mutation不是undoable Editor transaction。 |
| G21 Conflict handling | Fail | 无本地dirty与external reimport三方diff。 |
| G22 Tiled fidelity | Fail | backend未安装，corpus字段未解析。 |
| G23 Reimport dependency | Fail | 无TSX/image/template依赖触发和stable diff。 |
| G24 Plugin admission | Partial | capability Partial和DiagnosticOnly状态诚实；缺ZUI/factory时贡献仍可声明。 |
| G25 Job cancellation | Fail | Atlas/Tiled/fill/autotile/cook均无正式job/cancel terminal。 |
| G26 Malformed/fuzz | Partial | Atlas和paint有checked局部校验；无全格式fuzz与完整budget。 |
| G27 Performance telemetry | Partial | Sprite queue与UI stream局部统计存在；cull/upload/residency/chunk provenance缺失。 |
| G28 Large-scene baseline | Fail | 无十万Sprite/百万cell/千visible chunk可复现预算。 |
| G29 Visual matrix | Fail | 无lit/mask/normal/sort/dilate/secondary/preview图像矩阵。 |
| G30 Cross-platform | Fail | 无多平台texture/pixel-center/precision证据。 |
| G31 Headless cook/package | Fail | shipping没有正式Sprite/Atlas/TileMap cook/install链。 |
| G32 Truthful maturity | Partial | Runtime capability为Partial；README和Editor贡献仍高估可执行authoring。 |

## 10. 目标Owner与数据流

1. Runtime Interface只拥有跨进程稳定identity、source/artifact reference、generation、diagnostic与receipt DTO。
2. Runtime Asset拥有Sprite/Atlas/Flipbook/TileSet/TileMap source schema、migration、normalized import IR、dependency graph、compiler key与immutable artifact schema。
3. Runtime Scene拥有Sprite2D/AnimatedSprite/TileMap/CanvasLayer typed components、source mapper、runtime overrides与generation-qualified install。
4. Graphics拥有material-aware Sprite pipelines、persistent instance/ring buffers、bounds/culling、TileMap chunk renderer、2D light/mask/normal与GPU lifetime。
5. Physics/Navigation消费同一artifact generation，产出可追踪的collision/navigation/occlusion chunk artifacts，不解析Editor source。
6. Editor拥有transactional documents、toolkits、projection-aware canvas、selection、commands、preview、job orchestration和conflict/recovery。
7. Plugin只有在backend、resources、factories和capabilities全部可用时才贡献入口，descriptor数量不能代表功能完成度。

```text
Texture/Image Source
  -> Sprite Import Recipe
  -> Sprite Source (SpriteId, pivot, trim, geometry, collision, material inputs)
  -> Atlas/Flipbook Compiler
  -> Generation Artifact Set (pages, layout, animation, diagnostics)
  -> Scene Install
  -> Bounds/Visibility + Material Pipeline
  -> Sprite Renderer

Tiled/Aseprite/TexturePacker/Native Source
  -> Normalized 2D IR + Dependency Graph
  -> TileSet Source (stable sources/tiles/alternatives/data layers)
  -> Transactional TileMap Document (stable layers + sparse chunks)
  -> Chunk Cook (render/collision/navigation/occlusion)
  -> Generation-qualified Scene Install
  -> Streaming Renderer and Debug/Preview
```

## 11. 重构顺序

| 层 | 必须完成的工作 | 退出条件 |
|---|---|---|
| L0 Truthful admission | 缺provider/factory/ZUI时不注册TileMap贡献，README和maturity与产品一致 | G24、G32不再依赖人工解释 |
| L1 Schema与identity | Sprite/Atlas/Flipbook/TileSet/TileMap v2、stable IDs、migration、budgets | G03、G04、G08、G09、G10通过 |
| L2 Scene roundtrip | typed Sprite2D/AnimatedSprite/TileMap/CanvasLayer mapper与unknown preservation | G01、G02通过 |
| L3 Compiler/artifact | recipe、dependency key、DDC、atomic publish、generation install | G05、G06、G07、G16、G31通过 |
| L4 Runtime render | material/alpha/sort/bounds/culling、persistent Sprite batching、TileMap chunks | G11-G15通过 |
| L5 Derived domains | collision/nav/occlusion cook与generation fence | G17、G18通过 |
| L6 Editor product | toolkits、factory/controller、canvas/tools、transaction/save/conflict/preview | G19-G25通过 |
| L7 Qualification | fuzz、telemetry、large scene、visual/cross-platform/device-loss matrix | G26-G30通过 |

不要先补漂亮的TileMap workspace或更多descriptor。P0 Scene数据丢失、schema v2、typed runtime owner和truthful admission未完成前，UI越多只会扩大假完成面。

## 12. Failure与验收边界

| Failure | 当前状态 | 本报告处理 |
|---|---|---|
| Editor10 Atlas paint-time I/O | Open | warm cache和generation payload记为局部进展；缺immutable index、multi-manifest正确性与动态0-I/O证据，不关闭 |
| Plugins10 TileMap scene-mode factories | Open | paint kernel存在，但factory/controller/ZUI/provider仍缺，不关闭 |
| Runtime Render13 UI payload duplication | Open | 静态generation/compaction路径已增强；managed WGPU/RenderDoc证据仍缺，不关闭 |

实现阶段至少需要以下可复现证据：

1. Sprite/TileMap project Scene byte roundtrip、unknown provider三态与migration corpus。
2. 同texture不同material、Opaque/Mask/Blend、sorting/canvas/Y-axis与offscreen culling golden。
3. Atlas多manifest解析、stable layout、multi-page、atomic publish、cancel/crash/device-loss与generation overlap。
4. TMX/TSX/JSON fidelity corpus、dependency reimport、malformed/fuzz和size/admission预算。
5. TileMap brush/line/rect/fill/erase/picker的preview/commit parity、undo/redo、save/conflict/recovery。
6. collision/navigation/occlusion dirty chunk rebuild及generation stale reject。
7. 十万Sprite、百万cell、千visible chunk的CPU/GPU/upload/residency预算。
8. Windows/Linux/macOS和目标GPU visual/runtime matrix。

## 13. 本轮边界

本报告不实现生产代码，也不把静态tests数量当作功能完成度。所有结论均来自当前磁盘源码和本地`dev/`参考源码。后续实施前必须重算指纹、复核共享dirty worktree，并从L0/L1/L2开始按依赖顺序推进。
