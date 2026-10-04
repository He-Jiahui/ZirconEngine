---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-viewport-synchronous-readback
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/16-compute-neural.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/16
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/gateway/session/frame.rs
  - zircon_editor/src/core/gateway/session/contract.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/viewport/poll_captured_frame.rs
  - zircon_editor/src/ui/retained_host/viewport/poll_viewport_product.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_lifecycle.rs
  - zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/viewport_product_registry.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_diagnostics.rs
tests:
  - editor viewport steady-state no synchronous GPU poll guard
  - async readback ring backpressure and generation test
  - GPU texture interop resize and device-loss product test
  - completion_before_registration_remains_available_while_armed
  - late_completion_for_trimmed_generation_is_discarded
  - repeated_late_completions_do_not_grow_completed_storage
  - dropped_capture_request_cancels_its_reservation
---

# Render16：Editor viewport 常规帧同步整帧 readback

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 retained viewport 到 RenderFramework/SceneRenderer/WGPU readback 的产品调用链静态审查
- 修复责任计划：`docs/plans/zircon_runtime/render/16-compute-neural.md`
- 共同责任：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：Render16 已声明统一异步 readback owner；Editor01 需冻结 viewport GPU texture import/composition 契约，不能只在 UI 端少 clone 一次。

## 失败现象与复现证据

Editor retained viewport 用 `submit_frame_extract_with_ui` 提交常规帧。render submit 进入 `render_frame_with_pipeline` 后，无条件通过 `finish_viewport_frame -> read_texture_rgba` 把 final texture 读回 CPU。该 helper 每帧创建 staging buffer、encoder 和 MPSC，提交 copy，调用 `device.poll(wait_indefinitely)`，再复制到新的 RGBA `Vec`。随后 `record_capture` 每帧生成 graph dump 并深 clone compiled pipeline，`capture_frame` clone 整个 RGBA，Editor `SharedPixelBuffer::clone_from_slice` 又复制一次。

1080p RGBA payload 单帧约 7.9 MiB，60 FPS 约 475 MiB/s，且同步 poll 强制 CPU/GPU 栅栏。submit 期间还持有 framework operation/state guard，GPU stall 会放大多 viewport 与其他 render 操作的排队。

26-file retained viewport 审查进一步确认，Editor controller 的共享 mutex 跨 `capture_frame`、`SharedPixelBuffer::clone_from_slice`、render-framework resolve、viewport destroy/create 与 submit 持有；size 每次变化立即 destroy/create，resize burst没有 latest-value coalescing。旧 `poll_image` 在任何 capture/import error 后还会把 cached image作为“新图”返回，引发 stale redraw；本轮已直接修正为错误返回 `None` 并保留缓存/诊断，同时把 GI profile env解析缓存一次，但这不减少正常帧同步 readback。

2026-07-30 current 34-file复读把CPU链量化为从framework stored frame到host DTO至少 **5次整RGBA copy**：WGPU新generation clone `CapturedFrame`，Editor import复制、未读`latest_image`深clone，host再经`Image::to_rgba8`与DTO `to_vec`双copy并全量hash。前三次capture/import/retain发生在controller mutex覆盖区。4K单份33,177,600 bytes（约31.6MiB），转换边界可同时保有framework frame、controller image、app image、临时shared buffer与host DTO；需以live-owner/peak RSS counter确认。另有stable world-space commands在同一mutex内重建并调用foreign submit，进一步放大等待。

## 最低共享层根因

`submit_frame_extract` 把“渲染产品帧”和“同步截图”合成一个返回 `ViewportFrame` 的语义；Editor 没有可直接交给 retained UI compositor 的 GPU texture generation/handle contract，也没有有界 async readback fallback。

## 架构修复验收

- 同 GPU device/backend 时，viewport product 以稳定 texture handle/view + generation/lifetime fence 交给 UI；常规帧没有 MAP_READ、`wait_indefinitely` 或 CPU RGBA clone。
- 无法 GPU 互操作时使用有界 2–3 槽 async staging ring；UI 取最新 ready frame，积压有 drop/coalesce 计数且不阻塞 render/main thread。
- screenshot/headless/pixel-test 保留显式 capture/readback API；submit 与 capture 分离，现有产品测试可迁移而不丢像素验收能力。
- graph dump 与 compiled pipeline snapshot 按 pipeline generation 缓存/共享，正常帧不做文本重建和 compiled graph 深 clone；诊断请求保留相同内容。
- 1080p 30/60/120 FPS、resize、device loss、多 viewport 做 WPR/Tracy/RenderDoc；报告 GPU wait、copy bytes、allocation、main-thread p95 和 frame age。
- controller 锁不跨 framework/GPU调用或整帧 import；resize burst按 frame latest-value 合并，destroy/create≤1/frame；capture/import error不重发 stale generation。
- 同backend product frame从render target到host的CPU full-frame copy、RGBA hash与结构presentation carry bytes均为0；fallback每ready generation最多一次CPU完整copy，记录copy passes/bytes、live owners/peak RSS与frame age/drop。删除未读controller image owner，不以少一次copy代替product/capture分离。

## 禁止临时方案

- 不得只把同步 readback 移到另一个主线程函数或每 N 帧阻塞一次。
- 不得建立无界 pending map/readback 队列，或跨 ABI 暴露未定义生命周期的裸 WGPU pointer。
- 不得删除 screenshot/pixel-test 能力来伪造产品帧加速。

## 修复结果与回传

2026-08-13 source closeout：同-device 常规 viewport 帧已发布 backend-neutral
`RenderViewportProduct`，运行时在三代有界 registry 中保留独立 GPU texture snapshot，retained UI
通过 external-image provider 按 resource key/generation 解析，不携带 CPU pixels。presenter 首次
`confirm_resident` 后关闭该 viewport 的 async capture；确认前或无兼容 presenter 时，fallback 经
backend 的 product-diagnostic readback service 与 latest-ready mailbox 异步回传，底层 admission
同时限制 pending request 与 pending bytes。显式 screenshot/headless `capture_frame` 继续作为独立
同步 API，未混回普通 submit。

Editor poll/product/capture 均在 framework 调用前释放 shared state mutex；错误不重发 stale
generation。resize lifecycle 已将 framework destroy/create 放在 shared mutex 外，但
`viewport_lifecycle` operation mutex 仍覆盖 framework submit/create/destroy，尚需动态 contention
证据或进一步收窄。burst coalescing、device-loss/resize 产品动态证据、1080p/4K WPR/Tracy、copy
bytes/live-owner/peak-RSS 与 RenderDoc/PNG 仍待 coordinator。因此当前状态为
`open_source_and_dynamic_validation_pending`，不记 accepted closeout。

2026-09-21 current-source 复核发现 latest-ready mailbox 仍有一个 Important 生命周期缺口：旧
generation 从 `pending` 淘汰后，其 WGPU callback 若晚到，`complete()` 会把整帧 `Vec<u8>` 插入
`completed`，而后续再无 pending 驱动清理，重复发生可形成无界 CPU 内存保留。当前修复在 callback
交给 backend 前建立有界 `armed` generation window；request 被拒绝、错误路径丢弃或 RAII drop 时
原子取消，trim 同时移除 armed/pending/completed，只有仍 armed 的 callback 可写入。这样保留了
callback-before-register 的合法时序，同时丢弃已淘汰 generation 的晚到 payload。五个 mailbox
回归已加入，与既有 generation 用例组成 6-test focused filter。精确五文件快照的独立终审
`review-render16-readback-chain-r2` 为 C0/I0/M0；固定源码的 managed Cargo 尚未取得通过票据，且上述产品/性能门禁仍待完成，故本
failure 保持 `open`。

同一轮独立审查在修复前快照给出 C0/I4/M2。除 mailbox I2 已进入当前补丁外，以下 Important
仍未解除，且相关路径包含其他 owner 的 foreign dirty overlay，本 Session 不吸收：

- direct-render 分支没有携带 async capture request；compiled fallback 使用 product-diagnostic
  service，其默认单请求上限为 16 MiB，而 3840×2160 RGBA8 为 33,177,600 bytes，4K fallback
  会被 admission 拒绝并以 no-op callback 收束，因此没有新帧。
- fallback 当前至少存在 mapped bytes→`Vec<u8>` 与 runtime `CapturedFrame`/RGBA clone 两次整帧
  CPU copy；仍需在唯一 mapped-copy 边界转移所有权或共享不可变 backing，并以 copy/pointer 断言验收。
- Editor `submit_extract` 的全局 `viewport_lifecycle` operation mutex 仍跨越
  `ensure_viewport` 与 framework submit；size mismatch 仍立即 destroy/create，尚无每帧最多一次的
  latest-value resize coalescing。

因此即使 mailbox focused 测试通过，本 failure 也只能记录该 Important 的局部修复，不得 return
或 closeout；需由相应源码 owner 完成上述三项并补齐 4K/device-loss/resize、多 viewport、
copy/RSS/frame-age/drop 与 WPR/Tracy/RenderDoc/PNG 证据。

Managed Windows focused ticket `b3ae4113607a44b6a86d3ad3354ee24e` 冻结的是 rustfmt 修正前的两个
源码哈希；它已在 Cargo/test 启动前终态失败，因为隔离验证副本不能解析相对
`.codex/.../validate-matrix.ps1`，coordinator 记录 exit 1 / `failureCategory=coordinator`。这不是测试
失败，也不得复用为现行源码证据。唯一 successor `2a5e5ccf50c941d088a937c140b60459` 已冻结
edition-2021 修正后的当前哈希，改由仓库绝对 validator 路径配合 validation-copy
`-RepoRoot (Get-Location).Path` 执行同一 `capture_mailbox` 6-test filter，当前状态为 `queued`；在其
取得真实终态前不宣称 Cargo/WGPU GREEN。

### 2026-09-26 current-source successor handoff (`failure-roll-01a084c8-render16-editor-readback-r3`)

- 本次 successor 仅接管本 failure 文档。ownership transfer fingerprint 为
  `ff5e82afb116dd5596f0c5cb54c946c49ed5c8f561cb7b7e6f0ff2c5097770fa`；生产源码没有重复 claim，也没有吸收 Render16 queue 或 Editor01 的 foreign overlay。
- 本轮受管文档边界冻结在预审 snapshot `3895`，其 manifest hash 为
  `b145d138f36c81e9be74c26a121da108ae907f5eb99b64aa44ecc31bbf7689f0`；该 snapshot 是本次独立审查的权威输入，任何审查后措辞变更都必须在新的 post-review snapshot 中重新封存。
- 补齐上述边界声明后的 post-review snapshot 为 `3896`，manifest hash 为
  `97a5e3aa8be05f28e05f8400c209c17fe5faecbe30d09132b838dac0ab8eeb1d`；该快照明确绑定本次审查所复核的文档内容，后续审查回执写回仍须再封存新的最终 snapshot。
- 当前只读源码边界哈希如下，作为动态验收前的索引而非本 Session 的提交范围：
  - `zircon_editor/src/core/gateway/session/frame.rs` — `c17b78999a8b4c96141982e1ed86b8c8af0a3ef76f0aa4b1a3d389f2b701e375`
  - `zircon_editor/src/core/gateway/session/contract.rs` — `42016e2f7f20cf238faa3455305ed1a9dc8f5de3a7e660e6fe051a1cfd5962cf`
  - `zircon_editor/src/ui/retained_host/viewport/submit_extract.rs` — `27883e71aa3c2118633dcda5ba64c94978223bcb16124318e41caa5ee5cc3aa3`
  - `zircon_editor/src/ui/retained_host/viewport/poll_captured_frame.rs` — `74d27f23e7d6d8ca3a65f8adba8cf760269a46a8de23de288cfa6cfff91ea601`
  - `zircon_editor/src/ui/retained_host/viewport/poll_viewport_product.rs` — `5b2fc0eb32cd47958b68e1953ef7311d9873e4e59d7f648c182018296f2c94f3`
  - `zircon_editor/src/ui/retained_host/viewport/viewport_lifecycle.rs` — `3648e0f0df9dc39a8666f163a9ce3fa3f0436c9e7267276b172b410cd3e68677`
  - `zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs` — `6c8201abb65753c0bc674be1e265a407d41985de9a01fd6f161bf5e979049e7f`
  - `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs` — `f91bfd854fde6d621f2f9c590c43f95ed260fbde0c85081798ea3f7b667eb06b`
  - `zircon_runtime/src/graphics/backend/render_backend/render_backend_diagnostics.rs` — `73f404e4f39e0c006a1df8319df05920d7bfcb61293d49cb4dc3651fa77aeb09`
- 当前静态核对仍与既有 receipt 一致：`capture_mailbox` 保留 bounded `armed/pending/completed/ready` generation window，RHI diagnostic budget 仍为 16 MiB/request、32 MiB/frame、64 MiB pending；普通 product path 与显式 test capture 分离。与此同时，4K fallback、resize mutex/coalescing、copy/RSS/frame-age、device-loss/多 viewport 以及 WPR/Tracy/RenderDoc/PNG 仍没有动态证据，不能把静态 mailbox 修复提升为产品 GREEN。
- 已有 current-hash successor `2a5e5ccf50c941d088a937c140b60459` 仍是唯一可复用的 `capture_mailbox` 六测试排队请求；本轮不重复提交 Cargo，也不复用旧失败票据 `b3ae4113607a44b6a86d3ad3354ee24e`。failure 继续 `open`，固定回传、审查和 closeout 均待动态门完成。

## 产出记录与时间

| 时间 | 来源增量 | 状态 | 交接内容 |
| --- | --- | --- | --- |
| 2026-07-29 04:01 CST | Editor01 PERF-MVP-424 current source | 仍待修复，未回传 | Editor01 已把 runtime `tick_frame` 的完整 cadence demand 硬切为 `OnDemand/SleepUntil/Continuous` 并接入 retained host，因此 frame pacing 不再依赖每 tick 的布尔结果。该变更没有改变 `SessionGateway::capture_frame` 对 foreign RGBA 的 `Vec` 复制，也没有提供 texture view/generation/lifetime fence 或 2–3 槽 async readback；正常 viewport 的 GPU-resident product-frame 契约、显式 fallback capture 和 1080p/4K copy-bytes/WPR 证据继续由本 Render16 failure 的既有验收条目负责。 |
| 2026-07-30 23:30 CST | Performance01 retained viewport current-source 34/34 | 静态补证，动态未验收 | 新frame链当前至少1次framework+4次Editor整RGBA copy，未读`latest_image`长期留存；capture/import/retain与world-space build/submit处在controller mutex覆盖区。Godot常规viewport返回render-target texture RID，Bevy把截图单列为异步map/readback；Render16继续负责GPU product handle、explicit capture、有界fallback ring与WPR/RenderDoc验收。证据见`../../../performance/01/2026-07-30-editor-retained-viewport-current-review.md`。 |
