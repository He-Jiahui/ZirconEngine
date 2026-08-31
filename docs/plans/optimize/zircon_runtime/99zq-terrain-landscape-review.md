---
title: Runtime Terrain / Landscape / Heightfield / Clipmap / Quadtree / LOD / Material Layer / Virtual Texture / Foliage / World Partition / Physics / Navigation / Editor 当前源码复审
category: zircon_runtime
report_id: Runtime142
review_date: 2026-08-25
baseline_head: 3af73550dd00fe4805f71e96ce199f4ab633687f
baseline_epoch: 424
verification_head: 3af73550dd00fe4805f71e96ce199f4ab633687f
verification_epoch: 424
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/terrain
  - zircon_plugins/physics/runtime/src/backend/jolt
  - zircon_plugins/navigation/runtime/src/manager/bake/geometry.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - examples/vampire
  - examples/woc
plan_sources:
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/zircon_runtime/render/15-terrain-vegetation.md
  - docs/plans/zircon_runtime/render/15/2026-07-09-terrain-vegetation-output-records.md
  - docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/godot/scene/resources/multimesh.cpp
  - dev/godot/modules/jolt_physics/shapes/jolt_height_map_shape_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing/Common/TerrainToMesh.cs
  - dev/bevy/crates/bevy_render/src/view/visibility
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime142 · Terrain / Landscape 当前源码复审

## 1. 结论

当前 Zircon 没有可执行的 Terrain / Landscape 系统。仓库已有 `TerrainAsset`、`TerrainLayerStackAsset`、`SceneTerrainAsset`、TOML importer、artifact store/load、插件 registration、Editor import plan、`BuiltinRenderFeature::Terrain` 枚举、Physics HeightField DTO 和示例 terrain 文件；这些是可以保留的 carrier 与局部验证骨架，不等于 World 中存在 Terrain instance，更不等于 renderer、collision、navigation、foliage 或 World Partition 已经消费同一份地形真相。

最关键的断点位于 source 到 product 的纵链。Terrain source 插件没有进入 first-party runtime/editor catalog，`zircon_app` 也没有 Terrain feature 或默认选择；Editor registration 引用的三份 `plugins://terrain/...` 资源物理不存在。即使绕过产品入口直接注册插件，runtime importer 仍是 `DiagnosticOnlyAssetImporter`，dist 明确为 stateless 且没有 command/event/bridge。Editor 的 Import/Create/Open/Sculpt 只有 descriptor；通用 authoring batch 只注册 command metadata，不注册 operation factory，最终会落入 `MissingFactory`。

Scene 文档层已经比旧报告前进了一步：正式 `SceneAsset` schema、serializer、reference resolver、artifact cache 和 asset management 能保留 `SceneTerrainAsset` 引用。因此“Terrain 引用在所有保存链都丢失”已经不准确。但运行时 `World::from_scene_asset` 不消费 `entity.terrain`，`World::to_scene_asset` 在 `scene_asset.rs:599` 固定写 `terrain: None`；`SceneNode` 也没有 Terrain runtime component。当前只能判定“文档 carrier Partial，World execution Open”。

Graphics 没有 Terrain extract、patch geometry、LOD、material layer 或 pass executor。Terrain 被明确放在 `DESCRIPTOR_ONLY_ADVANCED_SLOTS`，只有 extract section 名称而没有 stage pass；测试还要求这类 slot 不产生 runtime pass。这是诚实的未完成标记，不是渲染能力。Vampire 示例的可见地表来自 `jungle_terrain.model.toml` 普通 mesh；同一 entity 上的 `TerrainAsset` 引用不会生成像素，不能把普通 mesh draw 或帧率归因于 Terrain runtime。

Physics 与 Navigation 只证明相邻基础设施存在。Jolt 路径会校验 HeightField 分辨率、样本数与有限值，但随后把每个 cell 展开成两个 triangle shape，再装进 static compound；它没有使用 Jolt 原生 HeightFieldShape，也没有接 `TerrainAsset`、sample spacing、height scale、hole、physical material、tile generation 或增量更新。Navigation 的 geometry collector 对 `TriangleMesh | HeightField` 明确不产出几何，并声称由 owning asset bake path 处理；仓库里没有对应 Terrain bake adapter。因此 terrain、collision、nav 三者不是同 generation 的表面。

World Building Workbench 仍在发布固定结果：Summit Valley、64 cells、84K instances、128 clusters、Cell A12/A13、96 cells 和 96 MB 等均来自硬编码 feedback，没有 document、job、artifact、runtime snapshot 或 cancellation ticket。production 也没有 World Partition manifest、stable cell、streaming source、data layer、HLOD builder 或 cell residency authority。继续增加按钮会扩大伪产品面，必须先做 capability truth 和 authority 硬切。

参考源码说明差距不是“还少一个 quadtree”。Unreal 本轮相关 Landscape/Foliage/WorldPartition 目录合计远超十万行，核心是 componentized source、edit layer、texture streaming、collision cook、grass/HISM、Nanite/HLOD、cell/source/data-layer 状态机和跨域失效闭环；Fyrox 仅 8 个 Terrain Rust 文件也已经形成真实 Terrain node、chunk heightmap、quadtree LOD、raycast、brush thread 与可撤销 chunk swap。Zircon 目前连 Fyrox 级最小可执行闭环都没有，更没有证据讨论“性能和表现优于 Unreal”。

本轮登记 **5 项 P0，全部 Open；72 项 P1，其中 62 Open、10 Partial；14 项 P2，全部 Open；36 项资格门为 31 Fail、5 Partial、0 Pass**。本报告只刷新事实、owner 边界和分层重构顺序，不修改生产代码。

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

统计口径为 working-tree 物理行、非空行、bytes、Rust `#[test]` 和 `#[ignore]` 声明。fingerprint 对 repository-relative lowercase path 排序，为每个文件拼接 `path + NUL + lowercase(file SHA-256) + LF` 后再取 SHA-256。产品 consumer 和参考集合是本轮明确选择集，不代表对应仓库总规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Terrain 插件全量 | **16 / 1,072 / 971 / 38,904 / 10 / 0** | `0969623ed6793b55e014d0b50cced3b8e697e045b79f685684290b008d309366` |
| Runtime asset 与 Scene carrier | **17 / 4,806 / 4,549 / 193,510 / 16 / 0** | `494c1a91f7a324eaf41c8edfb3d757abca7dae321878f46373abcb8c4e3c266c` |
| Render / Physics / Navigation consumer | **28 / 10,194 / 9,549 / 355,811 / 65 / 0** | `59ede2f75d237de0c12bb73e1ae372a31c4f167abff86d915b81ac8963f73a33` |
| Catalog / App / Editor / 示例产品纵切面 | **59 / 15,241 / 14,051 / 542,248 / 26 / 0** | `094926f913759df01ddb1f365485f9c638f33c0963088613197319454f0fedce` |
| Zircon selected union | **120 / 31,313 / 29,120 / 1,130,473 / 117 / 0** | `9c7765ca51f5deb8def3711d8f05f2dae8d6b29cf3f36c45fb7c76834f5b9d63` |
| Unreal selected | **20 / 44,641 / 38,010 / 1,757,513 / 0 / 0** | `3b3763504aaeef29f72857c12feada30506e70b45fca3243339034528b4f0333` |
| Fyrox selected | **8 / 6,437 / 5,990 / 257,149 / 28 / 0** | `b46b78946ea114aa774fba57e3dc9d2e329c341e6720853df66ef7fdef9a4374` |
| Godot selected | **10 / 2,048 / 1,647 / 81,231 / 0 / 0** | `af0b3fd5e07bd58c2dbd5ee898aac9577e6c059b19fe5a7fe7558ab7cedea907` |
| Unity Graphics selected | **12 / 4,467 / 3,784 / 204,991 / 0 / 0** | `2054a0f186f8f8f57eed4f97fbbea9055e05022c82990d0b625c4ef0d9be7d33` |
| Bevy selected | **4 / 8,957 / 8,204 / 359,695 / 3 / 0** | `4d272f0dcd50326b0a47410e9f5d512cdd76b681aad3d04f0a5c509f1db7d9fd` |
| 五引擎参考选择集 | **54 / 66,550 / 57,635 / 2,660,579 / 31 / 0** | `868f011cc0b35e9aafbcccc9f3429c02214509b9798ffb028343d60950035bc3` |
| all selected | **174 / 97,863 / 86,755 / 3,791,052 / 148 / 0** | `ab3098cffc64d5392e9f35bbb837e475dee6a057d6324b83d7dee46d7ce65575` |

Unreal 还做了目录级规模核对：Runtime Landscape 为 **143 文件 / 88,524 行**，LandscapeEditor 为 **89 / 40,444**，Runtime Foliage 为 **44 / 12,418**，FoliageEdit 为 **49 / 12,138**，WorldPartition Public/Private 合计 **329 / 70,568**。这些数字只证明审查范围与系统复杂度，不作为代码质量或性能结论。

### 2.2 currentness 与限制

- source baseline 和 verification HEAD 均为 `3af73550dd00fe4805f71e96ce199f4ab633687f`，coordinator epoch 424。
- selected 文件包含用户或其他 Session 的 working-tree 修改；本轮不覆盖、不回退，也不声称它们已被集成。
- 全文读取 canonical owner 与关键参考文件；产品目录和 WOC 脚本按冻结文件集做符号、调用与 carrier 扫描。没有把文件数写成“功能通过”。
- 本轮未运行 Cargo、native build、App、Editor、PIE、asset cook、GPU capture、Jolt/Recast 动态场景、streaming、fault、scale、soak 或 benchmark。
- Tooling 按用户要求不在本轮优化范围；未来 Rust tooling 迁移不改变 Runtime/Editor owner 判定。

## 3. 当前产品链事实

| 链路 | 当前事实 | 判定 |
|---|---|---|
| Package -> Catalog -> App | `terrain/plugin.toml` 可被静态 manifest test 发现，但 runtime/editor first-party catalog 与 `zircon_app` 都没有 Terrain provider feature/branch | Open |
| Editor operation | Import/Create/Open/Sculpt descriptor 存在；三份资源缺失；authoring batch 不注册 factory，dispatch 可返回 `MissingFactory` | Open |
| Source import | Editor 能检查扩展名、零尺寸、checked sample count，并拒绝无 layer semantics 的 LayerStack；不读 bytes、不解码 PNG/RAW/R16、不写 artifact | Partial |
| Runtime import | builtin TOML importer 与 artifact store/load 存在；Terrain source plugin importer 固定 DiagnosticOnly | Partial |
| Scene document | `SceneAsset`、serializer、resolver 与 artifact cache 能保存 Terrain reference | Partial |
| Runtime World | load 不创建 Terrain component/service，save 固定 `terrain: None` | Open |
| Graphics | Terrain 是 descriptor-only slot，零 runtime pass、零 extract consumer | Open |
| Physics | 独立 HeightField DTO/Jolt validation 存在；转 triangle compound，且无 Terrain projection | Partial |
| Navigation | Recast plugin 存在；Terrain/HeightField geometry 没有 owning bake adapter | Open |
| Foliage | 示例为普通 mesh/static batch；无 prototype/scatter/cluster runtime | Open |
| World Partition | 无 manifest/cell/source/data-layer/HLOD/residency owner | Open |
| Product evidence | Workbench 固定反馈；Vampire 由 baked mesh 可见；无 source-to-pixel/collision/nav/streaming 证据 | Open |

## 4. 必须保留的真实底座

1. `TerrainAsset`、`TerrainLayerAsset`、`TerrainLayerStackAsset` 的 typed serde carrier 与 direct reference 枚举。
2. builtin Terrain TOML importer、artifact cache/store/load、reference resolver 与正式 `SceneAsset` Terrain 引用保存。
3. Editor heightfield import request、source format 枚举、checked sample count、canonical extension 和 LayerStack fail-close。
4. plugin capability/registration manifest 的结构，以及 descriptor-only render slot 对“未执行”的诚实表达。
5. Framework/Jolt HeightField 的尺寸、样本数和 finite validation；实现必须替换 backend representation，不能删除验证。
6. Navigation 已有 Recast/Detour 基础设施；Terrain 只增加 canonical geometry/artifact adapter，不再创建第四套 nav runtime。
7. 普通 mesh、material、asset residency、GPU visibility/indirect、operation service、transaction/history 等共享底座；Terrain 必须接入这些 owner，不能私建平行简化版。

## 5. P0：产品虚假可达或 authority 断裂

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| TER2-P0-01 | Open | source Terrain 插件不在 first-party runtime/editor catalog 与 App target composition，三份 `plugins://terrain/...` 资源缺失 | publication 前原子验证 package、resource、provider、factory、target capability；普通 Client/Editor 有同一 selected provider receipt |
| TER2-P0-02 | Open | Import/Create/Open/Sculpt 只有 metadata，没有 operation factory、document owner、transaction 或 scene-mode lifecycle | Terrain AuthoringGateway 提供 typed prepare/apply/cancel/undo/save 与 mode teardown；factory 缺失时入口不可见且给 typed reason |
| TER2-P0-03 | Open | runtime plugin 为 DiagnosticOnly，World 无 Terrain owner，Graphics 零 pass；注册 component metadata 却无行为 | 建 per-World generation-bound Terrain service、runtime component、artifact residency、query snapshot 和 renderer provider |
| TER2-P0-04 | Open | Terrain/Foliage/Scatter/Level Streaming Workbench 显示固定构建、实例、cell、memory 和 success feedback | 立即标为 Prototype/Unavailable 或隐藏；只有真实 job ticket、generation result、runtime snapshot 才能显示 queued/completed/statistics |
| TER2-P0-05 | Open | 没有 transactional world document/partition authority，且 World export 会把 Terrain 写成 `None` | 未完成无损 roundtrip 前 fail-close；建立 authoring document -> partition manifest -> runtime cell state 的唯一 authority 与 atomic attach/detach |

## 6. P1：Package、Source、Runtime、Render 与大世界差距

### 6.1 Package、capability 与产品装配

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-001 | Open | static manifest 可发现 Terrain，但 first-party catalog/App 无 provider branch | `TerrainActivationPlan` 由 project/target/profile 编译，App 只提交 selected provider，不从文件存在性推断能力 |
| TER2-P1-002 | Partial | capability 仅为 generic `Partial`，没有 render/query/collision/nav/editor facet 与限制 | 发布 facet-level capability、backend、limits、artifact versions 与 evidence receipt，禁止一个 Partial 掩盖全部缺口 |
| TER2-P1-003 | Open | registration 前不解析三份资源，也不校验 template/inspector/default document schema | registration staging 先完成 bounded resource resolve/read/parse，再一次性 publish 或完整 rollback |
| TER2-P1-004 | Open | command descriptor 与 factory 分属不同系统，通用 batch 只注册 metadata | contribution batch 必须携带 factory/document/build/preview provider，或使用同 owner lease 的原子二阶段注册 |
| TER2-P1-005 | Open | dist 为 stateless、零 command/event/bridge，却可与“runtime-backed”文案同时发布 | 明确 manifest-only package 与 executable provider；若保留 native dist，必须有 versioned ABI methods、state、unload 与 failure receipt |
| TER2-P1-006 | Open | importer、component、operation、toolkit、menu、inspector 没有共享 publication transaction | 一个 registration generation 管理全部贡献点，任一点失败不留半注册入口 |
| TER2-P1-007 | Open | plugin disable/reload 没有 open document、active stroke、dirty artifact、resident instance 的终态 | 定义 cancel/flush/retain/read-only/rebind policy，并以 owner generation 拒绝 stale callback |
| TER2-P1-008 | Open | tests 只证明 registration/plan shape，README 继续声称 runtime-backed authoring/scene mode | 加 default product activation、resource resolve、operation execution、disable/reload 和 capability-doc consistency tests |

### 6.2 Source、import、cook 与 artifact

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-009 | Partial | Editor plan 已校验扩展、零尺寸、checked sample count，并显式拒绝 LayerStack | 保留 fail-close；将 request 扩展为 bytes/stream、stride、endianness、channel、signedness、scale/offset、no-data 与 decode budget |
| TER2-P1-010 | Open | RAW/R16/PNG 没有实际 decoder、header/color/bit-depth/gamma/interlace/row-order 检查 | canonical decoder 输出 normalized source tile 与 typed diagnostics，禁止将任意彩图或截断文件当高度场 |
| TER2-P1-011 | Open | source 没有尺寸推断政策、row pitch、trailing bytes、allocation/decoded-byte 上限 | 先验证 header 与预算，再分块 decode；所有 overflow、oversize、short read 有稳定 error code |
| TER2-P1-012 | Partial | core `TerrainAsset` 有 typed fields 与 sample-count check，但使用 unchecked `usize` 乘法并允许零维、空 samples、NaN/Inf、非法 spacing/scale | 建 versioned source validator，所有维度/数值/reference/limit 在 artifact admission 前 fail-close |
| TER2-P1-013 | Open | layer 只有 name/material/weightmap/strength，无 stable ID、blend、channel、UV、physical material、edit policy | 发布 `TerrainLayerId` 与 versioned layer schema，明确 layer ordering、blend、visibility/lock 和 target channel |
| TER2-P1-014 | Open | LayerStack 无 validator、weight normalization、dimension/channel alias、cycle 与 migration | compiler 生成 canonical layer packing、active-layer set 和逐 patch diagnostics；坏组合不能延迟到 shader |
| TER2-P1-015 | Partial | 通用 artifact cache/store/load、direct reference 与 Terrain categories 已存在 | 硬切 `TerrainSourceAsset`、`TerrainBuildArtifact`、`TerrainRuntimeInstance`，不得继续用一个 inline DTO 同时充当三层真相 |
| TER2-P1-016 | Open | inline `Vec<Real>` 不支持大地形 partial read、tile/page、compression、checksum 或 border | source/build artifact 按 chunk/page 存储 min/max/error/hole/normal/weight，支持随机访问与 bounded decompression |
| TER2-P1-017 | Open | artifact 没有 schema/builder/backend/profile/content hash、endianness、coordinate、quality 与 migration receipt | `TerrainArtifactKey` 必须覆盖 source revision、builder version、target capability/profile 和依赖 closure |
| TER2-P1-018 | Open | source/material/weightmap 的变化没有 dirty region 与跨 consumer 精确失效 | dependency graph 输出 changed tiles/pages，驱动 render/collision/nav/foliage/HLOD generation，而不是全量重建 |

### 6.3 Scene、World、runtime owner 与 query

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-019 | Partial | 正式 SceneAsset 文档链能保留 `SceneTerrainAsset` reference | 扩展最小 scene component policy，并建立 roundtrip/property/reflection/prefab/clone tests；不要复制 height samples |
| TER2-P1-020 | Open | `World::from_scene_asset` 忽略 `entity.terrain` | load 时解析 handle、提交 runtime ticket；坏引用、unsupported backend 和 budget denial 必须是 entity-scoped diagnostic |
| TER2-P1-021 | Open | `World::to_scene_asset` 固定 `terrain: None` | 完成前对含 Terrain runtime component 的 save/export fail-close；完成后无损保存 source reference 与 policy |
| TER2-P1-022 | Open | `SceneNode`/ECS 没有 typed Terrain component，plugin reflection metadata 不产生 storage/system | 建 canonical `TerrainComponent`、stable instance ID、world generation 与 query/reflection/property surface |
| TER2-P1-023 | Open | 没有 requested/preparing/resident/attached/retiring/failed runtime state | per-World service 管理 generation ticket、dependency、cost、cancel、device loss、world unload 与 terminal receipt |
| TER2-P1-024 | Open | Gameplay 无统一 sample height/normal/material、raycast、bounds 与 hole query | immutable `TerrainQuerySnapshot` 固定坐标、triangle diagonal、interpolation、boundary、hole 和 stale-generation 语义 |
| TER2-P1-025 | Open | edit/stream/source reload 无 affected-region propagation | one source generation 生成 render/physics/nav/foliage dirty sets；新 generation commit 前旧 snapshot 保持可用 |
| TER2-P1-026 | Open | transform、非均匀 scale、负坐标、grid origin、floating origin 与 shared border 未定义 | 使用 integer grid + local coordinates；origin shift 不改变 tile/patch/artifact identity，seam sample 只有一个 owner |

### 6.4 Geometry、LOD、culling 与 render execution

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-027 | Partial | Terrain slot 被明确标为 descriptor-only，测试阻止零 pass 被误当执行 | 保留 truth gate；provider 未安装时 compile 为 typed unavailable，而非同名空 feature |
| TER2-P1-028 | Open | 无 Terrain extract payload、executor、pass/resource dependency 与 output receipt | 建 immutable `TerrainRenderExtract`，只传 generation handles、visible patch decisions 和 material bindings |
| TER2-P1-029 | Open | 无共享 patch topology、chunk bounds、height/normal/hole GPU resource 与 draw packet | artifact compiler 生成可复用 topology、per-patch metadata、GPU pages 和 deterministic winding |
| TER2-P1-030 | Open | 无 quadtree、clipmap、CDLOD、screen-space error 或 virtualized geometry eligibility | 先交付可验证 quadtree/patch baseline，再以 provider 插件扩展 clipmap/Nanite-like route；两者共享 artifact/query truth |
| TER2-P1-031 | Open | 无 neighbor LOD constraint、stitch/skirt/geomorph 与 crack gate | 固定 level delta、edge ownership、morph policy，覆盖跨 chunk、负坐标、极端 scale 和 camera cut |
| TER2-P1-032 | Open | 无 patch frustum/HZB/occlusion culling、visibility reason 或 bounds update | 接 Runtime visibility 主数据，CPU/GPU route 输出相同 reason schema、capacity 与 overflow receipt |
| TER2-P1-033 | Open | 无 GPU patch compaction、indirect args、persistent allocation 与 CPU fallback | 接 Bevy/Unity 类 render-world preprocessing 与 shared allocator；fallback 保持同画质和同 identity |
| TER2-P1-034 | Open | height/weight/normal/page 不接 asset residency、priority、prefetch、eviction 与 budgets | geometry/material pages 共享 desired priority，分域记录 requested/resident/pinned/evictable bytes |
| TER2-P1-035 | Open | 无 depth/prepass/GBuffer/forward/picking/decal/reflection/GI eligibility matrix | 每个 pipeline 声明真实 pass、resource access、material permutation 与 skip reason |
| TER2-P1-036 | Open | 无 terrain shadow、motion vector、temporal history 与 LOD transition contract | component/patch generation 驱动 previous/current data，camera cut/device reset 有明确 history invalidation |
| TER2-P1-037 | Open | 无 ray tracing/path tracing representation；普通 mesh fallback 未定义来源与失效 | 参考 Unity TerrainToMesh/ray marching，构建 target-specific derived representation 与 quality/cost receipt |
| TER2-P1-038 | Open | 无 upload cancellation、retire fence、device loss/recovery、shutdown drain | 所有 GPU resource 绑定 owner generation 和 completion fence，失败保留 last-known-good 或 typed unavailable |

### 6.5 Material layer、surface、hole 与 virtual texture

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-039 | Partial | layer 已能引用 material/weightmap 并进入 direct reference closure | 在 compiler 中验证 kind、extent、format、channel、sampling、dependency cycle 和 target qualification |
| TER2-P1-040 | Open | strength/weight 没有 default layer、zero-sum、overweight、normalization 与 precision policy | 生成 deterministic normalized weights 和 clamp/renormalize diagnostics |
| TER2-P1-041 | Open | 无 splat channel packing、per-patch active layer、layer count limit 与 overflow policy | 按 target limits 编译 channel/page layout；超过上限时明确 bake/fallback/reject |
| TER2-P1-042 | Open | 无 height blend、normal/mask/AO/roughness、physical material 的统一 surface contract | 定义 `TerrainSurfaceData` 并接 Runtime material ABI，CPU/GPU/physics 使用同 layer identity |
| TER2-P1-043 | Open | 无 canonical hole mask 贯穿 depth/shadow/collision/nav/ray | 一个 hole artifact 驱动全部 consumer；LOD mip 与过滤不能产生视觉/碰撞不一致 |
| TER2-P1-044 | Open | 无 normal/tangent reconstruction、border sampling、sample spacing/height scale shader contract | 固定 gradient kernel、cell diagonal、world transform 与误差预算，并跨 CPU/WGSL/physics 验证 |
| TER2-P1-045 | Open | 无 macro variation、distance tiling、far basemap、runtime virtual texture 或 page feedback | 近景 layer 与远景 baked representation 分层；VT 为可选 provider，不得成为基础 Terrain 的前置死锁 |
| TER2-P1-046 | Open | 无 shader permutation budget、PSO warmup、layer combination key 与 image regression | cook 输出 permutation/warmup manifest；seam/layer/hole/shadow/motion golden 与性能门同时成立 |

### 6.6 Physics、height query 与 Navigation

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-047 | Partial | Framework/Jolt 有独立 HeightField DTO 与尺寸、样本数、finite validation | 由 Terrain artifact 生成 generation-bound physics view，删除可独立编辑的第二份高度真相 |
| TER2-P1-048 | Partial | Jolt 能把 HeightField 展开为可碰撞 triangle compound | 改用 Jolt 原生 HeightFieldShape/block/min-max/hole；mesh fallback 必须显式且报告 triangle/memory/build cost |
| TER2-P1-049 | Open | 没有 Terrain -> PhysicsMesh/shape 自动 projection、registration 与 world sync | Terrain runtime commit 原子注册 collision generation；unload/reload/rollback 不留 stale shape |
| TER2-P1-050 | Open | collision 不含 sample spacing/height scale/hole/layer physical material/simple-complex policy | profile 编译 collision resolution/material，并与 `TerrainQuerySnapshot` 的坐标和 diagonal 一致 |
| TER2-P1-051 | Open | 任意 edit/reload 只能全量替换，没有 dirty collision tile 与 last-good swap | affected region 生成 cancellable cook jobs，完成后在 physics safe point 原子替换 |
| TER2-P1-052 | Open | builtin physics 不支持 HeightField，却可能让 collider/能力表面继续存在 | backend admission 必须按 shape facet 拒绝，禁止注册成功后 ray/contact 静默返回空 |
| TER2-P1-053 | Open | Navigation render mesh 用单位顶面，TriangleMesh/HeightField 不产几何 | Terrain adapter 提交真实 height spans/triangles、holes、area/material 与 source generation，禁止占位 quad |
| TER2-P1-054 | Open | terrain edit/streaming 不驱动 dirty nav tile、cancel、attach 顺序 | collision commit、nav rebuild 与 world cell attach 使用同 source generation 和 terminal protocol |

### 6.7 Foliage、Scatter 与大规模实例

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-055 | Open | 无 FoliagePrototype、ScatterRule、instance artifact、AssetKind 或 runtime cluster | 建 versioned prototype/rule/cell artifact 与 stable IDs，和 Editor document 一一对应 |
| TER2-P1-056 | Open | 无 seed/cell/source revision/filter order 组成的 deterministic instance identity | 使用 counter-based RNG 与固定 tie-break，跨线程、平台、reload 得到相同结果 |
| TER2-P1-057 | Open | 无 density/slope/height/layer/mask/collision/exclusion/override schema 与手摆 ownership | compiler 固定 filter 顺序；generated/manual/override 分层，重建不覆盖用户意图 |
| TER2-P1-058 | Open | 无按 partition cell 的增量 scatter cache、dependency key 与 rollback | changed tile 只失效受影响 cell，构建失败保留 last-known-good generation |
| TER2-P1-059 | Open | 无 HISM/MultiMesh 类 cluster bounds、GPU instance buffer、LOD/cull/indirect draw | 接共享 instance/visibility/allocator 主数据，cluster/instance 双层 culling 与 overflow receipt |
| TER2-P1-060 | Partial | Vampire 已使用普通 mesh/static grass batch，证明通用 mesh/material 路径可作 oracle | 保留 baked mesh 作为 visual/perf oracle；不得把它改名为 foliage system，逐步替换为同画质 runtime clusters |
| TER2-P1-061 | Open | 无 impostor/crossfade/wind/bending/motion/shadow/ray/collision/nav/scalability coupling | prototype 编译 target eligibility，consumer 以 cell/generation 原子 attach/detach，不逐实例临时注册 |

### 6.8 World Partition、HLOD 与 residency

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-062 | Open | production 无 partition manifest、stable cell、grid policy、bounds/content ownership/version | Editor 从 authoring world 编译 versioned manifest；Runtime 只消费 manifest，不从 Workbench 字符串造 cell |
| TER2-P1-063 | Open | 无 camera/player/portal source、priority、hysteresis、prefetch、pin、data layer | desired-set compiler 合并多个 source，预算变化和 priority inversion 有确定终态 |
| TER2-P1-064 | Open | terrain/foliage/collision/nav/HLOD 没有 cell bundle dependency 与 atomicity | 一个 cell generation 记录各 artifact/cost；partial attach 失败执行完整 rollback |
| TER2-P1-065 | Open | 无 IO/decompress/deserialize/GPU upload/world attach 状态机与分域预算 | 实现 requested/loading/preparing/attaching/resident/evicting/failed；每阶段支持 cancel/backpressure/shutdown |
| TER2-P1-066 | Open | 无 HLOD layer/builder/source key/incremental invalidation/quality metric/transition | HLOD 是可流送派生 artifact，近远表示按 generation 原子切换并保留 visual compare evidence |
| TER2-P1-067 | Open | 无 cross-cell reference、external actor/content bundle、server policy 与 large-world cook | 定义 ownership/reference closure、dedicated server profile、patch/package 与坏 cell recovery |

### 6.9 Editor、测试、性能与竞争资格

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| TER2-P1-068 | Open | 无 Terrain document session、revision、dirty region、history/savepoint/autosave/recovery | `WorldAuthoringDocument` 与 Terrain subdocument 使用 bounded region delta、CAS save、atomic replace 和 recovery |
| TER2-P1-069 | Open | 无真实 pick/brush/stroke/height-weight-hole target、pressure/falloff/overlay/undo | 参考 Fyrox 的 raycast + brush thread + touched chunk swap，接统一 pointer capture、transaction 和 mode teardown |
| TER2-P1-070 | Open | Create/Open/Import 没有路径占用、template、decode/build、catalog acknowledgement、repair mode | 每个 operation 有 prepare/apply/progress/cancel/result，直到 durable source 与 catalog generation 都确认才完成 |
| TER2-P1-071 | Open | tests 没有 source-to-pixel/query/collision/nav/save/reopen/stream/evict 场景 | 建 canonical corpus，Client/Editor/Server/source/library/native 使用同 assets、camera、seed、profile 与 expected receipts |
| TER2-P1-072 | Open | 没有与 Unreal/Fyrox/旧 baked mesh 同画质同硬件 benchmark | 记录 CPU/GPU frame、VRAM/RAM、IO、build/edit latency、stutter、LOD pop、image/seam metrics；只有同口径胜出才可声称优于 Unreal |

## 7. P2：成熟度、可维护性与诊断差距

| ID | 状态 | 当前差距 | 改进要求 |
|---|---|---|---|
| TER2-P2-001 | Open | Terrain/Landscape/HeightField/patch/chunk 术语混乱 | 发布 canonical glossary 与 type/path ownership，旧名只存在于 versioned migration |
| TER2-P2-002 | Open | 无 patch/LOD/bounds/normal/layer/hole wireframe 与 debug view | 通过共享 debug framework 注册，按 view/generation/budget 输出 |
| TER2-P2-003 | Open | 无 visual/collision/nav surface divergence heatmap | 对同 generation 采样 position/normal/material/hole 差异并输出误差统计 |
| TER2-P2-004 | Open | 无 page/cell/HLOD desired/resident/pinned/evicting 可视化 | 显示真实 source、cost、reason、age，不显示固定数字 |
| TER2-P2-005 | Open | 无 artifact provenance inspector | 展示 source hash、builder/backend/profile、pages、layers、bounds、dependencies 与 diagnostics |
| TER2-P2-006 | Open | 无 deterministic scatter replay 与 instance diff | 按 cell/seed/prototype 输出 added/removed/moved 与 filter reason |
| TER2-P2-007 | Open | 缺 1xN、非方形、极端高差、hole、坏 weight、负 scale、oversize corpus | property/fuzz tests 要求 typed error、无 panic/OOM、bounded time/memory |
| TER2-P2-008 | Open | 无 CPU/WGSL/physics/nav/ray height-normal-diagonal 一致性门 | 共享 fixtures、坐标定义和误差预算 |
| TER2-P2-009 | Open | 无 shader permutation/PSO/page residency 统计 | cook/frame receipt 记录来源、命中率、warmup、miss 与 fallback |
| TER2-P2-010 | Open | 无 foliage cluster rebuild、wind、LOD transition profiler | 输出 CPU/GPU cost、occupancy、overdraw、crossfade pixels 与 churn |
| TER2-P2-011 | Open | 无 schema compatibility/migration fixtures | 每个 source/artifact/manifest version 有 forward/backward/loss policy |
| TER2-P2-012 | Open | 无 headless/dedicated server Terrain 资格 | 不初始化 render 时仍能加载 query/collision/nav artifact 并满足 memory budget |
| TER2-P2-013 | Open | 无 dimension/layer/page/cell/instance/backend 支持上限 | 发布 limits 与 failure semantics，不从实现常量反推合同 |
| TER2-P2-014 | Open | 无“性能提升不得降低质量”的联合门 | 所有快路径同时通过 seam/material/hole/collision/nav/image threshold |

## 8. 历史台账重判

### 8.1 Runtime29

Runtime29 的 62 项 P1 重判为 **56 Open、6 Partial、0 Closed**。Partial 为旧 `TER-P1-003`（已有弱 sample-count validator）、`TER-P1-005`（layer material/weightmap reference）、`TER-P1-008`（builtin importer/artifact plumbing）、`TER-P1-011`（SceneTerrainAsset carrier）、`TER-P1-017`（descriptor-only slot 明确未执行）和 `TER-P1-038`（Jolt HeightField 可经 triangle compound 运行）。14 项 P2 全部 Open。旧报告没有本地新增 P0 的裁决继续有效，但 Editor16 的五项跨层 P0 均未关闭。

### 8.2 Editor16

Editor16 的 5 项 P0 全部 Open。60 项 P1 重判为 **58 Open、1 Partial、1 Closed**：第 14 项“Editor import plan 未 checked multiplication”已由 `u64::checked_mul + usize::try_from` 关闭；第 15 项 LayerStack 至少已 fail-close 并解释缺少 channel/format semantics，故为 Partial；其余 operation、document、brush、runtime、foliage、partition 与产品链仍 Open。12 项 P2 全部 Open。

### 8.3 Render15 与 failure

- Render vegetation/terrain 路线中的 TV-M1 至 TV-M4 仍为 Not Started；普通 mesh/static batch 不能替代 Terrain/Foliage runtime。
- `docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md` 保持 Open。Terrain scene mode 已从 descriptor 面移除，但 production factory、authoring transaction、capability-disable lifecycle、toolbar 与 focused product evidence 仍不存在，不能以“删除入口”关闭原问题。

## 9. 五套参考源码的正确用法

| 参考 | 本轮确认的工程边界 | Zircon 应采用 | 不应误用 |
|---|---|---|---|
| Unreal Landscape/Foliage/WorldPartition | component height/weight textures、edit layers、streaming manager、collision/nav/physical material、grass async build/HISM、Nanite/HLOD、cell/source/data-layer state | 作为完整产品架构、cross-domain invalidation、streaming/HLOD 和资格门主标尺 | 不复制 UObject 宏观结构；本地 UE 树未找到 VirtualHeightfieldMesh，不声称已对照该模块 |
| Fyrox Terrain | real Terrain node、chunk R32F heightmap + margin/hole、quadtree LOD、raycast/render bundles、brush thread、undoable chunk swap | 作为 Zircon M3 “最小真实可执行 Terrain”底线和 Editor stroke 参考 | 不把较小实现当作 large-world/VT/Nanite 终点 |
| Godot HeightMapShape/MultiMesh | heightmap validation/NaN holes/Jolt heightfield、bulk instance buffer/visible count/AABB/render server RID、surface populate | 作为 physics heightfield 和 foliage instance server 边界参考 | Godot 本轮源码没有 built-in Terrain renderer，不把 MultiMesh 称为完整 landscape |
| Unity Graphics | HDRP/URP TerrainLit layer/splat/hole/instancing/ray paths，TerrainToMesh/ray marching，GPU Resident Drawer/culling/SpeedTree wind | 作为 material、ray/path、GPU-driven instance 与 render qualification 参考 | 不让 Graphics package 反向拥有 World/authoring authority |
| Bevy render | main-world/render-world extraction、visibility change separation、GPU preprocessing/indirect buffers、mesh allocator | 作为 data-oriented extract/cull/batch/allocation 参考 | Bevy 本轮没有 built-in Terrain/Foliage 系统，不能作为 Terrain feature parity 证据 |

## 10. 目标架构与硬切合同

```text
Editor-owned authoring truth
  WorldAuthoringDocument
    -> TerrainSource / EditLayer / FoliageRule / PartitionPolicy
    -> transaction + revision + dirty regions + atomic save
        |
        v
Runtime-owned neutral build truth
  TerrainCompiler
    -> versioned TerrainBuildArtifact
       {height/hole/layer pages, bounds/error, collision/nav/scatter/HLOD keys}
        |
        v
Runtime-owned per-World truth
  TerrainWorldService + WorldPartitionService
    -> generation-bound TerrainInstance / CellBundle / QuerySnapshot
    -> desired/requested/resident/attached/evicting/failed
        |
        +--> Graphics provider: patch/virtualized route, material/VT, shadow/ray
        +--> Physics provider: native heightfield, material, dirty tile swap
        +--> Navigation provider: real geometry, dirty tile generation
        +--> Foliage provider: deterministic cell artifact, cluster/indirect draw
        +--> Editor observation: preview snapshot, progress, diagnostics

App
  -> only selects target/profile/providers and hosts lifecycle
  -> never owns Terrain data, fake result, renderer state or editor transaction
```

硬切要求：

1. `TerrainAsset` 不再同时承担 source、cooked artifact 和 runtime instance。
2. builtin importer 与 source plugin importer 收敛为一套 canonical source/compile identity；compat alias 只用于 versioned migration。
3. plugin component descriptor 不能替代 ECS storage/system；descriptor-only render slot 不能升级为 Supported。
4. Physics、Navigation、Foliage、HLOD 只能消费同 Terrain generation 的派生 view，不得复制出可独立编辑的高度真相。
5. Editor preview、PIE、Client、Server 必须消费同 artifact/runtime contract；Workbench 不得拥有模拟 backend。
6. World Partition 为 Runtime owner，Editor 只 author manifest；App 只选择 provider 和生命周期。

## 11. 依赖有序重构里程碑

### M0：Truth Freeze 与 RED 证据

- 默认产品下证明 Terrain provider 不可达、三资源缺失、operation `MissingFactory`、World save 丢 Terrain、descriptor-only 零 pass。
- 将 Workbench 改为 Prototype/Unavailable 或隐藏固定 success；建立 deletion/owner/caller matrix。

### M1：Canonical Source 与 Artifact

- 发布 versioned Terrain source/layer/hole/grid schema、bounded decoders、validator、migration 与 artifact key。
- 建 tile/page、compression/checksum/min-max/error/border 与 dependency/dirty-region compiler。

### M2：World Persistence、Owner 与 Query

- 完成 SceneAsset <-> World 无损 roundtrip、typed component、per-World service、generation tickets。
- 交付 immutable height/normal/material/hole query snapshot 和 stale rejection。

### M3：最小真实可执行 Terrain

- 以 Fyrox 为最低线：真实 chunk height resource、shared patch geometry、quadtree LOD、bounds/raycast/render pass。
- 通过 source-to-pixel、seam、height query、save/reopen 与 unload tests；不等待高级 VT/Nanite-like route。

### M4：Material、LOD、GPU 与 Residency

- layer compiler、TerrainLit contract、holes、normal/tangent、shadow/motion/picking、GPU cull/indirect。
- 接共享 residency/budget/device-loss；可选 clipmap/virtualized provider 必须有资格与 fallback。

### M5：Physics 与 Navigation

- Terrain artifact 生成 Jolt native heightfield view、physical material、dirty tile atomic swap。
- Navigation 消费真实 surface/hole/area 与同 generation dirty tiles，删除 placeholder quad/skip 路线。

### M6：Editor Authoring

- AuthoringGateway、document、create/open/import、real scene mode、pick/brush/stroke、undo/redo/save/recovery。
- operation 使用 prepare/apply/progress/wake/cancel/result，plugin disable/reload 有完整终态。

### M7：Foliage 与 Scatter

- versioned prototype/rule、deterministic cell compiler、manual/generated ownership、cluster/GPU instance runtime。
- LOD/impostor/wind/shadow/ray/collision/nav/scalability 与 cell generation 组合。

### M8：World Partition 与 HLOD

- versioned manifest、stable cells、multiple sources、data layers、budgets、atomic cell bundle 与 cross-cell reference。
- HLOD builder/artifact/quality evidence、near/far transition、server/cook/package/recovery。

### M9：Failure、Platform 与 Scale

- corrupt/oversize/OOM/cancel/device loss/world unload/plugin reload/shutdown 均有 terminal receipt。
- 三平台、headless/server、fuzz/property、million-sample/large-cell/million-instance、churn/soak 门。

### M10：竞争资格

- Vampire/WOC 或专用 fixture 从 source import/build/save/reload 到 Client pixel、physics/nav、foliage、streaming 全链。
- 固定资产、camera、seed、画质、硬件、profile 对比 Unreal/Fyrox/旧 baked mesh，再以 CPU/GPU/VRAM/RAM/IO/tail latency/image metrics 判定；禁止先写“优于 Unreal”。

## 12. G01-G36 综合资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | ordinary Client/Editor 没有 selected Terrain provider receipt |
| G02 | Partial | typed source/schema 存在，但 validator 与 limits 不完整 |
| G03 | Fail | RAW/R16/PNG 没有真实 bounded decoder |
| G04 | Partial | 通用 artifact store/load 存在，无 Terrain build artifact/version |
| G05 | Partial | SceneAsset 文档 carrier 可保存 reference，World roundtrip 不可 |
| G06 | Fail | World load/save、component、service 与 query owner 缺失 |
| G07 | Fail | plugin resource/factory publication 不是原子硬门 |
| G08 | Fail | runtime plugin 为 DiagnosticOnly/stateless，无 executable backend |
| G09 | Fail | descriptor-only Terrain 零 pass、零 extract consumer |
| G10 | Fail | 无真实 patch geometry 与 source-to-pixel test |
| G11 | Fail | 无 LOD/screen error/crack-free seam contract |
| G12 | Fail | 无 frustum/HZB/GPU cull/indirect/capacity receipt |
| G13 | Fail | 无 geometry/material residency、budget、eviction、device recovery |
| G14 | Fail | 无 layer packing/blending/hole/normal/material ABI |
| G15 | Fail | 无 shadow/motion/picking/decal/GI/ray pipeline matrix |
| G16 | Fail | 无 virtual texture/far representation 与 fallback qualification |
| G17 | Fail | TerrainAsset 与 Physics HeightField 是独立真相 |
| G18 | Fail | 无 Terrain -> physics projection 和 generation atomic swap |
| G19 | Partial | Jolt HeightField 有 validation 与 triangle fallback，但不是 native heightfield |
| G20 | Fail | builtin physics 对 HeightField 无 fail-close capability admission |
| G21 | Fail | Navigation 不消费真实 Terrain/HeightField geometry |
| G22 | Fail | edit/stream/collision/nav 无同 generation dirty protocol |
| G23 | Fail | 无 FoliagePrototype/ScatterRule/runtime cluster |
| G24 | Fail | 无 deterministic scatter identity、override 与 incremental cell build |
| G25 | Fail | 无 cluster/instance culling、LOD/impostor/wind 与 budgets |
| G26 | Fail | 无 partition manifest/stable cell/source/data layer authority |
| G27 | Fail | 无 cell IO/prepare/attach/evict/cancel/backpressure 状态机 |
| G28 | Fail | 无 HLOD builder/artifact/transition/quality evidence |
| G29 | Fail | Workbench 仍发布固定成功、cell、instance 与 memory 数据 |
| G30 | Fail | Editor 无 document/transaction/brush/save/recovery 产品链 |
| G31 | Partial | Editor typed import plan 与 fail-close 已有局部基础，无 decoder/build/apply |
| G32 | Fail | Terrain failure handoff 未通过原 required gates 关闭 |
| G33 | Fail | 无 source-to-pixel/query/collision/nav/save/stream canonical scenario |
| G34 | Fail | 无 corrupt/oversize/fault/unload/reload/shutdown terminal matrix |
| G35 | Fail | 无三平台/headless/scale/soak/performance qualification |
| G36 | Fail | 本轮只有静态 review；没有动态证据支持实现完成或优于 Unreal |

## 13. 禁止的临时修补

1. 禁止把 `TerrainAsset` 转一次普通 mesh 后宣称 Terrain runtime 完成；mesh oracle 可以保留，但必须标明 representation 与失效成本。
2. 禁止在 descriptor-only Terrain slot 中塞一个无 LOD、无 layer、无 collision/nav 的单 pass 临时 renderer。
3. 禁止让 plugin component metadata、App enum 或 static manifest 替代 per-World owner 和 product reachability。
4. 禁止继续用 `DiagnosticOnlyAssetImporter`、缺 factory command、空 scene mode 或固定 Workbench feedback 暴露可执行入口。
5. 禁止 Physics、Navigation、Foliage 各自复制高度数组并形成多份 source truth。
6. 禁止用 Jolt triangle-per-cell compound 作为大地形合格后端，或把 Navigation quad/skip 作为 fallback success。
7. 禁止先实现 virtual texture、Nanite-like、复杂 erosion 等高级表面，再补 Scene/World/persistence/query 基础闭环。
8. 禁止 Terrain 私建 asset cache、job pool、operation service、transaction/history、debug overlay 或 GPU allocator。
9. 禁止用 ignored microbenchmark、registration test、枚举存在、普通 mesh pixels 或固定 UI 数据声称性能/表现优于 Unreal。
10. 禁止在 MVP baseline 前扩张 P2；当前允许的实施起点是 M0 truth RED、M1 canonical source/artifact 和 M2 owner/roundtrip。

## 14. 本轮完成定义

- 已冻结并检查 120 个 Zircon selected 文件与 54 个五引擎参考文件，分别记录可复现 fingerprint。
- 已纠正 Scene 文档 Terrain reference 已可保存、但 World execution/export 仍断裂这一关键 current-source 差异。
- 已登记 5 项 P0、72 项 P1、14 项 P2、M0-M10 与 G01-G36；跨层 P0 继续由 Editor16/本报告统一解释，不重复制造多个 owner。
- 已明确 Unreal/Fyrox/Godot/Unity Graphics/Bevy 各自的参考职责，未把不存在的 built-in Terrain 能力写入参考结论。
- 本轮只修改 review 与索引，不修改生产代码，不运行 Cargo，不关闭 failure，不给出动态通过或竞争性能结论。
- 任一实施 Session 开始前必须重取 HEAD/fingerprint、确认 failure owner/lease，并从 M0 capability truth 与唯一 authority 开始。
