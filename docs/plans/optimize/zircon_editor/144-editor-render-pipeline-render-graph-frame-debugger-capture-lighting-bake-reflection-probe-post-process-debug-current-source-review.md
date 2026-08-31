---
title: Editor Render Pipeline / Render Graph / Frame Debugger / Capture / Lighting Bake / Reflection Probe / Post Process / Debug Authoring 当前源码复审
category: zircon_editor
report_id: Editor144
review_date: 2026-08-26
baseline_head: 601472078e848164d2221967c55a77fea2452928
verification_head: a71cebf35c0be232ce734e483636d6c31c664ad0
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/96-editor-render-pipeline-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-debug-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99c-runtime-exposure-color-tonemap-lut-bloom-dof-motion-blur-ssr-output-transfer-terminal-composition-product-integration-current-source-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
related_code:
  - zircon_plugins/rendering/editor
  - zircon_plugins/rendering/features
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin
  - zircon_runtime/src/graphics/runtime
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Developer/RenderDocPlugin
  - dev/UnrealEngine/Engine/Plugins/Developer/DumpGPUServices
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore
  - dev/Graphics/Packages/com.unity.render-pipelines.core
  - dev/godot/editor/debugger
  - dev/godot/editor/plugins
  - dev/Fyrox/editor
  - dev/Fyrox/fyrox-impl/src/scene
  - dev/bevy/crates/bevy_render
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor144 · Rendering Authoring、Frame Debugger 与 Bake 当前源码复审

## 1. 当前结论

当前工作树仍没有可交付的 Rendering Editor 产品链。Rendering 根 Editor package 与 15 个 feature Editor crate 共 **66 个文件、1,182 行、16 次 capability 声明、0 个 asset/surface/operation/menu/graph/viewport/provider contribution**。`plugin.toml` 已声明 15 个 optional feature，根插件标为 `stable`、runtime 标为 `complete`，但 focused test 仍断言旧的 9 项全集，默认 first-party Editor catalog 只装配 Navigation 与 Neural，App 只是转发该 catalog。manifest、测试、默认产品装配与实际 provider 四套事实没有收敛。

Render Pipeline、Lighting Bake 与 Post Process 三份 Workbench ZUI 共 **689 行、81 个 node、57 条 route、0 个业务 provider**。它们固定显示 `Frame 1234`、`SceneColor -> BloomInput 1.84 ms`、`Windows DX12 30 fps GPU 6.24 ms`、`City_Block_A`、`87 assets`、`12 volumes`、`02:30`、`Global Stack`、`Cinematic Grade` 与 `EV +2.1`。Save/Compile/Diff/Preview/Bake/Apply 只写固定反馈或修改控件文本；generated bottom 只切换 panel/mode/route。界面因此把样例数据、命令已排队和结果成功伪装成工程事实。

Runtime 不是完全空白。`RenderFramework` 已有 frame capture、graphics debugger request/status、HDR capture；`CapturedFrame` 已携带 `capture_report`、`graph_dump` 与 `frame_profile_json`，`RenderFrameProfile` 已有 frame/pass/subsystem/memory/cache/degrade 数据。Editor viewport 却只校验 RGBA 尺寸和 generation，完全不读取后三类诊断载荷。Runtime post-process 也已有 typed settings/profile/volume/evaluator/pass graph/persistence，但 Editor 没有 profile/volume toolkit、serialized component list、override transaction 或 effective-stack inspector。

Bake 与 probe 暴露了更严重的产品真实性问题。`offline_bake_frame` 只按少量 mesh 与 directional-light intensity 派生最多 4 个 reflection probe，不产出 lightmap、atlas、UV、artifact、progress 或 cancel；`baked-lighting-composite` 和 rendering feature `post-process` executor 仍是 no-op。Reflection Probe Editor helper 不可从产品到达，且其 runtime capture 调用 `SceneRenderer::render_scene_color_hdr`，当前源码中不存在该方法，真实 HDR capture 已迁到 `RenderFramework::capture_scene_color_hdr`。因此这条选配 feature 在被选择构建时存在明确 API 断链。

Editor22/Editor96 的 canonical 结论维持：5 项 P0 为 **5 Open**，60 项 P1 为 **60 Open**，12 项 P2 为 **12 Open**，32 项资格门为 **32 Fail**。目标链路必须是：

```text
one manifest + linked runtime/editor providers + owner generation
  -> versioned RenderPipeline/PostProcess/Bake/Probe source documents
  -> shared runtime-owned validation/compiler/capability snapshot
  -> immutable revision-qualified job/capture request
  -> bounded execution + progress/cancel/deadline/device-loss handling
  -> atomic artifact or same-generation observation snapshot
  -> provider-backed graph/debug/bake/probe/post-process projections
  -> fault/scale/soak/visual/cross-engine qualification
```

本报告只做 review 与重构规划。MVP `00` 仍在进行且 F0-F5 被阻塞，Rendering Editor 属于后续分层里程碑；本轮未改 production source、未运行 Cargo，也未查询、轮询或等待协调器状态。

## 2. 物理范围、证据等级与 currentness

### 2.1 当前工作树扫描

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 结论 |
|---|---:|---|
| Rendering editor packages | **66 / 1,182 / 1,004 / 37,557 / 3 / 0** | 根 Editor package 与 15 个 feature crate，仍全部是 descriptor/capability shell |
| Workbench render product surface | **15 / 6,725 / 6,498 / 318,718 / 1 / 0** | 三份 ZUI、generated bottom、feedback/navigation/field 与 template binding |
| Manifest/catalog/App boundary | **6 / 1,144 / 1,028 / 40,445 / 5 / 0** | 15 项 manifest、9 项旧测试、默认 catalog/App 未装配 Rendering provider |
| Runtime capture/profile/bake bridge | **20 / 2,952 / 2,687 / 105,409 / 37 / 0** | 真实 capture/profile 底座与狭窄 offline bake，但 Editor 未消费 |
| Runtime bake/probe/post-process domain | **60 / 11,348 / 10,322 / 405,757 / 115 / 0** | typed post-process 存在；bake producer、probe product bridge 与部分 feature executor 不完整 |
| Selected reference engines | **22 / 13,152 / 11,244 / 494,610 / 1 / 0** | Unreal、Unity Graphics、Godot、Fyrox、Bevy 的选定同域路径 |

计数直接读取物理文件并包含工作树内已有修改。各组用于说明不同证据层，存在逻辑交集，不应相加为仓库唯一文件数。测试计数只说明静态/单元覆盖规模，不能证明 Editor 产品链、真实 GPU、设备丢失、长任务取消或 artifact durability。

### 2.2 Currentness 与在途修改

冻结基线为 `601472078e848164d2221967c55a77fea2452928`。当前 Workbench 路径已由旧报告中的平铺位置迁到 `modules/core/rendering` 与 `modules/extensions/rendering`，Render capture/profile 载荷也比早期报告更丰富；这些是需要保留的进展。它们没有新增 Rendering Editor provider、operation factory、document、job、capture consumer、bake artifact 或 probe selection bridge，因此不改变 canonical 状态。

本轮保留所有用户或其他 Session 的工作树修改，没有覆盖、格式化或回退 production source。`source_recheck_required: true` 表示实施前必须重新冻结 manifest/catalog/App、16 个 Editor package、三份 ZUI、feedback/bottom binding、Runtime capture/profile API、offline bake、probe capture 与 post-process schema。

### 2.3 动态证据边界

按用户要求本轮只做 review，没有运行 Editor/App、真实 GPU capture、RenderDoc、pipeline compile、lightmap bake、probe convolution、post-process save/reopen、device-loss、fault/scale/soak/profile 或同质量跨引擎 benchmark。现有 focused test 主要断言 manifest 字段、固定控件/路由和静态行为，不能关闭任何 Gate。

仓内存在 Runtime frame capture 与 RHI 相关 failure handoff；本轮只把它们作为动态验证边界，不等待其状态，也不把 sibling owner 的阻塞复制成 Editor finding。Editor144 只裁决当前源码可静态证明的作者链和产品桥。

## 3. 当前源码纵向事实

### 3.1 Manifest、package、catalog 与默认 App

1. `zircon_plugins/rendering/plugin.toml` 声明 15 个 optional feature，默认启用 post-process、SSAO、reflection probes 与 baked lighting；根 runtime 状态是 complete，maturity 是 stable。
2. 根 Editor package 与 15 个 feature Editor crate 只有 descriptor 与 `.with_capability(...)`；没有 asset、surface、operation、menu、graph、viewport、resource、controller、provider 或 `register_editor_extensions`。
3. focused test 仍硬编码 9 项 optional feature，遗漏 volumetric fog、OIT、light cookies、irradiance volumes、planar reflections 与 subsurface scattering；它同时期待 `editor.extension.rendering_authoring`，形成旧测试权威。
4. 默认 `first_party_editor_catalog` 只链接 Navigation 和 Neural；`zircon_app::entry::first_party_editor_plugins` 只委托 catalog，App Cargo 也没有 Rendering Editor feature。用户无法从默认产品获得与 manifest 承诺相匹配的 provider。
5. outside rendering packages 的源码引用只剩 lockfile 与测试，16 个 Editor crate 没有产品 consumer。capability 名称不能代替可到达的服务、资源和生命周期。

### 3.2 Workbench、feedback 与 generated bottom

1. Render workspace 是 229 行/27 nodes/19 routes；Lighting 是 230/27/19；Post 是 230/27/19，三者 provider 数均为 0。
2. Render 固定展示 `MainPipeline.rp`、Frame 1234、1.84 ms、6.24 ms、R11G11B10_FLOAT 与 Windows DX12；这些值不来自 `CapturedFrame` 或 `RenderFrameProfile`。
3. Lighting 固定展示 City_Block_A、87 assets、4 warnings、12 volumes、Interior_Lab 6 texels 与 02:30；当前 offline bake 不生产一个 lightmap，这些值没有领域来源。
4. Post 固定展示 Global Stack、Cinematic Grade、Filmic、+0.4、LUT_CityWarm、33 cube 与 Interior EV +2.1；Editor 没有读取 scene post-process component/profile/volume。
5. `module_command_feedback.rs` 直接写入 profile persisted、graph compile queued/compiled、pass changes compared、frame queued 与 6.24 ms compiled；`extension_module_feedback.rs` 直接写入 preview/bake/apply queued。
6. `module_field_edit.rs` 只改 retained control 的 `value/value_text`；没有 transaction、document revision、schema validation、dirty/savepoint、runtime acknowledgement 或 stale fence。
7. generated bottom 只保存 selected route/module/panel/mode。57 条 route 没有 domain provider、subscription、operation、job 或 bounded stream。

### 3.3 Runtime capture/profile 底座与 Editor 消费断层

1. `RenderFramework` 已暴露 graphics debugger request/status、frame capture/poll 与 viewport HDR capture；WGPU debugger state 能追踪 pending/queued/active/last frame/error，并限制一次 capture 的 frame count。
2. 默认 trait 的 debugger request 在不支持时仍可返回 `Ok(())`，而 status 默认 unavailable；这会允许上层把 unsupported request 误判成已接受，应由 Runtime90 修正为明确 capability/receipt。
3. `CapturedFrame` 已携带 RGBA、generation、capture report、graph dump 和 frame profile JSON；viewport capture 会将 compiled pipeline dump 与同 generation 的 profile 附着到 frame，并支持迟到 GPU timing 回填。
4. profile 已有 CPU submit、GPU frame/time status、pass executor/budget/cpu/gpu/pipeline stats/draw/instance/state/upload/dispatch、subsystem、mesh submission、memory、cache、warning 和 degrade step。
5. Editor viewport poll 只验证 width/height/RGBA 并存储 latest generation；不解析 capture report、graph dump 或 profile JSON。现有 UI presenter GPU metrics 是 Editor chrome 绘制统计，不是 scene Render Graph debugger。
6. Editor 没有调用 graphics debugger request/status，因此即使 Runtime bridge 存在，也没有产品命令、状态机、artifact、open/reveal 或 failure presentation。

### 3.4 Render Pipeline 与 Render Graph 控制面

1. `MainPipeline.rp` 只是输入框文本，没有资产类型、schema version、stable pass/resource ID、document revision、save/reopen/recovery 或 source control。
2. Compile 按钮没有调用 Runtime89 compiler，也不提交 `OperationCommandFactory` 或 background job；“compiled”只是固定字符串。
3. graph dump 已可从 captured frame 获得，但没有 versioned parser、unknown-field preservation、schema compatibility、selection model 或 source mapping。
4. UI 没有投影 culled/async/merged/parallel/queue、producer/consumer、resource lifetime/alias/barrier、descriptor/format/extent/mip/sample/residency。
5. Runtime89 当前仍有 generated pass name collision、SparseReserved materialization 与 storage texture type/default 等 owner 缺陷；Editor 必须显示真实 compiler diagnostic，不能在本层复制或掩盖。

### 3.5 Lighting Bake 与 artifact 生命周期

1. `offline_bake_frame` 只生成 reflection probe DTO，默认最多 4 个，并以 mesh 与 directional-light intensity 派生；没有 UV2、lightmap texture、atlas、irradiance volume artifact 或 scene registration。
2. 该函数在 production 中没有 caller，只有测试。Editor/App/rendering Editor crate 也没有 `LightmapBake` 或 offline bake 调用。
3. baked-lighting runtime feature 声明 `baked-lighting-composite`，executor 是 `noop_render_executor`。UI 的 87 lightmaps/02:30/ready 因而不是“尚未精确”，而是语义不成立。
4. 没有 immutable input、source/settings/backend key、preflight、admission、worker、stage progress、cancel acknowledgement、deadline、stale reject、staging artifact、atomic publish 或 rollback。
5. 没有 clear/unlink/delete/save/reopen、多 scene/scenario/streaming ownership、incremental dirty/cache/determinism 与 last-known-good。

### 3.6 Reflection Probe

1. Reflection Probe Editor crate 仍只注册 capability。仓内没有 asset toolkit、component drawer、scene gizmo、selection command、property operation 或 capture action contribution。
2. `capture/trigger.rs` 接收 mutable `SceneRenderer`、asset manager 与 scene snapshot，直接序列化 request/placement JSON；没有 job ID、progress、cancel、deadline、revision、selection identity、transaction 或 stale handling。
3. helper 没有 production consumer。即使手工调用，下游 `execute.rs` 仍调用已不存在的 `SceneRenderer::render_scene_color_hdr`；当前 HDR API 位于 `RenderFramework`。
4. WGPU acceptance test 被 `#[ignore]`，不能覆盖默认产品、真实 provider reachability、device loss、shutdown 或 artifact registration。
5. Runtime96 拥有 probe capture/convolution/streaming 语义修复；Editor 只应通过 stable request/result contract 编排，不应直接借用 mutable renderer 形成第二 authority。

### 3.7 Post Process 与 debug authoring

1. Runtime 已有 profile/settings/volume、scene component、project IO、volume evaluator、component registry、pass graph、compile integration 与 runtime execution，应作为 Editor 唯一 schema/validator 来源。
2. Editor 没有任何 `PostProcessVolumeComponent`/`PostProcessSettingsComponent` consumer；没有 profile toolkit、component list、override checkbox、add/remove/reorder、multi-edit、Undo 或 save。
3. Apply 只改变文本。没有 global/camera/local volume、blend distance/weight/priority/layer mask、effective stack、field provenance 或 runtime preview generation。
4. rendering feature `post-process` executor 仍 no-op；Runtime99c 还记录 HDR/persistence/double-execution 风险。Editor 必须以 capability snapshot 和 terminal acknowledgement fail-close。
5. wireframe、overdraw、light complexity、shadow/probe/volume/resource debug mode 没有 registry、per-viewport owner、release/reload 语义；warning 也没有 provider/severity/source/dedupe/retention/export/Console 路由。

## 4. 参考引擎差异与采用边界

| 参考 | 已检查的工程事实 | Zircon 采用边界 |
|---|---|---|
| Unreal RenderDoc / Render Capture | 可发现 Capture Frame 命令；current viewport/all activity、delayed、seconds/frames、multi-frame、callstack/resource/initial state、artifact directory、notification、modular provider 与 frame boundary | 建立共享 capture service、backend capability 与完整状态机；不把 RenderDoc API 直接扩散进 Workbench |
| Unreal RenderGraphTrace / DumpGPU | graph/pass/buffer/texture/scope event、pass timing、resource lifetime/external/extracted/culled/transient/cache；pass/resource filter、descriptor/binary、parameters/timing JSON、staging cap、async stream/OOM 与 viewer/upload | Runtime 产出 versioned snapshot/artifact，Editor 做兼容 parser 与虚拟化 projection；未知字段必须保留并诊断 |
| Unreal GPULightmass | Build menu、details settings、Start、Bake What You See、Save And Stop、Cancel、progress/status/notification 与 world lifecycle | bake 是长事务，不是按钮字符串；Zircon 需更严格 immutable input、deadline、generation 与 atomic artifact |
| Unreal Reflection/Post Process customization | selected component details、capture update；分组 property、override/conditional visibility、config 与 notification | 由 Runtime typed schema 驱动 drawer/toolkit/operation；不复制 UObject/Slate 所有权 |
| Unity Graphics RenderGraphViewer | local/remote debug session、schema compatibility、graph/execution/camera wait、pass/resource filter、culled/async/merged/sync、read/write/lifetime grid、selection/hover 与 persisted view | 作为 frame debugger UX 主参考；Zircon 还需补 owner generation、bounded snapshot 和 fail-closed disconnect |
| Unity APV/Volume Editors | bake driver 有 start/step/progress/stage/in-progress；volume profile/component 支持 serialized data、add/remove、override、Undo、copy/paste、SetDirty | 复用 staged job 与 serialized component workflow，不引入 Unity 资产模型 |
| Godot visual profiler / bake / gizmo | 有界历史、CPU/GPU plot、cursor、start/stop；LightmapGI 真实 bake/save/error/cancel；probe gizmo typed handle 与 undo/redo | 作为轻量产品下限；Zircon 的 remote、generation、artifact 与设备失败合同应更强 |
| Fyrox probe/lightmap | probe selection、preview、interaction mode、property command；lightmap 有 stage/progress/cancel、UV、texture save/artifact | Rust 实现参考；其 blocking bake API 只是下限，不能成为目标调度模型 |
| Bevy RenderDiagnosticsPlugin | 在 RenderGraph schedule 安装 CPU/GPU pass diagnostic 与 pipeline stats，可经 diagnostics store/log/Tracy 消费 | 采用运行时诊断生产与 UI 消费解耦；Bevy 没有同级完整 Editor，不能为产品缺失背书 |

Unreal/Unity 是 Render Graph、capture、bake 与 post-process authoring 的一等产品参考，Godot/Fyrox/Bevy 用于检验更轻量架构的最低工程合同。Zircon 要宣称优于 Unreal，必须用相同场景、质量、硬件、驱动、warm-up、capture overhead 和冻结 revision 的 correctness/performance 数据证明，不能从 API 数量或静态面板推断。

## 5. Owner 与架构边界

1. `zircon_runtime` 唯一拥有 Render Graph/RHI/capture/profile、post-process evaluator、bake/probe algorithm、artifact 与 capability schema；Editor 禁止复制 compiler、format rule 或 runtime resolved value。
2. `zircon_plugins/rendering` 拥有 feature manifest、runtime/editor package 配对、provider factory、resource 与 owner-generation lifecycle；Plugin04 裁决 metadata-only/no-op feature 的交付真实性。
3. `zircon_editor` 拥有 source document/toolkit、transaction adapter、operation/job controller、capture/debug reader、selection/gizmo、projection 与 diagnostic presentation。
4. `zircon_app` 只按 project/profile 组合已链接 package，并验证 runtime/editor/resources/provider 全集；不拥有 Rendering 语义。
5. Workbench 只导航和投影同一 document/session/job authority；不得通过 selected checkbox、固定文本或 local control value 推断业务状态。
6. capture/bake/probe/post-process 所有异步结果必须绑定 project/session/world/view/document/source/artifact/plugin generation，迟到结果不得覆盖新状态。

## 6. Editor22 父 P0 当前重判

| ID | 状态 | 当前证据与硬切要求 |
|---|---|---|
| RENED-P0-01 | Open | stable/complete、15 项 manifest 与 capability shell 继续制造 Rendering Editor 已可用的承诺；测试停在 9 项，默认 App 没有 provider。统一 manifest 解析、generated descriptor、link set、provider readiness 和 fail-closed Unavailable。 |
| RENED-P0-02 | Open | Render Pipeline、Frame Debugger 与 bottom panel 仍以固定 frame/pass/timing/resource/profile 作为第二 authority。删除 fixture authority，只消费同 generation 的 Runtime snapshot/receipt。 |
| RENED-P0-03 | Open | Lighting Bake 仍显示 ready/queued，却没有 lightmap producer、job/cancel/artifact/atomic commit；当前 offline bake 只产生少量 probe DTO。接 Editor09 与 Runtime97 的真实长事务。 |
| RENED-P0-04 | Open | Post Process Apply 仍只改文本/control；typed Runtime volume evaluator 的存在没有形成 authoring、persistence、undo、preview acknowledgement。建立 transactional profile/volume document。 |
| RENED-P0-05 | Open | Probe helper 仍不可达，并调用已删除的 renderer method。先由 Runtime96 修复 contract，再通过 selection operation、bounded job 与 artifact transaction 接入。 |

## 7. P1 当前源码差距账本

### 7.1 装配、插件与生命周期

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RENED-P1-01 | Open | feature 清单存在 15/9 漂移；generated descriptor、测试、UI 与 link set 必须由同一 manifest model 驱动并做双向完整性验证。 |
| RENED-P1-02 | Open | 16 个 capability 没有 owner lease/load/unload/reload/shutdown；每项贡献绑定 plugin generation 并可原子撤销。 |
| RENED-P1-03 | Open | 默认 App 不链接 Rendering Editor；project enablement 原子装配 runtime/editor/resources/provider/controller。 |
| RENED-P1-04 | Open | descriptor capability 可在零 provider/executor 时显示 ready；readiness 验证 factory、backend、resources 与 product reachability。 |
| RENED-P1-05 | Open | 15 个 feature Editor crate 重复 descriptor shell；收敛共享注册框架，同时保留 typed feature owner 与 contract。 |
| RENED-P1-06 | Open | disable/reload 没有 panel、capture reader、job、preview、runtime consumer 的 drain/revoke/close 顺序。 |
| RENED-P1-07 | Open | crates 在 plugin workspace 但 target Editor host 无产品 lane；补默认链接、feature matrix 与 missing-dependency fail-close。 |
| RENED-P1-08 | Open | capability ID 无 version/schema/compatibility；增加 contract version、producer generation 和 diagnostic。 |

### 7.2 Render Pipeline document 与 compiler 控制面

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RENED-P1-09 | Open | `MainPipeline.rp` 只是文本；建立 versioned source、document revision、dirty/save/reopen/recovery。 |
| RENED-P1-10 | Open | captured graph dump 已存在但 Editor 不消费；建立兼容 parser/view model，保留未知字段并报告 schema。 |
| RENED-P1-11 | Open | Compile 没有 source hash、target、feature set、shader/material artifact key；建立 typed request/result/provenance。 |
| RENED-P1-12 | Open | feedback 不区分 queued/running/succeeded/failed/canceled/stale；使用 Editor09 terminal job state。 |
| RENED-P1-13 | Open | 无 cycle、missing producer、format/usage/queue compatibility 的 pipeline validation 和 source location。 |
| RENED-P1-14 | Open | 无 last-known-good/staging；compile 失败不得替换 Runtime 当前 generation。 |
| RENED-P1-15 | Open | 无 dependency/recompile invalidation；接 shader/material artifact revision 与 feature capability generation。 |
| RENED-P1-16 | Open | 无 undo/diff/merge/external conflict；统一 Editor02/63 document transaction。 |

### 7.3 Render Graph 与 Frame Debugger

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RENED-P1-17 | Open | 无真实 pass tree、culled/async/merged/parallel/queue；从 Runtime89 versioned snapshot 投影。 |
| RENED-P1-18 | Open | 无 resource read/write、first/last use、alias、acquire/discard 与 barrier。 |
| RENED-P1-19 | Open | 无 texture/buffer descriptor、format、extent、mip/layer/sample、size 与 residency。 |
| RENED-P1-20 | Open | 无 pass-resource 双向选择、dependency、producer/consumer 和 source navigation。 |
| RENED-P1-21 | Open | 无 graph revision/hash、frame/pipeline/capture generation 与 source provenance。 |
| RENED-P1-22 | Open | 无 local/remote session、disconnect、incompatible schema、stale snapshot 与 reset。 |
| RENED-P1-23 | Open | Runtime 有 timing status，UI 仍固定 1.84/6.24 ms；逐状态显示 Disabled/Pending/Unavailable/Measured/Stale。 |
| RENED-P1-24 | Open | 无 bounded frame history、cursor、search/filter/sort/bookmark 与 retention policy。 |
| RENED-P1-25 | Open | 无 CPU submit、GPU pass、subsystem、memory、stats 与 latency 的统一时间线。 |
| RENED-P1-26 | Open | 无 snapshot entries/bytes/time budget、partial/drop/age/coverage 和 observer cost。 |

### 7.4 Capture、GPU profiler 与外部调试器

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RENED-P1-27 | Open | RenderDoc bridge 仍未形成共享 capture service；按 D3D11/D3D12/Vulkan/WGPU backend 暴露真实 capability。 |
| RENED-P1-28 | Open | Editor 只消费 RGBA；必须解析 `CapturedFrame` graph/profile/report 并严格匹配 generation。 |
| RENED-P1-29 | Open | 无 next-frame/delayed/multiframe/current viewport/all activity capture 状态机。 |
| RENED-P1-30 | Open | 无 admission、submit boundary、timeout/cancel、device loss、unsupported/NullRHI fail-close；默认 `Ok(())` 必须收紧。 |
| RENED-P1-31 | Open | 无 artifact 命名、目录、atomic finalize、retention、size、open/reveal/export 与 corruption validation。 |
| RENED-P1-32 | Open | 无 texture/buffer/pass dump filter、staging cap、OOM warning 与 async streaming。 |
| RENED-P1-33 | Open | 无 capture schema/tool/adapter/backend/build/source revision provenance。 |
| RENED-P1-34 | Open | 无 GPU profiler result tree、JSON schema、one-shot lifetime 与 end-frame completion 产品测试。 |

### 7.5 Lighting Bake、Lightmap 与 Irradiance Volume

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RENED-P1-35 | Open | 固定 City_Block_A/Interior_Lab 不是 projection；枚举真实 scene、baking set、scenario 与 dependency。 |
| RENED-P1-36 | Open | 无 world/RHI/hardware/ray tracing/project/backend capability 检查与精确禁用原因。 |
| RENED-P1-37 | Open | 无 immutable input/source/settings/quality/platform/backend key。 |
| RENED-P1-38 | Open | 无 UV2/mesh/material/light/path/atlas/texture/empty source preflight。 |
| RENED-P1-39 | Open | 无 worker/admission/resource budget/priority/dependency DAG/stage progress。 |
| RENED-P1-40 | Open | 无 cancel acknowledgement、deadline、source-mutation stale reject 与 shutdown barrier。 |
| RENED-P1-41 | Open | 无 lightmap/probe artifact staging、hash/format/atlas validation、atomic publication 与 rollback。 |
| RENED-P1-42 | Open | 无 apply/clear/unlink/delete、undo/redo、save/reopen 与 reference migration。 |
| RENED-P1-43 | Open | warning/error 不能定位 scene object、mesh、material、light、volume 和 generated file。 |
| RENED-P1-44 | Open | 无 incremental dirty set、cache hit/miss/reuse、determinism 与 last-known-good report。 |
| RENED-P1-45 | Open | 无 preview、baked/unbaked diff、texel density、overlap 与 coverage overlay。 |
| RENED-P1-46 | Open | 无 multi-scene/baking set/scenario/streaming cell 与 cross-scene ownership。 |

### 7.6 Reflection Probe、Post Process 与 Rendering Debug

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RENED-P1-47 | Open | 有 probe DTO/helper 但无 asset/toolkit、component drawer、gizmo、selection operation。 |
| RENED-P1-48 | Open | probe 无 position/shape/influence/parallax/priority/resolution/refresh policy 完整 authoring。 |
| RENED-P1-49 | Open | 无 capture/convolution/SH/PMREM stage、progress、artifact provenance 与 failure presentation。 |
| RENED-P1-50 | Open | 无 probe grid/irradiance volume placement、density、coverage、overlap 与 memory budget。 |
| RENED-P1-51 | Open | Runtime 有 typed evaluator，Editor 无 PostProcess Profile/Volume toolkit 和 serialized component list。 |
| RENED-P1-52 | Open | 无 add/remove/reorder/enable/override、multi-edit、undo 与 pipeline compatibility filter。 |
| RENED-P1-53 | Open | 无 global/camera/local volume、blend distance/weight/priority/layer 与 effective-stack inspection。 |
| RENED-P1-54 | Open | 无 preview camera/session/generation、before-after、effect isolate 与 runtime acknowledgement。 |
| RENED-P1-55 | Open | 无 exposure/histogram/color/LUT/bloom/DoF/motion blur/SSR typed validator 和 unit/range diagnostic。 |
| RENED-P1-56 | Open | 无 source profile、override source、blend contribution 与 final resolved value field trace。 |
| RENED-P1-57 | Open | 无 wireframe/overdraw/light complexity/shadow/probe/volume/resource debug registry 与 per-viewport state。 |
| RENED-P1-58 | Open | 无 diagnostic provider、severity/source navigation/dedupe/retention/export/Notification/Console route。 |
| RENED-P1-59 | Open | feature/debug UI 不从同一 capability snapshot 生成，6 项 feature 仍可在测试 authority 中消失。 |
| RENED-P1-60 | Open | 无 scale/soak/profile/correctness/visual golden 和 same-quality reference benchmark，不能声明领先。 |

## 8. P2 高阶能力

| ID | 状态 | 目标 |
|---|---|---|
| RENED-P2-01 | Open | 远程设备/主机、多 adapter 与多 viewport capture session。 |
| RENED-P2-02 | Open | 多人共享 capture、annotation、bookmark 与 source revision 锁定。 |
| RENED-P2-03 | Open | Render Graph 结构、frame timing 与 memory regression diff。 |
| RENED-P2-04 | Open | pass/resource breakpoint、conditional capture 与自动触发规则。 |
| RENED-P2-05 | Open | 超大 graph virtualization、paging、LOD、search index 与 progressive load。 |
| RENED-P2-06 | Open | 跨平台 capture artifact conversion、compatibility matrix 与 offline viewer。 |
| RENED-P2-07 | Open | distributed bake、worker capability/lease、checkpoint/resume 与 merge。 |
| RENED-P2-08 | Open | large-world cell/scenario incremental bake、streaming budget 与 batch validation。 |
| RENED-P2-09 | Open | probe auto layout、quality heuristic、overlap optimization 与 platform budget suggestion。 |
| RENED-P2-10 | Open | Post Process profile inheritance、variant、batch diff 与 migration。 |
| RENED-P2-11 | Open | scriptable capture/bake/validation command 与 stable machine-readable result。 |
| RENED-P2-12 | Open | long-term trend、regression threshold、automatic bisect input 与 reproducible evidence pack。 |

## 9. 目标架构

```text
RenderingPluginManifestSnapshot
  -> RuntimeFeatureProviderSet + EditorFeatureProviderSet + ResourceBundleSet
  -> RenderingCapabilitySnapshot(owner_generation, schema_versions, reasons)

AuthoringDocument<RenderPipelineSource | PostProcessProfile | BakeSet | ProbeAsset>
  -> EditorTransaction / OperationCommandFactory
  -> Runtime-owned validator/compiler
  -> ArtifactKey(source_hash, target, feature_set, backend, dependency_generations)
  -> staging -> validation -> atomic publish -> last-known-good

CaptureOrBakeRequest(qualified context, immutable revision, budget, deadline)
  -> admission -> queued -> running(stage/progress)
  -> succeeded(artifact/snapshot receipt)
     | failed(typed diagnostic)
     | canceled(acknowledged)
     | stale(rejected generation)

ObservationSnapshot
  -> graph + resources + profile + capture provenance
  -> bounded Editor reader lease
  -> virtualized graph/timeline/diagnostics projection
```

必须使用同一 capability snapshot 决定 menu、workspace、operation admission、debug mode 与 runtime execution。document/control/job/artifact/snapshot 各有单一 authority；UI 文本只从 typed state 派生，不保存业务真值。

## 10. 依赖顺序与重构里程碑

| 阶段 | Owner | 必须交付 | 退出条件 |
|---|---|---|---|
| R0 Truth hard cut | Plugin04 + App + Editor50 | 单一 15 项 manifest model、linked-set validation、provider readiness、Unavailable reason；删除固定成功字符串 | G01/G02/G27/G28 |
| R1 Runtime contracts | Runtime89/90/96/97/99c | versioned graph/profile/capture schema、fail-closed debugger receipt、probe API、真实 bake/post executor contract | Editor 不再猜测 Runtime 状态 |
| R2 Source documents | Editor02/63 + Rendering Editor | Pipeline/PostProcess/BakeSet/Probe typed document、transaction、save/reopen/recovery、external conflict | G03/G22 |
| R3 Operation and jobs | Editor09/47/50 | compile/capture/bake/probe/preview factory、qualified context、admission、progress/cancel/deadline/stale/shutdown | G04/G10/G11/G14-G18/G21/G24 |
| R4 Debug products | Rendering Editor | graph parser/view model、pass-resource selection、timeline/history、artifact browser、diagnostic routing、debug registry | G05-G09/G12/G13/G23/G25/G26 |
| R5 Authoring products | Rendering Editor | bake preflight/overlay/scenario、probe gizmo/capture、volume component/effective stack | G15-G24 |
| R6 Qualification | Runtime + Editor + App | fault/scale/soak/visual golden、real backend matrix、same-quality cross-engine benchmark | G29-G32 |

MVP `00` 与 F0-F5 没有通过前，不应并行扩张高级 Rendering UI。可以先实施 R0 的 fail-close、schema 设计与测试基建，但不能把 capability shell、no-op executor 或 fixture Workbench 重新标为交付完成。

## 11. G01-G32 Editor 资格门

| Gate | 状态 | 通过条件 |
|---|---|---|
| G01 | Fail | manifest 15 项与 generated descriptor、测试、默认 App 和 provider 全集一致。 |
| G02 | Fail | 每个可见 capability 有 owner、factory/provider、readiness、reload 与 shutdown。 |
| G03 | Fail | Render Pipeline create/open/edit/save/reopen/undo/recover 不丢失。 |
| G04 | Fail | Compile 经真实 source/artifact/backend 完成且失败保留 last-known-good。 |
| G05 | Fail | Frame Debugger pass/resource/alias/barrier/queue 来自同一 generation。 |
| G06 | Fail | culled/async/merged/parallel 与 Runtime89 execution fact 一致。 |
| G07 | Fail | GPU timing 状态不被伪装成数字，pending/stale/disabled 可解释。 |
| G08 | Fail | frame history 有界且 drop/age/coverage/observer cost 可见。 |
| G09 | Fail | capture frame/profile/graph 严格匹配 session/view/frame generation。 |
| G10 | Fail | RenderDoc 按真实 backend 选择，不支持/NullRHI/device loss 时 fail-close。 |
| G11 | Fail | delayed/multiframe/current viewport/all activity 状态机完成。 |
| G12 | Fail | capture artifact 原子落盘、版本化、可验证、可保留与可清理。 |
| G13 | Fail | GPU profiler tree/JSON/result lifetime/end-frame completion 有测试。 |
| G14 | Fail | Lighting Bake 在项目/RHI/硬件/backend 不支持时给出精确原因。 |
| G15 | Fail | UV/mesh/material/light/path/atlas/texture 输入错误在提交前定位。 |
| G16 | Fail | Bake 使用 immutable snapshot、bounded worker、cancel/deadline/generation reject。 |
| G17 | Fail | artifact staging/validation/atomic apply/rollback/last-known-good 通过。 |
| G18 | Fail | clear/unlink/delete/undo/redo/save/reopen 语义无混淆。 |
| G19 | Fail | incremental dirty/cache/determinism/scenario/multi-scene 有证据。 |
| G20 | Fail | Reflection Probe asset/component/gizmo/selection/capture 产品可达。 |
| G21 | Fail | Probe capture/convolution/register 经真实 Runtime path 完成且可取消。 |
| G22 | Fail | PostProcess profile/volume CRUD、override、Undo 与 compatibility 通过。 |
| G23 | Fail | effective stack 逐字段显示 source、weight、priority、layer 与 camera。 |
| G24 | Fail | preview session 与 Runtime generation 一致，stale reply 不覆盖新状态。 |
| G25 | Fail | debug mode registry 按 viewport 保存并在 disable/reload/close 时释放。 |
| G26 | Fail | diagnostics 有 severity/source/dedupe/retention/export 与 Console route。 |
| G27 | Fail | 三份 workspace 的 57 条 route 有真实 domain effect 或明确 Unavailable。 |
| G28 | Fail | 固定 frame/timing/count/profile/queued-success 字符串从 production 删除。 |
| G29 | Fail | keyboard/focus/a11y/high-DPI/multi-window/layout restore 不破坏工作流。 |
| G30 | Fail | fault/cancel race/device loss/runtime restart/8-hour soak 无泄漏。 |
| G31 | Fail | 大 graph/history/viewport/Bake/Probe/Volume 报告含 p50/p95/p99/RSS/drop。 |
| G32 | Fail | 同质量/场景/硬件/冻结 revision 的 Unreal/Unity/Godot/Fyrox 对照通过。 |

## 12. 验证边界与最终裁决

| Canonical 范围 | 当前状态 | 本轮裁决 |
|---|---:|---|
| 5 项 P0 | **5 Open** | 无 provider、固定第二 authority、假 Bake、假 Apply 与 probe API 断链均仍存在 |
| 60 项 P1 | **60 Open** | Runtime 底座比旧报告更真实，但没有一条形成默认 Editor 完整产品链 |
| 12 项 P2 | **12 Open** | 未发现 remote/collaboration/diff/distributed/large-world/scriptable/trend 产品证据 |
| 32 项 Gate | **32 Fail** | 静态源码与现有测试不能关闭任何资格门 |

当前应将 Rendering Editor 定义为“manifest 可见、Runtime 有部分真实底座、Editor 产品桥仍未建立”的原型阶段。优先级不是继续增加更多静态面板，而是先完成 R0 truth hard cut：默认 App 要么装配可执行 provider，要么明确 Unavailable；所有固定 frame、timing、count、queued/succeeded 文案必须退出 production authority。

其后按 Runtime contract -> source document -> operation/job -> observation/debug projection -> authoring product 的顺序推进。只有 32 个 Gate 全部通过，并完成真实硬件 correctness、fault、scale、soak、visual golden 和同质量 benchmark，才能声称该子系统达到工程级，更不能提前声称性能或表现优于 Unreal。
