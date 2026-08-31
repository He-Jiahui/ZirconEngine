---
title: Editor Scene Viewport Display Mode、Lighting、Skybox、Show Flag、Debug Visualization、Overlay Composition、Profile、Persistence 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor189
review_date: 2026-08-27
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
  - zircon_editor/src/ui/layouts/views/viewport_chrome.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/wireframe
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/scene/world/render.rs
tests:
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/state/play_mode.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/dispatch.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/187-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/188-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/99n-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ShowFlags.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ShowFlagsValues.inl
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShowFlags.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/BufferVisualizationData.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorViewportSettings.h
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/Fyrox/editor/src/settings/debugging.rs
  - dev/Fyrox/editor/src/settings/graphics.rs
  - dev/Fyrox/editor/src/settings/mod.rs
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene_viewer/mod.rs
  - dev/bevy/crates/bevy_pbr/src/wireframe.rs
  - dev/bevy/crates/bevy_gizmos/src/config.rs
  - dev/bevy/crates/bevy_dev_tools/src/diagnostics_overlay.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/IDebugDisplaySettingsQuery.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySerializer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Debug/DebugDisplaySettingsRendering.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Debug/DebugDisplaySettingsMaterial.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Debug/DebugDisplaySettingsLighting.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/SceneViewDrawMode.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/SceneViewDrawMode.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Viewport Display Mode、Lighting、Skybox、Show Flag、Debug Visualization、Overlay Composition、Profile、Persistence 与 Product Integration 当前源码复核

## 1. 结论

Editor68之后出现了一项值得保留的真实进展：Scene Viewport现在有独立`ViewportOverlayProviderRegistry`。它按stable provider id检查重复注册，保存`owner_id`和required capabilities，支持prepare/install、typed toggle error、plugin panic boundary、fault quarantine，并把已启用provider的`SceneGizmoOverlayExtract`追加到真实render packet。binding codec、command、event route和extension registration也已接通。这不是占位代码，因此ED68-P1-16/18/20/28/29及相关资格门获得部分基础。

但该registry仍只是“可开关的gizmo producer集合”，不是工程级Visualization authority。provider不能声明category、world/screen layer、depth/xray、priority、blend、settings schema、cost/budget、resource lease、generation、effective receipt或unload fence；全部输出仍被压入固定scene-gizmo通道。Runtime实际record顺序仍由源码固定，`PASS_ORDER`仅在`cfg(test)`下作为测试常量，不能形成产品可查询的composition plan。

其余核心结论没有改变。`SceneViewportSettings`仍只有三值`DisplayMode`、`preview_lighting`、`preview_skybox`和`gizmos_enabled`；没有stable visualization profile、Show Flag、material/buffer visualization、per-view identity、requested/effective split或持久化。Wireframe仍在CPU逐帧遍历mesh、查询static model `wire_segments`、构造selection `HashSet`和world-space line `Vec`；WireOnly继续跳过base opaque、transparent/sprite replay及OIT，缺线源的对象会静默消失。

Lighting Off仍不是严格Unlit：direct light、light grid和shadow会被裁掉，但scene ambient注入固定`0.55`，environment和post-process不由同一resolver决定。Skybox Off仍经`EnvironmentExtract::disabled()`同时清空背景、reflection probes、baked lighting和probe grid；用户不能隐藏背景同时保留IBL，也不能选择source environment、rotation、intensity、exposure或quality override。Editor继续对`virtual_geometry_debug`硬编码`None`，Runtime已有的VG调试能力只能被观察，不能从当前视口请求与恢复。

本轮不新增P0。Editor68的29项P1当前为 **21 Open / 8 Partial**，8项P2为 **7 Open / 1 Partial**；48门为 **38 Fail / 10 Partial / 0 Pass**。目标仍是stable `ViewportVisualizationProfile`、Runtime唯一resolver、generation-qualified effective receipt、GPU-native wire parity、可扩展Show Flag/debug/overlay provider以及per-view session/persistence/diagnostics产品闭环。

本轮只做review，没有修改production Rust，也没有运行Cargo、Editor、GPU render golden、save/reopen、multi-view、provider reload、device/capability matrix、fault/scale/soak/profile或同硬件跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。当前不能声称该域的功能、表现或性能达到或超过Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree边界

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。本报告以2026-08-27读取时当前磁盘为事实源，包括未跟踪的overlay provider实现；不以旧Editor68 fingerprint或HEAD内容覆盖共享工作树，也不回退、格式化或吸收其他会话修改。

MVP baseline recovery仍为`in_progress`。本报告是后续RED、架构拆分与hard cutover输入，不是实现、动态验证或性能receipt。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Editor visualization/product | **20 / 3,195 / 2,963 / 124,102 / 12** | settings、packet、provider、typed route、toolbar、Play状态与ZUI | `c09da78111ef8ddadd956599c3d646b62d9b8b532ec68ef10189474923a78105` |
| Runtime visualization/execution | **19 / 4,261 / 3,965 / 160,542 / 39** | camera/environment、wire/base、overlay order、lighting/shadow与scene extract | `cff9b36ccee135cabc48c9199541c230c8b536b9da5263352d085b27dc513496` |
| Diagnostics | **9 / 2,104 / 1,999 / 69,547 / 14** | frame profile、GPU timing、capability、VG/product stats与Editor projection | `e97fef35d8584d368359fd719b5bb2b9fb6d99709a81a24a843c3d2abbe0d8f2` |
| Focused tests | **10 / 3,248 / 2,970 / 112,980 / 65** | packet/state/play、binding/toolbar/template/chrome与pane projection | `2c3897a52c9c693e2177b4344cb05ea3e852bc684cd7eae752f09a35a5e3ed55` |
| Unreal selected set | **5 / 3,448 / 3,019 / 153,462 / 0** | Show Flag/View Mode resolver、buffer registry与viewport persistence | `d13d6e4a24748391f1249c177cb45f16a195cebc85e7571e2a97fa10d8cde879` |
| Godot selected set | **2 / 7,927 / 6,790 / 311,055 / 0** | per-view display/debug/environment/gizmo/diagnostic状态 | `19f90d99bbf52b085292af28ba82b4c2f0809e12457b5ef53d0f2b0b566f3bd0` |
| Fyrox selected set | **5 / 2,691 / 2,463 / 100,015 / 0** | persistent debug category、graphics quality与scene consumer | `ca6288f987ccfe448f37c4294d566422f05463193d1f2aeb83127e1a79489272` |
| Bevy selected set | **3 / 2,469 / 2,247 / 90,658 / 0** | GPU wire phase、capability gate、gizmo config与bounded diagnostics | `d92a46f1be0c3afadcdbe87bfb7a434f7786c57f052bcde941445f88f32313cc` |
| Unity Graphics selected set | **8 / 1,858 / 1,644 / 83,647 / 0** | modular debug fragments、effective query、serializer与mode validation | `8a51bf220eb578d43c78a72ac378a6efced8ac041a6f70c24d729a26bf6eb2d4` |

fingerprint方法为规范化相对路径、逐文件SHA-256、排序后的`path::hash`以当前环境换行连接，再对整体做SHA-256。它只证明选择集内容，不代表ABI、artifact、动态行为或性能。Godot、Fyrox、Bevy、Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor189只刷新Editor68拥有的display/view mode、lighting/environment preview、Show Flag、debug/buffer visualization、overlay composition、per-view profile/persistence/effective receipt与产品投影。Editor179提供ViewInstance/render product identity，Editor180提供selection/highlight/picking，Editor187提供camera session，Editor188提供transform/grid，Editor22提供通用diagnostics/capture；Runtime父报告拥有RHI、material、environment、post-process和quality执行。本报告只定义这些owner必须接受的visualization policy与receipt，不重复父finding。

## 3. 当前实现拓扑

### 3.1 Typed command与真实render packet应保留

`SetDisplayMode`、`SetPreviewLighting`、`SetPreviewSkybox`、`SetGizmosEnabled`和`ToggleOverlayProvider`均有typed codec/route并进入controller；packet会真实改变Runtime settings、environment与overlay extract。问题不是按钮无效，而是request只携enum/bool/provider string，没有ViewInstance、profile generation、capability snapshot或effective response。

### 3.2 Overlay Provider Registry是真进展，但仍是单通道producer

registry具备duplicate/unknown/disabled/quarantined typed error，安装前可prepare，callback经过plugin fault boundary，失败provider会被隔离。它还保存owner与required capabilities，证明extension overlay无需继续硬编码到core枚举。

当前却没有descriptor snapshot、provider generation、priority、layer/depth/xray、cost/settings、readiness、resource lease和unload。输出被`flat_map`成`Vec<SceneGizmoOverlayExtract>`并追加到同一gizmo list，无法与selection、wire、grid、handle形成统一composition plan，也无法向UI报告effective状态。

### 3.3 Wireframe覆盖、正确性和性能均未达标

`build_wireframe_vertices`仍创建WireOnly selection `HashSet`与输出`Vec`，遍历`frame.meshes()`后从streamer回取model并读取静态`wire_segments`，再在CPU变换为world-space line。direct/procedural、skinned/morph、VG、terrain、sprite和particle没有同代wire source。WireOnly在base pass提前跳过scene product，缺线源对象会消失；新增测试主要守卫early skip顺序，不证明primitive parity、实际像素或large-scene预算。

### 3.4 Lighting与Environment仍由互相独立的布尔值决定

lighting false会真实清空direct light、grid和shadow plan，但scene uniform使用固定ambient补偿；environment仍独立参与，post-process也没有统一推导。skybox false调用disabled extract，同时移除背景、IBL/probe/baked lighting。当前不能表达Lit、Unlit、Lighting Only、Detail、background-only、IBL-only或authored environment preview。

### 3.5 Overlay录制顺序仍是编译期固定事实

selection、wireframe、grid、scene gizmo和handle有独立pass，这是可保留基础；renderer仍按固定函数调用录制。新增`PASS_ORDER`只在test配置存在，既不是运行时plan，也不含category/layer/depth/xray/priority/cost，不能用于plugin冲突解析或产品回执。

### 3.6 Runtime诊断丰富，Scene Viewport消费仍不qualified

`RenderFrameProfile`、GPU timing Disabled/Unavailable/Pending/Measured、capability summary、RenderStats、VG available/visible cluster与多个product counters是真实底座。Editor pane仍消费framework级last snapshot，Scene HUD继续只显示scene/projection/display/grid；没有viewport/product/profile generation、age、sampling budget或degraded reason。Editor packet仍把`virtual_geometry_debug`设为`None`。

### 3.7 Play、持久化与测试未形成产品闭环

进入Play只捕获并关闭gizmo，退出只恢复gizmo；display、lighting、skybox、debug和provider没有profile transition token。`SceneViewportSettings`虽可序列化，Settings authority仍没有这些key。现有测试能证明route、bool传播、provider duplicate/capability error和部分fault isolation，但没有render golden、save/reopen、per-view isolation、provider unload、capability matrix或性能资格。

## 4. 五引擎参考结论

### 4.1 Unreal

`FEngineShowFlags`、View Mode override和Buffer Visualization registry共同形成stable mode到effective flags/targets的解析链；viewport instance settings持久化show flags、buffer/subsystem visualization、exposure、FOV、realtime与stats。可借鉴的是`profile -> resolver -> effective state -> per-view persistence`，不是复制宏、CVar或类层级。

### 4.2 Godot

3D viewport把display、environment、gizmo、information、frame time、quality和camera state作为per-view产品状态，并对高级debug mode附带rendering method约束。它证明capability-gated menu与per-view save/restore属于基本合同，而不是后期调试附属物。

### 4.3 Fyrox

Fyrox规模较小，但physics、bounds、TBN、terrain、light/camera bounds等debug category有typed settings和持久化consumer。它不能作为最终wire性能金标准，却足以证明单一`gizmos_enabled`不能替代category profile。

### 4.4 Bevy

Bevy wireframe进入GPU render phase，按GPU feature做准入，支持global/per-entity配置、color、width、topology与depth bias；gizmo按config group扩展，diagnostic overlay限制文本刷新。可借鉴的是GPU-native geometry、provider group、capability和bounded observation。

### 4.5 Unity Graphics

Core/URP把Rendering、Material、Lighting debug fragment聚合为单一effective policy，显式推导lighting、post-process与clear color，并序列化debug state、拒绝不支持的Scene View mode。本地镜像不代表完整Unity Editor源码，本报告只采用其modular settings、single resolver、serialization与rejection合同。

## 5. Canonical finding状态账本

### 5.1 P1：架构、正确性与产品闭环

#### ED68-P1-01 [Open]：没有单一Visualization Profile与resolver

settings仍平铺enum/bool，packet逐字段复制。必须定义stable profile fragment，并由Runtime唯一resolver共同推导display、lighting、environment、post-process、overlay、diagnostic与quality effective policy。

#### ED68-P1-02 [Open]：DisplayMode仍是闭集三循环

Shaded/WireOverlay/WireOnly在codec、toolbar和HUD重复穷举，没有stable descriptor、category、requirements、provider owner或deprecated mapping。新增模式仍要求修改核心枚举和所有match。

#### ED68-P1-03 [Open]：没有qualified request与effective receipt

command和render request不带view/profile/capability generation，UI只能假定写入即生效。需要包含requested/effective/rejected、fallback、provider/capability/frame generation的typed receipt。

#### ED68-P1-04 [Open]：Wire source只覆盖streamed static model

当前从asset `wire_segments`猜最终几何，direct/procedural、deformed、VG、terrain、sprite与particle无同代来源。应在canonical geometry submission层产生wire draw或明确拒绝。

#### ED68-P1-05 [Open]：WireOnly会静默丢失可见primitive

base与OIT提前返回，而wire builder覆盖不全；用户看不到unsupported count、degraded reason或fallback。必须保证owner集合守恒，或对每类primitive给出可观察的rejection。

#### ED68-P1-06 [Open]：Wireframe逐帧CPU全量重建

每帧分配`HashSet`/`Vec`、查询streamer、遍历segment并计算world vertex，没有topology cache、dirty generation、visibility reuse、instance transform或GPU phase。large-scene成本随边数线性增长且无预算。

#### ED68-P1-07 [Open]：Wire style/depth/topology硬编码

缺line width、triangle/quad/crease/boundary、hidden line、depth/xray、occluded tint、render layer和per-object override。必须把style与topology policy纳入profile/provider descriptor。

#### ED68-P1-08 [Open]：Material/attribute/GBuffer visualization缺失

BaseColor、Normal、Roughness、Metallic、Depth、Motion Vector、ID、UV、tangent、lightmap density及pre/post-tonemap HDR均无typed target registry。Editor不得通过专用shader旁路Runtime。

#### ED68-P1-09 [Open]：Lighting mode未形成模式族

只有direct lighting bool，没有Lit、Unlit、Lighting Only、Detail、Reflection、Shadow Cascade、Light Complexity或Cluster Occupancy。互斥语义应由resolver而非boolean fan-out表达。

#### ED68-P1-10 [Open]：Lighting Off不是严格Unlit

direct/shadow虽关闭，固定ambient、IBL和post-process仍可参与，结果依赖另一个Skybox bool。必须冻结并golden验证每个mode对material、ambient、IBL、fog、transparency和post-process的政策。

#### ED68-P1-11 [Open]：Skybox错误耦合背景与环境光照

`EnvironmentExtract::disabled()`同时清空sky、reflection probes、baked lighting和probe grid。background visibility、environment lighting source、probe/baked participation必须拆成独立fragment。

#### ED68-P1-12 [Open]：没有Environment Preview Scene

不能选择cubemap/HDRI、rotation、intensity、sun、clear color或neutral studio profile，也没有resource generation、async readiness和fallback receipt。

#### ED68-P1-13 [Partial]：已有camera/quality DTO，未进入产品

Runtime camera已有HDR、EV100、MSAA、dynamic resolution与jitter，quality/capability也有合同；Editor profile仍不表达requested/effective override。基础可复用，但产品与resolver未接。

#### ED68-P1-14 [Open]：没有Show Flag registry

mesh、sprite、particle、light、camera、audio、collision、navigation、AI、volume、decal、fog、bounds和LOD等类别无stable flag descriptor、scope、dependency/conflict或effective mask。

#### ED68-P1-15 [Open]：Overlay extract不是category profile

固定字段无法表达category id、priority、space、depth、xray、layer、blend、cost或disabled reason；pointer/hit也没有消费同代effective category generation。

#### ED68-P1-16 [Partial]：第一方component visualization覆盖不足

内建scene gizmo仍主要覆盖Camera与DirectionalLight，Point/Spot/Rect range、probe、volume、audio和camera clip等常用可视化缺失。新增provider registry允许扩展贡献gizmo，因而从Open升为Partial，但没有第一方category completeness或统一plan。

#### ED68-P1-17 [Open]：Overlay顺序与深度政策固定在源码

真实录制仍按selection、wireframe、grid、scene gizmo、handle的函数顺序执行。test-only `PASS_ORDER`不等于运行时composition authority，也不能解决mode重排、screen/world隔离或plugin conflict。

#### ED68-P1-18 [Partial]：扩展provider缺组合与生命周期合同

provider已有id、owner、capability、toggle、panic quarantine与prepare/install；但没有category/layer/depth/priority/settings/cost、generation、resource lease、unload fence或effective receipt。所有输出仍挤入scene-gizmo list。

#### ED68-P1-19 [Open]：Virtual Geometry debug仍不可从Editor请求

Scene packet继续写`virtual_geometry_debug: None`，forced mip、freeze cull、BVH/visbuffer和cluster readback无产品入口、readiness admission或退出恢复。

#### ED68-P1-20 [Partial]：跨子系统debug visualization没有统一目录

overlay registry证明plugin producer可被集中安装，因而有局部基础；但collision、nav、AI、physics、audio、visibility/HZB、LOD、streaming与cluster仍无统一descriptor、capability/cost和debug target registry。

#### ED68-P1-21 [Open]：Serializable settings没有持久化闭环

display、lighting、skybox、gizmo与provider没有User/Project/Layout/Session key、schema、migration、load/save或invalid source feedback。derive并不等于产品持久化。

#### ED68-P1-22 [Open]：没有per-view visualization identity

状态仍附着单controller，duplicate/floating/Scene/Game依赖Editor179尚未闭合的ViewInstance identity。销毁view也没有profile/provider subscription lease可释放。

#### ED68-P1-23 [Open]：Edit/Play/Scene/Game没有profile transition政策

Play只保存与恢复gizmo；其余mode沿用共享状态。需要context-aware override stack、generation-qualified restore和runtime-authoritative restriction，而非逐bool补丁。

#### ED68-P1-24 [Open]：Toolbar只有cycle/toggle而非可检查产品

没有descriptor-driven menu、搜索/分组、requested/effective差异、busy/degraded/unavailable状态或provider loading reason。复杂模式不能继续堆icon-only按钮。

#### ED68-P1-25 [Open]：HUD遗漏并可能误报有效事实

固定HUD仍只投影scene/projection/display/grid并直接读取requested settings；lighting、environment、debug target、quality、fallback、timing状态和frame generation不可见。

#### ED68-P1-26 [Partial]：RenderStats丰富但不viewport-qualified

Runtime已有真实CPU/GPU profile、pass counters、memory、capability与product stats，Editor pane也能投影部分字段；但数据仍是framework last snapshot，没有viewport/product/profile generation与age，不能证明属于当前Scene view。

#### ED68-P1-27 [Partial]：diagnostics有基础，没有per-view cost policy

pane可见时才刷新和Runtime GPU timing状态是正确底座；Scene stats没有visibility subscription、采样频率、history/readback budget、drop/stale状态或UI refresh limit。

#### ED68-P1-28 [Partial]：mode admission仅在overlay provider局部存在

overlay toggle能按required capabilities拒绝并返回typed missing list，Runtime也有capability summary；display/lighting/buffer/debug mode仍不使用统一admission，且没有Supported/Degraded/Pending/Unavailable effective state。

#### ED68-P1-29 [Partial]：测试新增provider/fault基础，资格矩阵仍缺

已有typed route、provider duplicate/capability/quarantine和base-pass guard等测试，因此不再是完全空白；但wire parity/pixel、lighting/environment组合、per-view isolation、persistence、unload、device matrix和scale budget均未覆盖。

### 5.2 P2：质量、可维护性与资格证据

#### ED68-P2-01 [Open]：视觉常量未收敛

颜色、clear、icon、pick radius与HUD尺寸缺theme/profile/DPI/HDR政策，应由typed visual style与provider descriptor拥有。

#### ED68-P2-02 [Open]：固定HUD不适配长文本与localization

固定高度、无换行和手写英文label无法承载长mode名与degraded reason，需要结构化、可折叠且限频的overlay layout。

#### ED68-P2-03 [Open]：label与codec重复穷举

mode名称散落在codec、cycle、HUD、chrome和测试。stable descriptor应成为serialization id、label、icon与localization key的单一来源。

#### ED68-P2-04 [Open]：Toolbar图标语义与可访问状态不足

Display/Lighting图标不能表达当前effective mode、不支持原因或provider provenance；tooltip与accessible label应从descriptor/receipt生成。

#### ED68-P2-05 [Open]：Wire测试仍偏源码词法

`include_str!`和early-return顺序断言不能证明几何、像素与对象守恒，应替换为builder output、graph、allocation和render golden。

#### ED68-P2-06 [Partial]：provider已有owner provenance，但无catalog

registry保存`owner_id`、provider id与required capabilities，形成最小来源信息；产品仍不能列出category、description、cost、settings、shortcut、availability或failure history。

#### ED68-P2-07 [Open]：没有profile migration telemetry

模式重命名、provider缺失、scope合并和invalid fallback没有versioned migration report，也不能保留unknown provider配置。

#### ED68-P2-08 [Open]：没有同语义跨引擎基线

必须冻结scene/camera/profile/hardware、warmup、采样帧、正确性与CPU/GPU/memory/stutter分位数；不同功能集合不能直接生成“优于Unreal”结论。

### 5.3 状态汇总

| 等级 | Open | Partial | Closed / Pass |
|---|---:|---:|---:|
| P1 | 21 | 8 | 0 |
| P2 | 7 | 1 | 0 |
| Qualification gates | 38 Fail | 10 Partial | 0 Pass |

## 6. 目标架构与职责边界

### 6.1 Runtime是唯一effective policy authority

```text
ViewportVisualizationRequest
  - ViewInstance / ViewFamily / document / source generations
  - profile id + schema/profile generation
  - display/material/lighting/environment fragments
  - show-flag and overlay requests
  - diagnostic requests + observation budget

ViewportVisualizationResolver
  - registry snapshot + provider generations
  - backend capabilities + pipeline feature readiness
  - conflict/dependency/exclusivity/admission resolution

EffectiveViewportVisualizationReceipt
  - requested/effective/rejected modes and flags
  - compiled OverlayCompositionPlan
  - environment/camera/quality effective state
  - degraded/pending/unavailable reasons
  - capability/provider/frame/product generations
```

Runtime core只拥有中立request/descriptor/receipt/capability合同，graphics provider拥有实际pipeline、resource和pass。Wireframe从canonical topology/instance/deformation generation消费；material/buffer/debug target进入render graph和feature resolver。Editor不得替换shader、直接读内部buffer或复制render truth。

### 6.2 Editor拥有per-view session、persistence与产品投影

建立`SceneViewportVisualizationSessionRegistry`，key来自Editor179的ViewInstance identity，持有profile selection、temporary override stack、provider catalog snapshot、last effective receipt与observation subscription。User/Project/Layout/Session scope保存stable id和version；provider缺失时保留unknown entry并显示Unavailable，不能静默改回Shaded后覆盖配置。

Toolbar只保留高频primary mode，Show Flag、Lighting、Environment、Buffer/Debug、Overlay和Stats进入descriptor-driven menu/panel。toolbar、HUD、shortcut、remote command与Play transition都投影同一session/effective receipt；view销毁和provider卸载必须取消subscription并释放resource lease。

### 6.3 Overlay provider硬合同

每个provider至少声明stable id、owner generation、category、world/screen space、layer、depth/xray、priority/tie-break、blend、requirements、readiness、settings schema、CPU/GPU/readback cost class和resource lifecycle。Runtime resolver编译不可变`OverlayCompositionPlan`；pointer/hit与renderer消费同一plan generation，插件不能另建未排序旁路。

### 6.4 Wireframe硬合同

static/direct/deformed/VG/terrain/sprite/particle必须各自声明Parity、Explicit Fallback或Rejected；WireOnly不能无声减少visible owner集合。thin wire优先走polygon/GPU phase，wide wire走缓存topology与instance/deformation输入；禁止每帧全Scene构造world-space线段。style、hidden/xray和selection override从effective profile读取。

## 7. 分阶段重构计划

### ED68-M0：真实性止血与RED

冻结三模式/两开关当前输出，增加WireOnly owner守恒、background-vs-IBL、lighting组合、unsupported/pending、per-view isolation和save/reopen RED。UI在receipt前不得显示请求已生效；对缺wire source和VG不可达给出typed diagnostic。

### ED68-M1：Stable schema、registry与single resolver

定义stable mode/flag/category/provider id、versioned profile、requirements/conflicts/dependencies与Runtime resolver，输出requested/effective/rejected receipt。禁止继续扩张`DisplayMode`和boolean fan-out。

### ED68-M2：Per-view session、scope与persistence

依赖Editor179建立session registry，实现User/Project/Layout/Session合并、migration、unknown provider preservation、Edit/Play/Scene/Game override stack和generation-qualified restore。

### ED68-M3：Wireframe hard cutover

建立immutable topology artifact、GPU/instanced transform、thin/wide wire pipeline和visibility reuse，覆盖各primitive family的parity/fallback/rejection。等价golden与预算通过后删除旧CPU world-line builder，不保留双authority shim。

### ED68-M4：Lighting、Environment与Camera resolver

实现Lit/Unlit/Lighting Only/Detail基础族，分离background、IBL、probe/baked participation，接入environment asset/rotation/intensity、fixed exposure、tonemap/post-process、AA/HDR和quality override。

### ED68-M5：Show Flag与Overlay Composition

建立grouped show-flag registry与compiled overlay plan，把selection、grid、transform、component gizmo和plugin provider硬切到同一合同，补齐第一方light/probe/volume/audio/camera visualization。

### ED68-M6：Material、Buffer与Subsystem Debug Provider

接入material/attribute/GBuffer/full-screen target以及VG、visibility/HZB、LOD/streaming、collision、navigation、AI、physics和audio provider。每个provider具备capability/readiness、async observation、退出恢复和resource lease。

### ED68-M7：Viewport-qualified diagnostics

Editor22/Runtime提供按viewport/product/profile generation绑定的frame observation；stats按可见性订阅、限频刷新并标注Pending/Unavailable/Stale。GPU query、readback、history和文本更新全部受预算控制。

### ED68-M8：单一产品投影与旧路径删除

toolbar/menu/HUD/shortcut/remote command从session + effective receipt生成；删除三循环、bool toggle猜测、重复label match和直接读requested settings的HUD。禁止双写legacy settings与新profile。

### ED68-M9：产品资格与跨引擎基线

完成render golden、save/reopen、multi-view/Play/provider reload、capability matrix、fault/soak/profile和同语义benchmark。只有正确性、可达性、currentness与统计显著性同时成立，才能讨论达到或超过Unreal。

## 8. 资格门

| Gate | 通过条件 | 当前 |
|---|---|---|
| ED68-G01 | stable profile id/schema/version可round-trip | Fail |
| ED68-G02 | mode/flag/provider registry支持generation snapshot | Fail |
| ED68-G03 | resolver对冲突、依赖和互斥有确定结果 | Fail |
| ED68-G04 | requested/effective/rejected receipt完整 | Fail |
| ED68-G05 | receipt绑定view、profile、capability和frame generation | Fail |
| ED68-G06 | unknown/deprecated provider配置可迁移且不丢失 | Fail |
| ED68-G07 | Runtime是唯一effective policy authority | Fail |
| ED68-G08 | 旧enum/bool双authority已删除 | Fail |
| ED68-G09 | static model已有wire topology source，但尚无shaded parity证明 | Partial |
| ED68-G10 | direct/procedural mesh wire parity成立 | Fail |
| ED68-G11 | skinned/morph/deformed wire generation同代 | Fail |
| ED68-G12 | VG/terrain/sprite/particle有明确parity或rejection | Fail |
| ED68-G13 | WireOnly不静默丢失任何可见primitive | Fail |
| ED68-G14 | line width/topology/depth/xray政策可配置 | Fail |
| ED68-G15 | large-scene wire path无每帧全量CPU world-line rebuild | Fail |
| ED68-G16 | wire CPU/GPU/memory budget达到冻结阈值 | Fail |
| ED68-G17 | Lit/Unlit/Lighting Only/Detail语义被golden验证 | Fail |
| ED68-G18 | direct/shadow/ambient/environment已有真实执行片段，尚无统一policy | Partial |
| ED68-G19 | background visibility与environment lighting分离 | Fail |
| ED68-G20 | authored/source environment asset可选且generation-qualified | Fail |
| ED68-G21 | exposure/tonemap/AA/HDR override显示effective值 | Fail |
| ED68-G22 | environment pending/failure有fallback receipt | Fail |
| ED68-G23 | Play/Scene/Game profile transition可逆 | Fail |
| ED68-G24 | save/reopen保持同一有效environment/profile | Fail |
| ED68-G25 | grouped Show Flag覆盖第一方内容类别 | Fail |
| ED68-G26 | overlay plan含category/layer/depth/xray/priority | Fail |
| ED68-G27 | 第一方category已有固定pass，但尚未进入同一plan | Partial |
| ED68-G28 | plugin provider已有capability gate，缺cost/settings/composition | Partial |
| ED68-G29 | provider unload取消资源/subscription且不留旧像素 | Fail |
| ED68-G30 | Runtime VG observation存在，Editor仍不能请求/恢复 | Partial |
| ED68-G31 | material/buffer debug target有typed registry | Fail |
| ED68-G32 | collision/nav/AI/physics/audio等不会各建旁路 | Fail |
| ED68-G33 | 每个ViewInstance拥有独立profile/session | Fail |
| ED68-G34 | 复制、浮动、Scene与Game不会串改显示状态 | Fail |
| ED68-G35 | User/Project/Layout/Session scope优先级确定 | Fail |
| ED68-G36 | toolbar/menu/HUD都投影effective receipt | Fail |
| ED68-G37 | unavailable/degraded/pending原因可见且可访问 | Fail |
| ED68-G38 | command链已typed，但没有完整profile intent与receipt | Partial |
| ED68-G39 | 长模式名/localization/DPI下布局不截断 | Fail |
| ED68-G40 | profile migration和invalid source有diagnostic | Fail |
| ED68-G41 | observation绑定viewport/product/profile generation | Fail |
| ED68-G42 | Runtime区分GPU timing Disabled/Unavailable/Pending/Measured | Partial |
| ED68-G43 | generic profiling/sampling已有底座，Scene stats无预算闭环 | Partial |
| ED68-G44 | render golden覆盖mode/environment/overlay组合 | Fail |
| ED68-G45 | Runtime有capability/device tests，缺visualization fallback矩阵 | Partial |
| ED68-G46 | provider已有fault quarantine，缺reload/unload/soak资格 | Partial |
| ED68-G47 | 10K/100K primitive场景满足CPU/GPU/memory阈值 | Fail |
| ED68-G48 | 同Scene/profile/hardware跨引擎基线可复现且有统计意义 | Fail |

## 9. 测试与验证矩阵

### 9.1 Runtime unit/property/fuzz

覆盖profile codec/migration、registry generation、resolver冲突/依赖、capability admission、environment policy组合、overlay order稳定性与receipt currentness。任意fragment顺序必须得到确定effective policy；invalid/unknown provider应fail-close而不panic。

### 9.2 Geometry与render integration

以同一Scene覆盖static/direct/skinned/morph/VG/terrain/sprite/particle，比较Shaded/WireOverlay/WireOnly可见owner集合、深度和selection表现；对line topology、hidden/xray、缺asset、streaming/reload与device loss生成typed receipt和pixel/geometry golden。

### 9.3 Editor product integration

覆盖每个ViewInstance独立切换、duplicate/floating/Scene/Game、Play override/restore、menu capability状态、profile save/reopen/migration、provider load/unload、HUD effective state与unknown provider preservation。禁止只验证按钮selected或字符串变化。

### 9.4 Diagnostics、fault与performance

验证GPU timing全部状态、observation age/generation、readback queue full、provider pending/failure、environment load failure和frame currentness；采集10K/100K primitive下CPU build/record、GPU pass、upload、allocation、transient/persistent memory、P50/P95/P99与stutter。

### 9.5 跨引擎比较

冻结同一资产、相机、分辨率、display/lighting/environment/overlay语义、warmup和采样帧；功能集合不等价时只报告差异。原始capture、driver/hardware、分析脚本与统计receipt必须可复现。

## 10. Source guards与currentness规则

冻结时production Rust对以下目标名称均为0：`ViewportVisualizationProfile`、`ViewportVisualizationResolver`、`EffectiveViewportVisualizationReceipt`、`ViewportShowFlagRegistry`、`ViewportViewModeRegistry`、`OverlayCompositionPlan`、`ViewportVisualizationPreferenceStore`、`ViewportDebugVisualizationRegistry`。这些名称只是目标合同建议，不应被误读为现有实现。

当前正向证据包括：Scene packet仍有一处`virtual_geometry_debug: None`；wire builder仍读取`wire_segments`并构造CPU world line；provider registry明确保存`owner_id`与`required_capabilities`并隔离fault；实际overlay recorder仍按固定调用录制。实施前若这些guard变化，必须重跑本报告选择集、重算fingerprint并逐项重新判定状态，禁止只更新数字。

## 11. Owner路由与非重复计数

| 依赖 / 已有问题 | Canonical owner | Editor189处理 |
|---|---|---|
| Render Graph inspector、capture/profiler、bake/probe/post-process authoring | Editor22 | 只消费viewport-qualified diagnostics/asset |
| ViewInstance、Scene/Game、多视口与present currentness | Editor179 | 作为profile/session/receipt identity前置条件 |
| Selection、HighlightSet、picking与pointer generation | Editor180 | 作为overlay provider和同帧hit generation消费者 |
| Camera session与projection | Editor187 | 提供camera identity/override入口，不重复导航finding |
| Transform gizmo、grid/workplane/snap | Editor188 | 迁入composition/category，不改solver owner |
| RHI/render graph、material、environment、post-process、quality | Runtime09A/09C/09F1/09H2/99N | Runtime执行effective policy，Editor不绕过 |
| Editor68的29项P1、8项P2 | Editor68 / 本刷新 | 保留canonical编号，不重复增加总数 |

## 12. 最终判定

当前Scene Viewport拥有真实command、render packet、wire/lighting/environment pass、diagnostic DTO和新overlay provider隔离基础，但产品架构仍停留在三值枚举、两个布尔开关、固定pass顺序、共享瞬态状态与CPU wire重建。

最严重的问题不是模式数量少，而是没有一个把request、capability、feature、environment、overlay、diagnostic和per-view persistence解析为同代有效事实的authority。

实施顺序必须从ED68-M0/M1的真实性RED和single resolver开始，再建立per-view identity/persistence、wire hard cutover、lighting/environment、Show Flag/overlay、debug provider与qualified diagnostics。禁止先堆更多toolbar按钮、继续扩张`DisplayMode`、在Editor写专用shader、保留旧bool兼容双写，或把framework global-last diagnostics直接贴到任意Scene pane。

本报告完成current-source refresh，不代表实现完成。21项Open P1、8项Partial P1、7项Open P2、1项Partial P2以及38 Fail/10 Partial资格门必须由代码、产品、动态验证和同语义性能证据逐项关闭。
