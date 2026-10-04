---
handoff_kind: failure
status: open
created_at: 2026-09-28
summary_slug: animation-editor-host-config-root-import
origin_plan: docs/plans/zircon_plugins/10-editor-integration.md
fixing_plan: docs/plans/zircon_plugins/04-animation.md
origin_child_dir: docs/plans/zircon_plugins/10
fixing_child_dir: docs/plans/zircon_plugins/04
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/animation/editor/src/plugin.rs
tests:
  - cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_editor --all-targets --locked
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_editor --lib --locked -- --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked
  - cargo +1.94.1 test -p zircon_editor --test integration_contracts --features integration-contracts --locked
---

# Animation Editor host config constant import

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/10-editor-integration.md`
- 来源执行切片：First-party Animation Editor host contract integration compile and registration gate.
- 修复责任计划：`docs/plans/zircon_plugins/04-animation.md`
- 交接原因：The Animation Editor package owns its production `editor_host_contract_marker()` consumer; the Editor host export is already correct.

## 失败现象与复现证据

At main `bc02eefafead65dbf5050482110e8175250a5e77`, the production
`editor_host_contract_marker()` in `zircon_plugins/animation/editor/src/plugin.rs`
referred to `zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY`. The current
`zircon_editor/src/lib.rs` has no crate-root export for that name. The constant
is defined in `zircon_editor/src/ui/host/editor_subsystems.rs` and publicly
exported through `zircon_editor::ui::host`. Navigation's Editor consumer and
App's engine entry already use the qualified host path.

This is a source-level unresolved-name finding. No Cargo compiler result is
claimed for this lifecycle. The source-comment audit completed its frozen
postimage at 3,845 bytes, SHA-256
`429658a743466fb54a0a87c0fb0c71ed870118ce284aa2d5cc15535fbfe8df1f`.
Its ownership boundary and backup preserve those comment bytes.

## 最低共享层根因

The Animation Editor consumer retained the retired crate-root name after the
Editor host contract moved. The existing public host module is the authority;
no Editor API or shared constant change is needed.

## 架构修复验收

- Use the public `zircon_editor::ui::host` path in this package, preserving
  the helper signature, value, registration, and all comment bytes.
- The managed Windows locked package check must compile the production helper
  and all declared targets. The unfiltered lib test must execute the existing
  two Animation Editor tests with zero failures.
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
Session `failure-roll-01a0df1a-plugin-animation-host-config-r1` claimed and
attributed the exact frozen source after the source-comment owner released its
lease. The only business edit replaced the retired qualified path. Current
source SHA-256 is
`acd2b045c3e4fd0b485ef18741cf69889cda0aea60303329f58fadd6d4d286f4`;
all other source and comment bytes remain unchanged. The previous comment
audit receipts are historical and cannot validate this business postimage.

Coordinator snapshot `5347` sealed the source and this open record before
import. Failure import request `a4b2fab380594e1abcc6e3225a140ef8` indexed
this lifecycle as node `3849818`. The current structural handoff validator
reported `839 handoff artifact(s): 0 errors`; source `rustfmt --check` and
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
