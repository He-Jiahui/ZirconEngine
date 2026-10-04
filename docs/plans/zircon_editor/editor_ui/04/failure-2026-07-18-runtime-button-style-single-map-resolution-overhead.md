---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-button-style-single-map-resolution-overhead
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/tests/material_button_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
---

# Runtime button style单表解析重复物化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui` root/binding/event_ui/theme/tree生产文件28/28
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md`
- 联动责任：EditorUI08的presentation/style generation消费同一immutable theme snapshot；回链PERF-MVP-161/182/183。
- 交接原因：单次解析止损属于runtime helper，跨节点/帧的style与theme generation缓存属于EditorUI04。

## 失败现象与复现证据

PERF-MVP-251：button单表解析原先clone完整values map、构造Arc/Weak，并让17个属性逐项upgrade；字符串枚举再复制和lowercase。本轮已直接改为借用一次map，保留旧trait必实现入口并让内置属性转发borrowed values，消除Arc/Weak和lowercase分配。

## 最低共享层根因

runtime helper把已解析的单一values map重新包装为完整scope图，再通过通用多scope property链读取；上层又没有按style/theme/state generation保存resolved结果，导致结构复制和重复解析同时存在。

## 架构修复验收

- 按`style document + theme + component semantic state generation`缓存`ResolvedButtonStyle`，stable generation不重复解析。
- changed node compile从同一immutable theme/style snapshot借用值；不得在每个property或paint helper读取全局锁。
- 1/100/10k nodes记录map clone bytes、Arc upgrade、String alloc、property lookup、resolve count、cache bytes/hit/miss与CPU p50/p95。
- 单次resolve除custom style/role必要所有权外map clone、Arc upgrade、lowercase alloc均为0；stable generation resolve=0。
- variant/color/size/icon/state/loading/disabled、custom hex/style name、serde与editor software/WGPU视觉语义不变；current-source Cargo通过。

## 禁止临时方案

- 不得用无界全局String intern或无失效style cache换取0 allocation。
- 不得恢复每property Weak upgrade，或在paint阶段重新从String猜semantic role。
- 不得只优化button而让theme snapshot继续按helper读取全局锁并宣称完成。

## 修复结果与回传

Open state: `等待EditorUI04回传generation-owned resolved style/theme snapshot、规模counter、current-source Cargo与视觉等价证据`。

## 2026-09-19 rolling repair successor

- Successor Session `failure-roll-01a084c8-editorui04-button-style-r1` owns the exact plan,
  failure record, runtime helper, and focused test scope. Archived EditorUI04/EditorLayout15
  attributions were transferred after preview fingerprint
  `fe6d4a4af92b74a173e8ab0db909d5711bd7d26b9da571fd15a6ab1c9b76492d` and apply request
  `431ebba87f4c4ab29237e506f7c394f6`; lease claim and baseline attribution were accepted for all
  four paths.
- Current source remains clean and already contains the narrow single-map resolver repair: the
  direct `resolve_button_style_from_values` path borrows the input map, avoids `values.clone()`,
  does not construct `Arc::downgrade`, and the exact focused test guards the retired
  `to_ascii_lowercase()` path. Current source hashes are `style.rs`
  `e32462027b09c58ec57a6cb93905113347f2506bfe7e0afb109c6c8a0a4be3c3` and focused test
  `3aeeb7be56cb7f751a389d5582ee6d0c97573742807c5d0ae8188bd4d29a6083`.
- This successor will first submit a static source-contract ticket for the narrow repair while
  preserving the unresolved higher-level requirement: generation-owned resolved theme/style cache,
  1/100/10k allocation/counter evidence, runtime/editor parity, and current-source Cargo remain
  required before any fixed return or closeout.
- The first submission (`failure-roll-01a084c8-editorui04-button-style-20260919-r1`) was rejected
  before execution with `validation_ticket_source_snapshot_stale` because its failure-doc hash was
  captured before the final successor note. No validator process started; the stale hash is not
  reused.
- Corrected static ticket `fccec502f26744b3b8a8190e66ec013c` (request
  `failure-roll-01a084c8-editorui04-button-style-20260919-r2`) source-sealed the current failure
  doc, runtime helper, and focused test. Managed copy `9b05613af5964d55a40f2efd650a1413` executed
  the Windows Rust 1.94.1 parse contract and completed `passed`; terminal output was
  `EDITORUI04_BUTTON_STYLE_SINGLE_MAP_SOURCE_CONTRACT_PARSE_PASS`. This is static source evidence
  only and does not satisfy Cargo, generation-cache, scale-counter, visual-parity, review, or
  fixed-return requirements.

## 2026-09-20 independent review

- Reviewer Session `review-editorui04-button-style-r1` inspected the source-sealed plan, failure
  record, `zircon_runtime/src/ui/style.rs`, and
  `zircon_runtime/src/ui/tests/material_button_style.rs` at the recorded hashes
  (`style.rs` `e32462027b09c58ec57a6cb93905113347f2506bfe7e0afb109c6c8a0a4be3c3`, focused test
  `3aeeb7be56cb7f751a389d5582ee6d0c97573742807c5d0ae8188bd4d29a6083`). The reviewer held the
  failure-document lease while checking the exact owned scope.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. The direct
  `resolve_button_style_from_values` path borrows one `BTreeMap` and materializes the resolved
  DTO from borrowed lookups; it does not clone `values` or construct `Arc::downgrade`. The parser
  coverage retains canonical aliases, custom role/hex handling, and the disabled/loading/pressed/
  dragging/focused/hovered priority cases. The focused guard rejects the retired
  `to_ascii_lowercase()` path and asserts the borrowed string/parser contract.
- Review boundary is explicit: the generic `StyleProperty::extract_values` compatibility entry
  still clones a map into `UiV2ResolvedStyle`, and `ButtonStyleFields::resolve` still follows the
  existing `Weak` cascade. Those paths, generation-owned theme/style caching, 1/100/10k allocation
  counters, runtime/editor parity, visual checks, and current-source Cargo gates remain outside
  this narrow repair and are not claimed as complete.
- Scoped hygiene checks recorded `git diff --check` passed. `rustfmt --check --edition 2024
  --config skip_children=true` reported the pre-existing import-order drift in the focused test
  only; no source was edited by the review. The managed static ticket above remains the only
  dynamic evidence and is not a substitute for the pending Cargo/performance/product gates.

### 2026-09-25 current-source rolling reconciliation (EditorUI04 owner)

- Session `failure-roll-01a084c8-editorui04-button-style-r2` owns this refresh. Snapshot `3817` seals the two owned paths:
  - `zircon_runtime/src/ui/style.rs` — `e32462027b09c58ec57a6cb93905113347f2506bfe7e0afb109c6c8a0a4be3c3`
  - `zircon_runtime/src/ui/tests/material_button_style.rs` — `3aeeb7be56cb7f751a389d5582ee6d0c97573742807c5d0ae8188bd4d29a6083`
- `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs` is already dirty foreign work and is outside this narrow runtime-owned scope; no source edits were made or absorbed. Scoped `git diff --check` passes (only normal LF→CRLF notices). `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check` passes `style.rs`; the focused test retains the documented pre-existing import-order drift and is recorded as non-passing.
- The exact current source probe `EDITORUI04_BUTTON_STYLE_SINGLE_MAP_CURRENT_SOURCE_PASS` confirms the direct `resolve_button_style_from_values` region borrows the input map without `values.clone()` or `Arc::downgrade`, while the focused test retains `material_button_single_map_resolution_borrows_style_values` and canonical alias/state coverage. This is static source evidence only.
- Generation-owned resolved theme/style caching, 1/100/10k allocation counters, runtime/editor parity, visual checks, managed Windows Cargo, fixed return, closeout, and WeCom remain pending; the failure stays `open`/`resolving_failure`.

### 2026-09-25 independent static review receipt

- Reviewer `review_editor03_gizmo_private` checked snapshot `3817`, both owned hashes, and the foreign style-selector path. Result: Critical/Important/Moderate = `0/0/0`.
- The review confirms the borrowed single-map resolver, focused ownership guard, alias/custom/state coverage, and the documented focused-test formatting drift; no Cargo or generation/scale/visual acceptance was inferred.
