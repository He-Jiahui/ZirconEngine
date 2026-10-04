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
  - tools/build/build-editor.ps1
---

# Render01: Render graph validation uses a Rust 2024 let-chain

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行记录：`docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md`
- 来源执行切片：M7 product editor bundle build
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：失败位于 Render01 新增的 compute-pass validation，低于 Editor/UI 构建层。

## 失败现象与复现证据

`tools/build/build-editor.ps1` 在 `cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked` 中失败：`zircon_runtime/src/render_graph/builder/compile.rs:231` 使用 let-chain，但 workspace crate 仍是 Rust 2021 edition。

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
- 本轮未执行原始 `tools/build/build-editor.ps1` 或受管 Cargo，不能从静态门推导 Editor bundle 已编译通过。

2026-08-24 follow-up:

- `BindingSchemaEntry` 的 buffer binding 已改为静态 range + usage 校验；复审发现新增的两个 let-chain 会再次违反 workspace Rust 2021 edition，已立即改为等价嵌套条件，没有添加兼容分支或放宽校验。
- `rustfmt --edition 2021 --check` 覆盖受影响 render-graph 与 generic-compute 文件，`compile.rs` 的 `&& let` 精确扫描和旧 buffer-offset API 扫描均通过；handoff 结构校验也通过。
- 原始 managed Editor bundle build 仍未执行，本 failure 保持 open，不能作为该 build 或任何 render milestone 的通过证据。

Open state: `rust_2021_source_repair_complete_pending_managed_editor_bundle_build`; no pass is claimed.

### 2026-09-19 rolling source-contract recheck

Successor Session `failure-roll-01a084c8-render01-rust2021-let-chain-r1` claimed
the failure record and `zircon_runtime/src/render_graph/builder/compile.rs` at
baseline epoch `611`. On the exact current source, the following lower-layer
checks passed:

- `rustfmt --edition 2021 --check zircon_runtime/src/render_graph/builder/compile.rs`;
- no `&& let` syntax remains;
- no legacy `ComputeBufferOffsetBinding` API remains;
- the buffer-range guard uses `if let Some(range) = binding.buffer_range`,
  preserves `ComputeBindingKind` matching, and retains bounds validation.

Current SHA-256 values are `621f607c2a78bccb93740401a6ec4061b5ed5388e3ba8a3cb3dddaf840b91529`
for this record and
`e39f577e041d6f04e3abccc0c7259a09844fe0603c15ca692435b036d2afed33` for the
Rust source. These are static source-contract results only; the original
managed `tools/build/build-editor.ps1` bundle build and Cargo acceptance remain
pending, so this lifecycle stays `open` and no fixed return is claimed.

The coordinator accepted static source-contract ticket
`6d84757fc9df4faaace2961f454a0610` for the exact record/source manifest. Its
PowerShell command checks the Rust 2021 syntax and buffer-range guards and runs
`rustfmt`; it explicitly defers the managed Editor bundle and Cargo gates. The
ticket is lower-layer evidence only and cannot be reused as Editor product
acceptance.

Corrected successor ticket `a5c8f12bd2c84a6b957ec5503dae3c94` (request
`failure-roll-01a084c8-render01-rust2021-let-chain-20260919-r1`) completed on
managed job/run `432a78d971c6444b8977827f48161513` with exit code 0 and marker
`RENDER01_RUST2021_SOURCE_CONTRACT_PASS`; cleanup completed. This remains
static source evidence only. The managed Editor bundle/Cargo render-graph
gates, independent review, fixed return and closeout are still pending.

### 2026-09-19 managed Cargo recheck

The existing lifecycle renewed its exact record/source leases before attempting
the deferred lower-layer gate. The approved Windows validator was invoked once
with:

```powershell
$env:CODEX_THREAD_ID='failure-roll-01a084c8-render01-rust2021-let-chain-r1:cargo-r1'
& '.\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1' -RepoRoot (Get-Location).Path -Package zircon_runtime -SkipBuild -LibTests -TestFilter 'render_graph' -VerboseOutput
```

Coordinator request `16fbfd6b648e476da415146affa4e3fb` was accepted and
terminalized as `failed` before Cargo acquisition with
`unmanaged_artifacts_detected`. The durable diagnostic identifies the external
path `D:\\ZirconBuilds\\mvp-test-fixtures-28916` (currently covered by cleanup
reservation `2026-09-19T19:23:02.803454+00:00`); no Cargo process, test result,
or RenderGraph diagnostic was produced. The coordinator-owned request remains
the sole evidence for this attempt and is not retried while that artifact is
unregistered. The managed Editor bundle, a successful RenderGraph Cargo run,
independent review, fixed return, and closeout therefore remain pending.

### 2026-09-21 independent source review (review-render01-let-chain-r1)

- The attributed `zircon_runtime/src/render_graph/builder/compile.rs` source matches the
  corrected static ticket manifest (`e39f577e041d6f04e3abccc0c7259a09844fe0603c15ca692435b036d2afed33`);
  the failure record has only the expected receipt drift. The reviewer did not absorb unrelated
  test-only `&& let` strings reported by the repository-wide scan.
- `validate_compute_pass_metadata` preserves the Rust 2021-compatible nested `if let Some(range)`
  shape, restricts ranges to uniform/storage buffer kinds, retains empty-range and out-of-bounds
  checks, and keeps the original `ComputeBufferRangeBindingNotBuffer` error fields. No legacy
  `ComputeBufferOffsetBinding` API, edition change, compatibility branch, or relaxed validation
  was found; texture-mip and resource-access validation remain in the same function.
- Independent source probes passed for absence of `&& let` and the retired offset API, nested
  buffer-range guard, all three allowed buffer kinds, empty/bounds errors, and the open failure
  marker. Scoped Rust 1.94.1 `rustfmt --check` and `git diff --check` passed.
- Independent review result: `Critical=0, Important=0, Moderate=0`. This is lower-layer
  source-contract evidence only. The managed Editor bundle, successful RenderGraph Cargo gate,
  product/upward acceptance, canonical fixed return, and coordinator closeout remain pending;
  the prior coordinator `unmanaged_artifacts_detected` attempt is not reused as a test result.
