---
handoff_kind: fixed
status: fixed
created_at: 2026-07-22
summary_slug: focus-fixture-typed-id-hardcut-drift
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_editor/editor_layout/19
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/focus/focus_tests.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime_interface --lib ui::focus::focus_tests:: --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime_interface ui_render_command_cache_generation --locked --jobs 1 -- --nocapture --test-threads=1
resolved_at: 2026-07-22
---


# Editor Layout19: focus fixture typed-id hard-cut drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行切片：Runtime09 generation streaming source-bound focused gate，snapshot 940 exact3
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`
- 交接原因：`focus_tests.rs` 是 Layout19 S1 的 focus mode、Tab chain、navigation boundary 与 focus-visible 契约夹具；其 typed-id/API 推断漂移阻断 interface lib-test 编译，不属于 Runtime09 generation 实现。

## 失败现象与复现证据

受管 reservation `dd66c94d2e5949adba4443170be71b30` 绑定 job `5f20293b706949e492df72852c51c725` / run `fb0328e4f22c45d98361d20e9acc4f11`，执行：

```text
cargo +1.94.1 test -p zircon_runtime_interface ui_render_command_cache_generation --locked --jobs 1 -- --nocapture --test-threads=1
```

作业 released，exit 101，live process 为空；目标测试执行 0 个。rustc 在 `focus_tests.rs:12` 报 `UiNodePath: From<String>` 不成立，在 `:24` 报 `UiTreeId: From<&str>` 不成立，并在 `:73` 报 E0283，`serde_json::from_value` 缺失结果类型；共 3 条编译错误。

## 最低共享层根因

Layout19 S1 新夹具仍依赖已硬切删除的字符串隐式转换，并让 navigation boundary roundtrip 依赖不稳定的后置推断。现行 owner 要求 `UiNodePath::new`、`UiTreeId::new` 与显式 `UiNavigationBoundary` 结果类型。

## 架构修复验收

- Layout19 夹具使用 typed-id 构造器和显式 navigation boundary 类型，不恢复字符串 `From`。
- focused `ui::focus::focus_tests::` lib-test 编译并实际执行 3 个 Layout19 测试通过。
- 原 Runtime09 generation reproduction 越过该夹具编译边界并执行目标测试。

## 禁止临时方案

- 不得恢复 `UiNodePath`/`UiTreeId` 字符串 `From`、alias 或兼容 shim。
- 不得删除、cfg-gate、跳过 Layout19 夹具或减弱 focus/navigation 契约。
- 不得把 Runtime09 filter 的 0-test/compile failure 声称为 generation red/green。

## 修复结果与回传

- 根因：Layout19 focus fixture retained removed string-to-typed-id conversions and an unconstrained navigation-boundary deserialize after the typed-id hard cut.
- 架构修复：Use UiNodePath::new and UiTreeId::new at the lowest fixture calls and bind the round-trip result to UiNavigationBoundary without restoring implicit conversions.
- 验证：Focused job fa5a2a52c7024673b8d3f213a6ab0597/run 6e3f5675a6e648778f974960fe8df4e4: 3 passed; upward Runtime09 job d1f58edc59e84a08bd38b30ace51149f/run 07349ea9c4aa4699a5871a580cbe4bae: 2 generation tests passed.
- 回传：Layout19 typed focus fixture is source-bound green at the lowest owner and the original Runtime09 reproduction now executes and passes.
