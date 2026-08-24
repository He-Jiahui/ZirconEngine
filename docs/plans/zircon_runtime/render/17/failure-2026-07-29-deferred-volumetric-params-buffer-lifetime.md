---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: deferred-volumetric-params-buffer-lifetime
origin_plan: docs/plans/zircon_plugins/09-export-publishing.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_plugins/09
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
tests:
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - cargo +1.94.1 test -p zircon_runtime --test plugins09_export_validate_report --bin zircon_export_validate --locked --jobs 1 --color never -- --nocapture --test-threads=1
---

# Render17: deferred volumetric params buffer lifetime

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 来源执行切片：Plugins09 compact validate-report failure closeout current-source successor
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：Render17 owns the active Deferred allocation/bind-group optimization. Plugins09 neither owns nor may rewrite the dirty Render source that now prevents the Runtime crate from compiling.

## 失败现象与复现证据

Managed Text01 job `30759b98ee60440f870caddf9196767b` / run
`87b0cb9b5fdc4fa1b1722ae9d6979f6d` ran
`cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` against the
current shared source. Rust 1.94.1 reported E0597 at
`execute_lighting.rs:128`: `volumetric_params_buffer` is dropped at the end of the
`full_lighting_bind_group` branch while `entries` retains its binding and is consumed
later by `create_bind_group` at line 136. No Runtime test reached execution.

The same full Runtime compile boundary is a prerequisite of the pending Plugins09
successor gate, so starting that gate before this owner repair would only reproduce the
foreign compile error.

## 最低共享层根因

The allocation optimization moved `volumetric_params_buffer` from function scope into
the `full_lighting_bind_group` branch, but the borrowed `wgpu::BindingResource` is stored
in the function-scoped `entries` vector. The resource owner therefore dies before the
vector is passed to `Device::create_bind_group`. This is a Render17 ownership/lifetime
regression, not a Plugins09 schema or CLI failure.

## 架构修复验收

- Keep the volumetric params buffer owner alive through `create_bind_group` whenever the full lighting profile contributes its entries; the environment-only profile must still avoid creating or binding that buffer.
- Add a focused owner regression that covers both full-lighting and environment-only bind-group construction without replacing compiler lifetime coverage with a source-text-only assertion.
- Re-run the managed Rust 1.94.1 Runtime lib check with exit 0 and no live PIDs.
- After the owner commit is visible, re-run the pending Plugins09 exact current-source gate and its pre/post source attestation.

## 禁止临时方案

- Do not leak the buffer, clone bindings into a second truth, add an unsafe lifetime extension, or move the environment-only profile back to unconditional volumetric allocation.
- Do not add aliases, compatibility shims, silent fallback, test-only bypasses, or call-site exceptions.
- Do not weaken the Runtime check or Plugins09 acceptance command to hide the compile failure.

## 修复结果与回传

Open state: the function-scoped optional buffer owner now remains alive through
`create_bind_group`, while `EnvironmentOnlyPbrPreview` still avoids creating or binding the
volumetric buffer. A real GPU regression constructs both the default `FullScene` and
environment-only `SceneRenderer` profiles and renders a lit empty frame through the production
deferred graph path; this reaches WGPU bind-group construction rather than replacing it with a
source-text assertion.

Immutable snapshot `1264` freezes the failure record and
`execute_lighting.rs` at SHA-256
`06c9b1d43130281a541fbd1ee7cf8f4fecbb8dbaeed8da5bbada1a45d08aae44`.
Independent review is Ready with Critical 0 / Important 0 / Moderate 0 / Minor 0 and zero
start/end drift. Managed current-source execution and a stable commit SHA remain pending behind
the open Render17 `scene-viewport-surface-projection-drift` and Runtime11 operation compile
repairs; no pass or fixed return is claimed before those lower gates release.

## 产出记录与时间

| 时间 | 范围 | 状态 | 完成项与后续门禁 |
| --- | --- | --- | --- |
| 2026-07-29 13:01 CST | deferred volumetric params buffer lifetime | implementation/review complete, validation pending | Buffer owner hard-cut and real FullScene/EnvironmentOnly WGPU render regression are frozen in snapshot `1264`; independent review C0/I0/M0/m0 Ready. Await lower Render17 surface and Runtime11 operation fixed returns, then run a fresh managed current-source GPU/lib gate and create the fixed return. |
