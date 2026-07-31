---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: bitmap-atlas-retry-budget-not-wired
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/atlas/bitmap_run/retry.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_driver.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_state.rs
  - zircon_runtime/src/text/atlas/render_submission/retry.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
---

# Bitmap atlas retry预算未接产品

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/atlas`当前源47/47 Rust文件及native bitmap retry产品调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 联动责任：共享frame CPU/bytes预算联动Text09，persistent slot/key与retry去重回链PERF-MVP-231。
- 交接原因：atlas retry/fairness属于Text04；全局文本worker/render frame预算属于Text09，产品默认值必须由共同预算下发。

## 失败现象与复现证据

PERF-MVP-245：retry planner和测试支持due retry/new source独立限额，但唯一产品调用通过`GlyphAtlasBitmapRetryFrameDriverConfig::with_defaults()`把两类限额都设为None。page pressure下所有到期blocked与所有new source同帧重建run/batch/GPU plan，失败项下一帧再来；queue没有count/bytes hard cap、age fairness或key dedup。

测试中的custom backpressure policy只能证明机制可用，不能证明产品受控。静态证据见`docs/plans/performance/01/2026-07-18-text-atlas-render-submission-static-review.md`。

## 最低共享层根因

budget作为可选测试config停在frame driver边界，没有由Text09全局预算和Text04 residency/slot状态驱动。queue保存source DTO并每帧整体collect/replace，缺少stable key、generation、age和累计bytes，无法做公平/有界调度。

## 架构修复验收

- 产品config必须显式设置due retry/new source的count、source bytes、estimated staging/upload bytes和CPU work预算；默认unlimited命中为0。
- queue设置entry+byte hard cap，保存轻量`GlyphRasterKey/slot request + face/page generation + first/next frame`；同key同generation去重。
- old retry与new source按age/配额公平调度；给出最大等待上界，任何一类不得长期饥饿。
- 与PERF231 persistent slot结合，只对真实miss/evicted key做allocation/upload，visible hit不进入retry/new规划。
- overflow/drop/cancel必须有显式reason和placeholder/fallback；不得静默丢glyph或在caller同步raster。
- max-pages=0/1，1/100/10k due+new sources跑300帧：每帧attempt count/bytes/CPU不越预算，queue不越hard cap，oldest wait有界。
- 记录scheduled/deferred/backpressured/dedup/dropped/canceled、queue count/bytes/age p50/p95、plan vertex/staging/upload bytes及frame CPU。
- stale invisible source、face invalidation、worker saturation、empty frame、shutdown和Softbuffer/WGPU像素等价。

## 禁止临时方案

- 不得只把None改成无依据的大常数；预算必须来自产品benchmark和全局frame配置。
- 不得只限制retry而让new source无限，或反向让old retry饥饿。
- 不得以无界defer queue换取单帧平滑；entry/bytes/age都必须有上限。
- 不得重复完整source bitmap bytes进retry queue；只保存shared handle或轻量key。

## 修复结果与回传

2026-07-31 current-source 实现复核：native bitmap 产品 policy 已将 Text09 的 256 glyph / 2 MiB 确定性帧预算均分为 old retry 与 new visible work 各 128 glyph / 1 MiB，并为 retained blocked queue 设置 256 entry / 2 MiB hard cap；oversized source、frame defer 与 queue overflow 分别进入 rejected/backpressured/overflow counter，terminal byte rejection 显式降级到 Glyphon。persistent slot hit 不进入 new-work budget，同 raster key 的 miss 帧内只计一次；old retry 保持队列顺序并轮转，300 帧饱和回归要求三项各尝试 100 次。Text09 权威表把 CPU time 定义为只观测不 gate，故本切片不发明未经 benchmark 的毫秒或像素常数。仅完成源码复核与 scoped 静态检查，未执行 current-source Cargo/WGPU。

Open state: `产品有限 count+source/upload-byte 默认、bounded fair key queue 与 overflow fallback 已实现；等待规模 counter/current-source Cargo、产品 WGPU 像素以及 Text09 观测型 CPU trace，成功回执前保持 open`。
