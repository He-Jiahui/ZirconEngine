---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-17 continue the next M5 slice without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-traced-radiance-source-uplift.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_normalizes_multi_region_radiance_instead_of_additive_saturation --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Normalized Multi-Region Gather

**Goal:** 在已经落地的 trace-region-localized radiance source 之上，把 `Hybrid GI` 的 GPU completion 再推进一层，让多个 trace region 同时影响同一 probe 时输出归一化后的 radiance blend，而不是继续做简单累加导致的亮度膨胀或通道饱和。

**Non-Goal:** 本轮仍然不实现真正 scene-driven radiance cache、screen probe hierarchy、hardware RT tracing 或更深的 Lumen-like scene representation。

## Delivered Slice

- `update_completion.wgsl::traced_contribution_rgb(...)` 不再把每个 trace region 的 RGB 贡献直接累加到 `accumulated`。
- shader 现在会对每个有效 trace region 保留：
  - `base_rgb * contribution_weight` 的 weighted sum
  - `total_weight`
- 最终 radiance source 改为 `weighted_sum / total_weight` 的归一化结果，因此：
  - 多个 region 会形成真正的加权 blend
  - 不会因为 trace count 增加就无条件把 probe 推向更亮的结果
  - `no-trace => 0 contribution` 的既有 contract 保持不变
- CPU 侧 `hybrid_gi_gpu.rs` 期望 helper 已同步到同一归一化 gather 公式，保证 GPU readback 回归继续绑定真实 shader 行为。

## Why This Slice Exists

- 上一轮 traced radiance-source uplift 已经把 probe irradiance 从 seeded placeholder 推进到 trace-region-localized spatial contribution，但在多个 region 同时命中时仍然是直接累加。
- 直接累加容易把多 region gather 变成“更多 trace region = 更亮”，这更接近 debug energy stacking，而不是真正的 radiance blend。
- 把多 region gather 改成归一化权重平均后，Hybrid GI completion 的 source 更接近真实 radiance-cache gather，为后续 scene-driven cache 或更深 tracing 路径预留了更健康的行为边界。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_normalizes_multi_region_radiance_instead_of_additive_saturation`
  - 证明两个 trace region 同时命中同一 pending probe 时，最终 luma 会留在单 region 结果的同一亮度带内，而不是 additive saturation
- `hybrid_gi_gpu`
  - 证明 normalized gather 没有破坏既有的 probe/trace completion、history-sensitive resident update、no-trace resident-history preserve 与 pending-black contract
- `hybrid_gi`、`cargo test -p zircon_graphics --lib --locked`、`validate-matrix.ps1 -Package zircon_graphics`
  - 证明这次 shader-level gather 改动没有回归 runtime host、resolve render、render-server bridge 或其他 M4/M5 功能族

## Remaining Route

- scene-driven radiance cache / probe gather，而不是当前 quantized trace-region heuristic scene representation
- tracing/update completion source 与 post-process resolve 的更深层闭环
- capability-gated RT hybrid lighting
