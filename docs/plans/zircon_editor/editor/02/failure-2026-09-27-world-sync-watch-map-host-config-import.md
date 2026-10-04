---
handoff_kind: failure
status: open
created_at: 2026-09-27
summary_slug: world-sync-watch-map-host-config-import
origin_plan: docs/plans/zircon_editor/editor/13-script-compilation-management.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/zircon_editor/editor/13
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/tests/editor_world_sync_watch_map.rs
tests:
  - cargo +1.94.1 check -p zircon_editor --test editor_world_sync_watch_map --locked
  - cargo +1.94.1 test -p zircon_editor --test editor_world_sync_watch_map --locked -- --test-threads=1
---

# Editor02: world-sync integration test host config import

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/13-script-compilation-management.md`
- 来源执行切片：已有 [Editor13 facade closure failure](../13/failure-2026-07-22-script-build-facade-validation-copy-closure.md) 的原始 Editor02 上行重放。
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：watch-map integration test 是 Editor02 消费者；Editor13 不应通过恢复 Editor crate root alias 来绕过现行 host 边界。

## 失败现象与复现证据

At main `bc02eefafead65dbf5050482110e8175250a5e77`, the test imported
`EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY` from the `zircon_editor` crate root.
The current `zircon_editor/src/lib.rs` has no such export; the public constant is
defined in `ui/host/editor_subsystems.rs` and exported by `ui/host/mod.rs`.
The test uses the constant in two host configuration setup calls.

This is a source-level unresolved-import finding, not a Cargo compiler receipt.
No Cargo command or test has executed for this new lifecycle. The two commands
above are the required current-source check and original full integration replay;
the latter must actually execute all seven declared tests with zero failures.

Original test: 13,147 bytes, SHA256 `9c58dc1881e4bb673caf117ace3b4c83aeed03ff0704ccdd569727c97ac7bd9a`.
Original bytes are retained in
`.codex/tmp/failure-roll-01a0df1a-editor02-host-config-test-before-20260927.rs`.

## 最低共享层根因

The host public boundary moved while this integration consumer retained the old
crate-root import. `zircon_editor::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY`
is the current authority. Production constant ownership and test behavior do not
need to change.

## 架构修复验收

- Import the existing constant through `zircon_editor::ui::host`.
- Preserve all seven integration tests, both configuration uses, and their assertions.
- Run the managed Windows 1.94.1 locked check and full integration target above;
  a queue receipt or static source check is not dynamic acceptance.
- Associate this result with the Editor13 original replay. Its separate facade
  closure gate, current-source attribution and managed SHA must also pass before
  that lifecycle can close.

## 禁止临时方案

- No crate-root alias/re-export, duplicated key, fallback, cfg bypass or ignored test.
- Do not delete integration assertions or use a filter that executes zero tests.
- Other plugin consumers of the retired root path belong to their own owners and
  require separate handoffs; this two-path repair does not accept those consumers.

## 修复结果与回传

Open state: `source_import_repaired_dynamic_validation_pending`.
Only the test's import groups were changed. Current test SHA256 is `616f991cf1a9ee203fbfec30d6b64cb61da34f3b6c7eecd18b7f472d9187fc0c`;
all seven test function bodies and assertions are byte-identical to the preimage.

Stable fixing Session: `failure-roll-01a0df1a-editor02-host-config-import-r1`,
base main `bc02eefafead65dbf5050482110e8175250a5e77`, baseline epoch 628. Exact predecessor ownership
was transferred from archived `failure-roll-01a084c8-editor02-world-sync-r2`
using fingerprint `c3141ec38671f026b8ca9c47f79ad21915a5b8c8a10332d5e8a54ed58241e452`.
Its queued static ticket `ccf68c21eb144bf591406a82b53579b3` retains its original
owner and original input; it does not cover the changed import.

Current managed Cargo admission is held for the existing external zr_vm capture
and network prerequisites. No duplicate validation request, acceptance, canonical
return, closeout, commit or WeCom result is claimed. All compiler products and
caches must physically stay below the coordinator-selected drive-root
`D:/cargo-targets`, `E:/cargo-targets` or `F:/cargo-targets`.
