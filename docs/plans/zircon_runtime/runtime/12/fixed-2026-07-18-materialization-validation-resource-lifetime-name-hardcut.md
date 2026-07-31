---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: materialization-validation-resource-lifetime-name-hardcut
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/render_graph/graph.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1
resolved_at: 2026-07-18
---


# Render01：materialization validation resource lifetime name hard cut

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行切片：Runtime12 current-source canonical library check
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：最低共享原因是 Render01 的 `CompiledRenderGraph` lifetime lookup 已完成 typed-resource 硬切，但 Render01 materialization validation 留有两个 name caller 未迁移。

## 失败现象与复现证据

Runtime12 managed diagnostic job `0a71bc4d34c24ccf88fc890f4b84db62` / run `df0d7cc83c8541b3ac72b6aaac6977d8` 执行 `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1` 时终止为 exit 101，目标 Runtime12 gate 未执行。编译器在 `materialization_validation.rs:68,72` 报告两个 `E0308`：`CompiledRenderGraph::resource_lifetime` 需要 `RenderGraphResource`，两个 filter closure 却传入 `&&str`。该 job 的 source manifest 后续因 `ui_surface.rs` rustfmt 漂移，仅作为诊断证据，不作为 source-bound acceptance。

## 最低共享层根因

`CompiledRenderGraph::resource_lifetime(RenderGraphResource)` 已硬切为 typed-resource owner；stale-binding validation 的两个 name-based caller 没有迁移到既有的 `resource_lifetime_by_name(&str)`。后者先通过 declaration name index 找回 typed resource，再委托 typed lifetime owner，不需要恢复旧签名或构造猜测性 handle。

## 架构修复验收

- 两个 stale-binding closure 直接调用 `resource_lifetime_by_name(name)`，并同步其 source guard。
- `rustfmt +1.94.1 --check` 与 exact diff check 通过，且 Render01 变更中没有兼容 shim、alias 或 typed owner 回退。
- Runtime12 以完整 current dirty runtime source manifest 重跑 canonical library check 并返回 source-bound exit 0；在此之前不声称 failure fixed。

## 禁止临时方案

- 不得恢复 `resource_lifetime(&str)`、添加兼容 shim/alias、静默 fallback、重复 name map、测试绕过或调用点特例。
- 不得构造或猜测 `RenderGraphResource`，不得回退 typed lifetime owner。
- 不得削弱 Runtime12 或 Render01 验收条件来隐藏编译失败。

## 修复结果与回传

- 根因：Render01 completed the CompiledRenderGraph lifetime lookup hard cut to resource_lifetime(RenderGraphResource), but materialization stale-binding validation retained two name-based callers that passed &&str into the typed owner.
- 架构修复：Migrated both stale-binding callers directly to the existing resource_lifetime_by_name(&str) indexed owner and synchronized the same-file source guard; retained the typed RenderGraphResource owner with no compatibility shim, alias, fallback, or guessed handle.
- 验证：`rustfmt +1.94.1 --check` 与 exact `git diff --check` 通过；独立 review C0/I0/M0。Runtime12 source-bound canonical job `f6841642e70c4a43b8674c92f9f18461` / run `230eaecd12ce4bfe97d92753efff6cdc` 以 reservation `6dd22367dd5041b88ad27c024ceb07ec` 执行 `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1`；630-path manifest fingerprint 为 `42216a3cdd88bbed30369bcce67ad5698effc2dfccfca6f08722e22ce27b1e44`，post-run 630/630 hashes match 且 relevant dirty outside manifest 为 0；21.53 秒后 released exit 0，`live_process_pids=[]`。
- 回传：Runtime12 current-source canonical library check can resume past the Render01 materialization lifetime lookup compile blocker; no Runtime12 or Render17 source was absorbed.
