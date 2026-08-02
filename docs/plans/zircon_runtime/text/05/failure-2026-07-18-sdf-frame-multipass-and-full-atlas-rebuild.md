---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: sdf-frame-multipass-and-full-atlas-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
---

# SDF frame多轮准备与整页重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/sdf`当前源24/24 Rust文件及产品调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md`
- 联动责任：Text09负责cache/frame budget，Render17负责persistent GPU page/buffer与dirty upload；回链PERF-MVP-231/243/244。
- 交接原因：SDF prepare/failure/bake/atlas artifact owner属于Text05，GPU写入由Render17消费。

## 失败现象与复现证据

PERF-MVP-249：产品每帧先做generation-failure bake，再做fallback CPU preparation，renderer又重复CPU preparation并build atlas。单次prepare最多三次分配render scalar Vec。glyph cache hit深clone bitmap；atlas每帧分配/清零所有R8/RGBA pages、复制全部slot、全atlas扫描nonzero，随后上传并新建vertex buffer。

## 最低共享层根因

failure、advance/decoration、glyph bitmap、slot page与GPU plan没有共享一个generation artifact；cache只避免FDSM计算，没有避免bitmap ownership、page materialization和render submission重建。

## 架构修复验收

- 每frame generation对每个text/run只构建一次scalar/key/metrics/failure结果，fallback与renderer共享artifact；无重复`prepare_sdf_runs_cpu`。
- glyph cache返回Arc/slot handle，不按cache hit clone bitmap；key共享font/family/language identity，不按glyph分配String。
- atlas page跨帧驻留；stable frame page alloc/zero/copy/nonzero scan/upload=0，changed glyph只写dirty rect并增量更新report。
- GPU texture/vertex/instance buffer按capacity与generation复用；viewport只更新uniform，device loss显式重建。
- 1/100/10k glyph稳定300帧记录prepare pass、Vec/String alloc、bitmap clone bytes、page touched/upload、buffer create与CPU/GPU p50/p95；增长按O(delta)。
- fallback whole-batch重规划最多一次有界二阶段提交，failure与slot generation一致；SDF/MSDF/MTSDF/offline/dynamic、clip/effects、reload/device loss、current-source Cargo与像素通过。

## 禁止临时方案

- 不得只缓存FDSM输出但仍每帧clone bitmap并重建整页。
- 不得关闭nonzero report而保留其他整页扫描并宣称完成。
- 不得用无界page/glyph cache换取稳定帧0工作；bytes/eviction/age必须可观测。

## 修复结果与回传

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / secondary_review_complete / managed_validation_pending`。

- fallback advance/decoration 与 renderer 现在共享同一 `SdfRunCpuPreparation`；产品 frame 只调用一次 `prepare_sdf_runs_cpu`，whole-batch/mixed overlay native fallback 复用原 run metrics。测试 leaf 中保留的 helper 调用不属于产品多轮准备。
- baked glyph bitmap 与 offline extracted rect 都由 `Arc<[u8]>` 共享；font/family/language key 改为 run-owned `Arc<str>`；runtime baked glyph cache 受 4096 entries/64 MiB LRU 约束并暴露 resident bytes/eviction/age，不以无界 cache 换稳定帧。
- `SdfPersistentAtlasCache` 跨帧持有 page pixels/nonzero count/placement。稳定 slot 的 page alloc/zero/clear/write/full-page scan 为 0并复用同一 page Arc；变更/移除只 clear/write dirty rect。异步 glyph completion 即使 slot 不移动，也会发布 bake dirty page 并与 plan transition dirty rect 合并，避免漏 GPU upload。
- GPU atlas upload 直接借用按 page 排序的 bake pages，并把 flat command offset checked 转为 page-local offset；旧 monolithic `atlas_write.rs`/flat `pixels` owner 已删除。SDF vertex buffer 按至少 4 KiB 二次幂 capacity 驻留，以 payload hash 跳过相同帧 queue write，stable frame create/write 为 0。
- 二次审查前向修复了 completion backpressure/panic 永久 pending、逐 glyph String 深拷贝、source/offline/baked cache 无界三个问题；新增 owner 均低于 800 行，旧 owner/production panic/unwrap/expect/dead-code allow 扫描为 0。
- `ScreenSpaceUiTextSystem` 现在只构建一个 generation-consistent `SdfAtlasBake`；同一 artifact 的 typed failures 驱动 fallback 并原样交给 renderer。旧 `generation_failures_for_plan` / `sdf_generation_failures` / `generation_failures_for_slots` 双轨已删除；whole-batch fallback 只允许一次显式第二阶段 replan/rebake。
- `SdfPreparedAtlasCache` 以 exact atlas size/slot identity 保存一份 input-bounded compiled artifact；pages/glyphs/failures 均为 `Arc<[T]>`。无 scheduler pending、无 pending/deferred retry且 plan 稳定时 O(1) 复用，dirty/work counter 为 0；pending、budget-deferred、font generation 变化都强制前向重建。`compiled_atlas_build_count/reuse_count` 已进入 bake report。
- stable atlas text snapshot 跳过 key/BTree/shelf/slot/run transition；failure 以 slot index + key 双校验，并用 failure Arc identity 跳过稳定帧映射。`SdfTextCpuFrame` 精确缓存 metrics/advance/decoration；无 fallback 快路径不再 take/collect Vec，font face 变化或 fallback 变换后失效。
- renderer 以 exact viewport/text/CPU metrics/decoration snapshot 复用 material/draw/text ranges/CPU vertex Vec；atlas resize 或任意 upload 都强制重建。CPU plan 与 vertex plan build/reuse 均进入 report；device recreate 会因 atlas mismatch 强制 full upload/build。
- 第二次静态审查又前向修复 stable bake metadata 分配、stable failure map 分配、no-fallback Vec 搬移和 renderer generation owner 四项问题。`sdf_atlas.rs` 已把 failure 映射拆到 49 行 child，根 owner 732 行；局部 rustfmt 和全树 `git diff --check` 通过。

当前不标记 fixed：1/100/10k glyph 稳定 300 帧 counter/p50/p95/RSS、managed current-source Cargo、reload/cancel/device-loss 组合门、真实 WGPU/RenderDoc 像素与请求的新截图仍待 coordinator receipt 后验收。没有生成新截图，也没有把已有 PNG 当作本轮证据；后续图片只能写入 `docs/tests/runtime/text`，不得写入 `target`。
