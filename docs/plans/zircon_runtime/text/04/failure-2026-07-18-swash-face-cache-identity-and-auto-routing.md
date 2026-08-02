---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: swash-face-cache-identity-and-auto-routing
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/raster
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/raster/policy.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches/auto_route.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/rendering.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
---

# Swash face cache identity失效与Auto路由未接线

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/raster/**`当前源10/10 Rust文件及raster worker/source-cache/product-route调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 联动责任：face/variation联动Text01，worker/cache联动Text09，font-byte copy回链PERF-MVP-229。
- 交接原因：bitmap raster key/scaler/atlas与Auto route归Text04；generation-owned face和worker总预算分别属于Text01/Text09，不能在性能切片中建立悬空face缓存。

## 失败现象与复现证据

PERF-MVP-240：每glyph调用Swash `FontRef::from_index`生成fresh cache key，使per-worker复用的`ScaleContext`仍持续font proxy/hint identity miss。caller同时为每glyph重新parse ttf face/axes、规范化variations并clone request variation Vec；PERF229另有完整font bytes复制。

PERF-MVP-241：`GlyphRasterPolicy`的24px/scalable/effects Bitmap/SDF选路只有测试消费；产品Auto由font asset default决定，无asset固定Native。大字号/缩放Auto文本无法使用policy，直接接固定阈值又缺hysteresis。静态证据见`docs/plans/performance/01/2026-07-18-text-raster-static-review.md`。

## 最低共享层根因

worker request只携临时font bytes、face index和owned variations，没有稳定的`FontFaceId/InstancedFaceId + generation` identity。于是Swash无法关联连续请求，FontDatabase也只能每glyph重建effective variations。产品route则在font asset default与raster policy之间存在两个owner，后者未控制真正batch分流。

## 架构修复验收

- Text01发布generation-owned shared font bytes、face index/offset、stable Swash identity、parsed variation axes及normalized instance coords；hot request不得重新parse face/axes。
- Text04 worker request携`FontFaceId/InstancedFaceId + face generation`；per-worker用Swash `builder_with_id`或安全owning face复用proxy/hint cache，face generation变化精确逐出。
- 同face/size/instance glyph按batch进入一个scaler；禁止每glyph`FontRef::from_index + builder + variation Vec`。color/alpha/subpixel source fallback仍按request保持。
- PERF229 shared Arc bytes完成后，同face requests只clone Arc handle；不得用bytes地址或内容hash临时冒充跨generation identity。
- Text09为每worker face/scaler/hint cache设置entry/byte上限及hit/miss/evict/parse counters；1/8/64 faces下无界增长为0。
- Auto native-vs-SDF route收敛为一个policy owner，输入physical px/scale、font capability/default、effects和residency；显式Native/Sdf/Msdf/Mtsdf不改写。
- route按command/layout generation计算一次并带benchmark阈值+hysteresis；zoom跨阈值不逐帧来回切换，stable route/raster miss=0。
- 1/100/1k glyph记录face/axis parse、proxy/hint hit/miss、variation alloc/bytes、raster/upload p50/p95；bitmap/emoji/variable/subpixel/fake italic、face hot reload与Softbuffer/WGPU/RenderDoc像素等价。

## 禁止临时方案

- 不得在Rasterizer中保存借用短命`Arc<[u8]>`的`FontRef`，或以raw pointer作为跨generation key。
- 不得仅扩大ScaleContext entry数；fresh key与per-glyph scaler构建必须消除。
- 不得关闭variable axes或固定wght以避免variation准备成本。
- 不得把所有Auto强制SDF或Native；必须保留capability、effects、residency、hysteresis和显式模式语义。

## 修复结果与回传

2026-07-30 第一段修复：`NativeBitmapAtlasSourceCache` 已按 `(face_epoch, backend_face)` 分配稳定 Swash identity，并在 face invalidation 时与 shared font bytes 一起逐出；worker request 将该 identity 交给 `ScaleContext::builder_with_id(...)`，未保存借用 `FontRef`，也未使用裸指针或 bytes hash。effective variations 已直接复用 `FontDatabase` 的 `Arc<VariationCoords>`，同 face/weight 不再为每个请求克隆坐标 Vec。`source_cache` 回归锁定同面连续 request identity 相同、invalidated face identity 改变；Swash request 回归锁定 shared variations handle。

2026-07-30 产品复验与第二段修复：受管 WGPU job `a0053c3a4254472cab826db7e32b3216` / run `d8964e378fdc4861ba134e2cd4c0ae8a` 已完整重编译当前源码，但 `export_runtime_multilingual_text_product_framebuffer_png` 在第 120 帧仍以 `last_ui_text_raster_worker_pending_count == 115`、`failed_count == 0` 失败，未 capture framebuffer、未生成新 PNG。稳定 identity 本身不足以满足 settle 门；per-worker `SwashRasterizer` 已确认跨请求存活，余下最低根因是同 face/size/instance 仍逐 glyph build scaler。Text04 r5 已将 worker 每次从有界队列最多聚合 32 个 work item，仅在 shared `Arc<[u8]>`、face identity、物理字号、hint 与变体完全一致时复用一个 `FontRef`/`Scaler`，每项结果仍立即走原 cancellation、completion byte budget 与诊断路径。为满足结构规范，执行/分组/取消/完成发布状态机已从 1049 行的 `raster_pool.rs` 拆到 251 行 `raster_pool/worker.rs`，pool 根降至 806 行。Swash 回归锁定三个真实 FiraSans glyph 仅 build 一个 scaler，worker 回归锁定批内取消不阻断其余兼容 glyph；受管定向 Cargo 预约 `cf4d662d664d4845988bf418890a6803` 正在 FIFO 中。handoff 仍为 `open`，没有截图、产出记录或提交完成声明。

2026-07-31 当前源编译门：受管 job `4d6d648211034a6492c2bd0b0443a757` / run `9a8eaaf1dfc04b23a5e4d82a656ac5c0` 在执行新 Swash scaler 回归前以 exit `101` 停在 `zircon_runtime` lib-test 编译。日志中的四个 error 均不在 Text04 授权路径：`scene/level_system.rs:181` 对表达式使用不稳定的 `#[cfg]` 属性（E0658，另有同位置移除表达式错误），`core/runtime/tests/tasks.rs:1` 导入了私有的 `core::framework::render::environment`（两处 E0603）。该诊断不否定 Text04 的批处理合同；没有修改产品 settle 阈值、没有生成 PNG，也没有将它记为已验收。修复这些全 crate 编译门后，Text04 必须先重跑两个定向回归，再重新运行真实 WGPU framebuffer 产品门。

2026-07-31 Auto route current-source 复核：`resolved_auto_text_render_mode(...)` 已成为 screen-space UI Auto batch 的单一产品 owner；它保留 font asset 的显式 Native/Sdf/Msdf/Mtsdf，只有未指定或 asset=Auto 时才把字号与 outline/shadow/glow effects 投影到 `GlyphRasterPolicyRequest`。现有产品回归固定 small→Native、24px→SDF、outline/shadow→SDF、glow true-distance→MTSDF。当前 batch DTO 没有稳定 command/layout identity，故不能用数组位置或 text hash 实现正确跨帧 hysteresis；该前置接口与 physical scale/residency 输入尚未完成。

2026-08-01 Auto route 前向收敛：产品 owner 已从 stateless helper 硬切到 `AutoTextRasterRouter`。render planning 在每条 command 只读取一次既有 `UiRenderCommand::cache_generation()`，并把单次 `Arc<str>` tree identity、`UiNodeId` 与 layout `source_range` 投影为稳定 fragment key；不同 tree、同 node 的多 line/rich/inline fragment 不会串状态，也没有使用数组位置或 text hash。相同 generation 直接命中 route state；generation 变化才进入 `GlyphRasterPolicy`。screen-space `font_size` 作为实际渲染 px 输入，font asset 的显式 Native/Sdf/Msdf/Mtsdf 保持作者语义，outline/shadow/glow 仍强制现有距离场策略。

`GlyphRasterPolicy` 现在单源持有 24px benchmark threshold 与上下各 2px hysteresis：warm Native 直到 26px 才切 SDF，warm SDF 低于 22px 才回 Native；effects/显式格式不被 hysteresis 改写。router 以 2048 entry hard cap、300 idle-frame 回收和 tokenized recency queue 保持有界，稳态 generation hit 只做 O(1) HashMap/queue 操作，超过四倍 recency cap 才按 live entry 稀疏压缩。prepare report 新增 capacity/entries/generation hits/policy evaluations/warm retain/switch/capacity eviction/idle eviction；1/100/1k exact scale 与 ignored 31-sample p50/p95 exporter 已写入。

2026-08-01 二次静态审查前向修复五项 actionable 缺口：补入 tree domain 防止跨树 node-id 碰撞；移除 `Arc<str>` identity 的错误 `Copy` 并修正 move；把 child visibility 精确开放到 `ui::text`；收敛 stateless/product effects projection；取消未使用 Auto 场景的 eager 2048-entry 预分配。current-source owners 为 auto-route 265 行、resolved-batches 180、render 771、rich-render 278、text-system 579、policy 275、interface command 523，均低于 800 行 production warning；测试 owner 453 行。scoped rustfmt、tracked/untracked whitespace 与 diff check 通过，production panic/unwrap/expect/dead-code allow 扫描为 0，未发现 remaining actionable P0/P1/P2。

Open state: `stable Swash/scaler batching + Auto identity/hysteresis/warm-route residency implementation complete / resolving_failure / managed_validation_pending`。仍需 coordinator receipt 后运行 focused/upward Cargo、1/100/1k 与 ignored p50/p95、真实 WGPU/RenderDoc 像素和 stable-route raster-miss 证据；成功前保持 open，不写 blocked/accepted，不生成或登记新 PNG。
