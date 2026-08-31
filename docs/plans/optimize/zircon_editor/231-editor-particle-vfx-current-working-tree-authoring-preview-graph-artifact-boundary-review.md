---
related_code:
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/src/plugin.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/particles/editor/authoring.zui
  - zircon_plugins/particles/editor/particle_system.drawer.zui
  - zircon_plugins/particles/editor/preview.zui
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/rendering/features/vfx_graph/editor/src/plugin.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
tests:
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/optional_features.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/230-editor-animation-current-working-tree-document-graph-timeline-preview-montage-ik-skinning-product-boundary-review.md
  - docs/plans/optimize/zircon_runtime/171-runtime-particle-vfx-current-working-tree-world-authority-gpu-graph-renderer-scalability-editor-boundary-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Public/NiagaraComponent.h
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraWorldManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraScalabilityManager.cpp
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Data/VFXDataParticle.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXGraphCompiledData.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXCodeGenerator.cs
  - dev/godot/scene/3d/gpu_particles_3d.cpp
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_render/src/render_asset.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Particle / VFX Authoring、Preview、Graph 与 Artifact 当前源码工程化差距

## 1. 结论

Particles editor plugin 已注册 authoring drawer、Particle Preview view、inspector customization、asset type、CPU Sprite creation template，以及 create/add/open/emitter/module/curve/validate/play/pause/stop/rewind/warmup 等 operation descriptor。三个 ZUI 也有稳定 control id 和 layout 骨架，测试能证明这些 descriptor 的注册形状。

这些是入口合同，不是可交付的编辑器产品。`authoring.rs` 只产生 descriptor/batch，没有 operation factory、document mutation、transaction、compiler job、artifact install 或 runtime receipt。`authoring.zui`、`particle_system.drawer.zui` 和 `preview.zui` 的核心区域大多是 `Space`；它们没有 emitter/module/curve/diagnostic/viewport/transport 的数据 binding。插件测试明确把所有 operation menu 标为 disabled，这证明当前菜单故意不可执行。

VFX Graph editor feature 只有 descriptor/capability，未注册 graph document、node palette、compiler、preview 或 artifact consumer。Core Workbench 的 VFX 页面和 feedback 文本可产生 “compile queued”“simulation running”“system saved” 等展示，但没有和 `VfxGraphCompileReport`、Particles runtime 或真实 PreviewWorld 相连。runtime VFX Graph 又是固定 `[1,1,1]` dispatch 和两个 no-op executor，因此编辑器不能把这些文案当作结果。

本轮是 review-only，未修改生产代码、ZUI、Cargo 或测试，也未运行 Editor/Cargo/WGPU。由于 VFX/Particles 仍 optional/Partial，本轮不新增 P0；新增 **18 项 P1、8 项 P2、18 项资格门**。Editor15 中菜单/compile/simulate/preview 假成功的既有 P0 继续开放，不重复计数。

## 2. 审查边界与统计

逐读顺序为 `editor plugin/extension -> authoring batch/operation descriptor -> ZUI -> test assertions -> core workbench/retained callback -> runtime component/asset/service -> VFX feature -> catalog/optional feature tests`。参考 Unreal Niagara、Unity VFX Graph、Godot、Fyrox 与 Bevy 的 document/compiler/preview/runtime-world ownership。当前插件切片冻结如下：

| 范围 | 文件 | 行数 | bytes | test attributes | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Particles editor + dist（Rust/TOML/ZUI） | 12 | 709 | 31,183 | 3 | 0 | `018a766b230305578548d770066423b7ba3080b33225684207db11c02e37a067` |
| Particles runtime（关联核对） | 44 | 7,336 | 280,249 | 45 | 1 | `eb5971f9a2e93d48fccb81aeb4068f399139484e1958a76418613b66ecbb9897` |

指纹为排序路径的逐文件 SHA-256 manifest。实施前必须重取指纹；工作树的其他会话修改保持不动。

## 3. 现有可保留基础

| 项目 | 当前事实 | 后续边界 |
|---|---|---|
| registration | `EditorPluginDescriptor`、capability、asset type、template 和 operation path 有稳定 ID | descriptor 只能表达入口；必须绑定可追踪 factory/handler/receipt |
| template schema | CPU Sprite TOML template 与三个 ZUI 的 id/version/root/control id 有测试 | 转为 versioned document schema，不把字符串存在性当成 UI 完成 |
| layout shell | authoring 有 toolbar/list/module/curve/diagnostic 区，preview 有 viewport/transport/stats 区 | 用真实 binding、selection、validation 和 retained controls 填充；保持布局可扩展 |
| optional policy | VFX Graph feature 默认关闭，Particles capability 可被 host 检查 | 继续 fail-close；enabled/available 必须反映 artifact/runtime dependency |
| runtime diagnostics | runtime 有 bounded diagnostic page 和 GPU feedback DTO | Editor 需要订阅带 sequence/generation 的 diagnostic stream，不复制或伪造状态 |

## 4. 参考引擎裁决

Unreal Niagara editor 的 system/emitter/module/renderer/parameter stack 会编译为可安装的 system data，并由 Preview Scene、preview component、simulation cache 和 scalability/compile diagnostics 消费；`UNiagaraComponent` 的 desired age、warmup、reset 等行为同时服务 editor preview 和 runtime。Unity VFX Graph 将 `VFXDataParticle`、context flow、attribute liveness、capacity、bounds、output context 和 `VFXGraphCompiledData`/`VFXCodeGenerator` 统一到可复用 artifact，而不是只改变 “compile queued” 文本。Godot/Fyrox 即使规模更小，也把 particle properties、material、playing、preprocess/fixed FPS、visibility bounds 和 Scene serialization 作为可编辑数据。Bevy 的 ExtractComponent/RenderAsset 证明 editor/runtime 与 render-world 之间需要明确 changed/prepare/retry 生命周期。

## 5. P1 差距与重构要求

| ID | 当前证据 | 工程化重构 |
|---|---|---|
| ED-PFX-01 | authoring.rs:28-62 只注册 drawer/view/template，未创建 document store、selection model 或 session owner | 建 `ParticleSourceDocument`、document id/revision、schema migration、selection/dirty state 和 session lifecycle；Editor 只编辑 source document |
| ED-PFX-02 | authoring.rs:66-180 注册 12 条 operation，但没有 factory/handler；tests.rs:158-176 还断言 menu disabled | 为每条 operation 注册 typed payload schema、factory、capability/policy gate、undo transaction、job handle 和 structured result；未实现时保持隐藏/disabled，不显示成功文案 |
| ED-PFX-03 | `authoring.zui` 的 emitter list/module stack/curve editor/diagnostics 全是 `Space` control | 替换为 emitter tree、module inspector、curve editor、event list、validation list；每个控件绑定 document path、stable key、selection generation 和 mutation command |
| ED-PFX-04 | `particle_system.drawer.zui` 只有 asset/playback/backend/seed/diagnostic 五个 Space row | 建 typed inspector：asset reference、backend decision、seed、time/loop、capacity、bounds、renderer/material、module requirements；展示 runtime receipt 与 degrade reason |
| ED-PFX-05 | `preview.zui` 的 viewport/transport/stats 是 Space；没有 camera, PreviewWorld, render target 或 clock | 建独立 PreviewWorld/PreviewRuntime，使用同一 compiled artifact；提供 play/pause/stop/seek/rewind/warmup、fixed-step、camera/light/background、event log 和 stale-frame fence |
| ED-PFX-06 | `PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT` 只提供 CPU Sprite TOML；GPU/VFX/renderer family 没有创建模板 | 定义 versioned system/emitter/module/renderer templates，编译 capability requirements；CPU/GPU 选择必须由 explicit profile/admission 决定，不以模板字符串猜测 |
| ED-PFX-07 | validate operation 只有 descriptor/payload id，没有 validator output、source span 或 fix-it model | 复用 ParticleSemanticCompiler validator，输出 code/severity/source span/fix-it/dependency/estimated cost；编辑器显示 last-good vs current artifact，不把 warning 当成功 |
| ED-PFX-08 | `open_asset` 绑定 toolkit，但没有 asset resolver、dependency graph、sub-asset/emitter identity 或 missing repair | 建 content-addressed source/artifact resolver、stable emitter/module IDs、dependency graph、repair transaction 和 external asset leases |
| ED-PFX-09 | add_emitter/add_module/edit_curve 没有 schema path、typed mutation 或 multi-document history | 所有 mutation 使用 typed patch + revision precondition + undo/redo journal；曲线保持 key identity/interpolation/units/space，不能只写数组位置 |
| ED-PFX-10 | compile/simulate/preview 状态由 core workbench callback/feedback 文本投影，未找到真实 compiler/job/runtime consumer | 将 UI feedback 绑定 `OperationReceipt`（queued/running/succeeded/failed/cancelled/stale），携带 document revision、artifact generation、diagnostics 和 runtime install result |
| ED-PFX-11 | VFX Graph editor plugin 仅 `EditorPluginDescriptor`/capability；无 node palette、graph document、edge validation、preview 或 asset toolkit | 建 VFX graph document + node/edge schema、context/attribute liveness validator、source map、typed compiler job 和 Particle compiled artifact consumer；移除第二套独立 placeholder graph |
| ED-PFX-12 | runtime VFX Graph compile report 只有 simulation/render pass 字符串；editor 无法展示实际 buffer/attribute/material output | Editor 展示 compiler IR、attribute layout、event/context flow、renderer outputs、material dependencies、dispatch extent、warnings/errors 和 artifact fingerprint |
| ED-PFX-13 | Preview 不消费 runtime GPU readback/diagnostics；Particle manager rewind 是同步调用 | Preview 通过 world job 接口消费 authoritative snapshot/readback，处理 latency/stale/drop；取消/关闭 view 必须回收 job、buffer、subscription |
| ED-PFX-14 | no first-class curves/renderer/scalability/debug panels；workbench VFX routes 只固定 controls/feedback | 分层实现 emitter/module/curve/renderer/scalability/diagnostic/performance panels，每个 route 都有 provider、document binding、operation 和 test，不用固定样例替代产品状态 |
| ED-PFX-15 | Editor/runtime 没有统一 backend decision、device profile、capacity admission 或 shader cache receipt | Editor 预览和 runtime 共用 compiler/device capability query，展示 CPU/GPU decision、capacity clamp、fallback reason、pipeline cache key 和 install generation |
| ED-PFX-16 | Scene component drawer 只能看到 `asset/playing/backend/seed/time_scale` descriptor；typed component 尚未进入 Scene serialization | 与 Runtime RT-PFX-01 对齐，接入 Scene inspector/serialization/Prefab/PIE snapshot；确保 editor mutation 可 round-trip 到 runtime component |
| ED-PFX-17 | save/autosave 的粒子 operation 未连接 durable source write、artifact publish、runtime install | Save 采用 source revision -> validation -> artifact build -> atomic publish -> runtime install receipt；失败保留 dirty state 和 last-good，不报告 saved |
| ED-PFX-18 | operation 全部 `callable_from_remote(false)`，但没有 local permission/lock/lease/telemetry 语义 | local-only 仍需 principal/session capability、document lease、audit/telemetry；未来 remote 执行必须显式授权，不能靠 descriptor 布尔值替代 |

## 6. P2 差距与重构要求

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ED-PFX-19 | ZUI 固定高度的 diagnostic/stats 行无法承载分页、stale、overflow 和多 emitter 数据 | 使用可虚拟化列表、severity/code filter、sequence cursor、acknowledge 和 responsive sizing |
| ED-PFX-20 | emitter/module/curve selection 没有 stable identity，数组重排会破坏 selection/history | 使用 UUID/path identity、document revision mapping 和 selection rebase |
| ED-PFX-21 | preview transport 不能表示 direction、loop occurrence、time dilation、fixed-step 或 discontinuity | 扩展 transport model，并把 clock receipt 传入 PreviewWorld/runtime |
| ED-PFX-22 | 没有 GPU/CPU compare、frame budget、particle count、draw/dispatch/readback latency 面板 | 建 Performance/Validation panel，数据来自 authoritative telemetry，不显示静态样例数值 |
| ED-PFX-23 | material/texture 只作为 runtime handle；Editor 没有 material domain/renderer compatibility picker | 接入 material asset resolver、domain validation、texture/color-space/alpha/blend contract 和 repair action |
| ED-PFX-24 | VFX/Particle workbench 与核心 Scene/Viewport/PIE 没有统一 preview artifact/generation | 统一 artifact registry、PreviewWorld snapshot、PIE install/rollback 和 stale generation fences |
| ED-PFX-25 | 测试主要是 registration/source-shape assertions，未执行 UI event -> document -> compiler -> runtime 闭环 | 增加 operation contract、document round-trip、compile golden、preview determinism、GPU readback mailbox、cancel/dispose、multi-viewport 和 product acceptance |
| ED-PFX-26 | 不同平台/设备的 shader compile、capacity clamp、fallback 没有矩阵化显示 | 建 device profile matrix、capability negotiation、artifact variant cache 与 deterministic fallback explanation |

## 7. 资格门

当前裁决为 **14 Fail / 4 Partial / 0 Pass**：G01-G14 为 Fail；G15（测试覆盖）、G16（feature policy）、G17（disabled/fallback 文案）与 G18（真实产品验收）为 Partial；没有任何门可以记为 Pass。

| Gate | 必须证明 |
|---|---|
| ED-PFX-G01 | Particle/VFX document 有稳定 identity、versioned schema、migration、revision、undo/redo 和 durable source write |
| ED-PFX-G02 | emitter/module/key/node/edge/renderer selection 在重排、merge、reload、undo 后仍稳定 |
| ED-PFX-G03 | 每条 operation 都有 typed payload、factory、handler、capability/lease、job handle、receipt 与失败诊断 |
| ED-PFX-G04 | ZUI 的每个可见 control 都有真实 binding/provider，不以 Space 或 fixture 文本替代 |
| ED-PFX-G05 | validate/compile 使用 canonical ParticleSemanticCompiler，输出 source span、code、fix-it、artifact fingerprint |
| ED-PFX-G06 | VFX Graph 使用同一 IR/artifact；node/context/attribute liveness/material output 可被编辑器和 runtime 共同消费 |
| ED-PFX-G07 | PreviewWorld 使用与 runtime 相同的 compiled program、backend decision、clock、renderer 和 bounds contract |
| ED-PFX-G08 | Preview play/pause/stop/seek/rewind/warmup 可取消、可重入，关闭 view 时 job/subscription/buffer 无泄漏 |
| ED-PFX-G09 | preview frame/readback 带 document revision、artifact generation、frame index、latency/stale/drop 语义 |
| ED-PFX-G10 | component inspector、Scene serialization、Prefab、PIE snapshot 和 runtime install 可 round-trip |
| ED-PFX-G11 | save/autosave 只有在 source/artifact atomic publish 和 runtime install receipt 成功后才报告成功 |
| ED-PFX-G12 | CPU/GPU/capacity/device profile/fallback decision 在 Editor 与 runtime 一致且可审计 |
| ED-PFX-G13 | renderer/material/texture/alpha/depth/velocity/visibility/scalability contract 可编辑、验证并产生 PSO/artifact dependency |
| ED-PFX-G14 | diagnostics/log/performance panel 消费真实 sequence/telemetry，支持 ack、stale cursor、overflow 和 frame budget |
| ED-PFX-G15 | editor tests 覆盖 document mutation、compile failure、cancel/stale、preview determinism、GPU readback、device loss 和 multi-viewport |
| ED-PFX-G16 | VFX/Particles optional feature 的 availability/enabled/dependency/artifact status 在 host、project policy、Editor 和 runtime 一致 |
| ED-PFX-G17 | disabled/unavailable operation 不生成“queued/running/saved”假成功；所有 fallback 公开 reason 和 recovery action |
| ED-PFX-G18 | 真实产品场景从 asset create 到 Scene attach、PIE、render、save/reload 全链验收；静态模板/source guard 不得单独记通过 |

## 8. 推荐实施顺序

1. 先定义统一 source document、schema、identity、typed operation/receipt 和 ParticleSemanticCompiler 输入；保留现有 registration ID，移除假成功 feedback。
2. 接入 Editor document/session/history、asset resolver、Scene component round-trip 和 durable save；让 CPU Sprite template 成为真实可编译 artifact，而非只读 TOML。
3. 建 PreviewWorld/clock/job/readback/diagnostics，复用 runtime compiled artifact；完成 emitter/module/curve/renderer/scalability panels。
4. 吸收 VFX Graph 到 canonical IR，注册 graph editor/operation/artifact consumer；让 graph preview 和 runtime Render Graph 使用同一 execution trace。
5. 补 device/fallback/performance telemetry、multi-viewport/PIE acceptance 与 failure recovery；通过全部 gate 后才提升 feature 状态。
