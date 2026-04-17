---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/extract_registration.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/extract_registration.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-17 Hybrid GI should replace runtime default irradiance_rgb with real GPU radiance-cache output
  - user: 2026-04-17 continue the remaining M5 milestones without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-temporal-radiance-cache-update.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_builds_prepare_frame_without_host_bootstrap_irradiance --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Runtime Bootstrap Removal

**Goal:** 把 `Hybrid GI` runtime host 从“先塞入一份 host-side 默认 irradiance，再等待 GPU 覆盖”推进到“只导出真实 GPU radiance-cache history；没有 GPU history 时保持未初始化黑值”的边界。

**Non-Goal:** 本轮不实现新的 GI shader、trace kernel 或 screen-probe gather；这里只收口 runtime bootstrap policy，不扩展 renderer pass 形态。

## Delivered Slice

- 删除 `runtime/hybrid_gi/default_probe_irradiance_rgb.rs` 这条 host-side 默认色路径。
- `HybridGiRuntimeState::register_extract(...)` 不再在 resident probe 注册时主动插入默认 `irradiance_rgb`。
- `HybridGiRuntimeState::build_prepare_frame()` 现在只会导出：
  - 已经由 GPU completion pass 写回到 runtime cache 的真实 `irradiance_rgb`
  - 对尚未收到 GPU history 的 resident probe 导出 `[0, 0, 0]`

## Runtime Contract

- runtime host 现在不再伪装自己“已经持有 radiance-cache 颜色”。
- 真实颜色生命周期变成：
  1. extract/register 只建立 probe budget、ray budget 与 residency
  2. render 完成后，GPU completion readback 通过 `complete_gpu_updates(...)` 回写 `probe_irradiance_rgb`
  3. 下一帧 `build_prepare_frame()` 才把这些 GPU-produced 颜色重新导出给 renderer/post-process
- 这让 `Hybrid GI` runtime host 与 `Virtual Geometry` runtime host 的 ownership 边界更一致：host 负责缓存与调度，不负责伪造 renderer 资源结果。

## Why This Slice Exists

- 前面的 `Hybrid GI` radiance-cache update 和 lighting resolve 已经把真实 GPU output 接进了主链，但 runtime host 仍然保留一条 `default_probe_irradiance_rgb(...)` bootstrap 路径。
- 这意味着首帧 resident probe 依然可能带着 host 默认色进入 prepare snapshot，与“真实 GPU output 是唯一 radiance source”的目标不完全一致。
- 去掉这条默认色后，remaining placeholder 明显减少，后续 traced radiance kernel 或更深的 temporal policy 可以直接围绕 GPU history 演进。

## Validation Summary

- `hybrid_gi_runtime_state_builds_prepare_frame_without_host_bootstrap_irradiance`
  - 证明 resident probe 在没有 GPU history 时会导出黑值，而不是旧的默认色。
- `hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule`
  - 证明一旦 GPU completion 回写真实 `probe_irradiance_rgb`，下一帧 prepare snapshot 仍会稳定导出这些颜色。
- `cargo test -p zircon_graphics hybrid_gi --locked`
  - 证明 runtime bootstrap policy 收口后，没有破坏 Hybrid GI 的 visibility、GPU completion 和 post-process lighting resolve。
- `cargo test -p zircon_graphics --lib --locked`
  - 证明这条改动没有破坏其他 M4/M5 行为层与 render server bridge。

## Remaining Route

- 用真实 traced radiance gather 替换当前 deterministic scene-seeded update kernel
- 引入 probe confidence / hysteresis / temporal reuse，而不是只靠当前 GPU history cache
- 与 RT hybrid lighting、screen probe、scene representation 的更深层联合路径
