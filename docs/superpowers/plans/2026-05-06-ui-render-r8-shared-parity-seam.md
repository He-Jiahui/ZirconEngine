# UI Render R8 Shared Parity Seam Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an R8 shared renderer parity packet in `zircon_runtime_interface` so runtime and editor can compare canonical paint, batch, clip, resource, and text identities without touching active renderer or painter implementation files.

**Architecture:** Keep ownership in the neutral interface crate under `ui::surface::render`. Add a focused `parity.rs` module and wire it into `UiRenderDebugSnapshot`; runtime/editor remain later consumers and are not edited in this slice.

**Tech Stack:** Rust, serde DTOs, focused `zircon_runtime_interface` render contract tests, repository-local milestone-first validation.

---

## Current Baseline

- R1-R7 render contracts are present in `zircon_runtime_interface/src/ui/surface/render`: paint, brush, batch, cache, debug, visualizer, and shaped text DTOs.
- R7 focused validation passed with 19 `render_contracts` tests.
- Active sibling sessions own editor painter/debug reflector, Material `.ui.toml` layout, and broader runtime/editor diagnostics. R8 must stay in the interface crate for this slice.

## Files

- Create: `zircon_runtime_interface/src/ui/surface/render/parity.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/render/mod.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/mod.rs`
- Modify: `zircon_runtime_interface/src/ui/surface/render/debug.rs`
- Modify: `zircon_runtime_interface/src/tests/render_contracts.rs`
- Modify: `docs/zircon_runtime_interface/ui/surface/render.md`
- Modify: `docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md`
- Modify: `.codex/sessions/20260506-0520-ui-render-slate-contract.md`

## Milestone 1: Interface Parity Contract

- [x] Add contract tests for `UiRendererParitySnapshot::from_render_extract(...)` covering mixed brush/image/text commands, batch order, clip keys, resource identity, and text render mode.
- [x] Add contract test proving `UiRenderDebugSnapshot` carries the parity packet and deserializes legacy snapshots with an empty default parity packet.
- [x] Implement `parity.rs` with `UiRendererParitySnapshot`, `UiRendererParityPaintRow`, `UiRendererParityBatchRow`, `UiRendererParityPayloadKind`, and `UiRendererParityStats`.
- [x] Wire `UiRenderDebugSnapshot.parity` with `#[serde(default)]` and derive it beside batches/cache/visualizer.
- [x] Export the new types through `render/mod.rs` and `surface/mod.rs` only; keep root files structural.
- [x] Update render docs and audit docs to mark R8 as a shared parity seam, not renderer cleanup completion.

## Testing Stage

- [x] If E: free space is `<= 50 GB`, run `cargo clean --target-dir "E:\zircon-build\targets\ui-render-r4-interface"` before Cargo validation. 2026-05-06 19:28 +08:00: E: was below the threshold, so `cargo clean --target-dir "E:\zircon-build\targets\ui-render-r4-interface"` removed 755.6 MiB.
- [x] Run `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/render/debug.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs" "zircon_runtime_interface/src/ui/surface/render/mod.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime_interface/src/tests/render_contracts.rs"`. 2026-05-06 19:29 +08:00: the final closeout command also included `zircon_runtime_interface/src/ui/layout/slot.rs` and `zircon_runtime_interface/src/ui/layout/linear_sizing.rs`; it passed with no output.
- [x] Run `CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never`. 2026-05-06 19:32 +08:00: finished dev profile successfully.
- [x] Run `CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture`. 2026-05-06 19:34 +08:00: passed, `21 passed; 0 failed; 31 filtered out`.
- [x] Record exact validation evidence in docs/session before closeout.

## Self-Review

- Spec coverage: The plan covers the approved R8 shared seam only and excludes runtime/editor implementation cleanup.
- Placeholder scan: No TBD/TODO placeholders are present.
- Type consistency: Public type names use the `UiRendererParity*` prefix and stay distinct from R7 `UiRenderVisualizer*` DTOs.
