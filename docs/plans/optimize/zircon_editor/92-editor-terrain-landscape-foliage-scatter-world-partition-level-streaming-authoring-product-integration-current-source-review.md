---
title: Editor Terrain / Landscape / Foliage / Scatter / World Partition / Level Streaming Authoring 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor92
review_date: 2026-08-25
baseline_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
verification_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
related_code:
  - zircon_plugins/terrain
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_runtime_catalog
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/document
  - zircon_editor/src/core/jobs
  - zircon_editor/src/scene/modes
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/level_system.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor
  - dev/UnrealEngine/Engine/Source/Editor/FoliageEdit
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition
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

# Editor92 · Terrain / Landscape / Foliage / World Partition 当前源码复审

## 1. 结论

当前 Zircon Editor 没有可执行、可保存、可恢复的 Terrain / Landscape、Foliage / Scatter 或 World Partition / Level Streaming authoring 产品。仓库已有 Terrain typed asset carrier、插件 manifest、Editor operation descriptor、类型化 import plan、通用 document/transaction/job/scene-mode 基础，以及四张外观完整的 World Building Workbench；但这些部分没有被连成 `source -> document -> transaction -> build job -> immutable artifact -> runtime generation -> truthful projection` 的纵向闭环。

最严重的问题仍是产品真相失配。Terrain source 插件没有进入 first-party Editor/Runtime executable provider catalog 与 App 默认组合；插件注册引用的 `plugins://terrain/editor/authoring.zui`、`plugins://terrain/editor/terrain_component.zui` 和 `plugins://terrain/templates/default_heightfield.toml` 三份资源物理不存在。Import/Create/Open/Sculpt 虽有 operation descriptor，但 `EditorAuthoringContributionBatch` 不承载 operation factory，产品 dispatch 会得到 typed `MissingFactory`。插件 README 声称存在 Terrain scene mode，实际 contribution 依赖 `Default` 得到空 `scene_modes`，Editor 全仓也没有 Terrain mode。

四份 World Building ZUI 当前各 230 行，合计 **920 行、108 个 node、76 个直接 action 引用、0 个 provider**。它们固定显示 Summit Valley、64 cells、84K instances、128 clusters、SC_Forest、18 rules、96 cells 与 96 MB；路由只改变 workspace/tab/row/control state，command feedback 只写三项 UI property。按钮不再预置 selected/checked 是正确的局部 UI 修复，但没有 document、job ticket、artifact generation、runtime snapshot 或 cancellation receipt，不能把这些文本当成产品能力。

Terrain import planning 有真实进展：source format 已类型化为 RAW/R16/PNG，sample count 使用 checked multiplication 与 `usize::try_from`，LayerStack 被 fail-close 拒绝，扩展名 canonicalization 也更清晰。该进展只关闭旧 P1-14，并让旧 P1-15 进入 Partial；它仍不读取 bytes、不解码、不产生 asset/artifact、不进入 transaction。runtime importer 仍固定返回 `terrain heightfield importer backend is not installed`，dist 仍是 stateless registration manifest。

下游也没有补齐。`SceneAsset` 能承载 `SceneTerrainAsset` 引用，但 `World::from_scene_asset` 不消费它，`World::to_scene_asset` 仍固定写 `terrain: None`；Graphics 明确把 Terrain 放在 descriptor-only advanced slot，零 runtime pass；Physics HeightField 与 Terrain source 是独立真相；Navigation 不消费 Terrain；Foliage/Scatter 没有 source schema、compiler、runtime cluster；`LevelSystem` 只有整 World Loaded/Unloaded，不存在 manifest、cell、streaming source、data layer、HLOD 或 residency owner。Vampire 示例的可见地表来自普通 mesh，grass 来自普通 static mesh batch，不能作为 Terrain/Foliage 后端证据。

参考源码表明，工程差距不等于“缺几个笔刷按钮”。Unreal 的核心合同包含独立 Validate/Import/Export、Height/Weight/Visibility target、stroke transaction、Foliage instance ownership、运行时 cell 状态、streaming source shape/priority、data-layer effective state 与 hash-qualified HLOD build；Fyrox 规模更小，但已经有真实 Terrain node、raycast、background brush、chunk swap undo 和 quadtree selection。Godot、Unity Graphics 与 Bevy分别提供 heightmap physics、bulk instance、TerrainLit/GPU-driven 和 extract/visibility/batching参考，不应被误当成完整 World Editor authority。

本轮重判 **5 项 P0，全部 Open；60 项 P1 为 58 Open、1 Partial、1 Closed；12 项 P2，全部 Open；32 项资格门为 30 Fail、1 Partial、1 Pass**。没有动态证据支持 Zircon Terrain 的完成度、性能或表现优于 Unreal；在 canonical scenario、同画质 benchmark 与失败矩阵通过前，禁止作此声明。

## 2. 审查边界、统计与 currentness

### 2.1 冻结范围

统计口径为 working-tree 物理文件、物理行、非空行、bytes、Rust `#[test]` 与 `#[ignore]` 声明。fingerprint 对 repository-relative lowercase path 排序，为每个文件拼接 `path + NUL + lowercase(file SHA-256) + LF` 后再取 SHA-256。选择集用于复现本报告，不代表参考仓库全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Terrain 插件全量 | **16 / 1,072 / 971 / 38,904 / 10 / 1** | `0969623ed6793b55e014d0b50cced3b8e697e045b79f685684290b008d309366` |
| Editor 产品面、路由、catalog 与 App 接线 | **26 / 6,579 / 6,280 / 301,938 / 17 / 0** | `9f16dcfc29343546bcb0bf22baae5c449af413a5a272dd700581df0e283a90d2` |
| Editor document、operation、transaction、job 与 scene-mode 共享承接 | **125 / 22,420 / 20,205 / 746,518 / 202 / 8** | `881689da38b438ed740a5b81686f591427a224367c60e5c13d315696a97746b5` |
| Runtime downstream、catalog、Physics/Nav 与 Vampire oracle | **43 / 11,940 / 10,761 / 427,072 / 24 / 2** | `ab962df4e2d7f42a6a273870f4c7a22fcc0bf23d119ff6c28058b3d49314fed2` |
| Zircon selected union | **210 / 42,011 / 38,217 / 1,514,432 / 253 / 11** | `1513d31094c3b8351aa581bb31d1caab6699c11eae1dbc94a2f802f8914c91da` |
| Unreal selected | **20 / 24,591 / 20,619 / 863,428 / 0 / 0** | `ae307f0e8b46dec480788ce6840eea11c8fb406b1872572bf5478fea20bf78fe` |
| Fyrox selected | **5 / 4,543 / 4,160 / 184,106 / 14 / 0** | `7fd000723b56b225b5cfc798d516b36a750592db09a3e45895d773ee80541e59` |
| Godot selected | **6 / 1,247 / 1,008 / 51,003 / 0 / 0** | `9cb112813a0b990910920c63ee3997ca744503a18a31ecf67a1f240d4db885d2` |
| Unity Graphics selected | **7 / 6,309 / 5,298 / 307,228 / 0 / 0** | `119263ee923943f3a82c5e03da2da87595973db9dc7d9276b597b818f75839df` |
| Bevy selected | **6 / 5,458 / 4,988 / 214,626 / 1 / 0** | `72caf2bf8b6dea6bfa2e370e43f69a89fe83e9b043b8392435842979173a8af2` |
| 五引擎 reference union | **44 / 42,148 / 36,073 / 1,620,391 / 15 / 0** | `bfe6a7ee37c52a4997c254c6a07cd8c4be32a5c3ef02653ac9e4671107a1050d` |

Unreal 目录级规模沿用同日 Runtime142 复核：Runtime Landscape **143 文件 / 88,524 行**，LandscapeEditor **89 / 40,444**，Runtime Foliage **44 / 12,418**，FoliageEdit **49 / 12,138**，WorldPartition Public/Private **329 / 70,568**。这些数字只用于校准系统面，不代表大目录天然优于小实现。

### 2.2 currentness 与限制

- baseline 与 verification HEAD 均为 `8ee9411db24b7b4bdaf3fe028194642a7557c0b6`，commit time 为 `2026-08-25T17:37:22+08:00`。
- 210 个 Zircon selected 文件中有 **95 个**包含用户或其他 Session 的 working-tree 修改或新增内容；本轮逐份按物理文件读取，不回退、不覆盖，也不把在途状态写成已集成。
- 按用户要求未查询、轮询或等待协调器；本报告不依赖协调器 epoch。
- Tooling 按用户要求排除；未来迁移到 Rust 不改变 Runtime/Editor authority 判定。
- 本轮只做源码 review 和文档记录，未运行 Cargo、App、Editor、PIE、asset import/cook、GPU capture、physics/nav、streaming、fault、scale、soak 或 benchmark。

### 2.3 Owner 边界

- Editor92 唯一负责 Terrain/Foliage/Scatter authoring document、scene mode、stroke transaction、build request、preview projection，以及 World Partition/Level Streaming 的 observation/action surface。
- [Runtime142](../zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md) 负责 runtime Terrain/Foliage/Streaming service、render、physics、navigation、residency 与 artifact execution，本报告不重复登记其 72 项 Runtime P1。
- [Editor39](39-spline-path-road-river-decal-brush-geometry-authoring-review.md) 负责 road/river/spline/decal brush geometry；[Editor40](40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md) 负责 PCG/rule graph/biome generation；[Editor41](41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md) 负责 Data Layer/Level Instance/Outliner UX。
- [Editor02](02-document-transaction-save-autosave-recovery-review.md)、[Editor03](03-scene-prefab-selection-mode-gizmo-picking-review.md)、[Editor09](09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md) 分别保有共享 document/transaction、scene input 与 background job authority。

## 3. 当前产品链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| Package -> Catalog -> App | Terrain manifest 可被生成清单测试静态发现；first-party Editor/Runtime executable provider catalog 与 App 默认组合均未选择 Terrain | Open |
| Resource publication | 三份 `plugins://terrain/...` 声明资源不存在；publication 前没有完整 resource/factory 原子门 | Open |
| Editor operation | Import/Create/Open/Sculpt descriptor 存在；batch 无 factory，dispatch 会返回 `MissingFactory` | Open |
| Terrain scene mode | README 声称存在；registration 的 `scene_modes` 为空，全仓无 Terrain mode | Open |
| Import planning | RAW/R16/PNG typed plan、checked count、canonical extension、LayerStack fail-close 存在；不读取或解码 source | Partial |
| Runtime import | core TOML carrier/importer可用；Terrain plugin importer固定 DiagnosticOnly | Partial |
| Document/transaction | 通用 document、history、journal、job、scene-mode底座存在；Terrain无任何consumer/session/stroke | Open |
| Scene roundtrip | SceneAsset保存Terrain reference；World load不实例化，World save固定`terrain: None` | Open |
| Render/Physics/Nav | Terrain零pass；HeightField与Terrain分裂；Nav不消费Terrain | Open |
| Foliage/Scatter | 四张产品面有固定数据；无source/compiler/artifact/runtime cluster | Open |
| World Partition | LevelSystem只有整World lifecycle；无manifest/cell/source/data layer/HLOD | Open |
| Evidence | 静态feedback与普通mesh可见；无source-to-pixel/query/collision/nav/streaming canonical scenario | Open |

## 4. 必须保留的真实底座

1. `TerrainAsset`、`TerrainLayerAsset`、`TerrainLayerStackAsset`、`SceneTerrainAsset` 的 typed serde carrier 与 direct reference 表达。
2. Editor `TerrainHeightfieldSourceFormat`、checked sample count、canonical extension、LayerStack fail-close 与 typed diagnostic。
3. Runtime artifact store/cache/load、SceneAsset serializer/reference resolver，以及 last-good/atomic save 可复用基础。
4. Editor operation factory dispatch、document lifecycle、transaction engine、durable journal、job admission/progress/cancel 与 scene-mode stack。
5. Descriptor-only Terrain render slot对“未执行”的诚实表达；不能用一个空 pass 把它伪装成 complete。
6. Physics HeightField 的尺寸/样本/finite验证和 Navigation Recast基础；Terrain应提供canonical projection，不应复制平行backend。
7. 普通 mesh/material、GPU visibility/instance、asset residency与runtime snapshot接口；Foliage必须接共享能力而非另造无预算列表。
8. Workbench retained route/binding机制本身可保留，但数据必须来自provider snapshot，命令必须来自typed request/receipt。

## 5. P0：产品虚假可达与 authority 断裂

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| WB2-P0-01 | Open | Terrain不在默认catalog/App组合，三份资源缺失，但类型、菜单与Workbench让能力看似可达 | publication前原子验证package/resource/provider/factory/profile；缺失时整能力disabled并给typed reason |
| WB2-P0-02 | Open | Import/Create/Open/Sculpt仅metadata；无factory、document owner与Terrain scene mode | `TerrainAuthoringGateway`提供typed prepare/apply/cancel/undo/save；mode具备activate/input/overlay/teardown lease |
| WB2-P0-03 | Open | runtime backend为DiagnosticOnly，World不实例化Terrain，Graphics零pass | Runtime142建立per-World generation-bound Terrain service/artifact/runtime handle后，Editor只消费receipt/snapshot |
| WB2-P0-04 | Open | 四份Workbench发布固定build/instance/cell/memory/success文本 | 立即标为Prototype/Unavailable或隐藏；只有job终态与runtime snapshot可显示queued/completed/statistics |
| WB2-P0-05 | Open | 无统一WorldAuthoringDocument/partition authority，World export还会写`terrain: None` | 无损roundtrip前fail-close；document、manifest、runtime cell generation必须同一authority链 |

## 6. P1：工程级完整性差距

### 6.1 Package、capability 与产品装配

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| WB2-P1-01 | Open | manifest、runtime builtin metadata、Editor descriptor与default catalog没有单一装配truth | 建profile-qualified capability selection与selected provider receipt |
| WB2-P1-02 | Open | runtime标记partial，Editor入口不解释backend/resource/factory缺口 | readiness由typed capability diagnostic投影，不能由按钮存在推断 |
| WB2-P1-03 | Open | 三份插件资源URI未在publication前解析、读取与schema验证 | preflight全部资源，任一失败整batch回滚 |
| WB2-P1-04 | Open | builtin registry识别Terrain/TerrainLayerStack，插件缺失时却无只读诊断toolkit | 提供真实toolkit或明确disabled/repair surface |
| WB2-P1-05 | Open | editor/runtime dist没有authoring command bridge，native behavior仍stateless | ABI必须承载typed request/receipt或明确禁止动态authoring |
| WB2-P1-06 | Open | contribution batch不承载operation factory、document/build/preview provider | 扩展注册必须原子带齐metadata与execution owner |
| WB2-P1-07 | Open | menu/toolkit/view/importer/customization缺共享owner lease与原子rollback | 建package generation lease、revoke barrier和残余入口检查 |
| WB2-P1-08 | Open | disable/reload未定义open document、active stroke、dirty source与resident preview处置 | 定义quiesce、save/abort、last-good和unknown element保留策略 |
| WB2-P1-09 | Open | README的scene mode/runtime-backed声明与真实空注册无自动一致性门 | 文档声明从可验证capability manifest生成或由CI校验 |
| WB2-P1-10 | Open | tests只验证descriptor/plan形状，未启动默认产品或执行operation | 增加default bootstrap、resource resolution、dispatch和disable矩阵 |

### 6.2 Terrain source、import、layer 与 artifact schema

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| WB2-P1-11 | Open | import plan不读取bytes，不能验证文件长度/header/endianness/bit depth/channel/row order | 独立bounded `Validate -> Decode -> Build` pipeline，诊断携带source range |
| WB2-P1-12 | Open | PNG缺颜色类型、gamma/profile、alpha、interlace、decode limit与height range政策 | 只接受明确定义格式，拒绝隐式颜色转换和allocation bomb |
| WB2-P1-13 | Open | RAW/R16缺尺寸推断、stride、byte order、signedness、scale/offset/no-data/trailing bytes | 所有歧义进入显式ImportSettings与可复现hash |
| WB2-P1-14 | Closed | 旧实现的`width * height`无checked转换 | 当前已使用`u64::checked_mul`和`usize::try_from`；保持回归测试并补全局allocation budget |
| WB2-P1-15 | Partial | LayerStack仍复用heightfield request，但当前会fail-close拒绝 | 定义独立LayerStack source、channel packing、stable layer identity、normalization与resolution policy |
| WB2-P1-16 | Open | runtime只声明heightfield importer，Editor两个import入口与runtime manifest不对称 | source kind、importer、artifact kind、toolkit与runtime consumer一一对应 |
| WB2-P1-17 | Open | core Terrain validator允许零维、空samples、非有限/非正spacing/scale和无界样本 | 建versioned schema validator、project limits和bounded diagnostics |
| WB2-P1-18 | Open | layer没有stable ID、blend、physical material、tiling、visibility/lock或edit layer | 定义稳定LayerId与non-destructive layer graph |
| WB2-P1-19 | Open | layer name/strength/material/weightmap缺重复、range、kind、resolution与cycle验证 | semantic validator输出定位明确的error/warning |
| WB2-P1-20 | Open | LayerStack无validator、version/migration与canonical ordering | source migration与canonical serialization必须可重复 |
| WB2-P1-21 | Open | 大型高度数据仍是inline `Vec<Real>` | 切为tile/page source与derived artifact，支持compression/checksum/min-max/error/partial read |
| WB2-P1-22 | Open | source、derived tile与resident section没有不同类型/generation | stable source revision、artifact key、runtime generation禁止混用 |

### 6.3 Terrain document、scene mode、笔刷与保存

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| WB2-P1-23 | Open | 无Terrain document session、source revision、dirty set、history context、autosave或CAS | 建`TerrainAuthoringDocument`并接Editor02共享生命周期 |
| WB2-P1-24 | Open | Create Heightfield没有真实template、topology/range校验、占用检查与atomic write | typed create plan、staging、fsync/replace、catalog acknowledgement |
| WB2-P1-25 | Open | Open没有selection -> toolkit -> document -> preview factory链 | 支持normal、read-only repair、unsupported-version三类open结果 |
| WB2-P1-26 | Open | Sculpt没有mode factory、terrain picking、pointer capture、pressure/spacing/falloff/overlay | 建独立mode/session/tool/brush合同和无分配hot path |
| WB2-P1-27 | Open | Height/Weight/Hole target不存在，UI tab、selected layer与edit target可分裂 | 单一typed target address和generation-qualified selection |
| WB2-P1-28 | Open | stroke无bounded region delta、before image、inverse、merge key、base revision或cancel rollback | 一次stroke一个可逆transaction，支持chunk swap与冲突检测 |
| WB2-P1-29 | Open | source/GPU/collision/nav/LOD/normal/foliage无统一affected-region传播 | publish generation-bound dirty region DAG与consumer acknowledgement |
| WB2-P1-30 | Open | smooth/flatten/ramp/erosion/spline/copy-paste/resize缺deterministic算法合同 | tool schema、limits、transaction和golden corpus一并交付 |
| WB2-P1-31 | Open | 无non-destructive edit layer、lock/order/blend/merge/bake/migration | edit layer成为source真相，不把GPU结果反写成source |
| WB2-P1-32 | Open | multi-terrain seam、shared border、world transform/origin rebasing语义未定义 | 使用heightmap-space brush与明确边界ownership |
| WB2-P1-33 | Open | save无validated snapshot、atomic replace、last-good、recovery与partial cleanup | 接DocumentSaveCoordinator并为build artifact单独commit |
| WB2-P1-34 | Open | Workbench field edit只改control value，固定feedback不进入history | field绑定typed document property；command返回immutable receipt |

### 6.4 Runtime承接、render、physics、navigation 与性能接口

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| WB2-P1-35 | Open | SceneTerrainAsset只有引用，无transform/collision/nav/material/streaming/generation政策 | Runtime142定义stable scene instance contract，Editor只author source override |
| WB2-P1-36 | Open | scene load不创建Terrain component/service，descriptor不产生行为 | load/save/runtime query必须有同generation roundtrip |
| WB2-P1-37 | Open | graphics无extract/renderer/pass/patch/height/material consumer | Editor preview只连接真实runtime view，禁止独立mock terrain |
| WB2-P1-38 | Open | 无topology、quadtree/clipmap、screen error、neighbor constraint/morph/skirt | runtime owner提供可诊断LOD snapshot与crack-free门 |
| WB2-P1-39 | Open | Terrain visibility未接frustum/occlusion/GPU Scene，bounds无增量更新 | runtime暴露source-qualified culling/residency stats |
| WB2-P1-40 | Open | splat/hole/normal/macro/VT与shadow/depth/ray paths无合同 | preview profile必须标注实际支持矩阵和fallback |
| WB2-P1-41 | Open | CPU/GPU query、raycast、collision与render surface无generation一致性 | Editor picking与runtime query消费同一artifact generation |
| WB2-P1-42 | Open | nav无dirty tile rebuild/cancel/commit/hole/slope/layer cost合同 | Editor显示真实Nav build ticket，不伪造Terrain成功 |
| WB2-P1-43 | Open | build/upload未接asset streaming、resource budget、device recovery与shutdown fence | 所有长任务进入Editor09/Runtime residency authority |
| WB2-P1-44 | Open | 无同画质frame/GPU/VRAM/IO/LOD pop/edit latency/stutter基线 | 固定硬件、驱动、场景、warmup、raw artifact后才做比较 |

### 6.5 Foliage、Scatter、实例与植被 authoring

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| WB2-P1-45 | Open | 无Foliage/Scatter source asset、versioned schema、AssetKind、importer、toolkit或runtime service | 定义Prototype/RuleSet/Override source与compiled cell artifact |
| WB2-P1-46 | Open | Foliage Workbench的84K/128 clusters与类型全是静态文本 | provider snapshot必须携带source/runtime generation和freshness |
| WB2-P1-47 | Open | Scatter Workbench的18 rules/64K/conflict无document/generator/validator | command连接deterministic compile job与structured diagnostic |
| WB2-P1-48 | Open | 无stable rule/prototype ID、seed stream、density/mask/slope/scale/collision/exclusion schema | schema hash纳入artifact key，随机流按cell/rule稳定分片 |
| WB2-P1-49 | Open | 无per-cell增量scatter artifact、dependency key/invalidation/cache/rollback | changed region只重建相交cell，失败保留last-good |
| WB2-P1-50 | Open | 手摆/generated/override/exclusion无ownership | 明确instance provenance与regenerate/undo/save语义 |
| WB2-P1-51 | Open | runtime无cluster bounds、GPU instance、culling、LOD/impostor/wind/shadow/budget | 复用共享GPU-driven/residency owner并提供bounded snapshot |
| WB2-P1-52 | Open | 无百万实例确定性、分布、约束、延迟、traversal和跨平台预算 | 建scale/soak/profile/visual quality qualification |

### 6.6 World Partition、Level Streaming、Data Layer 与 HLOD

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| WB2-P1-53 | Open | 无partition asset/manifest、stable cell ID、grid/bounds/content owner/version | canonical `WorldPartitionManifest`由source revision确定生成 |
| WB2-P1-54 | Open | LevelSystem只有整World Loaded/Unloaded | runtime cell至少有desired/queued/loading/preparing/attached/resident/unloading/failed/cancelled |
| WB2-P1-55 | Open | 无streaming source shape/priority、多source merge、always-loaded/pin/data layer/condition | typed source query与可解释desired-set decision log |
| WB2-P1-56 | Open | 无cell dependency、cross-cell reference、external actor/content bundle与atomic move | source ownership和reference fixup必须transactional |
| WB2-P1-57 | Open | IO/decompress/deserialize/upload/attach无ticket、budget、cancel ack、backpressure与shutdown终态 | runtime提供每阶段receipt和恰一终态 |
| WB2-P1-58 | Open | Level Streaming Workbench的96 cells/96 MB/HLOD_04无manifest或request | 投影generation-qualified manifest/residency snapshot与真实load request |
| WB2-P1-59 | Open | 无HLOD layer/builder/key/artifact/incremental rebuild/quality/transition/last-good | source/component/settings hash决定build；stale与last-good显式分离 |
| WB2-P1-60 | Open | 无partition convert/validate/minimap/data-layer/HLOD headless workflow与large-world package测试 | 实施归Rust tooling后，Editor只消费typed job/report，不嵌入脚本捷径 |

## 7. P2：成熟度、协作与可诊断性差距

| ID | 状态 | 当前差距 |
|---|---|---|
| WB2-P2-01 | Open | 无per-user brush preset、falloff/alpha library、pressure curve、overlay密度偏好 |
| WB2-P2-02 | Open | 无Terrain camera、streaming source、LOD/weight/cell debug bookmark |
| WB2-P2-03 | Open | 无最近Terrain/layer/rule/cell与跨document导航历史 |
| WB2-P2-04 | Open | 无height/weight/hole/edit-layer revision diff、heatmap与side-by-side compare |
| WB2-P2-05 | Open | 无stroke/rule/cell/HLOD comment、tag、review note与审批状态 |
| WB2-P2-06 | Open | 无layer/prototype/rule/data-layer/HLOD批量编辑与usage query |
| WB2-P2-07 | Open | 无bounded support bundle导出source、diagnostic、dirty region、artifact、streaming journal和capture引用 |
| WB2-P2-08 | Open | 无opt-in edit latency、cache hit、resident cells、IO/upload、LOD transition telemetry |
| WB2-P2-09 | Open | 无dry-run、批量reimport、partition validate、scatter rebuild与HLOD stale报告 |
| WB2-P2-10 | Open | 无typed Editor scripting/remote automation Terrain/Scatter/Partition surface |
| WB2-P2-11 | Open | 无多人terrain region/cell lock、冲突可视化、ownership transfer与revision annotation |
| WB2-P2-12 | Open | 无色盲安全weight palette、键盘笔刷、screen-reader cell table和完整i18n/unit formatting |

## 8. 历史台账重判

[Editor16](16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md) 继续是初始架构审查，本报告只取代其 currentness，不删除历史证据。

| 历史范围 | 当前重判 | 原因 |
|---|---:|---|
| 5 项 P0 | **5 Open** | catalog/resource/factory/runtime/Workbench/world authority五条主断点均未闭合 |
| 60 项 P1 | **58 Open / 1 Partial / 1 Closed** | P1-14 checked sample count已关闭；P1-15因LayerStack fail-close为Partial；其余未形成产品链 |
| 12 项 P2 | **12 Open** | 无成熟度产品consumer |
| 32 项 Gate | **30 Fail / 1 Partial / 1 Pass** | G07 overflow门Pass；G08 schema validation仅Partial；其余Fail |

Runtime事实以 Runtime142 为准；Editor92不得通过增加Editor mock、复制runtime状态或维护第二份terrain数据来“关闭”运行时问题。

## 9. 五套参考源码的工程映射

| 参考 | 本轮读到的关键合同 | Zircon必须吸收 | 不应照搬 |
|---|---|---|---|
| Unreal LandscapeEditor | brush/stroke生命周期；heightmap-space interactor；Height/Weight/Visibility target；unloaded component overlap；独立Validate/Import/Export；edit layer与scoped update | source validation与decode分离、target authority、bounded stroke、transaction、跨component invalidation和failure diagnostics | UObject/Slate宏、历史兼容与完整UE工具数量 |
| Unreal FoliageEdit/Runtime Foliage | brush开始/结束transaction；Add/Remove/Select/Reapply instance；instance hash、type settings与editor subsystem | instance provenance、可逆brush、stable prototype/rule、spatial index、重生成保留override | Actor/UObject ownership细节 |
| Unreal WorldPartition | cell `Unloaded/Loaded/Activated`；Load/Unload/Activate/Deactivate；streaming shape/range/priority/target；data layer effective state；HLOD settings/source hash与builder result | manifest/cell/source/data-layer/HLOD分层、hash-qualified incremental build、可解释state与warmup/transition | UE package/external actor历史布局 |
| Fyrox | Terrain node、R32F height资源、hole/layer、raycast；background brush thread；height/layer/hole target；chunk swap execute/revert；quadtree selection | 先达到小而真的authoring/runtime闭环：real node、real brush、undo、raycast、LOD与测试 | 单一实现的规模上限与UI具体结构 |
| Godot | HeightMapShape3D的physics/debug mesh与MultiMesh bulk instance/visible count | heightfield物理边界和大批实例资源接口参考 | 将其误当完整Terrain/World Editor |
| Unity Graphics | HDRP TerrainLit、hole/material路径、TerrainToMesh ray path、GPUResidentDrawer/InstanceData/InstanceCuller | render material ABI、GPU instance data/culling/residency的consumer设计 | Editor/World authority不在该checkout，不补推不存在的合同 |
| Bevy | extract/visibility range、GPU/no-GPU preprocessing与batching | ECS extraction、visibility与batching作为共享下游能力 | Bevy没有内建Landscape authoring，不应拿render primitive替代产品 |

## 10. 目标架构与硬切合同

### 10.1 唯一纵向数据流

```text
TerrainSource / LayerStackSource / FoliageRuleSet / PartitionSource
    -> Editor AuthoringDocument + stable element address
    -> reversible transaction + dirty region / changed cell set
    -> bounded BuildJob(request, source_revision, profile)
    -> immutable BuildArtifact + structured diagnostics + dependency hash
    -> Runtime World service atomic generation swap
    -> generation-qualified query / residency / render / physics / nav snapshot
    -> Editor projection and command receipt
```

禁止 Workbench 直接持有业务真相；禁止 Editor直接改runtime容器；禁止 Runtime反读Editor控件值；禁止source、artifact和resident state共享同一类型或generation。

### 10.2 Editor-owned contracts

- `TerrainAuthoringDocument`：source revision、layer/tool target、dirty regions、history namespace、save/CAS/recovery。
- `TerrainEditModeSession`：document lease、runtime preview generation、pointer capture、brush state、stroke transaction与teardown。
- `TerrainStrokeDelta`：bounded tile/region、before/after或swap payload、base revision、affected consumers和merge key。
- `FoliageAuthoringDocument`：prototype/rule/override/exclusion stable identity与deterministic seed policy。
- `WorldPartitionAuthoringDocument`：grid/cell/content ownership source；不复制Runtime resident state。
- `WorldBuildRequest/Receipt`：job ticket、source revision、profile、changed cells、artifact generation、terminal diagnostic。
- `WorldAuthoringSnapshot`：只读投影runtime cell/residency/HLOD状态，携带session、world、generation、sequence与freshness。

### 10.3 Runtime-owned contracts

Runtime142负责 `TerrainBuildArtifact`、`TerrainRuntimeHandle`、`FoliageCellArtifact`、`WorldPartitionManifest`、`StreamingSource`、`RuntimeCellHandle/State`、`HLODArtifact`、query/render/physics/nav/residency。Editor只能通过typed gateway提交request、观察receipt/snapshot；不能为了预览复制简化Terrain backend。

### 10.4 Publication 与卸载

package publication必须一次性验证资源、schema、toolkit、operation factory、scene-mode factory、import/build provider和目标runtime capability。卸载前先冻结新请求，再取消/完成active stroke与jobs，处理dirty document，等待provider reader lease归零，最后撤销入口；任何中间失败不得留下半注册UI。

## 11. 依赖有序重构里程碑

### M0 · Truth Freeze 与 RED 证据

隐藏或标记四张静态Workbench为Prototype/Unavailable；为缺资源、MissingFactory、DiagnosticOnly、`terrain: None`与固定feedback建立失败测试和source guards。

### M1 · Package、resource 与 factory 原子发布

补真实资源或移除声明；将operation/document/build/preview/mode provider纳入同一generation lease；默认catalog/App profile有可执行bootstrap矩阵。

### M2 · Canonical source、validator 与 decoder

定义versioned Terrain/LayerStack/Foliage/Partition source；实现bounded RAW/R16/PNG Validate/Decode/Build；输出stable diagnostics与artifact key。

### M3 · Terrain document、save 与 recovery

接入Editor02 document/transaction/save/autosave/CAS/recovery，完成Create/Open/Import/Save/Reopen和坏资产repair路径。

### M4 · Terrain scene mode 与可逆stroke

实现Height/Weight/Hole target、picking、capture、brush overlay、bounded delta、undo/redo/cancel，以及mode/plugin/document teardown。

### M5 · 最小真实Runtime preview

依赖Runtime142先交付source-to-runtime instance、height query、patch render与atomic generation swap；Editor preview只观察真实结果。

### M6 · Incremental build 与跨域失效

统一dirty region到GPU/collision/nav/LOD/normal/foliage的DAG；所有job支持dedup、progress、cancel acknowledgement、last-good与shutdown。

### M7 · Foliage/Scatter authoring

交付stable prototype/rule、deterministic per-cell compile、manual/generated/override/exclusion ownership、incremental rebuild与百万实例profile入口。

### M8 · World Partition 与 streaming observation/action

交付manifest/cell/source/data-layer runtime后，Editor实现source document、cell move transaction、load/pin request与generation-qualified状态视图。

### M9 · HLOD、failure 与规模化

接入hash-qualified HLOD build、stale/last-good、visual compare、warmup/transition；覆盖Nth-step failure、corrupt/oversize、unload/reload、device loss、large-world soak。

### M10 · 竞争资格

固定版本、硬件、驱动、场景、画质与raw artifacts，对Unreal/Fyrox测量edit latency、frame/GPU time、VRAM、IO、LOD pop、camera stutter和稳定性。统计显著领先前不得宣称性能或表现领先。

## 12. G01-G32 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | 默认Editor不能create/open/inspect真实Terrain；缺backend时也没有完整disabled reason |
| G02 | Fail | 三份插件资源不存在，publication前无schema/rollback门 |
| G03 | Fail | default linked/dynamic/builtin fallback profile矩阵缺失 |
| G04 | Fail | 四个可见operation均无生产factory |
| G05 | Fail | 无Terrain scene mode生命周期与teardown测试 |
| G06 | Fail | 无RAW/R16/PNG decode corpus与malformed/decode-limit测试 |
| G07 | Pass | width/height sample count使用checked multiplication与fallible usize转换 |
| G08 | Partial | typed request与部分fail-close存在；完整Terrain/LayerStack semantic schema validation缺失 |
| G09 | Fail | Create/Import无atomic write/catalog acknowledgement/failure rollback |
| G10 | Fail | 无Height/Weight/Hole可逆stroke roundtrip |
| G11 | Fail | 无active stroke cancel、mode switch、disable、close、crash recovery矩阵 |
| G12 | Fail | source/GPU/collision/nav/LOD/normal generation不一致 |
| G13 | Fail | 无Scene load到可见像素的真实Terrain产品测试 |
| G14 | Fail | 无crack-free LOD固定camera path与reference image |
| G15 | Fail | 无layer/hole/normal/shadow/depth多path GPU验证 |
| G16 | Fail | height query/raycast/collision/render surface无同generation误差门 |
| G17 | Fail | Nav不消费Terrain，无dirty tile/cancel/late-result门 |
| G18 | Fail | Terrain build/preview未进入共享job/residency/shutdown authority |
| G19 | Fail | 无source revision/artifact generation/last-good/error position产品链 |
| G20 | Fail | 无Foliage/Scatter跨run/线程/增量确定性 |
| G21 | Fail | 无changed-rule仅失效相交cell的artifact/invalidation门 |
| G22 | Fail | 无manual/generated/override/exclusion save/rebuild/undo roundtrip |
| G23 | Fail | 无百万实例CPU/GPU/VRAM/traversal预算 |
| G24 | Fail | 无稳定World Partition manifest/cell/dependency/data-layer/cost |
| G25 | Fail | 无多streaming source、priority、pin、always-loaded与data-layer决策日志 |
| G26 | Fail | 无完整cell状态机、ticket与恰一终态 |
| G27 | Fail | 无IO/decompress/upload/attach Nth-step failure原子性 |
| G28 | Fail | Workbench cell/memory/HLOD仍来自固定文本 |
| G29 | Fail | 无HLOD source/settings hash、增量build、stale与last-good |
| G30 | Fail | 无Rust headless convert/validate/scatter/HLOD typed report consumer |
| G31 | Fail | 无同内容同画质、可复现且保留raw artifact的竞争benchmark |
| G32 | Fail | Windows/GPU/failure/large-world/plugin lifecycle/target-platform发布矩阵未通过 |

## 13. 禁止的临时修补

1. 禁止只补三份空ZUI/TOML资源就把Terrain能力标为complete。
2. 禁止给operation挂一个永远成功的factory、sleep job或固定receipt来绕过`MissingFactory`。
3. 禁止新增空Terrain render pass、单quad/普通mesh wrapper或CPU debug网格来声称runtime完成。
4. 禁止Editor维护独立heightfield、foliage list、cell map或HLOD状态作为第二authority。
5. 禁止用control value、status text、row selection或toast作为document mutation/build/streaming完成证据。
6. 禁止把Vampire普通terrain mesh与Static Grass Batch统计成Terrain/Foliage backend。
7. 禁止全图clone作为每次stroke历史、无界inline高度数据、无budget decode或无cancel长任务进入产品。
8. 禁止以随机seed但非stable cell/rule stream的scatter结果通过“看起来合理”验收。
9. 禁止cell load请求没有generation/ticket/cancel/terminal receipt，或late result覆盖新World。
10. 禁止为了Editor进度复制Physics/Nav/Render/Streaming简化实现；必须等待Runtime owner并通过typed contract接入。
11. 禁止保留旧路径shim、compat module或双写source/artifact/runtime state；迁移按hard cutover执行。
12. 禁止在G31、G32未通过前写“达到/优于Unreal”的性能或表现结论。

## 14. 本轮完成定义

本轮只完成current-source review：冻结并扫描Terrain插件、Editor产品面与共享authoring承接、Runtime downstream和五引擎参考；刷新Editor16全部P0/P1/P2；明确与Runtime142、Editor39/40/41等owner边界；给出M0-M10依赖序与G01-G32资格门。没有修改生产代码，没有关闭implementation milestone，也没有用静态测试数量代替产品证据。

后续实施开始前必须重新读取上述owner文档、重新取working-tree fingerprint，并先完成M0 capability truth与RED证据；在此之前，本报告状态保持 `implementation_status: pending`。
