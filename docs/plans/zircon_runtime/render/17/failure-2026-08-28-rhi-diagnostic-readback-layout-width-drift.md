---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: rhi-diagnostic-readback-layout-width-drift
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/diagnostics.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/layout.rs
tests:
  - cargo +1.94.1 check -p zr_rhi_wgpu --lib --locked --jobs 1 --color never
  - cargo +1.94.1 test -p zircon_runtime --lib text_cache_indexes_keep_hot_lookup_and_eviction_work_constant --locked --jobs 1 --color never -- --test-threads=1
---

# Render17: RHI diagnostic readback layout width drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：Editor11 binary direct-decode 的 Text09 原始上行复验
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：texture/buffer diagnostic readback 的 row-byte 计算、对齐和 WGPU copy layout 由 Render17 的 RHI diagnostic owner 负责；Editor11 不能修改未归属的 RHI current source 来放行 serialization 上行测试。

## 失败现象与复现证据

Managed job `e91cafa704a644878089247129b8c8dd` 执行
`text_cache_indexes_keep_hot_lookup_and_eviction_work_constant` 上行复验，于
2026-08-28 01:06:53 CST 正常 `released`，exit `1`，`live_process_pids=[]`。
`zircon_runtime_interface` 已编译且没有 direct-decode 诊断，但目标测试执行数为 0；
`zr_rhi_wgpu` 先报三处 E0308：

- `production/device/diagnostics.rs:665`
- `production/device/diagnostics.rs:722`
- `production/device/diagnostics.rs:771`

三处调用把 `u32 copy_row_bytes` 传给
`DiagnosticTextureReadbackLayout::new(unpadded_bytes_per_row: u64, height: u32)`。
观测边界上，调用文件与 layout 文件均为 untracked current source，ownership matrix
显示 `attribution_missing`，无 lease、无 active Session scope；来源 owner未修改这两个文件。

## 最低共享层根因

readback layout authority 已把未对齐 row byte 宽度提升为 `u64`，但 device diagnostic
调用者仍保留 `u32` 中间表示。接口迁移没有在同一 owner transaction 中完成，导致所有经过
`zr_rhi_wgpu` 的上行 crate 在测试执行前停止。编译器建议的 `.into()` 只处理类型，不证明
texture extent、bytes-per-pixel 乘法和 WGPU 对齐后的范围/截断语义正确。

## 架构修复验收

- row-byte 计算从源头使用 checked `u64`，或在已证明不溢出的边界做显式 checked conversion；不得先在 `u32` 中溢出再拓宽。
- texture、depth 和 buffer diagnostic readback 三条路径统一消费同一 layout authority；WGPU API 所需的窄类型只在最终边界校验后转换。
- 回归覆盖最大合法 extent、乘法溢出、对齐溢出和零/非法布局，typed error 不得退化为 panic、截断或静默空读回。
- managed `zr_rhi_wgpu` lib check GREEN 后，重跑本记录中的 Text09 上行命令并实际执行目标测试。

## 禁止临时方案

- 禁止只在三处调用后追加未经范围证明的 `as u32`/`as u64`、`unwrap` 或饱和截断。
- 禁止复制 layout 计算、绕过 diagnostic readback、关闭 WGPU diagnostic feature，或降低上行测试范围来隐藏编译错误。
- 禁止把两份未跟踪 current-source 文件吸收到 Editor11/Coordinator01 commit；必须先由 RHI owner 完成正式 attribution、lease 和 source-bound validation。

## 修复结果与回传

Open state: `unowned RHI current-source contract drift routed / owner repair and managed validation pending`; no RHI or Text09 pass is claimed.

