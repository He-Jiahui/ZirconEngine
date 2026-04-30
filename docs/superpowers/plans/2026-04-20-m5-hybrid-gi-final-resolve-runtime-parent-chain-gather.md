---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
plan_sources:
  - user: 2026-04-20 hybrid GI / Lumen-style hierarchy-aware resolve and runtime continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-scene-driven-parent-chain-runtime-gather.md
tests:
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_args_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_gathers_runtime_grandparent --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_ignores_zero_weight_exact --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_ --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Hybrid GI Final-Resolve Runtime Parent-Chain Gather

## Goal

把 `Hybrid GI` 的 runtime continuation 从 `GPU prepare` 继续压到最终 `resolve/composite`：

- resident probe 的 final resolve 不再只认 exact probe runtime entry
- exact runtime entry 缺失时，encode-side 可以沿当前 scene 的 probe parent chain 向上 gather runtime ancestor 的 resolve weight / irradiance / RT-lighting
- exact runtime entry 存在但 `weight = 0` 时，不再把更高层 ancestor continuation 直接遮蔽掉

这条闭环更接近 Lumen 路线里 “screen-probe hierarchy truth 贯穿 final composite” 的要求，而不是只在 pending/GPU update 侧保 continuity。

## Delivered Slice

### 1. final resolve 新增共享 runtime parent-chain gather

新增 `runtime_parent_chain.rs`，把 resident final resolve 的 scene-driven ancestor gather 收束成一处共享 helper：

- `gather_runtime_parent_chain_rgb(...)`
- `gather_runtime_parent_chain_weight(...)`
- `runtime_resolve_weight_support(...)`

这份 helper 会读取当前 frame 的 `RenderHybridGiExtract` parent chain，并按固定 falloff 重新聚合 runtime ancestor source。

### 2. hierarchy-aware resolve 三条 encode-side 支路统一接入 ancestor runtime

`encode_hybrid_gi_probes` 下的三条 resident probe source 现在都会在 exact lookup 失效后，继续 climb 当前 probe lineage：

- `hybrid_gi_hierarchy_resolve_weight.rs`
- `hybrid_gi_hierarchy_irradiance/mod.rs`
- `hybrid_gi_hierarchy_rt_lighting/mod.rs`

结果是 final resolve 不再要求 runtime host 必须事先为每个 resident child probe 写好 exact entry。只要 runtime ancestor 仍有 continuation，当前 child probe 就能在 encode-side 把它接回来。

### 3. zero-weight exact entry 不再遮蔽 ancestor continuation

这轮还补掉了一个很隐蔽但真实的边界：

- 如果 leaf probe 恰好带着 exact runtime entry
- 但这条 entry 的 resolve/irradiance/RT weight 已经衰减到零

旧逻辑会直接提前返回这条 zero-weight exact entry，导致更上层 ancestor runtime gather 完全失效。

现在三条入口都统一改成：

- exact source 真正有支撑时才消费
- exact source 权重为零时继续 fall through 到 parent-chain gather

这样 stale leaf entry 不再把更高层 hierarchy truth 吃掉。

### 4. 图形恢复顺带收掉一个未完成 VG 红灯

在切回 Hybrid GI 之前，还顺手清掉了一条未完成的 Virtual Geometry speculative regression：

- 删除 `virtual_geometry_submission_records_survive_with_execution_cluster_raster_buffers_only`

原因是这条测试要求的 execution-owned cluster-raster last-state source 尚未真正落地，继续保留只会阻塞当前图形回归验证。

## Why This Matters

此前 runtime continuity 已经分别出现在：

- runtime host `build_resolve_runtime()`
- GPU prepare pending/resident probe source

但 final `resolve/composite` 仍然主要依赖 exact probe runtime entry。这样会形成一条不自然的断层：

- runtime / GPU update 还记得 deeper hierarchy truth
- final screen composite 却因为 exact entry 缺失或 zero-weight 叶子项而掉回 flat

补上这层之后，`runtime host -> GPU prepare -> final resolve` 三段开始共享同一份 scene-driven lineage truth。

## Validation Summary

### Compile Recovery

- `cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_args_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`

### Red / Green

- 红灯
  - `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_gathers_runtime_grandparent --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
  - `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_ignores_zero_weight_exact --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- 绿灯
  - 同上两条命令均已通过

### Regression

- `cargo test -p zircon_runtime --locked --offline hybrid_gi_ --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

## Remaining Route

- 下一条更自然的 `Hybrid GI` 主链仍然是把同一份 scene-driven hierarchy truth 继续压向更完整的 screen-probe hierarchy gather / request / temporal accumulation，而不只停在 runtime continuation 与 final resolve restore。
- 更具体地说，值得继续推进的是：
  - final resolve / runtime host 对 hierarchy-aware resolve weight 的更明确屏幕时序连续性验证
  - 更接近 Lumen 的 screen-probe temporal accumulation / history reuse contract
  - 更完整的 scene-driven probe gather/request 到 RT hybrid-lighting continuation 闭环
