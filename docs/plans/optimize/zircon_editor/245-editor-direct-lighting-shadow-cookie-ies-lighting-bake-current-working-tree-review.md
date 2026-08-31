---
title: Editor Direct Lighting、Shadow、Cookie、IES、Lighting Bake 与 Viewport Debug 当前工作树复核
category: zircon_editor
report_id: Editor245
review_date: 2026-08-30
baseline_head: 79ff31b5e6f3cf8319f809013b2f960493a1a96a
verification_head: 79ff31b5e6f3cf8319f809013b2f960493a1a96a
canonical_owner: Editor245
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/185-runtime-direct-lighting-shadow-current-working-tree-refresh.md
related_code:
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_toolbar_state.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_stats.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/document
  - zircon_editor/src/core/project
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/preview_scene
  - zircon_editor/src/ui/retained_host
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_plugins/rendering/features/contact_shadow
tests:
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_modules.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_snapshot.rs
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/PointLightComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/SpotLightComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/RectLightComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/StatsViewer/Classes/LightingBuildInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ShowFlags.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorViewportSettings.h
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/HDLightEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/HDLightEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/HDAdditionalLightDataEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/2D/Light2DEditor.cs
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/lightmap_gi_editor_plugin.cpp
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/settings/graphics.rs
  - dev/bevy/crates/bevy_dev_tools/src/diagnostics_overlay.rs
  - dev/bevy/crates/bevy_gizmos/src/config.rs
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor245 · Direct Lighting / Shadow / Lighting Bake 当前工程化差距

## 1. 结论

当前 Editor 的 viewport 控制有真实但很窄的底座：`SceneViewportSettings` 可切换 projection、display、grid、gizmos、preview lighting 和 preview skybox；render packet 会把这些值送入 Runtime snapshot；controller 有方向光 gizmo、selection overlay、capture/product polling；通用 document/transaction/PreviewScene/diagnostic host 也可复用。

它仍不是工程级灯光编辑器。`settings.rs` 没有 shadow quality、light channels、exposure/photometry、cookie/IES、cluster/atlas/debug show flags 或 per-view lighting profile。`render_packet.rs` 的 gizmo 只处理 Camera 和 DirectionalLight，Point/Spot/Rect/Ambient 明确落入空分支；没有 cone/range/area rectangle/IES/cookie visualization、shadow frusta 或 caster selection。preview skybox 直接使用 `FallbackSkyboxKind::ProceduralGradient`，并没有 physical sky/atmosphere provenance。

`workbench_extension_lighting_bake_workspace.zui` 的根节点是 `visibility = "collapsed"`。其中 `City_Block_A`、`Interior_Lab`、`Reflection Probe Grid`、`Shadow Maps 4096 px Stationary Ready`、`87 assets 4 warnings`、`Preview ready estimate 02:30` 等均为静态属性，按钮仅发 route；当前源码没有 lighting asset factory、bake document、operation handler、compiler job、artifact receipt、PreviewWorld 或 Runtime shadow/cache mirror。Editor 可以展示一套“Lighting Bake”外观，却不能证明任何灯光/阴影/光照贴图操作已执行。

因此 Runtime185 中的 scene -> renderer 闭环在 Editor 侧仍断裂。本报告不新增唯一 P0，继承 Editor189/Editor96/Editor22 与 Runtime185 的 parent owners；登记 **0 项新 P0、28 项 P1、10 项 P2、25 道资格门**，P1 28 Open，P2 10 Open，资格门 24 Fail、1 Partial、0 Pass。

目标编辑器架构：

```text
Scene light component / Lighting asset / Cookie / IES / Bake settings
  -> typed editor document + stable IDs + transaction/undo
  -> cancellable validate/compile/bake operation
  -> immutable artifact + generation + diagnostics
  -> PreviewWorld/PIE runtime install
  -> viewport gizmo/show-flag/cluster-atlas visualization
  -> Runtime receipt, capture provenance and persisted profile
```

## 2. 当前源码证据

### 2.1 Viewport settings 与 render packet

- `SceneViewportSettings` 只有 `projection_mode`、`display_mode`、`grid_mode`、`preview_lighting`、`preview_skybox`、`gizmos_enabled` 等通用开关；没有 per-light shadow/cookie/IES/photometry/channel 或 quality profile。
- `render_settings()` 只投影四个 viewport/environment 字段；不存在 runtime `LightingDebugSettings`、show-flag mask、cluster/atlas selection、shadow cache visualization 或 fallback reason。
- `build_render_packet()` 调用 `scene.build_viewport_render_packet`，随后以 `ProceduralGradient` 作为 skybox fallback。它没有引用 physical sky/atmosphere asset、lighting bake artifact、preview light rig 或 runtime generation。
- `build_scene_gizmos()` 只遍历 `NodeKind::Camera | NodeKind::DirectionalLight`；Ambient/Point/Rect/Spot 在 match 中返回 `None`。因此编辑器无法检查最常用的 local light bounds、spot cone、rect orientation 或 shadow receiver/caster。
- gizmo picking 只有 sphere/arrow/frustum 等通用 shape，没有 range/near/far/cascade/atlas page/cookie projection/IES lobe 的 pickable geometry。

### 2.2 Inspector、document 与 transaction boundary

- `src/core/asset/type_registry`、`src/core/editing`、`src/core/document` 和 `src/ui/asset_editor` 没有 LightProfile/IES/Cookie/LightingBake asset factory、typed document、schema migration 或 artifact install owner。
- 通用 reflection/property access 能读写已有 scene component 的基础字段，但没有 shadow settings、photometric units、channels、bake settings 的 domain validation，也没有把 source/effective/degraded provenance 投影到 inspector。
- Scene transaction/undo 可作为宿主，但 light property mutations 没有独立 operation identity、coalescing policy、dependency invalidation、compile/bake job 或 savepoint。
- `PreviewScene` 是通用 secondary session/subject/playback abstraction；没有 lighting rig、shadow atlas/cache generation、fixed-step bake preview、device loss 或 runtime receipt consumer。

### 2.3 Lighting Bake Workbench 与 routes

- `workbench_extension_lighting_bake_workspace.zui` 的 layout、tabs、rows、quality/target/denoise controls 具备静态 UI 结构，但 root collapsed，所有 values 和 status 在 `props` 中硬编码。
- Preview/Bake/quality/target/denoise routes 没有在当前 editor source 中找到对应 lighting operation factory、prepare/apply、progress/cancel/harvest、artifact generation 或 failure projection；route 到 UI 的存在不能证明执行。
- queue rows 把 shadow maps、lightmap UV、probe grid、bleed warning 和 output 混在一张表，缺少 task identity、dependency DAG、per-surface status、stale generation 和 partial artifact policy。
- `first_party_editor_catalog` 没有 lighting bake provider/extension owner 的可验证产品入口；plugin descriptor/capability 或测试注册不得被视为可用 editor feature。

### 2.4 Preview、debug、capture 与 persistence

- viewport capture/product polling 只交换最终 frame/product；没有 light list、cluster occupancy、atlas allocation、shadow cache hit、cookie/IES residency 或 photometry diagnostics。
- toolbar state 只有 preview lighting/skybox 字段，没有 shadow-only、lighting channels、unlit/normal/depth/shadow atlas/cluster heatmap/cascade split 等显示模式。
- `SceneViewportStats` 只有 node/camera/mesh/light 总数；不能区分 directional/point/spot/rect、accepted/rejected/degraded、shadowed、cluster overflow 或 bake state。
- editor preferences/viewport settings 没有 lighting profile scope、project/user/session precedence、multi-viewport isolation 或 generation fence；改变 preview setting 不能关联 artifact/runtime invalidation。
- 现有 UI tests 主要验证 toolbar projection、template/route 和 snapshot 结构，未验证真实 light authoring -> runtime capture、shadow pixel effect、bake artifact 或 device/fault recovery。

## 3. 参考引擎差异

Unreal 的 LightActor/LightComponentVisualizer、LevelEditor viewport settings、ShowFlags 与 Lighting Build 信息把组件属性、视图显示、lightmap/probe build、错误和结果资产连成 editor/runtime 路径。Unity HDRP LightEditor/HDLightEditor、Probe Volume 与 URP 2D light editor 提供 per-light unit/shape/cookie/shadow/channel controls、scene handles、preview and build diagnostics。Godot Node3D viewport/lightmap GI editor 和 Fyrox graphics settings/scene editor 是较轻量的可比基线，Bevy gizmo/diagnostic overlay 则展示如何把运行时状态作为结构化 debug stream。Zircon 当前仅实现通用 viewport chrome 与静态 Lighting Bake surface。

## 4. P1 重构任务

| ID | 当前差异 | 必须完成 |
|---|---|---|
| ED-LGT-01 | 无 light asset taxonomy | 注册 LightProfile/Photometry/Cookie/IES/LightingBake/Probe/Lightmap asset types、factory、thumbnail 与 provider。 |
| ED-LGT-02 | 无 typed light document | stable light/shadow/cookie/IES/channel IDs、schema version、revision、unknown field migration。 |
| ED-LGT-03 | inspector 只有基础字段 | 为 directional/point/spot/rect/ambient 提供单位、shape、mobility、shadow、channel、cookie/IES 字段与 validation。 |
| ED-LGT-04 | 无 photometry UI | lux/lumen/candela/nit、temperature/tint、exposure/working space、finite/negative errors 和 source/effective display。 |
| ED-LGT-05 | 无 shadow authoring | casts shadow、resolution tier、PCF/filter、bias、strength、cascade policy、contact method 与 per-device fallback。 |
| ED-LGT-06 | 无 cookie/IES authoring | asset picker、projection/UV/wrap、IES normalization/profile preview、dependency and residency status。 |
| ED-LGT-07 | 无 channel authoring | receiver/light/caster/volumetric channel mask、multi-select editing、inheritance/override 和 runtime parity。 |
| ED-LGT-08 | gizmo 只支持方向光 | point range、spot cone、rect area、ambient volume、shadow/cascade frusta 和 cookie/IES handles。 |
| ED-LGT-09 | gizmo pick shape 粗糙 | shape-aware hit test、screen/world scale、near clipping、multi-selection、snap and accessibility。 |
| ED-LGT-10 | settings 缺 lighting profile | project/user/session scoped quality、shadow/cluster/atlas/cookie/debug settings、precedence and persistence。 |
| ED-LGT-11 | 缺 show flags | shaded/unlit/wire/normal/depth/shadow-atlas/cluster/cascade/light-channel/overdraw modes with typed state。 |
| ED-LGT-12 | stats 只有总灯数 | per-family accepted/rejected/degraded/shadowed, cluster occupancy, atlas pressure, cookie/IES residency and GPU timings。 |
| ED-LGT-13 | preview skybox 是 gradient | physical sky/atmosphere/environment asset, fallback provenance, exposure and shadow lighting rig。 |
| ED-LGT-14 | PreviewScene 无 lighting rig | isolated preview world installs effective descriptor, fixed-step, camera/light controls and device generation。 |
| ED-LGT-15 | no bake document | scene/level/target set, static/stationary/dynamic policy, UV/probe/cascade settings, stable task IDs and dirty state。 |
| ED-LGT-16 | Workbench root collapsed | enable a real lighting workspace with active route/tab state, selection, filter and responsive content. |
| ED-LGT-17 | Workbench values static | all rows/status/estimate/warnings derive from live document, compiler, runtime and artifact receipts。 |
| ED-LGT-18 | routes lack operation owner | Preview/Bake/Validate/Cancel/Retry/Install use operation factory, admission, cancellable job and typed result。 |
| ED-LGT-19 | no dependency DAG | mesh UV/material/light/cookie/IES/probe dependencies, generation snapshots, stale-result rejection and incremental invalidation。 |
| ED-LGT-20 | no compiler/artifact | deterministic bake/lighting compiler emits immutable artifact, hash, source map, diagnostics and last-good install。 |
| ED-LGT-21 | no runtime bridge | editor artifact -> PreviewWorld/PIE -> Runtime185 EffectiveLightDescriptor/ShadowFramePlan receipt。 |
| ED-LGT-22 | no atlas/cluster visualization | expose allocation rectangles, safe UV/gutter, rejected slots, cluster heatmap, z-bin/tile and overflow reason。 |
| ED-LGT-23 | no shadow caster tools | select/highlight caster/receiver, inspect cascade/face/view, static/dynamic/cache hit/miss and material alpha mode。 |
| ED-LGT-24 | no capture provenance | frame capture records lighting profile, artifact/generation, device, exposure, atlas/cache/cluster counters and source hash。 |
| ED-LGT-25 | no save/reopen roundtrip | scene/light/bake settings and artifacts preserve IDs, references, unknown fields, dirty/last-good and migration status。 |
| ED-LGT-26 | no fault UI | missing cookie/IES, invalid unit, atlas overflow, compiler failure, device loss, stale job and rollback are actionable。 |
| ED-LGT-27 | tests are UI/fixture only | add property/transaction/roundtrip/operation/PreviewWorld/GPU pixel tests for all light families and debug modes。 |
| ED-LGT-28 | no scale/collaboration gate | 1/64/4K lights, multi-viewport, large scene, concurrent edit, source-control conflict and same-hardware perf/visual corpus。 |

## 5. P2 增强任务

| ID | 演进方向 | 前置资格 |
|---|---|---|
| ED-LGT-P2-01 | VSM page/cascade clipmap inspector | Runtime persistent page/cache receipts and atlas ownership。 |
| ED-LGT-P2-02 | area/emitter shape authoring | LTC/solid-angle/photometry baseline and oriented gizmo。 |
| ED-LGT-P2-03 | probe volume/irradiance authoring | baked artifact, residency and runtime sampling generation。 |
| ED-LGT-P2-04 | path-traced lighting preview | deterministic oracle, exposure/BRDF provenance and cancellable job。 |
| ED-LGT-P2-05 | GPU light linking and importance tools | channel/assignment owner and stable accepted/rejected receipt。 |
| ED-LGT-P2-06 | XR multiview/foveated debug | view family, foveation and per-eye capture contract。 |
| ED-LGT-P2-07 | collaborative lighting review | document lease, conflict/rebase and artifact provenance。 |
| ED-LGT-P2-08 | automated visual diff dashboard | same hardware, fixed exposure/scene and tolerance metadata。 |
| ED-LGT-P2-09 | scripted batch bake/CI | headless operation service, deterministic outputs and artifact cache。 |
| ED-LGT-P2-10 | advanced teaching/diagnostic overlays | runtime debug stream, budget throttling and non-invasive rendering path。 |

## 6. 资格门

| 门 | 当前结果 | 关闭证据 |
|---|---|---|
| light asset/provider | Fail | types/factory/catalog/provider/unsupported state and lifecycle。 |
| typed light document | Fail | stable IDs, revision, migration, dirty and unknown-field roundtrip。 |
| inspector parity | Fail | all families expose validated photometry/shape/shadow/cookie/IES/channel fields。 |
| photometry UI | Fail | unit conversion, color/exposure validation and source/effective provenance。 |
| shadow authoring | Fail | settings reach Runtime185 extract/plan and show acceptance/degrade reason。 |
| cookie/IES authoring | Fail | picker, projection/normalization, dependency generation and preview。 |
| channel authoring | Fail | receiver/caster/volumetric mask edits persist and affect capture。 |
| shape-aware gizmos | Partial | camera/directional gizmo exists; point/spot/rect/ambient and shadow handles absent。 |
| show flags | Fail | typed debug modes drive render packet and capture。 |
| lighting profile | Fail | scope/precedence/persistence and generation invalidation。 |
| physical preview | Fail | physical sky/light rig/units with explicit fallback provenance。 |
| PreviewWorld | Fail | effective artifact/provider install, fixed-step, device/world generation and cleanup。 |
| bake document | Fail | target/dependency/task identity and transaction/dirty lifecycle。 |
| workbench visibility | Fail | lighting workspace is enabled and stateful, not collapsed fixture. |
| operation lifecycle | Fail | validate/preview/bake/cancel/retry/install produce typed receipts. |
| dependency DAG | Fail | mesh/UV/material/light/probe/cookie/IES generation and stale rejection。 |
| compiler artifact | Fail | deterministic artifact hash/source map/last-good install。 |
| runtime bridge | Fail | editor artifact -> Runtime185 descriptor/plan/capture receipt。 |
| cluster/atlas debug | Fail | occupancy/overflow/allocation/cache and per-light selection。 |
| caster/receiver debug | Fail | view/cascade/face/static-dynamic/cache inspection and pixel effect。 |
| capture provenance | Fail | profile/artifact/device/exposure/counters/source hash attached to frame。 |
| roundtrip | Fail | save/reopen/migrate scene/light/bake settings and artifacts without ID loss。 |
| fault handling | Fail | missing dependency/overflow/compiler/device/stale job actionable and recoverable。 |
| test coverage | Fail | property/operation/PreviewWorld/GPU pixel/fault/scale/soak matrix。 |
| performance parity | Fail | same hardware/scene Unreal/Unity/Fyrox/Godot CPU/GPU/VRAM/visual comparison。 |

## 7. 实施顺序

1. 先让 scene light component 的 typed inspector 和 shadow/cookie/IES source 能通过统一 transaction/save roundtrip。
2. 建立 lighting document、profile、operation service 和 immutable artifact，启用真实 Workbench 状态投影。
3. 将 artifact 安装到 PreviewWorld/PIE，打通 Runtime185 的 descriptor、cluster、shadow plan 和 receipt。
4. 扩展形状 gizmo、show flags、atlas/cluster/caster diagnostics 与 capture provenance。
5. 以多灯、烘焙、故障、协作和同硬件视觉/性能基准关闭资格门。

本轮仅写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Editor/Cargo/GPU/PIE 动态验证。
