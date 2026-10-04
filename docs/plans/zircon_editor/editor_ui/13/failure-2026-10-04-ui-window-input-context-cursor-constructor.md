---
handoff_kind: failure
status: open
created_at: 2026-10-04
summary_slug: ui-window-input-context-cursor-constructor
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_editor/editor_ui/13-penpot-zui-native-parity-and-ue-gap-audit.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/13
plan_link_mode: child_record_only
priority: 0
related_code:
  - zircon_runtime_interface/src/ui/window/input/context.rs
  - zircon_runtime_interface/src/ui/window/input/window_event.rs
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
tests:
  - .\tools\dev\local-cargo.ps1 +1.94.1 test -p zircon_runtime_interface --locked --no-default-features --lib --jobs 1 tests::window_input_contracts:: -- --test-threads=1
  - .\tools\dev\local-cargo.ps1 +1.94.1 test -p zircon_runtime_interface --locked --no-default-features --lib --jobs 1 tests::window_runtime_event_adapter_contracts:: -- --test-threads=1
  - .\tools\dev\local-cargo.ps1 +1.94.1 check -p zircon_plugin_sdk --locked --no-default-features --features native --jobs 1 --all-targets
---

# EditorUI13: missing cursor field in the shared input-context constructor

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`。
- 来源执行切片：Root rolling Failure repair; rebinding the SDK owned-buffer and ordinary entry-unwind lower regressions to available Windows evidence.
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/13-penpot-zui-native-parity-and-ue-gap-audit.md`。
- 交接原因：EditorUI13 M0 I4 adds the shared cursor-history field and builder. The explicit shared constructor was not updated, so an unrelated SDK consumer cannot compile.

## 失败现象与复现证据

The actual October 4 Windows command was:

```text
python -u -B -S -X utf8 -m tools.dev.local_cargo --repo-root E:/Git/ZirconEngine --target-dir D:/cargo-targets/zircon-local/windows/failure-roll-sdk-native-20261004-r1/target -- +1.94.1 check -p zircon_plugin_sdk --locked --no-default-features --features native --jobs 1 --all-targets
```

This is the historical, pre-directory-migration command. Do not restore that
module path. Current acceptance uses the migrated `tools/dev` entry above.

Cargo exited 101 and reported E0063 at `context.rs:46`: the `Self` initializer
in `UiWindowInputContext::from_window_metadata` lacks `last_cursor_position`.
The derived `Default` is complete. None of the planned 16 SDK tests ran.

- Original log: `E:/cargo-targets/zircon-local/failure-roll/sdk-locked-rebind-20261004-r3/sdk-check.log`, SHA-256 `4e33ae2497831a7525b14a94e7db325183de9770caa563ecb3eca10b5bcacb32`.
- Original terminal: the same directory's `terminal.json`, SHA-256 `8424fa4c5e64617bc396e0a2699469435569545712d32fbcaae8e460a0c9b5ca`.

The original batch's full input guard did not pass: two unrelated, conservatively
captured Jenkins files changed. Retain that failed receipt. The compiler error
is diagnosis evidence, not passing dynamic acceptance or a quiet-tree claim.

## 最低共享层根因

The field and `with_last_cursor_position` belong to EditorUI13's I4 change,
recorded in `m0-quick-fix-evidence.md`. Window metadata does not contain a cursor
position. Its constructor must initialize this optional history to `None`,
matching the derived default; platform owners can then supply a known position
through the existing builder.

`UiWindowEvent::input_context` and the runtime-event adapter both use this
constructor. Fix it at that shared layer. Preserve the field, builder,
serialization, identity metadata and original comments.

## 架构修复验收

- Execute the existing window-input module's eight tests and runtime-event
  adapter module's seven tests; check actual test IDs and zero ignored tests.
- Rerun the original SDK native-only `--locked --all-targets` check, then resume
  its original 10 + 3 + 2 + 1 lower regressions with matching source evidence.
- Bind the current entry, configuration, toolchain, source and physical output
  paths. A dry run is entry evidence only.
- Keep SDK real-DLL and host acceptance separate, and keep EditorUI13 M0/M1
  behavior and parity gates open; this constructor repair does not accept them.

## 禁止临时方案

Do not remove the new field, add a compatibility alias, patch only an SDK call
site, invent a cursor coordinate, weaken tests or promote static review to
dynamic acceptance. Do not restore retired coordinator APIs or the old tools
directory. Preserve unrelated migration and source-author postimages.

## 修复结果与回传

Open: source repaired; lower, original and upward dynamic gates are pending.
Root added only `last_cursor_position: None,` (40 bytes) to the constructor.

- Before SHA-256: `53bb77e3331c7f09c48a6557e5e85ce9b0fdb1fa9e423760a2c6bd7c9255335b`.
- After SHA-256: `c66cdb209fed3dda7a4d53313986ce680eac91793347c451249ceec867ab4c61`.
- Exact edit receipt: `E:/cargo-targets/zircon-local/failure-roll/ui13-input-context-field-edit-20261004-r1/terminal.json`, SHA-256 `9526a6f1b7585bb684ec854b104162ca666078dc181c5d2c7c0ea050824a6d0a`.
- Independent source-only review: Critical 0 / Important 0 / Moderate 0.

All other bytes are preserved. Root owns this insertion, without adopting the
other author's complete UI13 business postimage. No fixed return, commit,
notification or product acceptance is claimed.
