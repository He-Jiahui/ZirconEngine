---
title: Editor Terrain / Landscape / World Partition 当前工作树复审
category: zircon_editor
report_id: Editor232
review_date: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/172-runtime-terrain-landscape-world-partition-current-working-tree-authority-render-physics-navigation-review.md
related_code:
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/terrain/editor/src/authoring.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/capability.rs
  - zircon_plugins/terrain/editor
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/terrain_and_foliage.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
tests:
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - zircon_editor/src/tests/workbench/reference_surface.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/172-runtime-terrain-landscape-world-partition-current-working-tree-authority-render-physics-navigation-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor
  - dev/UnrealEngine/Engine/Source/Editor/FoliageEdit
  - dev/UnrealEngine/Engine/Source/Editor/WorldPartitionEditor
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/bevy/crates/bevy_render/src/view/visibility
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Terrain / Landscape / World Partition 当前源码工程化差距

## 1. 结论

Terrain editor plugin 已注册 Terrain authoring surface、component drawer、heightfield/weightmap importer、heightfield creation template，以及 Import/Create/Open/Sculpt 五个 operation descriptor。`authoring.rs` 还能对 raw/r16/png 的扩展名、尺寸和 sample count 做一次 typed plan。这些是入口与验证骨架，应保留。

但当前 Editor 没有可执行 Terrain authoring 产品。`plugin.rs` 引用的 `plugins://terrain/editor/authoring.zui`、`plugins://terrain/editor/terrain_component.zui`、`plugins://terrain/templates/default_heightfield.toml` 在当前工作树均不存在；插件目录只有 Rust/TOML，没有这些 UI/template 资源。operation batch 没有 factory/handler/document owner，测试仅断言 command registration 和空 menu；不存在 terrain scene-mode factory、brush input、reversible edit delta、asset byte importer、build job、compiled artifact 或 runtime install receipt。

Core Workbench 另有一套 Terrain workspace，标记为 `visibility = "collapsed"`，固定显示 `Summit Valley`、`Heightfield Ridge`、`Layer_Rock`、`A12_08/A12_09`、`LOD 3`、`Terrain cells ready 2 warnings` 和 `Radius: 512 / Strength: 0.38`。callback/navigation/feedback routes 可以改变选中状态和文本，但没有连接 Terrain document、World Partition manifest、runtime generation、GPU preview、Physics/Nav bake 或真实 streaming snapshot。不能把这套 UI 视为 plugin authoring 的实现。

本轮刷新 Editor138/16，不重复其既有 5 项 P0；新增 **18 项 P1、8 项 P2、18 项资格门**，当前裁决为 **14 Fail / 4 Partial / 0 Pass**。在 operation factory、资源、document、artifact、PreviewWorld 和 truthful runtime receipt 闭合前，不应开放菜单或展示 build/preview 成功。

## 2. 审查边界与冻结统计

沿 `plugin descriptor -> authoring batch -> operation dispatch -> resource URI -> ZUI/workbench -> document/history/job -> asset import/build -> Scene/PIE -> runtime install/preview` 逐段读取，并对照 Unreal LandscapeEditor/FoliageEdit/WorldPartitionEditor、Fyrox terrain interaction/commands、Godot height map、Unity HDRP Terrain/GPUDriven 与 Bevy render-world ownership。未运行 Cargo、Editor、PIE、真实 file import、GPU preview、cook、streaming 或 benchmark。

| 范围 | files | lines | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| `zircon_plugins/terrain` 全量 | 16 | 968 | 38,580 | 10 | 1 | `e59fb4861fd39b49058175fc4d4d9cdd162febd67095fa0aa2cc0262fb219639` |
| Editor Terrain plugin 源码（editor/dist/runtime 关联） | 10 | 821 | 31,995 | 10 | 1 | recomputed from the current plugin slice |
| Terrain Workbench workspace + navigation/feedback routes | 选定当前资源与 Rust route | 未将固定 fixture 行数伪装成执行统计 | n/a | n/a | n/a | source recheck required |

三份插件资源的 `Test-Path` 均为 false。指纹按路径排序、逐文件 SHA-256 manifest 计算；实施前必须重取，且不得把静态测试或 fixture 数字当动态通过。

## 3. 可保留基础

| 项目 | 当前事实 | 后续处理 |
|---|---|---|
| registration | descriptor、capability、asset importer、toolkit、creation template 和 stable operation ids 存在 | 保留 ID，补 factory、handler、permission、lease、receipt |
| import plan | typed `TerrainHeightfieldImportRequest`、raw/r16/png canonical extension、checked sample count、LayerStack fail-close | 升级为真实 byte decoder、layer-aware request、source span/format/endianness diagnostics |
| generic Editor core | document/history/job/scene-mode/overlay 基础可复用 | Terrain 不另建平行 store；接入 typed mutation、cancel、atomic artifact swap |
| Workbench layout | 左侧 mode/asset、中心 cell/build、右侧 brush 参数的布局 skeleton 可作为 UX 起点 | 删除固定数据与 collapsed-only 展示，绑定 provider/document/runtime snapshot |
| optional policy | Terrain package beta/partial，Editor capability 可被 host 检查 | 保持 fail-close，availability 必须反映实际 resources/provider/artifact |

## 4. 参考引擎裁决

Unreal LandscapeEditor/FoliageEdit/WorldPartitionEditor 不是一排按钮，而是 World/asset/document/transaction、brush stroke、edit layer、selection、preview scene、build/cook、cell/HLOD、source/lease 与 runtime generation 的共同系统。Fyrox 的 terrain interaction 和 command 已有真实鼠标 stroke、selection、undoable command 与 chunk update。Godot/Unity 的 Terrain authoring 也以 height/layer/material/holes/visibility/bounds 的持久化资源驱动 runtime representation。Zircon 当前的 operation metadata 与 Workbench fixed rows 没有这些 owner 边界。

## 5. P1 差距与重构要求

| ID | 当前证据 | 工程化重构 |
|---|---|---|
| ED-TER-01 | `plugin.rs:45-46` 引用不存在的 `authoring.zui`；inspector 引用不存在的 `terrain_component.zui` | 将资源纳入插件 package/asset registry，启动时做 URI existence/schema/fingerprint 检查；缺失时隐藏入口并给 capability diagnostic |
| ED-TER-02 | creation template 引用不存在的 `plugins://terrain/templates/default_heightfield.toml` | 提供 versioned source template 与 content-addressed creation transaction，创建后真实打开 document、写 source、建立 artifact dependency |
| ED-TER-03 | plugin authoring 目录没有任何 ZUI；仅 core Workbench 有另一套 workspace | 明确 plugin surface 与 Workbench owner，二者共享 document/provider，不允许两套互不相干 UI |
| ED-TER-04 | 五个 operation 只有 `EditorCommandDescriptor`，无 factory/handler；tests 只断言 registration/menu empty | 为 import/create/open/sculpt/weightmap 注册 typed factory、payload schema、context validation、undo transaction、job handle、cancel 与 result receipt |
| ED-TER-05 | `authoring.rs` 只做扩展名/尺寸/sample count 检查，不读取 raw/r16/png bytes | 建真实 importer：bit depth、endianness、PNG channel/color-space、NaN/range、height scale、holes、normal、checksum 和 streaming source metadata |
| ED-TER-06 | LayerStack import 被 heightfield plan 拒绝，weightmap 仍复用 raw/r16/png 扩展 descriptor | 设计 layer-aware source schema（channel packing、format、material/layer identity、strength、order），拒绝不完整请求并提供 fix-it |
| ED-TER-07 | 没有 `TerrainDocument`、stable cell/layer identity、revision、selection 或 dependency graph | 建 source document/partition document、stable GUID、revision/dirty state、layer/cell dependency、merge/reload/repair 与 session lease |
| ED-TER-08 | sculpt operation 没有 brush input、stroke sampling、falloff、symmetry、mask、layer target 或 reversible delta | 建 deterministic brush command/stroke journal、chunk-local delta、preview overlay、undo/redo、cancel/atomic swap 和 conflict policy |
| ED-TER-09 | Terrain authoring batch 的 `scene_modes` 默认空；没有 scene mode factory/input effect/overlay provider | 注册 Terrain mode、viewport picking/raycast、brush cursor、gizmo/height readout、overlay retirement 与 mode teardown |
| ED-TER-10 | Workbench 固定 Summit Valley/A12_08、LOD、cell count、memory、warnings；核心 root collapsed | 所有 row/table/stat 由 document/build/streaming provider 投影；没有 snapshot 时显示 unavailable，而不是固定成功 fixture |
| ED-TER-11 | Workbench preview/build routes 只投影 action/feedback 文本，没有 job/artifact/runtime consumer | 绑定 `OperationReceipt`：queued/running/progress/succeeded/failed/cancelled/stale，携带 source revision、artifact generation、cell set 与 diagnostics |
| ED-TER-12 | 没有 Terrain PreviewWorld/PreviewScene/terrain renderer/physics/nav query | 建独立 PreviewWorld，消费同一 compiled terrain artifact，提供 camera/LOD/material/height query/collision/nav preview 与 stale generation fence |
| ED-TER-13 | inspector customization URI 不存在，runtime component descriptor 只有 terrain/layers 两个 asset refs | 生成真实 component inspector：asset/layer stack、transform/origin、bounds、LOD/streaming/physics/nav/foliage profile，并 round-trip 到 Scene/Prefab/PIE |
| ED-TER-14 | save/open/build 没有 source -> compiler -> artifact -> runtime install 闭环；runtime 世界保存 terrain 为 None | Editor save 必须 atomic source publish + validation + build artifact + install receipt；失败保留 dirty/last-good，禁止报告 saved |
| ED-TER-15 | 没有 cell graph、World Partition manifest、data layer、streaming source、HLOD/Nanite build UI | 建 versioned partition document、cell coordinates/keys、source priority、data-layer/HLOD dependencies、cook/build graph、load/unload/retire state |
| ED-TER-16 | 没有 foliage/scatter authoring model，Terrain layer 不能驱动 prototype/cluster/seed | 建 deterministic scatter document/rule graph、prototype/material/cluster/LOD/wind/physics/nav policy，与 Terrain artifact generation 绑定 |
| ED-TER-17 | 没有 capability/device profile、memory/IO/GPU budget、large-world/origin-rebase editor validation | 预览 build 使用 runtime capability query、budget admission、quality tier、origin/precision diagnostics、device variant artifact；失败给明确 recovery |
| ED-TER-18 | plugin tests 只有 registration/import-plan/static performance gate（1 ignored），没有 UI->document->build->runtime 测试 | 增加 required test matrix：resource existence、operation factory、byte decode、stroke undo、Scene round-trip、preview pixel/query、artifact install、cell streaming、device/fault/stress/product acceptance |

## 6. P2 差距与重构要求

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ED-TER-19 | fixed row/table 无虚拟化，无法表示大规模 cell/layer/diagnostic | 采用虚拟列表、stable key、filter/search、stale cursor 与 selection rebase |
| ED-TER-20 | brush 参数只显示字符串 `Radius: 512`/`Strength: 0.38`，单位和 precision 未定义 | 使用 typed numeric fields、units、snap/validation、per-project precision 和 undo coalescing |
| ED-TER-21 | 只有 Sculpt/Paint/Streaming tabs，没有 edit layer/holes/erosion/visibility/material/foliage/collision/nav 分层状态 | 按 source/runtime owner 拆分可加载 panels，避免将所有功能塞进一张 fixture workspace |
| ED-TER-22 | 没有 source-vs-artifact diff、cell dirty graph、last-good recovery | 提供 generation diff、per-cell dirty reason、rebuild dependency graph 与 rollback |
| ED-TER-23 | 没有 GPU preview profiling、LOD/page/cell residency 和 memory breakdown | 接 runtime telemetry、GPU timing、page/cell residency、budget/eviction 视图 |
| ED-TER-24 | routes/feedback 没有 principal/lease/remote policy/audit | 统一 operation authorization、document lease、audit event 与 local/remote capability |
| ED-TER-25 | 多 viewport/PIE/thumbnail 不共享 preview artifact，容易重复 cook/状态漂移 | artifact registry + PreviewWorld snapshot cache + viewport generation key |
| ED-TER-26 | static source assertions 与 ignored performance gate 不能证明真实工程行为 | 用 integration/golden/stress/fault tests 替换 source-shape 作为主要证据 |

## 7. 资格门

| Gate | 必须证明 |
|---|---|
| ED-TER-G01 | 所有 declared plugin URI 真实存在、版本化、可打包、可解析，资源缺失时入口 fail-close |
| ED-TER-G02 | operation factory/handler/payload/capability/lease/job/receipt 对 Import/Create/Open/Weightmap/Sculpt 全部可追踪 |
| ED-TER-G03 | Terrain source/document/cell/layer identity、revision、selection、undo/redo/reload/merge 稳定 |
| ED-TER-G04 | raw/r16/png/layer source 真实解码并有 format/endian/channel/finite/sample diagnostics |
| ED-TER-G05 | brush stroke deterministic、可取消、可恢复，chunk delta 与 undo/redo 不丢失 |
| ED-TER-G06 | Terrain scene mode 有 picking/raycast/brush input/overlay/gizmo 与 teardown owner |
| ED-TER-G07 | Workbench 所有可见数值来自 provider/document/job/runtime snapshot，不来自固定 fixture |
| ED-TER-G08 | PreviewWorld 使用 runtime 相同 artifact、clock、LOD、material、bounds、physics/nav query |
| ED-TER-G09 | source revision -> build -> artifact -> install/rollback receipt 可审计，不能只显示 saved/ready |
| ED-TER-G10 | Scene/Prefab/PIE/component inspector 保存和加载 terrain reference/layer/origin/generation 无损 |
| ED-TER-G11 | partition manifest/cell/data-layer/HLOD/streaming source 的 editor mutation 与 runtime state 同 generation |
| ED-TER-G12 | foliage/scatter rule/prototype/seed/cluster/LOD 与 terrain layer artifact 可重现 |
| ED-TER-G13 | device/quality/CPU-GPU/large-world budgets 在 editor preview 与 runtime profile 一致 |
| ED-TER-G14 | failure/cancel/stale/missing asset/device loss/overflow 都有 structured diagnostic 和 recovery UI |
| ED-TER-G15 | multi-viewport/PIE/thumbnail/remote session 使用正确 artifact lease，不重复或交叉污染状态 |
| ED-TER-G16 | test matrix 覆盖 document/import/stroke/preview/build/install/stream/physics/nav/render/fault/stress |
| ED-TER-G17 | unavailable/partial Terrain 不显示可执行 build/preview/streaming 成功文案或静态统计 |
| ED-TER-G18 | 真实产品场景能从 create/import 到 Scene attach、PIE、render/collision/nav/stream、save/reload 全链闭环 |

## 8. 推荐实施顺序

1. 先补齐 package 资源与 typed source/document schema，注册 operation factory、transaction、Scene mode 和 truthful capability policy。
2. 实现 byte importer、brush delta、artifact compiler、Terrain component round-trip 与 atomic save/install。
3. 建 PreviewWorld 和真实 runtime provider，接 patch/LOD/material/physics/nav，替换 collapsed fixed Workbench 数据。
4. 增加 partition/cell/HLOD/streaming、foliage/scatter、budget/device/telemetry 与 failure recovery。
5. 用真实产品场景和 required test matrix 验收，所有 gate 通过后再开放 Terrain menu、提升 maturity 或宣称完成。

