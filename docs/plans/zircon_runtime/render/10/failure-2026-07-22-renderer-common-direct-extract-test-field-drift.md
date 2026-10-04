---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: renderer-common-direct-extract-test-field-drift
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/render/10-renderer-family.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/render/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
tests:
  - cargo test -p zircon_runtime --lib native_callback_can_reenter_live_host_descriptor_without_deadlock --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
---

# Render10：RendererCommon hard-cut 后 direct extract 测试仍读取旧字段

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行者：`plugins01-native-callback-stable-owner-r1-20260722`
- 来源执行切片：native callback stable-owner focused Windows lib-test
- 修复责任计划：`docs/plans/zircon_runtime/render/10-renderer-family.md`
- 交接原因：`RenderMeshSnapshot` 的 `RendererCommon` hard-cut、场景提取夹具与字段投影由 Render10 活跃会话持有；Plugins01 不应在插件回调验证中越权改写渲染契约。

## 失败现象与复现证据

托管 reservation `226e5974212640de8a81b5db858f1d5b`、job
`9f186eaafc6946748b1f07ded964d17e`、run `c2b30ba68dca4763bc05ad56cb621ee0`
执行：

```text
cargo test -p zircon_runtime --lib native_callback_can_reenter_live_host_descriptor_without_deadlock --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
```

在 Plugins01 已识别并修复测试支撑导入/可见性错误后，编译尾部仍稳定报告：

```text
zircon_runtime/src/scene/tests/render_extract/direct_sections.rs:84:14:
error[E0609]: no field `render_layer_mask` on type `&scene_extract::RenderMeshSnapshot`
```

当前 `RenderMeshSnapshot` 已只公开 `common: RendererCommon`；同一测试中的 sprite
断言以及其他 render-extract 测试均通过 `common.layer_mask` 读取统一渲染层。第 84 行仍直接读取
hard-cut 前的 `dynamic_row.render_layer_mask`，使所有 `zircon_runtime --lib` 测试目标在执行
focused assertion 前失败。

## 最低共享层根因

RendererCommon 字段收敛已修改生产结构，但 direct extract 夹具中的 mesh 断言未同步到新的
单一字段 owner，导致生产类型与测试镜像漂移。这是 Render10 hard-cut 的最底层测试消费者回归，
不是 native callback 实现错误。

## 架构修复验收

- 将 mesh 层掩码断言切到 `dynamic_row.common.layer_mask`，与 `RenderMeshSnapshot` 当前单一 owner 对齐。
- 保留 scene-schema v1 mask 的 lossy 投影断言值 `0b0010`，不得删减行为覆盖。
- 搜索 RenderMeshSnapshot 消费者，确认没有其他已删除的顶层 renderer-common 字段读取。
- 运行 Render10 focused extract tests，并重跑上述 Plugins01 原始 `zircon_runtime --lib` 向上门。

## 禁止临时方案

- 不得为通过旧测试把 `render_layer_mask` 冗余字段重新加回 `RenderMeshSnapshot`。
- 不得在 Plugins01 测试过滤或 feature gate 中绕过 scene 测试模块编译。
- 不得删除层掩码行为断言。

## 修复结果与回传

Resolving state：Render10 活跃 owner 已把 direct mesh 断言同步为
`dynamic_row.common.layer_mask.to_scene_schema_v1_mask_lossy() == 0b0010`，没有恢复冗余字段，
Rust `1.94.1` scoped rustfmt 与 `git diff --check` 已通过。当前仍待 Render10 focused extract
测试与 Plugins01 原始 focused lib-test 向上门共同 GREEN；完成前本 failure 保持 `open`。

## 2026-09-11 rolling repair continuation

- Stable fixing Session `failure-roll-01a090ae-render10-direct-extract-field-r1` at
  baseline epoch `608` attributed the exact current hashes for the Render10 facade,
  `direct_sections.rs`, and this failure record before validation. The current
  `RenderMeshSnapshot` declaration remains owned by `common: RendererCommon`; the direct mesh
  assertion retains the scene-schema-v1 lossy projection and expected mask `0b0010`. The other
  `direct_sections.rs` access to `visibility.renderables[*].render_layer_mask` is a separate
  renderable-visibility contract and was intentionally preserved.
- A local current-source structural check printed
  `RENDER10_DIRECT_EXTRACT_FIELD_PASS`. Coordinator-managed static ticket
  `713d872e83684e5b88becc687457fd11` sealed manifest
  `4ff782582a8b60085377923e68552bcb8d2b9b019745db530396532d89a50b40` and remains
  `queued`; this is durable admission evidence, not a passing result.
- Managed Cargo request
  `failure-roll-01a090ae-render10-cargo-20260911-r1` used the original Plugins01 focused
  `zircon_runtime --lib` test with `core-min`, `--locked`, and no caller-owned `--jobs` flag.
  Coordinator admission rejected it before ticket creation or Cargo execution with
  `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`. Consequently there is no
  Cargo process, exit code, focused dynamic GREEN, fixed return, closeout commit, or notification
  to claim. Resume only after the external repository owner supplies a clean pinned revision;
  until then this failure remains `open` and the Session waits on managed validation.

### 2026-09-18 static ticket terminal result

The managed static ticket `713d872e83684e5b88becc687457fd11` reached terminal
`passed` with job `49180189b51545018f8fec5916eb1a63`, run
`713d872e83684e5b88becc687457fd11`, and exit code `0`. It emitted
`RENDER10_DIRECT_EXTRACT_FIELD_PASS` with no coordinator blockers. This is
static evidence only; the exact Render10 focused Cargo and Plugins01 upward

### 2026-09-19 current-session admission reconciliation

- A fresh current-source static submission was attempted from Session `failure-roll-01a084c8-render10-direct-extract-r2` after the prior manifest drifted. The coordinator rejected admission with `validation_copy_overlay_not_owned` because the successor Session write scope contains this failure record but does not own/attribute `zircon_runtime/src/core/framework/render/scene_extract.rs` or `zircon_runtime/src/scene/tests/render_extract/direct_sections.rs`.
- No validation copy, job, run, or test result was created by the rejected request. The prior static ticket remains non-reusable as a whole because its manifest included the older failure-record hash; no dynamic GREEN is claimed.
- Source ownership must be reconciled through the coordinator before a fresh static ticket can be submitted. Focused Render10 Cargo, Plugins01 upward acceptance, independent Critical/Important/Moderate review, canonical return, and closeout remain pending.

## 2026-09-26 Render10 r3 current-source/static receipt

- Successor Session `failure-roll-01a084c8-render10-direct-extract-r3` registered the
  Render10 fixing plan with the plan, failure record, renderer facade, and direct-extract
  fixture in its exact write scope. The audited ownership transfer fingerprint was
  `67320efc12d413f0490c7ce1345af5cf60d7dbf8284b8a7872c3eae94ccfce08`; both previously
  archived-owner source paths were eligible and transferred without edits.
- Current-source snapshot `3877` (baseline epoch `611`) froze the following hashes:
  `docs/plans/zircon_runtime/render/10-renderer-family.md`=`d6fcb4eb094555f53010335a4936654cfca44def480f8ba726caf2bad73d2663`;
  this failure record before this receipt=`94b281de34f871609a8069efb82811f1eb7321569244e0327a4d4a9380ba4632`;
  `zircon_runtime/src/core/framework/render/scene_extract.rs`=`3378b7c127f5c23e3c69cba6f7f015e897b9b95343ca2ab7ac528ee6c8ff6736`;
  `zircon_runtime/src/scene/tests/render_extract/direct_sections.rs`=`4bb395b50479edbe5572ece164fc257e0a829c159f47158db27e0652bb682123`.
- The exact current-source probe emitted `RENDER10_DIRECT_EXTRACT_FIELD_PASS`;
  `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` for both
  source paths exited `0`, and scoped `git diff --check` exited `0` (only the existing
  CRLF normalization warning was printed). The probe confirms the mesh assertion reads
  `dynamic_row.common.layer_mask.to_scene_schema_v1_mask_lossy() == 0b0010`, does not
  reintroduce `dynamic_row.render_layer_mask`, and leaves the visibility renderable
  contract separate.
- Fresh coordinator static ticket `20e3e7c5e789451da7acf4080b59ddc8` for request
  `render10-direct-extract-current-static-20260926-r3` ran as job
  `eeb6b491d4004b99815148c3cd52cf87` / run equal to the ticket, exited `0`, and emitted
  `RENDER10_DIRECT_EXTRACT_FIELD_PASS`. Its manifest is the four-path snapshot above;
  `executionKind=executed`, `terminalStatus=passed`, and `failureCategory=coordinator`.
- The first independent review correctly reported `Critical=0`, `Important=1`,
  `Moderate=0` only because this receipt was not yet present in the failure record. The
  reviewer also confirmed the semantic assertion and both source hashes. A post-receipt
  review is required before this static handoff is considered review-clean.
- This receipt is static/source evidence only. The exact Render10 focused extract gate and
  the original Plugins01 upward `zircon_runtime --lib` gate remain pending; the prior
  managed request was rejected at admission by dirty external `E:\Git\zr_vm`, with no
  Cargo job or test-level result. Canonical `fixed-*` return, closeout, and WeCom remain
  pending, so this failure stays `open`.

## 2026-09-26 independent review receipt

- Reviewer Session `review-editor03-gizmo-private` completed a read-only post-receipt
  review against snapshot `3879`. The plan, failure record, and both source hashes matched
  the frozen four-path manifest; the reviewer independently reconciled ticket
  `20e3e7c5e789451da7acf4080b59ddc8`, job/run `eeb6b491d4004b99815148c3cd52cf87`, exit
  code `0`, `executionKind=executed`, `terminalStatus=passed`, and the
  `RENDER10_DIRECT_EXTRACT_FIELD_PASS` marker.
- The review confirmed the direct mesh assertion uses the canonical
  `dynamic_row.common.layer_mask.to_scene_schema_v1_mask_lossy() == 0b0010` projection,
  does not restore a forbidden top-level `render_layer_mask`, and leaves the separate
  visibility-renderable mask contract intact. Review result: `Critical=0`, `Important=0`,
  `Moderate=0`.
- This is an independent static review only. Focused Render10 Cargo, Plugins01 upward
  acceptance, the external `E:\Git\zr_vm` clean-revision prerequisite, canonical fixed
  return, closeout, and WeCom remain pending; the failure remains `open`.
