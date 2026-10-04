---
related_code:
  - zircon_runtime/src/core/framework/render/anti_alias/fallback.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mode.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/taa_quality.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/execute_taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs
  - zircon_runtime/src/graphics/resource_identity.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/scene_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/ensure_offscreen_target.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resource_identities.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_taa_reactive_mask_graph_resource.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias/reactive_mask.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs
tests:
  - anti_alias five of five Rust files reviewed
  - focused frame context TAA history graph and GPU execution callers traced
  - source-level TAA graph/physical encoding and product assertions updated
  - current-source Cargo, GPU timing, F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: source_implementation_complete_dynamic_pending
---

# Runtime render anti-alias逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/anti_alias/**`当前5/5个Rust文件、609行，并聚焦追踪frame submission、temporal feature descriptor、history store、reactive-mask writer与TAA resolve。framework合同全部为Copy enum/report与常数分支，resolve每帧成本为O(1)，无String/Vec/锁或独立CPU热点；双history texture也按size/format key持久复用。实际瓶颈位于graphics实现，不应把framework小函数微优化冒充产品收益。

## PERF-MVP-350：固定 mask clear 的前向修复状态

已删除`taa-reactive-mask-clear`权威和 executor。编译图保留 logical mesh pass，以维持图覆盖与执行记录契约；但在 transient materialization 前，空 command stream 会把`taa-reactive-mask`绑定到共享 black texture。因此它不会分配或编码 R8 reactive-mask attachment，实际 mask pass 与 mask 写入字节均为零。TAA WGSL 依据`textureDimensions`在 black fallback 上安全取样。

非空 command stream 由唯一 mesh writer 以`clear_store`和 draw 在同一个 WGPU render pass 内完成，并在 pass 结束后记录物理编码次数与`width * height` mask 写入字节。frame execution record、renderer stats 与 diagnostics 还记录每帧 TAA resolve bind-group create count。产品测试覆盖空、opaque 与 transparent authored reactive stream 的 zero/one pass 和字节计数契约。

已建立仅 graphics 内部使用的 sampled texture identity。fixed scene/depth/velocity target、history 双槽、共享 black fallback 与 transient pool entry 分别持有稳定 identity；pool 的归还与重新取得会保留对应 backing 的 identity。TAA resolve cache 为最多八项的 LRU，仅在 reactive mask 是共享 black fallback 且五个采样 binding 均来自明确持久 backing 时启用，因此不会持有 transient mask view。history flip 可复用已见双槽；history pair 或固定 target identity 变化会清空旧 cache，防止 resize/history 重建复用 stale view。有 authored reactive command 时仍按帧创建 bind group，并允许 transient mask 正常回收到 pool。

## 2026-08-13 前向续作

离屏 target 重建现在返回明确事件；完整和简化帧入口均在重建的同一帧清空 TAA cache。history helper 也返回 `history_recreated`，新 history handle、尺寸、HZB topology、format 或 volumetric quality 重建时会在资源绑定前清空 cache；新建或重建的 history 不能继承上一帧的 valid signal。因此最多八项的 LRU 不会在 resize、camera history replacement 或 history backing 重建后继续持有旧 view。

`scene_velocity` 仅在图确实声明该资源时才被懒创建为 persistent offscreen target，而不再为未使用的 TAA/velocity 图每帧保留固定纹理。稳定帧只支付一个 target-recreate bool 分支；实际重建才会取得 mutex 并释放最多八个 cached bind group。

已增加 ignored 的真实 WGPU 产品导出入口 `export_taa_reactive_mask_wgpu_png`。它以同一 authored-material fixture 验证 inert mask 的零 pass/零写入、双 history slot 的首次建组、稳定帧 cache hit，以及 reactive mask 的一个全尺寸 pass，成功后才会写入 `docs/tests/runtime/render/render18_taa_reactive_mask_wgpu_20260813.png`。本机于 2026-08-13 执行该命令时 Cargo 在 65 秒内无编译或运行诊断即被环境时限终止；PNG 未生成，不能作为动态验收或像素正确性证据。

二次静态审查已完成：两个帧入口均消费 target-recreate 事件；history 的新建、重建和尺寸不匹配均不会继承上一帧有效位；cache key 覆盖所有五个采样 binding，且只缓存共享 black reactive mask。相关 Rust 文件的 `rustfmt --check` 已通过，定向 diff 检查未发现空白错误。动态 WGPU、PNG、GPU 时间戳和 RenderDoc 仍为待执行项，状态保持 `source_implementation_complete_dynamic_pending`。

## 验收要求

按720p/1080p/4K、reactive commands 0/1/100/10k、history cold/stable、resize/camera cut记录graph nodes、GPU render passes、mask attachment bytes、bind-group creates、CPU/GPU p50/p95：0-command mask pass=0且mask write bytes=0；有command mask pass=1而非2；TAA resolve仍恰好1 pass；若采用bind-group cache，stable resource generation create=0且resize/history slot切换不复用stale view。Off/FXAA/SMAA/MSAA/TAA fallback、history invalidation、reactive像素、透明/材质强度、camera cut与产品像素必须等价。current-source Cargo、timestamp/规模counter与RenderDoc未完成前，本批留在`pending.md`。
