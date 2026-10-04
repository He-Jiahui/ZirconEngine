---
handoff_kind: failure
status: open
created_at: 2026-09-27
summary_slug: ai-editor-host-config-root-import
origin_plan: docs/plans/zircon_plugins/10-editor-integration.md
fixing_plan: docs/plans/zircon_plugins/06-ai.md
origin_child_dir: docs/plans/zircon_plugins/10
fixing_child_dir: docs/plans/zircon_plugins/06
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/ai/editor/src/plugin.rs
tests:
  - cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_editor --all-targets --locked
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_editor --lib --locked -- --test-threads=1
---

# ai: editor host config consumer import

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/10-editor-integration.md`
- 来源执行切片：全仓 failure 滚动诊断发现的 first-party Editor helper 编译边界。
- 修复责任计划：`docs/plans/zircon_plugins/06-ai.md`
- 交接原因：该产品 Editor package 持有 `editor_host_contract_marker()`；共享 Editor host 常量已公开在规范 host 模块，消费者应迁移到该边界。

## 失败现象与复现证据

At main `bc02eefafead65dbf5050482110e8175250a5e77`, `zircon_plugins/ai/editor/src/plugin.rs` used
`zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY` in its production
`editor_host_contract_marker()` helper. Current `zircon_editor/src/lib.rs` does
not export that root symbol. `ui/host/editor_subsystems.rs` defines the public
constant and `ui/host/mod.rs` exports it through the current host boundary.
Navigation's Editor helper and App's engine entry already use that public path.

This is a source-level unresolved-name finding. No Cargo compiler or runtime
result has been produced for this lifecycle. The managed package check above
must compile the production helper and all declared targets, and the unfiltered
lib test command must execute the existing 14 non-ignored tests with zero failures
(9 root tests and 5 directly mounted child tests). Four preexisting release-only
performance tests remain ignored by this default batch and retain their separate
owner gates; no performance pass is claimed.
No tests or assertions are added, deleted, ignored or filtered by this repair.

Original source SHA256: `834e844e35f52b836b1810cfc2225382d3539e01f133a124ecfc6cd13b129751`.
Original bytes are retained in
`.codex/tmp/failure-roll-01a0df1a-plugin-ai-host-config-before-20260927.rs`.

## 最低共享层根因

The consumer retained a retired crate-root name after the host boundary moved.
The existing `zircon_editor::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY` is
the authority; this product consumer needs one qualified-path migration.

## 架构修复验收

- Resolve the existing host constant through its current public module.
- Keep the helper signature and returned constant value, plugin registration,
  package manifest and all existing test behavior unchanged.
- Execute the managed Windows 1.94.1 locked package check and all 14 non-ignored
  lib tests; preserve the 4 release-only ignored tests and their separate owner gates.
- Return the exact accepted product-owner snapshot to Plugins10's integration
  chain. Other affected product owners retain separate validation and closeout.

## 禁止临时方案

- No Editor root re-export, alias, copied key, fallback, cfg bypass or test skip.
- Do not absorb foreign plugin/source-comment edits or combine cross-plan commits.
- Static source, a queued receipt or a zero-test filter is not dynamic acceptance.

## 修复结果与回传

Open state: `source_path_migrated_managed_compile_and_tests_pending`.
Only the qualified path in `editor_host_contract_marker()` changed. Current
source SHA256 is `898619350b4803ccc24748e8be4ba3aecc58d8b747713309080d550db3e4e138`; all other source bytes and all test files are
unchanged. Stable fixing Session: `failure-roll-01a0df1a-plugin-ai-host-config-r1`, main `bc02eefafead65dbf5050482110e8175250a5e77`, epoch 628.

Current Cargo admission is held for the existing external zr_vm capture/network
prerequisites. No new validation ticket, Cargo pass, canonical return, closeout,
commit or WeCom result is claimed. Compiler products and caches must physically
stay below the coordinator-selected `D:/cargo-targets`, `E:/cargo-targets` or
`F:/cargo-targets` drive root.
