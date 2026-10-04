---
handoff_kind: failure
status: open
created_at: 2026-09-28
summary_slug: net-editor-host-config-root-import
origin_plan: docs/plans/zircon_plugins/10-editor-integration.md
fixing_plan: docs/plans/zircon_plugins/07-net.md
origin_child_dir: docs/plans/zircon_plugins/10
fixing_child_dir: docs/plans/zircon_plugins/07
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/net/editor/src/plugin.rs
tests:
  - cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_editor --all-targets --locked
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_editor --lib --locked -- --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked
  - cargo +1.94.1 test -p zircon_editor --test integration_contracts --features integration-contracts --locked
---

# Net Editor host config constant import

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/10-editor-integration.md`
- 来源执行切片：全仓 failure 滚动诊断发现的 first-party Net Editor helper integration gate.
- 修复责任计划：`docs/plans/zircon_plugins/07-net.md`
- 交接原因：该产品 Editor package 持有 `editor_host_contract_marker()`；共享 Editor host 常量已公开在规范 host 模块，消费者应迁移到该边界。

## 失败现象与复现证据

At main `bc02eefafead65dbf5050482110e8175250a5e77`, the production
`editor_host_contract_marker()` in `zircon_plugins/net/editor/src/plugin.rs`
referred to `zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY`. The current
`zircon_editor/src/lib.rs` has no such crate-root export. The constant is
defined in `zircon_editor/src/ui/host/editor_subsystems.rs` and publicly
exported by `zircon_editor::ui::host`. Navigation's Editor consumer and App's
engine entry already use that qualified host path.

This is a source-level unresolved-name finding. No Cargo compiler result is
claimed for this lifecycle. The original, completed source-comment postimage
was 2,506 bytes at SHA-256
`589607b1a8b42e295555e03b5c08ae3bc9ebc6ae7d1d65b2d334661c0d0622c3`.
The comment audit's frozen boundary and backup preserve its original bytes.

## 最低共享层根因

The Net Editor consumer retained the retired crate-root name after the Editor
host contract moved. The existing `zircon_editor::ui::host` export is the
authority; no shared constant or Editor API change is needed.

## 架构修复验收

- Use the current public host module in this Net Editor consumer, preserving
  the helper signature and value, registration behavior, and all comment bytes.
- The managed Windows locked package check must compile the production helper
  and every declared target. The unfiltered lib test must execute the existing
  Net Editor authoring-extension test with zero failures.
- Plugins10 sections 7 and 9 also require the full Editor lib and
  `integration_contracts` gates. Run both on the same managed source snapshot
  before returning this package result to the origin plan. A failure owned by
  another package remains separately attributed and reported.

## 禁止临时方案

- No crate-root re-export, copied constant, alias, fallback, cfg bypass, test
  skip, or assertion removal.
- Do not absorb or relabel the source-comment audit's edits or receipts.
- A source-only check or queued validation receipt is not dynamic acceptance.

## 修复结果与回传

Open state: `source_path_migrated_managed_compile_and_tests_pending`.
Session `failure-roll-01a0df1a-plugin-net-host-config-r1` claimed and
attributed the exact frozen source after the source-comment owner released its
lease. The only business edit replaced the retired qualified path. Current
source SHA-256 is
`8b143a42e696c6aef56703795edaa61ad327959e76bcff257c086fe79ad900b8`;
all other source and comment bytes remain unchanged. The previous comment
audit receipts are historical and cannot validate this business postimage.

Coordinator snapshot `5339` sealed the source and this open record before
import. Failure import request `4b2275032fcb41a9a5c542c6f04d86c7` indexed
this lifecycle as node `3848151`. The current structural handoff validator
reported `838 handoff artifact(s): 0 errors`; source `rustfmt --check` and
scoped `git diff --check` passed. These are preparatory checks only.

Independent pre-review found one Moderate acceptance-manifest omission: the
origin Plugins10 Editor lib and integration contract gates were absent from
the initial `tests` field. Both commands are now declared; the repaired source
bytes and lower package commands remain unchanged. No dynamic pass is claimed.

Managed Cargo admission still requires the external `E:/Git/zr_vm` capture and
network prerequisites. No managed Cargo pass, canonical `fixed-*` return,
closeout commit, or WeCom notification is claimed. Compiler products and caches
must stay physically under the coordinator-selected `D:/cargo-targets`,
`E:/cargo-targets`, or `F:/cargo-targets` drive root.
