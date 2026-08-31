---
title: Editor Render Pipeline / Render Graph / Frame Debugger / Capture / Lighting Bake / Reflection Probe / Post Process / Debug 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor96
review_date: 2026-08-25
baseline_head: 8ee9411db24b7b4bdaf3fe028194642a7557c0b6
verification_head: 0fd7df4ecdd157f9505cd51013780e3225cfb83c
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99c-runtime-exposure-color-tonemap-lut-bloom-dof-motion-blur-ssr-output-transfer-terminal-composition-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/91-editor-material-shader-graph-material-instance-vfx-particle-preview-compiler-diagnostics-authoring-product-integration-current-source-review.md
related_code:
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/editor
  - zircon_plugins/rendering/runtime
  - zircon_plugins/rendering/features
  - zircon_plugins/rendering/dist
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/zircon_shader_pbr_viewer
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated
  - zircon_editor/src/core/editor_manager
  - zircon_editor/src/core/editor_plugin
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Developer/RenderDocPlugin
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass
  - dev/UnrealEngine/Engine/Plugins/Experimental/Toolsets/PerfToolset
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphTrace.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/DumpGPU.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting
  - dev/godot/editor/scene/3d/lightmap_gi_editor_plugin.cpp
  - dev/godot/editor/debugger/editor_visual_profiler.cpp
  - dev/Fyrox/editor/src/light.rs
  - dev/Fyrox/editor/src/plugins/inspector/editors/property/probe.rs
  - dev/bevy/crates/bevy_render/src/diagnostic
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor96 · Rendering Authoring 与 Product Integration 当前源码复审

## 1. 结论

Zircon并非没有渲染运行时基础。Runtime已经有Render Graph materialization、resource/alias/coverage/profile/parallel/stage report，RenderFrameProfile还表达frame generation、GPU timing状态、CPU submit、pass/subsystem timing、transient/persistent memory、统计与latency。Viewport capture有generation、capture provenance、可选graph dump和frame profile JSON，并通过有界readback slot回传。Post Process也已有scene asset、volume component registry/evaluator、priority、weight、distance、layer与camera求值。这些结构必须成为Editor唯一事实源，不能在Workbench重做第二套统计或配置模型。

真正的P0在产品装配和真实性。Rendering manifest把根模块与15个optional feature都声明为Editor模块，maturity为stable，runtime.plugin.rendering状态为complete；但66个Editor Rust文件合计仅注册16个capability，没有register_editor_extensions、view、drawer、menu、settings、asset toolkit、operation factory、graph、viewport provider、event consumer或lifecycle owner。EditorPluginDescriptor内建catalog会把manifest投影成可见状态行，但默认EditorPlugin实现不会因此获得可执行贡献。默认App只显式链接Navigation/Neural Editor provider，没有链接Rendering Editor provider。

当前源码还出现更直接的静态漂移：manifest与Runtime declaration均列出15个feature，而Editor manager测试仍断言9个，只覆盖post_process、ssao、contact_shadow、decals、reflection_probes、baked_lighting、ray_tracing_policy、shader_graph和vfx_graph，遗漏volumetric_fog、oit、light_cookies、irradiance_volumes、planar_reflections、subsurface_scattering。所有Rendering crate虽已进入zircon_plugins工作区，这只解决编译成员身份，没有解决默认产品装配、provider readiness或功能合同。

Workbench继续显示第二套静态“成功”外观。Render Pipeline固定显示Frame 1234、SceneColor到BloomInput 1.84 ms、R11G11B10_FLOAT Read、Windows DX12 30 fps、GPU 6.24 ms和MainPipeline.rp；Lighting Bake固定显示87 assets、4 warnings、12 volumes和02:30；Post Process固定显示Global Stack、Cinematic Grade、Bloom 0.65、LUT_CityWarm与EV +2.1。三份workspace各27个control、19条route。command只回写预制文本，field edit只改retained control的value/value_text，没有document revision、transaction、job、runtime acknowledgement或artifact。

ReflectionProbeCaptureEditorTrigger是局部真实基础：它会capture、persist和register，并有request/placement roundtrip测试。但产品没有caller、operation factory、job、selection、transaction和diagnostic owner；其Runtime下游还调用已不存在的SceneRenderer::render_scene_color_hdr，具体执行缺陷由Runtime96持有。PBR Viewer已有真实RenderDoc 1.4.1桥、GPU debugger capture请求和非空rdc验证，但只属于viewer私有路径，尚未成为共享Editor capture service。

本轮对Editor22的 **5项P0、60项P1、12项P2全部重判为Open，32项Editor资格门全部Fail**。Editor96只刷新currentness，不重复增加canonical finding总数。当前没有动态、规模或竞争证据支持Zircon Rendering Editor的功能、性能或表现优于Unreal。

## 2. 审查边界、统计与currentness

### 2.1 冻结范围

统计对象为当前working tree物理文件。行与非空行按文本物理行统计，bytes取文件长度；tests/ignored只计Rust test/ignore属性，因此Unreal C++与Unity C#测试不增加该列。fingerprint按repository-relative lowercase path排序，为每个文件拼接path、NUL、lowercase文件SHA-256与LF后再取SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Rendering plugin/editor selected | **73 / 2,379 / 2,077 / 79,498 / 9 / 0** | bccdf74ae4ec7fb2eb2f6172e1d9e32abe0162ee0f7de313b7fe4405f92aaa7c |
| Catalog与默认产品装配 | **19 / 2,645 / 2,441 / 97,470 / 19 / 0** | 541fcdc90d97ea329d0a12ebe0028ede619e9cd06d08bdf2a6fcb71e974c2d28 |
| Workbench selected | **27 / 9,932 / 9,529 / 439,815 / 27 / 0** | a93c3f9235e72fa5cf22f51d0d4477e537200874e7ac4e7e55cade4fef143479 |
| Runtime boundary selected | **28 / 6,065 / 5,515 / 217,462 / 50 / 0** | f77a69f9d42430937a755ba4355a2cacb46d4ea63e411116a163bf6706bb7e42 |
| PBR Viewer capture selected | **5 / 3,934 / 3,626 / 160,009 / 60 / 0** | 7a7db5d7727c965176919cd9c6aa7286d14b3a5bdfab653454922bfef92eef09 |
| Zircon selected union | **152 / 24,955 / 23,188 / 994,254 / 165 / 0** | 7956dd0ad78ad88a76ff3b65029e5b04a34ed37a554dab3b69d7a99c0f32fd31 |
| Unreal selected | **10 / 6,310 / 5,387 / 227,700 / 0 / 0** | 6f4a1fd17365db327ed82de68ae5c7b655ba05bd37d2df708289480811b1b640 |
| Unity Graphics selected | **17 / 7,766 / 6,531 / 350,141 / 0 / 0** | 9eb2a41895c51423a9465ca04f0e2ab34cefbfe45124329df83676f3352c974a |
| Godot selected | **6 / 1,633 / 1,343 / 60,515 / 0 / 0** | f45b5cf26fbdc968edfb63d678a13d330353d1c72cd5591c106b25ae75a94c2a |
| Fyrox selected | **5 / 1,222 / 1,127 / 46,284 / 0 / 0** | 5c0508042dd61963951d3467efa84475fc0e3ccb5013987c34a4dccf0932c7b7 |
| Bevy selected | **4 / 2,372 / 2,108 / 83,668 / 0 / 0** | 675cc3b4eaf17ff7b08c7513f1a4d5659b6b48184bad32dca22da9ae4b377239 |
| 五引擎reference selected union | **42 / 19,303 / 16,496 / 768,308 / 0 / 0** | 2378abe6b4c9d5981a943a5c8bf2a48aa0975d2f029ce65c2eb20b2576e83ce1 |

### 2.2 currentness与限制

- baseline HEAD为8ee9411db24b7b4bdaf3fe028194642a7557c0b6；写入时verification HEAD为0fd7df4ecdd157f9505cd51013780e3225cfb83c，二者间相关Rendering/Workbench路径没有已提交差异。
- 选择集包含用户或其他Session在途修改与未跟踪文件；本轮读取当前物理状态，不回退、不覆盖，也不把在途代码写成已集成能力。
- 参考revision：Unity Graphics a7e4c051d256a781ab362c64316b125a1e104694、Godot 8c7e6c5877a78e8e61ea4fd42673219a9091dca7、Fyrox 8d815db36494f1badb347547dfc7094bf4fbbdf8、Bevy fb89a8649d9b359e53ffb6e5492ebb7c059ac8af；Unreal无独立Git元数据，以所选文件fingerprint冻结。
- 按用户要求未查询、轮询或等待协调器；Tooling不在本轮范围。
- 本轮仅静态review，没有运行Cargo、App/Editor、真实GPU capture、RenderDoc、Bake、save/reopen、PIE、fault、scale、soak、profile或竞争benchmark。

### 2.3 Owner边界

- Editor96唯一负责Rendering Editor provider装配、Render Pipeline document、Render Graph/Frame Debugger、capture控制面、Lighting/Probe build workflow、Post Process profile/volume authoring和真实Workbench投影。
- [Runtime89](../zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md)与[Runtime90](../zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md)唯一负责graph执行、RHI资源、barrier、queue、completion、readback与设备故障。
- [Runtime96](../zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md)、[Runtime97](../zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md)与[Runtime99c](../zircon_runtime/99c-runtime-exposure-color-tonemap-lut-bloom-dof-motion-blur-ssr-output-transfer-terminal-composition-product-integration-current-source-review.md)分别拥有Probe/IBL、baker/artifact/runtime sampling和Post Process执行。
- [Editor91](91-editor-material-shader-graph-material-instance-vfx-particle-preview-compiler-diagnostics-authoring-product-integration-current-source-review.md)拥有Material/Shader/VFX资产、compiler与diagnostic；Editor96只消费其artifact。

## 3. 当前产品链事实

| 层 | 当前事实 | 判定 |
|---|---|---|
| Manifest | 根Rendering加15个optional feature都有runtime/editor module，4个默认开启 | 声明完整不等于产品完整 |
| Plugin workspace | 根、dist及15类feature crate均是zircon_plugins工作区成员 | 只关闭旧“未进workspace”的字面部分 |
| Editor descriptor | 66个Editor文件，16次with_capability，0次extension/provider/operation注册 | capability-only skeleton |
| Builtin catalog | 由manifest生成descriptor和status row | 可见metadata，不是已加载provider |
| Default App | target-editor-host链接Navigation/Neural Editor，未链接Rendering Editor | 默认产品不可执行 |
| Catalog test | 仍断言9项feature，源码/manifest实际15项 | 当前合同漂移 |
| Render workspace | 27 controls、19 routes、固定frame/pass/resource/platform/profile | 静态第二authority |
| Lighting workspace | 27 controls、19 routes、固定scene/count/warning/estimate | 假Bake控制面 |
| Post Process workspace | 27 controls、19 routes、固定stack/profile/effect值 | 假资产编辑 |
| Bottom panel | 61 controls、48 routes，Render Pipeline五行只是route label | 无provider数据 |
| Command feedback | compile/preview/bake/apply均返回预制文本 | 无domain effect |
| Field edit | 仅修改retained control value/value_text | 无transaction/revision |
| Viewport capture | CapturedFrame可带graph dump/profile JSON | Editor只转发RGBA，数据未消费 |
| Probe helper | 有capture/persist/register薄适配和2个roundtrip测试 | 无产品caller且下游静态断裂 |
| PBR RenderDoc | viewer可请求capture并验证rdc | 私有能力，非共享Editor service |

## 4. 必须保留的工程基础

1. 保留RenderFrameProfile、RenderGpuTimingStatus及pass/subsystem/memory/latency字段，Editor只做query、filter和projection。
2. 保留Render Graph materialization/resource/alias/coverage/profile/parallel/stage report，建立稳定调试快照，不复制编译器。
3. 保留CapturedFrame的generation、provenance、graph dump与frame profile JSON；补齐Editor消费、匹配、过期拒绝和持久化。
4. 保留有界readback mailbox和matching-generation profile attach语义，继续由Runtime控制GPU完成与背压。
5. 保留typed Post Process scene asset、volume registry/evaluator及camera/layer/priority求值，Editor围绕同一schema建设document。
6. 保留ReflectionProbeCaptureEditorTrigger的DTO与register意图，但必须通过共享operation/job/transaction/gateway进入产品。
7. 保留PBR Viewer RenderDoc ABI、设备capture请求和rdc验证，把可复用部分提升为能力驱动共享服务。

## 5. P0：产品真实性与数据安全

| ID | 状态 | 差距 | 必须重构 |
|---|---|---|---|
| RENED-P0-01 | Open | stable/complete、15项manifest和builtin catalog row制造“已具备Rendering Editor”的错误承诺；默认App无Rendering provider，测试还停在9项 | 统一manifest、generated descriptor、linked provider、extension registry与readiness；任何不一致必须fail-close并显示Unavailable |
| RENED-P0-02 | Open | Render Pipeline、Frame Debugger和bottom panel由固定frame/pass/timing/resource/profile充当第二authority | 删除fixture authority，建立generation-qualified Runtime observation snapshot；compile/capture只在真实receipt后成功 |
| RENED-P0-03 | Open | Lighting Bake页面可显示Preview ready/Bake queued，却没有真实baker、JobId、cancel、artifact、atomic commit或failure owner | 接Editor09与Runtime97，建立snapshot、admission、progress、cancel、staging、validation、generation-safe apply和last-known-good |
| RENED-P0-04 | Open | Post Process Apply只改文本/control，用户会认为profile/scene/runtime已改变 | 建立transactional profile/volume document、serialized persistence、undo/recovery、runtime preview generation与acknowledgement |
| RENED-P0-05 | Open | Probe helper产品不可达，且下游调用已删除renderer方法；不存在transaction/job/selection/lifecycle | 通过operation factory与bounded job接入selected probe，先修复Runtime96 owner，再以artifact transaction注册并处理cancel/stale/shutdown |

## 6. P1：工程化完整性

### 6.1 装配、插件与生命周期

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P1-01 | Open | feature清单存在15对9漂移；生成、声明、测试与UI必须由同一manifest解析结果驱动并做双向完整性验证 |
| RENED-P1-02 | Open | 16个capability没有owner lease、load/unload/reload/shutdown；每项贡献必须绑定plugin generation并可原子撤销 |
| RENED-P1-03 | Open | 默认App不链接Rendering Editor；项目enablement必须原子装配runtime/editor/resources/provider/controller |
| RENED-P1-04 | Open | descriptor capability可在零executor时显示ready；readiness必须验证provider、factory、runtime backend与资源 |
| RENED-P1-05 | Open | 15个feature Editor crate仅重复四字段plugin descriptor；收敛共享注册框架，但保留feature owner与typed contract |
| RENED-P1-06 | Open | disable/reload没有关闭panel、capture reader、job、preview和runtime consumer的顺序合同 |
| RENED-P1-07 | Open | rendering crates虽已进入plugin workspace，但target-editor-host仍无产品lane；补默认链接、feature矩阵与缺失依赖fail-close |
| RENED-P1-08 | Open | capability ID没有版本/schema/compatibility；增加contract version、producer generation与compat diagnostic |

### 6.2 Render Pipeline document与compiler控制面

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P1-09 | Open | MainPipeline.rp只是文本输入；建立versioned RenderPipelineSource、document revision、dirty/save/reopen/recovery |
| RENED-P1-10 | Open | captured graph dump已存在但Editor不消费；建立snapshot parser/view model并保留未知字段与版本诊断 |
| RENED-P1-11 | Open | Compile没有source hash、target、feature set、shader/material artifact key；建立typed request/result/provenance |
| RENED-P1-12 | Open | compile feedback不区分queued/running/succeeded/failed/canceled/stale；使用Editor09 job terminal状态 |
| RENED-P1-13 | Open | 无pipeline validation、cycle、missing producer、format/usage/queue不兼容诊断定位 |
| RENED-P1-14 | Open | 无last-known-good与staging；编译失败不得替换当前Runtime generation |
| RENED-P1-15 | Open | 无dependency/recompile invalidation；接Material/Shader artifact revision与feature capability generation |
| RENED-P1-16 | Open | 无undo/redo、diff、merge和外部修改冲突；统一Editor02 document transaction |

### 6.3 Render Graph与Frame Debugger

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P1-17 | Open | 无真实pass tree、culled/async/merged/parallel/queue状态；从Runtime89快照投影 |
| RENED-P1-18 | Open | 无resource read/write、first/last use、alias、acquire/discard与barrier可视化 |
| RENED-P1-19 | Open | 无texture/buffer descriptor、format、extent、mip/layer/sample、size和residency inspection |
| RENED-P1-20 | Open | 无pass-resource双向选择、依赖边、producer/consumer与source定位 |
| RENED-P1-21 | Open | 无graph revision/hash、frame generation、pipeline generation和capture source显示 |
| RENED-P1-22 | Open | 无local/remote session、disconnect、incompatible schema、stale snapshot和reset语义 |
| RENED-P1-23 | Open | Runtime已有timing status但UI固定1.84/6.24 ms；逐状态显示Disabled/Pending/Unavailable/Measured等真实含义 |
| RENED-P1-24 | Open | 无历史帧、选择游标、搜索、过滤、排序、bookmark与bounded retention |
| RENED-P1-25 | Open | 无CPU submit、GPU pass、subsystem、memory、stats和latency统一时间线 |
| RENED-P1-26 | Open | 无snapshot预算、partial/drop/age/coverage telemetry；调试本身的observer cost不可见 |

### 6.4 Capture、GPU profiler与外部调试器

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P1-27 | Open | PBR Viewer已有RenderDoc桥但为私有；提升为共享capture service并按D3D11/D3D12/Vulkan/wgpu backend选择能力 |
| RENED-P1-28 | Open | CapturedFrame只被Editor转RGBA；消费graph/profile/provenance并严格匹配frame generation |
| RENED-P1-29 | Open | 无next-frame/delayed/multiframe/current-viewport/all-activity capture状态机 |
| RENED-P1-30 | Open | 无capture admission、GPU idle/submit boundary、timeout/cancel、device-loss与NullRHI fail-close |
| RENED-P1-31 | Open | 无artifact命名、目录、atomic finalize、retention、size budget、open/reveal/export和corrupt验证 |
| RENED-P1-32 | Open | 无texture/buffer/pass dump filter、staging cap、OOM warning和异步流式写出 |
| RENED-P1-33 | Open | 无capture schema/version/tool version/adapter/backend/build/source revision provenance |
| RENED-P1-34 | Open | 无GPU profiler真实result tree、JSON schema、one-shot lifetime与end-frame completion测试 |

### 6.5 Lighting Bake、Lightmap与Irradiance Volume

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P1-35 | Open | 固定City_Block_A/Interior_Lab不是scene projection；枚举真实scene、baking set、scenario与依赖 |
| RENED-P1-36 | Open | 无world/RHI/hardware/ray tracing/project setting/backend capability检查与禁用原因 |
| RENED-P1-37 | Open | 无immutable bake input snapshot、source hash、settings、quality、platform与backend key |
| RENED-P1-38 | Open | 无UV2、mesh、material、light、writable path、atlas、texture与empty source前置验证 |
| RENED-P1-39 | Open | 无worker/admission/resource budget/priority/dependency DAG和进度阶段 |
| RENED-P1-40 | Open | 无cancel acknowledgement、deadline、source mutation stale reject和shutdown barrier |
| RENED-P1-41 | Open | 无lightmap/probe artifact staging、hash/format/atlas验证、atomic publication与rollback |
| RENED-P1-42 | Open | 无apply/clear/unlink/delete差异、undo/redo、save/reopen和reference migration |
| RENED-P1-43 | Open | 无warning/error定位到scene object、mesh、material、light、volume和生成文件 |
| RENED-P1-44 | Open | 无incremental dirty set、cache hit/miss、reuse、determinism和last-known-good报告 |
| RENED-P1-45 | Open | 无preview mode、baked/unbaked difference、texel density、overlap和coverage overlay |
| RENED-P1-46 | Open | 无多scene/baking set/scenario/streaming cell与cross-scene ownership合同 |

### 6.6 Reflection Probe、Post Process与Rendering Debug

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P1-47 | Open | Runtime有probe DTO/helper但无Editor asset/toolkit、scene component drawer、gizmo和selection operation |
| RENED-P1-48 | Open | probe capture无position/shape/influence/parallax/priority/resolution/refresh policy完整authoring |
| RENED-P1-49 | Open | 无capture/convolution/SH/PMREM阶段、progress、artifact provenance与failure presentation |
| RENED-P1-50 | Open | 无probe grid/irradiance volume placement、density、coverage、overlap与memory预算工具 |
| RENED-P1-51 | Open | Runtime已有typed volume evaluator，Editor仍无PostProcessProfile/Volume toolkit与serialized component list |
| RENED-P1-52 | Open | 无add/remove/reorder/enable/override、multi-edit、undo及pipeline compatibility过滤 |
| RENED-P1-53 | Open | 无global/camera/local volume、blend distance、weight、priority、layer mask与effective stack inspection |
| RENED-P1-54 | Open | 无preview camera/session/generation、before-after、effect isolate和runtime acknowledgement |
| RENED-P1-55 | Open | 无exposure/histogram/color/LUT/bloom/DoF/motion blur/SSR typed validators和单位/范围诊断 |
| RENED-P1-56 | Open | 无source profile、override来源、混合贡献与最终resolved value逐字段追踪 |
| RENED-P1-57 | Open | 无wireframe/overdraw/light complexity/shadow/probe/volume/resource等debug mode registry与per-viewport状态 |
| RENED-P1-58 | Open | 无diagnostic provider、severity、source navigation、dedupe、retention、export与Notification/Console路由 |
| RENED-P1-59 | Open | feature enablement与debug UI不从同一capability snapshot生成，六项feature可在测试合同中消失 |
| RENED-P1-60 | Open | 无scale/soak/profile/correctness/visual golden与同质量参考对照，不能声明性能和表现领先 |

## 7. P2：大型项目、协作与高级调试

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| RENED-P2-01 | Open | 支持远程设备/主机、多adapter与多viewport capture session |
| RENED-P2-02 | Open | 支持多人共享capture、annotation、bookmark和source revision锁定 |
| RENED-P2-03 | Open | 支持Render Graph结构diff、frame-to-frame timing/memory regression diff |
| RENED-P2-04 | Open | 支持pass/resource breakpoint、条件capture与自动触发规则 |
| RENED-P2-05 | Open | 支持超大graph虚拟化、分页、LOD、搜索索引与渐进加载 |
| RENED-P2-06 | Open | 支持跨平台capture artifact转换、兼容性矩阵与离线查看 |
| RENED-P2-07 | Open | 支持distributed bake、worker capability、lease、checkpoint、resume与结果合并 |
| RENED-P2-08 | Open | 支持large-world cell/scenario增量Bake、streaming预算和批量验证 |
| RENED-P2-09 | Open | 支持probe自动布局、质量启发式、overlap优化与平台预算建议 |
| RENED-P2-10 | Open | 支持Post Process profile inheritance、variant、批量diff与迁移 |
| RENED-P2-11 | Open | 支持可脚本化capture/bake/validation命令及稳定machine-readable result |
| RENED-P2-12 | Open | 支持长期趋势、回归阈值、自动bisect输入与可复现实验包 |

## 8. Editor22当前性重判

| Editor22范围 | 当前状态 | 新证据 |
|---|---|---|
| 5项P0 | **5 Open** | 默认产品仍无Rendering provider，三份workspace仍是假成功，probe helper仍不可达且下游断裂 |
| 60项P1 | **60 Open** | Runtime profile/capture/volume与viewer RenderDoc提供了可复用底座，但没有形成任一完整Editor产品链 |
| 12项P2 | **12 Open** | 未发现远程、多用户、diff、distributed bake、large-world qualification或自动回归产品 |
| 32项Gate | **32 Fail** | 无动态证据能关闭任何Gate |

关键currentness修正：

1. 旧报告“Rendering crates未进入root workspace”已不再精确；它们现已进入zircon_plugins工作区，但默认App链接和可执行provider仍缺失，因此对应finding保持Open。
2. 旧报告“缺RenderDoc”需收窄：PBR Viewer已有真实桥，但Editor无共享服务、产品命令和artifact workflow，因此finding保持Open。
3. Runtime capture现在能携带graph dump/profile，Post Process已有typed volume evaluator；Editor仍未消费，相关finding保持Open而非Closed。
4. builtin catalog能显示Rendering及capability只是manifest投影，不能被判为provider接入完成。

## 9. 参考引擎差异

| 参考 | 当前源码证据 | Zircon必须吸收的合同 |
|---|---|---|
| Unreal RenderDoc | backend选择、render command边界、GPU idle、NullRHI fail-close、delayed/multiframe、路径移动与外部启动 | capture state machine、设备能力、严格完成点、typed failure与artifact lifecycle |
| Unreal PerfToolset/RenderGraphTrace/DumpGPU | delayed frame、one-shot RHI capture、真实JSON tree测试、pass/resource/alias/lifetime事件、dump过滤与staging cap | generation-qualified profiler result、结构化graph snapshot、资源dump预算和observer cost |
| Unreal GPU Lightmass | world subsystem Start/Stop，明确project/RHI/hardware/ray tracing禁用原因 | capability gating、cancelable world-scoped job和可解释失败 |
| Unity RenderGraph Viewer | local/remote session、schema兼容、culled/async/merged pass、resource lifetime与fence边 | 调试会话、兼容诊断、pass/resource双向关系和可视化 |
| Unity Volume/Lighting | serialized profile、Undo、component兼容；异步Bake/Cancel、scenario、warning、clear与cleanup | transactional profile、pipeline-aware schema、真实Bake lifecycle |
| Godot | 真实LightmapGI bake、UV/mesh/path/atlas错误、progress cancel；visual profiler有界历史 | fail-close验证、可取消progress和bounded frame history |
| Fyrox | reflected bake settings、scene input snapshot、worker/cancel token、texture保存与apply | typed settings、后台任务、stage progress、artifact apply/clear |
| Bevy | render diagnostics plugin registry和lifecycle | feature-owned diagnostics注册、enable/disable和资源清理 |

## 10. 目标架构

唯一产品链固定为：

RenderPipelineSource / PostProcessProfile / BakeSettings
→ Editor02 transactional document
→ typed operation与Editor09 bounded job
→ RuntimeGateway request（source/document/world/plugin generation）
→ Runtime89/90/96/97/99c execution
→ immutable result/artifact/observation snapshot
→ generation-safe atomic commit
→ toolkit / viewport / Frame Debugger / Job Center / Console真实投影。

核心边界：

- RenderingEditorProvider负责注册views、toolkits、operations、runtime consumers、viewport providers与diagnostics，所有注册绑定plugin generation。
- RenderObservationService只缓存Runtime发布的不可变快照，按session/world/view/frame/pipeline generation索引，并有容量、drop、age与reader lease。
- RenderPipelineDocument和PostProcessProfileDocument拥有source revision、transaction、save/recovery；不允许ZUI control成为authoritative state。
- RenderBuildController只编排Job、Runtime请求、staging与commit；算法、RHI和baker仍属于Runtime owner。
- CaptureService统一内部frame capture、Render Graph/Profile snapshot和外部RenderDoc capture，产出versioned artifact。

## 11. 必须执行的硬切

1. 在provider可执行前，将Rendering Editor maturity/readiness降为Prototype或Unavailable；禁止stable/complete仅由manifest推导。
2. 删除Render、Lighting与Post Process的固定业务fixture和固定成功feedback；未接真实backend时明确Unavailable。
3. 删除9项手写feature测试清单，改为对manifest生成的15项全集做完整性测试。
4. 禁止新增capability-only feature crate；每个capability必须有owner、factory/provider、readiness与shutdown验证。
5. 禁止Editor复制Render Graph、Frame Profile、Post Process或Bake Runtime schema；通过versioned DTO消费唯一owner。
6. 将PBR Viewer可复用RenderDoc桥迁入共享服务，viewer与Editor共同消费，旧私有authority不长期并存。
7. Reflection Probe产品接入必须等待Runtime96下游方法恢复或替代，不得把静态断裂包装成queued/succeeded。

## 12. 依赖顺序里程碑

### M0：Truthful Catalog与默认装配

- 修复15对9漂移，建立generated feature matrix、provider链接与readiness。
- 默认App按项目manifest装配Rendering runtime/editor/resources，并验证disable/reload/shutdown。

退出门：零executor capability不会显示Ready，默认产品能打开真实owner surface。

### M1：Document与Observation基座

- 建立RenderPipelineDocument、PostProcessProfileDocument和RenderingEditorProvider。
- 建立generation-qualified RenderObservationService，消费graph/profile/capture。

退出门：source edit/save/reopen/undo/recovery与frame stale reject通过。

### M2：Render Graph与Frame Debugger

- 实现pass/resource/lifetime/alias/barrier/queue/timing/memory视图。
- 实现filter/search/history/bookmark与有界snapshot。

退出门：所有显示值均能追溯到同一frame/pipeline generation。

### M3：Capture与GPU Profiler

- 提升共享RenderDoc bridge，建立delayed/multiframe状态机和capture artifact。
- 完成profile result tree、JSON schema、路径/retention/budget/corrupt处理。

退出门：设备不支持、timeout、device loss、cancel与成功artifact均有真实终态。

### M4：Lighting与Probe Build

- 接入Runtime97 bake request/output/snapshot及Runtime96 probe artifact。
- 完成capability validation、bounded job、cancel、staging、atomic apply、undo与last-known-good。

退出门：真实scene可Bake/cancel/fail/reopen，固定87 assets/02:30文本为0。

### M5：Post Process Authoring

- 实现profile/volume toolkit、serialized components、override、compatibility和effective stack。
- 实现preview session、before-after、effect isolate与runtime acknowledgement。

退出门：global/camera/local混合与save/reopen/undo在Runtime结果中一致。

### M6：Debug、Diagnostics与故障恢复

- 实现per-viewport debug modes、typed diagnostics、source navigation与导出。
- 验证reload、world switch、runtime restart、device loss、late reply和shutdown cleanup。

退出门：旧generation下一帧不可见，所有reader/job/provider可回收。

### M7：规模与产品资格

- 完成大graph、长历史、多viewport、large-world bake、probe grid和volume stack压力矩阵。
- 保存correctness、fault、soak、profile、raw terminal与同质量参考对照证据。

退出门：性能结论有可复现实验，而不是UI样例数字。

### M8：删除第二authority

- 删除所有rendering Workbench固定fixture、兼容桥和旧私有capture入口。
- 复核manifest、catalog、provider、command、job、runtime与artifact一一对应。

退出门：production tree中固定Frame 1234、GPU 6.24 ms、87 assets、02:30和Global Stack样例authority为0。

## 13. G01-G32 Editor资格门

| Gate | 状态 | 通过条件 |
|---|---|---|
| G01 | Fail | manifest 15项feature与generated descriptor、测试、默认App和provider全集一致 |
| G02 | Fail | 每个可见capability都有owner、factory/provider、readiness、reload与shutdown |
| G03 | Fail | Render Pipeline create/open/edit/save/reopen/undo/recover不丢失 |
| G04 | Fail | Compile经真实source/artifact/backend完成且失败保留last-known-good |
| G05 | Fail | Frame Debugger pass/resource/alias/barrier/queue来自同一generation |
| G06 | Fail | culled/async/merged/parallel状态与Runtime89执行事实一致 |
| G07 | Fail | GPU timing各状态不被伪装成数字，pending/stale/disabled可解释 |
| G08 | Fail | frame history有界且drop/age/coverage/observer cost可见 |
| G09 | Fail | capture frame/profile/graph严格匹配session/view/frame generation |
| G10 | Fail | RenderDoc按真实backend选择，在不支持/NullRHI/device loss时fail-close |
| G11 | Fail | delayed/multiframe/current viewport/all activity状态机完成 |
| G12 | Fail | capture artifact原子落盘、版本化、可验证、可保留与可清理 |
| G13 | Fail | GPU profiler tree/JSON/result lifetime与end-frame completion有测试 |
| G14 | Fail | Lighting Bake在项目/RHI/硬件/backend不支持时给出精确禁用原因 |
| G15 | Fail | UV/mesh/material/light/path/atlas/texture输入错误在提交前被定位 |
| G16 | Fail | Bake使用immutable snapshot、bounded worker、cancel/deadline与generation reject |
| G17 | Fail | artifact staging/validation/atomic apply/rollback/last-known-good通过 |
| G18 | Fail | clear/unlink/delete/undo/redo/save/reopen语义无混淆 |
| G19 | Fail | incremental dirty/cache/determinism/scenario/multi-scene有证据 |
| G20 | Fail | Reflection Probe asset/component/gizmo/selection/capture产品可达 |
| G21 | Fail | Probe capture/convolution/register经真实Runtime路径完成且可取消 |
| G22 | Fail | PostProcess profile/volume增删改、override、Undo和兼容性通过 |
| G23 | Fail | effective stack逐字段显示来源、weight、priority、layer与camera |
| G24 | Fail | preview session与Runtime generation一致，stale reply不覆盖新状态 |
| G25 | Fail | debug mode registry按viewport保存并在disable/reload/close时释放 |
| G26 | Fail | diagnostics有severity、source、dedupe、retention、export与Console路由 |
| G27 | Fail | 三份workspace所有route有真实domain effect或明确Unavailable |
| G28 | Fail | 固定frame/timing/count/profile/queued成功字符串从production删除 |
| G29 | Fail | keyboard/focus/a11y/high-DPI/multi-window/layout restore不破坏工作流 |
| G30 | Fail | fault、cancel race、device loss、runtime restart和8小时soak无泄漏 |
| G31 | Fail | 大graph/历史/viewport/Bake/Probe/Volume规模报告含p50/p95/p99/RSS/drop |
| G32 | Fail | 同质量、同场景、同硬件、冻结revision的Unreal/Unity/Godot/Fyrox对照通过 |

## 14. 完成定义

Rendering Editor只有同时满足以下条件，才能从“manifest可见的静态骨架”升级为工程级产品：

1. 默认App实际加载RenderingEditorProvider，15项feature的声明、链接、readiness与生命周期一致。
2. Render Pipeline和Post Process由transactional document持有，control只投影状态。
3. Frame Debugger完整消费Runtime graph/profile/capture，所有结果绑定generation并有明确预算。
4. Capture、GPU profiler和RenderDoc有真实设备边界、状态机、artifact与typed failure。
5. Lighting/Probe使用可取消job、immutable input、staging artifact、atomic apply与last-known-good。
6. Post Process profile/volume authoring与Runtime evaluator共享唯一schema，preview有真实ack。
7. Workbench不再包含固定业务样例或虚假成功；Unavailable、Pending、Failed、Canceled、Stale与Succeeded语义可区分。
8. correctness、fault、scale、soak与同质量参考对照证据通过后，才讨论性能或表现领先。

## 15. 本轮验证状态

- 完成Rendering plugin/editor、catalog/App、Workbench、Runtime边界与PBR Viewer capture的静态逐文件核对。
- 完成Unreal、Unity Graphics、Godot、Fyrox与Bevy所选源码的交叉对照及fingerprint冻结。
- 完成Editor22的5/60/12项current-source重判和32项Gate重判。
- 本轮不修改生产代码；只新增Editor96并更新索引/覆盖记录。
- 未运行动态产品或性能验证，因此所有需要运行证据的Gate保持Fail。
