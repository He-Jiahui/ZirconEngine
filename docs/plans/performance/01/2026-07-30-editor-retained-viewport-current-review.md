---
related_code:
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/viewport
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/16/failure-2026-07-17-editor-viewport-synchronous-readback.md
tests:
  - inline tests: 15
  - rustfmt check: blocked by pre-existing import-order drift in 2 externally modified app files
  - current-source managed Windows Cargo pending
  - WPR/Tracy viewport copy, lock and input trace pending
  - RenderDoc product-frame/readback capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained viewport当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/{viewport.rs,viewport/**,viewport_image_redraw.rs}`与`zircon_editor/src/ui/retained_host/viewport/**`当前源 **34/34** 个Rust文件、**1,908** 行、**15** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`cb7d750920b3820c3342ed45f079a95f9f7a5e87c6de54c9f7ea0dc0cc3687b0`。其中`app/viewport/pointer_event.rs`、`app/viewport/toolbar_pointer/click.rs`和`viewport/poll_image.rs`含外部未提交内容，本轮只读纳入current-source审查，未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| app viewport adapter | 8/8 | 334 | 0 | pointer/world-space route、toolbar click/size与image redraw |
| controller production | 16/16 | 652 | 0 | lazy framework、viewport lifetime、submit/capture/import与world-space extract |
| controller tests/helpers | 10/10 | 922 | 15 | create/resize/aspect/capture/world-space/job/drop合同 |

## 发现

- **正向边界**：render framework通过EditorJobSystem惰性解析并带cancel/drop合同；同尺寸不会destroy/create viewport；quality profile环境值用`OnceLock`解析一次；WGPU capture在generation未变化时先返回`None`，不会深clone RGBA；capture error不再把cached image伪装为新frame。现有15条测试覆盖这些语义，但没有锁等待、整帧bytes或真实GPU证据。
- **PERF-MVP-023 / controller锁跨foreign work**：`poll_image()`持有`Mutex<ViewportState>`时调用foreign `capture_frame_if_newer`、复制RGBA到`Image`并深clone到`latest_image`；`submit_extract_with_ui()`持锁完成viewport ensure、world-space command构造并调用foreign submit；resize还在同一锁下destroy/create/configure viewport。GPU/readback、字符串分配或framework阻塞会串行阻塞poll、submit、pointer和error consumer。
- **PERF-MVP-023 / 新frame至少5次整帧复制**：WGPU `capture_frame_if_newer`对新generation深clone stored `CapturedFrame`；Editor `SharedPixelBuffer::clone_from_slice`复制一次，未被任何读取方消费的`latest_image = image.clone()`再复制一次；host projection随后经`Image::to_rgba8()`和`SharedPixelBuffer::as_bytes().to_vec()`再复制两次。即从framework stored frame到`HostViewportImageData`共 **1次framework + 4次Editor** 全RGBA copy，其中前三次位于controller锁覆盖的capture/import/retain链。host DTO还全量hash RGBA并格式化resource key。
- `RetainedViewportController::latest_image`只有赋值、resize/error清空和field定义，没有读取点；它既增加copy pass也长期保留一份整帧owner。4K RGBA单份为33,177,600 bytes，约 **31.6 MiB**；framework stored frame、controller retained image、app image、host转换临时buffer与host DTO在转换边界可形成至少5份payload owner，必须以实测peak live bytes确认，不得只统计stale poll。
- **PERF-MVP-121 / stable world-space仍重建**：每次submit都在controller锁内遍历全部submission，重建commands Vec；每个command至少显式复制background、foreground、border、font和control id五个String并构造完整style。pointer hit按逆序O(S)扫描，down/move/scroll会clone完整submission或capture，其中含surface/node/control/camera四个String。稳定generation没有command extract或route handle复用。
- **PERF-MVP-106 / toolbar click宽投影**：每次click先对相同surface size重跑toolbar layout并忽略返回值，再同步pointer bridge；随后`get_host_presentation()`深clone整份presentation，只为扫描4个dock和floating windows取得一个`UiSurfaceFrame`。fallback还会owned-clone全部view instances后线性查找，layout frames可在同一次click读取两次。既有action-key upsert/same-frame no-op只避免pointer surface错误重建，未关闭上游clone/build。
- resize当前先destroy旧viewport再create/configure新viewport，连续size事件没有latest-value frame coalescing。动态验收应同时记录recreate次数与失败后的可用性，但本轮性能计划不借机扩张为独立correctness改造。
- 本模块没有可孤立提交的小修：即使删除未读`latest_image`能直接少一份copy/owner，最终GPU texture generation、explicit capture、host DTO和controller锁边界仍需Render16 + EditorUI08批准；在当前设计批准未完成前不修改Rust，避免保留第二套CPU frame authority。

## 参考与目标

- Godot `dev/godot/servers/rendering/renderer_viewport.cpp:995,1227-1231`创建render target并以texture RID返回viewport product，不把常规viewport消费绑定到CPU RGBA。Zircon应以定义生命周期的GPU texture generation/handle作为同device产品路径。
- Bevy `dev/bevy/crates/bevy_render/src/view/window/screenshot.rs:55-64,631-662`把截图定义为显式、异步、稍后交付的操作；`map_async`完成后才复制mapped bytes。它证明capture/readback应与正常render target消费分离，但不意味着Zircon可以无界排队或省略lifetime fence。

Render16把product viewport与explicit capture拆开：同backend以GPU texture/view + generation/lifetime fence交给host；跨设备fallback使用有界2-3槽async readback ring、latest-ready/drop counters。EditorUI08让host viewport image独立于结构presentation，仅在image generation变化时提交shared owned bytes/handle；controller只在短锁内capture state/cursors，framework call、GPU wait、RGBA import、world-space command build和submit均在锁外，按generation CAS/commit。

PERF-MVP-121使用immutable world-space generation与`Arc<[UiRenderCommand]>`；变更时构建一次，稳定frame复用。pointer capture只持stable surface/node/control route handle，不clone完整submission；z-order/depth/billboard/cancel语义由同generation route table保持。

## 动态验收

按viewport `1/2/4`、resolution `1280x720/1920x1080/3840x2160`、rate `30/60/120Hz`、frame state `stale/new/error`、resize burst `1/100/10K`、world-space surfaces `0/1/100/1K`和pointer `1/1K/1M`运行cold/warm矩阵。记录framework capture clone、Editor RGBA copy passes/bytes、live owners/peak RSS、full hash bytes、controller lock wait/hold、submit/readback fence、frame age/drop、viewport recreate、world command/String alloc、submission clone、toolbar presentation/layout/view-instance build及UI thread p50/p95。

验收要求：常规同backend viewport的CPU full-frame copy/readback/reupload=0；fallback每ready generation CPU完整copy不超过1且ring有界；controller锁不跨foreign/GPU/full-frame/world-command工作；`latest_image`冗余owner=0；stable world-space command/String alloc与pointer full-submission clone=0；same toolbar click的presentation clone/layout/view-instance build=0；resize recreate每frame不超过1。managed Cargo、WPR/Tracy、F4产品trace、RenderDoc资源/pass证据与independent review完成前保留在`pending.md`，不进入`review.md`。
