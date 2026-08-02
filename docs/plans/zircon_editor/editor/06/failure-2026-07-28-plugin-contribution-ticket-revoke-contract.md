---
handoff_kind: failure
status: open
created_at: 2026-07-28
summary_slug: plugin-contribution-ticket-revoke-contract
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/catalog_snapshot.rs
  - zircon_editor/src/core/plugin/extension_materialization.rs
tests:
  - zircon_editor ContributionStore plugin contribute/revoke and changed_since matrix
  - cargo test -p zircon_editor --lib --locked
  - Editor12 PostWorkbench enable-disable upstream lifecycle matrix
---

# Editor06: plugin contribution ticket and revoke contract

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：M3 management panel and PostWorkbench hot enable-disable.
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：跨插件贡献的 ticket、revoke、merged index 与 generation delta 是 Editor06
  `ContributionStore` 的唯一职责。Editor12 只拥有插件生命周期编排和 generation-bound panel reads。

## 失败现象与复现证据

`EditorPluginManager::set_enabled` only publishes a new manager-state snapshot. Its replacement
snapshot retains `Arc::clone(snapshot_slot.catalog_snapshot())`, while
`EditorPluginCatalogSnapshot` already owns one materialized
`Arc<EditorExtensionCatalogReport>` built from every registration. A transition from `Active`
through `Revoking` to `Disabled` therefore changes the panel state but cannot remove the
disabled plugin's views, drawers, menus, templates, importers, or command contributions.

The lowest extension layer is also not capable of that removal: `core/extension/` does not
exist, and `core/editor_extension.rs` stores per-kind `BTreeMap` tables behind add-only
`register_*` APIs. It exposes no `ContributionTicket`, `ContributionSource`, `revoke`, or
`changed_since` contract. `extension_materialization.rs` consequently rebuilds a single registry
from all catalog registrations with no owner ticket to withdraw.

## 最低共享层根因

Editor06 M1's planned `ContributionStore` has not replaced the add-only
`EditorExtensionRegistry`. Without a contribution owner/ticket and an atomically published
generation delta, Editor12 cannot implement real PostWorkbench contribution/revoke semantics.
Filtering rows in the plugin panel would leave the live workbench registry, command routing, and
template hosts active, so it is not an architectural repair.

## 架构修复验收

- Editor06 hard-cuts the extension owner to `core/extension/` and implements one
  `ContributionStore` with typed `ContributionSource::{Builtin, Plugin(..)}`, opaque ticket,
  collision validation, atomic `contribute`/`revoke`, and `changed_since` generation deltas.
- A successful plugin revoke removes every contribution family from the merged read model while
  old readers retain their immutable prior generation; failed batches publish neither partial
  data nor a ticket.
- Focused Editor06 tests cover plugin namespace validation, view/menu/template/command cleanup,
  generation delta ordering, capability filtering, duplicate ids, and old-generation readers.
- After the returned contract, Editor12 reruns the PostWorkbench enable-disable lifecycle matrix:
  the manager must request exactly one owner-scoped contribution change, the panel must observe
  the new shared generation, and no disabled contribution remains routable.

## 禁止临时方案

- Do not make Editor12, a panel, or a retained host hide disabled rows while retaining live
  contributions, commands, templates, or callbacks.
- Do not add an Editor12-local contribution cache, per-plugin registry clone, rebuild-on-read
  filter, compatibility alias, or test-only revoke path.
- Do not weaken collision, old-generation, or cross-family cleanup assertions.

## 修复结果与回传

Open state: `Editor06 ContributionStore 已具备 ticket/revoke/delta contract，待当前源码受管验收；
Editor12 仍须完成 catalog materializer 的权威 Store hard-cut 与上游 lifecycle matrix`。Editor12 may
continue slices independent of real contribution revocation, but it must not claim PostWorkbench hot
disable or this M3 gate passed.

## 产出记录与时间

- 2026-08-02：状态 `open`（forward repair active）。实读 `core/extension/store/model.rs` 已确认 `ContributionStore` 对每个
  `ContributionSource` 批次原子发布 opaque ticket、immutable snapshot、`revoke` 与有序
  `changed_since` delta；`store/tests.rs` 已覆盖全族回收、旧 reader 稳定、能力门控与冲突不发布。
  但 `core/plugin/extension_materialization.rs` 仍将 catalog registration 逐项写入旧
  `EditorExtensionRegistry`，没有可撤销的 plugin ticket；该 plugin-owned 入口已纳入 Editor12 既有
  [catalog hard-cut failure](../12/failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md)
  的 revoke/compiled-extension owner 范围。当前尚无受管 terminal validation 或 Editor12
  PostWorkbench 上游矩阵，禁止 `fixed` return。
