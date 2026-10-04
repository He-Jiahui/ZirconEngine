---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: gpu-timestamp-feature-set-const
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs
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

### 2026-08-27 hard-cut ownership anchor repair

Commit `cb62fe090eb917ebd59fc3aea5d3c01d52093782` moved the timer implementation
into the `zr_rhi_wgpu` crate. The timer `related_code` anchor above now names the
existing post-cut file. This repair changes failure ownership metadata only: it does
not modify the live timer implementation or add managed compile/GPU evidence, so the
failure remains open.

### 2026-09-01 current-source managed validation

The post-cut implementation constructs `GPU_TIMESTAMP_REQUIRED_FEATURES` with the
const-safe bitflags `union` API and keeps the capability gate all-or-nothing. A
managed validation copy proved the owning `zr_rhi_wgpu` regression green:

- copy: `4c2d9e851eaf447ebc2e68e00e9b68e0`
- immutable input manifest: `b1432da58092a086b35b08ad5b01b366749bc492f86a555107e2e636e96275ac`
- run: `b8d5180e82be4bde809919fb47ad0411`
- command: `cargo +1.94.1 test -p zr_rhi_wgpu --lib render_perf_gpu_timer_capability_gate --locked --jobs 1 -- --nocapture --test-threads=1`
- result: exit 0; 1 passed, 0 failed, 410 filtered out

The required Runtime-level gate has not yet run. Copy
`a53a4a41151e484896632e0ddfc1985f` failed closed during `closure_planning`, before
Cargo, with `validation_copy_compile_time_resource_missing`:

- source: `zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs`
- missing resource: `zircon_runtime/src/core/resource/tests.rs`

Those paths belong to another Runtime failure chain and were not modified here.
This failure therefore remains open pending repair of that stale compile-time resource
edge, followed by the Runtime and Plugins09 upward gates listed above.
