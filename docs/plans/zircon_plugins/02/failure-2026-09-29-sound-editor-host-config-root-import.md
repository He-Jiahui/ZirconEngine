---
handoff_kind: failure
status: open
created_at: 2026-09-29
summary_slug: sound-editor-host-config-root-import
origin_plan: docs/plans/zircon_plugins/10-editor-integration.md
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_plugins/10
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/editor/src/plugin.rs
tests:
  - cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_editor --all-targets --locked
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_editor --lib --locked -- --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked
  - cargo +1.94.1 test -p zircon_editor --test integration_contracts --features integration-contracts --locked
---

# Sound Editor host config constant import

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/10-editor-integration.md`
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 来源执行切片：Sound Editor `editor_host_contract_marker()` 的现行 host 路径迁移。
- 交接原因：Plugins10 生产调用链中的 Sound Editor 消费者保留已退休的 crate 根路径，由 Sound02 持有者迁移到现行共享 host 接口。
- 稳定 Session：`failure-roll-01a0df1a-plugin-sound-host-config-r1`。
- 日期沿用 2026-09-29 的已注册未来路径；本记录首次写入时间为 2026-10-01T04:49:26.943216+00:00。

## 失败现象与复现证据

At main `bc02eefafead65dbf5050482110e8175250a5e77`, the production helper in `zircon_plugins/sound/editor/src/plugin.rs` still referred to
`zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY`. The current Editor
crate root does not export that name. Its public producer is
`zircon_editor/src/ui/host/editor_subsystems.rs`, exported by
`zircon_editor::ui::host`; Net, Animation, Navigation and App consumers already
use that host path. This is a source-level unresolved-name finding, with no
Cargo failure or dynamic pass claimed for this lifecycle.

The frozen source-comment postimage is 4,189 bytes at SHA-256 `6b0a45cb6437c1ac56ea9abfec8cd541f5c41ddd41b0f32eb99f0849d7bfae47`.
It exactly matches the preserved attribution object at epoch 628. Original
comment bytes and line endings are retained in
`.codex/tmp/failure-roll-20261001-sound-host-config-frozen-comment-postimage.rs`.

## 最低共享层根因

The Sound consumer retained a retired crate-root path. The shared host
producer is already public and needs no change. Migrate this consumer to
`zircon_editor::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY`, preserving
the helper signature and value, registration behavior and every comment byte.

## 架构修复验收

- Managed Windows locked package check must compile the production helper and
  every declared target.
- The unfiltered Sound Editor lib batch must execute the three existing tests:
  `sound_editor_plugin_contributes_authoring_extensions`,
  `sound_editor_ui_template_routes_are_registered_operations`, and
  `sound_editor_ui_template_asset_ids_match_registered_surfaces`, with zero
  failures. Compile-only or zero-test results do not satisfy this gate.
- Plugins10 sections 7 and 9 require Editor lib and `integration_contracts`
  acceptance on the same exact managed source snapshot. Preserve separately
  owned dependency failures and link their handoffs when they block admission
  or execution.
- Preserve source-comment evidence and current source attribution, and obtain
  independent Critical/Important/Moderate zero review before return/closeout.

## 禁止临时方案

- No crate-root re-export, alias, copied constant, cfg bypass, test skip or
  assertion removal.
- Do not promote historical comment-review or static evidence to Cargo
  acceptance, and do not relabel the comment audit's original contribution.

## 修复结果与回传

Open state: `source_path_migrated_managed_compile_and_tests_pending`.
The old comment Session was already cancelled by the user; this repair did
not change its status or scope. Fresh supported exact transfer preview request
`3d15b8f9c92a41deaffa62c8a602e99e` and apply request `f61767f4552c49faabab5b860275d6e5`
move only this frozen source to the original Sound fixing Session. The
previous active-owner refusal remains historical and was not replayed.

The sole business edit qualifies the constant through `ui::host`. Current
source SHA-256 is `81c5c0d3156860c084724bfcaf575c6b51935ff5ad07be1629d5b3e39cd34520`. Replacing that exact token back restores every
original byte, including all comments and line endings. The failure remains
open pending normal managed lower and upward validation, canonical `fixed-*`
return, independent review and coordinator closeout. No commit or WeCom result
is claimed. Compiler products and caches must physically stay under a
coordinator-approved `D:/cargo-targets`, `E:/cargo-targets` or `F:/cargo-targets`
drive root.
