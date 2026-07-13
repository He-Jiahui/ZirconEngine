---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: native-plugin-runtime-target-mode-test-path
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/12
related_code:
  - zircon_editor/src/tests/host/manager/minimal_host_contract/native_plugins.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
tests:
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1
resolved_at: 2026-07-13
---


# Editor 12：native plugin 测试仍引用退役 RuntimeTargetMode 路径

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1.4 source authority 与只读 command when/dispatch guard 的 Windows 编译门禁
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：失败全部位于 Editor12 native plugin minimal-host 合同测试；目标模式类型已硬切到 Runtime framework/platform owner，Editor09 不应恢复 `zircon_runtime::builtin` 兼容导出。

## 失败现象与复现证据

Text01 下层 E0061 修复后，Windows 受管门禁
`cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 已编译到 `zircon_editor`，随后在
`zircon_editor/src/tests/host/manager/minimal_host_contract/native_plugins.rs` 报 11 个 E0433：

- 行 185、302、307、323、393、394、456、464、537、618、626 仍引用
  `zircon_runtime::builtin::RuntimeTargetMode`；
- 当前 canonical owner 是
  `zircon_runtime::core::framework::platform::RuntimeTargetMode`；
- rustc 明确建议导入 canonical enum。Editor09 新增的 source authority、registry policy 与 dispatch guard 没有产生编译诊断。

完整日志：`.codex/tmp/editor09-m1-4-source-authority-compile-r2-20260713.log`。

## 最低共享层根因

RuntimeTargetMode 的 workspace hard cut 已删除 builtin 路径，但 Editor12 native-plugin 测试调用方未迁移。最低修复是测试使用 canonical platform 类型，禁止在 Runtime builtin 重新导出旧路径。

## 架构修复验收

- `native_plugins.rs` 只从 canonical `core::framework::platform` 路径导入一次 `RuntimeTargetMode`，不保留全限定旧路径。
- native plugin minimal-host focused tests 通过，并继续覆盖 ClientRuntime / ServerRuntime / EditorHost 三种模式。
- 原始 `cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 越过这 11 个 E0433，Editor09 M1 恢复向上验证。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 `zircon_runtime::builtin` 恢复 `RuntimeTargetMode` re-export，禁止跳过 native plugin tests。

## 修复结果与回传

- 根因：Editor12 native plugin tests retained the retired zircon_runtime::builtin RuntimeTargetMode path after the hard cut.
- 架构修复：The test imports the canonical zircon_runtime::core::framework::platform::RuntimeTargetMode path; no builtin re-export was restored.
- 验证：cargo test -p zircon_editor --lib --no-run --locked --jobs 1 reached successful test-binary generation; artifact .codex/tmp/zircon_editor-editor09-m1-4-source-authority-r4-20260713.exe.
- 回传：Editor09 r4 no-run gate compiled past all 11 former E0433 diagnostics and produced the current lib-test binary.
