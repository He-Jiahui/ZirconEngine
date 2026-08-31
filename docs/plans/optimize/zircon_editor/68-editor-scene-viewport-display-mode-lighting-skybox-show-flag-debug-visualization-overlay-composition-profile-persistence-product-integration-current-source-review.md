---
title: Editor Scene Viewport Display Mode、Lighting、Skybox、Show Flag、Debug Visualization、Overlay Composition、Profile、Persistence、Performance 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor68
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/edit_mode_projection
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/binding_dispatch/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
  - zircon_editor/src/ui/layouts/views/viewport_chrome.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - zircon_runtime/src/core/framework/render/backend_types
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/wireframe
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
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
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/67-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
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
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Display Mode、Lighting、Skybox、Show Flag、Debug Visualization、Overlay Composition、Profile、Persistence、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前Scene Viewport的显示控制不是纯占位。`SceneViewportSettings`、typed `ViewportCommand`、retained toolbar route和render packet已经闭合到真实渲染输入；Shaded、WireOverlay、WireOnly会进入Runtime pass，Preview Lighting会关闭直接灯光buffer、light grid和shadow plan，Preview Skybox会切换procedural environment，grid、scene gizmo、handle与selection overlay也会真实录制。Runtime另外已经具有`RenderFrameProfile`、GPU timing状态、capability summary、virtual geometry debug state/readback和graphics debugger入口。这些基础应保留并收敛，不能再造第二套假统计或纯Editor shader旁路。

但当前系统仍是原型级“几个开关”，不是工程级视口可视化产品。显示策略只有三值`DisplayMode`与`preview_lighting/preview_skybox`两个布尔值；没有Show Flag、material/buffer visualization、lighting/debug family、overlay category profile、capability admission、请求值与有效值receipt，也没有per-view profile identity。所有切换直接写一个共享controller；`Serialize/Deserialize`只提供类型能力，display、lighting、skybox、gizmo并未进入Settings authority或layout/session持久化。

Wireframe尤其不能作为成熟显示模式。它每个交互帧遍历`frame.meshes()`，从streamer回取静态model的`wire_segments`，CPU变换为world-space line vertex并重新分配`HashSet`/`Vec`；direct mesh、deformed/skinned/morph、virtual geometry以及sprite/particle等非model产品没有等价线框来源。WireOnly同时跳过base opaque/transparent shaded replay与OIT，因而缺少线框源的对象会直接消失，而不是以“不可用/降级”被诚实表达。颜色、宽度、拓扑、hidden-line、depth/xray和per-object override也均为硬编码。

Lighting与Skybox的语义同样过粗。Lighting Off不是定义明确的Unlit：它关闭直接灯光和shadow，却把ambient替换为固定`0.55`，当Skybox仍开时environment/IBL仍可参与，post-process也没有由统一resolver决定是否关闭。Skybox Off又不是“只隐藏背景”，而是以`EnvironmentExtract::disabled()`把skybox、reflection probes、baked lighting和probe grid一起归零。Editor不能选择environment asset、rotation、intensity、sun、clear/background、exposure、tonemap、AA、HDR或dynamic resolution，尽管Runtime camera与environment合同已经承载其中不少状态。

Unreal以View Mode解析数百个Show Flag并把buffer、lighting、complexity、LOD、collision和高级renderer visualization纳入同一策略；Godot按视口持久化display/environment/gizmo/information/frame-time/half-resolution/audio状态并对高级模式做renderer method约束；Unity Graphics把rendering/material/lighting debug fragment聚合为一个effective policy，显式推导lighting、post-process和clear color；Fyrox即使规模较小也有持久化debug category；Bevy的wireframe和gizmo有capability gate、render phase、per-entity override、line width、depth bias与render layer。Zircon不能把“按钮能切换且像素发生变化”当作达到这些系统的证据。

Editor22、58、59、67与Runtime渲染报告继续分别拥有通用render tool/capture、multi-viewport产品identity、highlight/picking、transform/grid以及底层renderer feature父合同；本报告不重复抬高其开放问题。本轮新增 **0项P0、29项P1、8项P2**，登记 **48个全部Fail的资格门**。目标是建立stable `ViewportVisualizationProfile`、Runtime唯一`ViewportVisualizationResolver`、generation-qualified `EffectiveViewportVisualizationReceipt`、可扩展debug/show-flag/overlay provider、GPU-native wireframe parity以及Editor per-view session/persistence/diagnostic projection。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、GPU render golden、save/reopen、multi-viewport、capability fallback、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。当前不能声称Scene Viewport显示、诊断、表现或性能达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Editor visualization state and product route | **23 / 3,367 / 3,109 / 128,089 / 10** | settings、packet、command/codec、toolbar、chrome、play transition与project settings | `af46616849505fb25156c784dcee2be537411228c1809951870208cee1ad0c26` |
| Zircon Runtime visualization contracts and execution | **19 / 5,334 / 4,892 / 185,553 / 47** | camera/overlay/environment、wire/base/OIT、lighting/shadow、post-process/runtime feature | `3ea33df6015dfa90a9d3a7323f629813ff53c6e210d3caf64bbdc7009c44e2aa` |
| Zircon render diagnostics and Editor consumption | **10 / 1,857 / 1,728 / 78,364 / 13** | frame profile、GPU timing、capabilities、framework query与diagnostics pane | `0618086f87d782945ece4e2e871151cd40bf8a8e1d240e97b68e4f1862b2e213` |
| Zircon focused Editor tests | **10 / 2,310 / 2,089 / 80,333 / 53** | packet/state/play、binding/dispatch、toolbar/template/chrome与diagnostics body | `ffe0abc00b1e3ef5799f14f355f053719dbaadc349f08f30bcc796a2c8d39aa7` |
| Unreal selected set | **5 / 3,448 / 3,019 / 153,462 / 0** | Show Flag/View Mode resolver、buffer registry与per-instance viewport persistence | `a48e699d48cab98110e071fcd2fabfd7f3e273b832c14294dcfe28e53305497a` |
| Godot selected set | **2 / 7,927 / 6,790 / 311,055 / 0** | display/debug modes、per-view state、environment/gizmo/info/perf与capability metadata | `464935fa172c4d19cd40bd8af41ed42df20f050e28c19155962826c748f766d7` |
| Fyrox selected set | **5 / 2,691 / 2,463 / 100,015 / 0** | persistent debug categories、graphics quality、grid、mode consumer与save | `cfefaea885bbad9871303b7d7a32aa716c9d4d406c52532fa426d8f1426fb042` |
| Bevy selected set | **3 / 2,469 / 2,247 / 90,658 / 0** | GPU wire phase、capability gate、gizmo group config、diagnostic overlay | `eb3a4dd20cbd947ee36eb3d26cb01b780a462c19dda1f491ac37118fdd2c1a4c` |
| Unity Graphics selected set | **8 / 1,858 / 1,644 / 83,647 / 0** | modular debug fragments、effective query、serializer与SceneView mode validation | `ae958cf51605919ed8e371854fd3ca90687d468a4a08de4e28227b6656333a8a` |

fingerprint按规范化相对路径排序，并将每个`path + newline + file SHA-256 + newline`聚合后再做SHA-256；它只证明本轮读取的working-tree语料，不是ABI、artifact、动态结果或性能receipt。主仓与Unreal镜像基线为`bee4c707b714738346b49bba15c59468b8bd9b39`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。

### 2.2 在途修改隔离

冻结时`pane_payload_builders/runtime_diagnostics.rs`含非本轮修改：把缺省snapshot从clone改为借用局部default，减少一次clone。它不改变当前pane只投影backend、viewport/frame数量和少量Hybrid GI/virtual geometry计数，也没有新增per-view frame profile消费；本轮按working tree计入fingerprint，实施前必须重算。相邻viewport pointer/input文件与renderer graph/IBL文件还有其他Session在途修改，但均未计入本轮冻结集合，也不据此推导显示策略结论。

共享索引含Editor60-67及其他current-source review的在途修改，本轮只追加Editor68并更新对应汇总，不覆盖或回退其他Session内容。coordinator Session为`optimize-editor68-viewport-visualization-review-r1-20260822`，baseline epoch为339；新报告和三个共享索引取得精确lease。MVP `00-current-source-baseline-recovery`仍为`in_progress`，且已有Cargo验证占用，因此本轮不以共享工作树Cargo结果包装静态审查结论。

### 2.3 范围与非范围

本报告拥有Scene Viewport的display/view mode、preview lighting/environment/background、Show Flag、debug/buffer visualization、overlay category/composition、per-view visualization profile、persistence、effective receipt、产品UI和per-view performance projection。

Editor22继续拥有通用Render Graph inspector、capture/profiler、lighting bake、reflection probe与post-process asset authoring；Editor58拥有ViewInstance/Scene-Game/multi-viewport identity和present currentness；Editor59拥有selection/highlight/picking；Editor67拥有transform gizmo和grid/workplane/snap；Runtime09A/09C/09F1/09H2/99N拥有底层RHI、material、environment、post-process与quality/scalability执行。Editor68只定义这些父产品必须接受的viewport-specific policy、identity与receipt，不重复计数父P0。

## 3. 当前实现拓扑与可保留基础

### 3.1 Typed command到render packet是真实闭环

`ViewportCommand::SetDisplayMode/SetPreviewLighting/SetPreviewSkybox/SetGizmosEnabled`经codec和dispatch进入`SceneViewportController::apply_command`，随后`build_render_packet`投影到`ViewportRenderSettings`、`PreviewEnvironmentExtract`和`RenderOverlayExtract`。这是可保留的控制链；问题是payload只表达enum/bool，没有profile generation、view identity、request/effective split或rejection。

### 3.2 Runtime已有足够丰富的相机与环境DTO

`ViewportCameraSnapshot`已经承载HDR、EV100 exposure、MSAA、dynamic resolution和temporal jitter；`EnvironmentExtract`可表达procedural/source cubemap、rotation/intensity、reflection probes、baked lighting和probe grid。Editor当前没有把这些能力纳入Scene Viewport visualization session，反而用两个布尔值把整个环境压缩为procedural或disabled。

### 3.3 Lighting开关会真实裁剪直接灯光与shadow

`pack_lighting_extract*`在lighting disabled时返回空buffer，light grid基于该空结果，shadow plan也提前退出。该路径不是no-op，应保留。问题是`SceneUniform`同时注入固定0.55 ambient，environment和post-process没有由同一policy resolver推导，因此它不能被命名为严格Unlit、Lighting Only或Detail Lighting。

### 3.4 Overlay有独立pass，但组合顺序是编译期固定调用

selection、wireframe、grid、scene gizmo、handle各有pass和prepared buffer，证明基础分层存在。`record_overlays`却固定按selection -> wireframe -> grid -> scene gizmo -> handle调用；extract只有固定字段，没有category descriptor、screen/world layer、depth/xray、priority、mutual exclusion、provider capability或effective reason。

### 3.5 Runtime virtual geometry debug产品已经存在但Editor硬编码None

`RenderVirtualGeometryDebugState`包含forced mip、freeze cull、BVH/visbuffer visualization和leaf cluster输出，Runtime还有snapshot/readback stream。`SceneViewportExtractRequest`已经预留`virtual_geometry_debug`，但Editor `build_render_packet`永远传`None`，独立Runtime Diagnostics pane也只显示“available”和visible cluster数，用户无法请求、观察和恢复这些模式。

### 3.6 RenderStats与FrameProfile是真实观测基础

Runtime暴露current CPU profile、异步resolved GPU profile、GPU timing状态、per-pass draw/instance/state/upload/dispatch、subsystem budget、transient/persistent memory、visibility、graph、degrade和capability数据。Editor diagnostics pane只投影backend、active viewports、submitted frames及少量GI/VG条目；Scene Viewport HUD只显示scene mode、projection、display和grid。目标应复用这些结构，而不是在HUD里重新计时。

### 3.7 现有测试证明route和布尔传播，不证明显示产品正确

focused Editor测试能证明toolbar cycle、typed command、packet lighting/skybox flag和HUD glyph会到达真实frame；Runtime测试能证明部分lighting/shadow guard。Wireframe唯一focused test只用`include_str!`检查早返回源码顺序；没有对line geometry、缺失model、deformed/VG/sprite parity、WireOnly对象守恒或实际像素做断言。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：View Mode解析Show Flag，而不是把所有语义塞进枚举

`FEngineShowFlags`提供分组、遍历、字符串持久化和custom flag注册，并明确Show Flag不是scalability；`ApplyViewMode`和`EngineShowFlagOverride`把Lit、Unlit、Wireframe、Lighting Only、Detail Lighting、shader/light complexity、quad overdraw、LOD/HLOD、streaming、collision、buffer、Nanite和Lumen等高层模式解析为协调后的有效flag集合。`BufferVisualizationData`又通过registry组织BaseColor、Specular、WorldNormal、Depth、Roughness、Metallic、ShadingModel和pre/post tonemap HDR等目标。

`FLevelEditorViewportInstanceSettings`按实例保存perspective/orthographic mode、Editor/Game show flags、buffer/subsystem visualization、exposure、FOV、realtime和enabled stats。可借鉴的是“stable mode/profile -> single resolver -> effective flags/receipt -> per-view persistence”，不是复制Unreal宏、全局CVar或类层级。

### 4.2 Godot：显示、环境、gizmo、diagnostic与质量都是per-view产品状态

Godot 3D viewport覆盖normal、wireframe、overdraw、lighting、unshaded，以及shadow split/atlas、normal buffer、decal/area light atlas、VoxelGI/SDFGI、luminance、SSAO/SSIL、GI buffer、disable LOD、cluster、occluder、motion vector和internal buffer。高级菜单项携带supported rendering method metadata，而不是点击后静默失败。

`get_state/set_state`保存display mode、use environment、gizmos、transform gizmo、grid、information、frame time、half resolution、audio listener/doppler和camera state。它证明per-view persistence、capability-gated menu和可切换诊断不是后期装饰，而是Scene Viewport基本产品合同。

### 4.3 Fyrox：较小实现也有typed persistent debug categories

Fyrox settings分别保存physics、bounds、TBN、terrain、light bounds、camera bounds和pictogram size，graphics settings保存quality与grid；Scene按类别条件消费，settings通过dirty notification保存RON。Scene viewer的Shaded/Wireframe本身不如Unreal完整，因此不能作为最终显示模式金标准；但它足以证明Zircon单一`gizmos_enabled`无法替代typed category与持久化consumer。

### 4.4 Bevy：Wireframe与Gizmo走GPU phase、capability和group config

Bevy wireframe在插件装配时检查所需GPU feature，不满足就告警并拒绝加载；实现进入binned render phase，支持global/per-entity enable/disable、color、screen-space width、triangle/quad topology，并区分薄线polygon mode与宽线vertex-pulling路径。它不要求每帧在CPU把每条边变换成world-space line vertex。

`GizmoConfigStore`按config group扩展，具有enabled、line width、perspective、solid/dotted/dashed、line joints、depth bias和render layers。diagnostics overlay对缺失数据诚实显示，并限制文本刷新频率。Zircon应吸收“provider group + render layer/depth + bounded observation”思想，不照搬ECS API。

### 4.5 Unity Graphics：模块化debug fragment最终解析为一个有效策略

Core `IDebugDisplaySettingsQuery`统一询问active、post-process allowed、lighting active和screen clear color；URP把Rendering、Material、Lighting分成独立settings data，再聚合为有效debug policy。Material debug启用时会显式关闭lighting/post-process，lighting debug按具体mode决定post-process，wireframe/solid-wireframe/shaded-wireframe与overdraw也先做互斥解析。serializer保存可序列化debug state，URP/HDRP SceneView注册validation callback拒绝不支持的draw mode。

本地Unity Graphics镜像不是Unity Editor完整Scene View源码，不能据此声称复核了全部Unity视口交互；它在本报告中只作为“模块化设置、单一resolver、serialization和capability rejection”的直接证据。

## 5. 差异矩阵

| 能力 | Zircon current source | Unreal / 其他参考 | 结论 |
|---|---|---|---|
| View mode | Shaded/WireOverlay/WireOnly闭集 | stable mode catalog + resolver | 原型枚举 |
| Show flags | 无；只有grid/gizmo和preview bool | grouped/extensible flags | 产品域缺失 |
| Wire source | streamed static model wire segments | GPU phase/polygon or pulling、多产品支持 | 覆盖与性能不足 |
| WireOnly truth | 跳过shaded/OIT，缺线源对象消失 | capability/fallback/effective state | 不诚实降级 |
| Material/buffer | 无 | material、attribute、GBuffer、depth、HDR | 产品域缺失 |
| Lighting modes | direct lighting bool | Lit/Unlit/Lighting Only/Detail/complexity | 语义过粗 |
| Environment | procedural/disabled bool | background、IBL、probe、asset、rotation/intensity分离 | authority混叠 |
| Camera overrides | Runtime有HDR/exposure/MSAA/dynres，Editor不消费 | per-view exposure/FOV/quality | 产品缺口 |
| Debug renderer | VG state存在但Editor传None | Nanite/VG/GI/cluster/debug families | 接线缺失 |
| Overlay categories | fixed fields和固定顺序 | group/layer/depth/category/provider | 不可组合 |
| Gizmo visibility | 单bool；仅Camera/DirectionalLight生成 | typed category与完整provider | 覆盖不足 |
| Capability | 无requested/effective/rejection | mode validation和feature gate | 不能fail-close |
| Persistence | display/lighting/skybox transient | per-view/versioned serialized state | 未闭环 |
| Multi-view | 单controller共享状态 | per-instance profile | 依赖Editor58 identity |
| Diagnostics | global pane少量字符串；HUD固定4段 | per-view stats、timing、missing state | 未消费既有数据 |
| Tests | route/flag与源码词法为主 | render golden、mode/cap/device矩阵 | 资格证据不足 |

## 6. 新增发现

### 6.1 P1：架构、正确性与产品闭环

#### ED68-P1-01：没有单一Viewport Visualization Profile与resolver

`SceneViewportSettings`把display enum、lighting bool、skybox bool、grid和gizmo直接平铺，`build_render_packet`再逐字段复制。任何新模式都只能继续加字段和分支，lighting、environment、post-process、overlay与clear policy不会共同解析。必须建立stable profile fragment与Runtime唯一resolver，输出不可变effective policy。

#### ED68-P1-02：DisplayMode是闭集三循环，没有stable descriptor或扩展目录

toolbar通过`next_display_mode_name`硬编码Shaded -> WireOverlay -> WireOnly，codec和HUD又维护字符串match。没有stable id、display metadata、category、mutual exclusion、required capability、provider owner、serialization version或deprecated mapping。新renderer/plugin无法贡献模式而不修改核心枚举和所有穷举分支。

#### ED68-P1-03：请求没有view/profile generation，返回也没有effective receipt

command只携带mode/bool，render request没有ViewInstance、profile generation或transition token；UI只能假定写入值就是实际值。必须返回`EffectiveViewportVisualizationReceipt`，至少含view/document/source/profile generation、requested/effective mode、resolved flags、fallback/rejection、capability snapshot和frame generation。

#### ED68-P1-04：Wireframe只覆盖streamer中存在的静态model wire segments

producer遍历`frame.meshes()`后再次按model id查询streamer；model缺失就skip，并使用asset中静态`wire_segments`。direct mesh、runtime procedural geometry、deformed/skinned/morph最终顶点、virtual geometry、terrain、sprite和particle没有同代线框source。必须在canonical geometry submission层生成或选择wire draw，不得从过时asset副本猜最终几何。

#### ED68-P1-05：WireOnly通过删掉shaded产品制造不完整画面

Base pass在WireOnly跳过opaque与transparent replay，OIT也提前返回；wire producer没有覆盖的产品因此直接消失。当前没有degraded badge、unsupported object count或fallback policy，用户会把“没画出来”误判为Scene没有对象。必须规定每类primitive的wire parity或明确的per-object rejection/fallback。

#### ED68-P1-06：Wireframe每个交互帧CPU重建全部world-space line buffer

`prepare_buffers`每帧调用wire builder，后者分配selection `HashSet`和vertex `Vec`，遍历所有mesh/segment并在CPU做model transform。没有asset topology cache、instance buffer、dirty generation、visibility reuse、GPU phase、budget或large-scene backpressure。目标至少需要immutable topology artifact + instanced/GPU transform，宽线另走专用pipeline。

#### ED68-P1-07：Wire style、depth与topology政策全部硬编码

颜色固定为三组`Vec4`，没有line width、triangle/quad边界、crease/boundary edge、hidden line、occluded tint、depth test、xray、selection override、render layer或per-entity `NoWireframe`。这使WireOverlay既不能用于建模检查，也无法对复杂Scene保持可读性。

#### ED68-P1-08：Material、vertex attribute、GBuffer与full-screen buffer visualization完全缺失

没有BaseColor、Normal、Roughness、Metallic、Depth、Motion Vector、Object/Material ID、UV、tangent、lightmap density、HDR pre/post tonemap等mode或registry。Editor22拥有通用debug surface，本报告要求Scene Viewport profile能选择Runtime提供的typed visualization target，并得到有效/不支持receipt。

#### ED68-P1-09：Lighting mode没有形成工程级模式族

当前只有direct lighting on/off，缺Lit、Unlit、Lighting Only、Detail Lighting、Reflections、Diffuse/Specular、Shadow Cascades、Light Complexity、Cluster Occupancy等稳定语义。不能用不断叠加boolean替代互斥mode与feature mask；resolver必须明确每种模式对material、lighting、shadow、IBL和post-process的影响。

#### ED68-P1-10：Lighting Off不是定义明确且可验证的Unlit

关闭时direct lights/light grid/shadow为空，但ambient被替换为固定0.55，environment仍由Skybox bool独立决定，post-process feature也没有统一关停。结果随skybox和pipeline配置变化，且UI只显示Light开关。必须为Unlit定义material override、ambient/background、IBL、fog、post-process和transparency政策，并由receipt公开。

#### ED68-P1-11：Skybox开关错误耦合背景、IBL、probe与baked environment

`from_preview_skybox_enabled(false)`返回`EnvironmentExtract::disabled()`，会同时清空skybox、reflection probes、baked lighting和probe grid；true则强制procedural default。用户无法“隐藏背景但保留IBL”，也无法预览authored/source cubemap环境。必须把background visibility、environment lighting source、probe/baked participation分成独立profile fragment。

#### ED68-P1-12：缺Environment Preview Scene与可控参数

Editor不能选择source cubemap/HDRI、rotation、intensity、sun direction/intensity、ground/horizon/zenith、clear color或neutral studio profile。`SCENE_CLEAR_COLOR`和procedural gradient固定在代码中。需要versioned environment profile、resource handle/generation、async readiness与fallback receipt。

#### ED68-P1-13：已有camera/render quality能力没有进入视口显示产品

Runtime camera含HDR、exposure EV100、MSAA、dynamic resolution与temporal jitter，quality/capability也另有合同；Editor68 profile不表达这些override或effective state。Scene Viewport因此无法做fixed exposure、post-process bypass、AA compare、half-res/debug quality或稳定截图，且用户看不到默认profile是否降级。

#### ED68-P1-14：没有Show Flag registry和内容类别过滤

mesh、sprite、particle、light、camera、audio、collision、navigation、AI、volume、decal、fog、bounds、LOD/HLOD等内容没有typed show flag。单`gizmos_enabled`和grid enum无法承担内容/overlay/diagnostic三类不同权限。需要stable flag descriptor、group、default、scope、provider owner、dependency/conflict与effective mask。

#### ED68-P1-15：Overlay extract不是可组合category profile

`RenderOverlayExtract`固定持有highlights、selection anchors、grid、handles、scene gizmos和display mode，无法表达category id、priority、screen/world space、depth、xray、layer mask、blend或disabled reason。必须把overlay payload和composition descriptor分离，并保证同一frame的pointer/hit product消费同一effective category generation。

#### ED68-P1-16：第一方Scene Gizmo覆盖只到Camera和DirectionalLight

`build_scene_gizmos`明确跳过Ambient/Point/Rect/Spot light及Empty/Cube/Mesh；icon enum也只有Camera/DirectionalLight。Point/Spot/Rect light range/cone/shape、audio、probe、volume、camera clip等工程常用可视化不存在。Editor67拥有transform gizmo数学，本项只要求typed component visualization provider和类别开关。

#### ED68-P1-17：Overlay顺序与深度政策是固定源码调用

pass顺序固定为selection、wireframe、grid、scene gizmo、handle，无法按mode重排、隔离screen-space overlay、选择always-on-top/xray或解决plugin conflict。`PASS_ORDER`还只在test cfg暴露，产品没有可查询的effective composition receipt。需要编译后的overlay plan和稳定tie-break规则。

#### ED68-P1-18：扩展overlay provider不能声明组合与能力合同

现有Editor host registry/ToggleOverlayProvider是可保留入口，但provider payload最终仍塞入固定scene gizmo通道，不能声明category、layer、depth、priority、settings schema、capability或cost。开放的plugin overlay runtime wiring failure继续由Editor05拥有；Editor68只要求该接线最终落入同一composition plan，而不是另建plugin pass旁路。

#### ED68-P1-19：Virtual Geometry debug入口已存在却被Editor永久关闭

Scene request每帧硬编码`virtual_geometry_debug: None`，forced mip、freeze cull、BVH/visbuffer和cluster readback无法从Scene Viewport触达。独立diagnostics pane显示available并不等于可控制。需要typed debug provider、per-view request、capability/readiness admission、异步snapshot age和退出模式后的完整恢复。

#### ED68-P1-20：跨子系统debug visualization没有统一目录

Collision、navigation mesh、AI perception、physics shapes、audio attenuation、light/probe bounds、visibility/HZB、LOD、streaming与cluster等没有统一Scene Viewport debug mode/category入口。各模块未来若直接画线会形成多个不可排序authority。必须通过provider registry贡献descriptor、extract、cost/capability与diagnostic，而不是修改核心toolbar枚举。

#### ED68-P1-21：Serializable viewport settings没有真实持久化闭环

`SceneViewportSettings`虽derive Serialize/Deserialize，Settings registry只注册translate/rotate/scale snap。display、lighting、skybox、gizmo和view mode没有user/project/session/layout key、schema version、migration、load/save或invalid-source反馈；重启即回默认。必须明确哪些是per-view layout state、user preference、project profile或临时session override。

#### ED68-P1-22：没有per-view visualization profile identity

一个`SceneViewportController`持有一份settings，Editor58已经证明Scene/Game/复制/浮动view尚未拥有独立render product。Editor68不能再把profile绑到全局controller；必须消费Editor58的ViewInstanceId/epoch，使每个view可以独立Lit/Wire/Debug、environment和stats，并在view销毁时释放provider lease。

#### ED68-P1-23：Edit、Play、Scene与Game没有显式profile transition政策

进入Play只捕获并关闭gizmo，退出只恢复gizmo；display、lighting、skybox和debug mode沿用共享瞬态值，没有Scene/Game默认、inheritance、temporary override、restore token或runtime-authoritative restriction。需要context-aware profile stack，退出临时debug或Play必须按generation恢复原有效状态。

#### ED68-P1-24：Toolbar只有icon cycle/toggle，没有可检查的模式产品

Display按钮只循环三态，Lighting/Skybox/Gizmo只反转当前snapshot；没有菜单、搜索、分组、当前值、请求/有效差异、disabled/busy/degraded、provider loading或unsupported reason。复杂模式不能靠继续堆28px图标。需要descriptor-driven menu/segmented primary modes和状态投影。

#### ED68-P1-25：Viewport HUD遗漏关键状态并可能报告错误事实

固定HUD只显示scene mode、projection、display和grid，不显示lighting、environment、debug target、overlay profile、quality、capability、fallback、GPU timing状态或frame generation。它直接读requested settings，而不是render receipt，因此即使Runtime拒绝/降级也会显示请求值。必须只投影effective receipt和observation age。

#### ED68-P1-26：既有RenderStats没有viewport-qualified消费合同

`RenderStats`含丰富last-frame数据，但整体是framework全局last值；diagnostics pane也按全局snapshot显示，无法证明数据属于当前Scene view、Game view还是另一pipeline generation。Editor22拥有通用diagnostics bridge；Editor68需要其提供`ViewportFrameObservation { viewport, product_generation, profile_generation, age }`，禁止把global last值贴到任意pane。

#### ED68-P1-27：per-view diagnostics没有bounded sampling与cost policy

host只在Runtime Diagnostics surface可见时触发presentation refresh，这是正确的避免全Workbench重建方向，但Scene Viewport没有独立stats visibility、采样频率、history长度、readback budget或drop/stale状态。必须按overlay visibility和capability启停GPU query/readback，文本刷新限频，渲染帧与UI展示解耦。

#### ED68-P1-28：没有mode admission、capability validation和fail-close

Runtime capability summary已表达timestamp、pipeline statistics、readback、storage、indirect、VG等能力，profile validator也能拒绝缺失能力；viewport display command完全不调用这些合同。未来debug mode若不支持只能静默无效或走错误路径。每个descriptor必须声明requirements，resolver返回Supported/Degraded/Pending/Unavailable与原因。

#### ED68-P1-29：测试没有覆盖视觉守恒、转换、持久化和性能资格

现有测试集中在route、flag传播、HUD glyph和少数guard；没有三类wire模式的真实几何/pixel golden、primitive family parity、lighting/environment组合、background-vs-IBL、mode conflict、capability rejection、per-view isolation、save/reopen、Play restore、provider unload、stats age或large-scene budget。必须先建RED矩阵再重构，不能以新增更多字符串测试替代。

### 6.2 P2：质量、可维护性与资格证据

#### ED68-P2-01：颜色、clear、icon size、pick radius和HUD尺寸散落为裸常量

这些值没有profile/theme/scale来源，也没有HDR/SDR或DPI政策。应收敛为typed visual style和component descriptor，避免每个provider复制magic constant。

#### ED68-P2-02：固定280x28 HUD不支持长模式名、localization和多行状态

HUD使用`Wrap::None`、固定高度和手写英文标签；custom scene mode名或详细degraded reason可能截断。应使用结构化overlay layout、最小刷新和可折叠diagnostic，而不是继续扩展拼接字符串。

#### ED68-P2-03：mode label与字符串codec在多个层重复穷举

DisplayMode在codec、cycle、HUD、chrome和测试中分别维护名称。stable descriptor应成为label/localization/icon/serialization id的单一来源，迁移后删除平行match和legacy string猜测。

#### ED68-P2-04：Toolbar图标语义模糊且缺状态辅助信息

Display和Lighting复用lit图标，icon-only控件没有模式详情、有效值和不支持原因投影。descriptor-driven tooltip/accessibility label应说明当前有效模式，不能只给静态“Display/Light”。

#### ED68-P2-05：Wireframe测试是源码词法断言，不是行为测试

`include_str!`只验证early return出现在selection/mesh loop前，重命名或等价重构会误报，错误geometry也可通过。应替换为builder output、allocation counter、render graph和pixel receipt测试。

#### ED68-P2-06：没有用户可发现的mode catalog与provider provenance

复杂debug mode需要category、owner、description、cost、shortcut和availability；当前只有三个隐式cycle名称。产品应能列出“为什么不可用”和由哪个Runtime/plugin提供，但不在画面里堆使用说明文本。

#### ED68-P2-07：缺少profile schema/version/migration telemetry

即使后续把struct直接序列化，也无法处理模式重命名、provider缺失、字段新增、project/user层合并和invalid fallback。需要显式schema version、migration report和unknown provider preservation。

#### ED68-P2-08：没有同语义跨引擎视觉与性能基线

不能拿不同Scene、不同分辨率或不同功能集合比较“优于Unreal”。需要冻结scene/camera/profile、warmup、frame count、GPU/driver、输出正确性、CPU/GPU/memory和stutter分位数，并保留原始capture与分析脚本receipt。

## 7. 目标架构与职责边界

### 7.1 Runtime：唯一解析和执行有效可视化政策

Runtime中立合同建议至少包含：

```text
ViewportVisualizationRequest
  - qualified viewport/view-family identity
  - profile id + schema/profile generation
  - display/material/lighting/environment fragments
  - show-flag and overlay requests
  - diagnostic requests + observation budget

ViewportVisualizationResolver
  - registry snapshot + backend capabilities + pipeline features
  - conflict/dependency/exclusivity resolution
  - requested -> effective policy + rejection/fallback

EffectiveViewportVisualizationReceipt
  - effective modes/flags/overlay plan/environment/quality
  - capability and provider generations
  - degraded/pending/unavailable reasons
  - produced frame/product generation
```

Runtime graphics provider拥有mode implementation、GPU resource/pipeline和primitive parity；core framework只暴露中立descriptor/request/receipt，不引用Editor UI。Wireframe必须从canonical geometry/topology artifact和instance/deformation generation消费；material/buffer/debug mode必须接入render graph与feature/capability resolver，禁止Editor直接替换shader或读内部buffer。

### 7.2 Editor：per-view session、profile persistence和产品编排

Editor应建立`SceneViewportVisualizationSessionRegistry`，key来自Editor58的ViewInstanceId/epoch，持有profile selection、temporary override stack、provider menu snapshot、last effective receipt和observation subscription。持久化层按User/Project/Layout/Session scope保存stable ids与version，provider缺失时保留unknown entry并显示Unavailable，不能静默改回Shaded后覆盖用户配置。

Toolbar只保留少量高频primary mode，其他Show Flag、Lighting、Environment、Buffer/Debug、Overlay和Stats进入descriptor-driven menu/panel。所有UI、HUD、shortcut和remote command投影同一个session state/effective receipt；Play/Game transition使用显式override token，销毁view或卸载provider会取消subscription并释放资源。

### 7.3 Owner规则

1. Runtime core拥有中立profile/request/receipt/capability合同，graphics拥有实际pass/pipeline/provider；Editor不得复制render truth。
2. Editor68拥有per-view visualization session、persistence、产品UI和composition request；Editor58提供view/product identity和currentness。
3. Editor22提供通用render diagnostics/capture/asset authoring；Editor68只消费viewport-qualified observation，不另建profiler authority。
4. Editor59的HighlightSet和Editor67的transform/grid作为overlay provider进入同一plan；其交互数学与selection语义仍由原报告拥有。
5. Runtime feature父报告负责底层correctness/performance，Editor68通过capability/effective receipt消费，不以UI workaround关闭父差距。

## 8. 分阶段重构计划

### ED68-M0：真实性止血与RED基线

冻结当前三模式/两开关输出，增加WireOnly对象守恒、lighting/environment组合、unsupported/pending、per-view隔离和save/reopen RED。UI在没有effective receipt前不得把请求值标成已生效；为WireOnly缺线源和VG debug不可达增加可观测诊断。

### ED68-M1：Stable profile schema、registry与resolver

定义stable mode/flag/category/provider id、versioned `ViewportVisualizationProfile`、requirements/conflicts/dependencies及Runtime resolver。建立requested/effective/rejected typed receipt，删除继续扩张`DisplayMode` boolean fan-out的路径。

### ED68-M2：Per-view session、scope与persistence

依赖Editor58 ViewInstance identity建立session registry；实现User/Project/Layout/Session合并、migration、unknown provider preservation、Edit/Play/Scene/Game override stack及generation-qualified restore。

### ED68-M3：Wireframe hard cutover与primitive parity

建立immutable topology artifact、GPU/instanced transform路径、thin/wide line pipeline和visibility reuse；覆盖static/direct/deformed/VG/terrain/sprite/particle策略。旧CPU全量world-line builder在等价golden和budget通过后删除，不保留compat shim。

### ED68-M4：Lighting、Environment与Camera override resolver

实现Lit/Unlit/Lighting Only/Detail等基础模式，分离background、IBL、probe/baked participation，接入environment asset/rotation/intensity与fixed exposure/tonemap/post-process/AA/HDR/quality override。所有组合由一个effective policy推导。

### ED68-M5：Show Flag与Overlay Composition

建立grouped show-flag registry和compiled overlay plan，支持category/layer/depth/xray/priority/cost；把selection、grid、transform、scene component gizmo和plugin provider硬切到同一合同，补齐first-party light/probe/volume visualization。

### ED68-M6：Material、Buffer与Subsystem Debug Provider

接入material/vertex/GBuffer/full-screen target registry以及VG、visibility/HZB、LOD/streaming、collision、navigation、AI、physics、audio等provider。每个mode具备capability/readiness、async observation、退出恢复和资源lease。

### ED68-M7：Viewport-qualified diagnostics与bounded observation

Editor22/Runtime提供按viewport/product/profile generation绑定的frame observation；Scene Viewport stats overlay按可见性采样、限频刷新并标注Pending/Unavailable/Stale。GPU query、readback、history和文本更新均有预算与drop计数。

### ED68-M8：单一产品投影与旧路径删除

Toolbar/menu/HUD/shortcut/command全部从session + effective receipt生成；删除硬编码三循环、bool toggle猜测、重复label match和直接读requested settings的HUD。禁止双写legacy `SceneViewportSettings`与新profile。

### ED68-M9：产品资格与跨引擎基线

完成render golden、save/reopen、multi-view/Play/provider reload、capability matrix、fault/soak/profile和跨引擎同语义benchmark。只有正确性、可达性、currentness和统计显著性同时成立，才能讨论达到或超过Unreal。

## 9. 资格门

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
| ED68-G09 | static model wire geometry与shaded topology一致 | Fail |
| ED68-G10 | direct/procedural mesh wire parity成立 | Fail |
| ED68-G11 | skinned/morph/deformed wire generation同代 | Fail |
| ED68-G12 | VG/terrain/sprite/particle有明确parity或rejection | Fail |
| ED68-G13 | WireOnly不静默丢失任何可见primitive | Fail |
| ED68-G14 | line width/topology/depth/xray政策可配置 | Fail |
| ED68-G15 | large-scene wire path无每帧全量CPU world-line rebuild | Fail |
| ED68-G16 | wire CPU/GPU/memory budget达到冻结阈值 | Fail |
| ED68-G17 | Lit/Unlit/Lighting Only/Detail语义被golden验证 | Fail |
| ED68-G18 | Lighting policy统一推导direct/ambient/IBL/shadow/post-process | Fail |
| ED68-G19 | background visibility与environment lighting分离 | Fail |
| ED68-G20 | authored/source environment asset可选且generation-qualified | Fail |
| ED68-G21 | exposure/tonemap/AA/HDR override显示effective值 | Fail |
| ED68-G22 | environment pending/failure有fallback receipt | Fail |
| ED68-G23 | Play/Scene/Game profile transition可逆 | Fail |
| ED68-G24 | save/reopen保持同一有效environment/profile | Fail |
| ED68-G25 | grouped Show Flag覆盖第一方内容类别 | Fail |
| ED68-G26 | overlay plan含category/layer/depth/xray/priority | Fail |
| ED68-G27 | selection/grid/transform/component gizmo进入同一plan | Fail |
| ED68-G28 | plugin provider可声明capability、cost和settings | Fail |
| ED68-G29 | provider unload取消资源/subscription且不留旧像素 | Fail |
| ED68-G30 | VG debug可从Editor请求、观察、恢复 | Fail |
| ED68-G31 | material/buffer debug target有typed registry | Fail |
| ED68-G32 | collision/nav/AI/physics/audio等不会各建旁路 | Fail |
| ED68-G33 | 每个ViewInstance拥有独立profile/session | Fail |
| ED68-G34 | 复制、浮动、Scene与Game不会串改显示状态 | Fail |
| ED68-G35 | User/Project/Layout/Session scope优先级确定 | Fail |
| ED68-G36 | toolbar/menu/HUD都投影effective receipt | Fail |
| ED68-G37 | unavailable/degraded/pending原因可见且可访问 | Fail |
| ED68-G38 | command/shortcut/remote使用同一typed intent | Fail |
| ED68-G39 | 长模式名/localization/DPI下布局不截断 | Fail |
| ED68-G40 | profile migration和invalid source有diagnostic | Fail |
| ED68-G41 | observation绑定viewport/product/profile generation | Fail |
| ED68-G42 | GPU timing Disabled/Unavailable/Pending/Measured不混淆 | Fail |
| ED68-G43 | stats overlay采样、历史和文本刷新有预算 | Fail |
| ED68-G44 | render golden覆盖mode/environment/overlay组合 | Fail |
| ED68-G45 | capability/device/pipeline matrix覆盖fallback | Fail |
| ED68-G46 | provider reload、fault与长时soak无泄漏/陈旧状态 | Fail |
| ED68-G47 | 10K/100K primitive场景满足CPU/GPU/memory阈值 | Fail |
| ED68-G48 | 同Scene/profile/hardware跨引擎基线可复现且有统计意义 | Fail |

## 10. 测试与验证矩阵

### 10.1 Runtime unit / property / fuzz

覆盖profile codec/migration、registry generation、resolver冲突/依赖、capability admission、environment policy组合、overlay order稳定性和receipt currentness；对任意flag fragment顺序应产生确定effective policy，对invalid/unknown provider必须fail-close而不panic。

### 10.2 Geometry与render integration

以同一Scene覆盖static/direct/skinned/morph/VG/terrain/sprite/particle，比较Shaded/WireOverlay/WireOnly可见owner集合、深度与选择表现；对line topology、hidden/xray、缺asset、streaming/reload和device loss生成typed receipt与pixel/geometry golden。

### 10.3 Editor product integration

覆盖每个ViewInstance独立切换、复制/浮动/Scene/Game、Play override/restore、menu capability状态、profile save/reopen/migration、provider load/unload、HUD effective state和unknown provider preservation。禁止仅验证按钮selected或字符串改变。

### 10.4 Diagnostics、fault与performance

验证GPU timing全部状态、observation age/generation、readback queue full、provider pending/failure、device capability缺失、environment load failure和frame currentness；采集10K/100K primitive下CPU build/record、GPU pass、upload、allocation、transient/persistent memory、P50/P95/P99和stutter。

### 10.5 跨引擎比较

冻结同一资产、相机、分辨率、display/lighting/environment/overlay语义、warmup与采样帧；记录Unreal、Godot、Fyrox、Bevy适用切片与Unity Graphics consumer差异。功能集合不等价时只报告差异，不生成“更快”结论。

## 11. Owner路由与非重复计数

| 依赖 / 已有问题 | Canonical owner | Editor68处理 |
|---|---|---|
| Render Graph inspector、capture/profiler、bake/probe/post-process authoring | Editor22 | 只消费viewport-qualified diagnostics/asset，不重复P0/P1 |
| ViewInstance、Scene/Game、多视口与present currentness | Editor58 | 作为profile/session/receipt identity前置条件 |
| Selection、HighlightSet、picking与pointer generation | Editor59 | 作为overlay provider和同帧hit generation消费者 |
| Transform gizmo、grid/workplane/snap | Editor67 | 只迁入composition/category，不改其数学owner |
| plugin viewport overlay runtime wiring failure | Editor05 failure | 不新增P0；要求回归同一overlay plan |
| highlight runtime frame consumption failure | Editor59 / runtime父owner | 不重复计数；作为selection overlay资格依赖 |
| RHI/render graph、material、environment、post-process、quality | Runtime09A/09C/09F1/09H2/99N | Runtime实现effective policy；Editor不绕过 |
| Editor68新增29项P1、8项P2 | 本报告 | 唯一计数display/profile/composition/persistence/product差距 |

## 12. 最终判定

当前Scene Viewport显示链具有真实命令、render packet、wire/lighting/skybox pass和诊断DTO基础，但产品架构仍停留在三值枚举、两个布尔开关、固定overlay顺序和全局瞬态状态。最严重的问题不是“模式数量少”，而是没有一个能把请求、capability、feature、environment、overlay、diagnostic和per-view persistence解析为同代有效事实的authority；因此功能扩张会继续制造互相矛盾的开关和静默降级。

实施必须从ED68-M0/M1的真实性、RED和single resolver开始，再做per-view identity/persistence、wire hard cutover、lighting/environment、show flag/overlay、debug provider与diagnostics。禁止先堆更多toolbar按钮、继续扩张`DisplayMode`、在Editor写专用shader、保留旧bool兼容桥，或把Runtime Diagnostics全局last值直接贴到任意Scene pane。

本报告完成current-source review，不代表实现完成。29项P1、8项P2和48个资格门保持Open/Fail，直到代码、产品、动态验证和同语义性能证据逐项关闭。
