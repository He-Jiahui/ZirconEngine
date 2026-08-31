---
title: Editor Terrain / Landscape / Foliage / Scatter / World Partition / Level Streaming Authoring 当前源码复审
category: zircon_editor
report_id: Editor138
review_date: 2026-08-26
baseline_head: 8e56165c4c789416c328898d3d8937d934b52efa
verification_head: 8e56165c4c789416c328898d3d8937d934b52efa
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/92-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
related_handoff:
  - docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md
related_code:
  - zircon_plugins/terrain
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_runtime_catalog
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/document
  - zircon_editor/src/core/jobs
  - zircon_editor/src/scene/modes
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor
  - dev/UnrealEngine/Engine/Source/Editor/FoliageEdit
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/godot/scene/resources/multimesh.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/bevy/crates/bevy_render/src/view/visibility
  - dev/bevy/crates/bevy_render/src/batching
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor138 · Terrain / Landscape / Foliage / World Partition 当前源码复审

## 1. 结论

当前 Zircon Editor 仍没有可执行、可保存、可恢复的 Terrain / Landscape、Foliage / Scatter 或 World Partition / Level Streaming authoring 产品。仓库拥有 typed Terrain carrier、插件 manifest、5 个 operation descriptor、类型化 import plan、通用 document/transaction/job/scene-mode 基础和四张外观完整的 World Building Workbench，但没有把它们连成 `source -> document -> transaction -> build job -> immutable artifact -> runtime generation -> truthful projection` 的工程闭环。

产品真相失配仍是 P0。Terrain 没有进入 first-party Editor/Runtime executable provider catalog 与 App 默认组合；插件声明的 `plugins://terrain/editor/authoring.zui`、`plugins://terrain/editor/terrain_component.zui`、`plugins://terrain/templates/default_heightfield.toml` 物理不存在。Import Heightfield、Import Weightmap、Create Heightfield、Open、Sculpt 共 5 个 operation 只有 metadata，没有生产 factory。`EditorAuthoringContributionBatch` 没有 operation factory 字段；Terrain contribution 使用默认空 `scene_modes`，仓库中也没有 Terrain mode。对应 failure handoff 仍为 Open，本轮按用户要求不等待或轮询协调器。

四份 World Building ZUI 仍各 230 行，合计 **920 行、108 个 node、76 个 action route、0 个 provider**。它们继续固定显示 Summit Valley、SC_Forest、84K/64K instances、128 clusters、18 rules、96 cells 与 96 MB；feedback 只改 UI property。没有 document revision、transaction receipt、job ticket、artifact generation、runtime snapshot、cancel acknowledgement 或 terminal result，因此这些数值与 queued/success 文本都不是产品证据。

Editor92 之后可确认两项局部变化，但不改变状态。第一，`TerrainHeightfieldImportRequest` 继续使用 RAW/R16/PNG typed source format、`checked_mul` 与 `usize::try_from`，LayerStack 继续 fail-close；实现仍只产生 plan，不读取 bytes、不解码、不写 source/artifact，也不进入 transaction。第二，`LevelSystem` 增加 per-World `WorldTimeController`、`TimePolicyTransaction`、fixed-step/interpolation state；它仍只有整 World 的 Loaded/Unloaded lifecycle，没有 partition manifest、cell、streaming source、data layer、HLOD、residency 或 IO pipeline。把时间域补强解释为 Level Streaming 完成是错误的。

Runtime downstream 仍未闭合。Terrain runtime importer 是 `DiagnosticOnlyAssetImporter`，固定报告 `terrain heightfield importer backend is not installed`；`TerrainAsset::validate_dimensions` 仍允许零尺寸、空 samples、非有限或非正 spacing/scale，且用普通 `usize` 乘法比较样本数；`SceneAsset` 能携带 `SceneTerrainAsset`，但 World load 不实例化，save 仍固定写 `terrain: None`。Graphics 的 Terrain 仍在 descriptor-only advanced slot；Physics HeightField、Navigation geometry 与 Terrain source 没有 generation-qualified adapter；Foliage/Scatter 没有 source/compiler/artifact/runtime cluster。

本轮重判保持 **5 项 P0 全部 Open；60 项 P1 为 58 Open、1 Partial、1 Closed；12 项 P2 全部 Open；32 项资格门为 30 Fail、1 Partial、1 Pass**。没有动态证据支持 Terrain/Foliage/World Partition 的完成度、性能或表现达到或优于 Unreal；在 canonical scenario、同画质 benchmark 与失败矩阵通过前，禁止作此声明。

## 2. 审查边界、统计与 currentness

统计基于 working-tree 物理文件，扩展名限定为 Rust、TOML、ZUI、Markdown、C/C++、C# 与 HLSL；行数为物理行，tests 为精确 Rust `#[test]` 声明，ignored 为精确 `#[ignore]` 声明。选择集用于复现本报告，不代表参考仓库全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据边界 |
|---|---:|---|
| Terrain 插件全量 | **16 / 1,072 / 971 / 38,904 / 10 / 0** | editor/runtime/dist/manifest/tests 全量 |
| Editor 产品面、路由、catalog 与 App | **23 / 5,713 / 5,471 / 267,368 / 11 / 0** | 四份 World Building ZUI、feedback/navigation/binding、catalog 与 App |
| Editor command/document/transaction/job/scene-mode 共享承接 | **172 / 33,267 / 30,148 / 1,112,355 / 264 / 0** | 只判定可复用底座是否有 Terrain consumer |
| Runtime downstream、Physics/Nav、catalog 与示例 oracle | **41 / 13,861 / 12,685 / 500,200 / 75 / 0** | asset/scene/level/render/physics/nav/catalog/Vampire |
| Zircon selected union | **252 / 53,913 / 49,275 / 1,918,827 / 360 / 0** | 上述四组去重物理集合 |
| Unreal/Fyrox/Godot/Unity Graphics/Bevy reference | **30 / 39,058 / 33,890 / 1,498,071 / 0 / 0** | Landscape/Foliage/Partition、Terrain node、heightmap physics、GPU instance 与 visibility/batching |
| Plan/docs evidence | **11 / 3,996 / 2,940 / 416,616 / 0 / 0** | Editor16/92、Runtime142、共享 owner 与 open failure handoff |
| 全部证据 union | **293 / 96,967 / 86,105 / 3,833,514 / 360 / 0** | `2026-08-26T15:45:31+08:00` 完成受影响 owner 重扫后的当前物理集合 |

- baseline 与 verification HEAD 均为 `8e56165c4c789416c328898d3d8937d934b52efa`，commit time 为 `2026-08-26T07:46:39+08:00`。
- working tree 含用户与其他 Session 的大量修改/新增文件；本报告读取物理文件，不回退、不覆盖，也不把在途内容写成已集成能力。
- 按用户要求未查询、轮询或等待协调器；open handoff 作为静态阻塞事实记录，但不阻塞本轮 review。
- 本轮只写 review 文档；未运行 Cargo、Editor、PIE、import/cook、GPU capture、physics/nav、streaming、fault、scale、soak 或 benchmark。

## 3. 当前链路事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| Package -> Catalog -> App | manifest 可被静态发现；默认 Editor/Runtime executable catalog 与 App 不选择 Terrain | Open |
| Resource publication | 3 个声明资源不存在；无 publication 原子门 | Open |
| Editor operation | 5 个 descriptor 存在；batch 无 factory，dispatch 不具执行 owner | Open |
| Terrain scene mode | README/意图存在；contribution `scene_modes` 为空，全仓无 Terrain mode | Open |
| Import planning | RAW/R16/PNG typed plan、checked count、LayerStack fail-close；无 read/decode/commit | Partial |
| Runtime import | core TOML carrier存在；Terrain plugin importer固定 DiagnosticOnly | Open |
| Document/transaction | 通用底座存在；Terrain document/session/stroke/save consumer 为零 | Open |
| Scene roundtrip | SceneAsset 保存 reference；World load 不实例化，save 固定 `terrain: None` | Open |
| Render/Physics/Nav | Terrain 零 pass；HeightField/Nav 与 Terrain source 分裂 | Open |
| Foliage/Scatter | Workbench 有固定数据；无 source/compiler/artifact/runtime cluster | Open |
| World Partition | LevelSystem 新增时间策略但仍只有整 World lifecycle | Open |
| Evidence | 普通 mesh 与静态 feedback 不能证明 Terrain/Foliage/Streaming backend | Open |

## 4. P0：产品虚假可达与 authority 断裂

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| WB2-P0-01 | Open | Terrain 不在默认 catalog/App，3 个资源缺失，但类型与 Workbench 让能力看似可达 | publication 前原子验证 package/resource/provider/factory/profile；缺失时整能力 disabled 并给 typed reason |
| WB2-P0-02 | Open | 5 个 operation 仅 metadata；无 factory、document owner 与 Terrain scene mode | `TerrainAuthoringGateway` 提供 typed prepare/apply/cancel/undo/save；mode 持有 input/overlay/teardown lease |
| WB2-P0-03 | Open | runtime importer 为 DiagnosticOnly，World 不实例化 Terrain，Graphics 零 pass | 先由 Runtime142 交付 per-World generation-bound Terrain service/artifact/runtime handle |
| WB2-P0-04 | Open | 4 份 Workbench 发布固定 build/instance/cell/memory/success 文本 | 立即标为 Prototype/Unavailable 或隐藏；只投影 job 终态与 runtime snapshot |
| WB2-P0-05 | Open | 无统一 WorldAuthoringDocument/partition authority，World export 写 `terrain: None` | 无损 roundtrip 前 fail-close；document、manifest、runtime generation 必须同链 |

## 5. P1：工程级完整性差距

### 5.1 Package、capability 与产品装配

| ID | 状态 | 差距与重构要求 |
|---|---|---|
| WB2-P1-01 | Open | manifest、runtime metadata、Editor descriptor 与默认 catalog 无单一装配 truth；建立 profile-qualified provider receipt |
| WB2-P1-02 | Open | backend/resource/factory 缺口没有统一 readiness；由 typed capability diagnostic 投影 |
| WB2-P1-03 | Open | 3 个资源 URI 未在 publication 前 resolve/read/schema validate；任一失败整 batch rollback |
| WB2-P1-04 | Open | builtin registry 识别 Terrain/LayerStack，插件缺失时无只读诊断 toolkit |
| WB2-P1-05 | Open | editor/runtime dist 无 authoring command bridge；ABI 必须承载 typed request/receipt 或禁用动态 authoring |
| WB2-P1-06 | Open | contribution batch 不含 operation/document/build/preview factory；扩展注册必须原子带齐 execution owner |
| WB2-P1-07 | Open | menu/toolkit/view/importer/customization 缺共同 owner lease、rollback 与 revoke barrier |
| WB2-P1-08 | Open | disable/reload 未定义 open document、active stroke、dirty source、resident preview 的处置 |
| WB2-P1-09 | Open | README 的 scene-mode/runtime-backed 声明与空注册无自动一致性门 |
| WB2-P1-10 | Open | tests 只验证 descriptor/plan 形状；缺 default bootstrap、resource resolution、dispatch、disable 矩阵 |

### 5.2 Terrain source、import、layer 与 artifact

| ID | 状态 | 差距与重构要求 |
|---|---|---|
| WB2-P1-11 | Open | plan 不读 bytes，不能验证长度/header/endianness/bit depth/channel/row order；拆成 bounded Validate/Decode/Build |
| WB2-P1-12 | Open | PNG 缺颜色类型、gamma/profile、alpha、interlace、decode limit 与 height-range policy |
| WB2-P1-13 | Open | RAW/R16 缺尺寸推断、stride、byte order、signedness、scale/offset/no-data/trailing-bytes policy |
| WB2-P1-14 | Closed | sample count 已使用 `u64::checked_mul` 与 `usize::try_from`；保留回归门并补全局 allocation budget |
| WB2-P1-15 | Partial | LayerStack 当前 fail-close，但仍复用 heightfield request；需要独立 source/channel/stable layer schema |
| WB2-P1-16 | Open | runtime 只声明 heightfield importer，Editor 两个 import 入口与 artifact/runtime consumer 不对称 |
| WB2-P1-17 | Open | `TerrainAsset` validator 允许零维、空 samples、非有限/非正 spacing/scale，样本比较使用普通乘法 |
| WB2-P1-18 | Open | layer 无 stable ID、blend、physical material、tiling、visibility/lock/edit-layer |
| WB2-P1-19 | Open | layer name/strength/material/weightmap 缺重复、range、kind、resolution 与 cycle validation |
| WB2-P1-20 | Open | LayerStack 无 validator、version/migration、canonical ordering |
| WB2-P1-21 | Open | 大型高度数据仍是 inline `Vec<Real>`；改为 tile/page source 与 derived artifact |
| WB2-P1-22 | Open | source、derived tile、resident section 无不同类型/revision/generation |

### 5.3 Terrain document、scene mode、笔刷与保存

| ID | 状态 | 差距与重构要求 |
|---|---|---|
| WB2-P1-23 | Open | 无 Terrain document session/source revision/dirty set/history/autosave/CAS |
| WB2-P1-24 | Open | Create 无真实 template、topology/range validation、占用检查与 atomic write |
| WB2-P1-25 | Open | Open 无 selection -> toolkit -> document -> preview factory 链 |
| WB2-P1-26 | Open | Sculpt 无 mode factory、terrain picking、pointer capture、pressure/spacing/falloff/overlay |
| WB2-P1-27 | Open | Height/Weight/Hole target 不存在；UI tab、selected layer 与 edit target 可分裂 |
| WB2-P1-28 | Open | stroke 无 bounded region delta、before image/inverse、merge key、base revision、cancel rollback |
| WB2-P1-29 | Open | source/GPU/collision/nav/LOD/normal/foliage 无统一 affected-region DAG |
| WB2-P1-30 | Open | smooth/flatten/ramp/erosion/spline/copy-paste/resize 无 deterministic contract |
| WB2-P1-31 | Open | 无 non-destructive edit layer、lock/order/blend/merge/bake/migration |
| WB2-P1-32 | Open | multi-terrain seam、shared border、world transform/origin rebasing 未定义 |
| WB2-P1-33 | Open | save 无 validated snapshot、atomic replace、last-good、recovery、partial cleanup |
| WB2-P1-34 | Open | Workbench field edit 只改 control value，固定 feedback 不进入 history |

### 5.4 Runtime 承接、render、physics、navigation 与性能

| ID | 状态 | 差距与重构要求 |
|---|---|---|
| WB2-P1-35 | Open | `SceneTerrainAsset` 只有引用，无 transform/collision/nav/material/streaming/generation policy |
| WB2-P1-36 | Open | scene load 不创建 Terrain component/service，descriptor 不产生行为 |
| WB2-P1-37 | Open | graphics 无 extract/renderer/pass/patch/height/material consumer；Editor preview 不得另造 mock backend |
| WB2-P1-38 | Open | 无 topology、quadtree/clipmap、screen error、neighbor constraint、morph/skirt |
| WB2-P1-39 | Open | visibility 未接 frustum/occlusion/GPU Scene，bounds 无增量更新 |
| WB2-P1-40 | Open | splat/hole/normal/macro/VT 与 shadow/depth/ray paths 无合同 |
| WB2-P1-41 | Open | CPU/GPU query、raycast、collision、render surface 无 generation 一致性 |
| WB2-P1-42 | Open | Nav 无 dirty tile rebuild/cancel/commit/hole/slope/layer-cost contract |
| WB2-P1-43 | Open | build/upload 未接 asset streaming、resource budget、device recovery、shutdown fence |
| WB2-P1-44 | Open | 无同画质 frame/GPU/VRAM/IO/LOD-pop/edit-latency/stutter 基线 |

### 5.5 Foliage、Scatter 与实例 authoring

| ID | 状态 | 差距与重构要求 |
|---|---|---|
| WB2-P1-45 | Open | 无 Foliage/Scatter source、versioned schema、AssetKind、importer、toolkit、runtime service |
| WB2-P1-46 | Open | 84K/128 clusters 全是静态文本；snapshot 必须含 source/runtime generation 与 freshness |
| WB2-P1-47 | Open | 18 rules/64K/conflict 无 document/generator/validator；接 deterministic compile job |
| WB2-P1-48 | Open | 无 stable rule/prototype ID、seed stream、density/mask/slope/scale/collision/exclusion schema |
| WB2-P1-49 | Open | 无 per-cell incremental artifact、dependency key、invalidation、cache、rollback |
| WB2-P1-50 | Open | 手摆/generated/override/exclusion 无 ownership 与 regenerate/undo/save 语义 |
| WB2-P1-51 | Open | runtime 无 cluster bounds、GPU instance、culling、LOD/impostor/wind/shadow/budget |
| WB2-P1-52 | Open | 无百万实例确定性、分布、约束、延迟、traversal、跨平台预算 |

### 5.6 World Partition、Level Streaming、Data Layer 与 HLOD

| ID | 状态 | 差距与重构要求 |
|---|---|---|
| WB2-P1-53 | Open | 无 partition asset/manifest、stable cell ID、grid/bounds/content owner/version |
| WB2-P1-54 | Open | 新增 time policy 后仍只有整 World Loaded/Unloaded；建立完整 cell state machine |
| WB2-P1-55 | Open | 无 streaming source shape/priority、multi-source merge、always-loaded/pin/data-layer/condition |
| WB2-P1-56 | Open | 无 cell dependency、cross-cell reference、external actor/content bundle 与 atomic move |
| WB2-P1-57 | Open | IO/decompress/deserialize/upload/attach 无 ticket、budget、cancel ack、backpressure、shutdown terminal |
| WB2-P1-58 | Open | 96 cells/96 MB/HLOD_04 无 manifest/request；改为 generation-qualified residency snapshot |
| WB2-P1-59 | Open | 无 HLOD layer/builder/key/artifact/incremental rebuild/quality/transition/last-good |
| WB2-P1-60 | Open | 无 partition convert/validate/minimap/data-layer/HLOD typed headless workflow |

## 6. P2：成熟度、协作与诊断差距

| ID | 状态 | 当前差距 |
|---|---|---|
| WB2-P2-01 | Open | 无 per-user brush preset、falloff/alpha library、pressure curve、overlay density preference |
| WB2-P2-02 | Open | 无 Terrain camera、streaming source、LOD/weight/cell debug bookmark |
| WB2-P2-03 | Open | 无最近 Terrain/layer/rule/cell 与跨 document 导航历史 |
| WB2-P2-04 | Open | 无 height/weight/hole/edit-layer revision diff、heatmap、side-by-side compare |
| WB2-P2-05 | Open | 无 stroke/rule/cell/HLOD comment、tag、review note、approval state |
| WB2-P2-06 | Open | 无 layer/prototype/rule/data-layer/HLOD batch edit 与 usage query |
| WB2-P2-07 | Open | 无 bounded support bundle 导出 source/diagnostic/dirty-region/artifact/streaming-journal/capture reference |
| WB2-P2-08 | Open | 无 opt-in edit latency、cache hit、resident cell、IO/upload、LOD transition telemetry |
| WB2-P2-09 | Open | 无 dry-run、batch reimport、partition validate、scatter rebuild、HLOD stale report |
| WB2-P2-10 | Open | 无 typed Editor scripting/remote automation Terrain/Scatter/Partition surface |
| WB2-P2-11 | Open | 无多人 terrain region/cell lock、冲突可视化、ownership transfer、revision annotation |
| WB2-P2-12 | Open | 无色盲安全 weight palette、键盘笔刷、screen-reader cell table、完整 i18n/unit formatting |

## 7. 参考源码映射

| 参考 | 关键合同 | Zircon 应吸收 | 边界 |
|---|---|---|---|
| Unreal LandscapeEditor | Height/Weight/Visibility target、brush/stroke lifecycle、scoped transaction、edit layer、Validate/Import/Export、tiled/cropped import error | target authority、bounded stroke、transaction、跨 component invalidation、可定位 import diagnostics | 不照搬 UObject/Slate 与历史兼容面 |
| Unreal Foliage | Add/Remove/Select/Reapply、transaction、foliage type、instance ownership、spatial hash | stable prototype/rule、instance provenance、可逆 brush、spatial index | 不照搬 Actor ownership 细节 |
| Unreal World Partition | runtime cell state、streaming source shape/range/priority、data-layer effective state、hash-qualified HLOD | manifest/cell/source/data-layer/HLOD 分层、可解释状态与增量 build | 不照搬 package/external actor 历史布局 |
| Fyrox | 真实 Terrain node/raycast/background brush、height/layer/hole target、chunk swap undo、quadtree | 先交付小而真的 source-to-runtime authoring 闭环 | 不把较小规模当最终性能上限 |
| Godot | HeightMapShape3D physics/debug mesh、MultiMesh bulk instance | heightfield physics 与 bulk instance resource 边界 | 不是完整 World Editor 参考 |
| Unity Graphics | TerrainLit/hole path、GPUResidentDrawer、InstanceDataSystem、InstanceCuller | material ABI、GPU instance data/culling/residency consumer | checkout 不含 Unity Editor authority |
| Bevy | extract、visibility range、GPU/no-GPU preprocessing、batching | 共享 ECS extraction/visibility/batching primitive | 不能替代 Landscape authoring 产品 |

## 8. 目标架构与 owner 边界

```text
TerrainSource / LayerStackSource / FoliageRuleSet / PartitionSource
    -> AuthoringDocument + stable element address
    -> reversible transaction + dirty region / changed cell set
    -> bounded BuildJob(source_revision, profile)
    -> immutable artifact + diagnostics + dependency hash
    -> Runtime World service atomic generation swap
    -> generation-qualified render/query/physics/nav/residency snapshot
    -> Editor projection + command receipt
```

- Editor138 唯一负责 Terrain/Foliage/Scatter authoring document、mode、stroke transaction、build request、preview projection，以及 World Partition/Streaming 的 observation/action surface。
- Runtime142 负责 runtime Terrain/Foliage/Streaming service、render、physics、navigation、residency 与 artifact execution；本报告不重复登记其 Runtime findings。
- Editor02/03/09 分别拥有共享 document/transaction、scene input/mode 与 background job authority；领域实现必须接入，不能复制简化版。
- Workbench 不得持有业务真相；Editor 不得直接修改 runtime 容器；source、artifact、resident state 不得共用同一类型或 generation。

## 9. 依赖有序重构里程碑

| 里程碑 | 必须交付 |
|---|---|
| M0 Truth Freeze | 隐藏或标记四张静态 Workbench；为缺资源、无 factory、DiagnosticOnly、`terrain: None`、固定 feedback 建 RED 证据 |
| M1 Publication | 原子发布 resource、operation/document/build/preview/mode provider；default linked/dynamic/builtin profile 矩阵 |
| M2 Source/Decoder | versioned Terrain/LayerStack/Foliage/Partition source；bounded RAW/R16/PNG Validate/Decode/Build |
| M3 Document/Save | Create/Open/Import/Save/Reopen、transaction、autosave、CAS、recovery、repair mode |
| M4 Terrain Mode | Height/Weight/Hole target、picking/capture/overlay、bounded delta、undo/redo/cancel、teardown |
| M5 Runtime Preview | 依赖 Runtime142 交付真实 instance/query/patch render/atomic generation swap；Editor 只观察真实结果 |
| M6 Incremental Build | dirty region 到 GPU/collision/nav/LOD/normal/foliage DAG；dedup/progress/cancel/LKG/shutdown |
| M7 Foliage/Scatter | stable prototype/rule、deterministic per-cell compile、override ownership、incremental rebuild、scale profile |
| M8 Partition/Streaming | manifest/cell/source/data-layer runtime 后，Editor 提供 source transaction、load/pin request、qualified state |
| M9 HLOD/Failure/Scale | hash-qualified HLOD、stale/LKG、visual compare、Nth-step failure、corrupt/oversize、large-world soak |
| M10 Competition | 固定硬件/驱动/场景/画质与 raw artifacts 比较 edit latency、GPU、VRAM、IO、LOD pop、stutter、stability |

## 10. G01-G32 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | 默认 Editor 不能 create/open/inspect 真实 Terrain，也无完整 disabled reason |
| G02 | Fail | 3 个资源缺失，publication 前无 schema/rollback gate |
| G03 | Fail | default linked/dynamic/builtin fallback profile matrix 缺失 |
| G04 | Fail | 5 个可见 operation 均无 production factory |
| G05 | Fail | 无 Terrain scene-mode lifecycle/teardown test |
| G06 | Fail | 无 RAW/R16/PNG decode corpus 与 malformed/decode-limit test |
| G07 | Pass | width/height sample count 使用 checked multiplication 与 fallible usize conversion |
| G08 | Partial | typed request 与部分 fail-close 存在；完整 Terrain/LayerStack semantic validation 缺失 |
| G09 | Fail | Create/Import 无 atomic write/catalog acknowledgement/failure rollback |
| G10 | Fail | 无 Height/Weight/Hole reversible stroke roundtrip |
| G11 | Fail | 无 active-stroke cancel、mode switch、disable、close、crash recovery matrix |
| G12 | Fail | source/GPU/collision/nav/LOD/normal generation 不一致 |
| G13 | Fail | 无 Scene load 到可见像素的真实 Terrain product test |
| G14 | Fail | 无 crack-free LOD fixed-camera/reference-image gate |
| G15 | Fail | 无 layer/hole/normal/shadow/depth multipath GPU validation |
| G16 | Fail | height query/raycast/collision/render surface 无 same-generation error gate |
| G17 | Fail | Nav 不消费 Terrain，无 dirty-tile/cancel/late-result gate |
| G18 | Fail | build/preview 未进入共享 job/residency/shutdown authority |
| G19 | Fail | 无 source revision/artifact generation/LKG/error position 产品链 |
| G20 | Fail | 无 Foliage/Scatter 跨 run/thread/incremental determinism |
| G21 | Fail | 无 changed-rule 仅失效相交 cell 的 artifact/invalidation gate |
| G22 | Fail | 无 manual/generated/override/exclusion save/rebuild/undo roundtrip |
| G23 | Fail | 无百万实例 CPU/GPU/VRAM/traversal budget |
| G24 | Fail | 无稳定 partition manifest/cell/dependency/data-layer/cost |
| G25 | Fail | 无多 streaming source、priority、pin、always-loaded、data-layer decision log |
| G26 | Fail | 无完整 cell state machine、ticket、exactly-one terminal |
| G27 | Fail | 无 IO/decompress/upload/attach Nth-step failure atomicity |
| G28 | Fail | Workbench cell/memory/HLOD 仍来自固定文本 |
| G29 | Fail | 无 HLOD source/settings hash、incremental build、stale/LKG |
| G30 | Fail | 无 typed headless convert/validate/scatter/HLOD report consumer |
| G31 | Fail | 无同内容同画质、可复现且保留 raw artifact 的竞争 benchmark |
| G32 | Fail | Windows/GPU/failure/large-world/plugin lifecycle/target-platform matrix 未通过 |

## 11. 禁止的临时修补

1. 禁止只补 3 个空 ZUI/TOML 文件就把 Terrain 标为 complete。
2. 禁止给 operation 挂永远成功的 factory、sleep job、固定 receipt 来绕过无 backend。
3. 禁止用空 Terrain pass、单 quad、普通 mesh wrapper 或 CPU debug mesh 声称 runtime 完成。
4. 禁止 Editor 维护独立 heightfield、foliage list、cell map、HLOD state 作为第二 authority。
5. 禁止把 control value、status text、row selection、toast 当成 document mutation/build/streaming 证据。
6. 禁止把 Vampire 的普通 terrain mesh 与 static grass batch 统计为 Terrain/Foliage backend。
7. 禁止每个 stroke 全图 clone、无界 inline height、无 budget decode 或无 cancel 长任务进入产品。
8. 禁止 scatter 只使用随机 seed 而无 stable cell/rule stream 与 deterministic artifact key。
9. 禁止 cell load request 缺 generation/ticket/cancel/terminal receipt，或 late result 覆盖新 World。
10. 禁止为了 Editor 进度复制 Physics/Nav/Render/Streaming 简化实现；必须等待 runtime owner typed contract。
11. 禁止保留旧路径 shim、compat module 或双写 source/artifact/runtime state；迁移按 hard cutover 执行。
12. 禁止在 G31/G32 通过前声称达到或优于 Unreal 的性能与表现。

## 12. 本轮完成定义

本轮仅完成 current-source review：逐项复核 Terrain plugin、Editor 产品面与共享 authoring 底座、Runtime downstream、open handoff 和五套本地参考源码；刷新 Editor16/92 的全部 P0/P1/P2，保持旧 ID 和状态；明确 Runtime142 与共享 Editor owner 边界；给出 M0-M10 依赖序和 G01-G32 资格门。

本轮没有修改 production code，没有关闭 implementation milestone，也没有用声明、fixture、静态 UI 或测试数量代替 source-to-runtime 产品证据。实施前必须重新冻结 working tree，并从 M0 capability truth 与失败测试开始；在此之前保持 `implementation_status: pending`。
