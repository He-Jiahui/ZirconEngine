---
related_code:
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/editor
  - zircon_plugins/rendering/features
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/types.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/document_module.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_inspector_property_edit.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/text_and_module_input.rs
  - zircon_editor/src/tests/workbench/reference_surface.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Developer/RenderDocPlugin/Source/RenderDocPlugin/Private/RenderDocPluginModule.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/RenderDocPlugin/Source/RenderDocPlugin/Private/SRenderDocPluginEditorExtension.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmassEditor/Private/GPULightmassEditorModule.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/Toolsets/PerfToolset/Source/PerfToolset/Private/GpuProfilerToolset.cpp
  - dev/godot/editor/scene/3d/lightmap_gi_editor_plugin.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeProfileEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 22 · Render Pipeline / Frame Capture / Lighting Bake / Reflection Probe / Post Process / Debug Authoring 工程化差距

## 1. 结论

Zircon并非没有渲染运行时基础。Runtime已经定义Render Graph materialization/execution/resource/alias/coverage/profile/stage report、frame profile、GPU timing status、per-pass timing、viewport capture与反射探针capture/register helper。这些结构应当保留并成为Editor产品数据源，不能因为当前Workbench没有接线就重复实现另一套统计模型。

真正缺失的是Editor产品层。`rendering/plugin.toml`把根Rendering和15个optional feature都声明为Editor模块，根capability又标为`complete`、插件maturity标为`stable`；但根Editor crate与15个feature Editor crate总共只注册16个capability，没有asset、surface、operation、menu、graph、viewport或authoring contribution。`first_party_editor_catalog`只装配Navigation和Neural，完全排除Rendering，而first-party runtime catalog却装配Rendering。于是manifest承诺的Editor模块既不进入默认产品，也没有可执行贡献。

Workbench同时制造了第二层“完成”外观。Render workspace固定显示`Frame 1234`、`SceneColor -> BloomInput 1.84 ms`、`R11G11B10_FLOAT Read`、`Windows DX12 30 fps GPU 6.24 ms`和`MainPipeline.rp`；Compile、Preview、Save等action只写固定feedback。Lighting Bake固定显示87 assets、4 warnings、12 volumes、6 texels、02:30 estimate，Bake/Preview同样只写字符串。Post Process固定显示Global Stack、Cinematic、Bloom 0.65、Filmic +0.4、LUT_CityWarm和EV +2.1，Apply/Preview不修改资产、scene、runtime或preview generation。字段编辑仍只改retained control的`value/value_text`。

反射探针Editor helper是本轮必须保护的例外：`ReflectionProbeCaptureEditorTrigger`会调用runtime capture并可注册产物，且有request/placement roundtrip测试。问题不是它“完全虚假”，而是它没有产品caller、operation factory、job、transaction、scene selection、diagnostic或catalog owner，用户无法从Editor可靠到达它。Rendering其余15个Editor descriptor连这种薄适配都没有。

参考实现显示工程级渲染工具必须读取真实运行时事实。Unreal RenderDoc extension按D3D11/D3D12/Vulkan取得设备、在正确帧边界begin/end capture、等待GPU并启动外部捕获；GPU Lightmass Editor检查world subsystem、RHI、硬件、项目设置和插件能力后才允许Start/Stop。Godot LightmapGI Editor执行真实bake、报告UV/mesh/权限/atlas错误并支持进度取消。Unity Rendering Debugger从render pipeline注册panel/widget、校验pipeline状态并刷新实际数据；VolumeProfileEditor编辑真实serialized profile且处理不兼容component。Zircon可以有自己的UI，但不能以静态样例代替这些authority、generation与failure contract。

本轮登记5项P0、60项P1、12项P2。M0先撤销`stable/complete`与静态成功语义，补齐catalog/provider；M1建立generation-qualified render diagnostics bridge；M2完成Render Graph inspector与resource lifetime；M3完成capture/profiler；M4完成lighting bake与probe产品链；M5完成post-process profile/volume authoring；M6收敛跨平台、job、diagnostic和故障恢复；M7完成规模/性能资格；M8删除静态第二authority。Shader/Material/VFX authoring由Editor15拥有，本篇只消费其artifact与diagnostic；runtime执行语义分别由09A、09C、09F1、09F2和09H2拥有。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes | 证据等级 |
|---|---:|---:|---|
| Rendering根Editor、15个feature Editor与plugin manifest | 67 / 1,676 / 54,434 | 3 | E3：全部descriptor、manifest、capture helper和测试逐文件 |
| Render、Lighting Bake、Post Process与generated bottom surfaces | 4 / 1,223 / 78,047 | 0 | E3：所有可见字段、表格、按钮、固定值和route逐control |
| route、binding、field edit、feedback与bottom panel | 19 / 6,515 / 285,195 | 2 | E3：从event到最终UI mutation逐分支追踪 |
| focused Workbench tests | 5 / 2,373 / 85,949 | 24 | E3静态阅读：模块选择、字段输入、固定feedback与投影 |
| first-party Editor catalog | 4 / 239 / 8,423 | 6 | E3完整分支：默认只装配Navigation和Neural，无Rendering |
| selected combined scope | 99 / 12,026 / 512,048 | 35 | 当前工作树fingerprint `2db19ed87c23b3a0c96ad8c1ad9cb111d8e7b6470eed81ca50c23a9b9aa09fc0`；0 ignored，2个纯import排序在途source |

行数为物理文本行；fingerprint按相对路径排序，对每个当前工作树文件计算SHA-256，再对`path<TAB>hash<LF>`清单计算SHA-256。`zircon_editor/src/ui/retained_host/workbench_preview_actions.rs`与`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs`存在非本轮产生的纯import排序修改，本轮保持原样。实施前必须重算fingerprint并复核route inventory；仓库其余用户与Session修改不属于本篇，不吸收、不回退。

16个Editor descriptor的结构性扫描结果为：16处`with_capability`，0处`with_asset`、`with_surface`、`with_operation`、`with_menu`、`with_graph`、`with_viewport`，0个`OperationCommandFactory`、0个`EditorAuthoringContributionBatch`、0个产品`plugin_registration`。Reflection Probe的capture helper不通过descriptor暴露，因此不能抵消上述产品装配缺口。

### 2.2 动态证据边界

本轮没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误、122个warning阻断；本轮没有重复无源码修复且不能到达Rendering产品域的同一lane。selected scope中的35个test attributes仅作为静态inventory，其中Reflection Probe测试证明序列化适配可运行，Workbench测试主要证明固定字符串和selected状态会被写入control，不能证明真实graph、capture、bake、profile、volume或GPU数据成立。

### 2.3 参考边界

- Unreal RenderDoc插件负责backend device、capture boundary、GPU idle、capture path、viewport command与外部工具启动；Zircon必须把backend capability、frame token和capture artifact做成typed contract，不能让一个“Capture”按钮无条件成功。
- Unreal GPU Lightmass Editor由world subsystem拥有build state，并给出hardware/RHI/project/plugin不可用原因；Godot LightmapGI Editor进一步覆盖save path、UV、mesh、atlas、权限、progress与cancel。Zircon的Bake必须同样以世界revision和build manifest为输入，以可验证artifact为输出。
- Unity Rendering Debugger消费当前render pipeline注册的panel/widget，处理pipeline切换、搜索、reset和刷新；Zircon的Frame Debugger必须订阅Runtime09A报告，而非另存一份静态graph。
- Unity VolumeProfileEditor对真实serialized profile做component增删、兼容性判断与undo；Zircon的Post Process必须由09H2的typed registry/schema驱动，并通过Editor02 transaction保存。
- Fyrox、Bevy可用于检查插件组合、诊断与资源生命周期，但本地源码中没有比上述三条更完整的同级RenderDoc/Lightmass/Volume产品闭环；它们不构成降低目标的理由。

## 3. 必须保留的基础

1. 保留Runtime09A现有Render Graph execution/materialization/resource/alias/coverage/profile/stage report，补version与generation，不在Editor复制graph truth。
2. 保留frame profile、GPU timing status、per-pass timing与viewport capture基础，将其接入有界snapshot/stream provider。
3. 保留Reflection Probe capture/register helper及roundtrip测试，把它包装成真实operation/job，而不是删除后重写。
4. 保留Rendering插件按feature拆分的package意图，但descriptor必须按能力注册真实贡献，manifest状态必须来自qualification而非手填。
5. 保留Workbench稳定control/action identity和纯presentation导航，逐个把固定业务handler替换为provider投影。
6. 保留generated bottom panel布局入口，数据源改为compile、barrier、warning、error和job journal的typed view model。
7. 保留字段Change/Submit事件区分，但只允许accepted document revision回写control；非法输入必须保留用户文本并显示typed diagnostic。
8. 复用Editor02 transaction/save/recovery、Editor09 job、Editor11 diagnostic journal、Editor15 shader artifact，不建Rendering私有简化栈。

## 4. 目标架构

```text
ProjectPluginManifest(rendering + feature selections)
  -> first_party_editor_catalog
  -> RenderingEditorRegistration
       -> surface / menu / operation / settings / asset toolkit providers
       -> backend capability + feature availability projection

Runtime Render Authority
  -> RenderGraphSnapshot(graph_generation, frame_token)
  -> ResourceLifetimeSnapshot + Barrier/Queue/Alias events
  -> FrameProfile(cpu/gpu/timing_generation/confidence)
  -> CaptureService(backend, viewport, range, artifact)
  -> LightingBuildService(world_revision, settings_revision, artifact_generation)
  -> PostProcessRegistry(profile/volume/schema/runtime generation)

Editor Product Layer
  -> transactional documents + undo/save/recovery
  -> cancellable jobs + progress + diagnostics + LKG
  -> reader-gated bounded streams / immutable snapshots
  -> Workbench, viewport overlays, inspectors and generated bottom projections
```

任何状态至少携带`project_id`、`world_id`、`source_revision`、`artifact_generation`、`runtime_generation`或`frame_token`中适用字段。UI不得把不同generation的graph、resource、timing、capture或bake结果拼成一帧“看似一致”的数据。

## 5. P0：必须先修正的产品真实性与数据安全问题

| ID | 差距 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-01 | Rendering Editor未进入默认产品 | runtime catalog装配Rendering，Editor catalog只装配Navigation/Neural；manifest却声明根与15个feature Editor module | 建立manifest-driven first-party Editor registration；缺失binary/contribution时显示typed unavailable并阻止maturity/complete发布 |
| P0-02 | Render Pipeline/Frame视图是静态第二authority | Frame 1234、1.84 ms、6.24 ms、DX12、资源格式/访问与compile成功均硬编码在ZUI/feedback | 从Runtime09A snapshot/profile/capture authority投影；无provider时清空样例并显示原因，不得伪造成功 |
| P0-03 | Lighting Bake向用户承诺不存在的build | 87 lightmaps、12 volumes、02:30与queued结果固定；Runtime09F2确认没有合格baker/artifact产品链 | 在真实baker、world revision、job、cancel、artifact validation完成前硬切Unavailable；禁止写“queued/ready” |
| P0-04 | Post Process Apply不修改任何权威状态 | Global Stack/Cinematic/Filmic/LUT/EV值固定，Apply/Preview只写feedback；字段只改control字符串 | 以09H2 typed registry与transactional profile/volume document驱动，Apply必须返回accepted revision/runtime generation或失败 |
| P0-05 | 16个Editor descriptor只有capability，Reflection helper不可达 | capability scan为16，其余贡献API为0；Reflection trigger只有本crate/tests caller | 每个enabled feature注册最小真实surface/operation/settings/diagnostic；capture通过operation factory、job、selection和catalog到达，未实现feature不得声称complete |

## 6. P1：工程级功能与架构差距

### 6.1 装配、能力与产品状态（P1-01 至 P1-08）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-01 | manifest module、Cargo feature、catalog registration和运行时selection没有同一验证器 | 生成并校验四者映射，启动时输出missing/duplicate/incompatible contribution诊断 |
| P1-02 | 根Rendering与feature capability不能表达backend、platform、driver、project policy和runtime availability | 引入reason-coded capability snapshot与generation，UI只消费快照 |
| P1-03 | `stable`与`complete`是静态元数据，不受产品资格控制 | maturity/status由可重复qualification artifact生成，缺P0 gate自动降级 |
| P1-04 | 15个feature Editor crate重复薄descriptor且无共享注册协议 | 定义RenderingFeatureEditorContribution，明确surface/settings/operation/debug hooks与依赖 |
| P1-05 | feature启停不描述restart、shader rebuild、scene invalidation或artifact失效 | 建立enablement transaction与impact plan，失败可回滚/恢复 |
| P1-06 | Editor看不到runtime选中的render backend、adapter、feature level与fallback原因 | 提供只读RenderEnvironmentSnapshot并写入diagnostic/export |
| P1-07 | package不在根workspace且产品catalog无依赖，常规CI无法证明它们编译 | 由Plugin01决定workspace/独立package策略，并加入唯一可复现验证入口 |
| P1-08 | 未实现feature仍可通过manifest被发现为Editor module | discovery返回Available/Unavailable/Degraded与原因，菜单和surface遵循同一状态 |

### 6.2 Render Graph与pipeline inspector（P1-09 至 P1-22）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-09 | Workbench固定`MainPipeline.rp`，没有当前world/view/pipeline owner | 选择器绑定真实viewport、view family、pipeline asset与generation |
| P1-10 | graph节点/边/queue来自静态行而非Runtime09A materialization report | 提供immutable RenderGraphSnapshot，包含pass/resource/dependency/stage ID |
| P1-11 | 没有source graph与materialized graph对照 | 支持声明、裁剪、合并、alias后的差异视图并标出原因 |
| P1-12 | pass状态缺enablement、culled、merged、fallback与failure reason | 采用typed pass disposition，不用颜色/文本推断 |
| P1-13 | resource表没有创建/首次使用/最后使用/alias owner/lifetime range | 投影ResourceLifetimeSnapshot并支持timeline定位 |
| P1-14 | access只显示固定`Read`，没有stage/access/layout/queue ownership | 显示typed barrier state与producer/consumer边 |
| P1-15 | 没有cross-queue semaphore/fence、ownership transfer和async overlap | 建立queue timeline、wait/signal链与critical path |
| P1-16 | transient alias没有物理heap、offset、size、compatibility与峰值 | 接Runtime09A alias report，提供bytes与fragmentation视图 |
| P1-17 | graph compile反馈不含source/artifact generation | compile operation返回pipeline artifact ID、input digest、diagnostic set与LKG |
| P1-18 | compile失败没有定位pass/resource/shader/include/source span | 统一Editor15 diagnostic schema并提供跳转 |
| P1-19 | graph snapshot没有frame fence，暂停后仍可能与实时资源状态混合 | freeze生成captured frame token，所有pane按token读取 |
| P1-20 | 无搜索、filter、dependency isolate、resource usages与bookmark | 提供可扩展query模型，不在UI扫描完整graph |
| P1-21 | generated bottom五类row只切换固定label | 由compile/barrier/warning/error/log provider投影，带计数、severity、generation和navigation target |
| P1-22 | 没有headless dump与机器可比对格式 | 输出versioned JSON/trace artifact，支持CI diff与bug report附件 |

### 6.3 Frame capture、GPU profiler与资源检查（P1-23 至 P1-34）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-23 | `Frame 1234 captured`不对应真实capture token | CaptureService返回Pending/Captured/Failed、frame range、backend和artifact URI |
| P1-24 | capture没有viewport/window/process/world选择 | 通过Editor07 play/viewport session选择器解析稳定target，禁止隐式抓错进程 |
| P1-25 | capture没有backend capability或Null/headless检查 | 每个backend adapter声明支持范围与不可用原因 |
| P1-26 | 没有GPU idle/command boundary/ownership规则 | runtime service拥有begin/end，Editor不得直接碰backend handle |
| P1-27 | 没有RenderDoc/PIX/Nsight等外部工具版本、路径与launch错误 | 建立可选tool adapter、settings、probe、sanitized command与diagnostic |
| P1-28 | GPU 6.24 ms与pass 1.84 ms没有timestamp availability/confidence | timing携带calibration、validity、latency、disjoint/unsupported状态 |
| P1-29 | CPU submit、GPU queue、present、wait与frame pacing未关联 | 统一FrameProfile timeline并显示critical stall |
| P1-30 | profiler没有bounded history、采样策略与reader backpressure | 采用ring buffer/immutable sample batch，断开consumer不拖慢render thread |
| P1-31 | 没有counter定义、单位、聚合方式与missing semantics | 建立versioned metric registry，区分0、unknown、not sampled |
| P1-32 | resource inspection没有缩略图、mip/layer/aspect/format解释与安全readback | 异步限额readback，明确转换、颜色空间、隐私和lifetime |
| P1-33 | capture/profiler artifact不进入日志、crash/session evidence | 与Editor11、Tooling07统一session/build/backend/device identity |
| P1-34 | 没有多帧比较、baseline或regression budget | 支持相同场景/设置下统计比较，明确噪声、warmup与置信区间 |

### 6.4 Lighting Bake、Lightmap与Reflection Probe（P1-35 至 P1-46）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-35 | Bake没有world/source/settings snapshot | 输入manifest冻结scene revision、geometry/material/light/probe/settings digest |
| P1-36 | 没有baker provider与hardware/RHI/project capability原因 | provider返回typed readiness，UI展示可修复action |
| P1-37 | `Production/High/4096`是独立字符串而非typed preset | preset/schema与runtime/cook共享，版本化并参与digest |
| P1-38 | 87 assets/4 warnings没有真实UV检查 | 实现mesh UV channel、overlap、padding、density、atlas limit验证与asset navigation |
| P1-39 | 6 texel bleed warning无法定位surface/chart/atlas | diagnostic绑定asset/submesh/chart/texel region并可视化overlay |
| P1-40 | Bake没有Editor09 job、进度阶段、取消与shutdown fence | 建立prepare/trace/denoise/pack/write/validate阶段和durable outcome |
| P1-41 | 没有增量invalidations与DDC key | 以输入digest拆分geometry/material/light/probe artifact，复用合法中间结果 |
| P1-42 | 没有原子publish、LKG与失败回滚 | 临时产物validate后交换manifest；失败保留旧generation和诊断 |
| P1-43 | Reflection trigger没有selection/world/placement transaction | operation从scene selection创建typed request，检查world revision并记录undo/diagnostic |
| P1-44 | capture/register不是background job且缺重复/覆盖策略 | job拥有capture、filter、encode、asset register，显式处理stable ID、overwrite和cancel |
| P1-45 | probe/lightmap结果没有scene与runtime hot-reload generation | publish事件携带artifact/world generation，runtime只接受兼容结果 |
| P1-46 | 没有质量验证和自动化渲染对比 | 对seam、leak、invalid texel、probe coverage、memory和load time建立资格门 |

### 6.5 Post Process profile、volume与preview（P1-47 至 P1-56）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-47 | Global/Cinematic/profile/volume来自静态样例 | 注册真实Profile与Volume asset/toolkit，读取project/scene selection |
| P1-48 | UI schema与09H2 effect registry不一致 | 由同一typed registry生成property editor、serialization、cook与runtime binding |
| P1-49 | profile component增删、排序、override enablement不存在 | transactional component list，stable component/property ID与undo |
| P1-50 | volume blend没有shape、priority、weight、distance、layer与camera上下文 | scene owner提供volume stack evaluation snapshot和contributor列表 |
| P1-51 | preview不绑定viewport/camera/world/time/exposure history | PreviewSession冻结上下文并返回runtime generation与history reset原因 |
| P1-52 | `Bloom 0.65`、`Filmic +0.4`允许任意字符串 | schema定义类型、范围、单位、曲线、enum、dependency与validation |
| P1-53 | LUT只显示名字/33 cube，没有import、颜色空间与兼容性 | 建立LUT asset/import artifact、dimension/format/gamut validation与preview |
| P1-54 | Apply不区分save、preview override、scene commit与runtime hot apply | 四类command分离，返回document revision、transaction ID和runtime ack |
| P1-55 | warning不绑定property、camera、volume与frame | typed diagnostic携带owner path、source span/property ID和generation |
| P1-56 | pipeline/feature不支持时仍展示相同控件 | capability-driven visibility/read-only reason，保留未知component roundtrip |

### 6.6 共享任务、诊断、测试与性能（P1-57 至 P1-60）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-57 | compile/capture/bake/readback各自可能形成私有队列 | 全部通过Editor09 admission、priority、quota、cancel、shutdown与late-commit fence |
| P1-58 | status/footer/fixed output与diagnostic journal分裂 | 以Editor11 journal为唯一持久authority，surface只做过滤投影 |
| P1-59 | focused tests把固定成功文本当正确结果 | 改为fake provider contract、generation mismatch、failure/cancel/recovery和真实artifact tests |
| P1-60 | 没有大graph、大capture、4K/8K bake、长session的预算 | 建立CPU/GPU/内存/I/O/延迟/磁盘配额与可重复benchmark，资格结果进入maturity |

## 7. P2：达到或超过成熟商用引擎后的增强项

| ID | 增强项 | 目标 |
|---|---|---|
| P2-01 | 受控Render Graph authoring | custom pass、dependency与resource声明在shared compiler和sandbox validator下编辑 |
| P2-02 | 多帧GPU trace与事件相关性 | 将CPU task、render submission、GPU queue、streaming和present跨帧关联 |
| P2-03 | 外部捕获深度集成 | RenderDoc/PIX/Nsight marker、resource name、shader debug info与一键定位 |
| P2-04 | 分布式/远程lighting build | 内容寻址输入、worker capability、可恢复分片、结果验证和零信任publish |
| P2-05 | Adaptive Probe Volume authoring | placement、subdivision、streaming cells、scenario、validity与debug visualization |
| P2-06 | 自动probe placement与质量建议 | 基于scene/visibility/importance提出可审查建议，不静默修改场景 |
| P2-07 | 高级颜色管理套件 | OCIO/display transform、HDR calibration、scopes、LUT bake与shot comparison |
| P2-08 | 多平台pipeline comparison | 同一source比较backend、shader variant、feature fallback、内存和frame cost |
| P2-09 | 远程设备GPU诊断 | 安全连接、capability negotiation、bounded telemetry和artifact回传 |
| P2-10 | 自动化render regression | golden/metric/semantic region比较、噪声模型、triage和bisect metadata |
| P2-11 | 插件化debug panel/widget API | versioned schema、权限、成本预算、卸载generation与故障隔离 |
| P2-12 | 协作式capture/bake审阅 | artifact comment、annotation、revision compare、owner与审计，不共享可变live state |

## 8. 分层实施里程碑

| Milestone | 交付内容 | 前置 | 退出条件 |
|---|---|---|---|
| M0 真实性硬切 | catalog装配、typed unavailable、移除固定成功、maturity降级 | 无 | 5项P0都有失败优先测试；默认UI不再声称假compile/bake/apply/capture |
| M1 Runtime observation bridge | graph/profile/resource/capability snapshot与generation | Runtime09A/Editor11 | 可在无GPU timing、provider重启、frame变化下保持一致投影 |
| M2 Render Graph inspector | pass/resource/barrier/queue/alias/diagnostic与dump | M1、Runtime09A/09C | 真实frame可冻结、搜索、定位、导出；无静态graph数据 |
| M3 Capture与Profiler | target选择、capture service、external adapters、timeline、readback | M1、Editor07/09 | 成功/失败/cancel都有artifact或typed reason，render thread无阻塞consumer |
| M4 Lighting与Probe | readiness、manifest、job、incremental artifact、LKG、probe operation | M0、Runtime09F1/09F2 | scene revision到validated artifact闭环，失败不破坏旧结果 |
| M5 Post Process | profile/volume asset、schema、transaction、preview/apply | M0、Runtime09H2、Editor02 | Editor/cook/runtime使用同一schema，Apply有ack与generation |
| M6 平台与运维收敛 | backend矩阵、quota、journal、shutdown、crash evidence、recovery | M2-M5 | device loss、project close、plugin unload和磁盘失败均有确定终态 |
| M7 规模与性能资格 | 大graph、长trace、4K/8K bake、大volume stack benchmark | M6、Tooling07 | 预算、baseline、回归阈值和qualification artifact可重复 |
| M8 第二authority删除 | 删除固定样例业务handler/断言，manifest status由资格生成 | M7 | 静态扫描与产品测试证明所有成功状态来自真实provider |

实施顺序不得跳过M0。Runtime09F2没有真实baker时，M4只能交付Unavailable/readiness与接口，不得用mock产物把Bake重新标为可用。

## 9. 验收门（32项）

### 9.1 产品装配与真实性

- G01：启用Rendering的EditorHost由manifest驱动注册唯一RenderingEditor provider；未链接时返回reason-coded unavailable。
- G02：15个optional feature的manifest、Cargo、catalog、descriptor映射可机器校验，无幽灵Editor module。
- G03：`stable/complete`只能由当前commit/platform/backend的qualification artifact发布。
- G04：静态扫描确认ZUI/feedback不再包含Frame 1234、6.24 ms、87 lightmaps、02:30、Global Stack等业务成功样例。

### 9.2 Render Graph与资源

- G05：冻结frame后所有graph、resource、barrier、timing pane共享同一frame token。
- G06：pass/resource ID稳定且可从diagnostic、bottom panel双向导航。
- G07：culled/merged/fallback/failed pass均有typed disposition与reason。
- G08：resource lifetime、alias heap/offset/size和峰值bytes可由runtime report复算。
- G09：cross-queue wait/signal/ownership transfer能形成无断边timeline。
- G10：compile结果携带input digest、artifact generation、diagnostic set与LKG。
- G11：provider重启或generation mismatch时旧pane标为stale，不拼接新数据。
- G12：versioned headless graph dump可由CI读取并与UI计数一致。

### 9.3 Capture与Profiler

- G13：capture明确绑定project/world/session/viewport/process/backend/frame range。
- G14：unsupported backend、Null/headless、device loss、tool missing均返回不同typed reason。
- G15：begin/end capture由runtime frame boundary owner执行，Editor不持有裸backend handle。
- G16：capture success产生存在、可读、带metadata的artifact；失败不写假artifact。
- G17：GPU timing区分valid、pending、unsupported、disjoint并显示采样generation。
- G18：profiler reader停顿不会令render thread阻塞或使内存无界增长。
- G19：resource readback受bytes/rate/concurrency配额约束，并处理资源过期。
- G20：CPU/GPU/present timeline对同一frame可关联且单位/聚合定义可查询。

### 9.4 Lighting、Probe与Post Process

- G21：Bake输入manifest完整覆盖world、geometry、material、light、probe、settings revision/digest。
- G22：无合格baker时按钮不可执行并显示可修复原因，绝不返回queued/ready。
- G23：Bake job支持阶段进度、取消、关闭栅栏；late result不能提交到新project/world。
- G24：publish前验证artifact，失败保留LKG且不会留下半写manifest。
- G25：UV/bleed/atlas diagnostic可定位到asset、submesh/chart与viewport overlay。
- G26：Reflection Probe capture从产品operation可达，具备selection、transaction、job、register和overwrite policy。
- G27：Post Process profile/volume变更具备undo/save/recovery与stable component/property ID。
- G28：Editor、cook和runtime从同一09H2 registry/schema读取effect与property。
- G29：Preview/Apply返回source revision、runtime generation与ack，stale ack被拒绝。
- G30：不兼容pipeline/feature的component保留roundtrip并展示read-only原因。

### 9.5 故障、规模与删除

- G31：大graph、长capture、4K/8K bake和大volume stack通过声明的CPU/GPU/内存/I/O/磁盘预算。
- G32：删除所有固定业务feedback及对应“字符串成功”测试，故障注入覆盖provider missing、cancel、device loss、disk full、plugin unload和generation race。

## 10. 跨计划边界与依赖

- Runtime09A拥有RHI、Render Graph执行、resource lifetime、barrier、queue、alias和GPU report真值；本篇只定义Editor provider与投影。
- Runtime09C与Editor15拥有shader/material/PSO、Shader Editor和VFX authoring；本篇只展示其compile artifact/diagnostic，不重建graph compiler。
- Runtime09F1拥有环境、IBL与Reflection Probe runtime capture/消费合同；本篇拥有scene operation、job、asset register与debug UI。
- Runtime09F2拥有Lightmap/Irradiance真实baker、artifact与runtime consumption；在其退出门前，本篇Bake保持Unavailable。
- Runtime09H2拥有exposure/color/bloom/DoF/motion blur/SSR registry、composition和runtime profile语义；本篇拥有transactional profile/volume authoring与preview/apply。
- Editor02、09、11分别拥有transaction/recovery、job与diagnostic journal；Rendering只能接入，不能复制私有实现。
- Plugin01拥有Rendering Editor packages是否进入workspace、dist/ABI/catalog生成的机械边界；本篇拥有产品贡献与资格要求。
- Tooling07拥有benchmark/profile/capture/crash evidence基础设施；本篇提供渲染域场景、指标和退出阈值。

## 11. 实施前复核清单

1. 重算99文件fingerprint，特别复核两个在途import排序文件是否出现语义修改。
2. 重扫16个Editor descriptor贡献API与first-party catalog，防止其他Session已经接入产品owner。
3. 重扫Reflection Probe helper product caller；若已有caller，按真实链更新P0-05而非覆盖实现。
4. 从Runtime09A重新生成report/type inventory，确认snapshot lifetime、thread ownership与generation字段终态。
5. 复核Runtime09F2是否已经引入合格baker；没有则M4不得启用Bake。
6. 复核09H2 registry与profile schema，禁止Editor先发明不兼容字段。
7. 先修复或隔离当前Editor测试编译阻断，再运行focused contract、fault injection、benchmark和平台矩阵。
8. 动态证据必须记录commit、target、backend、adapter/driver、project fixture、command、duration和artifact；只贴成功截图不构成退出证据。
