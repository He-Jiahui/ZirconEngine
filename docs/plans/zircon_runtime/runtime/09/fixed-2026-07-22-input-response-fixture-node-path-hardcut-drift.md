---
handoff_kind: fixed
status: fixed
created_at: 2026-07-22
summary_slug: input-response-fixture-node-path-hardcut-drift
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_editor/editor_layout/18
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/tests/input_response_contracts.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime_interface --lib input_response --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime_interface ui_render_command_cache_generation --locked --jobs 1 -- --nocapture --test-threads=1
resolved_at: 2026-07-22
---


# Editor Layout18: input-response fixture node-path hard-cut drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行切片：Runtime09 generation streaming source-bound focused gate，snapshot 940 exact3
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`
- 交接原因：`input_response_contracts.rs` 是 Layout18 S1 的 pointer-events、hit-path 与 dispatch-phase 契约夹具；其过时 node-path 构造阻断了整个 `zircon_runtime_interface` lib-test 编译，不属于 Runtime09 generation 实现。

## 失败现象与复现证据

受管 reservation `dd66c94d2e5949adba4443170be71b30` 绑定 job `5f20293b706949e492df72852c51c725` / run `fb0328e4f22c45d98361d20e9acc4f11`，执行：

```text
cargo +1.94.1 test -p zircon_runtime_interface ui_render_command_cache_generation --locked --jobs 1 -- --nocapture --test-threads=1
```

作业 released，exit 101，live process 为空；目标测试执行 0 个。rustc 在 `input_response_contracts.rs:49/50/67` 报 3 条 E0277，因为 `UiNodePath` 已硬切删除 `From<&str>`，夹具仍用字符串 `.into()`。

## 最低共享层根因

Layout18 S1 新夹具没有跟随 `UiNodePath::new(...)` 强类型构造边界。生产 API 已完成硬切，错误只在 Layout18 夹具调用点；恢复 `From` 会重新扩大隐式转换面，不是允许的修复。

## 架构修复验收

- Layout18 夹具全部使用 `UiNodePath::new(...)`，不恢复 `From<&str>` 或兼容 helper。
- focused `input_response` lib-test 编译并执行通过。
- 原 Runtime09 generation reproduction 越过该夹具编译边界并执行目标测试。

## 禁止临时方案

- 不得恢复字符串到 `UiNodePath` 的隐式 `From`、alias 或兼容 shim。
- 不得删除、cfg-gate、跳过 Layout18 夹具或减弱契约断言。
- 不得把 Runtime09 filter 的 0-test/compile failure 声称为 generation red/green。

## 修复结果与回传

- 根因：Layout18 input-response fixture retained removed string-to-UiNodePath implicit conversions after the typed-id hard cut.
- 架构修复：Update the three lowest fixture call sites to UiNodePath::new without restoring From implementations, aliases, or compatibility helpers.
- 验证：Focused job 1f7474ecc92e4617a36afaafc971541f/run a76484580de24b58b67e01428d655260: 7 passed; upward Runtime09 job d1f58edc59e84a08bd38b30ace51149f/run 07349ea9c4aa4699a5871a580cbe4bae: 2 generation tests passed.
- 回传：Layout18 typed node-path fixture is source-bound green at the lowest owner and the original Runtime09 reproduction now executes and passes.
