---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-29
summary_slug: gpu-timestamp-feature-set-const
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/gpu_pass_timer.rs
tests:
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - cargo test -p zircon_runtime --lib offscreen_device_features_request_gpu_timestamps_only_when_fully_supported --locked --jobs 1 --color never -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --test plugins09_export_validate_report --bin zircon_export_validate --locked --jobs 1 --color never -- --nocapture --test-threads=1
---

# Render17: GPU timestamp feature set const construction

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 default-feature Runtime lib current-source recovery gate
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：GPU timestamp device capability construction belongs to the Render17 profiling owner. Text01 and the pending Plugins09 gate must not patch or bypass this renderer compile boundary.

## 失败现象与复现证据

Managed job `29f23fccc4b74ee88b1422d763667d29` / run
`a5986ba06f614ea98da451b03da1ce96` ran
`cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` under
Rust 1.94.1 and naturally terminated with exit 101. The sole compiler error was E0015
at `gpu_pass_timer.rs:6`: wgpu 29's bitflags `BitOr` implementation is not const, so
`TIMESTAMP_QUERY | TIMESTAMP_QUERY_INSIDE_ENCODERS` cannot initialize a constant.

The earlier Text04 retry-count and Render17 `BTreeSet` failures were absent from this
run. No Runtime tests executed.

## 最低共享层根因

`GPU_TIMESTAMP_REQUIRED_FEATURES` is intentionally the single compile-time feature-set
authority used by adapter negotiation and timer construction. Its value is valid, but
the expression used a trait operator that wgpu's bitflags implementation does not make
const. The bitflags API already supplies the const-safe `union` constructor, so no
runtime branch or duplicate bit mask is required.

## 架构修复验收

- Construct the authoritative feature set with the const-safe bitflags API while retaining both timestamp capabilities.
- Keep adapter negotiation all-or-nothing: partial support must not enable either timestamp feature.
- Re-run the managed Rust 1.94.1 Runtime lib check and the focused adapter feature-selection regression.
- After current-source stability is attested, re-run the pending Plugins09 upward gate.

## 禁止临时方案

- Do not demote the constant to mutable runtime state, duplicate raw feature bits, or request only one timestamp capability.
- Do not add a compatibility alias, fallback feature path, conditional compilation bypass, or test-only exception.
- Do not weaken the Runtime or Plugins09 validation commands to avoid compiling Render17.

## 修复结果与回传

Open state: implementation uses the const-safe feature-set constructor, and both scene/offscreen
and retained-UI device negotiation reuse that single all-or-nothing feature authority. Retained UI
now requests no timestamp features while its descriptor-level timing switch is off; an explicit
profiling descriptor requests the complete feature set only when the adapter supports all required
bits. Managed current-source compile and focused regression evidence remain pending.
