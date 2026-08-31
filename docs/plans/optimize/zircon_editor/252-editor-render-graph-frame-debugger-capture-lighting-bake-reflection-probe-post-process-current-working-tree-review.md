---
title: Editor Render Graph、Frame Debugger、Capture、Lighting Bake、Reflection Probe 与 Post Process 当前工作树工程化差距
category: zircon_editor
report_id: Editor252
review_date: 2026-08-30
baseline_head: working-tree
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/96-editor-render-pipeline-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/144-editor-render-pipeline-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-debug-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/166-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99c-runtime-exposure-color-tonemap-lut-bloom-dof-motion-blur-ssr-output-transfer-terminal-composition-product-integration-current-source-review.md
related_code:
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/editor
  - zircon_plugins/rendering/features
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - zircon_runtime/src/render_graph/dump.rs
  - zircon_runtime/src/graphics/runtime/offline_bake
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/capture/trigger.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/RenderDocPlugin/Source/RenderDocPlugin/Private/RenderDocPluginModule.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/DumpGPUServices/Source/DumpGPUServices/Private/DumpGPUServices.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmassEditor/Private/GPULightmassEditorModule.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.Compiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Debug/RenderGraph.DebugData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Debug/RenderGraphDebugSession.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphViewer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphViewer.SidePanel.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphEditorRemoteDebugSession.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/AdaptiveProbeVolumes.BakePipelineDriver.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentListEditor.cs
  - dev/godot/servers/rendering/rendering_device_graph.h
  - dev/godot/servers/rendering/rendering_device_graph.cpp
  - dev/godot/editor/debugger/editor_visual_profiler.cpp
  - dev/godot/editor/scene/3d/lightmap_gi_editor_plugin.cpp
  - dev/godot/editor/scene/3d/gizmos/reflection_probe_gizmo_plugin.cpp
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_context.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/Fyrox/fyrox-graphics/src/server.rs
  - dev/Fyrox/editor/src/light.rs
  - dev/Fyrox/editor/src/plugins/probe.rs
doc_type: current_working_tree_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor Render Graph、Frame Debugger、Capture、Lighting Bake、Reflection Probe 与 Post Process 当前工作树工程化差距

## 1. 结论

当前 Editor 已经具备真实的底层观测与呈现基础，但 Rendering Workbench 仍然是静态演示面板，不是工程数据产品。Runtime `CapturedFrame` 已携带 `capture_report`、`graph_dump` 和 `frame_profile_json`，`RenderGraphDump` 也能导出 pass、resource、version、queue fallback、culling、topology layer、lifetime 与 transient slot；Retained viewport 能按 generation 轮询 RGBA capture 或 GPU-resident `RenderViewportProduct`。然而 Editor 对 `RenderGraphDump`、`RenderFrameProfile`、`graph_dump`、`frame_profile_json`、graphics debugger request/status 的精确引用均为 **0**，只检查 width/height/RGBA 并把帧交给 surface。

Workbench 三份核心 ZUI 当前共 **629 行、73 个 node、46 条 route**。Render 页面把 `Frame 1234`、`1.84 ms`、`Windows DX12 30 fps GPU 6.24 ms` 写死；Lighting Bake 把 City_Block_A、87 assets、12 volumes、02:30 写死；Post Process 把 Global Stack、Filmic、LUT_CityWarm、EV +2.1 写死。`Compile`、`Bake`、`Apply` 只修改 retained control 与固定 feedback；field edit 只回写 `value/value_text`，没有 document、transaction、job、compiler、artifact 或 Runtime acknowledgement。

本轮也确认了实质进展。Reflection Probe 的旧 API 断链已经修复：Editor command 可序列化 request/placement，并通过 `RenderFramework` 非阻塞 `submit/poll/cancel`；Runtime request 带 schema、source revision、可选 source hash，consume 层能读取并校验 PMREM artifact。这个边界从旧报告的 Open 重判为 Partial。但它仍没有 production caller、Editor provider/toolkit/selection operation、job owner 或原子 scene/asset transaction；两个 artifact registration 函数也只有定义和局部测试，没有产品调用。

Editor252 将 Editor22 的 **5 项父 P0 重判为 4 Open / 1 Partial**，登记 **40 项 P1（32 Open / 8 Partial）、12 项 P2（12 Open）和 30 个资格门（24 Fail / 6 Partial / 0 Pass）**。Runtime166 继续唯一拥有 Render Graph barrier、queue、completion、external binding 与 execution authority 的 P0，本报告不重复计数。本轮只写 review/index/coverage，不修改 Editor、Runtime、plugin、Cargo、ABI 或 ZUI；Tooling 按用户要求排除。

## 2. 审查边界与证据

### 2.1 当前工作树选集

| 范围 | files | lines | bytes | tests | ignored | 结论 |
|---|---:|---:|---:|---:|---:|---|
| Rendering editor packages + manifest | **67** | **1,665** | **54,507** | **3** | **0** | 根包与15个feature Editor包仍以descriptor/capability为主，Probe command是唯一明显业务例外。 |
| Workbench/catalog/routes/tests | **22** | **8,367** | **364,133** | **34** | **1** | layout、binding、selection、field mutation与固定feedback有测试，但没有领域provider。 |
| Viewport capture observation bridge | **36** | **4,282** | **154,227** | **44** | **1** | lazy RenderFramework resolve、submit、GPU product与RGBA poll是真实底座。 |
| Runtime bake/probe/post-process adjacent evidence | **41** | **4,597** | **165,213** | **50** | **2** | graph/capture/profile、typed lightmap/probe contract与post-process runtime存在，Editor产品链未消费。 |
| Editor/adjacent Runtime focused unique union | **297** | **29,981** | **1,125,767** | **252** | **13** | fingerprint `3cc3166578e128b333d441fa098c1070de0c199b0205ede2dde02722ce91f818`。 |
| Unreal/Unity/Godot/Bevy/Fyrox selected references | **31** | **23,821** | **996,610** | n/a | n/a | fingerprint `ecb4b1cc68f326e01ee98a89f4ae4eba4c8358ef04101a51760164f83f6e9fb3`。 |

分组存在逻辑重叠，不应相加为仓库唯一文件数。测试数量只表示静态/单元 marker；本轮没有运行 Editor host、Cargo、真实 GPU capture、RenderDoc、lightmap bake、probe convolution、post-process save/reopen、device-loss、fault、scale、soak 或 benchmark。

### 2.2 纵向检查链

本轮沿 `plugin manifest -> linked editor catalog -> contribution/provider -> Workbench route -> field/command feedback -> viewport RenderFramework bridge -> CapturedFrame/RenderViewportProduct -> graph/profile/debugger contract -> bake/probe/post-process request -> artifact/transaction` 检查。Runtime 编译器和 RHI 结论以 Runtime166/90 为 owner，只读取其 Editor 所需的观察合同。

### 2.3 证据等级

- **E3**：当前工作树 production Rust、ZUI、plugin manifest/catalog、tests 与五引擎选定源码已逐文件读取。
- **E2**：精确搜索证明 Editor 不引用 graph/profile/debugger/lightmap/post-process contract；调用点搜索证明 probe registration 与 offline bake 没有产品 caller。
- **E1**：现有测试能证明 UI route、viewport generation 和局部 runtime contract，不能证明业务成功、设备失败、持久化或产品可达性。
- **E0**：没有同场景、同质量、同硬件、同驱动和冻结 revision 的画质/性能数据，不能声称优于 Unreal/Unity。

## 3. 当前可保留底座

1. `RenderFramework` 的 typed manager resolution、viewport create/resize/destroy、frame extract submit 与 error retention 可作为 Editor rendering service 接入点。
2. `CapturedFrame` 的 generation、capture report、graph dump 与 profile JSON 已把同帧观测载荷放在一个产品对象中；应扩展为 versioned observation envelope，而不是另建第二套统计采集器。
3. `RenderGraphDump` 已有 pass/resource/access/version/lifetime/transient topology 的结构化中间模型；应公开稳定 schema 或序列化 DTO，避免 Editor 解析当前人读文本。
4. `RenderFrameProfile` 和迟到 GPU timing 回填可作为 Frame Debugger timeline 的生产者；Editor 必须展示 Disabled/Pending/Unavailable/Measured/Stale，不得硬编码毫秒数。
5. `RenderViewportProduct` 让 retained host 无 CPU readback 呈现 GPU texture，这是正常 viewport hot path；debug capture 应是旁路观察，不应强迫每帧回读。
6. Editor contribution store、operation/job/document/transaction、asset catalog/toolkit 的通用基础已存在；Rendering package 应注册真实 contribution，而不是再发明平行插件系统。
7. Reflection Probe request/placement JSON、source revision/hash、nonblocking environment capture 和 artifact decoder 是可保留的跨层合同。
8. Lightmap request/output/consume validation、RGBA16F texture conversion以及 Runtime post-process profile/volume/evaluator/pass graph是真实领域底座；缺的是生产者、Editor authoring和事务闭环。

## 4. 当前源码事实与断路

### 4.1 Manifest、catalog 与 provider

1. `plugin.toml` 声明 15 个 optional feature、16 个 Editor module，根插件仍标 `stable`，runtime capability 仍标 `complete`。
2. `minimal_host_contract/optional_features.rs` 仍断言 Rendering 只有 9 个 optional feature，遗漏 volumetric fog、OIT、light cookies、irradiance volumes、planar reflections、subsurface scattering。
3. 根 Editor package 与多数 feature Editor package 只实现 `descriptor()` 和 `.with_capability(...)`，没有 `register_editor_extensions`、asset/toolkit/operation/view/menu/provider contribution。
4. `first_party_editor_catalog` 只有 Navigation 与 Neural feature/provider；Rendering selection 无 linked Editor registration，App 只是转发该 catalog。
5. Workbench Rendering 页面由内建 ZUI 总索引直接挂载，页面可见性与真实 plugin selection、linked provider、backend capability、schema version 或 unavailable reason无关。

### 4.2 Render Pipeline 与 Render Graph

1. `MainPipeline.rp` 只是 `WorkbenchField` 的初始字符串；没有 asset ID、document ID/revision、source hash、target/backend、dirty/savepoint、save/reopen/recovery。
2. `module_field_edit.rs` 只验证 control 拥有 Change/Submit binding，随后修改 control 的 `value` 与 `value_text`；Submit 不产生 domain transaction。
3. Compile command 直接写 `Render graph compiled` 和 `Windows DX12 30 fps GPU 6.24 ms compiled`；没有调用 Runtime graph builder/compiler，也没有 request/receipt/LKG。
4. `RenderGraphDump` 当前文本包含 pass order、id、queue/fallback、culled、executor、dependencies、access/version、resource descriptor/lifetime/allocation/slot/bucket；Workbench 只展示 Frame Start、Lighting、SceneColor 三个 fixture row。
5. capture 只附带 graph dump 文本，Editor 没有兼容 parser、schema version、unknown-field preservation、source mapping、partial/drop/size budget。
6. barrier batch、native queue wait/signal、ownership transfer、GPU completion 与 external binding 仍由 Runtime166 的 P0 裁决；Editor 必须显示真实状态与缺失原因，不能推断或复制 compiler。

### 4.3 Viewport observation 与 Frame Debugger

1. `poll_captured_frame` 校验非零尺寸与 `width*height*4`，但忽略 capture report、graph dump 和 profile JSON。
2. `poll_viewport_product` 校验 GPU product identity，能避免 CPU capture；这是 presentation path，不是 debugger snapshot。
3. 两条 poll 路径共用 `ViewportState.latest_generation`。如果 GPU product 先发布 generation N，随后同代 CPU capture会因共享游标被过滤；反向亦然。presentation cursor 与 observation cursor必须分离。
4. Editor 没有 graph/execution/camera selection、pass/resource双向选择、lifetime grid、culled/async/merged/queue filter、frame history、timeline、search、bookmark或source navigation。
5. Runtime graphics debugger 有 pending/active/last/error状态，但 trait 默认 `request_graphics_debugger_capture` 返回 `Ok(())`，unsupported实现仍可被误判为接受；Editor也没有任何request/status调用。
6. Play preview frame只保留RGBA、size和gateway/session/transport/generation identity，graph/profile/report不会跨动态API进入Game View debugger。

### 4.4 Capture 与 diagnostics

1. 没有 next-frame/delayed/multi-frame/current-view/all-activity 的 typed capture request，也没有 admission、deadline、cancel acknowledgement与device-loss终态。
2. 没有 artifact目录、atomic finalize、tool/backend/adapter/build/source revision、文件大小/retention、open/reveal/export与corruption validation。
3. 没有将 capture report、graph/profile warning 路由到 Notification、Console、Problems或source location。
4. Runtime profile已有pass/subsystem/memory/cache/degrade等信息，但Editor rendering面板固定显示6.24ms，无法区分GPU query pending、disabled、unsupported或stale。
5. 没有observer cost预算、snapshot bytes/entry cap、drop/partial/age/coverage统计，超大graph会直接把调试功能变成新的性能问题。

### 4.5 Lighting Bake 与 Lightmap

1. Runtime已有versioned `LightmapBakeRequest`、scene snapshot、atlas budget、`LightmapBakeOutput`、slot/probe-grid验证与RGBA16F asset转换，这是contract而不是bake producer。
2. `offline_bake_frame`仍只从mesh数量和directional light intensity派生少量ReflectionProbeData；production caller为0，只有测试调用，不产出lightmap atlas、UV、progress或artifact。
3. `texture_asset_from_lightmap_bake_output`只在定义/单元测试/render product fixture使用；没有Editor bake job、staging store、atomic publish或scene transaction。
4. optional baked-lighting feature注册默认pass，但executor仍是`noop_render_executor`；Workbench却显示87 assets/Ready/02:30。
5. Bake/Preview按钮只写固定queued文本；没有immutable scene/settings/backend key、preflight、worker、stage progress、cancel/deadline、stale source reject与shutdown barrier。
6. 没有BakingSet/scenario/streaming-cell owner、UV/texel-density/overlap/coverage overlay、incremental dirty、cache/determinism、apply/clear/save/reopen/rollback。

### 4.6 Reflection Probe

1. `ReflectionProbeCaptureEditorCommand`能保留request/placement序列化，trigger通过`RenderFramework`提交、轮询和取消；旧的mutable SceneRenderer与已删除方法断链已经消失。
2. request有schema version、position、clip、face size、quality、source revision和可选source hash；placement有shape、blend、box projection、intensity、priority、layer与bake timing。
3. Runtime consume能验证runtime-cache/asset-derived artifact并构造texture/probe，但`register_captured_reflection_probe*`没有production caller。
4. `ReflectionProbeCaptureEditorTrigger::new`没有caller；Rendering editor plugin也未把它注册为operation/provider，因此用户无法从Lighting Bake页到达这条真实底座。
5. 没有选中probe的typed component drawer、scene gizmo/interaction mode、capture operation、EditorJob、progress/cancel UI、document/world generation或undoable asset/scene commit。
6. Workbench的Reflection Probe Grid、12 volumes、Adaptive、Ready仍是固定展示，与request/status/artifact无绑定。

### 4.7 Post Process

1. Runtime已有typed settings/profile/volume、scene component、volume evaluator、pass graph与真实built-in effect execution；Editor对`PostProcessSettingsComponent`和`PostProcessVolumeComponent`引用均为0。
2. optional rendering feature仍注册一个`post-process` legacy pass并使用no-op executor，容易与Runtime built-in post-process形成“capability已启用但feature executor无行为”的双重事实。
3. Editor没有Profile/Volume asset toolkit、serialized component list、override checkbox、add/remove/reorder、multi-edit、Undo、save/reopen或external conflict。
4. Apply只写固定文本；没有global/camera/local volume、blend distance/weight/priority/layer、effective stack、field provenance、preview generation或Runtime acknowledgement。
5. 没有before/after、effect isolate、histogram/exposure/color/LUT/bloom/DoF/motion blur/SSR typed validator和per-viewport debug-mode lease。

## 5. 参考引擎差异与采用边界

| 参考 | 已检查的工程事实 | Zircon 应采用的边界 |
|---|---|---|
| Unreal RDG / RenderDoc / DumpGPU | RDG builder把pass/resource/external extraction与execute放在同一图合同；RenderDoc有viewport/activity、延迟/多帧、通知与provider；DumpGPU有artifact服务。 | Runtime生成唯一graph/capture事实，Editor只编排session、读取artifact和投影诊断；backend API不得泄漏到Workbench。 |
| Unreal GPULightmass | Editor module有真实settings、Start、Bake What You See、Save And Stop、Cancel、progress/status和world lifecycle。 | Bake必须是长事务与artifact publication，不是按钮后固定queued字符串。 |
| Unity RenderGraph DebugData / Viewer | DebugData记录graph hash、pass type/read/write/culled/async/sync、resource producer/consumer/lifetime/descriptor；Viewer支持local/remote session、execution/camera等待、schema incompatible、filter、selection、resource lifetime grid和持久化view options。 | 以versioned observation snapshot和local/remote session作为Frame Debugger UX下限，并增加Zircon owner generation、bounded stream和fail-closed disconnect。 |
| Unity APV / Volume Editor | Bake pipeline有stage/progress/in-progress；BakingSet与Volume component list有serialized source、override/add/remove/reorder与Editor transaction。 | 复用Zircon document/job/transaction，不复制Unity资产模型；所有runtime resolved值需显示provenance。 |
| Godot RenderingDeviceGraph / Visual Profiler / LightmapGI | graph有command/resource/barrier语义；visual profiler保留有界frame metric与CPU/GPU area；LightmapGI与probe gizmo可从Editor到达。 | 作为轻量产品下限；Zircon仍需更强的remote、generation、artifact、device-loss与stale contract。 |
| Bevy Render schedule / diagnostics | RenderGraph schedule明确Begin/Render/Submit/Finish，screenshot/readback/present在root render system完成，diagnostic与renderer解耦。 | 借鉴生产者/consumer分离和显式schedule；Bevy没有同级完整Editor，不能为Zircon产品缺失背书。 |
| Fyrox graphics/lightmap/probe Editor | GraphicsServer有统一frame/stats接口；Editor lightmap有input、thread、cancel、texture save、apply/clear；probe有preview panel、interaction mode和property command。 | Rust实现可参考，但blocking/thread细节只是最低线；Zircon目标应是有预算、可取消、generation-qualified、原子提交的job。 |

参考代码用于验证工程合同，不意味着复制其所有权模型。要宣称表现或性能优于 Unreal，必须完成同输入、同输出质量、同硬件、同驱动、同warm-up、同capture overhead的可重复对拍。

## 6. Owner 边界与父 P0 当前重判

| ID | 状态 | 当前证据与硬切要求 |
|---|---|---|
| RENED-P0-01 | Open | `stable/complete`、15项manifest、9项host status、零linked Rendering provider仍是四套事实；统一generated catalog、link set、readiness与Unavailable reason。 |
| RENED-P0-02 | Open | Workbench仍以Frame1234/1.84ms/6.24ms等fixture作为第二authority；删除业务fixture，只投影同generation Runtime snapshot/receipt。 |
| RENED-P0-03 | Open | Lighting Bake仍宣称queued/ready但没有lightmap producer/job/artifact/atomic commit，baked-lighting optional executor仍no-op。 |
| RENED-P0-04 | Open | Post Process Apply仍只改control文本，Editor未消费typed Runtime profile/volume/evaluator，optional executor仍no-op。 |
| RENED-P0-05 | Partial | Probe旧API断链已修复，request/poll/cancel与artifact validator存在；但trigger/registration无产品caller、provider、job与原子scene/asset commit。 |

Owner 划分如下：Runtime166/90唯一拥有graph compiler、RHI state/barrier/queue/completion、capture/profile schema；Runtime96/97/99c拥有probe/bake/post-process算法与artifact；Plugin04拥有manifest/runtime-editor package配对和provider生命周期；Editor02/09/50/63拥有document/job/contribution/transaction；Editor252只拥有Rendering source/toolkit、operation controller、debug reader和projection。

## 7. P1 差距与重构要求（40项）

### 7.1 装配、truth 与生命周期

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ERG5-P1-001 | Open | 15项manifest与9项host status漂移；由同一declaration生成manifest/catalog/status/tests/UI capability。 |
| ERG5-P1-002 | Open | 根包与15个feature包多数只有descriptor；每个可见能力必须有typed contribution/provider或明确Unavailable。 |
| ERG5-P1-003 | Open | first-party catalog不链接Rendering；按project selection原子装配runtime/editor/resources/provider。 |
| ERG5-P1-004 | Partial | 通用contribution store、operation/job/document基础存在；Rendering必须接入owner generation、reload revoke与shutdown drain。 |
| ERG5-P1-005 | Partial | ZUI、binding、selection、field mutation及UI测试真实存在；必须删除control-local业务authority，改投影provider snapshot。 |
| ERG5-P1-006 | Open | 页面可见性不受selection/backend/schema/readiness控制；menu/workspace/command统一消费capability snapshot。 |

### 7.2 Pipeline、Graph 与 Frame Debugger

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ERG5-P1-007 | Open | `MainPipeline.rp`无source document；建立versioned schema、stable pass/resource ID、revision、save/reopen/recovery。 |
| ERG5-P1-008 | Open | Compile固定成功；提交typed compile request，返回diagnostic/artifact generation并保留LKG。 |
| ERG5-P1-009 | Partial | Runtime capture已携带report/graph/profile；Editor建立同帧observation envelope consumer，禁止只取RGBA。 |
| ERG5-P1-010 | Partial | Runtime有typed `RenderGraphDump`但capture只暴露人读文本；发布versioned DTO/serializer并保留未知字段。 |
| ERG5-P1-011 | Open | 三个fixture row不是graph view；实现virtualized pass tree/resource grid与bounded projection。 |
| ERG5-P1-012 | Open | 缺pass-resource双向选择、producer/consumer、read/write/version、first/last use、alias/acquire/discard/source link。 |
| ERG5-P1-013 | Open | 缺culled/async/fallback/parallel/topology/queue filter及真实barrier/wait/signal显示；缺失字段须标Unavailable。 |
| ERG5-P1-014 | Open | 缺graph hash、source/artifact/frame/view/capture generation与backend/build provenance。 |
| ERG5-P1-015 | Open | GPU product和CPU capture共用`latest_generation`游标会互相吞同代结果；拆presentation/observation cursor。 |
| ERG5-P1-016 | Open | 缺bounded frame history、cursor、search/filter/sort/bookmark、retention与stale/incompatible reset。 |
| ERG5-P1-017 | Open | 缺local/remote session、graph/execution/view选择、disconnect/reconnect和schema negotiation。 |
| ERG5-P1-018 | Open | 固定1.84/6.24ms掩盖timing状态；投影Pending/Measured/Disabled/Unavailable/Stale和回填generation。 |

### 7.3 Capture、Profiler 与 diagnostics

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ERG5-P1-019 | Partial | Runtime debugger request/status有真实状态机；默认unsupported request仍`Ok(())`，需fail-close capability/receipt。 |
| ERG5-P1-020 | Open | Editor没有debugger request/status caller；建立current-view/all-activity/next/delay/multiframe command与session。 |
| ERG5-P1-021 | Open | capture缺project/world/view/pipeline/source/artifact owner identity、deadline、sequence、adapter/backend/tool版本。 |
| ERG5-P1-022 | Open | 缺queued/running/succeeded/failed/canceled/stale/device-lost终态与late callback拒绝。 |
| ERG5-P1-023 | Open | 缺artifact staging/finalize/retention/size/open/reveal/export/corruption validation。 |
| ERG5-P1-024 | Open | 缺CPU/GPU/pass/subsystem/memory/cache/degrade统一timeline与Notification/Console/source routing。 |

### 7.4 Lighting Bake 与 Lightmap

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ERG5-P1-025 | Partial | typed request/output/consume和texture转换存在；建立真实bake producer与backend capability。 |
| ERG5-P1-026 | Open | Bake按钮不提交EditorJob；增加immutable input、preflight、admission、stage progress、cancel/deadline/shutdown。 |
| ERG5-P1-027 | Open | `offline_bake_frame`只产probe且无production caller；禁止将其当lightmap实现或成功fallback。 |
| ERG5-P1-028 | Open | baked-lighting optional executor为no-op；未有真实executor前capability必须Unavailable且UI不可宣称Ready。 |
| ERG5-P1-029 | Open | 缺atlas/UV/material/light/probe-grid artifact staging、validation、atomic publish、rollback、LKG。 |
| ERG5-P1-030 | Open | 缺BakingSet/scenario/streaming-cell、incremental dirty/cache/determinism和texel/overlap/coverage overlay。 |

### 7.5 Reflection Probe 与 Post Process

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ERG5-P1-031 | Partial | probe request/placement与nonblocking trigger真实存在；注册operation/provider并绑定Document/World/View generation。 |
| ERG5-P1-032 | Open | trigger和artifact registration无产品caller；接selection/component drawer/gizmo/capture command与EditorJob。 |
| ERG5-P1-033 | Open | poll/cancel未形成progress、deadline、cancel ack、stale reject、reload/shutdown状态机。 |
| ERG5-P1-034 | Open | PMREM注册与scene probe更新非原子；使用staging asset + transaction + rollback + reference migration。 |
| ERG5-P1-035 | Partial | Runtime typed post-process与built-in execution存在；Editor必须直接消费schema/effective result，禁止复制validator。 |
| ERG5-P1-036 | Open | 建立Profile/Volume toolkit、serialized component list、override/add/remove/reorder/multi-edit/Undo/save。 |
| ERG5-P1-037 | Open | 建立global/camera/local volume、blend/layer/priority、effective-stack与field provenance inspector。 |
| ERG5-P1-038 | Open | Preview/Apply必须有isolated session、before-after/effect isolate、requested/installed/drawn generation与ack。 |

### 7.6 生命周期、规模与资格

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| ERG5-P1-039 | Open | graph/capture/bake/probe/post-process缺plugin reload、world replace、document close、device loss、stale callback fault矩阵。 |
| ERG5-P1-040 | Open | 缺10K pass/100K resource、长capture、100K bake object、multi-volume/probe、RSS/latency/observer-cost基线与visual golden。 |

## 8. P2 高阶能力（12项）

| ID | 状态 | 目标 |
|---|---|---|
| ERG5-P2-001 | Open | 多主机、多adapter、多viewport远程debug session与权限隔离。 |
| ERG5-P2-002 | Open | 多人共享capture、annotation、bookmark与source revision锁定。 |
| ERG5-P2-003 | Open | graph结构、timing、memory和artifact regression diff。 |
| ERG5-P2-004 | Open | pass/resource breakpoint、conditional capture与自动触发规则。 |
| ERG5-P2-005 | Open | 超大graph paging、LOD、search index和progressive load。 |
| ERG5-P2-006 | Open | 跨backend capture artifact转换、compatibility matrix与offline viewer。 |
| ERG5-P2-007 | Open | distributed bake、worker capability/lease、checkpoint/resume与merge。 |
| ERG5-P2-008 | Open | large-world cell/scenario增量bake与streaming budget。 |
| ERG5-P2-009 | Open | probe自动布局、质量heuristic、overlap优化和平台budget建议。 |
| ERG5-P2-010 | Open | Post Process profile inheritance/variant/batch diff/migration。 |
| ERG5-P2-011 | Open | scriptable capture/bake/validation command与稳定machine-readable result。 |
| ERG5-P2-012 | Open | 长期trend、regression threshold、automatic bisect input与reproducible evidence pack。 |

## 9. 目标架构

```text
RenderingPluginDeclaration
  -> LinkedRuntimeProviderSet + LinkedEditorProviderSet + ResourceBundleSet
  -> RenderingCapabilitySnapshot(owner_generation, schema_versions, unavailable_reasons)

RenderPipelineDocument
  -> Runtime compiler request(source_revision, target, features, backend)
  -> immutable CompiledRenderPipelineArtifact + diagnostics + last-known-good

FrameObservationRequest(view, frame policy, budget, deadline)
  -> CapturedFrameObservation(report, graph DTO, frame profile, provenance)
  -> bounded local/remote DebugSession
  -> virtualized pass/resource/lifetime/timeline projection

BakeOrProbeRequest(document/world revision, immutable inputs, artifact key)
  -> admission -> queued -> running(stage/progress)
  -> succeeded(staged artifact) | failed | canceled | stale | device-lost
  -> validation -> atomic asset + scene transaction -> terminal receipt

PostProcessDocument/Profile/Volume
  -> Runtime-owned schema/validator/evaluator
  -> PreviewSession(requested/compiled/installed/drawn generation)
  -> effective-stack/provenance projection
```

UI control、document、job、artifact、observation snapshot与runtime installed state必须各有单一authority。所有异步结果都绑定project/session/world/view/document/source/artifact/plugin/device generation；迟到结果不得覆盖新状态。

## 10. 重构里程碑

| 阶段 | Owner | 交付 | 退出条件 |
|---|---|---|---|
| M252.0 Truth hard cut | Plugin04 + App + Editor50 | 15项单一declaration、linked provider readiness、Unavailable；删除固定成功事实 | G01-G03 |
| M252.1 Observation schema | Runtime166/90 | versioned graph/profile/capture envelope、独立cursor、fail-closed debugger receipt | G04-G10 |
| M252.2 Debug product | Editor252 | local/remote session、graph/resource/lifetime/timeline、artifact browser、diagnostic routing | G11-G16 |
| M252.3 Source documents | Editor02/63 + Rendering | Pipeline/PostProcess/BakingSet/Probe document、transaction、save/reopen/recovery | G17/G23/G27 |
| M252.4 Operations/jobs | Editor09/47/50 | compile/capture/bake/probe/preview operation、admission、progress/cancel/deadline/stale/shutdown | G18-G26 |
| M252.5 Authoring | Rendering Editor | probe drawer/gizmo、bake overlays/scenario、volume component/effective stack | G24-G28 |
| M252.6 Qualification | Runtime + Editor + App | fault/scale/soak/visual golden、backend matrix、same-quality benchmark | G29-G30 |

顺序不能倒置。先让provider和Runtime事实可达，再做复杂graph canvas；否则只会扩大fixture UI和第二authority。

## 11. G01-G30 资格门

| Gate | 状态 | 通过条件 |
|---|---|---|
| G01 | Fail | 15项manifest与generated catalog/status/tests完全一致。 |
| G02 | Fail | 每个可见Rendering能力有linked provider、owner generation、readiness和Unavailable reason。 |
| G03 | Fail | Workbench不再包含Frame1234/1.84ms/6.24ms/87 assets等业务fixture authority。 |
| G04 | Partial | Runtime graph dump存在；须改为versioned DTO并覆盖schema兼容。 |
| G05 | Fail | Pipeline create/open/edit/save/reopen/recover/compile走真实document与artifact。 |
| G06 | Fail | pass/resource/access/version/lifetime/alias/queue视图与同帧Runtime事实一致。 |
| G07 | Fail | barrier/wait/signal/completion缺失时明确Unavailable，不伪造或推断。 |
| G08 | Fail | GPU timing正确显示Pending/Measured/Disabled/Unavailable/Stale。 |
| G09 | Fail | local/remote session支持disconnect、reconnect、schema mismatch和stale reset。 |
| G10 | Partial | viewport capture/product可poll；须拆presentation与observation generation cursor。 |
| G11 | Fail | graph/profile/report全部被Editor消费且绑定同一view/frame/capture provenance。 |
| G12 | Fail | 10K pass/100K resource在预算内virtualize/filter/select，无无界分配。 |
| G13 | Fail | frame history、timeline、search、bookmark、retention和observer cost可测。 |
| G14 | Fail | capture artifact有typed request、终态、atomic finalize、retention和open/reveal。 |
| G15 | Partial | Runtime debugger状态机存在；unsupported request fail-close且Editor有真实caller。 |
| G16 | Fail | device loss/timeout/cancel/reload/shutdown不遗留capture session或误报成功。 |
| G17 | Fail | BakingSet与immutable request可save/reopen，并绑定scene/settings/backend revision。 |
| G18 | Fail | Lightmap producer产出validated atlas/slots/probe grid，不以probe-only fallback冒充。 |
| G19 | Fail | Bake job有preflight、progress、cancel ack、deadline、stale reject和atomic publish。 |
| G20 | Partial | typed lightmap request/output存在；须有production producer/caller和Editor operation。 |
| G21 | Fail | UV/texel/overlap/coverage、incremental/cache/determinism与large-world场景通过。 |
| G22 | Fail | no-op baked-lighting feature不可报告Ready/complete。 |
| G23 | Fail | Reflection Probe component/toolkit/gizmo/selection/capture workflow可从产品到达。 |
| G24 | Partial | probe nonblocking trigger存在；须接job/progress/stale/transaction/artifact caller。 |
| G25 | Fail | PMREM asset与scene probe更新原子提交，失败/撤销/重载可回滚。 |
| G26 | Fail | probe capture在cancel/device-loss/source mutation下给出唯一terminal receipt。 |
| G27 | Partial | Runtime post-process真实存在；Editor须消费typed schema/effective result。 |
| G28 | Fail | Profile/Volume override/add/remove/reorder/Undo/save/preview/apply全链闭合。 |
| G29 | Fail | plugin/world/document/device fault矩阵、scale/soak/RSS/latency基线通过。 |
| G30 | Fail | same-quality visual golden与跨引擎benchmark可复现，才允许领先性声明。 |

## 12. 验证边界与裁决

本轮只做静态/current working-tree review。报告确认的进展是 Runtime observation payload、typed graph dump、GPU product presentation、typed lightmap contract、真实 post-process runtime以及非阻塞 probe capture bridge；确认的产品断路是 Rendering provider未链接、Workbench fixture authority、Editor不消费graph/profile/debugger、共享generation cursor、无lightmap producer、no-op feature executor、probe无caller和post-process无authoring。

实施前必须重新冻结这些owner路径，因为工作树仍在变化。任何里程碑都不能仅凭descriptor、route、固定feedback、单元DTO或no-op executor关闭；至少需要真实provider reachability、source-to-artifact-to-installed/drawn receipt、故障终态和产品E2E证据。
