---
handoff_kind: failure
status: open
created_at: 2026-08-08
summary_slug: render-graph-rust-2021-let-chain
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/render_graph/builder/compile.rs
tests:
  - tools/build-editor.ps1
---

# Render01: Render graph validation uses a Rust 2024 let-chain

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行记录：`docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md`
- 来源执行切片：M7 product editor bundle build
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：失败位于 Render01 新增的 compute-pass validation，低于 Editor/UI 构建层。

## 失败现象与复现证据

`tools/build-editor.ps1` 在 `cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked` 中失败：`zircon_runtime/src/render_graph/builder/compile.rs:231` 使用 let-chain，但 workspace crate 仍是 Rust 2021 edition。

## 最低共享层根因

Compute binding 的 buffer-offset validation 把 `if let` 与 kind guard 写成仅 Rust 2024 可用的 let-chain；逻辑本身不需要 edition 升级。

## 架构修复验收

- 保持原有 buffer binding 类型校验和错误信息不变。
- 用 Rust 2021 支持的嵌套条件表达同一约束。
- 原始 Editor bundle production build 通过。

## 禁止临时方案

- 不升级单 crate edition 来掩盖语法错误。
- 不删除或放宽 compute binding validation。
- 不添加兼容分支、静默 fallback 或测试绕过。

## 修复结果与回传

2026-08-10 current-source result:

- `validate_compute_pass_metadata` 已将 buffer-offset kind guard 收敛为 Rust 2021 支持的嵌套 `if let Some(offset)` + `matches!`，保持原 binding kind 约束与 `ComputeBufferOffsetBindingNotBuffer` 错误字段不变。
- 当前 `zircon_runtime/src/**/*.rs` 源码扫描未发现 `&& let` 形式；`rustfmt --edition 2021 --check zircon_runtime/src/render_graph/builder/compile.rs` 与 scoped source contract 通过。
- 本轮未执行原始 `tools/build-editor.ps1` 或受管 Cargo，不能从静态门推导 Editor bundle 已编译通过。

2026-08-24 follow-up:

- `BindingSchemaEntry` 的 buffer binding 已改为静态 range + usage 校验；复审发现新增的两个 let-chain 会再次违反 workspace Rust 2021 edition，已立即改为等价嵌套条件，没有添加兼容分支或放宽校验。
- `rustfmt --edition 2021 --check` 覆盖受影响 render-graph 与 generic-compute 文件，`compile.rs` 的 `&& let` 精确扫描和旧 buffer-offset API 扫描均通过；handoff 结构校验也通过。
- 原始 managed Editor bundle build 仍未执行，本 failure 保持 open，不能作为该 build 或任何 render milestone 的通过证据。

Open state: `rust_2021_source_repair_complete_pending_managed_editor_bundle_build`; no pass is claimed.
